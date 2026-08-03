use crate::application::ports::actor_repository::{ActorRepository, RegistrySummary};
use crate::application::ports::actor_source::ActorSource;
use crate::application::ports::assembly_source::AssemblySource;
use crate::application::ports::dossier_repository::DossierRepository;

use super::refresh_actor_registry::RefreshActorRegistry;
use super::refresh_dossiers::{RefreshDossiers, RefreshError};

#[derive(Debug, Clone)]
pub struct RefreshOutcome {
    pub registry: Option<RegistrySummary>,
    /// Anomalie signalee plutot que silencieuse: le referentiel n'a pas pu etre
    /// rafraichi, les rattachements reposent sur la version precedente.
    pub registry_anomaly: Option<String>,
    pub dossiers: usize,
}

/// Rafraichissement complet, dans l'ordre impose: referentiel d'abord, dossiers
/// ensuite. L'inverse calculerait les rattachements sur des appartenances
/// perimees.
pub struct RefreshAll<'a> {
    actor_source: &'a dyn ActorSource,
    actor_repository: &'a dyn ActorRepository,
    assembly_source: &'a dyn AssemblySource,
    dossier_repository: &'a dyn DossierRepository,
}

impl<'a> RefreshAll<'a> {
    pub fn new(
        actor_source: &'a dyn ActorSource,
        actor_repository: &'a dyn ActorRepository,
        assembly_source: &'a dyn AssemblySource,
        dossier_repository: &'a dyn DossierRepository,
    ) -> Self {
        Self {
            actor_source,
            actor_repository,
            assembly_source,
            dossier_repository,
        }
    }

    pub async fn execute(&self) -> Result<RefreshOutcome, RefreshError> {
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
        .execute()
        .await?;

        Ok(RefreshOutcome {
            registry,
            registry_anomaly,
            dossiers,
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

        let outcome = RefreshAll::new(
            &actor_source,
            &actor_repository,
            &assembly_source,
            &dossier_repository,
        )
        .execute()
        .await
        .unwrap();

        assert_eq!(*actor_source.order.lock().unwrap(), vec!["registry"]);
        assert!(actor_repository
            .requested_uids
            .lock()
            .unwrap()
            .contains(&ActorUid::new("PA111111".into()).unwrap()));
        assert_eq!(outcome.dossiers, 1);
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

        let outcome = RefreshAll::new(
            &actor_source,
            &actor_repository,
            &assembly_source,
            &dossier_repository,
        )
        .execute()
        .await
        .unwrap();

        assert!(outcome.registry.is_none());
        assert!(outcome.registry_anomaly.is_some());
        assert_eq!(outcome.dossiers, 1);

        // Le referentiel precedent a bien servi au rattachement.
        let store = dossier_repository.dossiers.lock().unwrap();
        assert_eq!(store["D1"].initiators[0].group().unwrap().abbrev, "A");
    }
}
