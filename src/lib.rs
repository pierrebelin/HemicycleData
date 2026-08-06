pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;

use std::sync::Arc;

use application::ports::actor_repository::ActorRepository;
use application::ports::actor_source::ActorSource;
use application::ports::assembly_source::AssemblySource;
use application::ports::dossier_repository::DossierRepository;
use application::ports::final_vote_repository::FinalVoteRepository;
use application::ports::scrutin_repository::ScrutinRepository;
use application::ports::scrutin_source::ScrutinSource;
use application::ports::theme_classifier::ThemeClassifier;
use application::ports::theme_repository::ThemeRepository;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub assembly_source: Arc<dyn AssemblySource>,
    pub dossier_repository: Arc<dyn DossierRepository>,
    pub actor_source: Arc<dyn ActorSource>,
    pub actor_repository: Arc<dyn ActorRepository>,
    pub scrutin_source: Arc<dyn ScrutinSource>,
    pub scrutin_repository: Arc<dyn ScrutinRepository>,
    pub final_vote_repository: Arc<dyn FinalVoteRepository>,
    pub theme_repository: Arc<dyn ThemeRepository>,
    pub theme_classifier: Arc<dyn ThemeClassifier>,
    /// Jeton de l'ecran d'arbitrage (CU-03). Absent = arbitrage ferme.
    pub admin_token: Option<String>,
}
