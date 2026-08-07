use crate::application::ports::actor_repository::{ActorRepository, RegistrySummary};
use crate::application::ports::actor_source::ActorSource;
use crate::application::ports::assembly_source::AssemblySource;
use crate::application::ports::dossier_repository::DossierRepository;
use crate::application::ports::scrutin_repository::ScrutinRepository;
use crate::application::ports::scrutin_source::ScrutinSource;

use super::refresh_actor_registry::RefreshActorRegistry;
use super::refresh_dossiers::{DossiersSummary, RefreshDossiers, RefreshError, RefreshScope};
use super::refresh_scrutins::{RefreshScrutins, ScrutinsSummary};

#[derive(Debug, Clone)]
pub struct RefreshOutcome {
    pub registry: Option<RegistrySummary>,
    /// Anomalie signalee plutot que silencieuse: le referentiel n'a pas pu etre
    /// rafraichi, les rattachements reposent sur la version precedente.
    pub registry_anomaly: Option<String>,
    pub dossiers: DossiersSummary,
    pub scrutins: Option<ScrutinsSummary>,
    /// Idem cote scrutins: source indisponible, les scrutins deja stockes
    /// restent en place et la lacune est signalee.
    pub scrutins_anomaly: Option<String>,
}

/// Rafraichissement complet, dans l'ordre impose: referentiel d'abord, dossiers
/// et scrutins ensuite. L'inverse calculerait les rattachements et les
/// repartitions reconstruites sur des appartenances perimees (RM-11).
pub struct RefreshAll<'a> {
    actor_source: &'a dyn ActorSource,
    actor_repository: &'a dyn ActorRepository,
    assembly_source: &'a dyn AssemblySource,
    dossier_repository: &'a dyn DossierRepository,
    scrutin_source: &'a dyn ScrutinSource,
    scrutin_repository: &'a dyn ScrutinRepository,
}

impl<'a> RefreshAll<'a> {
    pub fn new(
        actor_source: &'a dyn ActorSource,
        actor_repository: &'a dyn ActorRepository,
        assembly_source: &'a dyn AssemblySource,
        dossier_repository: &'a dyn DossierRepository,
        scrutin_source: &'a dyn ScrutinSource,
        scrutin_repository: &'a dyn ScrutinRepository,
    ) -> Self {
        Self {
            actor_source,
            actor_repository,
            assembly_source,
            dossier_repository,
            scrutin_source,
            scrutin_repository,
        }
    }

    pub async fn execute(&self) -> Result<RefreshOutcome, RefreshError> {
        self.execute_with(RefreshScope::Incremental).await
    }

