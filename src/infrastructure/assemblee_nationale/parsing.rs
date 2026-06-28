use serde::Deserialize;

#[derive(Deserialize)]
pub struct RawDossierWrapper {
    #[serde(rename = "dossierParlementaire")]
    pub dossier_parlementaire: RawDossier,
}

#[derive(Deserialize)]
pub struct RawDossier {
    pub uid: String,
    #[serde(rename = "titreDossier")]
    pub titre_dossier: RawTitre,
    #[serde(rename = "procedureParlementaire")]
    pub procedure_parlementaire: RawProcedure,
    #[serde(rename = "actesLegislatifs")]
    pub actes_legislatifs: Option<RawActesContainer>,
}

#[derive(Deserialize)]
pub struct RawTitre {
    pub titre: String,
}

#[derive(Deserialize)]
pub struct RawProcedure {
    pub libelle: String,
}

#[derive(Deserialize)]
pub struct RawActesContainer {
    #[serde(rename = "acteLegislatif")]
    pub acte_legislatif: SingleOrVec<RawActe>,
}

#[derive(Deserialize)]
pub struct RawActe {
    #[serde(rename = "dateActe")]
    pub date_acte: Option<String>,
    #[serde(rename = "libelleActe")]
    pub libelle_acte: Option<RawLibelleActe>,
    #[serde(rename = "actesLegislatifs")]
    pub actes_legislatifs: Option<RawActesContainer>,
}

#[derive(Deserialize)]
pub struct RawLibelleActe {
    #[serde(rename = "libelleCourt")]
    pub libelle_court: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum SingleOrVec<T> {
    Vec(Vec<T>),
    Single(Box<T>),
}

pub struct ActeInfo {
    pub date: String,
    pub libelle: String,
}

pub fn find_latest_acte(actes: &Option<RawActesContainer>) -> Option<ActeInfo> {
    let container = actes.as_ref()?;
    let mut latest: Option<ActeInfo> = None;

    fn walk(acte: &RawActe, latest: &mut Option<ActeInfo>) {
        if let Some(ref date_str) = acte.date_acte {
            let date_short = &date_str[..10.min(date_str.len())];
            let libelle = acte
                .libelle_acte
                .as_ref()
                .and_then(|l| l.libelle_court.as_deref())
                .unwrap_or("?")
                .to_string();

            let dominated = latest
                .as_ref()
                .map(|l| date_short > l.date.as_str())
                .unwrap_or(true);

            if dominated {
                *latest = Some(ActeInfo {
                    date: date_short.to_string(),
                    libelle,
                });
            }
        }
        if let Some(ref children) = acte.actes_legislatifs {
            walk_container(children, latest);
        }
    }

    fn walk_container(container: &RawActesContainer, latest: &mut Option<ActeInfo>) {
        match &container.acte_legislatif {
            SingleOrVec::Vec(v) => {
                for a in v {
                    walk(a, latest);
                }
            }
            SingleOrVec::Single(a) => walk(a, latest),
        }
    }

    walk_container(container, &mut latest);
    latest
}
