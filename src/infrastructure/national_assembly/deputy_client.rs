use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use async_trait::async_trait;
use serde::Deserialize;

use crate::application::ports::deputy_source::DeputySource;
use crate::domain::dossier::Initiator;

const DEPUTIES_URL: &str = "https://www.nosdeputes.fr/deputes/json";
const CACHE_TTL_SECS: u64 = 3600;

#[derive(Deserialize)]
struct DeputiesResponse {
    deputes: Vec<DeputyWrapper>,
}

#[derive(Deserialize)]
struct DeputyWrapper {
    depute: DeputyData,
}

#[derive(Deserialize)]
struct DeputyData {
    id_an: Option<String>,
    prenom: String,
    nom_de_famille: String,
    groupe_sigle: Option<String>,
}

struct CachedDeputies {
    by_an_id: HashMap<String, (String, Option<String>)>,
    fetched_at: Instant,
}

pub struct NosDeputesClient {
    http: reqwest::Client,
    cache: Mutex<Option<CachedDeputies>>,
}

impl NosDeputesClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
            cache: Mutex::new(None),
        }
    }

    async fn ensure_loaded(&self) {
        {
            let cache = self.cache.lock().unwrap();
            if let Some(ref c) = *cache {
                if c.fetched_at.elapsed().as_secs() < CACHE_TTL_SECS {
                    return;
                }
            }
        }

        let result = self.fetch_deputies().await;
        match result {
            Ok(map) => {
                let mut cache = self.cache.lock().unwrap();
                *cache = Some(CachedDeputies {
                    by_an_id: map,
                    fetched_at: Instant::now(),
                });
                tracing::info!("Deputies cache loaded");
            }
            Err(e) => {
                tracing::warn!("Failed to load deputies: {e}");
            }
        }
    }

    async fn fetch_deputies(
        &self,
    ) -> Result<HashMap<String, (String, Option<String>)>, reqwest::Error> {
        let resp: DeputiesResponse = self.http.get(DEPUTIES_URL).send().await?.json().await?;

        let mut map = HashMap::new();
        for w in resp.deputes {
            let d = w.depute;
            if let Some(ref id_an) = d.id_an {
                let key = format!("PA{id_an}");
                let full_name = format!("{} {}", d.prenom, d.nom_de_famille);
                map.insert(key, (full_name, d.groupe_sigle));
            }
        }
        Ok(map)
    }
}

#[async_trait]
impl DeputySource for NosDeputesClient {
    async fn resolve_initiators(&self, acteur_refs: &[String]) -> Vec<Initiator> {
        self.ensure_loaded().await;

        let cache = self.cache.lock().unwrap();
        acteur_refs
            .iter()
            .map(|r| {
                if let Some(ref c) = *cache {
                    if let Some((name, group)) = c.by_an_id.get(r) {
                        return Initiator::new(name.clone(), group.clone())
                            .expect("deputy name is non-empty");
                    }
                }
                Initiator::new(r.clone(), None).expect("ref is non-empty")
            })
            .collect()
    }
}
