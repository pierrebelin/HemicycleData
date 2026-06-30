use async_trait::async_trait;

use crate::domain::dossier::Initiator;

#[async_trait]
pub trait DeputySource: Send + Sync {
    async fn resolve_initiators(&self, acteur_refs: &[String]) -> Vec<Initiator>;
}
