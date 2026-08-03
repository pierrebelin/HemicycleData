use async_trait::async_trait;

use crate::domain::actor::ActorRegistry;

pub use super::SourceError;

/// Source du referentiel des acteurs et des groupes.
///
/// Le referentiel est charge en un lot: la source officielle publie une archive
/// complete (RM-05) et le volume — quelques milliers d'acteurs, un millier
/// d'appartenances — interdit une requete par acteur.
#[async_trait]
pub trait ActorSource: Send + Sync {
    /// Instantane du referentiel restreint a une legislature (RM-07).
    async fn fetch_registry(&self, legislature: u16) -> Result<ActorRegistry, SourceError>;
}
