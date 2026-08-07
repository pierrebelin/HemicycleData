//! Telechargement des archives de data.assemblee-nationale.fr.
//!
//! Les trois sources (dossiers, scrutins, referentiel des acteurs) suivent le
//! meme schema: un fichier ZIP complet, republie tel quel, sans sous-ensemble a
//! demander ni flux incremental (RM-01, RM-05). Elles partagent donc le meme
//! client: une identite declaree, un cache en memoire, et surtout une
//! revalidation conditionnelle plutot qu'un retelechargement systematique.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use reqwest::header::{ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use reqwest::StatusCode;

use crate::application::ports::SourceError;

/// Duree pendant laquelle l'archive en memoire est servie sans meme demander a
/// la source si elle a change. Couvre le cas d'un rafraichissement relance a la
/// main juste apres un autre.
const CACHE_TTL: Duration = Duration::from_secs(3600);

/// Identite du client aupres de la source.
///
/// Un client anonyme est le premier profil qu'un administrateur filtre quand il
/// resserre les vannes. Se nommer, avec une adresse ou nous joindre, c'est la
/// difference entre « bot inconnu » et « projet qu'on previent avant de
/// couper ».
const USER_AGENT: &str = concat!(
    "hemicycle.data/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/pierrebelin/HemicycleData)"
);

/// Le telechargement complet porte plusieurs dizaines de mega-octets: large,
/// mais borne. Au-dela, la connexion est bloquee, pas lente.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(600);

/// Ce que la source a renvoye avec l'archive et qui permet de lui redemander
/// « seulement si ca a change ».
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Validators {
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

impl Validators {
    fn is_empty(&self) -> bool {
        self.etag.is_none() && self.last_modified.is_none()
    }
}

/// Ce qu'il y a a faire avant de servir l'archive.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Decision {
    /// Copie recente: la servir sans rien demander.
    ServeCached,
    /// Copie datee mais identifiable: demander a la source si elle a change.
    Revalidate(Validators),
    /// Rien en cache, ou une copie que la source ne sait pas identifier.
    Download,
}

struct CachedArchive {
    data: Vec<u8>,
    fetched_at: Instant,
    validators: Validators,
}

pub struct ArchiveFetcher {
    http: reqwest::Client,
    url: &'static str,
    /// Nom court pour les journaux: « dossiers », « scrutins »...
    label: &'static str,
    cache: Mutex<Option<CachedArchive>>,
}

impl ArchiveFetcher {
    pub fn new(url: &'static str, label: &'static str) -> Self {
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            // Repris tel quel du code precedent. A reexaminer: desactiver la
            // verification du certificat n'a de raison d'etre que si la chaine
            // de la source est cassee, ce qui se verifie.
            .danger_accept_invalid_certs(true)
            .build()
            .expect("failed to build HTTP client");

        Self {
            http,
            url,
            label,
            cache: Mutex::new(None),
        }
    }

    fn decide(cached: Option<&CachedArchive>, now: Instant) -> Decision {
        let Some(cached) = cached else {
            return Decision::Download;
        };

        if now.duration_since(cached.fetched_at) < CACHE_TTL {
            return Decision::ServeCached;
        }

        if cached.validators.is_empty() {
            return Decision::Download;
        }

        Decision::Revalidate(cached.validators.clone())
    }

