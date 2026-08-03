use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;

use crate::AppState;

use super::handlers::{dossier_handlers, scrutin_handlers};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/dossiers", get(dossier_handlers::get_recent_dossiers))
        .route(
            "/api/dossiers/{uid}",
            get(dossier_handlers::get_dossier_detail),
        )
        .route(
            "/api/dossiers/{uid}/save",
            post(dossier_handlers::save_dossier),
        )
        .route(
            "/api/suggestions",
            get(dossier_handlers::get_suggestions),
        )
        .route(
            "/api/dossiers/{uid}/curate",
            post(dossier_handlers::curate_dossier),
        )
        .route(
            "/api/dossiers/{uid}/scrutins",
            get(scrutin_handlers::get_dossier_scrutins),
        )
        .route("/api/scrutins", get(scrutin_handlers::list_scrutins))
        .route(
            "/api/scrutins/refresh",
            post(scrutin_handlers::refresh_scrutins),
        )
        .route(
            "/api/scrutins/{uid}",
            get(scrutin_handlers::get_scrutin_detail),
        )
        .route("/api/refresh", post(dossier_handlers::refresh_dossiers))
        .route(
            "/api/registry/refresh",
            post(dossier_handlers::refresh_actor_registry),
        )
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> Json<Value> {
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .is_ok();

    Json(json!({
        "status": "ok",
        "database": if db_ok { "connected" } else { "disconnected" }
    }))
}
