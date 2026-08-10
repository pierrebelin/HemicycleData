//! Read model officiel utilise par la fiche dossier.

use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;

use crate::domain::scrutin::VoteTally;

pub use super::RepositoryError;

#[derive(Debug, Clone, Serialize)]
pub struct DossierGroupFacts {
    pub dossier_uid: String,
    pub title: String,
    pub official_url: Option<String>,
    pub legislature: u16,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub groups: Vec<GroupFacts>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GroupFacts {
    pub uid: String,
    pub abbrev: String,
    pub label: String,
    pub color: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub final_votes: Vec<FinalVoteFact>,
    pub amendments: Vec<AmendmentFact>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FinalVoteFact {
    pub scrutin_uid: String,
    pub number: String,
    pub date: NaiveDate,
    pub legislature: u16,
    pub subject: String,
    pub text_label: String,
    pub reading: Option<String>,
    pub outcome_code: String,
    pub outcome_label: String,
    pub majority_position: Option<String>,
    pub member_count: Option<u16>,
    pub tally: VoteTally,
}

#[derive(Debug, Clone, Serialize)]
pub struct AmendmentFact {
    pub uid: String,
    pub number: String,
    pub target_title: String,
    pub target_kind: Option<String>,
    pub fate_code: String,
    pub fate_label: String,
    pub deposited_on: Option<NaiveDate>,
    pub summary_available: bool,
}

#[async_trait]
pub trait DossierGroupActionsRepository: Send + Sync {
    async fn load_facts(
        &self,
        dossier_uid: &str,
    ) -> Result<Option<DossierGroupFacts>, RepositoryError>;
    async fn list_facts(&self, limit: usize) -> Result<Vec<DossierGroupFacts>, RepositoryError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SummaryStatus {
    Pending,
    Ready,
}

#[derive(Debug, Clone)]
pub struct SummarySource {
    pub source_id: String,
    pub kind: String,
    pub uid: String,
    pub label: String,
    pub official_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StoredGroupSummary {
    pub group_uid: String,
    pub status: SummaryStatus,
    pub paragraph: Option<String>,
    pub facts_fingerprint: String,
    pub model: Option<String>,
    pub prompt_version: Option<String>,
    pub generated_at: Option<NaiveDateTime>,
    pub sources: Vec<SummarySource>,
}

#[async_trait]
pub trait DossierSummaryRepository: Send + Sync {
    async fn summaries_for(
        &self,
        dossier_uid: &str,
    ) -> Result<Vec<StoredGroupSummary>, RepositoryError>;
    async fn mark_pending(
        &self,
        dossier_uid: &str,
        group_uids: &[String],
        facts_fingerprint: &str,
    ) -> Result<(), RepositoryError>;
    async fn save_ready(
        &self,
        dossier_uid: &str,
        facts_fingerprint: &str,
        model: &str,
        prompt_version: &str,
        summaries: &[GeneratedGroupSummary],
    ) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct GeneratedGroupSummary {
    pub group_uid: String,
    pub paragraph: String,
    pub sources: Vec<SummarySource>,
}
