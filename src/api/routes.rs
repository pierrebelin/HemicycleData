use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;

use crate::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> Json<Value> {
    let db_ok = sqlx::query("SELECT 1").execute(&state.db).await.is_ok();

    Json(json!({
        "status": "ok",
        "database": if db_ok { "connected" } else { "disconnected" }
    }))
}
