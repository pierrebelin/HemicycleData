use crate::application::ports::assembly_source::AssemblySource;
use crate::application::ports::deputy_source::DeputySource;
use crate::application::ports::dossier_repository::DossierRepository;
use crate::domain::dossier::{DossierUid, LegislativeDossier};

#[derive(Debug, thiserror::Error)]
pub enum GetDossierError {
    #[error("{0}")]
    Repository(#[from] crate::application::ports::dossier_repository::RepositoryError),
    #[error("{0}")]
    Source(#[from] crate::application::ports::assembly_source::SourceError),
}

pub struct DossierDetailResult {
    pub dossier: LegislativeDossier,
    pub persisted: bool,
}

pub struct GetDossierDetail<'a> {
    repository: &'a dyn DossierRepository,
    source: &'a dyn AssemblySource,
    deputy_source: &'a dyn DeputySource,
}

impl<'a> GetDossierDetail<'a> {
    pub fn new(
        repository: &'a dyn DossierRepository,
        source: &'a dyn AssemblySource,
        deputy_source: &'a dyn DeputySource,
    ) -> Self {
        Self {
            repository,
            source,
            deputy_source,
        }
    }

    pub async fn execute(
        &self,
        uid: &DossierUid,
    ) -> Result<Option<DossierDetailResult>, GetDossierError> {
        if let Some(dossier) = self.repository.find_by_uid(uid).await? {
            return Ok(Some(DossierDetailResult {
                dossier,
                persisted: true,
            }));
        }

        if let Some((mut dossier, refs)) =
            self.source.fetch_dossier_by_uid_with_refs(uid).await?
        {
            if !refs.is_empty() {
                dossier.initiators = self.deputy_source.resolve_initiators(&refs).await;
            }
            return Ok(Some(DossierDetailResult {
                dossier,
                persisted: false,
            }));
        }

        Ok(None)
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
    use crate::domain::dossier::{CurationStatus, Initiator, LegislativeAct, Score};

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
            uid: &DossierUid,
        ) -> Result<Option<LegislativeDossier>, RepositoryError> {
            let store = self.dossiers.lock().unwrap();
            Ok(store.get(uid.as_str()).cloned())
        }

        async fn find_suggestions(
            &self,
            _count: usize,
        ) -> Result<Vec<LegislativeDossier>, RepositoryError> {
            unreachable!()
        }

        async fn update_curation_status(
            &self,
            _uid: &DossierUid,
            _status: CurationStatus,
        ) -> Result<bool, RepositoryError> {
            unreachable!()
        }
    }

    struct FakeSource {
        dossiers: Mutex<HashMap<String, (LegislativeDossier, Vec<String>)>>,
    }

    #[async_trait]
    impl AssemblySource for FakeSource {
        async fn fetch_dossiers_since(
            &self,
            _since: NaiveDate,
        ) -> Result<Vec<LegislativeDossier>, SourceError> {
            unreachable!()
        }

        async fn fetch_dossiers_since_with_refs(
            &self,
            _since: NaiveDate,
        ) -> Result<Vec<(LegislativeDossier, Vec<String>)>, SourceError> {
            unreachable!()
        }

        async fn fetch_dossier_by_uid(
            &self,
            uid: &DossierUid,
        ) -> Result<Option<LegislativeDossier>, SourceError> {
            let store = self.dossiers.lock().unwrap();
            Ok(store.get(uid.as_str()).map(|(d, _)| d.clone()))
        }

        async fn fetch_dossier_by_uid_with_refs(
            &self,
            uid: &DossierUid,
        ) -> Result<Option<(LegislativeDossier, Vec<String>)>, SourceError> {
            let store = self.dossiers.lock().unwrap();
            Ok(store.get(uid.as_str()).cloned())
        }
    }

    struct FakeDeputySource;

    #[async_trait]
    impl DeputySource for FakeDeputySource {
        async fn resolve_initiators(&self, acteur_refs: &[String]) -> Vec<Initiator> {
            acteur_refs
                .iter()
                .map(|r| Initiator::new(format!("Deputy {r}"), Some("GRP".into())).unwrap())
                .collect()
        }
    }

    fn sample_dossier() -> LegislativeDossier {
        LegislativeDossier {
            uid: DossierUid::new("DLR5L17N12345".into()).unwrap(),
            title: "Projet de loi de finances".into(),
            procedure: "PL".into(),
            legislature: 17,
            url: None,
            summary: None,
            last_activity_date: NaiveDate::from_ymd_opt(2026, 6, 25).unwrap(),
            last_activity_label: "Vote solennel".into(),
            acts: vec![
                LegislativeAct {
                    date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
                    label: "Dépôt".into(),
                    code: Some("AN1-DEPOT".into()),
                },
                LegislativeAct {
                    date: NaiveDate::from_ymd_opt(2026, 6, 25).unwrap(),
                    label: "Vote solennel".into(),
                    code: None,
                },
            ],
            documents: vec![],
            score: Score::new(9, 10, 4, 85).unwrap(),
            current_stage: None,
            initiators: vec![],
            committee: None,
            curation_status: CurationStatus::New,
        }
    }

    #[tokio::test]
    async fn returns_persisted_dossier_from_repository() {
        let mut map = HashMap::new();
        map.insert("DLR5L17N12345".into(), sample_dossier());

        let repo = InMemoryDossierRepository {
            dossiers: Mutex::new(map),
        };
        let source = FakeSource {
            dossiers: Mutex::new(HashMap::new()),
        };
        let deputies = FakeDeputySource;
        let uc = GetDossierDetail::new(&repo, &source, &deputies);
        let uid = DossierUid::new("DLR5L17N12345".into()).unwrap();
        let result = uc.execute(&uid).await.unwrap().unwrap();

        assert!(result.persisted);
        assert_eq!(result.dossier.uid.as_str(), "DLR5L17N12345");
        assert_eq!(result.dossier.acts.len(), 2);
    }

    #[tokio::test]
    async fn falls_back_to_source_and_resolves_initiators() {
        let repo = InMemoryDossierRepository {
            dossiers: Mutex::new(HashMap::new()),
        };
        let mut source_map = HashMap::new();
        source_map.insert(
            "DLR5L17N12345".into(),
            (sample_dossier(), vec!["PA123456".into()]),
        );
        let source = FakeSource {
            dossiers: Mutex::new(source_map),
        };
        let deputies = FakeDeputySource;
        let uc = GetDossierDetail::new(&repo, &source, &deputies);
        let uid = DossierUid::new("DLR5L17N12345".into()).unwrap();
        let result = uc.execute(&uid).await.unwrap().unwrap();

        assert!(!result.persisted);
        assert_eq!(result.dossier.initiators.len(), 1);
        assert_eq!(result.dossier.initiators[0].full_name(), "Deputy PA123456");
        assert_eq!(result.dossier.initiators[0].group(), Some("GRP"));
    }

    #[tokio::test]
    async fn returns_none_when_not_found_anywhere() {
        let repo = InMemoryDossierRepository {
            dossiers: Mutex::new(HashMap::new()),
        };
        let source = FakeSource {
            dossiers: Mutex::new(HashMap::new()),
        };
        let deputies = FakeDeputySource;
        let uc = GetDossierDetail::new(&repo, &source, &deputies);
        let uid = DossierUid::new("UNKNOWN".into()).unwrap();
        let result = uc.execute(&uid).await.unwrap();
        assert!(result.is_none());
    }
}
