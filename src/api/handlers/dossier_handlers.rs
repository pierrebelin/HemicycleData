use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::api::dto::{
    CurateBody, DossierDetailDto, DossierDto, RecentActivityQuery, RecentDossiersResponse,
    RefreshResponse, SuggestionsQuery, SuggestionsResponse,
};
use crate::application::use_cases::curate_dossier::CurateDossier;
use crate::application::use_cases::fetch_recent_dossiers::FetchRecentDossiers;
use crate::application::use_cases::get_dossier_detail::GetDossierDetail;
use crate::application::use_cases::refresh_dossiers::RefreshDossiers;
use crate::application::use_cases::save_dossier::SaveDossier;
use crate::application::use_cases::suggest_dossiers::SuggestDossiers;
use crate::domain::dossier::DossierUid;
use crate::AppState;

pub async fn get_recent_dossiers(
    State(state): State<AppState>,
    Query(params): Query<RecentActivityQuery>,
) -> Result<Json<RecentDossiersResponse>, (StatusCode, String)> {
    let uc = FetchRecentDossiers::new(state.dossier_repository.as_ref());

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
    let uid = DossierUid::new(uid)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let uc = GetDossierDetail::new(
        state.dossier_repository.as_ref(),
        state.assembly_source.as_ref(),
        state.deputy_source.as_ref(),
    );

    let result = uc
        .execute(&uid)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Dossier not found".to_string()))?;

    Ok(Json(DossierDetailDto::from_result(result)))
}

pub async fn save_dossier(
    State(state): State<AppState>,
    Path(uid): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let uid = DossierUid::new(uid)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let uc = SaveDossier::new(
        state.assembly_source.as_ref(),
        state.dossier_repository.as_ref(),
        state.deputy_source.as_ref(),
    );

    uc.execute(&uid)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_suggestions(
    State(state): State<AppState>,
    Query(params): Query<SuggestionsQuery>,
) -> Result<Json<SuggestionsResponse>, (StatusCode, String)> {
    let uc = SuggestDossiers::new(state.dossier_repository.as_ref());

    let dossiers = uc
        .execute(params.count)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let dtos: Vec<DossierDto> = dossiers.into_iter().map(DossierDto::from).collect();

    Ok(Json(SuggestionsResponse {
        count: dtos.len(),
        suggestions: dtos,
    }))
}

pub async fn curate_dossier(
    State(state): State<AppState>,
    Path(uid): Path<String>,
    Json(body): Json<CurateBody>,
) -> Result<StatusCode, (StatusCode, String)> {
    let uid = DossierUid::new(uid)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let uc = CurateDossier::new(state.dossier_repository.as_ref());

    uc.execute(&uid, body.status)
        .await
        .map_err(|e| match &e {
            crate::application::use_cases::curate_dossier::CurateError::NotFound(_) => {
                (StatusCode::NOT_FOUND, e.to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn refresh_dossiers(
    State(state): State<AppState>,
) -> Result<Json<RefreshResponse>, (StatusCode, String)> {
    let uc = RefreshDossiers::new(
        state.assembly_source.as_ref(),
        state.dossier_repository.as_ref(),
        state.deputy_source.as_ref(),
    );

    let count = uc
        .execute()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(RefreshResponse { count }))
}
