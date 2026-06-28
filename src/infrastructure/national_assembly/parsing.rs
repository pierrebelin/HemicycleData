use serde::Deserialize;

#[derive(Deserialize)]
pub struct RawDossierWrapper {
    #[serde(rename = "dossierParlementaire")]
    pub parliamentary_dossier: RawDossier,
}

#[derive(Deserialize)]
pub struct RawDossier {
    pub uid: String,
    #[serde(rename = "titreDossier")]
    pub dossier_title: RawTitle,
    #[serde(rename = "procedureParlementaire")]
    pub parliamentary_procedure: RawProcedure,
    #[serde(rename = "actesLegislatifs")]
    pub legislative_acts: Option<RawActsContainer>,
}

#[derive(Deserialize)]
pub struct RawTitle {
    pub titre: String,
}

#[derive(Deserialize)]
pub struct RawProcedure {
    pub libelle: String,
}

#[derive(Deserialize)]
pub struct RawActsContainer {
    #[serde(rename = "acteLegislatif")]
    pub legislative_act: SingleOrVec<RawAct>,
}

#[derive(Deserialize)]
pub struct RawAct {
    #[serde(rename = "dateActe")]
    pub act_date: Option<String>,
    #[serde(rename = "libelleActe")]
    pub act_label: Option<RawActLabel>,
    #[serde(rename = "actesLegislatifs")]
    pub legislative_acts: Option<RawActsContainer>,
}

#[derive(Deserialize)]
pub struct RawActLabel {
    #[serde(rename = "libelleCourt")]
    pub short_label: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum SingleOrVec<T> {
    Vec(Vec<T>),
    Single(Box<T>),
}

pub struct ActInfo {
    pub date: String,
    pub label: String,
}

pub fn find_latest_act(acts: &Option<RawActsContainer>) -> Option<ActInfo> {
    let container = acts.as_ref()?;
    let mut latest: Option<ActInfo> = None;

    fn walk(act: &RawAct, latest: &mut Option<ActInfo>) {
        if let Some(ref date_str) = act.act_date {
            let date_short = &date_str[..10.min(date_str.len())];
            let label = act
                .act_label
                .as_ref()
                .and_then(|l| l.short_label.as_deref())
                .unwrap_or("?")
                .to_string();

            let dominated = latest
                .as_ref()
                .map(|l| date_short > l.date.as_str())
                .unwrap_or(true);

            if dominated {
                *latest = Some(ActInfo {
                    date: date_short.to_string(),
                    label,
                });
            }
        }
        if let Some(ref children) = act.legislative_acts {
            walk_container(children, latest);
        }
    }

    fn walk_container(container: &RawActsContainer, latest: &mut Option<ActInfo>) {
        match &container.legislative_act {
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

pub fn collect_all_acts(acts: &Option<RawActsContainer>) -> Vec<ActInfo> {
    let mut result = Vec::new();

    fn walk(act: &RawAct, result: &mut Vec<ActInfo>) {
        if let Some(ref date_str) = act.act_date {
            let date_short = &date_str[..10.min(date_str.len())];
            let label = act
                .act_label
                .as_ref()
                .and_then(|l| l.short_label.as_deref())
                .unwrap_or("?")
                .to_string();
            result.push(ActInfo {
                date: date_short.to_string(),
                label,
            });
        }
        if let Some(ref children) = act.legislative_acts {
            walk_container(children, result);
        }
    }

    fn walk_container(container: &RawActsContainer, result: &mut Vec<ActInfo>) {
        match &container.legislative_act {
            SingleOrVec::Vec(v) => {
                for a in v {
                    walk(a, result);
                }
            }
            SingleOrVec::Single(a) => walk(a, result),
        }
    }

    if let Some(container) = acts.as_ref() {
        walk_container(container, &mut result);
    }

    result.sort_by(|a, b| a.date.cmp(&b.date));
    result
}
