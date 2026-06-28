use chrono::NaiveDate;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DossierLegislatif {
    pub uid: String,
    pub titre: String,
    pub procedure: String,
    pub derniere_activite_date: NaiveDate,
    pub derniere_activite_libelle: String,
}
