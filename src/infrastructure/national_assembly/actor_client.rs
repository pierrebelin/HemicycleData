use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read};

use async_trait::async_trait;

use crate::application::ports::actor_source::{ActorSource, SourceError};
use crate::domain::actor::{
    Actor, ActorRegistry, ActorRole, ActorUid, GroupMembership, GroupUid, MembershipPeriod,
    MembershipQuality, ParliamentaryGroup,
};

use super::archive_fetcher::ArchiveFetcher;
use super::actor_parsing::{
    mandates, parse_date, text, RawActorWrapper, RawOrganeWrapper, ASSEMBLY_MANDATE_CODE,
    GROUP_ORGANE_CODE, MINISTRY_MANDATE_CODE, SENATE_MANDATE_CODE,
};

/// Jeu historique: tous les acteurs, tous leurs mandats, tous les organes.
///
/// RM-05 impose ce jeu et non celui des « deputes en exercice »: ce dernier
/// laisse sans reponse les acteurs qui ont quitte l'Assemblee en cours de
/// legislature.
const REGISTRY_URL: &str = "https://data.assemblee-nationale.fr/static/openData/repository/17/amo/tous_acteurs_mandats_organes_xi_legislature/AMO30_tous_acteurs_tous_mandats_tous_organes_historique.json.zip";

const ACTOR_PATH: &str = "acteur/";
const ORGANE_PATH: &str = "organe/";

pub struct AmoActorClient {
    archive: ArchiveFetcher,
}

impl AmoActorClient {
    pub fn new() -> Self {
        Self {
            archive: ArchiveFetcher::new(REGISTRY_URL, "actor registry"),
        }
    }

    async fn get_zip(&self) -> Result<bytes::Bytes, SourceError> {
        self.archive.fetch().await
    }

    /// Groupes parlementaires de la legislature demandee (RM-07).
    ///
    /// Les non-inscrits en font partie: la source les publie comme un organe de
    /// type GP a part entiere (RM-03).
    fn parse_groups(
        archive: &mut zip::ZipArchive<Cursor<&[u8]>>,
        legislature: u16,
    ) -> Vec<ParliamentaryGroup> {
        let mut groups = Vec::new();

        for i in 0..archive.len() {
            let Ok(mut file) = archive.by_index(i) else {
                continue;
            };

            let name = file.name().to_string();
            if !name.contains(ORGANE_PATH) || !name.ends_with(".json") {
                continue;
            }

            let mut content = String::new();
            if file.read_to_string(&mut content).is_err() {
                continue;
            }

            let Ok(wrapper) = serde_json::from_str::<RawOrganeWrapper>(&content) else {
                continue;
            };
            let raw = wrapper.organe;

            if raw.code_type.as_deref() != Some(GROUP_ORGANE_CODE) {
                continue;
            }
            if raw.legislature.as_deref().and_then(|l| l.parse::<u16>().ok()) != Some(legislature) {
                continue;
            }

            let Ok(uid) = GroupUid::new(raw.uid.clone()) else {
                continue;
            };

            let label = text(&raw.libelle).unwrap_or_default().to_string();
            let abbrev = text(&raw.libelle_abrev)
                .or_else(|| text(&raw.libelle_abrege))
                .unwrap_or_default()
                .to_string();

            let (start_date, end_date) = raw
                .vie_mode
                .as_ref()
                .map(|v| {
                    (
                        parse_date(v.date_debut.as_deref()),
                        parse_date(v.date_fin.as_deref()),
                    )
                })
                .unwrap_or((None, None));

            match ParliamentaryGroup::new(
                uid,
                legislature,
                label,
                abbrev,
                text(&raw.couleur_associee).map(String::from),
                start_date,
                end_date,
            ) {
                Ok(group) => groups.push(group),
                Err(e) => tracing::warn!("Skipping group {}: {e}", raw.uid),
            }
        }

        groups
    }

