use std::io::{Cursor, Read};
use std::sync::Mutex;
use std::time::Instant;

use async_trait::async_trait;
use chrono::NaiveDate;

use crate::application::ports::assembly_source::{AssemblySource, SourceError};
use crate::domain::dossier::{Initiator, LegislativeAct, LegislativeDossier};
use crate::domain::scoring::compute_score;

use super::committees::resolve_committee;
use super::parsing::{
    collect_all_acts, extract_initiator_refs, find_committee_organe_ref, find_current_stage,
    find_latest_act, RawDossierWrapper,
};

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
        // data.assemblee-nationale.fr sometimes serves a self-signed certificate
        let http = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("failed to build HTTP client");
        Self {
            http,
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

    fn parse_raw_dossier(
        raw: &super::parsing::RawDossier,
    ) -> Option<(LegislativeDossier, Vec<String>)> {
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
        let current_stage = find_current_stage(&raw.legislative_acts);
        let committee = find_committee_organe_ref(&raw.legislative_acts)
            .and_then(|ref_id| resolve_committee(&ref_id).map(String::from));
        let initiator_refs = extract_initiator_refs(&raw.initiator);

        let is_government_bill = raw.parliamentary_procedure.libelle.starts_with("Projet de loi");

        let initiators = if initiator_refs.is_empty() && is_government_bill {
            vec![Initiator {
                full_name: "Gouvernement".to_string(),
                group: None,
            }]
        } else {
            vec![]
        };

        Some((
            LegislativeDossier {
                uid: raw.uid.clone(),
                title: raw.dossier_title.titre.clone(),
                procedure: raw.parliamentary_procedure.libelle.clone(),
                last_activity_date: date,
                last_activity_label: act_info.label,
                acts: all_acts,
                score,
                current_stage,
                initiators,
                committee,
            },
            initiator_refs,
        ))
    }

    fn parse_dossiers(
        data: &[u8],
        since: NaiveDate,
    ) -> Result<Vec<(LegislativeDossier, Vec<String>)>, SourceError> {
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

            let (dossier, refs) =
                match Self::parse_raw_dossier(&wrapper.parliamentary_dossier) {
                    Some(d) => d,
                    None => continue,
                };

            if dossier.last_activity_date < since {
                continue;
            }

            dossiers.push((dossier, refs));
        }

        Ok(dossiers)
    }

    fn find_dossier_by_uid(
        data: &[u8],
        uid: &str,
    ) -> Result<Option<(LegislativeDossier, Vec<String>)>, SourceError> {
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
        let results = self.fetch_dossiers_since_with_refs(since).await?;
        Ok(results.into_iter().map(|(d, _)| d).collect())
    }

    async fn fetch_dossiers_since_with_refs(
        &self,
        since: NaiveDate,
    ) -> Result<Vec<(LegislativeDossier, Vec<String>)>, SourceError> {
        let zip_data = self.get_zip().await?;

        let parsed =
            tokio::task::spawn_blocking(move || Self::parse_dossiers(&zip_data, since))
                .await
                .map_err(|e| SourceError::Parse(e.to_string()))??;

        tracing::info!("Found {} dossiers since {since}", parsed.len());
        Ok(parsed)
    }

    async fn fetch_dossier_by_uid(
        &self,
        uid: &str,
    ) -> Result<Option<LegislativeDossier>, SourceError> {
        let zip_data = self.get_zip().await?;
        let uid = uid.to_string();

        let result =
            tokio::task::spawn_blocking(move || Self::find_dossier_by_uid(&zip_data, &uid))
                .await
                .map_err(|e| SourceError::Parse(e.to_string()))??;

        Ok(result.map(|(d, _refs)| d))
    }

    async fn fetch_dossier_by_uid_with_refs(
        &self,
        uid: &str,
    ) -> Result<Option<(LegislativeDossier, Vec<String>)>, SourceError> {
        let zip_data = self.get_zip().await?;
        let uid = uid.to_string();

        tokio::task::spawn_blocking(move || Self::find_dossier_by_uid(&zip_data, &uid))
            .await
            .map_err(|e| SourceError::Parse(e.to_string()))?
    }
}
