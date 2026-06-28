use std::io::{Cursor, Read};
use std::sync::Mutex;
use std::time::Instant;

use async_trait::async_trait;
use chrono::NaiveDate;

use crate::application::ports::assemblee_source::{AssembleeSource, SourceError};
use crate::domain::dossier::DossierLegislatif;

use super::parsing::{find_latest_acte, RawDossierWrapper};

const DOSSIERS_URL: &str = "https://data.assemblee-nationale.fr/static/openData/repository/17/loi/dossiers_legislatifs/Dossiers_Legislatifs.json.zip";
const CACHE_TTL_SECS: u64 = 3600;

struct CachedZip {
    data: Vec<u8>,
    fetched_at: Instant,
}

pub struct AssembleeNationaleClient {
    http: reqwest::Client,
    cache: Mutex<Option<CachedZip>>,
}

impl AssembleeNationaleClient {
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

    fn parse_dossiers(data: &[u8], since: NaiveDate) -> Result<Vec<DossierLegislatif>, SourceError> {
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

            let raw = wrapper.dossier_parlementaire;

            let acte_info = match find_latest_acte(&raw.actes_legislatifs) {
                Some(info) => info,
                None => continue,
            };

            let date = match NaiveDate::parse_from_str(&acte_info.date, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => continue,
            };

            if date < since {
                continue;
            }

            dossiers.push(DossierLegislatif {
                uid: raw.uid,
                titre: raw.titre_dossier.titre,
                procedure: raw.procedure_parlementaire.libelle,
                derniere_activite_date: date,
                derniere_activite_libelle: acte_info.libelle,
            });
        }

        Ok(dossiers)
    }
}

#[async_trait]
impl AssembleeSource for AssembleeNationaleClient {
    async fn fetch_dossiers_since(
        &self,
        since: NaiveDate,
    ) -> Result<Vec<DossierLegislatif>, SourceError> {
        let zip_data = self.get_zip().await?;

        let dossiers =
            tokio::task::spawn_blocking(move || Self::parse_dossiers(&zip_data, since))
                .await
                .map_err(|e| SourceError::Parse(e.to_string()))??;

        tracing::info!("Found {} dossiers since {since}", dossiers.len());
        Ok(dossiers)
    }
}
