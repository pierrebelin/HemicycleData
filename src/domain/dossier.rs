use chrono::NaiveDate;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct LegislativeAct {
    pub date: NaiveDate,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Score {
    pub progress: u8,
    pub magnitude: u8,
    pub momentum: u8,
    pub total: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct LegislativeDossier {
    pub uid: String,
    pub title: String,
    pub procedure: String,
    pub last_activity_date: NaiveDate,
    pub last_activity_label: String,
    pub acts: Vec<LegislativeAct>,
    pub score: Score,
}
