use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::domain::dossier::DossierLegislatif;

#[derive(Deserialize)]
pub struct RecentActivityQuery {
    #[serde(default = "default_days")]
    pub days: u32,
}

fn default_days() -> u32 {
    7
}

#[derive(Serialize)]
pub struct DossierDto {
    pub uid: String,
    pub titre: String,
    pub procedure: String,
    pub derniere_activite_date: NaiveDate,
    pub derniere_activite_libelle: String,
    pub score_total: u8,
}

impl From<DossierLegislatif> for DossierDto {
    fn from(d: DossierLegislatif) -> Self {
        Self {
            uid: d.uid,
            titre: d.titre,
            procedure: d.procedure,
            derniere_activite_date: d.derniere_activite_date,
            derniere_activite_libelle: d.derniere_activite_libelle,
            score_total: d.score.total,
        }
    }
}

#[derive(Serialize)]
pub struct RecentDossiersResponse {
    pub count: usize,
    pub dossiers: Vec<DossierDto>,
}

#[derive(Serialize)]
pub struct ActeDto {
    pub date: NaiveDate,
    pub libelle: String,
}

#[derive(Serialize)]
pub struct ScoreDto {
    pub avancement: u8,
    pub ampleur: u8,
    pub total: u8,
}

#[derive(Serialize)]
pub struct DossierDetailDto {
    pub uid: String,
    pub titre: String,
    pub procedure: String,
    pub derniere_activite_date: NaiveDate,
    pub derniere_activite_libelle: String,
    pub actes: Vec<ActeDto>,
    pub score: ScoreDto,
}

impl From<DossierLegislatif> for DossierDetailDto {
    fn from(d: DossierLegislatif) -> Self {
        Self {
            uid: d.uid,
            titre: d.titre,
            procedure: d.procedure,
            derniere_activite_date: d.derniere_activite_date,
            derniere_activite_libelle: d.derniere_activite_libelle,
            actes: d
                .actes
                .into_iter()
                .map(|a| ActeDto {
                    date: a.date,
                    libelle: a.libelle,
                })
                .collect(),
            score: ScoreDto {
                avancement: d.score.avancement,
                ampleur: d.score.ampleur,
                total: d.score.total,
            },
        }
    }
}
