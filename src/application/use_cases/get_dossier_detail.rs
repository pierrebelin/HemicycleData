use crate::application::ports::assemblee_source::{AssembleeSource, SourceError};
use crate::domain::dossier::DossierLegislatif;

pub struct GetDossierDetail<'a> {
    source: &'a dyn AssembleeSource,
}

impl<'a> GetDossierDetail<'a> {
    pub fn new(source: &'a dyn AssembleeSource) -> Self {
        Self { source }
    }

    pub async fn execute(&self, uid: &str) -> Result<Option<DossierLegislatif>, SourceError> {
        self.source.fetch_dossier_by_uid(uid).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::NaiveDate;

    use crate::domain::dossier::{ActeLegislatif, Score};

    struct FakeSource {
        dossiers: Vec<DossierLegislatif>,
    }

    #[async_trait]
    impl AssembleeSource for FakeSource {
        async fn fetch_dossiers_since(
            &self,
            _since: NaiveDate,
        ) -> Result<Vec<DossierLegislatif>, SourceError> {
            unreachable!()
        }

        async fn fetch_dossier_by_uid(
            &self,
            uid: &str,
        ) -> Result<Option<DossierLegislatif>, SourceError> {
            Ok(self.dossiers.iter().find(|d| d.uid == uid).map(|d| {
                DossierLegislatif {
                    uid: d.uid.clone(),
                    titre: d.titre.clone(),
                    procedure: d.procedure.clone(),
                    derniere_activite_date: d.derniere_activite_date,
                    derniere_activite_libelle: d.derniere_activite_libelle.clone(),
                    actes: d.actes.clone(),
                    score: d.score.clone(),
                }
            }))
        }
    }

    #[tokio::test]
    async fn returns_dossier_when_found() {
        let source = FakeSource {
            dossiers: vec![DossierLegislatif {
                uid: "DLR5L17N12345".into(),
                titre: "Projet de loi de finances".into(),
                procedure: "PL".into(),
                derniere_activite_date: NaiveDate::from_ymd_opt(2026, 6, 25).unwrap(),
                derniere_activite_libelle: "Vote solennel".into(),
                actes: vec![
                    ActeLegislatif {
                        date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
                        libelle: "Dépôt".into(),
                    },
                    ActeLegislatif {
                        date: NaiveDate::from_ymd_opt(2026, 6, 25).unwrap(),
                        libelle: "Vote solennel".into(),
                    },
                ],
                score: Score {
                    avancement: 9,
                    ampleur: 10,
                    total: 95,
                },
            }],
        };
        let uc = GetDossierDetail::new(&source);
        let result = uc.execute("DLR5L17N12345").await.unwrap();

        assert!(result.is_some());
        let dossier = result.unwrap();
        assert_eq!(dossier.uid, "DLR5L17N12345");
        assert_eq!(dossier.actes.len(), 2);
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
