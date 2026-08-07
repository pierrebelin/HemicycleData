use crate::application::ports::dossier_repository::{
    DossierPage, DossierRepository, RepositoryError,
};

pub const DEFAULT_PER_PAGE: i64 = 20;
pub const MAX_PER_PAGE: i64 = 100;

/// Page demandée par le visiteur, ramenée à des bornes tenables.
///
/// Une page hors bornes ne renvoie pas d'erreur : elle renvoie une tranche
/// vide avec le total réel. Le visiteur qui tape `?page=9999` doit voir combien
/// de dossiers existent, pas un message d'erreur.
#[derive(Debug, Clone, Copy)]
pub struct PageRequest {
    page: i64,
    per_page: i64,
}

impl PageRequest {
    pub fn new(page: i64, per_page: i64) -> Self {
        Self {
            page: page.max(1),
            per_page: per_page.clamp(1, MAX_PER_PAGE),
        }
    }

    pub fn page(&self) -> i64 {
        self.page
    }

    pub fn per_page(&self) -> i64 {
        self.per_page
    }

    fn offset(&self) -> i64 {
        (self.page - 1) * self.per_page
    }
}

impl Default for PageRequest {
    fn default() -> Self {
        Self::new(1, DEFAULT_PER_PAGE)
    }
}

/// Parcourir tous les dossiers, du plus récent au plus ancien.
///
/// Aucun filtre, aucun seuil de score : la liste est le point d'entrée du site
/// et doit rester exhaustive (README.md §2). Le tri porte sur la dernière
/// activité — c'est la date qui dit où en est un texte.
pub struct BrowseDossiers<'a> {
    repository: &'a dyn DossierRepository,
}

impl<'a> BrowseDossiers<'a> {
    pub fn new(repository: &'a dyn DossierRepository) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, request: PageRequest) -> Result<DossierPage, RepositoryError> {
        self.repository
            .find_page(request.per_page(), request.offset())
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use std::sync::Mutex;

    use crate::domain::dossier::{CurationStatus, DossierOutcome, DossierUid, LegislativeDossier};
    use crate::domain::scoring::compute_score;

    struct InMemoryDossierRepository {
        dossiers: Mutex<Vec<LegislativeDossier>>,
    }

    #[async_trait]
    impl DossierRepository for InMemoryDossierRepository {
        async fn load_states(
            &self,
        ) -> Result<
            std::collections::HashMap<
                String,
                crate::application::ports::dossier_repository::StoredDossierState,
            >,
            RepositoryError,
        > {
            unreachable!()
        }
        async fn save_all(
            &self,
            _dossiers: &[LegislativeDossier],
        ) -> Result<usize, RepositoryError> {
            unreachable!()
        }

        async fn find_recent(
            &self,
            _since: NaiveDate,
        ) -> Result<Vec<LegislativeDossier>, RepositoryError> {
            unreachable!()
        }

        async fn find_page(&self, limit: i64, offset: i64) -> Result<DossierPage, RepositoryError> {
            let store = self.dossiers.lock().unwrap();
            let mut sorted = store.clone();
            sorted.sort_by(|a, b| b.last_activity_date.cmp(&a.last_activity_date));
            Ok(DossierPage {
                total: sorted.len() as i64,
                items: sorted
                    .into_iter()
                    .skip(offset as usize)
                    .take(limit as usize)
                    .collect(),
            })
        }

        async fn find_by_uid(
            &self,
            _uid: &DossierUid,
        ) -> Result<Option<LegislativeDossier>, RepositoryError> {
            unreachable!()
        }

        async fn find_suggestions(
            &self,
            _count: usize,
        ) -> Result<Vec<LegislativeDossier>, RepositoryError> {
            unreachable!()
        }

        async fn update_curation_status(
            &self,
            _uid: &DossierUid,
            _status: CurationStatus,
        ) -> Result<bool, RepositoryError> {
            unreachable!()
        }
    }

    fn dossier(uid: &str, day: u32) -> LegislativeDossier {
        LegislativeDossier {
            uid: DossierUid::new(uid.into()).unwrap(),
            title: uid.into(),
            procedure: "Projet de loi".into(),
            legislature: 17,
            url: None,
            summary: None,
            deposit_date: None,
            last_activity_date: NaiveDate::from_ymd_opt(2026, 6, day).unwrap(),
            last_activity_label: "Dépôt".into(),
            acts: vec![],
            documents: vec![],
            score: compute_score(uid, "Dépôt", 0),
            current_stage: None,
            initiators: vec![],
            committee: None,
            curation_status: CurationStatus::New,
            outcome: DossierOutcome::NoRecordedConclusion,
        }
    }

    fn repo(count: u32) -> InMemoryDossierRepository {
        InMemoryDossierRepository {
            dossiers: Mutex::new((1..=count).map(|d| dossier(&format!("D{d}"), d)).collect()),
        }
    }

    #[tokio::test]
    async fn first_page_holds_the_most_recent_dossiers() {
        let repo = repo(5);
        let page = BrowseDossiers::new(&repo)
            .execute(PageRequest::new(1, 2))
            .await
            .unwrap();

        assert_eq!(page.total, 5);
        let uids: Vec<_> = page.items.iter().map(|d| d.uid.as_str()).collect();
        assert_eq!(uids, vec!["D5", "D4"]);
    }

    #[tokio::test]
    async fn following_page_continues_without_repeating() {
        let repo = repo(5);
        let page = BrowseDossiers::new(&repo)
            .execute(PageRequest::new(2, 2))
            .await
            .unwrap();

        let uids: Vec<_> = page.items.iter().map(|d| d.uid.as_str()).collect();
        assert_eq!(uids, vec!["D3", "D2"]);
    }

    /// Une page au-delà du dernier dossier reste une réponse valide.
    #[tokio::test]
    async fn page_beyond_the_end_is_empty_and_still_reports_the_total() {
        let repo = repo(5);
        let page = BrowseDossiers::new(&repo)
            .execute(PageRequest::new(99, 20))
            .await
            .unwrap();

        assert!(page.items.is_empty());
        assert_eq!(page.total, 5);
    }

    #[test]
    fn out_of_range_parameters_are_brought_back_within_bounds() {
        assert_eq!(PageRequest::new(0, 20).page(), 1);
        assert_eq!(PageRequest::new(-3, 20).page(), 1);
        assert_eq!(PageRequest::new(1, 0).per_page(), 1);
        assert_eq!(PageRequest::new(1, 10_000).per_page(), MAX_PER_PAGE);
    }
}
