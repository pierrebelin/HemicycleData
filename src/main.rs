use std::sync::Arc;

use hemicycle_data::api;
use hemicycle_data::application::ports::actor_repository::ActorRepository;
use hemicycle_data::application::ports::actor_source::ActorSource;
use hemicycle_data::application::ports::assembly_source::AssemblySource;
use hemicycle_data::application::ports::dossier_repository::DossierRepository;
use hemicycle_data::application::ports::scrutin_repository::ScrutinRepository;
use hemicycle_data::application::ports::scrutin_source::ScrutinSource;
use hemicycle_data::application::ports::theme_classifier::ThemeClassifier;
use hemicycle_data::application::ports::theme_repository::ThemeRepository;
use hemicycle_data::infrastructure::config;
use hemicycle_data::infrastructure::llm::anthropic_classifier::AnthropicThemeClassifier;
use hemicycle_data::infrastructure::llm::unavailable_classifier::UnavailableClassifier;
use hemicycle_data::infrastructure::national_assembly::actor_client::AmoActorClient;
use hemicycle_data::infrastructure::national_assembly::client::NationalAssemblyClient;
use hemicycle_data::infrastructure::national_assembly::scrutin_client::ScrutinClient;
use hemicycle_data::infrastructure::persistence::pg_actor_repository::PgActorRepository;
use hemicycle_data::infrastructure::persistence::pg_dossier_repository::PgDossierRepository;
use hemicycle_data::infrastructure::persistence::pg_scrutin_repository::PgScrutinRepository;
use hemicycle_data::infrastructure::persistence::pg_theme_repository::PgThemeRepository;
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
    let actor_source: Arc<dyn ActorSource> = Arc::new(AmoActorClient::new());
    let actor_repository: Arc<dyn ActorRepository> = Arc::new(PgActorRepository::new(db.clone()));
    let scrutin_source: Arc<dyn ScrutinSource> = Arc::new(ScrutinClient::new());
    let scrutin_repository: Arc<dyn ScrutinRepository> =
        Arc::new(PgScrutinRepository::new(db.clone()));
    let theme_repository: Arc<dyn ThemeRepository> = Arc::new(PgThemeRepository::new(db.clone()));

    // BYOK: sans cle, la thematisation ne propose rien et le reste du site
    // tourne. Un texte non propose reste consultable, non rattache (RM-01).
    let theme_classifier: Arc<dyn ThemeClassifier> = match AnthropicThemeClassifier::from_env() {
        Some(classifier) => {
            tracing::info!("Theme classifier ready ({})", classifier.model());
            Arc::new(classifier)
        }
        None => {
            tracing::warn!("ANTHROPIC_API_KEY absent: no theme proposal will be produced");
            Arc::new(UnavailableClassifier)
        }
    };

    let admin_token = std::env::var("ADMIN_TOKEN").ok().filter(|t| !t.is_empty());
    if admin_token.is_none() {
        tracing::warn!("ADMIN_TOKEN absent: arbitration screen closed");
    }

    let state = AppState {
        db,
        assembly_source,
        dossier_repository,
        actor_source,
        actor_repository,
        scrutin_source,
        scrutin_repository,
        theme_repository,
        theme_classifier,
        admin_token,
    };

    let app = api::routes::create_router(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("Server running on http://localhost:{port}");
    axum::serve(listener, app).await.unwrap();
}
