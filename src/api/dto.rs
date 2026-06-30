use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::domain::dossier::{CurationStatus, LegislativeDossier};

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
    pub title: String,
    pub procedure: String,
    pub last_activity_date: NaiveDate,
    pub last_activity_label: String,
    pub score_total: u8,
    pub current_stage: Option<StageDto>,
    pub committee: Option<String>,
    pub curation_status: String,
}

impl From<LegislativeDossier> for DossierDto {
    fn from(d: LegislativeDossier) -> Self {
        Self {
            uid: d.uid.as_str().to_string(),
            title: d.title,
            procedure: d.procedure,
            last_activity_date: d.last_activity_date,
            last_activity_label: d.last_activity_label,
            score_total: d.score.total(),
            current_stage: d.current_stage.map(StageDto::from),
            committee: d.committee.map(|c| c.as_str().to_string()),
            curation_status: d.curation_status.as_str().to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct RecentDossiersResponse {
    pub count: usize,
    pub dossiers: Vec<DossierDto>,
}

#[derive(Serialize)]
pub struct ActDto {
    pub date: NaiveDate,
    pub label: String,
}

#[derive(Serialize)]
pub struct ScoreDto {
    pub progress: u8,
    pub magnitude: u8,
    pub momentum: u8,
    pub total: u8,
}

#[derive(Serialize)]
pub struct StageDto {
    pub label: String,
    pub chamber: String,
}

impl From<crate::domain::dossier::LegislativeStage> for StageDto {
    fn from(s: crate::domain::dossier::LegislativeStage) -> Self {
        let chamber = match s.chamber() {
            crate::domain::dossier::Chamber::AssembleeNationale => {
                "Assembl\u{00e9}e nationale".to_string()
            }
            crate::domain::dossier::Chamber::Senat => "S\u{00e9}nat".to_string(),
            crate::domain::dossier::Chamber::Joint => "Conjointe".to_string(),
            crate::domain::dossier::Chamber::None => String::new(),
        };
        Self {
            label: s.label().to_string(),
            chamber,
        }
    }
}

#[derive(Serialize)]
pub struct InitiatorDto {
    pub full_name: String,
    pub group: Option<String>,
}

#[derive(Serialize)]
pub struct RefreshResponse {
    pub count: usize,
}

#[derive(Serialize)]
pub struct DossierDetailDto {
    pub uid: String,
    pub title: String,
    pub procedure: String,
    pub last_activity_date: NaiveDate,
    pub last_activity_label: String,
    pub acts: Vec<ActDto>,
    pub score: ScoreDto,
    pub persisted: bool,
    pub current_stage: Option<StageDto>,
    pub initiators: Vec<InitiatorDto>,
    pub committee: Option<String>,
    pub curation_status: String,
}

impl DossierDetailDto {
    pub fn from_result(
        result: crate::application::use_cases::get_dossier_detail::DossierDetailResult,
    ) -> Self {
        let d = result.dossier;
        Self {
            uid: d.uid.as_str().to_string(),
            title: d.title,
            procedure: d.procedure,
            last_activity_date: d.last_activity_date,
            last_activity_label: d.last_activity_label,
            acts: d
                .acts
                .into_iter()
                .map(|a| ActDto {
                    date: a.date,
                    label: a.label,
                })
                .collect(),
            score: ScoreDto {
                progress: d.score.progress(),
                magnitude: d.score.magnitude(),
                momentum: d.score.momentum(),
                total: d.score.total(),
            },
            persisted: result.persisted,
            current_stage: d.current_stage.map(StageDto::from),
            initiators: d
                .initiators
                .into_iter()
                .map(|i| InitiatorDto {
                    full_name: i.full_name().to_string(),
                    group: i.group().map(String::from),
                })
                .collect(),
            committee: d.committee.map(|c| c.as_str().to_string()),
            curation_status: d.curation_status.as_str().to_string(),
        }
    }
}

#[derive(Deserialize)]
pub struct SuggestionsQuery {
    #[serde(default = "default_count")]
    pub count: usize,
}

fn default_count() -> usize {
    3
}

#[derive(Serialize)]
pub struct SuggestionsResponse {
    pub count: usize,
    pub suggestions: Vec<DossierDto>,
}

#[derive(Deserialize)]
pub struct CurateBody {
    pub status: CurationStatus,
}
