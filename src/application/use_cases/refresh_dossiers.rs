use chrono::NaiveDate;

use crate::application::ports::assembly_source::AssemblySource;
use crate::application::ports::dossier_repository::DossierRepository;

#[derive(Debug, thiserror::Error)]
pub enum RefreshError {
    #[error("{0}")]
    Source(#[from] crate::application::ports::assembly_source::SourceError),
    #[error("{0}")]
    Repository(#[from] crate::application::ports::dossier_repository::RepositoryError),
}

pub struct RefreshDossiers<'a> {
    source: &'a dyn AssemblySource,
    repository: &'a dyn DossierRepository,
}

impl<'a> RefreshDossiers<'a> {
    pub fn new(source: &'a dyn AssemblySource, repository: &'a dyn DossierRepository) -> Self {
        Self { source, repository }
    }

    pub async fn execute(&self) -> Result<usize, RefreshError> {
        let since = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let dossiers = self.source.fetch_dossiers_since(since).await?;
        let count = self.repository.save_all(&dossiers).await?;
        Ok(count)
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
    use crate::domain::dossier::{LegislativeAct, LegislativeDossier, Score};

    struct FakeSource {
        dossiers: Vec<LegislativeDossier>,
    }

    #[async_trait]
    impl AssemblySource for FakeSource {
        async fn fetch_dossiers_since(
            &self,
            _since: NaiveDate,
        ) -> Result<Vec<LegislativeDossier>, SourceError> {
            Ok(self
                .dossiers
                .iter()
                .map(|d| LegislativeDossier {
                    uid: d.uid.clone(),
                    title: d.title.clone(),
                    procedure: d.procedure.clone(),
                    last_activity_date: d.last_activity_date,
                    last_activity_label: d.last_activity_label.clone(),
                    acts: d.acts.clone(),
                    score: d.score.clone(),
                })
                .collect())
        }

        async fn fetch_dossier_by_uid(
            &self,
            _uid: &str,
        ) -> Result<Option<LegislativeDossier>, SourceError> {
            unreachable!()
        }
    }

    struct InMemoryDossierRepository {
        dossiers: Mutex<HashMap<String, LegislativeDossier>>,
    }

    impl InMemoryDossierRepository {
        fn new() -> Self {
            Self {
                dossiers: Mutex::new(HashMap::new()),
            }
        }

        fn count(&self) -> usize {
            self.dossiers.lock().unwrap().len()
        }
    }

    #[async_trait]
    impl DossierRepository for InMemoryDossierRepository {
        async fn save_all(
            &self,
            dossiers: &[LegislativeDossier],
        ) -> Result<usize, RepositoryError> {
            let mut store = self.dossiers.lock().unwrap();
            for d in dossiers {
                store.insert(
                    d.uid.clone(),
                    LegislativeDossier {
                        uid: d.uid.clone(),
                        title: d.title.clone(),
                        procedure: d.procedure.clone(),
                        last_activity_date: d.last_activity_date,
                        last_activity_label: d.last_activity_label.clone(),
                        acts: d.acts.clone(),
                        score: d.score.clone(),
                    },
                );
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
    async fn saves_all_dossiers_from_source() {
        let source = FakeSource {
            dossiers: vec![
                LegislativeDossier {
                    uid: "D1".into(),
                    title: "Loi A".into(),
                    procedure: "PL".into(),
                    last_activity_date: NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
                    last_activity_label: "Dépôt".into(),
                    acts: vec![LegislativeAct {
                        date: NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
                        label: "Dépôt".into(),
                    }],
                    score: Score {
                        progress: 2,
                        magnitude: 4,
                        momentum: 2,
                        total: 23,
                    },
                },
                LegislativeDossier {
                    uid: "D2".into(),
                    title: "Loi B".into(),
                    procedure: "PPL".into(),
                    last_activity_date: NaiveDate::from_ymd_opt(2026, 6, 25).unwrap(),
                    last_activity_label: "Vote".into(),
                    acts: vec![],
                    score: Score {
                        progress: 9,
                        magnitude: 4,
                        momentum: 2,
                        total: 62,
                    },
                },
            ],
        };

        let repo = InMemoryDossierRepository::new();
        let uc = RefreshDossiers::new(&source, &repo);
        let count = uc.execute().await.unwrap();

        assert_eq!(count, 2);
        assert_eq!(repo.count(), 2);
    }
}
