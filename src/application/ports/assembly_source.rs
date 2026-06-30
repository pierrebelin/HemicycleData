use async_trait::async_trait;
use chrono::NaiveDate;

use crate::domain::dossier::LegislativeDossier;

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("download failed: {0}")]
    Download(String),
    #[error("parse failed: {0}")]
    Parse(String),
}

#[async_trait]
pub trait AssemblySource: Send + Sync {
    async fn fetch_dossiers_since(
        &self,
        since: NaiveDate,
    ) -> Result<Vec<LegislativeDossier>, SourceError>;

    async fn fetch_dossier_by_uid(
        &self,
        uid: &str,
    ) -> Result<Option<LegislativeDossier>, SourceError>;

    async fn fetch_dossiers_since_with_refs(
        &self,
        since: NaiveDate,
    ) -> Result<Vec<(LegislativeDossier, Vec<String>)>, SourceError>;

    async fn fetch_dossier_by_uid_with_refs(
        &self,
        uid: &str,
    ) -> Result<Option<(LegislativeDossier, Vec<String>)>, SourceError>;
}