    /// Rend l'archive, en la retelechargeant seulement si la source dit qu'elle
    /// a change.
    ///
    /// Le processus qui appelle est le serveur lui-meme, vivant plusieurs
    /// jours: le cache et ses validateurs traversent donc les passages
    /// successifs du rafraichissement periodique. Sur une archive republiee une
    /// fois par jour et relue toutes les deux heures, onze passages sur douze
    /// se resolvent en un `304 Not Modified` de quelques octets — et evitent au
    /// passage de reparser un ZIP identique.
    pub async fn fetch(&self) -> Result<Vec<u8>, SourceError> {
        let decision = {
            let cache = self.cache.lock().unwrap();
            Self::decide(cache.as_ref(), Instant::now())
        };

        let validators = match decision {
            Decision::ServeCached => {
                tracing::debug!("Using cached {} archive", self.label);
                let cache = self.cache.lock().unwrap();
                // Seule la tache de rafraichissement ecrit ici, et jamais deux
                // en parallele: la copie decidee juste au-dessus est encore la.
                if let Some(cached) = cache.as_ref() {
                    return Ok(cached.data.clone());
                }
                Validators::default()
            }
            Decision::Revalidate(validators) => validators,
            Decision::Download => Validators::default(),
        };

        let mut request = self.http.get(self.url);
        if let Some(etag) = &validators.etag {
            request = request.header(IF_NONE_MATCH, etag);
        }
        if let Some(last_modified) = &validators.last_modified {
            request = request.header(IF_MODIFIED_SINCE, last_modified);
        }

        if validators.is_empty() {
            tracing::info!("Downloading {}", self.url);
        } else {
            tracing::info!("Revalidating {}", self.url);
        }

        let response = request
            .send()
            .await
            .map_err(|e| SourceError::Download(e.to_string()))?;

        if response.status() == StatusCode::NOT_MODIFIED {
            let cache = self.cache.lock().unwrap();
            let Some(cached) = cache.as_ref() else {
                // La source repond « inchangee » a une question que seule une
                // copie en cache permet de poser. Sans elle, il n'y a rien a
                // servir: le dire plutot que de rendre du vide.
                return Err(SourceError::Download(format!(
                    "{} archive: 304 Not Modified without a cached copy",
                    self.label
                )));
            };
            tracing::info!(
                "{} archive unchanged ({} bytes served from cache)",
                self.label,
                cached.data.len()
            );
            return Ok(cached.data.clone());
        }

        if !response.status().is_success() {
            return Err(SourceError::Download(format!("HTTP {}", response.status())));
        }

        let validators = Validators {
            etag: header_value(&response, ETAG),
            last_modified: header_value(&response, LAST_MODIFIED),
        };
        if validators.is_empty() {
            // Sans validateur, chaque passage repartira sur un telechargement
            // complet. Ce n'est pas une panne, c'est un cout: le signaler.
            tracing::warn!(
                "{} archive carries neither ETag nor Last-Modified: every pass will download it in full",
                self.label
            );
        }

        let data = response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| SourceError::Download(e.to_string()))?;

        tracing::info!("Downloaded {} bytes ({})", data.len(), self.label);

        {
            let mut cache = self.cache.lock().unwrap();
            *cache = Some(CachedArchive {
                data: data.clone(),
                fetched_at: Instant::now(),
                validators,
            });
        }

        Ok(data)
    }
}

fn header_value(response: &reqwest::Response, name: reqwest::header::HeaderName) -> Option<String> {
    response
        .headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cached(age: Duration, validators: Validators) -> CachedArchive {
        CachedArchive {
            data: vec![1, 2, 3],
            fetched_at: Instant::now() - age,
            validators,
        }
    }

    fn etag(value: &str) -> Validators {
        Validators {
            etag: Some(value.to_string()),
            last_modified: None,
        }
    }

    #[test]
    fn nothing_cached_means_downloading() {
        assert_eq!(
            ArchiveFetcher::decide(None, Instant::now()),
            Decision::Download
        );
    }

    #[test]
    fn a_recent_copy_is_served_without_asking_anything() {
        let cached = cached(Duration::from_secs(60), etag("\"abc\""));

        assert_eq!(
            ArchiveFetcher::decide(Some(&cached), Instant::now()),
            Decision::ServeCached
        );
    }

    /// Le cas du rafraichissement periodique: la copie a deux heures, la source
    /// est interrogee, mais avec son validateur — pas un telechargement sec.
    #[test]
    fn a_dated_copy_is_revalidated_not_downloaded() {
        let cached = cached(Duration::from_secs(7200), etag("\"abc\""));

        assert_eq!(
            ArchiveFetcher::decide(Some(&cached), Instant::now()),
            Decision::Revalidate(etag("\"abc\""))
        );
    }

    #[test]
    fn last_modified_alone_is_enough_to_revalidate() {
        let validators = Validators {
            etag: None,
            last_modified: Some("Wed, 06 Aug 2026 09:00:00 GMT".to_string()),
        };
        let cached = cached(Duration::from_secs(7200), validators.clone());

        assert_eq!(
            ArchiveFetcher::decide(Some(&cached), Instant::now()),
            Decision::Revalidate(validators)
        );
    }

    /// Sans validateur, il n'y a rien a demander conditionnellement: la seule
    /// facon de savoir si l'archive a change est de la reprendre.
    #[test]
    fn a_dated_copy_without_validator_is_downloaded_again() {
        let cached = cached(Duration::from_secs(7200), Validators::default());

        assert_eq!(
            ArchiveFetcher::decide(Some(&cached), Instant::now()),
            Decision::Download
        );
    }

    #[test]
    fn the_client_names_itself_and_says_where_to_reach_us() {
        assert!(USER_AGENT.starts_with("hemicycle.data/"));
        assert!(USER_AGENT.contains("https://github.com/pierrebelin/HemicycleData"));
    }
}
