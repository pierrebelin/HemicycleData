mod api;
mod application;
mod domain;
mod infrastructure;

use std::sync::Arc;

use application::ports::actor_repository::ActorRepository;
use application::ports::actor_source::ActorSource;
use application::ports::assembly_source::AssemblySource;
use application::ports::dossier_repository::DossierRepository;
use application::ports::scrutin_repository::ScrutinRepository;
use application::ports::scrutin_source::ScrutinSource;
use infrastructure::config;
use infrastructure::national_assembly::actor_client::AmoActorClient;
use infrastructure::national_assembly::client::NationalAssemblyClient;
use infrastructure::national_assembly::scrutin_client::ScrutinClient;
use infrastructure::persistence::pg_actor_repository::PgActorRepository;
use infrastructure::persistence::pg_dossier_repository::PgDossierRepository;
use infrastructure::persistence::pg_scrutin_repository::PgScrutinRepository;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub assembly_source: Arc<dyn AssemblySource>,
    pub dossier_repository: Arc<dyn DossierRepository>,
    pub actor_source: Arc<dyn ActorSource>,
    pub actor_repository: Arc<dyn ActorRepository>,
    pub scrutin_source: Arc<dyn ScrutinSource>,
    pub scrutin_repository: Arc<dyn ScrutinRepository>,
}

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

    let state = AppState {
        db,
        assembly_source,
        dossier_repository,
        actor_source,
        actor_repository,
        scrutin_source,
        scrutin_repository,
    };

    let app = api::routes::create_router(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("Server running on http://localhost:{port}");
    axum::serve(listener, app).await.unwrap();
}
