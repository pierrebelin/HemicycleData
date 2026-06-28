use crate::application::ports::dossier_repository::{DossierRepository, RepositoryError};
use crate::domain::dossier::LegislativeDossier;

pub struct GetDossierDetail<'a> {
    repository: &'a dyn DossierRepository,
}

impl<'a> GetDossierDetail<'a> {
    pub fn new(repository: &'a dyn DossierRepository) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, uid: &str) -> Result<Option<LegislativeDossier>, RepositoryError> {
        self.repository.find_by_uid(uid).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use std::collections::HashMap;
    use std::sync::Mutex;

    use crate::application::ports::dossier_repository::RepositoryError;
    use crate::domain::dossier::{LegislativeAct, LegislativeDossier, Score};

    struct InMemoryDossierRepository {
        dossiers: Mutex<HashMap<String, LegislativeDossier>>,
    }

    #[async_trait]
    impl DossierRepository for InMemoryDossierRepository {
        async fn save_all(
            &self,
            _dossiers: &[LegislativeDossier],
        ) -> Result<usize, RepositoryError> {
            unreachable!()
        }

        async fn find_recent(
            &self,
            _since: NaiveDate,
        ) -> Result<Vec<LegislativeDossier>, RepositoryError> {
            unreachable!()
        }

        async fn find_by_uid(
            &self,
            uid: &str,
        ) -> Result<Option<LegislativeDossier>, RepositoryError> {
            let store = self.dossiers.lock().unwrap();
            Ok(store.get(uid).map(|d| LegislativeDossier {
                uid: d.uid.clone(),
                title: d.title.clone(),
                procedure: d.procedure.clone(),
                last_activity_date: d.last_activity_date,
                last_activity_label: d.last_activity_label.clone(),
                acts: d.acts.clone(),
                score: d.score.clone(),
            }))
        }
    }

    #[tokio::test]
    async fn returns_dossier_when_found() {
        let mut map = HashMap::new();
        map.insert(
            "DLR5L17N12345".into(),
            LegislativeDossier {
                uid: "DLR5L17N12345".into(),
                title: "Projet de loi de finances".into(),
                procedure: "PL".into(),
                last_activity_date: NaiveDate::from_ymd_opt(2026, 6, 25).unwrap(),
                last_activity_label: "Vote solennel".into(),
                acts: vec![
                    LegislativeAct {
                        date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
                        label: "Dépôt".into(),
                    },
                    LegislativeAct {
                        date: NaiveDate::from_ymd_opt(2026, 6, 25).unwrap(),
                        label: "Vote solennel".into(),
                    },
                ],
                score: Score {
                    progress: 9,
                    magnitude: 10,
                    momentum: 4,
                    total: 85,
                },
            },
        );

        let repo = InMemoryDossierRepository {
            dossiers: Mutex::new(map),
        };
        let uc = GetDossierDetail::new(&repo);
        let result = uc.execute("DLR5L17N12345").await.unwrap();

        assert!(result.is_some());
        let dossier = result.unwrap();
        assert_eq!(dossier.uid, "DLR5L17N12345");
        assert_eq!(dossier.acts.len(), 2);
    }

    #[tokio::test]
    async fn returns_none_when_not_found() {
        let repo = InMemoryDossierRepository {
            dossiers: Mutex::new(HashMap::new()),
        };
        let uc = GetDossierDetail::new(&repo);
        let result = uc.execute("UNKNOWN").await.unwrap();
        assert!(result.is_none());
    }
}
