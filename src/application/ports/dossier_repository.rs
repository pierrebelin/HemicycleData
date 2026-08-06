use async_trait::async_trait;
use chrono::NaiveDate;

use crate::domain::dossier::{CurationStatus, DossierUid, LegislativeDossier};

pub use super::RepositoryError;

/// Tranche de la liste complète des dossiers.
///
/// `total` est le nombre de dossiers en base, pagination exclue : le visiteur
/// doit voir combien de dossiers existent, pas seulement ceux de sa page
/// (PROJECT.md §2).
#[derive(Debug, Clone)]
pub struct DossierPage {
    pub items: Vec<LegislativeDossier>,
    pub total: i64,
}

#[async_trait]
pub trait DossierRepository: Send + Sync {
    async fn save_all(&self, dossiers: &[LegislativeDossier]) -> Result<usize, RepositoryError>;
    async fn find_recent(&self, since: NaiveDate) -> Result<Vec<LegislativeDossier>, RepositoryError>;
    /// Tous les dossiers, du plus récent au plus ancien. Aucun filtre.
    async fn find_page(&self, limit: i64, offset: i64) -> Result<DossierPage, RepositoryError>;
    async fn find_by_uid(&self, uid: &DossierUid) -> Result<Option<LegislativeDossier>, RepositoryError>;
    async fn find_suggestions(&self, count: usize) -> Result<Vec<LegislativeDossier>, RepositoryError>;
    async fn update_curation_status(&self, uid: &DossierUid, status: CurationStatus) -> Result<bool, RepositoryError>;
}
