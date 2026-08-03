pub mod actor_repository;
pub mod actor_source;
pub mod assembly_source;
pub mod dossier_repository;
pub mod scrutin_repository;
pub mod scrutin_source;

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("download failed: {0}")]
    Download(String),
    #[error("parse failed: {0}")]
    Parse(String),
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    Database(String),
}
