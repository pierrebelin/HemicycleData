//! Registre source des versions de texte explicitement reliees a un scrutin.
//!
//! Le fichier est versionne avec le code afin que chaque ajout soit relisible.
//! Il ne contient jamais de rapprochement deduit d'un titre, d'une date ou d'un
//! dossier (README.md §7).

use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;
use std::collections::HashSet;

const BUNDLED_REGISTRY: &str = include_str!("../../data/official-text-versions.json");

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("registre des versions officielles invalide: {0}")]
    Json(#[from] serde_json::Error),
    #[error("entrée {entry}: {field} ne doit pas être vide")]
    EmptyField { entry: usize, field: &'static str },
    #[error("entrée {entry}: {field} doit être une URL HTTPS")]
    NonHttpsUrl { entry: usize, field: &'static str },
    #[error("entrée {entry}: numéro de législature invalide")]
    InvalidLegislature { entry: usize },
    #[error("entrée {entry}: le scrutin {scrutin_number} de la {legislature}e législature est déjà référencé")]
    DuplicateScrutin {
        entry: usize,
        legislature: u16,
        scrutin_number: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct VerifiedOfficialTextVersion {
    pub legislature: u16,
    pub scrutin_number: String,
    pub document_uid: String,
    pub document_title: String,
    pub version_label: String,
    pub document_published_on: Option<NaiveDate>,
    pub official_url: String,
    /// Version Open Data HTML du document, explicitement liée depuis la page
    /// officielle. Elle est figée sur le VPS avant toute synthèse.
    pub content_url: String,
    /// Acte officiel qui rattache explicitement ce scrutin a cette version.
    pub mapping_source_url: String,
    pub source_producer: String,
    pub source_license: String,
    pub source_metadata_fingerprint: Option<String>,
    pub source_retrieved_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct RegistryFile {
    versions: Vec<VerifiedOfficialTextVersion>,
}

pub fn bundled_versions() -> Result<Vec<VerifiedOfficialTextVersion>, RegistryError> {
    parse(BUNDLED_REGISTRY)
}

fn parse(raw: &str) -> Result<Vec<VerifiedOfficialTextVersion>, RegistryError> {
    let registry: RegistryFile = serde_json::from_str(raw)?;
    let mut referenced_scrutins = HashSet::new();
    for (index, version) in registry.versions.iter().enumerate() {
        validate(version, index + 1)?;
        if !referenced_scrutins.insert((version.legislature, version.scrutin_number.clone())) {
            return Err(RegistryError::DuplicateScrutin {
                entry: index + 1,
                legislature: version.legislature,
                scrutin_number: version.scrutin_number.clone(),
            });
        }
    }
    Ok(registry.versions)
}

fn validate(version: &VerifiedOfficialTextVersion, entry: usize) -> Result<(), RegistryError> {
    if version.legislature == 0 {
        return Err(RegistryError::InvalidLegislature { entry });
    }

    for (field, value) in [
        ("scrutin_number", version.scrutin_number.as_str()),
        ("document_uid", version.document_uid.as_str()),
        ("document_title", version.document_title.as_str()),
        ("version_label", version.version_label.as_str()),
        ("source_producer", version.source_producer.as_str()),
        ("source_license", version.source_license.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(RegistryError::EmptyField { entry, field });
        }
    }

    for (field, value) in [
        ("official_url", version.official_url.as_str()),
        ("content_url", version.content_url.as_str()),
        ("mapping_source_url", version.mapping_source_url.as_str()),
    ] {
        if !value.starts_with("https://") {
            return Err(RegistryError::NonHttpsUrl { entry, field });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMPLETE_ENTRY: &str = r#"
    {
      "versions": [{
        "legislature": 17,
        "scrutin_number": "123",
        "document_uid": "PIONANR5L17B1234",
        "document_title": "Proposition de loi n° 1234",
        "version_label": "Texte adopté n° 1",
        "document_published_on": "2026-01-02",
        "official_url": "https://www.assemblee-nationale.fr/dyn/17/textes/l17t0001_texte-adopte-seance",
        "content_url": "https://www.assemblee-nationale.fr/dyn/opendata/PIONANR5L17BTA0001.html",
        "mapping_source_url": "https://www.assemblee-nationale.fr/dyn/17/dossiers/exemple",
        "source_producer": "Assemblée nationale",
        "source_license": "Licence Ouverte / Open Licence",
        "source_metadata_fingerprint": "sha256:example",
        "source_retrieved_at": "2026-08-16T12:00:00Z"
      }]
    }"#;

    #[test]
    fn accepts_an_explicitly_sourced_version() {
        let versions = parse(COMPLETE_ENTRY).unwrap();

        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].scrutin_number, "123");
    }

    #[test]
    fn bundled_registry_contains_the_five_verified_pilots() {
        let versions = bundled_versions().unwrap();

        assert_eq!(versions.len(), 5);
        assert_eq!(
            versions
                .iter()
                .map(|version| version.scrutin_number.as_str())
                .collect::<Vec<_>>(),
            ["611", "612", "617", "653", "1018"]
        );
    }

    #[test]
    fn rejects_a_non_official_document_url() {
        let invalid = COMPLETE_ENTRY.replace(
            "https://www.assemblee-nationale.fr/dyn/17/textes/l17t0001_texte-adopte-seance",
            "http://example.test/document",
        );

        assert!(matches!(
            parse(&invalid),
            Err(RegistryError::NonHttpsUrl {
                field: "official_url",
                ..
            })
        ));
    }

    #[test]
    fn rejects_a_non_https_content_url() {
        let invalid = COMPLETE_ENTRY.replace(
            "https://www.assemblee-nationale.fr/dyn/opendata/PIONANR5L17BTA0001.html",
            "http://example.test/document.html",
        );

        assert!(matches!(
            parse(&invalid),
            Err(RegistryError::NonHttpsUrl {
                field: "content_url",
                ..
            })
        ));
    }

    #[test]
    fn rejects_an_entry_without_a_scrutin_number() {
        let invalid =
            COMPLETE_ENTRY.replace("\"scrutin_number\": \"123\"", "\"scrutin_number\": \" \"");

        assert!(matches!(
            parse(&invalid),
            Err(RegistryError::EmptyField {
                field: "scrutin_number",
                ..
            })
        ));
    }

    #[test]
    fn rejects_two_versions_for_the_same_scrutin() {
        let duplicate = COMPLETE_ENTRY.replace(
            "}]\n    }",
            "}, {\n        \"legislature\": 17,\n        \"scrutin_number\": \"123\",\n        \"document_uid\": \"PIONANR5L17B1235\",\n        \"document_title\": \"Proposition de loi n° 1235\",\n        \"version_label\": \"Texte adopté n° 2\",\n        \"document_published_on\": null,\n        \"official_url\": \"https://www.assemblee-nationale.fr/dyn/17/textes/l17t0002_texte-adopte-seance\",\n        \"content_url\": \"https://www.assemblee-nationale.fr/dyn/opendata/PIONANR5L17BTA0002.html\",\n        \"mapping_source_url\": \"https://www.assemblee-nationale.fr/dyn/17/dossiers/exemple-2\",\n        \"source_producer\": \"Assemblée nationale\",\n        \"source_license\": \"Licence Ouverte / Open Licence\",\n        \"source_metadata_fingerprint\": null,\n        \"source_retrieved_at\": \"2026-08-16T12:00:00Z\"\n      }]\n    }",
        );

        assert!(matches!(
            parse(&duplicate),
            Err(RegistryError::DuplicateScrutin {
                entry: 2,
                legislature: 17,
                ref scrutin_number,
            }) if scrutin_number == "123"
        ));
    }
}
