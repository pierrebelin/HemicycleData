use crate::application::ports::final_vote_repository::{
    FinalVoteFilter, FinalVoteRecord, FinalVoteRepository, GroupOption, GroupTallyRecord,
    RepositoryError,
};
use crate::application::ports::theme_repository::AssignedFamily;
use crate::domain::actor::GroupUid;
use crate::domain::final_vote::{reading_of, FinalVote, GroupIdentity, GroupStance};
use crate::domain::scrutin::{Outcome, VotePosition};
use crate::domain::theme::FamilyCode;

/// Deux groupes au plus. Au-dela, la page cesse d'etre une comparaison lisible
/// et devient un tableau de bord comparatif — soit exactement l'agregat qui se
/// lit comme un classement, interdit par PROJECT.md §6.
pub const MAX_COMPARED_GROUPS: usize = 2;
pub const DEFAULT_PAGE_SIZE: i64 = 20;
pub const MAX_PAGE_SIZE: i64 = 100;

#[derive(Debug, Clone, Default)]
pub struct BrowseFinalVotesCommand {
    pub family: Option<String>,
    /// Groupes demandes, par identifiant ou par sigle, dans l'ordre d'affichage.
    pub groups: Vec<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, thiserror::Error)]
pub enum BrowseFinalVotesError {
    #[error("at most {MAX_COMPARED_GROUPS} groups can be compared, got {0}")]
    TooManyGroups(usize),
    #[error("unknown parliamentary group: {0}")]
    UnknownGroup(String),
    #[error("unknown theme family: {0}")]
    UnknownFamily(String),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

/// Un vote et ses rattachements thematiques.
///
/// Les familles restent hors de l'agregat: elles viennent du texte debattu, pas
/// du vote, et le domaine du vote n'a pas a en dependre.
#[derive(Debug, Clone)]
pub struct FinalVoteEntry {
    pub vote: FinalVote,
    pub families: Vec<AssignedFamily>,
}

#[derive(Debug, Clone)]
pub struct FinalVoteView {
    pub items: Vec<FinalVoteEntry>,
    /// Votes correspondant au filtre.
    pub total: i64,
    /// Votes sur l'ensemble, filtre thematique exclu. Rend visible ce que le
    /// filtre laisse de cote (PROJECT.md §2).
    pub total_unfiltered: i64,
    /// Votes sur l'ensemble deja rattaches a une famille. Un filtre par theme
    /// ne peut rien trouver au-dela: l'avancement de la thematisation est
    /// affiche plutot que subi.
    pub total_with_family: i64,
    /// Referentiel complet, pour le selecteur de groupes.
    pub groups: Vec<GroupOption>,
    /// Groupes retenus, dans l'ordre demande.
    pub selected: Vec<GroupOption>,
}

/// CU-07 — Consulter les votes sur l'ensemble d'un texte, groupe par groupe.
pub struct BrowseFinalVotes<'a> {
    repository: &'a dyn FinalVoteRepository,
}

impl<'a> BrowseFinalVotes<'a> {
    pub fn new(repository: &'a dyn FinalVoteRepository) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        command: BrowseFinalVotesCommand,
    ) -> Result<FinalVoteView, BrowseFinalVotesError> {
        if command.groups.len() > MAX_COMPARED_GROUPS {
            return Err(BrowseFinalVotesError::TooManyGroups(command.groups.len()));
        }

        let family = match command.family.as_deref().filter(|f| !f.is_empty()) {
            Some(raw) => Some(
                FamilyCode::parse(raw)
                    .map_err(|_| BrowseFinalVotesError::UnknownFamily(raw.to_string()))?,
            ),
            None => None,
        };

        let groups = self.repository.groups().await?;
        let selected = resolve_groups(&groups, &command.groups)?;

        let filter = FinalVoteFilter {
            family,
            group_uids: selected.iter().map(|g| g.uid.clone()).collect(),
            limit: command
                .limit
                .unwrap_or(DEFAULT_PAGE_SIZE)
                .clamp(1, MAX_PAGE_SIZE),
            offset: command.offset.unwrap_or(0).max(0),
        };

        let page = self.repository.list_final_votes(&filter).await?;
        let totals = self.repository.totals().await?;

        let items = page
            .items
            .into_iter()
            .map(|record| build_entry(record, &selected))
            .collect();

        Ok(FinalVoteView {
            items,
            total: page.total,
            total_unfiltered: totals.total,
            total_with_family: totals.with_family,
            groups,
            selected,
        })
    }
}

