use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("dossier uid must not be empty")]
    EmptyDossierUid,
    #[error("score progress must be 0-10, got {0}")]
    InvalidProgress(u8),
    #[error("score magnitude must be 0-10, got {0}")]
    InvalidMagnitude(u8),
    #[error("score momentum must be 0-10, got {0}")]
    InvalidMomentum(u8),
    #[error("score total must be 0-100, got {0}")]
    InvalidTotal(u8),
    #[error("initiator name must not be empty")]
    EmptyInitiatorName,
    #[error("committee name must not be empty")]
    EmptyCommittee,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct DossierUid(String);

impl DossierUid {
    pub fn new(raw: String) -> Result<Self, DomainError> {
        if raw.is_empty() {
            return Err(DomainError::EmptyDossierUid);
        }
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DossierUid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Committee(String);

impl Committee {
    pub fn new(raw: String) -> Result<Self, DomainError> {
        if raw.is_empty() {
            return Err(DomainError::EmptyCommittee);
        }
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LegislativeAct {
    pub date: NaiveDate,
    pub label: String,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LegislativeDocument {
    pub document_uid: String,
    pub title: String,
    pub short_title: Option<String>,
    pub doc_type: String,
    pub date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Score {
    progress: u8,
    magnitude: u8,
    momentum: u8,
    total: u8,
}

impl Score {
    pub fn new(progress: u8, magnitude: u8, momentum: u8, total: u8) -> Result<Self, DomainError> {
        if progress > 10 {
            return Err(DomainError::InvalidProgress(progress));
        }
        if magnitude > 10 {
            return Err(DomainError::InvalidMagnitude(magnitude));
        }
        if momentum > 10 {
            return Err(DomainError::InvalidMomentum(momentum));
        }
        if total > 100 {
            return Err(DomainError::InvalidTotal(total));
        }
        Ok(Self {
            progress,
            magnitude,
            momentum,
            total,
        })
    }

    pub fn progress(&self) -> u8 {
        self.progress
    }

    pub fn magnitude(&self) -> u8 {
        self.magnitude
    }

    pub fn momentum(&self) -> u8 {
        self.momentum
    }

    pub fn total(&self) -> u8 {
        self.total
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Chamber {
    AssembleeNationale,
    Senat,
    Joint,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum LegislativeStage {
    FirstReadingAN,
    FirstReadingSenate,
    SecondReadingAN,
    SecondReadingSenate,
    JointCommittee,
    NewReadingAN,
    NewReadingSenate,
    FinalReading,
    ConstitutionalReview,
    Promulgation,
}

impl LegislativeStage {
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "AN1" => Some(Self::FirstReadingAN),
            "SN1" => Some(Self::FirstReadingSenate),
            "AN2" => Some(Self::SecondReadingAN),
            "SN2" => Some(Self::SecondReadingSenate),
            "CMP" => Some(Self::JointCommittee),
            "ANNLEC" => Some(Self::NewReadingAN),
            "SNNLEC" => Some(Self::NewReadingSenate),
            "ANLDEF" => Some(Self::FinalReading),
            "CC" => Some(Self::ConstitutionalReview),
            "PROM" => Some(Self::Promulgation),
            _ => None,
        }
    }

    pub fn to_code(&self) -> &'static str {
        match self {
            Self::FirstReadingAN => "AN1",
            Self::FirstReadingSenate => "SN1",
            Self::SecondReadingAN => "AN2",
            Self::SecondReadingSenate => "SN2",
            Self::JointCommittee => "CMP",
            Self::NewReadingAN => "ANNLEC",
            Self::NewReadingSenate => "SNNLEC",
            Self::FinalReading => "ANLDEF",
            Self::ConstitutionalReview => "CC",
            Self::Promulgation => "PROM",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::FirstReadingAN => "1\u{00e8}re lecture",
            Self::FirstReadingSenate => "1\u{00e8}re lecture",
            Self::SecondReadingAN => "2\u{00e8}me lecture",
            Self::SecondReadingSenate => "2\u{00e8}me lecture",
            Self::JointCommittee => "Commission mixte paritaire",
            Self::NewReadingAN => "Nouvelle lecture",
            Self::NewReadingSenate => "Nouvelle lecture",
            Self::FinalReading => "Lecture d\u{00e9}finitive",
            Self::ConstitutionalReview => "Conseil constitutionnel",
            Self::Promulgation => "Promulgation",
        }
    }

    pub fn chamber(&self) -> Chamber {
        match self {
            Self::FirstReadingAN
            | Self::SecondReadingAN
            | Self::NewReadingAN
            | Self::FinalReading => Chamber::AssembleeNationale,
            Self::FirstReadingSenate
            | Self::SecondReadingSenate
            | Self::NewReadingSenate => Chamber::Senat,
            Self::JointCommittee => Chamber::Joint,
            Self::ConstitutionalReview | Self::Promulgation => Chamber::None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Initiator {
    full_name: String,
    group: Option<String>,
}

impl Initiator {
    pub fn new(full_name: String, group: Option<String>) -> Result<Self, DomainError> {
        if full_name.is_empty() {
            return Err(DomainError::EmptyInitiatorName);
        }
        Ok(Self { full_name, group })
    }

    pub fn full_name(&self) -> &str {
        &self.full_name
    }

    pub fn group(&self) -> Option<&str> {
        self.group.as_deref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurationStatus {
    New,
    Selected,
    Dismissed,
    Published,
}

impl CurationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Selected => "selected",
            Self::Dismissed => "dismissed",
            Self::Published => "published",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "new" => Some(Self::New),
            "selected" => Some(Self::Selected),
            "dismissed" => Some(Self::Dismissed),
            "published" => Some(Self::Published),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LegislativeDossier {
    pub uid: DossierUid,
    pub title: String,
    pub procedure: String,
    pub legislature: u16,
    pub url: Option<String>,
    pub summary: Option<String>,
    pub last_activity_date: NaiveDate,
    pub last_activity_label: String,
    pub acts: Vec<LegislativeAct>,
    pub documents: Vec<LegislativeDocument>,
    pub score: Score,
    pub current_stage: Option<LegislativeStage>,
    pub initiators: Vec<Initiator>,
    pub committee: Option<Committee>,
    pub curation_status: CurationStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dossier_uid_rejects_empty() {
        assert!(DossierUid::new("".into()).is_err());
    }

    #[test]
    fn dossier_uid_accepts_valid() {
        let uid = DossierUid::new("DLR5L17N12345".into()).unwrap();
        assert_eq!(uid.as_str(), "DLR5L17N12345");
    }

    #[test]
    fn score_rejects_out_of_range() {
        assert!(Score::new(11, 5, 5, 50).is_err());
        assert!(Score::new(5, 11, 5, 50).is_err());
        assert!(Score::new(5, 5, 11, 50).is_err());
        assert!(Score::new(5, 5, 5, 101).is_err());
    }

    #[test]
    fn score_accepts_valid() {
        let s = Score::new(10, 10, 10, 100).unwrap();
        assert_eq!(s.progress(), 10);
        assert_eq!(s.magnitude(), 10);
        assert_eq!(s.momentum(), 10);
        assert_eq!(s.total(), 100);
    }

    #[test]
    fn score_accepts_zero() {
        let s = Score::new(0, 0, 0, 0).unwrap();
        assert_eq!(s.total(), 0);
    }

    #[test]
    fn initiator_rejects_empty_name() {
        assert!(Initiator::new("".into(), None).is_err());
    }

    #[test]
    fn initiator_accepts_valid() {
        let i = Initiator::new("Jean Dupont".into(), Some("RE".into())).unwrap();
        assert_eq!(i.full_name(), "Jean Dupont");
        assert_eq!(i.group(), Some("RE"));
    }

    #[test]
    fn committee_rejects_empty() {
        assert!(Committee::new("".into()).is_err());
    }

    #[test]
    fn committee_accepts_valid() {
        let c = Committee::new("Finances".into()).unwrap();
        assert_eq!(c.as_str(), "Finances");
    }

    #[test]
    fn curation_status_roundtrip() {
        for status in [
            CurationStatus::New,
            CurationStatus::Selected,
            CurationStatus::Dismissed,
            CurationStatus::Published,
        ] {
            assert_eq!(CurationStatus::parse(status.as_str()), Some(status));
        }
    }

    #[test]
    fn curation_status_parse_invalid() {
        assert_eq!(CurationStatus::parse("invalid"), None);
    }

    #[test]
    fn stage_from_known_codes() {
        assert_eq!(
            LegislativeStage::from_code("AN1"),
            Some(LegislativeStage::FirstReadingAN)
        );
        assert_eq!(
            LegislativeStage::from_code("CMP"),
            Some(LegislativeStage::JointCommittee)
        );
        assert_eq!(
            LegislativeStage::from_code("PROM"),
            Some(LegislativeStage::Promulgation)
        );
    }

    #[test]
    fn stage_from_unknown_code_returns_none() {
        assert_eq!(LegislativeStage::from_code("UNKNOWN"), None);
        assert_eq!(LegislativeStage::from_code("AN1-DEPOT"), None);
    }

    #[test]
    fn stages_are_ordered_by_progression() {
        assert!(LegislativeStage::FirstReadingAN < LegislativeStage::FirstReadingSenate);
        assert!(LegislativeStage::JointCommittee < LegislativeStage::Promulgation);
    }

    #[test]
    fn chamber_assignment() {
        assert_eq!(
            LegislativeStage::FirstReadingAN.chamber(),
            Chamber::AssembleeNationale
        );
        assert_eq!(
            LegislativeStage::FirstReadingSenate.chamber(),
            Chamber::Senat
        );
        assert_eq!(LegislativeStage::JointCommittee.chamber(), Chamber::Joint);
        assert_eq!(LegislativeStage::Promulgation.chamber(), Chamber::None);
    }
}
