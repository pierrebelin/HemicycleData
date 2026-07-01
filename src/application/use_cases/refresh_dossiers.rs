use chrono::NaiveDate;

use crate::application::ports::assembly_source::AssemblySource;
use crate::application::ports::deputy_source::DeputySource;
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
    deputy_source: &'a dyn DeputySource,
}

impl<'a> RefreshDossiers<'a> {
    pub fn new(
        source: &'a dyn AssemblySource,
        repository: &'a dyn DossierRepository,
        deputy_source: &'a dyn DeputySource,
    ) -> Self {
        Self {
            source,
            repository,
            deputy_source,
        }
    }

    pub async fn execute(&self) -> Result<usize, RefreshError> {
        let since = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let dossiers_with_refs = self.source.fetch_dossiers_since_with_refs(since).await?;

        let all_refs: Vec<String> = dossiers_with_refs
            .iter()
            .flat_map(|(_, refs)| refs.iter().cloned())
            .collect();

        let resolved = if !all_refs.is_empty() {
            self.deputy_source.resolve_initiators(&all_refs).await
        } else {
            vec![]
        };

        let ref_to_initiator: std::collections::HashMap<&str, &crate::domain::dossier::Initiator> =
            all_refs.iter().zip(resolved.iter()).map(|(r, i)| (r.as_str(), i)).collect();

        let dossiers: Vec<_> = dossiers_with_refs
            .into_iter()
            .map(|(mut dossier, refs)| {
                if !refs.is_empty() && dossier.initiators.is_empty() {
                    dossier.initiators = refs
                        .iter()
                        .filter_map(|r| ref_to_initiator.get(r.as_str()).map(|i| (*i).clone()))
                        .collect();
                }
                dossier
            })
            .collect();

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
    use crate::domain::dossier::{CurationStatus, DossierUid, Initiator, LegislativeAct, LegislativeDossier, Score};

    struct FakeSource {
        dossiers_with_refs: Vec<(LegislativeDossier, Vec<String>)>,
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
            Ok(self.dossiers_with_refs.clone())
        }

        async fn fetch_dossier_by_uid(
            &self,
            _uid: &DossierUid,
        ) -> Result<Option<LegislativeDossier>, SourceError> {
            unreachable!()
        }

        async fn fetch_dossier_by_uid_with_refs(
            &self,
            _uid: &DossierUid,
        ) -> Result<Option<(LegislativeDossier, Vec<String>)>, SourceError> {
            unreachable!()
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
                store.insert(d.uid.as_str().to_string(), d.clone());
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
            _uid: &DossierUid,
        ) -> Result<Option<LegislativeDossier>, RepositoryError> {
            unreachable!()
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

    #[tokio::test]
    async fn saves_all_dossiers_from_source() {
        let source = FakeSource {
            dossiers_with_refs: vec![
                (
                    LegislativeDossier {
                        uid: DossierUid::new("D1".into()).unwrap(),
                        title: "Loi A".into(),
                        procedure: "PL".into(),
                        legislature: 17,
                        url: None,
                        summary: None,
                        last_activity_date: NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
                        last_activity_label: "Dépôt".into(),
                        acts: vec![LegislativeAct {
                            date: NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
                            label: "Dépôt".into(),
                            code: Some("AN1-DEPOT".into()),
                        }],
                        documents: vec![],
                        score: Score::new(2, 4, 2, 23).unwrap(),
                        current_stage: None,
                        initiators: vec![],
                        committee: None,
                        curation_status: CurationStatus::New,
                    },
                    vec!["PA111111".into()],
                ),
                (
                    LegislativeDossier {
                        uid: DossierUid::new("D2".into()).unwrap(),
                        title: "Loi B".into(),
                        procedure: "PPL".into(),
                        legislature: 17,
                        url: None,
                        summary: None,
                        last_activity_date: NaiveDate::from_ymd_opt(2026, 6, 25).unwrap(),
                        last_activity_label: "Vote".into(),
                        acts: vec![],
                        documents: vec![],
                        score: Score::new(9, 4, 2, 62).unwrap(),
                        current_stage: None,
                        initiators: vec![],
                        committee: None,
                        curation_status: CurationStatus::New,
                    },
                    vec![],
                ),
            ],
        };

        let repo = InMemoryDossierRepository::new();
        let deputies = FakeDeputySource;
        let uc = RefreshDossiers::new(&source, &repo, &deputies);
        let count = uc.execute().await.unwrap();

        assert_eq!(count, 2);
        assert_eq!(repo.count(), 2);

        let store = repo.dossiers.lock().unwrap();
        assert_eq!(store["D1"].initiators.len(), 1);
        assert_eq!(store["D1"].initiators[0].full_name(), "Deputy PA111111");
        assert!(store["D2"].initiators.is_empty());
    }
}
