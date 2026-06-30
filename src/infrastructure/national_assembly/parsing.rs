use serde::Deserialize;

use crate::domain::dossier::LegislativeStage;

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
    #[serde(rename = "initiateur")]
    pub initiator: Option<RawInitiator>,
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
    #[serde(rename = "codeActe")]
    pub code: Option<String>,
    #[serde(rename = "organeRef")]
    pub organe_ref: Option<String>,
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
pub struct RawInitiator {
    pub acteurs: Option<RawActeurs>,
}

#[derive(Deserialize)]
pub struct RawActeurs {
    pub acteur: SingleOrVec<RawActeur>,
}

#[derive(Deserialize)]
pub struct RawActeur {
    #[serde(rename = "acteurRef")]
    pub acteur_ref: String,
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

pub fn find_current_stage(acts: &Option<RawActsContainer>) -> Option<LegislativeStage> {
    let container = acts.as_ref()?;
    let mut best: Option<LegislativeStage> = None;

    fn check_top_level(act: &RawAct, best: &mut Option<LegislativeStage>) {
        if let Some(ref code) = act.code {
            if let Some(stage) = LegislativeStage::from_code(code) {
                if best.map_or(true, |b| stage > b) {
                    *best = Some(stage);
                }
            }
        }
    }

    match &container.legislative_act {
        SingleOrVec::Vec(v) => {
            for a in v {
                check_top_level(a, &mut best);
            }
        }
        SingleOrVec::Single(a) => check_top_level(a, &mut best),
    }

    best
}

pub fn find_committee_organe_ref(acts: &Option<RawActsContainer>) -> Option<String> {
    let container = acts.as_ref()?;
    let mut result: Option<String> = None;

    fn walk(act: &RawAct, result: &mut Option<String>) {
        if result.is_some() {
            return;
        }
        if let Some(ref code) = act.code {
            if code.ends_with("-COM-FOND-SAISIE") {
                if let Some(ref organe) = act.organe_ref {
                    *result = Some(organe.clone());
                    return;
                }
            }
        }
        if let Some(ref children) = act.legislative_acts {
            walk_container(children, result);
        }
    }

    fn walk_container(container: &RawActsContainer, result: &mut Option<String>) {
        match &container.legislative_act {
            SingleOrVec::Vec(v) => {
                for a in v {
                    walk(a, result);
                    if result.is_some() {
                        return;
                    }
                }
            }
            SingleOrVec::Single(a) => walk(a, result),
        }
    }

    walk_container(container, &mut result);
    result
}

pub fn extract_initiator_refs(initiator: &Option<RawInitiator>) -> Vec<String> {
    let Some(init) = initiator.as_ref() else {
        return vec![];
    };
    let Some(acteurs) = init.acteurs.as_ref() else {
        return vec![];
    };
    match &acteurs.acteur {
        SingleOrVec::Vec(v) => v.iter().map(|a| a.acteur_ref.clone()).collect(),
        SingleOrVec::Single(a) => vec![a.acteur_ref.clone()],
    }
}
