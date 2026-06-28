mod api;
mod application;
mod domain;
mod infrastructure;

use std::sync::Arc;

use application::ports::assembly_source::AssemblySource;
use application::ports::dossier_repository::DossierRepository;
use infrastructure::config;
use infrastructure::national_assembly::client::NationalAssemblyClient;
use infrastructure::persistence::pg_dossier_repository::PgDossierRepository;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub assembly_source: Arc<dyn AssemblySource>,
    pub dossier_repository: Arc<dyn DossierRepository>,
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

    let state = AppState {
        db,
        assembly_source,
        dossier_repository,
    };

    let app = api::routes::create_router(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("Server running on http://localhost:{port}");
    axum::serve(listener, app).await.unwrap();
}