    /// Acteurs et appartenances.
    ///
    /// L'identite des acteurs est conservee sans filtre de legislature: un
    /// initiateur senateur ou ministre doit garder son nom (RM-04), faute de
    /// quoi le site afficherait un identifiant brut. RM-07 s'applique aux
    /// groupes et aux appartenances, qui sont bien restreints.
    fn parse_actors(
        archive: &mut zip::ZipArchive<Cursor<&[u8]>>,
        legislature: u16,
        known_groups: &HashSet<String>,
    ) -> (Vec<Actor>, Vec<GroupMembership>, usize) {
        let mut actors = Vec::new();
        let mut memberships = Vec::new();
        let mut orphan_memberships = 0;
        let target = legislature.to_string();

        for i in 0..archive.len() {
            let Ok(mut file) = archive.by_index(i) else {
                continue;
            };

            let name = file.name().to_string();
            if !name.contains(ACTOR_PATH) || !name.ends_with(".json") {
                continue;
            }

            let mut content = String::new();
            if file.read_to_string(&mut content).is_err() {
                continue;
            }

            let Ok(wrapper) = serde_json::from_str::<RawActorWrapper>(&content) else {
                tracing::warn!("Skipping unreadable actor file {name}");
                continue;
            };
            let raw = wrapper.acteur;

            let Ok(uid) = ActorUid::new(raw.uid.as_str().to_string()) else {
                continue;
            };

            let ident = raw.etat_civil.as_ref().and_then(|e| e.ident.as_ref());
            let civility = ident.and_then(|i| text(&i.civ)).map(String::from);
            let first_name = ident
                .and_then(|i| text(&i.prenom))
                .unwrap_or_default()
                .to_string();
            let last_name = ident
                .and_then(|i| text(&i.nom))
                .unwrap_or_default()
                .to_string();

            let mandats = mandates(&raw.mandats);
            let role = Self::infer_role(mandats, &target);

            match Actor::new(uid.clone(), civility, first_name, last_name, role) {
                Ok(actor) => actors.push(actor),
                Err(e) => {
                    tracing::warn!("Skipping actor {uid}: {e}");
                    continue;
                }
            }

            for mandat in mandats {
                if mandat.type_organe.as_deref() != Some(GROUP_ORGANE_CODE) {
                    continue;
                }
                if mandat.legislature.as_deref() != Some(target.as_str()) {
                    continue;
                }

                let Some(group_ref) = mandat.organes.as_ref().and_then(|o| o.first_ref()) else {
                    orphan_memberships += 1;
                    continue;
                };
                if !known_groups.contains(group_ref) {
                    orphan_memberships += 1;
                    tracing::warn!("Membership {} points to unknown group {group_ref}", mandat.uid);
                    continue;
                }

                let Some(start) = parse_date(mandat.date_debut.as_deref()) else {
                    orphan_memberships += 1;
                    tracing::warn!("Membership {} has no usable start date", mandat.uid);
                    continue;
                };
                let end = parse_date(mandat.date_fin.as_deref());

                let Ok(period) = MembershipPeriod::new(start, end) else {
                    orphan_memberships += 1;
                    tracing::warn!("Membership {} has a reversed period", mandat.uid);
                    continue;
                };

                // RM-02: la qualite est conservee telle quelle, aucune n'est filtree.
                let quality_code = mandat
                    .infos_qualite
                    .as_ref()
                    .and_then(|q| text(&q.code_qualite))
                    .unwrap_or("Membre");
                let Ok(quality) = MembershipQuality::new(quality_code.to_string()) else {
                    orphan_memberships += 1;
                    continue;
                };

                let Ok(group_uid) = GroupUid::new(group_ref.to_string()) else {
                    orphan_memberships += 1;
                    continue;
                };

                memberships.push(GroupMembership::new(
                    mandat.uid.clone(),
                    uid.clone(),
                    group_uid,
                    legislature,
                    period,
                    quality,
                ));
            }
        }

        (actors, memberships, orphan_memberships)
    }

