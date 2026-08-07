use async_trait::async_trait;
use chrono::NaiveDate;

use crate::domain::group_profile::{ParticipationCounts, QualityCount};

pub use super::RepositoryError;

/// Un groupe du referentiel, tel que la base le porte, avant tout rapprochement
/// de lignee.
///
/// Aucun groupe n'est ecarte, meme sans une seule ligne de vote: le referentiel
/// est publie en entier, et la couverture nulle est affichee comme telle
/// (PROJECT.md §2).
#[derive(Debug, Clone)]
pub struct GroupRecord {
    pub uid: String,
    pub legislature: i16,
    pub label: String,
    pub abbrev: String,
    pub color: Option<String>,
    /// Dates de constitution et de dissolution publiees par le referentiel.
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    /// Date a laquelle l'effectif est compte: aujourd'hui pour un groupe
    /// actif, son dernier jour pour un groupe dissous. Compter un groupe
    /// dissous a la date du jour donnerait zero membre, ce qui se lit comme une
    /// donnee manquante.
    pub reference_date: NaiveDate,
    /// Deputes rattaches au groupe a `reference_date`, toutes qualites
    /// confondues (SPEC-acteurs RM-02).
    pub member_count: i64,
    /// Scrutins ou la source publie une ligne pour ce groupe.
    pub scrutin_count: i64,
    pub first_scrutin_date: Option<NaiveDate>,
    pub last_scrutin_date: Option<NaiveDate>,
}

/// Chiffres de participation d'un groupe, cumules sur ses lignes de vote.
///
/// `Default` decrit un groupe sur lequel la source ne publie rien: c'est l'etat
/// d'un groupe cree sans avoir encore vote, pas un remplissage par zero.
#[derive(Debug, Clone, Default)]
pub struct GroupStatisticsRecord {
    /// Deputes distincts ayant appartenu au groupe sur toute son existence.
    pub total_member_count: i64,
    /// Effectif a la date de reference, par qualite d'appartenance publiee.
    pub qualities: Vec<QualityCount>,
    /// Bornes de l'effectif publie sur les scrutins (H4).
    pub min_published_member_count: Option<u16>,
    pub max_published_member_count: Option<u16>,
    /// Lignes de vote du groupe, une par scrutin.
    pub line_count: i64,
    /// Lignes reconstituees depuis les positions nominales, a signaler partout
    /// ou elles s'affichent (SPEC-scrutins RM-03).
    pub reconstructed_count: i64,
    /// Lignes ou les cinq comptes sont a zero: la source publie la ligne, mais
    /// aucun membre du groupe n'y figure. H5 en denombre 8 834 sur la
    /// legislature; un cumul muet les avalerait.
    pub silent_line_count: i64,
    pub counts: ParticipationCounts,
}

#[async_trait]
pub trait GroupRepository: Send + Sync {
    /// Tous les groupes du referentiel, effectif compte a `today` pour les
    /// groupes actifs.
    async fn list_groups(&self, today: NaiveDate) -> Result<Vec<GroupRecord>, RepositoryError>;

    /// Chiffres cumules sur les identifiants donnes.
    ///
    /// La liste porte tous les identifiants d'une meme lignee: un groupe
    /// renomme garde ses chiffres d'avant le changement de nom, sinon la fiche
    /// afficherait la moitie de son activite (`domain::group_lineage`).
    async fn statistics(
        &self,
        group_uids: &[String],
        reference_date: NaiveDate,
    ) -> Result<GroupStatisticsRecord, RepositoryError>;
}
