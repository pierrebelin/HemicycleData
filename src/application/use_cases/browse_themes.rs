use std::collections::HashMap;

use crate::application::ports::theme_repository::{
    AssignedFamily, MethodReport, RepositoryError, TextPage, TextSummary, ThemeRepository,
};
use crate::domain::theme::{FamilyCode, SubjectRef, TextKey, ThemeAssignment, ThemeProposal};

/// Une famille et son perimetre publie, tels qu'affiches sur le site.
#[derive(Debug, Clone)]
pub struct FamilyDescription {
    pub code: FamilyCode,
    pub label: &'static str,
    pub scope: &'static str,
}

/// Detail d'un texte: ses familles courantes, son historique, la proposition
/// qui l'a produit.
#[derive(Debug, Clone)]
pub struct TextDetail {
    pub summary: TextSummary,
    pub history: Vec<ThemeAssignment>,
    pub proposal: Option<ThemeProposal>,
}

/// CU-04, CU-05, CU-06 — Parcours par famille, non rattaches, methode.
pub struct BrowseThemes<'a> {
    repository: &'a dyn ThemeRepository,
}

impl<'a> BrowseThemes<'a> {
    pub fn new(repository: &'a dyn ThemeRepository) -> Self {
        Self { repository }
    }

    /// Referentiel ferme, servi tel quel: le site publie les huit familles et
    /// leur perimetre (RM-08, CU-06).
    pub fn families(&self) -> Vec<FamilyDescription> {
        FamilyCode::ALL
            .into_iter()
            .map(|code| FamilyDescription {
                code,
                label: code.label(),
                scope: code.scope(),
            })
            .collect()
    }

    pub async fn texts_of_family(
        &self,
        family: FamilyCode,
        limit: i64,
        offset: i64,
    ) -> Result<TextPage, RepositoryError> {
        self.repository.texts_by_family(family, limit, offset).await
    }

    /// CU-05 — Les non rattaches restent atteignables depuis toute page de
    /// theme (RM-01).
    pub async fn unassigned_texts(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<TextPage, RepositoryError> {
        self.repository.unassigned_texts(limit, offset).await
    }

    pub async fn text_detail(&self, key: &TextKey) -> Result<Option<TextDetail>, RepositoryError> {
        let Some(summary) = self.repository.text_by_key(key).await? else {
            return Ok(None);
        };
        let subject = SubjectRef::Text(key.clone());
        let history = self.repository.assignment_history(&subject).await?;
        let proposal = self.repository.latest_proposal(&subject).await?;
        Ok(Some(TextDetail {
            summary,
            history,
            proposal,
        }))
    }

    pub async fn families_of_scrutins(
        &self,
        scrutin_uids: &[String],
    ) -> Result<HashMap<String, Vec<AssignedFamily>>, RepositoryError> {
        if scrutin_uids.is_empty() {
            return Ok(HashMap::new());
        }
        self.repository.families_of_scrutins(scrutin_uids).await
    }

    pub async fn families_of_dossier(
        &self,
        dossier_uid: &str,
    ) -> Result<Vec<AssignedFamily>, RepositoryError> {
        self.repository.families_of_dossier(dossier_uid).await
    }

    pub async fn method(&self) -> Result<MethodReport, RepositoryError> {
        self.repository.method_report().await
    }
}
