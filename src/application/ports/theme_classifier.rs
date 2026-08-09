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

/// Port d'effet: le modele propose des familles pour des libelles de texte.
///
/// Il ne recoit que les libelles (RM-04) et ne rend ni note, ni rang, ni
/// decompte (RM-10).
///
/// Le port est **par lot** et non par texte: le cadrage envoye au modele — le
/// referentiel des familles et les regles de rattachement — pese bien plus
/// lourd qu'un libelle de loi. Le soumettre une fois pour vingt libelles plutot
/// que vingt fois pour un divise d'autant le cout fixe (RM-14).
#[async_trait]
pub trait ThemeClassifier: Send + Sync {
    /// Rend une entree par libelle soumis, dans l'ordre des libelles.
    ///
    /// - `Some(familles)` — le modele a repondu pour ce libelle. Une liste vide
    ///   signifie qu'il n'a retenu aucune famille, ce qui n'est pas une erreur:
    ///   le texte reste consultable, non rattache (RM-01).
    /// - `None` — le modele n'a rien rendu pour ce libelle. Le texte sera
    ///   repris a la passe suivante.
    ///
    /// `Err` porte sur l'appel entier: aucun libelle du lot n'a de reponse.
    async fn propose_batch(
        &self,
        labels: &[String],
    ) -> Result<Vec<Option<Vec<ProposedFamily>>>, ClassifierError>;

    /// Modele interroge, conserve avec la proposition pour la rendre inspectable.
    fn model(&self) -> &str;

    /// Version de l'instruction, conservee pour la meme raison.
    fn prompt_version(&self) -> &str;

    /// Libelles soumis en un appel. Le use case decoupe ses lots dessus.
    fn batch_size(&self) -> usize;
}
