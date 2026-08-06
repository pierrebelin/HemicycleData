//! Groupes renommes en cours de legislature.
//!
//! Le referentiel de l'Assemblee ouvre un nouvel identifiant quand un groupe
//! change de nom, sans jamais relier l'ancien au nouveau. Les deux apparaissent
//! donc cote a cote dans le selecteur, chacun muet sur la periode de l'autre:
//! choisir l'ancien nom affiche « aucune ligne » sur tous les votes recents.
//!
//! Rapprocher les deux identifiants est une decision editoriale, pas une
//! deduction: elle est ecrite ici en toutes lettres, verifiable ligne a ligne,
//! plutot que devinee a l'execution sur une ressemblance de sigle ou de
//! couleur (PROJECT.md §8).
//!
//! La lignee vaut pour l'affichage — un groupe, une colonne, sur toute la
//! legislature. Elle ne modifie rien en base: les ventilations restent portees
//! par l'identifiant que la source leur donne.

/// Un meme groupe parlementaire sous ses identifiants successifs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GroupLineage {
    /// Identifiant retenu pour la lignee: celui de la periode en cours.
    pub canonical_uid: &'static str,
    /// Tous les identifiants de la lignee, le canonique compris.
    pub uids: &'static [&'static str],
    /// Sigle affiche: le plus recent, celui que la source emploie aujourd'hui.
    pub abbrev: &'static str,
    pub label: &'static str,
    /// Sigles anterieurs. Une adresse partagee avec l'ancien sigle continue de
    /// designer le groupe (PROJECT.md §8.1).
    pub former_abbrevs: &'static [&'static str],
}

impl GroupLineage {
    pub fn contains_uid(&self, uid: &str) -> bool {
        self.uids.contains(&uid)
    }

    /// Vrai quand le jeton designe ce groupe, par identifiant ou par sigle,
    /// ancien ou courant.
    pub fn matches(&self, token: &str) -> bool {
        self.contains_uid(token)
            || self.abbrev.eq_ignore_ascii_case(token)
            || self
                .former_abbrevs
                .iter()
                .any(|abbrev| abbrev.eq_ignore_ascii_case(token))
    }
}

/// Renommages constates dans le referentiel de la 17e legislature.
///
/// UDR (PO847173) porte 108 votes sur l'ensemble, du 27/01/2025 au 10/07/2025;
/// UDDPLR (PO872880) en porte 114, a partir du 15/10/2025. Les deux periodes
/// sont disjointes, la couleur publiee est la meme (#3367A7), et 108 + 114
/// fait les 222 votes sur l'ensemble de la legislature (mesure du 06/08/2026).
pub const GROUP_LINEAGES: &[GroupLineage] = &[GroupLineage {
    canonical_uid: "PO872880",
    uids: &["PO872880", "PO847173"],
    abbrev: "UDDPLR",
    label: "Union des droites pour la République (anciennement UDR)",
    former_abbrevs: &["UDR"],
}];

/// Lignee portant cet identifiant, s'il en existe une.
pub fn lineage_of_uid(uid: &str) -> Option<&'static GroupLineage> {
    GROUP_LINEAGES
        .iter()
        .find(|lineage| lineage.contains_uid(uid))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_identifiers_of_a_renamed_group_lead_to_the_same_lineage() {
        let before = lineage_of_uid("PO847173").unwrap();
        let after = lineage_of_uid("PO872880").unwrap();

        assert_eq!(before, after);
        assert_eq!(before.canonical_uid, "PO872880");
    }

    #[test]
    fn a_group_outside_any_lineage_is_left_alone() {
        assert!(lineage_of_uid("PO800000").is_none());
    }

    #[test]
    fn the_former_abbrev_still_designates_the_group() {
        let lineage = lineage_of_uid("PO872880").unwrap();

        assert!(lineage.matches("UDR"));
        assert!(lineage.matches("udr"));
        assert!(lineage.matches("UDDPLR"));
        assert!(lineage.matches("PO847173"));
        assert!(!lineage.matches("RN"));
    }

    /// Le canonique doit appartenir a sa propre lignee, sinon la selection
    /// renverrait un identifiant qu'aucune ventilation ne porte.
    #[test]
    fn every_canonical_uid_belongs_to_its_lineage() {
        for lineage in GROUP_LINEAGES {
            assert!(
                lineage.contains_uid(lineage.canonical_uid),
                "{} absent de sa lignee",
                lineage.canonical_uid
            );
        }
    }
}