/// Un groupe est designe par son identifiant ou par son sigle. Le sigle rend
/// l'adresse partageable (`?groupes=RN,SOC`), l'identifiant la rend stable
/// (PROJECT.md §8.1).
fn resolve_groups(
    known: &[GroupOption],
    requested: &[String],
) -> Result<Vec<GroupOption>, BrowseFinalVotesError> {
    requested
        .iter()
        .filter(|token| !token.trim().is_empty())
        .map(|token| {
            let token = token.trim();
            known
                .iter()
                .find(|group| {
                    group.uid == token || group.abbrev.eq_ignore_ascii_case(token)
                })
                .cloned()
                .ok_or_else(|| BrowseFinalVotesError::UnknownGroup(token.to_string()))
        })
        .collect()
}

fn build_entry(record: FinalVoteRecord, selected: &[GroupOption]) -> FinalVoteEntry {
    // L'ordre des positions suit l'ordre demande, pas celui de la base: la
    // colonne de gauche doit rester le groupe choisi en premier.
    let stances = selected
        .iter()
        .filter_map(|group| {
            record
                .tallies
                .iter()
                .find(|tally| tally.group_uid == group.uid)
                .map(build_stance)
        })
        .collect();

    let outcome = Outcome::new(record.outcome_code, record.outcome_label)
        .expect("outcome codes come from the database, never empty");

    let vote = FinalVote {
        reading: reading_of(&record.subject, &record.text_label),
        scrutin_uid: record.scrutin_uid,
        number: record.number,
        date: record.date,
        ballot_type_label: record.ballot_type_label,
        outcome,
        text_key: record.text_key,
        text_label: record.text_label,
        dossier_uid: record.dossier_uid,
        dossier_label: record.dossier_label,
        synthesis: record.synthesis,
        stances,
    };

    FinalVoteEntry {
        vote,
        families: record.families,
    }
}

