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
}

impl From<DossierLegislatif> for DossierDto {
    fn from(d: DossierLegislatif) -> Self {
        Self {
            uid: d.uid,
            titre: d.titre,
            procedure: d.procedure,
            derniere_activite_date: d.derniere_activite_date,
            derniere_activite_libelle: d.derniere_activite_libelle,
        }
    }
}

#[derive(Serialize)]
pub struct RecentDossiersResponse {
    pub count: usize,
    pub dossiers: Vec<DossierDto>,
}