    fn infer_role(mandats: &[super::actor_parsing::RawMandat], legislature: &str) -> ActorRole {
        let mut has_ministry = false;
        let mut has_senate = false;

        for mandat in mandats {
            match mandat.type_organe.as_deref() {
                Some(ASSEMBLY_MANDATE_CODE)
                    if mandat.legislature.as_deref() == Some(legislature) =>
                {
                    return ActorRole::Deputy;
                }
                Some(MINISTRY_MANDATE_CODE) => has_ministry = true,
                Some(SENATE_MANDATE_CODE) => has_senate = true,
                _ => {}
            }
        }

        if has_ministry {
            ActorRole::Minister
        } else if has_senate {
            ActorRole::Senator
        } else {
            ActorRole::Other
        }
    }

    fn parse_registry(data: &[u8], legislature: u16) -> Result<ActorRegistry, SourceError> {
        let cursor = Cursor::new(data);
        let mut archive =
            zip::ZipArchive::new(cursor).map_err(|e| SourceError::Parse(e.to_string()))?;

        let groups = Self::parse_groups(&mut archive, legislature);
        let known_groups: HashSet<String> = groups
            .iter()
            .map(|g| g.uid().as_str().to_string())
            .collect();

        let (actors, memberships, orphans) =
            Self::parse_actors(&mut archive, legislature, &known_groups);

        if orphans > 0 {
            tracing::warn!("{orphans} memberships discarded: unusable at the source");
        }

        let by_group: HashMap<&str, usize> =
            memberships
                .iter()
                .fold(HashMap::new(), |mut acc, membership| {
                    *acc.entry(membership.group_uid().as_str()).or_default() += 1;
                    acc
                });

        tracing::info!(
            "Registry parsed: {} actors, {} groups (legislature {legislature}), {} memberships across {} groups",
            actors.len(),
            groups.len(),
            memberships.len(),
            by_group.len(),
        );

        Ok(ActorRegistry {
            actors,
            groups,
            memberships,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifie le parseur contre l'archive officielle.
    ///
    /// Ignore par defaut: telecharger l'archive rendrait la suite dependante du
    /// reseau. Pour l'executer:
    ///   AMO30_ZIP=/chemin/AMO30....zip cargo test -- --ignored
    #[test]
    #[ignore]
    fn parses_the_official_archive() {
        let path = std::env::var("AMO30_ZIP").expect("AMO30_ZIP must point to the archive");
        let data = std::fs::read(path).expect("archive must be readable");

        let registry = AmoActorClient::parse_registry(&data, 17).unwrap();

        assert!(!registry.actors.is_empty());
        assert!(!registry.groups.is_empty());
        assert!(!registry.memberships.is_empty());

        // RM-03: les non-inscrits sont un groupe, pas une absence d'appartenance.
        assert!(registry
            .groups
            .iter()
            .any(|g| g.label().to_lowercase().contains("non inscrit")));

        // RM-02: les presidents de groupe restent dans leur groupe.
        assert!(registry
            .memberships
            .iter()
            .any(|m| m.quality().as_str() == "Pr\u{00e9}sident"));

        // RM-07: aucune appartenance d'une autre legislature.
        assert!(registry.memberships.iter().all(|m| m.legislature() == 17));

        // Toute appartenance pointe vers un groupe connu.
        let known: HashSet<&str> = registry.groups.iter().map(|g| g.uid().as_str()).collect();
        assert!(registry
            .memberships
            .iter()
            .all(|m| known.contains(m.group_uid().as_str())));
    }
}

#[async_trait]
impl ActorSource for AmoActorClient {
    async fn fetch_registry(&self, legislature: u16) -> Result<ActorRegistry, SourceError> {
        let zip_data = self.get_zip().await?;

        tokio::task::spawn_blocking(move || Self::parse_registry(&zip_data, legislature))
            .await
            .map_err(|e| SourceError::Parse(e.to_string()))?
    }
}
