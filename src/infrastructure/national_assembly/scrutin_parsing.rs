//! Formes brutes du jeu de donnees des scrutins.
//!
//! Le JSON est une conversion mecanique du XML de l'Assemblee: les nombres sont
//! des chaines, un element unique remplace un tableau d'un element, et un
//! element vide devient `null` au milieu d'un tableau. Ces formes sont absorbees
//! ici plutot que dans le domaine.

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(Box<T>),
    Many(Vec<T>),
}

impl<T> OneOrMany<T> {
    pub fn into_vec(self) -> Vec<T> {
        match self {
            Self::One(item) => vec![*item],
            Self::Many(items) => items,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RawScrutinWrapper {
    pub scrutin: RawScrutin,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawScrutin {
    pub uid: String,
    pub numero: String,
    pub legislature: Option<String>,
    pub session_ref: Option<String>,
    pub seance_ref: Option<String>,
    pub date_scrutin: Option<String>,
    pub type_vote: Option<RawTypeVote>,
    pub sort: Option<RawSort>,
    pub demandeur: Option<RawDemandeur>,
    pub objet: Option<RawObjet>,
    pub synthese_vote: Option<RawSynthese>,
    pub ventilation_votes: Option<RawVentilation>,
    pub mise_au_point: Option<RawMiseAuPoint>,
    pub lieu_vote: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawTypeVote {
    pub code_type_vote: Option<String>,
    pub libelle_type_vote: Option<String>,
    pub type_majorite: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawSort {
    pub code: Option<String>,
    pub libelle: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawDemandeur {
    pub texte: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawObjet {
    pub libelle: Option<String>,
    pub dossier_legislatif: Option<RawDossierRef>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawDossierRef {
    pub dossier_ref: Option<String>,
    pub libelle: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawSynthese {
    pub nombre_votants: Option<String>,
    pub suffrages_exprimes: Option<String>,
    pub nbr_suffrages_requis: Option<String>,
    pub annonce: Option<String>,
    pub decompte: Option<RawDecompte>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawDecompte {
    pub pour: Option<String>,
    pub contre: Option<String>,
    pub abstentions: Option<String>,
    pub non_votants: Option<String>,
    pub non_votants_volontaires: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawVentilation {
    pub organe: Option<RawVentilationOrgane>,
}

#[derive(Debug, Deserialize)]
pub struct RawVentilationOrgane {
    pub groupes: Option<RawGroupes>,
}

#[derive(Debug, Deserialize)]
pub struct RawGroupes {
    pub groupe: Option<OneOrMany<RawGroupe>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawGroupe {
    pub organe_ref: String,
    pub nombre_membres_groupe: Option<String>,
    pub vote: Option<RawVote>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawVote {
    pub position_majoritaire: Option<String>,
    pub decompte_voix: Option<RawDecompte>,
    pub decompte_nominatif: Option<RawNominatif>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawNominatif {
    pub pours: Option<RawVotantBlock>,
    pub contres: Option<RawVotantBlock>,
    pub abstentions: Option<RawVotantBlock>,
    pub non_votants: Option<RawVotantBlock>,
}

#[derive(Debug, Deserialize)]
pub struct RawVotantBlock {
    pub votant: Option<OneOrMany<RawVotant>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawVotant {
    pub acteur_ref: String,
    pub par_delegation: Option<String>,
    pub num_place: Option<String>,
    pub cause_position_vote: Option<String>,
}

/// Mises au point. Les valeurs prennent toutes les formes que produit la
/// conversion XML — d'ou `Value` et une lecture manuelle.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawMiseAuPoint {
    pub pours: Option<Value>,
    pub contres: Option<Value>,
    pub abstentions: Option<Value>,
    pub non_votants: Option<Value>,
    pub non_votants_volontaires: Option<Value>,
    pub dysfonctionnement: Option<RawDysfonctionnement>,
}

/// Meme contenu que la mise au point, cle au singulier a la source.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawDysfonctionnement {
    pub pour: Option<Value>,
    pub contre: Option<Value>,
    pub abstentions: Option<Value>,
    pub non_votants: Option<Value>,
    pub non_votants_volontaires: Option<Value>,
}

/// Extrait les votants d'une valeur de mise au point.
///
/// Accepte `null`, un objet `{ votant: ... }`, un votant nu, et les tableaux
/// contenant tout cela — y compris les `null` intercalaires du XML.
pub fn votants_in(value: Option<&Value>) -> Vec<RawVotant> {
    let mut out = Vec::new();
    collect_votants(value, &mut out);
    out
}

fn collect_votants(value: Option<&Value>, out: &mut Vec<RawVotant>) {
    let Some(value) = value else { return };
    match value {
        Value::Array(items) => {
            for item in items {
                collect_votants(Some(item), out);
            }
        }
        Value::Object(map) => {
            if let Some(inner) = map.get("votant") {
                collect_votants(Some(inner), out);
            } else if map.contains_key("acteurRef") {
                if let Ok(votant) = serde_json::from_value::<RawVotant>(value.clone()) {
                    out.push(votant);
                }
            }
        }
        _ => {}
    }
}

pub fn count(raw: Option<&String>) -> u16 {
    raw.and_then(|v| v.trim().parse::<u16>().ok()).unwrap_or(0)
}

pub fn optional_count(raw: Option<&String>) -> Option<u16> {
    raw.and_then(|v| v.trim().parse::<u16>().ok())
}

pub fn non_empty(raw: Option<String>) -> Option<String> {
    raw.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

pub fn is_true(raw: Option<&String>) -> bool {
    matches!(raw.map(|v| v.trim()), Some("true") | Some("1"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_single_element_as_a_list() {
        let json = r#"{"votant": {"acteurRef": "PA1", "parDelegation": "false"}}"#;
        let block: RawVotantBlock = serde_json::from_str(json).unwrap();
        let votants = block.votant.unwrap().into_vec();
        assert_eq!(votants.len(), 1);
        assert_eq!(votants[0].acteur_ref, "PA1");
    }

    #[test]
    fn reads_a_list_of_elements() {
        let json = r#"{"votant": [{"acteurRef": "PA1"}, {"acteurRef": "PA2"}]}"#;
        let block: RawVotantBlock = serde_json::from_str(json).unwrap();
        assert_eq!(block.votant.unwrap().into_vec().len(), 2);
    }

    #[test]
    fn walks_the_null_padded_arrays_of_a_mise_au_point() {
        let json = r#"{
            "nonVotants": [null, {"votant": {"acteurRef": "PA795982", "numPlace": "558"}}],
            "pours": {"votant": {"acteurRef": "PA841947"}},
            "abstentions": [null, null],
            "contres": null
        }"#;
        let raw: RawMiseAuPoint = serde_json::from_str(json).unwrap();

        let non_votants = votants_in(raw.non_votants.as_ref());
        assert_eq!(non_votants.len(), 1);
        assert_eq!(non_votants[0].acteur_ref, "PA795982");

        assert_eq!(votants_in(raw.pours.as_ref()).len(), 1);
        assert!(votants_in(raw.abstentions.as_ref()).is_empty());
        assert!(votants_in(raw.contres.as_ref()).is_empty());
        assert!(votants_in(None).is_empty());
    }

    #[test]
    fn parses_counts_and_flags_published_as_strings() {
        assert_eq!(count(Some(&"72".to_string())), 72);
        assert_eq!(count(None), 0);
        assert_eq!(count(Some(&"".to_string())), 0);
        assert_eq!(optional_count(Some(&"123".to_string())), Some(123));
        assert_eq!(optional_count(Some(&"x".to_string())), None);
        assert!(is_true(Some(&"true".to_string())));
        assert!(!is_true(Some(&"false".to_string())));
        assert!(!is_true(None));
    }
}