    pub async fn execute_with(&self, scope: RefreshScope) -> Result<RefreshOutcome, RefreshError> {
        let registry_result = RefreshActorRegistry::new(self.actor_source, self.actor_repository)
            .execute()
            .await;

        // CU-01: source indisponible, le referentiel precedent est conserve et
        // le rafraichissement des dossiers continue.
        let (registry, registry_anomaly) = match registry_result {
            Ok(summary) => (Some(summary), None),
            Err(e) => {
                tracing::warn!("Actor registry refresh failed, keeping the previous one: {e}");
                (None, Some(e.to_string()))
            }
        };

        let dossiers = RefreshDossiers::new(
            self.assembly_source,
            self.dossier_repository,
            self.actor_repository,
        )
        .execute_with(scope)
        .await?;

        // Meme regle que pour le referentiel: une source indisponible ne fait
        // pas echouer le reste, elle se signale (README.md §2).
        let (scrutins, scrutins_anomaly) = match RefreshScrutins::new(
            self.scrutin_source,
            self.scrutin_repository,
            self.actor_repository,
        )
        .execute()
        .await
        {
            Ok(summary) => (Some(summary), None),
            Err(e) => {
                tracing::warn!("Scrutins refresh failed, keeping the stored ones: {e}");
                (None, Some(e.to_string()))
            }
        };

        Ok(RefreshOutcome {
            registry,
            registry_anomaly,
            dossiers,
            scrutins,
            scrutins_anomaly,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use std::sync::Mutex;

    use crate::application::ports::actor_source::SourceError as ActorSourceError;
    use crate::application::ports::assembly_source::SourceError;
    use crate::application::ports::scrutin_repository::{
        DatasetShape, RepositoryError as ScrutinRepositoryError, ScrutinFilter, ScrutinPage,
        ScrutinSummary,
    };
    use crate::application::ports::scrutin_source::SourceError as ScrutinSourceError;
    use crate::domain::scrutin::{Scrutin, ScrutinUid};
    use crate::domain::actor::{ActorRegistry, ActorUid};
    use crate::domain::dossier::{DossierUid, LegislativeDossier};

    use super::super::refresh_dossiers::tests::{
        dossier, InMemoryActorRepository, InMemoryDossierRepository,
    };

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    struct FakeAssemblySource;

    #[async_trait]
    impl AssemblySource for FakeAssemblySource {
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
            Ok(vec![(
                dossier("D1", Some(date(2025, 5, 13)), date(2025, 5, 13)),
                vec!["PA111111".into()],
            )])
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

    struct FakeScrutinSource {
        available: bool,
    }

    #[async_trait]
    impl ScrutinSource for FakeScrutinSource {
        async fn fetch_scrutins(
            &self,
            _legislature: u16,
        ) -> Result<Vec<Scrutin>, ScrutinSourceError> {
            if !self.available {
                return Err(ScrutinSourceError::Download("source unavailable".into()));
            }
            Ok(vec![])
        }
    }

    struct InMemoryScrutinRepository {
        saved: Mutex<usize>,
    }

    #[async_trait]
    impl ScrutinRepository for InMemoryScrutinRepository {
        async fn save_scrutins(
            &self,
            scrutins: &[Scrutin],
        ) -> Result<usize, ScrutinRepositoryError> {
            *self.saved.lock().unwrap() += scrutins.len();
            Ok(scrutins.len())
        }
        async fn list(
            &self,
            _filter: &ScrutinFilter,
        ) -> Result<ScrutinPage, ScrutinRepositoryError> {
            unreachable!()
        }
        async fn by_uid(
            &self,
            _uid: &ScrutinUid,
        ) -> Result<Option<Scrutin>, ScrutinRepositoryError> {
            unreachable!()
        }
        async fn by_dossier(
            &self,
            _uid: &str,
        ) -> Result<Vec<ScrutinSummary>, ScrutinRepositoryError> {
            unreachable!()
        }

        async fn dataset_shape(&self) -> Result<DatasetShape, ScrutinRepositoryError> {
            unreachable!()
        }
    }

    struct RecordingActorSource {
        available: bool,
        order: Mutex<Vec<&'static str>>,
    }

    #[async_trait]
    impl ActorSource for RecordingActorSource {
        async fn fetch_registry(
            &self,
            _legislature: u16,
        ) -> Result<ActorRegistry, ActorSourceError> {
            self.order.lock().unwrap().push("registry");
            if !self.available {
                return Err(ActorSourceError::Download("source unavailable".into()));
            }
            Ok(ActorRegistry {
                actors: vec![],
                groups: vec![],
                memberships: vec![],
            })
        }
    }

    #[tokio::test]
    async fn refreshes_the_registry_before_the_dossiers() {
        let actor_source = RecordingActorSource {
            available: true,
            order: Mutex::new(vec![]),
        };
        let actor_repository = InMemoryActorRepository::with_deputy_changing_group();
        let assembly_source = FakeAssemblySource;
        let dossier_repository = InMemoryDossierRepository::new();
        let scrutin_source = FakeScrutinSource { available: true };
        let scrutin_repository = InMemoryScrutinRepository {
            saved: Mutex::new(0),
        };

        let outcome = RefreshAll::new(
            &actor_source,
            &actor_repository,
            &assembly_source,
            &dossier_repository,
            &scrutin_source,
            &scrutin_repository,
        )
        .execute()
        .await
        .unwrap();

        assert_eq!(*actor_source.order.lock().unwrap(), vec!["registry"]);
        assert!(outcome.scrutins.is_some());
        assert!(outcome.scrutins_anomaly.is_none());
        assert!(actor_repository
            .requested_uids
            .lock()
            .unwrap()
            .contains(&ActorUid::new("PA111111".into()).unwrap()));
        assert_eq!(outcome.dossiers.written, 1);
        assert!(outcome.registry_anomaly.is_none());
    }

    #[tokio::test]
    async fn keeps_refreshing_dossiers_when_the_registry_source_is_down() {
        let actor_source = RecordingActorSource {
            available: false,
            order: Mutex::new(vec![]),
        };
        let actor_repository = InMemoryActorRepository::with_deputy_changing_group();
        let assembly_source = FakeAssemblySource;
        let dossier_repository = InMemoryDossierRepository::new();
        let scrutin_source = FakeScrutinSource { available: false };
        let scrutin_repository = InMemoryScrutinRepository {
            saved: Mutex::new(0),
        };

        let outcome = RefreshAll::new(
            &actor_source,
            &actor_repository,
            &assembly_source,
            &dossier_repository,
            &scrutin_source,
            &scrutin_repository,
        )
        .execute()
        .await
        .unwrap();

        assert!(outcome.registry.is_none());
        // Source scrutins indisponible: le reste passe, la lacune est signalee.
        assert!(outcome.scrutins.is_none());
        assert!(outcome.scrutins_anomaly.is_some());
        assert_eq!(*scrutin_repository.saved.lock().unwrap(), 0);
        assert!(outcome.registry_anomaly.is_some());
        assert_eq!(outcome.dossiers.written, 1);

        // Le referentiel precedent a bien servi au rattachement.
        let store = dossier_repository.dossiers.lock().unwrap();
        assert_eq!(store["D1"].initiators[0].group().unwrap().abbrev, "A");
    }
}
