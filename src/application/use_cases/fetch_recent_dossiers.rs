use chrono::Utc;

use crate::application::ports::assemblee_source::{AssembleeSource, SourceError};
use crate::domain::dossier::DossierLegislatif;

pub struct FetchRecentDossiers<'a> {
    source: &'a dyn AssembleeSource,
}

impl<'a> FetchRecentDossiers<'a> {
    pub fn new(source: &'a dyn AssembleeSource) -> Self {
        Self { source }
    }

    pub async fn execute(&self, days: u32) -> Result<Vec<DossierLegislatif>, SourceError> {
        let since = Utc::now().date_naive() - chrono::Duration::days(days as i64);
        let mut dossiers = self.source.fetch_dossiers_since(since).await?;
        dossiers.sort_by(|a, b| b.derniere_activite_date.cmp(&a.derniere_activite_date));
        Ok(dossiers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::NaiveDate;

    struct FakeSource {
        dossiers: Vec<DossierLegislatif>,
    }

    #[async_trait]
    impl AssembleeSource for FakeSource {
        async fn fetch_dossiers_since(
            &self,
            since: NaiveDate,
        ) -> Result<Vec<DossierLegislatif>, SourceError> {
            Ok(self
                .dossiers
                .iter()
                .filter(|d| d.derniere_activite_date >= since)
                .map(|d| DossierLegislatif {
                    uid: d.uid.clone(),
                    titre: d.titre.clone(),
                    procedure: d.procedure.clone(),
                    derniere_activite_date: d.derniere_activite_date,
                    derniere_activite_libelle: d.derniere_activite_libelle.clone(),
                })
                .collect())
        }
    }

    #[tokio::test]
    async fn returns_dossiers_sorted_by_date_desc() {
        let source = FakeSource {
            dossiers: vec![
                DossierLegislatif {
                    uid: "D1".into(),
                    titre: "Ancien".into(),
                    procedure: "PL".into(),
                    derniere_activite_date: NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
                    derniere_activite_libelle: "Dépôt".into(),
                },
                DossierLegislatif {
                    uid: "D2".into(),
                    titre: "Récent".into(),
                    procedure: "PPL".into(),
                    derniere_activite_date: NaiveDate::from_ymd_opt(2026, 6, 27).unwrap(),
                    derniere_activite_libelle: "Vote".into(),
                },
            ],
        };
        let uc = FetchRecentDossiers::new(&source);
        let result = uc.execute(365).await.unwrap();

        assert_eq!(result[0].uid, "D2");
        assert_eq!(result[1].uid, "D1");
    }
}
