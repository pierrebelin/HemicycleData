use std::collections::HashMap;

use async_trait::async_trait;
use chrono::NaiveDate;

use crate::domain::theme::{
    AssignmentOrigin, DebatedText, FamilyCode, SubjectRef, TextKey, ThemeAssignment, ThemeProposal,
};

pub use super::RepositoryError;

/// Objet d'un scrutin, matiere premiere de l'extraction (CU-01).
#[derive(Debug, Clone)]
pub struct ScrutinSubject {
    pub uid: String,
    pub subject: String,
}

/// Lien scrutin -> texte, produit par l'extraction.
#[derive(Debug, Clone)]
pub struct TextLink {
    pub scrutin_uid: String,
    pub text_key: String,
}

/// Issue de la derniere tentative de proposition sur un texte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptOutcome {
    /// Le modele a rendu au moins une famille.
    Proposed,
    /// Le modele a repondu sans retenir de famille.
    NoFamily,
    /// Le modele n'a pas repondu.
    Failed,
}

impl AttemptOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::NoFamily => "no_family",
            Self::Failed => "failed",
        }
    }
}

/// Famille courante d'un objet, telle qu'affichee (RM-09).
#[derive(Debug, Clone)]
pub struct AssignedFamily {
    pub family: FamilyCode,
    pub origin: AssignmentOrigin,
    pub opened_on: NaiveDate,
    pub motive: Option<String>,
}

/// Ligne de liste d'un texte debattu.
#[derive(Debug, Clone)]
pub struct TextSummary {
    pub key: String,
    pub label: String,
    pub scrutin_count: i64,
    pub first_vote: Option<NaiveDate>,
    pub last_vote: Option<NaiveDate>,
    pub dossier_uid: Option<String>,
    pub dossier_label: Option<String>,
    pub families: Vec<AssignedFamily>,
    pub last_attempt_outcome: Option<String>,
}

/// Ligne de scrutin sur la fiche d'un texte. Les chiffres viennent de la base
/// (RM-10); le detail complet reste sur la page du scrutin.
#[derive(Debug, Clone)]
pub struct TextScrutin {
    pub uid: String,
    pub number: String,
    pub date: NaiveDate,
    pub subject: String,
    pub outcome_label: String,
    pub votes_for: i16,
    pub votes_against: i16,
    pub abstentions: i16,
}

#[derive(Debug, Clone)]
pub struct TextPage {
    pub items: Vec<TextSummary>,
    pub total: i64,
}

/// Couverture d'une famille. Chiffres lus en base, jamais produits par le
/// modele (RM-10).
#[derive(Debug, Clone)]
pub struct FamilyCoverage {
    pub family: FamilyCode,
    pub text_count: i64,
    pub scrutin_count: i64,
    pub arbitrated_text_count: i64,
}

/// Ce que la page methode publie (CU-06).
#[derive(Debug, Clone)]
pub struct MethodReport {
    pub families: Vec<FamilyCoverage>,
    pub texts_total: i64,
    pub texts_assigned: i64,
    pub texts_arbitrated: i64,
    pub texts_awaiting_arbitration: i64,
    pub texts_without_family: i64,
    pub texts_attempt_failed: i64,
    pub texts_never_attempted: i64,
    pub scrutins_total: i64,
    pub scrutins_with_text: i64,
    pub scrutins_assigned: i64,
    pub dossiers_total: i64,
    pub dossiers_linked_to_text: i64,
    pub dossiers_assigned: i64,
}

#[async_trait]
pub trait ThemeRepository: Send + Sync {
    // -- Extraction (CU-01) ------------------------------------------------

    async fn scrutin_subjects(&self) -> Result<Vec<ScrutinSubject>, RepositoryError>;

    async fn save_texts(&self, texts: &[DebatedText]) -> Result<usize, RepositoryError>;

    async fn link_scrutins(&self, links: &[TextLink]) -> Result<usize, RepositoryError>;

    /// Relie chaque dossier au texte que ses propres scrutins nomment. Le lien
    /// vient des donnees publiees, jamais d'un rapprochement de libelles
    /// (RM-06). Rend le nombre de dossiers relies.
    async fn link_dossiers_through_scrutins(&self) -> Result<usize, RepositoryError>;

    // -- Proposition (CU-02) -----------------------------------------------

    /// Textes sans rattachement courant et jamais proposes avec succes, du plus
    /// vote au moins vote: le travail utile d'abord.
    async fn texts_awaiting_proposal(&self, limit: i64)
        -> Result<Vec<DebatedText>, RepositoryError>;

    async fn record_attempt(
        &self,
        key: &TextKey,
        on: NaiveDate,
        outcome: AttemptOutcome,
    ) -> Result<(), RepositoryError>;

    async fn save_proposal(&self, proposal: &ThemeProposal) -> Result<(), RepositoryError>;

    async fn latest_proposal(
        &self,
        subject: &SubjectRef,
    ) -> Result<Option<ThemeProposal>, RepositoryError>;

    // -- Rattachements (CU-02, CU-03) --------------------------------------

    /// Clot tous les rattachements courants de l'objet a `closed_on`, puis
    /// ouvre ceux fournis. Une seule transaction: l'objet n'est jamais sans
    /// etat lisible (RM-07).
    async fn replace_assignments(
        &self,
        subject: &SubjectRef,
        closed_on: NaiveDate,
        opened: &[ThemeAssignment],
    ) -> Result<(), RepositoryError>;

    async fn assignment_history(
        &self,
        subject: &SubjectRef,
    ) -> Result<Vec<ThemeAssignment>, RepositoryError>;

    // -- Lecture (CU-04, CU-05, CU-06) -------------------------------------

    async fn text_by_key(&self, key: &TextKey) -> Result<Option<TextSummary>, RepositoryError>;

    /// Scrutins portant ce texte, du plus recent au plus ancien.
    async fn scrutins_of_text(
        &self,
        key: &TextKey,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TextScrutin>, RepositoryError>;

    async fn texts_by_family(
        &self,
        family: FamilyCode,
        limit: i64,
        offset: i64,
    ) -> Result<TextPage, RepositoryError>;

    /// Textes sans aucune famille courante (RM-01).
    async fn unassigned_texts(&self, limit: i64, offset: i64) -> Result<TextPage, RepositoryError>;

    /// Familles courantes des scrutins demandes, heritees de leur texte (RM-06).
    async fn families_of_scrutins(
        &self,
        scrutin_uids: &[String],
    ) -> Result<HashMap<String, Vec<AssignedFamily>>, RepositoryError>;

    /// Familles courantes d'un dossier: celles du texte que ses scrutins
    /// nomment, a defaut son propre rattachement (RM-06).
    async fn families_of_dossier(
        &self,
        dossier_uid: &str,
    ) -> Result<Vec<AssignedFamily>, RepositoryError>;

    async fn method_report(&self) -> Result<MethodReport, RepositoryError>;
}
