use async_trait::async_trait;

use crate::domain::theme::ProposedFamily;

#[derive(Debug, thiserror::Error)]
pub enum ClassifierError {
    #[error("classifier unavailable: {0}")]
    Unavailable(String),
    #[error("classifier call failed: {0}")]
    Call(String),
    #[error("classifier answer unusable: {0}")]
    Answer(String),
}

/// Port d'effet: le modele propose des familles pour un libelle de texte.
///
/// Il ne recoit que le libelle (RM-04) et ne rend ni note, ni rang, ni
/// decompte (RM-10). Une liste vide = aucune famille retenue, ce qui n'est pas
/// une erreur: le texte reste consultable, non rattache (RM-01).
#[async_trait]
pub trait ThemeClassifier: Send + Sync {
    async fn propose(&self, text_label: &str) -> Result<Vec<ProposedFamily>, ClassifierError>;

    /// Modele interroge, conserve avec la proposition pour la rendre inspectable.
    fn model(&self) -> &str;

    /// Version de l'instruction, conservee pour la meme raison.
    fn prompt_version(&self) -> &str;
}
