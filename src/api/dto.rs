use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::domain::dossier::LegislativeDossier;

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
}

impl From<LegislativeDossier> for DossierDto {
    fn from(d: LegislativeDossier) -> Self {
        Self {
            uid: d.uid,
            title: d.title,
            procedure: d.procedure,
            last_activity_date: d.last_activity_date,
            last_activity_label: d.last_activity_label,
            score_total: d.score.total,
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
}

impl DossierDetailDto {
    pub fn from_result(
        result: crate::application::use_cases::get_dossier_detail::DossierDetailResult,
    ) -> Self {
        let d = result.dossier;
        Self {
            uid: d.uid,
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
                progress: d.score.progress,
                magnitude: d.score.magnitude,
                momentum: d.score.momentum,
                total: d.score.total,
            },
            persisted: result.persisted,
        }
    }
}
