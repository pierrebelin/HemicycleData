use crate::application::ports::scrutin_repository::{
    RepositoryError, ScrutinFilter, ScrutinPage, ScrutinRepository, ScrutinSummary,
};

/// CU-02 — Consulter la liste des scrutins.
///
/// Les filtres restreignent l'affichage demande, jamais le contenu de la base
/// (RM-01). Le total renvoye est celui du filtre: le visiteur voit combien de
/// scrutins il regarde.
pub struct ListScrutins<'a> {
    repository: &'a dyn ScrutinRepository,
}

impl<'a> ListScrutins<'a> {
    pub fn new(repository: &'a dyn ScrutinRepository) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, filter: ScrutinFilter) -> Result<ScrutinPage, RepositoryError> {
        self.repository.list(&filter).await
    }

    /// CU-04 — Scrutins d'un dossier.
    pub async fn for_dossier(
        &self,
        dossier_uid: &str,
    ) -> Result<Vec<ScrutinSummary>, RepositoryError> {
        self.repository.by_dossier(dossier_uid).await
    }
}
