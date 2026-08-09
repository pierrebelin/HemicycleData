//! Formes brutes du jeu de donnees des amendements.
//!
//! Meme conversion mecanique du XML que les scrutins: les nombres sont des
//! chaines, un element unique remplace un tableau d'un element, et un element
//! vide devient `null` au milieu d'un tableau. Ces formes sont absorbees ici
//! plutot que dans le domaine.
//!
//! Tous les champs sont optionnels, y compris ceux que la source publie
//! toujours. L'archive n'est pas joignable depuis l'environnement de
//! developpement (SPEC-amendements §6): un nom de champ qui differe doit rendre
//! un amendement incomplet, jamais faire echouer le parcours entier.

use serde::{de::DeserializeOwned, Deserialize, Deserializer};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct RawAmendmentWrapper {
    pub amendement: RawAmendment,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawAmendment {
    #[serde(default, deserialize_with = "lenient")]
    pub uid: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub legislature: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub texte_legislatif_ref: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub examen_ref: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub identifiant: Option<RawIdentifiant>,
    #[serde(default, deserialize_with = "lenient")]
    pub division: Option<RawDivision>,
    #[serde(default, deserialize_with = "lenient")]
    pub signataires: Option<RawSignataires>,
    #[serde(default, deserialize_with = "lenient")]
    pub corps: Option<RawCorps>,
    #[serde(default, deserialize_with = "lenient")]
    pub cycle_de_vie: Option<RawCycleDeVie>,
    #[serde(default, deserialize_with = "lenient")]
    pub sort_en_seance: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub etat: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub amendement_parent: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawIdentifiant {
    #[serde(default, deserialize_with = "lenient")]
    pub numero: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub numero_long: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawDivision {
    #[serde(default, deserialize_with = "lenient")]
    pub titre: Option<String>,
    #[serde(rename = "type")]
    #[serde(default, deserialize_with = "lenient")]
    pub kind: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawSignataires {
    #[serde(default, deserialize_with = "lenient")]
    pub auteur: Option<RawAuteur>,
    #[serde(default, deserialize_with = "lenient")]
    pub cosignataires: Option<RawCosignataires>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawAuteur {
    #[serde(default, deserialize_with = "lenient")]
    pub acteur_ref: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub groupe_politique_ref: Option<String>,
    /// « Député », « Gouvernement », « Commission ». Sert a distinguer un auteur
    /// nominatif d'un auteur institutionnel quand `acteurRef` est absent.
    #[serde(default, deserialize_with = "lenient")]
    pub type_auteur: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub organe_ref: Option<String>,
    /// Libelle publie d'un auteur institutionnel, quand la source en donne un.
    #[serde(default, deserialize_with = "lenient")]
    pub libelle: Option<String>,
}

/// Les cosignataires prennent toutes les formes de la conversion XML: absent,
/// objet unique, tableau, tableau a `null` intercalaires. D'ou `Value` et une
/// lecture manuelle, comme les mises au point des scrutins.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawCosignataires {
    pub acteur_ref: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawCorps {
    #[serde(default, deserialize_with = "lenient")]
    pub contenu_auteur: Option<RawContenuAuteur>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawContenuAuteur {
    #[serde(default, deserialize_with = "lenient")]
    pub expose_sommaire: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawCycleDeVie {
    #[serde(default, deserialize_with = "lenient")]
    pub date_depot: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    pub etat_des_traitements: Option<RawEtatDesTraitements>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawEtatDesTraitements {
    #[serde(default, deserialize_with = "lenient")]
    pub etat: Option<RawLibelle>,
    #[serde(default, deserialize_with = "lenient")]
    pub sort: Option<RawLibelle>,
}

#[derive(Debug, Deserialize)]
pub struct RawLibelle {
    #[serde(default, deserialize_with = "lenient")]
    pub libelle: Option<String>,
}

/// Une discordance de type dans un sous-bloc ne rend pas l'entree illisible.
/// Elle laisse l'amendement incomplet, afin que les invariants de domaine
/// decident ensuite s'il peut etre conserve.
fn lenient<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let value = Value::deserialize(deserializer)?;
    if value.is_null() {
        return Ok(None);
    }
    Ok(serde_json::from_value(value).ok())
}

/// References d'acteurs d'un bloc de cosignataires, quelle qu'en soit la forme.
///
/// Jumeau de `votants_in` cote scrutins: la source imbrique, replie les tableaux
/// d'un element, et laisse des `null`. On descend jusqu'aux chaines.
pub fn actor_refs_in(value: Option<&Value>) -> Vec<String> {
    let mut out = Vec::new();
    collect_actor_refs(value, &mut out);
    out
}

fn collect_actor_refs(value: Option<&Value>, out: &mut Vec<String>) {
    match value {
        Some(Value::String(raw)) => {
            let trimmed = raw.trim();
            if !trimmed.is_empty() {
                out.push(trimmed.to_string());
            }
        }
        Some(Value::Array(items)) => {
            for item in items {
                collect_actor_refs(Some(item), out);
            }
        }
        Some(Value::Object(map)) => {
            for nested in map.values() {
                collect_actor_refs(Some(nested), out);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_single_cosignatory_reads_like_a_list_of_one() {
        let value: Value = serde_json::from_str(r#""PA1592""#).unwrap();
        assert_eq!(actor_refs_in(Some(&value)), vec!["PA1592"]);
    }

    #[test]
    fn a_null_padded_array_yields_only_the_real_refs() {
        let value: Value = serde_json::from_str(r#"["PA1", null, "PA2", "  "]"#).unwrap();
        assert_eq!(actor_refs_in(Some(&value)), vec!["PA1", "PA2"]);
    }

    #[test]
    fn a_nested_block_is_walked_down_to_the_strings() {
        let value: Value = serde_json::from_str(r#"{"acteurRef": ["PA1", "PA2"]}"#).unwrap();
        assert_eq!(actor_refs_in(Some(&value)), vec!["PA1", "PA2"]);
    }

    #[test]
    fn an_absent_block_yields_nothing() {
        assert!(actor_refs_in(None).is_empty());
        let null: Value = serde_json::from_str("null").unwrap();
        assert!(actor_refs_in(Some(&null)).is_empty());
    }
}
