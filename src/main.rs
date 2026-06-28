mod api;
mod application;
mod domain;
mod infrastructure;

use std::sync::Arc;

use application::ports::assemblee_source::AssembleeSource;
use infrastructure::assemblee_nationale::client::AssembleeNationaleClient;
use infrastructure::config;

#[derive(Clone)]
pub struct AppState {
    pub db: Option<sqlx::PgPool>,
    pub assemblee_source: Arc<dyn AssembleeSource>,
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

    let assemblee_source = Arc::new(AssembleeNationaleClient::new());

    let state = AppState {
        db,
        assemblee_source,
    };

    let app = api::routes::create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    tracing::info!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
