use chrono::NaiveDate;

use crate::application::ports::theme_classifier::ThemeClassifier;
use crate::application::ports::theme_repository::{
    AttemptOutcome, RepositoryError, ThemeRepository,
};
use crate::domain::theme::{SubjectRef, ThemeProposal};

/// Resultat d'une passe de proposition. Chiffres comptes ici, pas rendus par
/// le modele (RM-10).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProposalRun {
    pub attempted: usize,
    pub proposed: usize,
    /// Le modele a repondu sans retenir de famille.
    pub without_family: usize,
    /// Le modele n'a pas repondu. Re-tente a la passe suivante.
    pub failed: usize,
}

/// CU-02 — Proposer les familles d'un texte.
///
/// La proposition est publiee telle quelle, portant sa mention d'origine
/// (RM-09). Un echec du modele ne fait echouer aucune autre proposition: le
/// texte reste non rattache et consultable (RM-01).
pub struct ProposeThemeFamilies<'a> {
    repository: &'a dyn ThemeRepository,
    classifier: &'a dyn ThemeClassifier,
}

impl<'a> ProposeThemeFamilies<'a> {
    pub fn new(repository: &'a dyn ThemeRepository, classifier: &'a dyn ThemeClassifier) -> Self {
        Self {
            repository,
            classifier,
        }
    }

