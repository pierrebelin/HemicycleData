use chrono::NaiveDate;
use serde::Deserialize;

use super::parsing::SingleOrVec;

/// Champ texte de la source, qui peut aussi porter la forme XML `xsi:nil`
/// serialisee en objet. Toute autre forme est traitee comme absente plutot que
/// de faire echouer l'acteur entier.
#[derive(Deserialize)]
#[serde(untagged)]
pub enum RawText {
    Text(String),
    Other(serde::de::IgnoredAny),
}

impl RawText {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s.as_str()),
            Self::Other(_) => None,
        }
    }
}

pub fn text(value: &Option<RawText>) -> Option<&str> {
    value.as_ref().and_then(RawText::as_str)
}

pub fn parse_date(value: Option<&str>) -> Option<NaiveDate> {
    let raw = value?;
    NaiveDate::parse_from_str(&raw[..10.min(raw.len())], "%Y-%m-%d").ok()
}

#[derive(Deserialize)]
pub struct RawActorWrapper {
    pub acteur: RawActor,
}

#[derive(Deserialize)]
pub struct RawActor {
    pub uid: RawUid,
    #[serde(rename = "etatCivil")]
    pub etat_civil: Option<RawEtatCivil>,
    pub mandats: Option<RawMandats>,
}

/// L'identifiant d'acteur est publie sous forme d'objet typé (`{"#text": "PA..."}`),
/// alors que celui des organes est une chaine simple.
#[derive(Deserialize)]
#[serde(untagged)]
pub enum RawUid {
    Text(String),
    Object {
        #[serde(rename = "#text")]
        text: String,
    },
}

impl RawUid {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Text(s) => s,
            Self::Object { text } => text,
        }
    }
}

#[derive(Deserialize)]
pub struct RawEtatCivil {
    pub ident: Option<RawIdent>,
}

#[derive(Deserialize)]
pub struct RawIdent {
    pub civ: Option<RawText>,
    pub prenom: Option<RawText>,
    pub nom: Option<RawText>,
}

#[derive(Deserialize)]
pub struct RawMandats {
    pub mandat: SingleOrVec<RawMandat>,
}

#[derive(Deserialize)]
pub struct RawMandat {
    pub uid: String,
    pub legislature: Option<String>,
    #[serde(rename = "typeOrgane")]
    pub type_organe: Option<String>,
    #[serde(rename = "dateDebut")]
    pub date_debut: Option<String>,
    #[serde(rename = "dateFin")]
    pub date_fin: Option<String>,
    #[serde(rename = "infosQualite")]
    pub infos_qualite: Option<RawQualite>,
    pub organes: Option<RawOrganes>,
}

#[derive(Deserialize)]
pub struct RawQualite {
    #[serde(rename = "codeQualite")]
    pub code_qualite: Option<RawText>,
}

#[derive(Deserialize)]
pub struct RawOrganes {
    #[serde(rename = "organeRef")]
    pub organe_ref: Option<SingleOrVec<String>>,
}

impl RawOrganes {
    pub fn first_ref(&self) -> Option<&str> {
        match self.organe_ref.as_ref()? {
            SingleOrVec::Vec(v) => v.first().map(String::as_str),
            SingleOrVec::Single(s) => Some(s.as_str()),
        }
    }
}

#[derive(Deserialize)]
pub struct RawOrganeWrapper {
    pub organe: RawOrgane,
}

#[derive(Deserialize)]
pub struct RawOrgane {
    pub uid: String,
    #[serde(rename = "codeType")]
    pub code_type: Option<String>,
    pub libelle: Option<RawText>,
    #[serde(rename = "libelleAbrev")]
    pub libelle_abrev: Option<RawText>,
    #[serde(rename = "libelleAbrege")]
    pub libelle_abrege: Option<RawText>,
    pub legislature: Option<String>,
    #[serde(rename = "couleurAssociee")]
    pub couleur_associee: Option<RawText>,
    #[serde(rename = "viMoDe")]
    pub vie_mode: Option<RawVieMode>,
}

#[derive(Deserialize)]
pub struct RawVieMode {
    #[serde(rename = "dateDebut")]
    pub date_debut: Option<String>,
    #[serde(rename = "dateFin")]
    pub date_fin: Option<String>,
}

pub const GROUP_ORGANE_CODE: &str = "GP";
pub const ASSEMBLY_MANDATE_CODE: &str = "ASSEMBLEE";
pub const MINISTRY_MANDATE_CODE: &str = "MINISTERE";
pub const SENATE_MANDATE_CODE: &str = "SENAT";

pub fn mandates(mandats: &Option<RawMandats>) -> &[RawMandat] {
    match mandats.as_ref().map(|m| &m.mandat) {
        Some(SingleOrVec::Vec(v)) => v.as_slice(),
        Some(SingleOrVec::Single(m)) => std::slice::from_ref(m.as_ref()),
        None => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actor_uid_reads_object_form() {
        let raw: RawUid = serde_json::from_str(
            r##"{"@xsi:type":"IdActeur_type","#text":"PA720916"}"##,
        )
        .unwrap();
        assert_eq!(raw.as_str(), "PA720916");
    }

    #[test]
    fn actor_uid_reads_plain_string_form() {
        let raw: RawUid = serde_json::from_str(r#""PA720916""#).unwrap();
        assert_eq!(raw.as_str(), "PA720916");
    }

    #[test]
    fn nil_object_reads_as_absent_text() {
        let raw: Option<RawText> =
            serde_json::from_str(r#"{"@xsi:nil":"true"}"#).unwrap();
        assert_eq!(text(&raw), None);
    }

    #[test]
    fn plain_text_reads_as_present() {
        let raw: Option<RawText> = serde_json::from_str(r#""Membre""#).unwrap();
        assert_eq!(text(&raw), Some("Membre"));
    }

    #[test]
    fn parse_date_accepts_short_and_full_timestamps() {
        assert_eq!(
            parse_date(Some("2024-07-19")),
            NaiveDate::from_ymd_opt(2024, 7, 19)
        );
        assert_eq!(
            parse_date(Some("2025-05-13T00:00:00.000+02:00")),
            NaiveDate::from_ymd_opt(2025, 5, 13)
        );
        assert_eq!(parse_date(None), None);
        assert_eq!(parse_date(Some("nope")), None);
    }

    #[test]
    fn single_mandate_is_read_as_a_one_element_slice() {
        let raw: RawMandats = serde_json::from_str(
            r#"{"mandat":{"uid":"PM1","typeOrgane":"GP","dateDebut":"2024-07-19"}}"#,
        )
        .unwrap();
        let wrapped = Some(raw);
        let mandats = mandates(&wrapped);
        assert_eq!(mandats.len(), 1);
        assert_eq!(mandats[0].uid, "PM1");
    }

    #[test]
    fn organe_ref_reads_single_and_list_forms() {
        let single: RawOrganes = serde_json::from_str(r#"{"organeRef":"PO845414"}"#).unwrap();
        assert_eq!(single.first_ref(), Some("PO845414"));

        let list: RawOrganes =
            serde_json::from_str(r#"{"organeRef":["PO845414","PO845415"]}"#).unwrap();
        assert_eq!(list.first_ref(), Some("PO845414"));
    }
}
