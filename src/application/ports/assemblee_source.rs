use async_trait::async_trait;
use chrono::NaiveDate;

use crate::domain::dossier::DossierLegislatif;

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("download failed: {0}")]
    Download(String),
    #[error("parse failed: {0}")]
    Parse(String),
}

#[async_trait]
pub trait AssembleeSource: Send + Sync {
    async fn fetch_dossiers_since(
        &self,
        since: NaiveDate,
    ) -> Result<Vec<DossierLegislatif>, SourceError>;
}