    pub async fn execute(
        &self,
        batch: i64,
        today: NaiveDate,
    ) -> Result<ProposalRun, RepositoryError> {
        let texts = self.repository.texts_awaiting_proposal(batch).await?;
        let mut run = ProposalRun::default();

        for text in texts {
            run.attempted += 1;
            let subject = SubjectRef::Text(text.key().clone());

            let families = match self.classifier.propose(text.label()).await {
                Ok(families) => families,
                Err(error) => {
                    tracing::warn!(text = text.label(), %error, "theme proposal failed");
                    run.failed += 1;
                    self.repository
                        .record_attempt(text.key(), today, AttemptOutcome::Failed)
                        .await?;
                    continue;
                }
            };

            if families.is_empty() {
                run.without_family += 1;
                self.repository
                    .record_attempt(text.key(), today, AttemptOutcome::NoFamily)
                    .await?;
                continue;
            }

            let proposal = match ThemeProposal::new(
                subject.clone(),
                families,
                self.classifier.model().to_string(),
                self.classifier.prompt_version().to_string(),
                today,
            ) {
                Ok(proposal) => proposal,
                Err(error) => {
                    tracing::warn!(text = text.label(), %error, "theme proposal refused");
                    run.without_family += 1;
                    self.repository
                        .record_attempt(text.key(), today, AttemptOutcome::NoFamily)
                        .await?;
                    continue;
                }
            };

            let assignments = proposal
                .into_assignments()
                .map_err(|e| RepositoryError::Database(e.to_string()))?;

            self.repository.save_proposal(&proposal).await?;
            self.repository
                .replace_assignments(&subject, today, &assignments)
                .await?;
            self.repository
                .record_attempt(text.key(), today, AttemptOutcome::Proposed)
                .await?;
            run.proposed += 1;
        }

        Ok(run)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::use_cases::theme_fakes::{InMemoryThemeRepository, StubClassifier};
    use crate::domain::theme::{AssignmentOrigin, DebatedText, FamilyCode};

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, 3).unwrap()
    }

    fn text(label: &str) -> DebatedText {
        DebatedText::new(label.to_string()).unwrap()
    }

    #[tokio::test]
    async fn a_proposal_is_published_as_a_proposal() {
        let repository = InMemoryThemeRepository::default();
        let logement = text("proposition de loi de simplification du droit de l'urbanisme et du logement");
        *repository.awaiting.lock().unwrap() = vec![logement.clone()];
        let classifier = StubClassifier::new().answering(
            logement.label(),
            vec![(FamilyCode::Logement, "Le texte porte sur l'urbanisme et le logement.")],
        );

        let run = ProposeThemeFamilies::new(&repository, &classifier)
            .execute(10, today())
            .await
            .unwrap();

        assert_eq!(run.proposed, 1);
        let subject = SubjectRef::Text(logement.key().clone());
        assert_eq!(repository.current_families(&subject), vec![FamilyCode::Logement]);
        let assignments = repository.assignments.lock().unwrap();
        assert_eq!(assignments[0].origin(), AssignmentOrigin::Proposal);
        assert_eq!(
            assignments[0].motive(),
            Some("Le texte porte sur l'urbanisme et le logement.")
        );
    }

    #[tokio::test]
    async fn more_than_three_families_are_truncated() {
        let repository = InMemoryThemeRepository::default();
        let plf = text("projet de loi de finances pour 2026");
        *repository.awaiting.lock().unwrap() = vec![plf.clone()];
        let classifier = StubClassifier::new().answering(
            plf.label(),
            vec![
                (FamilyCode::PouvoirAchatFiscalite, "fiscalité"),
                (FamilyCode::InstitutionsProcedure, "budget"),
                (FamilyCode::SanteSocial, "crédits sociaux"),
                (FamilyCode::Logement, "crédits logement"),
                (FamilyCode::Numerique, "crédits numériques"),
            ],
        );

        ProposeThemeFamilies::new(&repository, &classifier)
            .execute(10, today())
            .await
            .unwrap();

        let subject = SubjectRef::Text(plf.key().clone());
        assert_eq!(repository.current_families(&subject).len(), 3);
    }

    #[tokio::test]
    async fn a_failing_model_leaves_the_text_unassigned_and_retriable() {
        let repository = InMemoryThemeRepository::default();
        let first = text("projet de loi de finances pour 2026");
        let second = text("proposition de loi relative au droit à l'aide à mourir");
        *repository.awaiting.lock().unwrap() = vec![first.clone(), second.clone()];
        let classifier = StubClassifier::new()
            .failing(first.label())
            .answering(second.label(), vec![(FamilyCode::SanteSocial, "fin de vie")]);

        let run = ProposeThemeFamilies::new(&repository, &classifier)
            .execute(10, today())
            .await
            .unwrap();

        assert_eq!(run.failed, 1);
        assert_eq!(run.proposed, 1);
        assert!(repository
            .current_families(&SubjectRef::Text(first.key().clone()))
            .is_empty());
        let attempts = repository.attempts.lock().unwrap();
        assert!(attempts.contains(&(
            first.key().as_str().to_string(),
            today(),
            AttemptOutcome::Failed
        )));
    }

    #[tokio::test]
    async fn a_model_retaining_nothing_is_not_an_error() {
        let repository = InMemoryThemeRepository::default();
        let odd = text("proposition de résolution tendant à modifier le règlement de l'Assemblée");
        *repository.awaiting.lock().unwrap() = vec![odd.clone()];
        let classifier = StubClassifier::new().defaulting_to_nothing();

        let run = ProposeThemeFamilies::new(&repository, &classifier)
            .execute(10, today())
            .await
            .unwrap();

        assert_eq!(run.without_family, 1);
        assert_eq!(run.failed, 0);
        let attempts = repository.attempts.lock().unwrap();
        assert_eq!(attempts[0].2, AttemptOutcome::NoFamily);
    }

    #[tokio::test]
    async fn the_model_only_ever_sees_the_text_label() {
        let repository = InMemoryThemeRepository::default();
        let text = text("projet de loi de finances pour 2026");
        *repository.awaiting.lock().unwrap() = vec![text.clone()];
        let classifier = StubClassifier::new()
            .answering(text.label(), vec![(FamilyCode::PouvoirAchatFiscalite, "fiscalité")]);

        ProposeThemeFamilies::new(&repository, &classifier)
            .execute(10, today())
            .await
            .unwrap();

        assert_eq!(*classifier.calls.lock().unwrap(), vec![text.label().to_string()]);
    }
}
