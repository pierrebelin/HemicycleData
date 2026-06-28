use async_trait::async_trait;
use chrono::NaiveDate;

use crate::domain::dossier::LegislativeDossier;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    Database(String),
}

#[async_trait]
pub trait DossierRepository: Send + Sync {
    async fn save_all(&self, dossiers: &[LegislativeDossier]) -> Result<usize, RepositoryError>;
    async fn find_recent(&self, since: NaiveDate) -> Result<Vec<LegislativeDossier>, RepositoryError>;
    async fn find_by_uid(&self, uid: &str) -> Result<Option<LegislativeDossier>, RepositoryError>;
}
