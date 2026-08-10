use sha2::{Digest, Sha256};

use crate::application::ports::dossier_group_actions_repository::{
    DossierGroupActionsRepository, DossierSummaryRepository, RepositoryError, StoredGroupSummary,
    SummaryStatus,
};

#[derive(Debug, Clone)]
pub struct DossierGroupActions {
    pub facts: crate::application::ports::dossier_group_actions_repository::DossierGroupFacts,
    pub facts_fingerprint: String,
    pub summaries: Vec<StoredGroupSummary>,
}

pub struct GetDossierGroupActions<'a> {
    facts: &'a dyn DossierGroupActionsRepository,
    summaries: &'a dyn DossierSummaryRepository,
}

impl<'a> GetDossierGroupActions<'a> {
    pub fn new(
        facts: &'a dyn DossierGroupActionsRepository,
        summaries: &'a dyn DossierSummaryRepository,
    ) -> Self {
        Self { facts, summaries }
    }

    pub async fn execute(
        &self,
        dossier_uid: &str,
    ) -> Result<Option<DossierGroupActions>, RepositoryError> {
        let Some(facts) = self.facts.load_facts(dossier_uid).await? else {
            return Ok(None);
        };
        let facts_fingerprint = fingerprint(&facts);
        let summaries = self
            .summaries
            .summaries_for(dossier_uid)
            .await?
            .into_iter()
            .filter(|summary| {
                summary.status == SummaryStatus::Ready
                    && summary.facts_fingerprint == facts_fingerprint
                    && facts
                        .groups
                        .iter()
                        .any(|group| group.uid == summary.group_uid)
            })
            .collect();

        Ok(Some(DossierGroupActions {
            facts,
            facts_fingerprint,
            summaries,
        }))
    }
}

pub fn fingerprint(
    facts: &crate::application::ports::dossier_group_actions_repository::DossierGroupFacts,
) -> String {
    let bytes = serde_json::to_vec(facts).expect("facts are serializable");
    let digest = Sha256::digest(bytes);
    format!("{digest:x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use std::sync::Mutex;

    use crate::application::ports::dossier_group_actions_repository::{
        DossierGroupFacts, GeneratedGroupSummary, SummarySource,
    };
    use crate::domain::scrutin::VoteTally;

    fn facts() -> DossierGroupFacts {
        DossierGroupFacts {
            dossier_uid: "D1".into(),
            title: "Texte".into(),
            official_url: Some("https://example.test/dossier".into()),
            legislature: 17,
            period_start: Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
            period_end: Some(NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()),
            groups: vec![crate::application::ports::dossier_group_actions_repository::GroupFacts {
                uid: "G1".into(),
                abbrev: "G".into(),
                label: "Groupe".into(),
                color: None,
                start_date: None,
                end_date: None,
                final_votes: vec![crate::application::ports::dossier_group_actions_repository::FinalVoteFact {
                    scrutin_uid: "S1".into(), number: "1".into(),
                    date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(), legislature: 17,
                    subject: "l'ensemble du texte (première lecture)".into(),
                    text_label: "texte".into(), reading: Some("première lecture".into()),
                    outcome_code: "adopted".into(), outcome_label: "Adopté".into(),
                    majority_position: Some("pour".into()), member_count: None,
                    tally: VoteTally::default(),
                }],
                amendments: vec![],
            }],
        }
    }

    struct FakeFacts(DossierGroupFacts);
    #[async_trait]
    impl DossierGroupActionsRepository for FakeFacts {
        async fn load_facts(&self, _: &str) -> Result<Option<DossierGroupFacts>, RepositoryError> {
            Ok(Some(self.0.clone()))
        }
        async fn list_facts(&self, _: usize) -> Result<Vec<DossierGroupFacts>, RepositoryError> {
            Ok(vec![self.0.clone()])
        }
    }

    struct FakeSummaries(Mutex<Vec<StoredGroupSummary>>);
    #[async_trait]
    impl DossierSummaryRepository for FakeSummaries {
        async fn summaries_for(&self, _: &str) -> Result<Vec<StoredGroupSummary>, RepositoryError> {
            Ok(self.0.lock().unwrap().clone())
        }
        async fn mark_pending(
            &self,
            _: &str,
            _: &[String],
            _: &str,
        ) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn save_ready(
            &self,
            _: &str,
            _: &str,
            _: &str,
            _: &str,
            _: &[GeneratedGroupSummary],
        ) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn hides_stale_summaries_but_keeps_raw_facts() {
        let facts = facts();
        let summaries = FakeSummaries(Mutex::new(vec![StoredGroupSummary {
            group_uid: "G1".into(),
            status: SummaryStatus::Ready,
            paragraph: Some("Texte descriptif".into()),
            facts_fingerprint: "old".into(),
            model: Some("model".into()),
            prompt_version: Some("v1".into()),
            generated_at: None,
            sources: vec![SummarySource {
                source_id: "dossier:D1".into(),
                kind: "dossier".into(),
                uid: "D1".into(),
                label: "Dossier".into(),
                official_url: None,
            }],
        }]));
        let result = GetDossierGroupActions::new(&FakeFacts(facts), &summaries)
            .execute("D1")
            .await
            .unwrap()
            .unwrap();
        assert!(result.summaries.is_empty());
        assert_eq!(result.facts.groups.len(), 1);
    }
}
