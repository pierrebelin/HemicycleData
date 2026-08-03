use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::domain::actor::{
    ActorDirectory, ActorRole, ActorUid, GroupUid, MembershipQuality, ParliamentaryGroup,
};

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("dossier uid must not be empty")]
    EmptyDossierUid,
    #[error("an initiator group must always carry its reference date")]
    GroupWithoutReferenceDate,
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

/// Groupe d'un initiateur, tel qu'il etait a la date de depot du dossier.
#[derive(Debug, Clone, Serialize)]
pub struct InitiatorGroup {
    pub uid: GroupUid,
    pub abbrev: String,
    pub label: String,
    pub quality: Option<MembershipQuality>,
}

impl InitiatorGroup {
    pub fn from_group(group: &ParliamentaryGroup, quality: Option<MembershipQuality>) -> Self {
        Self {
            uid: group.uid().clone(),
            abbrev: group.abbrev().to_string(),
            label: group.label().to_string(),
            quality,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Initiator {
    full_name: String,
    actor_uid: Option<ActorUid>,
    role: Option<ActorRole>,
    group: Option<InitiatorGroup>,
    reference_date: Option<NaiveDate>,
    official_url: Option<String>,
}

impl Initiator {
    /// Initiateur non resolu: le nom brut est conserve, aucun groupe n'est
    /// devine (RM-04).
    pub fn unresolved(full_name: String) -> Result<Self, DomainError> {
        if full_name.is_empty() {
            return Err(DomainError::EmptyInitiatorName);
        }
        Ok(Self {
            full_name,
            actor_uid: None,
            role: None,
            group: None,
            reference_date: None,
            official_url: None,
        })
    }

    /// Initiateur rattache au referentiel.
    ///
    /// RM-01: un groupe ne peut etre porte sans la date a laquelle il a ete lu.
    pub fn resolved(
        full_name: String,
        actor_uid: ActorUid,
        role: ActorRole,
        group: Option<InitiatorGroup>,
        reference_date: Option<NaiveDate>,
        official_url: Option<String>,
    ) -> Result<Self, DomainError> {
        if full_name.is_empty() {
            return Err(DomainError::EmptyInitiatorName);
        }
        if group.is_some() && reference_date.is_none() {
            return Err(DomainError::GroupWithoutReferenceDate);
        }
        Ok(Self {
            full_name,
            actor_uid: Some(actor_uid),
            role: Some(role),
            group,
            reference_date,
            official_url,
        })
    }

    pub fn full_name(&self) -> &str {
        &self.full_name
    }

    pub fn actor_uid(&self) -> Option<&ActorUid> {
        self.actor_uid.as_ref()
    }

    pub fn role(&self) -> Option<ActorRole> {
        self.role
    }

    pub fn group(&self) -> Option<&InitiatorGroup> {
        self.group.as_ref()
    }

    /// Date de l'acte a laquelle le groupe a ete lu. Toujours presente quand un
    /// groupe est affiche (RM-01).
    pub fn reference_date(&self) -> Option<NaiveDate> {
        self.reference_date
    }

    pub fn official_url(&self) -> Option<&str> {
        self.official_url.as_deref()
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
    /// Date de depot du texte. Date de reference du rattachement des
    /// initiateurs a leur groupe (RM-01).
    pub deposit_date: Option<NaiveDate>,
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

/// Ce qu'un rattachement d'initiateurs a produit, y compris ce qu'il n'a pas pu
/// produire. Toute lacune est comptee pour etre signalee plutot que silencieuse.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InitiatorAttachment {
    pub resolved: usize,
    /// Acteurs absents du referentiel: nom brut conserve, aucun groupe (RM-04).
    pub unknown_actors: usize,
    /// Acteurs connus sans appartenance valide a la date de depot.
    pub without_group: usize,
    /// Plusieurs groupes concurrents a la date de depot: rien n'est affiche.
    pub ambiguous: usize,
    /// Dossier sans date de depot: le groupe ne peut pas etre date (RM-01).
    pub missing_deposit_date: bool,
}

impl LegislativeDossier {
    /// Rattache chaque initiateur au groupe qu'il avait a la date de depot.
    ///
    /// Sans date de depot, l'identite de l'acteur est conservee mais aucun
    /// groupe n'est porte: un groupe non date reecrirait l'histoire (RM-01).
    pub fn attach_initiators(
        &mut self,
        actor_refs: &[ActorUid],
        directory: &ActorDirectory,
    ) -> InitiatorAttachment {
        let mut report = InitiatorAttachment {
            missing_deposit_date: self.deposit_date.is_none(),
            ..Default::default()
        };

        let mut initiators = Vec::with_capacity(actor_refs.len());

        for actor_ref in actor_refs {
            let resolved = self
                .deposit_date
                .and_then(|date| directory.resolve_at(actor_ref, date))
                .or_else(|| {
                    // Sans date de depot on lit quand meme l'identite: la
                    // resolution a une date arbitraire ne renverrait pas de
                    // groupe utilisable, seul l'acteur nous interesse ici.
                    directory
                        .actor(actor_ref)
                        .map(|actor| crate::domain::actor::ActorAtDate {
                            actor,
                            group: None,
                            quality: None,
                            ambiguous: false,
                        })
                });

            let Some(resolved) = resolved else {
                report.unknown_actors += 1;
                if let Ok(initiator) = Initiator::unresolved(actor_ref.as_str().to_string()) {
                    initiators.push(initiator);
                }
                continue;
            };

            if resolved.ambiguous {
                report.ambiguous += 1;
            } else if resolved.group.is_none() {
                report.without_group += 1;
            }

            let group = resolved
                .group
                .map(|g| InitiatorGroup::from_group(g, resolved.quality.cloned()));
            let reference_date = self.deposit_date;

            let initiator = Initiator::resolved(
                resolved.actor.full_name(),
                resolved.actor.uid().clone(),
                resolved.actor.role(),
                group.filter(|_| reference_date.is_some()),
                reference_date,
                resolved.actor.official_url(),
            );

            match initiator {
                Ok(initiator) => {
                    report.resolved += 1;
                    initiators.push(initiator);
                }
                Err(_) => {
                    report.unknown_actors += 1;
                    if let Ok(initiator) = Initiator::unresolved(actor_ref.as_str().to_string()) {
                        initiators.push(initiator);
                    }
                }
            }
        }

        self.initiators = initiators;
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actor::{
        Actor, GroupMembership, MembershipPeriod,
    };

    fn dossier_deposited_on(deposit_date: Option<NaiveDate>) -> LegislativeDossier {
        LegislativeDossier {
            uid: DossierUid::new("D1".into()).unwrap(),
            title: "Loi test".into(),
            procedure: "Proposition de loi".into(),
            legislature: 17,
            url: None,
            summary: None,
            deposit_date,
            last_activity_date: NaiveDate::from_ymd_opt(2025, 5, 13).unwrap(),
            last_activity_label: "D\u{00e9}p\u{00f4}t".into(),
            acts: vec![],
            documents: vec![],
            score: Score::new(2, 4, 2, 23).unwrap(),
            current_stage: None,
            initiators: vec![],
            committee: None,
            curation_status: CurationStatus::New,
        }
    }

    fn directory_with_group_change() -> ActorDirectory {
        let actor = Actor::new(
            ActorUid::new("PA1".into()).unwrap(),
            Some("M.".into()),
            "Jean".into(),
            "Dupont".into(),
            ActorRole::Deputy,
        )
        .unwrap();

        let group_a = ParliamentaryGroup::new(
            GroupUid::new("PO_A".into()).unwrap(),
            17,
            "Groupe A".into(),
            "A".into(),
            None,
            None,
            None,
        )
        .unwrap();
        let group_b = ParliamentaryGroup::new(
            GroupUid::new("PO_B".into()).unwrap(),
            17,
            "Groupe B".into(),
            "B".into(),
            None,
            None,
            None,
        )
        .unwrap();

        let membership = |source: &str, group: &str, start, end| {
            GroupMembership::new(
                source.into(),
                ActorUid::new("PA1".into()).unwrap(),
                GroupUid::new(group.into()).unwrap(),
                17,
                MembershipPeriod::new(start, end).unwrap(),
                MembershipQuality::new("Membre".into()).unwrap(),
            )
        };

        ActorDirectory::new(
            vec![actor],
            vec![group_a, group_b],
            vec![
                membership(
                    "PM1",
                    "PO_A",
                    NaiveDate::from_ymd_opt(2024, 7, 19).unwrap(),
                    Some(NaiveDate::from_ymd_opt(2025, 6, 30).unwrap()),
                ),
                membership(
                    "PM2",
                    "PO_B",
                    NaiveDate::from_ymd_opt(2025, 7, 1).unwrap(),
                    None,
                ),
            ],
        )
    }

    #[test]
    fn attaches_the_group_held_on_the_deposit_date_not_the_current_one() {
        let mut dossier =
            dossier_deposited_on(Some(NaiveDate::from_ymd_opt(2025, 5, 13).unwrap()));
        let directory = directory_with_group_change();
        let refs = vec![ActorUid::new("PA1".into()).unwrap()];

        let report = dossier.attach_initiators(&refs, &directory);

        assert_eq!(report.resolved, 1);
        let initiator = &dossier.initiators[0];
        assert_eq!(initiator.full_name(), "Jean Dupont");
        assert_eq!(initiator.group().unwrap().abbrev, "A");
        assert_eq!(
            initiator.reference_date(),
            Some(NaiveDate::from_ymd_opt(2025, 5, 13).unwrap())
        );
        assert_eq!(
            initiator.official_url(),
            Some("https://www.assemblee-nationale.fr/dyn/deputes/PA1")
        );
    }

    #[test]
    fn a_later_deposit_gets_the_later_group() {
        let mut dossier =
            dossier_deposited_on(Some(NaiveDate::from_ymd_opt(2025, 9, 1).unwrap()));
        let directory = directory_with_group_change();
        let refs = vec![ActorUid::new("PA1".into()).unwrap()];

        dossier.attach_initiators(&refs, &directory);

        assert_eq!(dossier.initiators[0].group().unwrap().abbrev, "B");
    }

    #[test]
    fn unknown_actor_keeps_its_raw_reference_and_no_group() {
        let mut dossier =
            dossier_deposited_on(Some(NaiveDate::from_ymd_opt(2025, 5, 13).unwrap()));
        let directory = directory_with_group_change();
        let refs = vec![ActorUid::new("PA_UNKNOWN".into()).unwrap()];

        let report = dossier.attach_initiators(&refs, &directory);

        assert_eq!(report.unknown_actors, 1);
        assert_eq!(dossier.initiators[0].full_name(), "PA_UNKNOWN");
        assert!(dossier.initiators[0].group().is_none());
    }

    #[test]
    fn without_deposit_date_the_actor_is_named_but_carries_no_group() {
        let mut dossier = dossier_deposited_on(None);
        let directory = directory_with_group_change();
        let refs = vec![ActorUid::new("PA1".into()).unwrap()];

        let report = dossier.attach_initiators(&refs, &directory);

        assert!(report.missing_deposit_date);
        let initiator = &dossier.initiators[0];
        assert_eq!(initiator.full_name(), "Jean Dupont");
        assert!(initiator.group().is_none());
        assert!(initiator.reference_date().is_none());
    }

    #[test]
    fn actor_outside_every_membership_period_is_reported_without_group() {
        let mut dossier =
            dossier_deposited_on(Some(NaiveDate::from_ymd_opt(2024, 7, 1).unwrap()));
        let directory = directory_with_group_change();
        let refs = vec![ActorUid::new("PA1".into()).unwrap()];

        let report = dossier.attach_initiators(&refs, &directory);

        assert_eq!(report.without_group, 1);
        assert!(dossier.initiators[0].group().is_none());
        assert_eq!(dossier.initiators[0].full_name(), "Jean Dupont");
    }

    #[test]
    fn attaching_twice_replaces_instead_of_accumulating() {
        let mut dossier =
            dossier_deposited_on(Some(NaiveDate::from_ymd_opt(2025, 5, 13).unwrap()));
        let directory = directory_with_group_change();
        let refs = vec![ActorUid::new("PA1".into()).unwrap()];

        dossier.attach_initiators(&refs, &directory);
        dossier.attach_initiators(&refs, &directory);

        assert_eq!(dossier.initiators.len(), 1);
    }

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
        assert!(Initiator::unresolved("".into()).is_err());
    }

    #[test]
    fn unresolved_initiator_keeps_the_raw_name_without_group() {
        let i = Initiator::unresolved("PA999999".into()).unwrap();
        assert_eq!(i.full_name(), "PA999999");
        assert!(i.group().is_none());
        assert!(i.reference_date().is_none());
        assert!(i.official_url().is_none());
    }

    #[test]
    fn resolved_initiator_carries_group_and_reference_date() {
        let group = ParliamentaryGroup::new(
            GroupUid::new("PO845414".into()).unwrap(),
            17,
            "Ensemble pour la R\u{00e9}publique".into(),
            "EPR".into(),
            None,
            None,
            None,
        )
        .unwrap();
        let quality = MembershipQuality::new("Membre".into()).unwrap();
        let reference = NaiveDate::from_ymd_opt(2025, 5, 13).unwrap();

        let i = Initiator::resolved(
            "Jean Dupont".into(),
            ActorUid::new("PA1".into()).unwrap(),
            ActorRole::Deputy,
            Some(InitiatorGroup::from_group(&group, Some(quality))),
            Some(reference),
            Some("https://www.assemblee-nationale.fr/dyn/deputes/PA1".into()),
        )
        .unwrap();

        assert_eq!(i.group().unwrap().abbrev, "EPR");
        assert_eq!(i.reference_date(), Some(reference));
        assert_eq!(i.role(), Some(ActorRole::Deputy));
    }

    #[test]
    fn resolved_initiator_rejects_group_without_reference_date() {
        let group = ParliamentaryGroup::new(
            GroupUid::new("PO845414".into()).unwrap(),
            17,
            "Ensemble pour la R\u{00e9}publique".into(),
            "EPR".into(),
            None,
            None,
            None,
        )
        .unwrap();

        let result = Initiator::resolved(
            "Jean Dupont".into(),
            ActorUid::new("PA1".into()).unwrap(),
            ActorRole::Deputy,
            Some(InitiatorGroup::from_group(&group, None)),
            None,
            None,
        );

        assert!(matches!(
            result,
            Err(DomainError::GroupWithoutReferenceDate)
        ));
    }

    #[test]
    fn minister_initiator_is_resolved_without_group() {
        let i = Initiator::resolved(
            "Jean Dupont".into(),
            ActorUid::new("PA2".into()).unwrap(),
            ActorRole::Minister,
            None,
            Some(NaiveDate::from_ymd_opt(2025, 5, 13).unwrap()),
            None,
        )
        .unwrap();

        assert!(i.group().is_none());
        assert_eq!(i.role(), Some(ActorRole::Minister));
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
