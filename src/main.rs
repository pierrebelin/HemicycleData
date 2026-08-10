use std::sync::Arc;

use hemicycle_data::api;
use hemicycle_data::api::security::AdminGuard;
use hemicycle_data::application::ports::actor_repository::ActorRepository;
use hemicycle_data::application::ports::actor_source::ActorSource;
use hemicycle_data::application::ports::amendment_repository::AmendmentRepository;
use hemicycle_data::application::ports::amendment_source::AmendmentSource;
use hemicycle_data::application::ports::assembly_source::AssemblySource;
use hemicycle_data::application::ports::dossier_group_actions_repository::{
    DossierGroupActionsRepository, DossierSummaryRepository,
};
use hemicycle_data::application::ports::dossier_repository::DossierRepository;
use hemicycle_data::application::ports::dossier_summary_generator::DossierSummaryGenerator;
use hemicycle_data::application::ports::final_vote_repository::FinalVoteRepository;
use hemicycle_data::application::ports::group_repository::GroupRepository;
use hemicycle_data::application::ports::scrutin_repository::ScrutinRepository;
use hemicycle_data::application::ports::scrutin_source::ScrutinSource;
use hemicycle_data::application::ports::theme_classifier::ThemeClassifier;
use hemicycle_data::application::ports::theme_repository::ThemeRepository;
use hemicycle_data::infrastructure::config;
use hemicycle_data::infrastructure::llm::anthropic_classifier::AnthropicThemeClassifier;
use hemicycle_data::infrastructure::llm::anthropic_dossier_summary::AnthropicDossierSummaryGenerator;
use hemicycle_data::infrastructure::llm::openai_classifier::OpenAiThemeClassifier;
use hemicycle_data::infrastructure::llm::openai_dossier_summary::OpenAiDossierSummaryGenerator;
use hemicycle_data::infrastructure::llm::provider::LlmProvider;
use hemicycle_data::infrastructure::llm::unavailable_classifier::UnavailableClassifier;
use hemicycle_data::infrastructure::llm::unavailable_dossier_summary::UnavailableDossierSummaryGenerator;
use hemicycle_data::infrastructure::national_assembly::actor_client::AmoActorClient;
use hemicycle_data::infrastructure::national_assembly::amendment_client::AmendmentClient;
use hemicycle_data::infrastructure::national_assembly::client::NationalAssemblyClient;
use hemicycle_data::infrastructure::national_assembly::scrutin_client::ScrutinClient;
use hemicycle_data::infrastructure::persistence::pg_actor_repository::PgActorRepository;
use hemicycle_data::infrastructure::persistence::pg_amendment_repository::PgAmendmentRepository;
use hemicycle_data::infrastructure::persistence::pg_dossier_group_actions_repository::PgDossierGroupActionsRepository;
use hemicycle_data::infrastructure::persistence::pg_dossier_repository::PgDossierRepository;
use hemicycle_data::infrastructure::persistence::pg_dossier_summary_repository::PgDossierSummaryRepository;
use hemicycle_data::infrastructure::persistence::pg_final_vote_repository::PgFinalVoteRepository;
use hemicycle_data::infrastructure::persistence::pg_group_repository::PgGroupRepository;
use hemicycle_data::infrastructure::persistence::pg_scrutin_repository::PgScrutinRepository;
use hemicycle_data::infrastructure::persistence::pg_theme_repository::PgThemeRepository;
use hemicycle_data::infrastructure::security::AdminTokenSecret;
use hemicycle_data::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let db = config::try_connect_database()
        .await
        .expect("Database connection required");
    tracing::info!("Database connected");

    let assembly_source: Arc<dyn AssemblySource> = Arc::new(NationalAssemblyClient::new());
    let dossier_repository: Arc<dyn DossierRepository> =
        Arc::new(PgDossierRepository::new(db.clone()));
    let dossier_group_actions_repository: Arc<dyn DossierGroupActionsRepository> =
        Arc::new(PgDossierGroupActionsRepository::new(db.clone()));
    let dossier_summary_repository: Arc<dyn DossierSummaryRepository> =
        Arc::new(PgDossierSummaryRepository::new(db.clone()));
    let actor_source: Arc<dyn ActorSource> = Arc::new(AmoActorClient::new());
    let actor_repository: Arc<dyn ActorRepository> = Arc::new(PgActorRepository::new(db.clone()));
    let scrutin_source: Arc<dyn ScrutinSource> = Arc::new(ScrutinClient::new());
    let scrutin_repository: Arc<dyn ScrutinRepository> =
        Arc::new(PgScrutinRepository::new(db.clone()));
    let amendment_source: Arc<dyn AmendmentSource> = Arc::new(AmendmentClient::new());
    let amendment_repository: Arc<dyn AmendmentRepository> =
        Arc::new(PgAmendmentRepository::new(db.clone()));
    let final_vote_repository: Arc<dyn FinalVoteRepository> =
        Arc::new(PgFinalVoteRepository::new(db.clone()));
    let group_repository: Arc<dyn GroupRepository> = Arc::new(PgGroupRepository::new(db.clone()));
    let theme_repository: Arc<dyn ThemeRepository> = Arc::new(PgThemeRepository::new(db.clone()));

    let llm_provider = match LlmProvider::from_env() {
        Ok(provider) => {
            tracing::info!("LLM provider selected: {}", provider.label());
            Some(provider)
        }
        Err(error) => {
            tracing::error!(%error, "LLM disabled");
            None
        }
    };

    // BYOK: sans la cle du fournisseur selectionne, le site tourne mais les
    // traitements LLM restent explicitement indisponibles. Les donnees brutes
    // et les rattachements deterministes restent accessibles (RM-01).
    let theme_classifier: Arc<dyn ThemeClassifier> = match llm_provider {
        Some(LlmProvider::Anthropic) => match AnthropicThemeClassifier::from_env() {
            Some(classifier) => {
                tracing::info!("Theme classifier ready ({})", classifier.model());
                Arc::new(classifier)
            }
            None => {
                tracing::warn!("ANTHROPIC_API_KEY absent: no theme proposal will be produced");
                Arc::new(UnavailableClassifier)
            }
        },
        Some(LlmProvider::OpenAi) => match OpenAiThemeClassifier::from_env() {
            Some(classifier) => {
                tracing::info!("Theme classifier ready ({})", classifier.model());
                Arc::new(classifier)
            }
            None => {
                tracing::warn!("OPENAI_API_KEY absent: no theme proposal will be produced");
                Arc::new(UnavailableClassifier)
            }
        },
        None => Arc::new(UnavailableClassifier),
    };

    let dossier_summary_generator: Arc<dyn DossierSummaryGenerator> = match llm_provider {
        Some(LlmProvider::Anthropic) => match AnthropicDossierSummaryGenerator::from_env() {
            Some(generator) => {
                tracing::info!("Dossier summary generator ready ({})", generator.model());
                Arc::new(generator)
            }
            None => {
                tracing::warn!("ANTHROPIC_API_KEY absent: dossier summaries will stay pending");
                Arc::new(UnavailableDossierSummaryGenerator)
            }
        },
        Some(LlmProvider::OpenAi) => match OpenAiDossierSummaryGenerator::from_env() {
            Some(generator) => {
                tracing::info!("Dossier summary generator ready ({})", generator.model());
                Arc::new(generator)
            }
            None => {
                tracing::warn!("OPENAI_API_KEY absent: dossier summaries will stay pending");
                Arc::new(UnavailableDossierSummaryGenerator)
            }
        },
        None => Arc::new(UnavailableDossierSummaryGenerator),
    };

    // Le jeton d'ecriture n'est pas pose dans l'environnement: il est derive
    // du secret et de la date du jour, cote serveur comme cote client
    // (src/infrastructure/security.rs).
    let admin_guard = match std::env::var("ADMIN_TOKEN_SECRET") {
        Ok(raw) => match AdminTokenSecret::new(raw) {
            Ok(secret) => {
                tracing::info!("Admin write routes protected by daily token");
                AdminGuard::new(Some(secret))
            }
            // Un secret refuse ferme l'ecriture, il ne la laisse pas ouverte.
            Err(error) => {
                tracing::error!("ADMIN_TOKEN_SECRET rejected ({error}): write routes closed");
                AdminGuard::closed()
            }
        },
        Err(_) => {
            tracing::warn!("ADMIN_TOKEN_SECRET absent: write routes closed");
            AdminGuard::closed()
        }
    };

    let state = AppState {
        db,
        assembly_source,
        dossier_repository,
        dossier_group_actions_repository,
        dossier_summary_repository,
        dossier_summary_generator,
        actor_source,
        actor_repository,
        scrutin_source,
        amendment_source,
        scrutin_repository,
        amendment_repository,
        final_vote_repository,
        group_repository,
        theme_repository,
        theme_classifier,
    };

    let app = api::routes::create_router(state, admin_guard);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    // Boucle locale par defaut: en production Nginx est le seul client du
    // backend, et le pare-feu n'a plus a etre la seule chose qui empeche
    // d'atteindre l'API en direct. `BIND_ADDR=0.0.0.0` pour un conteneur.
    let host = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1".to_string());
    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("Server running on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
