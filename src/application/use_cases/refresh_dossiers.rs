use std::collections::BTreeSet;

use chrono::NaiveDate;

use crate::application::ports::actor_repository::ActorRepository;
use crate::application::ports::assembly_source::AssemblySource;
use crate::application::ports::dossier_repository::DossierRepository;
use crate::domain::actor::ActorUid;
use crate::domain::dossier::InitiatorAttachment;

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
    actor_repository: &'a dyn ActorRepository,
}

impl<'a> RefreshDossiers<'a> {
    pub fn new(
        source: &'a dyn AssemblySource,
        repository: &'a dyn DossierRepository,
        actor_repository: &'a dyn ActorRepository,
    ) -> Self {
        Self {
            source,
            repository,
            actor_repository,
        }
    }

    pub async fn execute(&self) -> Result<usize, RefreshError> {
        let since = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let dossiers_with_refs = self.source.fetch_dossiers_since_with_refs(since).await?;

        let actor_uids: Vec<ActorUid> = dossiers_with_refs
            .iter()
            .flat_map(|(_, refs)| refs.iter())
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter_map(|r| ActorUid::new(r).ok())
            .collect();

        let directory = self.actor_repository.load_directory_for(&actor_uids).await?;
        if directory.is_empty() && !actor_uids.is_empty() {
            tracing::warn!(
                "Actor registry is empty: {} initiator references cannot be resolved",
                actor_uids.len()
            );
        }

        let mut totals = InitiatorAttachment::default();
        let mut dossiers_without_deposit_date = 0;

        let dossiers: Vec<_> = dossiers_with_refs
            .into_iter()
            .map(|(mut dossier, refs)| {
                if !refs.is_empty() && dossier.initiators.is_empty() {
                    let uids: Vec<ActorUid> =
                        refs.into_iter().filter_map(|r| ActorUid::new(r).ok()).collect();
                    let report = dossier.attach_initiators(&uids, &directory);

                    totals.resolved += report.resolved;
                    totals.unknown_actors += report.unknown_actors;
                    totals.without_group += report.without_group;
                    totals.ambiguous += report.ambiguous;
                    if report.missing_deposit_date {
                        dossiers_without_deposit_date += 1;
                    }
                }
                dossier
            })
            .collect();

        tracing::info!(
            "Initiators attached: {} resolved, {} unknown actors, {} without a group at the deposit date, {} ambiguous; {} dossiers have no deposit date",
            totals.resolved,
            totals.unknown_actors,
            totals.without_group,
            totals.ambiguous,
            dossiers_without_deposit_date
        );

        let count = self.repository.save_all(&dossiers).await?;
        Ok(count)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use std::collections::HashMap;
    use std::sync::Mutex;

    use crate::application::ports::actor_repository::{RegistrySummary, RepositoryError};
    use crate::application::ports::assembly_source::SourceError;
    use crate::domain::actor::{
        Actor, ActorDirectory, ActorRegistry, ActorRole, GroupMembership, GroupUid,
        MembershipPeriod, MembershipQuality, ParliamentaryGroup,
    };
    use crate::domain::dossier::{
        CurationStatus, DossierUid, LegislativeAct, LegislativeDossier, Score,
    };

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

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

    /// Referentiel in-memory: un depute passe du groupe A au groupe B le 1er juillet 2025.
    pub(crate) struct InMemoryActorRepository {
        registry: Mutex<ActorRegistry>,
        pub(crate) requested_uids: Mutex<Vec<ActorUid>>,
    }

    impl InMemoryActorRepository {
        pub(crate) fn with_deputy_changing_group() -> Self {
            let actor = Actor::new(
                ActorUid::new("PA111111".into()).unwrap(),
                Some("M.".into()),
                "Jean".into(),
                "Dupont".into(),
                ActorRole::Deputy,
            )
            .unwrap();

            let group = |uid: &str, abbrev: &str, label: &str| {
                ParliamentaryGroup::new(
                    GroupUid::new(uid.into()).unwrap(),
                    17,
                    label.into(),
                    abbrev.into(),
                    None,
                    None,
                    None,
                )
                .unwrap()
            };

            let membership = |source: &str, group: &str, start, end| {
                GroupMembership::new(
                    source.into(),
                    ActorUid::new("PA111111".into()).unwrap(),
                    GroupUid::new(group.into()).unwrap(),
                    17,
                    MembershipPeriod::new(start, end).unwrap(),
                    MembershipQuality::new("Membre".into()).unwrap(),
                )
            };

            Self {
                registry: Mutex::new(ActorRegistry {
                    actors: vec![actor],
                    groups: vec![
                        group("PO_A", "A", "Groupe A"),
                        group("PO_B", "B", "Groupe B"),
                    ],
                    memberships: vec![
                        membership("PM1", "PO_A", date(2024, 7, 19), Some(date(2025, 6, 30))),
                        membership("PM2", "PO_B", date(2025, 7, 1), None),
                    ],
                }),
                requested_uids: Mutex::new(vec![]),
            }
        }
    }

    #[async_trait]
    impl ActorRepository for InMemoryActorRepository {
        async fn save_registry(
            &self,
            registry: &ActorRegistry,
        ) -> Result<RegistrySummary, RepositoryError> {
            let summary = RegistrySummary {
                actors: registry.actors.len(),
                groups: registry.groups.len(),
                memberships: registry.memberships.len(),
            };
            *self.registry.lock().unwrap() = registry.clone();
            Ok(summary)
        }

        async fn load_directory_for(
            &self,
            actor_uids: &[ActorUid],
        ) -> Result<ActorDirectory, RepositoryError> {
            self.requested_uids.lock().unwrap().extend_from_slice(actor_uids);

            let registry = self.registry.lock().unwrap();
            let actors: Vec<Actor> = registry
                .actors
                .iter()
                .filter(|a| actor_uids.contains(a.uid()))
                .cloned()
                .collect();
            let memberships: Vec<GroupMembership> = registry
                .memberships
                .iter()
                .filter(|m| actor_uids.contains(m.actor_uid()))
                .cloned()
                .collect();

            Ok(ActorDirectory::new(
                actors,
                registry.groups.clone(),
                memberships,
            ))
        }
    }

    pub(crate) struct InMemoryDossierRepository {
        pub(crate) dossiers: Mutex<HashMap<String, LegislativeDossier>>,
    }

    impl InMemoryDossierRepository {
        pub(crate) fn new() -> Self {
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
        ) -> Result<usize, crate::application::ports::dossier_repository::RepositoryError> {
            let mut store = self.dossiers.lock().unwrap();
            for d in dossiers {
                store.insert(d.uid.as_str().to_string(), d.clone());
            }
            Ok(dossiers.len())
        }

        async fn find_recent(
            &self,
            _since: NaiveDate,
        ) -> Result<Vec<LegislativeDossier>, crate::application::ports::dossier_repository::RepositoryError>
        {
            unreachable!()
        }

        async fn find_page(
            &self,
            _limit: i64,
            _offset: i64,
        ) -> Result<crate::application::ports::dossier_repository::DossierPage, RepositoryError>
        {
            unreachable!()
        }

        async fn find_by_uid(
            &self,
            _uid: &DossierUid,
        ) -> Result<
            Option<LegislativeDossier>,
            crate::application::ports::dossier_repository::RepositoryError,
        > {
            unreachable!()
        }

        async fn find_suggestions(
            &self,
            _count: usize,
        ) -> Result<Vec<LegislativeDossier>, crate::application::ports::dossier_repository::RepositoryError>
        {
            unreachable!()
        }

        async fn update_curation_status(
            &self,
            _uid: &DossierUid,
            _status: CurationStatus,
        ) -> Result<bool, crate::application::ports::dossier_repository::RepositoryError> {
            unreachable!()
        }
    }

    pub(crate) fn dossier(
        uid: &str,
        deposit_date: Option<NaiveDate>,
        last_activity: NaiveDate,
    ) -> LegislativeDossier {
        LegislativeDossier {
            uid: DossierUid::new(uid.into()).unwrap(),
            title: format!("Loi {uid}"),
            procedure: "Proposition de loi".into(),
            legislature: 17,
            url: None,
            summary: None,
            deposit_date,
            last_activity_date: last_activity,
            last_activity_label: "D\u{00e9}p\u{00f4}t".into(),
            acts: vec![LegislativeAct {
                date: last_activity,
                label: "D\u{00e9}p\u{00f4}t".into(),
                code: Some("AN1-DEPOT".into()),
            }],
            documents: vec![],
            score: Score::new(2, 4, 2, 23).unwrap(),
            current_stage: None,
            initiators: vec![],
            committee: None,
            curation_status: CurationStatus::New,
        }
    }

    #[tokio::test]
    async fn attaches_each_initiator_to_the_group_held_on_its_own_deposit_date() {
        let source = FakeSource {
            dossiers_with_refs: vec![
                (
                    dossier("D1", Some(date(2025, 5, 13)), date(2025, 5, 13)),
                    vec!["PA111111".into()],
                ),
                (
                    dossier("D2", Some(date(2025, 9, 1)), date(2025, 9, 1)),
                    vec!["PA111111".into()],
                ),
            ],
        };

        let repo = InMemoryDossierRepository::new();
        let actors = InMemoryActorRepository::with_deputy_changing_group();

        let count = RefreshDossiers::new(&source, &repo, &actors)
            .execute()
            .await
            .unwrap();

        assert_eq!(count, 2);
        assert_eq!(repo.count(), 2);

        let store = repo.dossiers.lock().unwrap();
        let d1 = &store["D1"].initiators[0];
        assert_eq!(d1.full_name(), "Jean Dupont");
        assert_eq!(d1.group().unwrap().abbrev, "A");
        assert_eq!(d1.reference_date(), Some(date(2025, 5, 13)));

        let d2 = &store["D2"].initiators[0];
        assert_eq!(d2.group().unwrap().abbrev, "B");
        assert_eq!(d2.reference_date(), Some(date(2025, 9, 1)));
    }

    #[tokio::test]
    async fn asks_the_registry_for_each_referenced_actor_only_once() {
        let source = FakeSource {
            dossiers_with_refs: vec![
                (
                    dossier("D1", Some(date(2025, 5, 13)), date(2025, 5, 13)),
                    vec!["PA111111".into()],
                ),
                (
                    dossier("D2", Some(date(2025, 9, 1)), date(2025, 9, 1)),
                    vec!["PA111111".into()],
                ),
            ],
        };

        let repo = InMemoryDossierRepository::new();
        let actors = InMemoryActorRepository::with_deputy_changing_group();

        RefreshDossiers::new(&source, &repo, &actors)
            .execute()
            .await
            .unwrap();

        assert_eq!(actors.requested_uids.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn dossier_without_initiator_reference_keeps_an_empty_list() {
        let source = FakeSource {
            dossiers_with_refs: vec![(
                dossier("D3", Some(date(2025, 5, 13)), date(2025, 5, 13)),
                vec![],
            )],
        };

        let repo = InMemoryDossierRepository::new();
        let actors = InMemoryActorRepository::with_deputy_changing_group();

        RefreshDossiers::new(&source, &repo, &actors)
            .execute()
            .await
            .unwrap();

        let store = repo.dossiers.lock().unwrap();
        assert!(store["D3"].initiators.is_empty());
    }

    #[tokio::test]
    async fn unknown_actor_is_kept_without_a_group() {
        let source = FakeSource {
            dossiers_with_refs: vec![(
                dossier("D4", Some(date(2025, 5, 13)), date(2025, 5, 13)),
                vec!["PA_UNKNOWN".into()],
            )],
        };

        let repo = InMemoryDossierRepository::new();
        let actors = InMemoryActorRepository::with_deputy_changing_group();

        RefreshDossiers::new(&source, &repo, &actors)
            .execute()
            .await
            .unwrap();

        let store = repo.dossiers.lock().unwrap();
        let initiator = &store["D4"].initiators[0];
        assert_eq!(initiator.full_name(), "PA_UNKNOWN");
        assert!(initiator.group().is_none());
    }

    #[tokio::test]
    async fn dossier_without_deposit_date_gets_a_name_but_no_group() {
        let source = FakeSource {
            dossiers_with_refs: vec![(
                dossier("D5", None, date(2025, 5, 13)),
                vec!["PA111111".into()],
            )],
        };

        let repo = InMemoryDossierRepository::new();
        let actors = InMemoryActorRepository::with_deputy_changing_group();

        RefreshDossiers::new(&source, &repo, &actors)
            .execute()
            .await
            .unwrap();

        let store = repo.dossiers.lock().unwrap();
        let initiator = &store["D5"].initiators[0];
        assert_eq!(initiator.full_name(), "Jean Dupont");
        assert!(initiator.group().is_none());
        assert!(initiator.reference_date().is_none());
    }
}
