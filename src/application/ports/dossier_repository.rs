use std::collections::HashMap;

use async_trait::async_trait;
use chrono::NaiveDate;

use crate::domain::dossier::{CurationStatus, DossierUid, LegislativeDossier};

pub use super::RepositoryError;

/// Ce que la base sait deja d'un dossier, lu avant toute reecriture.
///
/// Le rafraichissement compare cet etat a celui de la source pour n'ecrire
/// que ce qui a bouge : reecrire les 3 035 dossiers coute une quinzaine
/// d'allers-retours chacun sur une base serverless.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredDossierState {
    pub last_activity_date: NaiveDate,
    pub act_count: usize,
    /// Sort qui ne peut plus changer (promulgation, retrait, fusion).
    pub outcome_is_final: bool,
}

/// Tranche de la liste complète des dossiers.
///
/// `total` est le nombre de dossiers en base, pagination exclue : le visiteur
/// doit voir combien de dossiers existent, pas seulement ceux de sa page
/// (README.md §2).
#[derive(Debug, Clone)]
pub struct DossierPage {
    pub items: Vec<LegislativeDossier>,
    pub total: i64,
}

#[async_trait]
pub trait DossierRepository: Send + Sync {
    async fn save_all(&self, dossiers: &[LegislativeDossier]) -> Result<usize, RepositoryError>;
    /// Etat stocke de tous les dossiers, indexe par uid. Une seule requete.
    async fn load_states(&self) -> Result<HashMap<String, StoredDossierState>, RepositoryError>;
    async fn find_recent(&self, since: NaiveDate) -> Result<Vec<LegislativeDossier>, RepositoryError>;
    /// Tous les dossiers, du plus récent au plus ancien. Aucun filtre.
    async fn find_page(&self, limit: i64, offset: i64) -> Result<DossierPage, RepositoryError>;
    async fn find_by_uid(&self, uid: &DossierUid) -> Result<Option<LegislativeDossier>, RepositoryError>;
    async fn find_suggestions(&self, count: usize) -> Result<Vec<LegislativeDossier>, RepositoryError>;
    async fn update_curation_status(&self, uid: &DossierUid, status: CurationStatus) -> Result<bool, RepositoryError>;
}
