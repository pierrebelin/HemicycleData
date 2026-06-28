use crate::application::ports::assembly_source::{AssemblySource, SourceError};
use crate::domain::dossier::LegislativeDossier;

pub struct GetDossierDetail<'a> {
    source: &'a dyn AssemblySource,
}

impl<'a> GetDossierDetail<'a> {
    pub fn new(source: &'a dyn AssemblySource) -> Self {
        Self { source }
    }

    pub async fn execute(&self, uid: &str) -> Result<Option<LegislativeDossier>, SourceError> {
        self.source.fetch_dossier_by_uid(uid).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::NaiveDate;

    use crate::domain::dossier::{LegislativeAct, Score};

    struct FakeSource {
        dossiers: Vec<LegislativeDossier>,
    }

    #[async_trait]
    impl AssemblySource for FakeSource {
        async fn fetch_dossiers_since(
            &self,
            _since: NaiveDate,
        ) -> Result<Vec<LegislativeDossier>, SourceError> {
            unreachable!()
        }

        async fn fetch_dossier_by_uid(
            &self,
            uid: &str,
        ) -> Result<Option<LegislativeDossier>, SourceError> {
            Ok(self.dossiers.iter().find(|d| d.uid == uid).map(|d| {
                LegislativeDossier {
                    uid: d.uid.clone(),
                    title: d.title.clone(),
                    procedure: d.procedure.clone(),
                    last_activity_date: d.last_activity_date,
                    last_activity_label: d.last_activity_label.clone(),
                    acts: d.acts.clone(),
                    score: d.score.clone(),
                }
            }))
        }
    }

    #[tokio::test]
    async fn returns_dossier_when_found() {
        let source = FakeSource {
            dossiers: vec![LegislativeDossier {
                uid: "DLR5L17N12345".into(),
                title: "Projet de loi de finances".into(),
                procedure: "PL".into(),
                last_activity_date: NaiveDate::from_ymd_opt(2026, 6, 25).unwrap(),
                last_activity_label: "Vote solennel".into(),
                acts: vec![
                    LegislativeAct {
                        date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
                        label: "Dépôt".into(),
                    },
                    LegislativeAct {
                        date: NaiveDate::from_ymd_opt(2026, 6, 25).unwrap(),
                        label: "Vote solennel".into(),
                    },
                ],
                score: Score {
                    progress: 9,
                    magnitude: 10,
                    total: 95,
                },
            }],
        };
        let uc = GetDossierDetail::new(&source);
        let result = uc.execute("DLR5L17N12345").await.unwrap();

        assert!(result.is_some());
        let dossier = result.unwrap();
        assert_eq!(dossier.uid, "DLR5L17N12345");
        assert_eq!(dossier.acts.len(), 2);
    }

    #[tokio::test]
    async fn returns_none_when_not_found() {
        let source = FakeSource {
            dossiers: vec![],
        };
        let uc = GetDossierDetail::new(&source);
        let result = uc.execute("UNKNOWN").await.unwrap();
        assert!(result.is_none());
    }
}
