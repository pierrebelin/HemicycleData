use crate::application::ports::assembly_source::AssemblySource;
use crate::application::ports::dossier_repository::DossierRepository;

#[derive(Debug, thiserror::Error)]
pub enum SaveDossierError {
    #[error("{0}")]
    Source(#[from] crate::application::ports::assembly_source::SourceError),
    #[error("{0}")]
    Repository(#[from] crate::application::ports::dossier_repository::RepositoryError),
    #[error("dossier not found: {0}")]
    NotFound(String),
}

pub struct SaveDossier<'a> {
    source: &'a dyn AssemblySource,
    repository: &'a dyn DossierRepository,
}

impl<'a> SaveDossier<'a> {
    pub fn new(source: &'a dyn AssemblySource, repository: &'a dyn DossierRepository) -> Self {
        Self { source, repository }
    }

    pub async fn execute(&self, uid: &str) -> Result<(), SaveDossierError> {
        let dossier = self
            .source
            .fetch_dossier_by_uid(uid)
            .await?
            .ok_or_else(|| SaveDossierError::NotFound(uid.to_string()))?;

        self.repository.save_all(&[dossier]).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use std::collections::HashMap;
    use std::sync::Mutex;

    use crate::application::ports::assembly_source::SourceError;
    use crate::application::ports::dossier_repository::RepositoryError;
    use crate::domain::dossier::{LegislativeDossier, Score};

    struct FakeSource {
        dossiers: Mutex<HashMap<String, LegislativeDossier>>,
    }

    #[async_trait]
    impl AssemblySource for FakeSource {
        async fn fetch_dossiers_since(
            &self,
            _since: NaiveDate,
        ) -> Result<Vec<LegislativeDossier>, SourceError> {
            unreachable!()
        }

        async fn fetch_dossier_by_uid(
            &self,
            uid: &str,
        ) -> Result<Option<LegislativeDossier>, SourceError> {
            let store = self.dossiers.lock().unwrap();
            Ok(store.get(uid).cloned())
        }
    }

    struct InMemoryDossierRepository {
        dossiers: Mutex<HashMap<String, LegislativeDossier>>,
    }

    #[async_trait]
    impl DossierRepository for InMemoryDossierRepository {
        async fn save_all(
            &self,
            dossiers: &[LegislativeDossier],
        ) -> Result<usize, RepositoryError> {
            let mut store = self.dossiers.lock().unwrap();
            for d in dossiers {
                store.insert(d.uid.clone(), d.clone());
            }
            Ok(dossiers.len())
        }

        async fn find_recent(
            &self,
            _since: NaiveDate,
        ) -> Result<Vec<LegislativeDossier>, RepositoryError> {
            unreachable!()
        }

        async fn find_by_uid(
            &self,
            _uid: &str,
        ) -> Result<Option<LegislativeDossier>, RepositoryError> {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn saves_dossier_from_source() {
        let mut source_map = HashMap::new();
        source_map.insert(
            "D1".into(),
            LegislativeDossier {
                uid: "D1".into(),
                title: "Loi test".into(),
                procedure: "PL".into(),
                last_activity_date: NaiveDate::from_ymd_opt(2026, 6, 25).unwrap(),
                last_activity_label: "Dépôt".into(),
                acts: vec![],
                score: Score {
                    progress: 2,
                    magnitude: 4,
                    momentum: 2,
                    total: 23,
                },
            },
        );

        let source = FakeSource {
            dossiers: Mutex::new(source_map),
        };
        let repo = InMemoryDossierRepository {
            dossiers: Mutex::new(HashMap::new()),
        };

        let uc = SaveDossier::new(&source, &repo);
        uc.execute("D1").await.unwrap();

        assert_eq!(repo.dossiers.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn returns_not_found_when_missing_from_source() {
        let source = FakeSource {
            dossiers: Mutex::new(HashMap::new()),
        };
        let repo = InMemoryDossierRepository {
            dossiers: Mutex::new(HashMap::new()),
        };

        let uc = SaveDossier::new(&source, &repo);
        let result = uc.execute("UNKNOWN").await;

        assert!(matches!(result, Err(SaveDossierError::NotFound(_))));
    }
}
