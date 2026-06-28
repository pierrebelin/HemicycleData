use chrono::Utc;

use crate::application::ports::assembly_source::{AssemblySource, SourceError};
use crate::domain::dossier::LegislativeDossier;

pub struct FetchRecentDossiers<'a> {
    source: &'a dyn AssemblySource,
}

impl<'a> FetchRecentDossiers<'a> {
    pub fn new(source: &'a dyn AssemblySource) -> Self {
        Self { source }
    }

    pub async fn execute(&self, days: u32) -> Result<Vec<LegislativeDossier>, SourceError> {
        let since = Utc::now().date_naive() - chrono::Duration::days(days as i64);
        let mut dossiers = self.source.fetch_dossiers_since(since).await?;
        dossiers.sort_by(|a, b| b.last_activity_date.cmp(&a.last_activity_date));
        Ok(dossiers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::NaiveDate;

    use crate::domain::scoring::compute_score;

    struct FakeSource {
        dossiers: Vec<LegislativeDossier>,
    }

    #[async_trait]
    impl AssemblySource for FakeSource {
        async fn fetch_dossiers_since(
            &self,
            since: NaiveDate,
        ) -> Result<Vec<LegislativeDossier>, SourceError> {
            Ok(self
                .dossiers
                .iter()
                .filter(|d| d.last_activity_date >= since)
                .map(|d| LegislativeDossier {
                    uid: d.uid.clone(),
                    title: d.title.clone(),
                    procedure: d.procedure.clone(),
                    last_activity_date: d.last_activity_date,
                    last_activity_label: d.last_activity_label.clone(),
                    acts: d.acts.clone(),
                    score: d.score.clone(),
                })
                .collect())
        }

        async fn fetch_dossier_by_uid(
            &self,
            _uid: &str,
        ) -> Result<Option<LegislativeDossier>, SourceError> {
            unreachable!()
        }
    }

    fn make_dossier(uid: &str, title: &str, procedure: &str, date: NaiveDate, label: &str) -> LegislativeDossier {
        let score = compute_score(title, label);
        LegislativeDossier {
            uid: uid.into(),
            title: title.into(),
            procedure: procedure.into(),
            last_activity_date: date,
            last_activity_label: label.into(),
            acts: vec![],
            score,
        }
    }

    #[tokio::test]
    async fn returns_dossiers_sorted_by_date_desc() {
        let source = FakeSource {
            dossiers: vec![
                make_dossier("D1", "Ancien", "PL", NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(), "Dépôt"),
                make_dossier("D2", "Récent", "PPL", NaiveDate::from_ymd_opt(2026, 6, 27).unwrap(), "Vote"),
            ],
        };
        let uc = FetchRecentDossiers::new(&source);
        let result = uc.execute(365).await.unwrap();

        assert_eq!(result[0].uid, "D2");
        assert_eq!(result[1].uid, "D1");
    }
}
