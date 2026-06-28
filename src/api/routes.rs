use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;

use crate::AppState;

use super::handlers::dossier_handlers;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/dossiers", get(dossier_handlers::get_recent_dossiers))
        .route(
            "/api/dossiers/{uid}",
            get(dossier_handlers::get_dossier_detail),
        )
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> Json<Value> {
    let db_ok = match &state.db {
        Some(pool) => sqlx::query("SELECT 1").execute(pool).await.is_ok(),
        None => false,
    };

    Json(json!({
        "status": "ok",
        "database": if db_ok { "connected" } else { "disconnected" }
    }))
}
