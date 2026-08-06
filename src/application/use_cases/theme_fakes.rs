//! Doublures partagees par les tests des use cases de thematisation.
//!
//! Ports d'etat: fake in-memory, on verifie l'etat persiste. Le classifieur est
//! un port d'effet: sa doublure joue une reponse programmee.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use chrono::NaiveDate;

use crate::application::ports::theme_classifier::{ClassifierError, ThemeClassifier};
use crate::application::ports::theme_repository::{
    AssignedFamily, AttemptOutcome, MethodReport, RepositoryError, ScrutinSubject, TextLink,
    TextPage, TextScrutin, TextSummary, ThemeRepository,
};
use crate::domain::theme::{
    DebatedText, FamilyCode, ProposedFamily, SubjectRef, TextKey, ThemeAssignment, ThemeProposal,
};

#[derive(Default)]
pub struct InMemoryThemeRepository {
    pub subjects: Mutex<Vec<ScrutinSubject>>,
    pub texts: Mutex<HashMap<String, DebatedText>>,
    pub links: Mutex<HashMap<String, String>>,
    pub attempts: Mutex<Vec<(String, NaiveDate, AttemptOutcome)>>,
    pub proposals: Mutex<Vec<ThemeProposal>>,
    pub assignments: Mutex<Vec<ThemeAssignment>>,
    pub awaiting: Mutex<Vec<DebatedText>>,
}

impl InMemoryThemeRepository {
    pub fn current_families(&self, subject: &SubjectRef) -> Vec<FamilyCode> {
        let mut families: Vec<FamilyCode> = self
            .assignments
            .lock()
            .unwrap()
            .iter()
            .filter(|a| a.subject() == subject && a.is_current())
            .map(|a| a.family())
            .collect();
        families.sort();
        families
    }

    pub fn closed_count(&self, subject: &SubjectRef) -> usize {
        self.assignments
            .lock()
            .unwrap()
            .iter()
            .filter(|a| a.subject() == subject && !a.is_current())
            .count()
    }
}

#[async_trait]
impl ThemeRepository for InMemoryThemeRepository {
    async fn scrutin_subjects(&self) -> Result<Vec<ScrutinSubject>, RepositoryError> {
        Ok(self.subjects.lock().unwrap().clone())
    }

    async fn save_texts(&self, texts: &[DebatedText]) -> Result<usize, RepositoryError> {
        let mut stored = self.texts.lock().unwrap();
        for text in texts {
            stored.insert(text.key().as_str().to_string(), text.clone());
        }
        Ok(texts.len())
    }

    async fn link_scrutins(&self, links: &[TextLink]) -> Result<usize, RepositoryError> {
        let mut stored = self.links.lock().unwrap();
        for link in links {
            stored.insert(link.scrutin_uid.clone(), link.text_key.clone());
        }
        Ok(links.len())
    }

    async fn link_dossiers_through_scrutins(&self) -> Result<usize, RepositoryError> {
        Ok(0)
    }

    async fn texts_awaiting_proposal(
        &self,
        limit: i64,
    ) -> Result<Vec<DebatedText>, RepositoryError> {
        let awaiting = self.awaiting.lock().unwrap();
        Ok(awaiting.iter().take(limit as usize).cloned().collect())
    }

    async fn record_attempt(
        &self,
        key: &TextKey,
        on: NaiveDate,
        outcome: AttemptOutcome,
    ) -> Result<(), RepositoryError> {
        self.attempts
            .lock()
            .unwrap()
            .push((key.as_str().to_string(), on, outcome));
        Ok(())
    }

    async fn save_proposal(&self, proposal: &ThemeProposal) -> Result<(), RepositoryError> {
        self.proposals.lock().unwrap().push(proposal.clone());
        Ok(())
    }

    async fn latest_proposal(
        &self,
        subject: &SubjectRef,
    ) -> Result<Option<ThemeProposal>, RepositoryError> {
        Ok(self
            .proposals
            .lock()
            .unwrap()
            .iter()
            .rev()
            .find(|p| p.subject() == subject)
            .cloned())
    }

    async fn replace_assignments(
        &self,
        subject: &SubjectRef,
        closed_on: NaiveDate,
        opened: &[ThemeAssignment],
    ) -> Result<(), RepositoryError> {
        let mut stored = self.assignments.lock().unwrap();
        for assignment in stored.iter_mut() {
            if assignment.subject() == subject && assignment.is_current() {
                assignment
                    .close(closed_on)
                    .map_err(|e| RepositoryError::Database(e.to_string()))?;
            }
        }
        stored.extend(opened.iter().cloned());
        Ok(())
    }

