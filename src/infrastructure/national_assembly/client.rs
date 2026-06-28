use std::io::{Cursor, Read};
use std::sync::Mutex;
use std::time::Instant;

use async_trait::async_trait;
use chrono::NaiveDate;

use crate::application::ports::assembly_source::{AssemblySource, SourceError};
use crate::domain::dossier::{LegislativeAct, LegislativeDossier};
use crate::domain::scoring::compute_score;

use super::parsing::{collect_all_acts, find_latest_act, RawDossierWrapper};

const DOSSIERS_URL: &str = "https://data.assemblee-nationale.fr/static/openData/repository/17/loi/dossiers_legislatifs/Dossiers_Legislatifs.json.zip";
const CACHE_TTL_SECS: u64 = 3600;

struct CachedZip {
    data: Vec<u8>,
    fetched_at: Instant,
}

pub struct NationalAssemblyClient {
    http: reqwest::Client,
    cache: Mutex<Option<CachedZip>>,
}

impl NationalAssemblyClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
            cache: Mutex::new(None),
        }
    }

    async fn get_zip(&self) -> Result<Vec<u8>, SourceError> {
        {
            let cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.as_ref() {
                if cached.fetched_at.elapsed().as_secs() < CACHE_TTL_SECS {
                    tracing::debug!("Using cached dossiers ZIP");
                    return Ok(cached.data.clone());
                }
            }
        }

        tracing::info!("Downloading {DOSSIERS_URL}");
        let response = self
            .http
            .get(DOSSIERS_URL)
            .send()
            .await
            .map_err(|e| SourceError::Download(e.to_string()))?;

        if !response.status().is_success() {
            return Err(SourceError::Download(format!(
                "HTTP {}",
                response.status()
            )));
        }

        let data = response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| SourceError::Download(e.to_string()))?;

        tracing::info!("Downloaded {} bytes", data.len());

        {
            let mut cache = self.cache.lock().unwrap();
            *cache = Some(CachedZip {
                data: data.clone(),
                fetched_at: Instant::now(),
            });
        }

        Ok(data)
    }

    fn parse_raw_dossier(raw: &super::parsing::RawDossier) -> Option<LegislativeDossier> {
        let act_info = find_latest_act(&raw.legislative_acts)?;
        let date = NaiveDate::parse_from_str(&act_info.date, "%Y-%m-%d").ok()?;

        let all_acts: Vec<LegislativeAct> = collect_all_acts(&raw.legislative_acts)
            .into_iter()
            .filter_map(|a| {
                NaiveDate::parse_from_str(&a.date, "%Y-%m-%d")
                    .ok()
                    .map(|d| LegislativeAct {
                        date: d,
                        label: a.label,
                    })
            })
            .collect();

        let score = compute_score(&raw.dossier_title.titre, &act_info.label, all_acts.len());

        Some(LegislativeDossier {
            uid: raw.uid.clone(),
            title: raw.dossier_title.titre.clone(),
            procedure: raw.parliamentary_procedure.libelle.clone(),
            last_activity_date: date,
            last_activity_label: act_info.label,
            acts: all_acts,
            score,
        })
    }

    fn parse_dossiers(
        data: &[u8],
        since: NaiveDate,
    ) -> Result<Vec<LegislativeDossier>, SourceError> {
        let cursor = Cursor::new(data);
        let mut archive =
            zip::ZipArchive::new(cursor).map_err(|e| SourceError::Parse(e.to_string()))?;

        let mut dossiers = Vec::new();

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| SourceError::Parse(e.to_string()))?;

            let name = file.name().to_string();
            if !name.contains("dossierParlementaire/") || !name.ends_with(".json") {
                continue;
            }

            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| SourceError::Parse(e.to_string()))?;

            let wrapper: RawDossierWrapper = match serde_json::from_str(&content) {
                Ok(w) => w,
                Err(_) => continue,
            };

            let dossier = match Self::parse_raw_dossier(&wrapper.parliamentary_dossier) {
                Some(d) => d,
                None => continue,
            };

            if dossier.last_activity_date < since {
                continue;
            }

            dossiers.push(dossier);
        }

        Ok(dossiers)
    }

    fn find_dossier_by_uid(
        data: &[u8],
        uid: &str,
    ) -> Result<Option<LegislativeDossier>, SourceError> {
        let cursor = Cursor::new(data);
        let mut archive =
            zip::ZipArchive::new(cursor).map_err(|e| SourceError::Parse(e.to_string()))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| SourceError::Parse(e.to_string()))?;

            let name = file.name().to_string();
            if !name.contains("dossierParlementaire/") || !name.ends_with(".json") {
                continue;
            }

            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| SourceError::Parse(e.to_string()))?;

            let wrapper: RawDossierWrapper = match serde_json::from_str(&content) {
                Ok(w) => w,
                Err(_) => continue,
            };

            if wrapper.parliamentary_dossier.uid != uid {
                continue;
            }

            return Ok(Self::parse_raw_dossier(&wrapper.parliamentary_dossier));
        }

        Ok(None)
    }
}

#[async_trait]
impl AssemblySource for NationalAssemblyClient {
    async fn fetch_dossiers_since(
        &self,
        since: NaiveDate,
    ) -> Result<Vec<LegislativeDossier>, SourceError> {
        let zip_data = self.get_zip().await?;

        let dossiers =
            tokio::task::spawn_blocking(move || Self::parse_dossiers(&zip_data, since))
                .await
                .map_err(|e| SourceError::Parse(e.to_string()))??;

        tracing::info!("Found {} dossiers since {since}", dossiers.len());
        Ok(dossiers)
    }

    async fn fetch_dossier_by_uid(
        &self,
        uid: &str,
    ) -> Result<Option<LegislativeDossier>, SourceError> {
        let zip_data = self.get_zip().await?;
        let uid = uid.to_string();

        tokio::task::spawn_blocking(move || Self::find_dossier_by_uid(&zip_data, &uid))
            .await
            .map_err(|e| SourceError::Parse(e.to_string()))?
    }
}
