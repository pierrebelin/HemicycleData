mod api;
mod application;
mod domain;
mod infrastructure;

use infrastructure::config;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let pool = config::connect_database().await;
    let state = AppState { db: pool };

    let app = api::routes::create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    tracing::info!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