fn build_stance(record: &GroupTallyRecord) -> GroupStance {
    let identity = GroupIdentity {
        uid: GroupUid::new(record.group_uid.clone())
            .expect("group uids come from the referential, never empty"),
        abbrev: record.abbrev.clone(),
        label: record.label.clone(),
        color: record.color.clone(),
    };

    GroupStance::new(
        identity,
        record.member_count,
        record
            .majority_position
            .as_deref()
            .and_then(VotePosition::parse),
        record.tally.clone(),
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;
    use chrono::NaiveDate;

    use super::*;
    use crate::application::ports::final_vote_repository::{FinalVotePage, FinalVoteTotals};
    use crate::domain::scrutin::VoteTally;

    #[derive(Default)]
    struct InMemoryFinalVoteRepository {
        records: Mutex<Vec<FinalVoteRecord>>,
        groups: Mutex<Vec<GroupOption>>,
        last_filter: Mutex<Option<FinalVoteFilter>>,
    }

    #[async_trait]
    impl FinalVoteRepository for InMemoryFinalVoteRepository {
        async fn list_final_votes(
            &self,
            filter: &FinalVoteFilter,
        ) -> Result<FinalVotePage, RepositoryError> {
            *self.last_filter.lock().unwrap() = Some(filter.clone());
            let records = self.records.lock().unwrap();
            let items: Vec<FinalVoteRecord> = records
                .iter()
                .filter(|record| match filter.family {
                    Some(family) => record.families.iter().any(|f| f.family == family),
                    None => true,
                })
                .cloned()
                .collect();
            let total = items.len() as i64;
            Ok(FinalVotePage {
                items: items
                    .into_iter()
                    .skip(filter.offset as usize)
                    .take(filter.limit as usize)
                    .collect(),
                total,
            })
        }

        async fn groups(&self) -> Result<Vec<GroupOption>, RepositoryError> {
            Ok(self.groups.lock().unwrap().clone())
        }

        async fn totals(&self) -> Result<FinalVoteTotals, RepositoryError> {
            let records = self.records.lock().unwrap();
            Ok(FinalVoteTotals {
                total: records.len() as i64,
                with_family: records.iter().filter(|r| !r.families.is_empty()).count() as i64,
            })
        }
    }

    fn group(uid: &str, abbrev: &str) -> GroupOption {
        GroupOption {
            uid: uid.to_string(),
            abbrev: abbrev.to_string(),
            label: format!("Groupe {abbrev}"),
            color: None,
            final_vote_count: 1,
        }
    }

    fn tally_record(uid: &str, abbrev: &str, votes_for: u16, votes_against: u16) -> GroupTallyRecord {
        GroupTallyRecord {
            group_uid: uid.to_string(),
            abbrev: abbrev.to_string(),
            label: format!("Groupe {abbrev}"),
            color: None,
            member_count: Some(60),
            majority_position: Some(
                if votes_for >= votes_against { "for" } else { "against" }.to_string(),
            ),
            tally: VoteTally {
                votes_for,
                votes_against,
                abstentions: 0,
                not_voting: 0,
                voluntary_not_voting: 0,
            },
        }
    }

    fn record(uid: &str, tallies: Vec<GroupTallyRecord>) -> FinalVoteRecord {
        FinalVoteRecord {
            scrutin_uid: uid.to_string(),
            number: "42".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 5, 27).unwrap(),
            subject: "l'ensemble de la proposition de loi sur le logement (première lecture)."
                .to_string(),
            ballot_type_label: "scrutin public solennel".to_string(),
            outcome_code: "adopté".to_string(),
            outcome_label: "Adopté".to_string(),
            text_key: "proposition de loi sur le logement".to_string(),
            text_label: "proposition de loi sur le logement".to_string(),
            dossier_uid: None,
            dossier_label: None,
            synthesis: VoteTally {
                votes_for: 300,
                votes_against: 200,
                abstentions: 10,
                not_voting: 2,
                voluntary_not_voting: 0,
            },
            families: Vec::new(),
            tallies,
        }
    }

    fn repository() -> InMemoryFinalVoteRepository {
        let repository = InMemoryFinalVoteRepository::default();
        *repository.groups.lock().unwrap() = vec![group("PO1", "RN"), group("PO2", "SOC")];
        *repository.records.lock().unwrap() = vec![record(
            "S1",
            vec![
                tally_record("PO1", "RN", 2, 84),
                tally_record("PO2", "SOC", 60, 0),
            ],
        )];
        repository
    }

    #[tokio::test]
    async fn two_groups_are_returned_in_the_requested_order() {
        let repository = repository();

        let view = BrowseFinalVotes::new(&repository)
            .execute(BrowseFinalVotesCommand {
                groups: vec!["SOC".to_string(), "RN".to_string()],
                ..Default::default()
            })
            .await
            .unwrap();

        let stances = &view.items[0].vote.stances;
        assert_eq!(stances.len(), 2);
        assert_eq!(stances[0].group.abbrev, "SOC");
        assert_eq!(stances[1].group.abbrev, "RN");
    }

    #[tokio::test]
    async fn a_group_is_found_by_uid_as_well_as_by_abbrev() {
        let repository = repository();

        let view = BrowseFinalVotes::new(&repository)
            .execute(BrowseFinalVotesCommand {
                groups: vec!["PO1".to_string()],
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(view.selected[0].abbrev, "RN");
    }

    #[tokio::test]
    async fn the_share_is_computed_on_voters() {
        let repository = repository();

        let view = BrowseFinalVotes::new(&repository)
            .execute(BrowseFinalVotesCommand {
                groups: vec!["RN".to_string()],
                ..Default::default()
            })
            .await
            .unwrap();

        let share = view.items[0].vote.stances[0].share.clone().unwrap();
        assert_eq!(share.voters, 86);
        assert_eq!(share.against_percent, 98);
        assert_eq!(share.for_percent, 2);
    }

    #[tokio::test]
    async fn a_group_absent_from_a_scrutin_yields_no_stance() {
        let repository = repository();
        *repository.groups.lock().unwrap() = vec![
            group("PO1", "RN"),
            group("PO2", "SOC"),
            group("PO3", "UDR"),
        ];

        let view = BrowseFinalVotes::new(&repository)
            .execute(BrowseFinalVotesCommand {
                groups: vec!["UDR".to_string()],
                ..Default::default()
            })
            .await
            .unwrap();

        // Le groupe est bien selectionne, mais le scrutin ne porte aucune ligne
        // pour lui: rien n'est invente, l'absence reste visible a l'affichage.
        assert_eq!(view.selected.len(), 1);
        assert!(view.items[0].vote.stances.is_empty());
    }

    #[tokio::test]
    async fn an_unknown_group_is_refused_rather_than_ignored() {
        let repository = repository();

        let error = BrowseFinalVotes::new(&repository)
            .execute(BrowseFinalVotesCommand {
                groups: vec!["PS".to_string()],
                ..Default::default()
            })
            .await
            .unwrap_err();

        assert!(matches!(error, BrowseFinalVotesError::UnknownGroup(g) if g == "PS"));
    }

    #[tokio::test]
    async fn comparing_more_than_two_groups_is_refused() {
        let repository = repository();

        let error = BrowseFinalVotes::new(&repository)
            .execute(BrowseFinalVotesCommand {
                groups: vec!["RN".to_string(), "SOC".to_string(), "PO1".to_string()],
                ..Default::default()
            })
            .await
            .unwrap_err();

        assert!(matches!(error, BrowseFinalVotesError::TooManyGroups(3)));
    }

    #[tokio::test]
    async fn an_unknown_family_is_refused() {
        let repository = repository();

        let error = BrowseFinalVotes::new(&repository)
            .execute(BrowseFinalVotesCommand {
                family: Some("agriculture".to_string()),
                ..Default::default()
            })
            .await
            .unwrap_err();

        assert!(matches!(error, BrowseFinalVotesError::UnknownFamily(f) if f == "agriculture"));
    }

    #[tokio::test]
    async fn the_page_size_stays_within_bounds() {
        let repository = repository();

        BrowseFinalVotes::new(&repository)
            .execute(BrowseFinalVotesCommand {
                limit: Some(5_000),
                offset: Some(-3),
                ..Default::default()
            })
            .await
            .unwrap();

        let filter = repository.last_filter.lock().unwrap().clone().unwrap();
        assert_eq!(filter.limit, MAX_PAGE_SIZE);
        assert_eq!(filter.offset, 0);
    }

    #[tokio::test]
    async fn the_unfiltered_total_stays_visible_under_a_theme_filter() {
        let repository = repository();
        // Un vote rattache au logement, un autre sans famille: le filtre en
        // retient un, le total complet reste affichable.
        let mut themed = record("S2", Vec::new());
        themed.families = vec![AssignedFamily {
            family: FamilyCode::Logement,
            origin: crate::domain::theme::AssignmentOrigin::HumanArbitration,
            opened_on: NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
            motive: None,
        }];
        repository.records.lock().unwrap().push(themed);

        let view = BrowseFinalVotes::new(&repository)
            .execute(BrowseFinalVotesCommand {
                family: Some("logement".to_string()),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(view.total, 1);
        assert_eq!(view.total_unfiltered, 2);
        assert_eq!(view.total_with_family, 1);
    }
}
