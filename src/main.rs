mod api;
mod application;
mod domain;
mod infrastructure;

use std::sync::Arc;

use application::ports::assembly_source::AssemblySource;
use infrastructure::national_assembly::client::NationalAssemblyClient;
use infrastructure::config;

#[derive(Clone)]
pub struct AppState {
    pub db: Option<sqlx::PgPool>,
    pub assembly_source: Arc<dyn AssemblySource>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let db = match config::try_connect_database().await {
        Ok(pool) => {
            tracing::info!("Database connected");
            Some(pool)
        }
        Err(e) => {
            tracing::warn!("No database connection: {e}");
            None
        }
    };

    let assembly_source = Arc::new(NationalAssemblyClient::new());

    let state = AppState {
        db,
        assembly_source,
    };

    let app = api::routes::create_router(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("Server running on http://localhost:{port}");
    axum::serve(listener, app).await.unwrap();
}
