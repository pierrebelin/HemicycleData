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
    pub legislature: u16,
    pub url: Option<String>,
    pub deposit_date: Option<NaiveDate>,
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
            legislature: d.legislature,
            url: d.url,
            deposit_date: d.deposit_date,
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
    pub code: Option<String>,
}

#[derive(Serialize)]
pub struct DocumentDto {
    pub document_uid: String,
    pub title: String,
    pub short_title: Option<String>,
    pub doc_type: String,
    pub date: Option<NaiveDate>,
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
pub struct InitiatorGroupDto {
    pub uid: String,
    pub abbrev: String,
    /// Libelle officiel du groupe, jamais traduit en parti (RM-06).
    pub label: String,
    pub quality: Option<String>,
}

#[derive(Serialize)]
pub struct InitiatorDto {
    pub full_name: String,
    pub actor_uid: Option<String>,
    pub role: Option<String>,
    pub group: Option<InitiatorGroupDto>,
    /// Date a laquelle le groupe a ete lu. Toujours servie avec le groupe: un
    /// groupe sans sa date de reference n'est pas affichable (RM-01).
    pub reference_date: Option<NaiveDate>,
    pub official_url: Option<String>,
}

impl From<crate::domain::dossier::Initiator> for InitiatorDto {
    fn from(i: crate::domain::dossier::Initiator) -> Self {
        Self {
            full_name: i.full_name().to_string(),
            actor_uid: i.actor_uid().map(|u| u.as_str().to_string()),
            role: i.role().map(|r| r.label().to_string()),
            group: i.group().map(|g| InitiatorGroupDto {
                uid: g.uid.as_str().to_string(),
                abbrev: g.abbrev.clone(),
                label: g.label.clone(),
                quality: g.quality.as_ref().map(|q| q.as_str().to_string()),
            }),
            reference_date: i.reference_date(),
            official_url: i.official_url().map(String::from),
        }
    }
}

#[derive(Serialize)]
pub struct RegistryResponse {
    pub actors: usize,
    pub groups: usize,
    pub memberships: usize,
}

impl From<crate::application::ports::actor_repository::RegistrySummary> for RegistryResponse {
    fn from(s: crate::application::ports::actor_repository::RegistrySummary) -> Self {
        Self {
            actors: s.actors,
            groups: s.groups,
            memberships: s.memberships,
        }
    }
}

#[derive(Serialize)]
pub struct RefreshResponse {
    pub count: usize,
    pub registry: Option<RegistryResponse>,
    /// Renseigne quand le referentiel n'a pas pu etre rafraichi: les
    /// rattachements reposent alors sur la version precedente.
    pub registry_anomaly: Option<String>,
}

impl From<crate::application::use_cases::refresh_all::RefreshOutcome> for RefreshResponse {
    fn from(o: crate::application::use_cases::refresh_all::RefreshOutcome) -> Self {
        Self {
            count: o.dossiers,
            registry: o.registry.map(RegistryResponse::from),
            registry_anomaly: o.registry_anomaly,
        }
    }
}

#[derive(Serialize)]
pub struct DossierDetailDto {
    pub uid: String,
    pub title: String,
    pub procedure: String,
    pub legislature: u16,
    pub url: Option<String>,
    pub summary: Option<String>,
    pub deposit_date: Option<NaiveDate>,
    pub last_activity_date: NaiveDate,
    pub last_activity_label: String,
    pub acts: Vec<ActDto>,
    pub documents: Vec<DocumentDto>,
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
            legislature: d.legislature,
            url: d.url,
            summary: d.summary,
            deposit_date: d.deposit_date,
            last_activity_date: d.last_activity_date,
            last_activity_label: d.last_activity_label,
            acts: d
                .acts
                .into_iter()
                .map(|a| ActDto {
                    date: a.date,
                    label: a.label,
                    code: a.code,
                })
                .collect(),
            documents: d
                .documents
                .into_iter()
                .map(|doc| DocumentDto {
                    document_uid: doc.document_uid,
                    title: doc.title,
                    short_title: doc.short_title,
                    doc_type: doc.doc_type,
                    date: doc.date,
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
            initiators: d.initiators.into_iter().map(InitiatorDto::from).collect(),
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
