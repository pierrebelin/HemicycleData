use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::api::dto::{
    DossierDetailDto, DossierDto, RecentActivityQuery, RecentDossiersResponse,
};
use crate::application::use_cases::fetch_recent_dossiers::FetchRecentDossiers;
use crate::application::use_cases::get_dossier_detail::GetDossierDetail;
use crate::AppState;

pub async fn get_recent_dossiers(
    State(state): State<AppState>,
    Query(params): Query<RecentActivityQuery>,
) -> Result<Json<RecentDossiersResponse>, (StatusCode, String)> {
    let uc = FetchRecentDossiers::new(state.assembly_source.as_ref());

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

pub async fn get_dossier_detail(
    State(state): State<AppState>,
    Path(uid): Path<String>,
) -> Result<Json<DossierDetailDto>, (StatusCode, String)> {
    let uc = GetDossierDetail::new(state.assembly_source.as_ref());

    let dossier = uc
        .execute(&uid)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Dossier not found".to_string()))?;

    Ok(Json(DossierDetailDto::from(dossier)))
}