    async fn assignment_history(
        &self,
        subject: &SubjectRef,
    ) -> Result<Vec<ThemeAssignment>, RepositoryError> {
        Ok(self
            .assignments
            .lock()
            .unwrap()
            .iter()
            .filter(|a| a.subject() == subject)
            .cloned()
            .collect())
    }

    async fn text_by_key(&self, key: &TextKey) -> Result<Option<TextSummary>, RepositoryError> {
        let texts = self.texts.lock().unwrap();
        Ok(texts.get(key.as_str()).map(|text| TextSummary {
            key: text.key().as_str().to_string(),
            label: text.label().to_string(),
            scrutin_count: 0,
            first_vote: None,
            last_vote: None,
            dossier_uid: None,
            dossier_label: None,
            families: vec![],
            last_attempt_outcome: None,
        }))
    }

    async fn scrutins_of_text(
        &self,
        _key: &TextKey,
        _limit: i64,
        _offset: i64,
    ) -> Result<Vec<TextScrutin>, RepositoryError> {
        Ok(vec![])
    }

    async fn texts_by_family(
        &self,
        _family: FamilyCode,
        _limit: i64,
        _offset: i64,
    ) -> Result<TextPage, RepositoryError> {
        Ok(TextPage {
            items: vec![],
            total: 0,
        })
    }

    async fn unassigned_texts(
        &self,
        _limit: i64,
        _offset: i64,
    ) -> Result<TextPage, RepositoryError> {
        Ok(TextPage {
            items: vec![],
            total: 0,
        })
    }

    async fn families_of_scrutins(
        &self,
        _scrutin_uids: &[String],
    ) -> Result<HashMap<String, Vec<AssignedFamily>>, RepositoryError> {
        Ok(HashMap::new())
    }

    async fn families_of_dossier(
        &self,
        _dossier_uid: &str,
    ) -> Result<Vec<AssignedFamily>, RepositoryError> {
        Ok(vec![])
    }

    async fn method_report(&self) -> Result<MethodReport, RepositoryError> {
        unimplemented!("read model, teste sur la base")
    }

    async fn text_count(&self) -> Result<i64, RepositoryError> {
        Ok(self.texts.lock().unwrap().len() as i64)
    }
}

/// Doublure du port d'effet: rend la reponse programmee pour chaque libelle.
pub struct StubClassifier {
    answers: Mutex<HashMap<String, Result<Vec<ProposedFamily>, String>>>,
    default: Mutex<Option<Result<Vec<ProposedFamily>, String>>>,
    pub calls: Mutex<Vec<String>>,
}

impl StubClassifier {
    pub fn new() -> Self {
        Self {
            answers: Mutex::new(HashMap::new()),
            default: Mutex::new(None),
            calls: Mutex::new(vec![]),
        }
    }

    pub fn answering(self, label: &str, families: Vec<(FamilyCode, &str)>) -> Self {
        self.answers.lock().unwrap().insert(
            label.to_string(),
            Ok(families
                .into_iter()
                .map(|(family, why)| ProposedFamily::new(family, why.to_string()).unwrap())
                .collect()),
        );
        self
    }

    pub fn failing(self, label: &str) -> Self {
        self.answers
            .lock()
            .unwrap()
            .insert(label.to_string(), Err("modèle injoignable".to_string()));
        self
    }

    pub fn defaulting_to_nothing(self) -> Self {
        *self.default.lock().unwrap() = Some(Ok(vec![]));
        self
    }
}

impl Default for StubClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ThemeClassifier for StubClassifier {
    async fn propose(&self, text_label: &str) -> Result<Vec<ProposedFamily>, ClassifierError> {
        self.calls.lock().unwrap().push(text_label.to_string());
        let programmed = self.answers.lock().unwrap().get(text_label).cloned();
        let answer = programmed
            .or_else(|| self.default.lock().unwrap().clone())
            .unwrap_or(Ok(vec![]));
        answer.map_err(ClassifierError::Call)
    }

    fn model(&self) -> &str {
        "modele-de-test"
    }

    fn prompt_version(&self) -> &str {
        "test-v1"
    }
}
