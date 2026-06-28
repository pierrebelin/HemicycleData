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

    use crate::domain::scoring::compute_score;

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
                    actes: d.actes.clone(),
                    score: d.score.clone(),
                })
                .collect())
        }

        async fn fetch_dossier_by_uid(
            &self,
            _uid: &str,
        ) -> Result<Option<DossierLegislatif>, SourceError> {
            unreachable!()
        }
    }

    fn make_dossier(uid: &str, titre: &str, procedure: &str, date: NaiveDate, libelle: &str) -> DossierLegislatif {
        let score = compute_score(titre, libelle);
        DossierLegislatif {
            uid: uid.into(),
            titre: titre.into(),
            procedure: procedure.into(),
            derniere_activite_date: date,
            derniere_activite_libelle: libelle.into(),
            actes: vec![],
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
