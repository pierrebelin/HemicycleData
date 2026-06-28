use chrono::NaiveDate;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ActeLegislatif {
    pub date: NaiveDate,
    pub libelle: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Score {
    pub avancement: u8,
    pub ampleur: u8,
    pub total: u8,
}

#[derive(Debug, Serialize)]
pub struct DossierLegislatif {
    pub uid: String,
    pub titre: String,
    pub procedure: String,
    pub derniere_activite_date: NaiveDate,
    pub derniere_activite_libelle: String,
    pub actes: Vec<ActeLegislatif>,
    pub score: Score,
}
