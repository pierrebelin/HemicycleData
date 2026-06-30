use async_trait::async_trait;
use chrono::NaiveDate;

use crate::domain::dossier::{CurationStatus, DossierUid, LegislativeDossier};

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    Database(String),
}

#[async_trait]
pub trait DossierRepository: Send + Sync {
    async fn save_all(&self, dossiers: &[LegislativeDossier]) -> Result<usize, RepositoryError>;
    async fn find_recent(&self, since: NaiveDate) -> Result<Vec<LegislativeDossier>, RepositoryError>;
    async fn find_by_uid(&self, uid: &DossierUid) -> Result<Option<LegislativeDossier>, RepositoryError>;
    async fn find_suggestions(&self, count: usize) -> Result<Vec<LegislativeDossier>, RepositoryError>;
    async fn update_curation_status(&self, uid: &DossierUid, status: CurationStatus) -> Result<bool, RepositoryError>;
}
