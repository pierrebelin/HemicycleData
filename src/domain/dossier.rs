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
    pub full_name: String,
    pub group: Option<String>,
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
    pub current_stage: Option<LegislativeStage>,
    pub initiators: Vec<Initiator>,
    pub committee: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
