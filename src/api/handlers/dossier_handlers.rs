use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::api::dto::{DossierDto, RecentActivityQuery, RecentDossiersResponse};
use crate::application::use_cases::fetch_recent_dossiers::FetchRecentDossiers;
use crate::AppState;

pub async fn get_recent_dossiers(
    State(state): State<AppState>,
    Query(params): Query<RecentActivityQuery>,
) -> Result<Json<RecentDossiersResponse>, (StatusCode, String)> {
    let uc = FetchRecentDossiers::new(state.assemblee_source.as_ref());

    let dossiers = uc
        .execute(params.days)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let dtos: Vec<DossierDto> = dossiers.into_iter().map(DossierDto::from).collect();

    Ok(Json(RecentDossiersResponse {
        count: dtos.len(),
        dossiers: dtos,
    }))
}
