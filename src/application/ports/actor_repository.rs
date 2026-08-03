use async_trait::async_trait;

use crate::domain::actor::{ActorDirectory, ActorRegistry, ActorUid};

pub use super::RepositoryError;

/// Nombre de lignes ecrites par un rafraichissement du referentiel.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RegistrySummary {
    pub actors: usize,
    pub groups: usize,
    pub memberships: usize,
}

#[async_trait]
pub trait ActorRepository: Send + Sync {
    /// Ecrit l'instantane. Les appartenances closes sont conservees: elles
    /// portent les actes de leur periode.
    async fn save_registry(
        &self,
        registry: &ActorRegistry,
    ) -> Result<RegistrySummary, RepositoryError>;

    /// Charge la vue de lecture pour les acteurs demandes, avec toutes leurs
    /// appartenances — actives comme closes — afin de resoudre le groupe a la
    /// date de l'acte (RM-01).
    async fn load_directory_for(
        &self,
        actor_uids: &[ActorUid],
    ) -> Result<ActorDirectory, RepositoryError>;
}
