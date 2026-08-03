use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::api::scrutin_dto::{
    DossierScrutinsResponse, ScrutinDetailDto, ScrutinListQuery, ScrutinListResponse,
    ScrutinSummaryDto, ScrutinsRefreshResponse, SHOW_OF_HANDS_NOTE,
};
use crate::application::ports::scrutin_repository::ScrutinFilter;
use crate::application::use_cases::get_scrutin_detail::GetScrutinDetail;
use crate::application::use_cases::list_scrutins::ListScrutins;
use crate::application::use_cases::refresh_scrutins::RefreshScrutins;
use crate::domain::scrutin::ScrutinUid;
use crate::AppState;

/// CU-02 — Liste des scrutins.
pub async fn list_scrutins(
    State(state): State<AppState>,
    Query(params): Query<ScrutinListQuery>,
) -> Result<Json<ScrutinListResponse>, (StatusCode, String)> {
    let filter: ScrutinFilter = params.into();
    let offset = filter.offset.max(0);

    let page = ListScrutins::new(state.scrutin_repository.as_ref())
        .execute(filter)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ScrutinListResponse::from((page, offset))))
}

/// CU-03 — Detail d'un scrutin.
pub async fn get_scrutin_detail(
    State(state): State<AppState>,
    Path(uid): Path<String>,
) -> Result<Json<ScrutinDetailDto>, (StatusCode, String)> {
    let uid = ScrutinUid::new(uid).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let detail = GetScrutinDetail::new(
        state.scrutin_repository.as_ref(),
        state.actor_repository.as_ref(),
    )
    .execute(&uid)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Scrutin not found".to_string()))?;

    Ok(Json(ScrutinDetailDto::from(detail)))
}

/// CU-04 — Scrutins d'un dossier. La section existe meme vide: la source peut
/// ne rattacher aucun scrutin a un dossier, et le taire laisserait croire a une
/// absence de vote.
pub async fn get_dossier_scrutins(
    State(state): State<AppState>,
    Path(uid): Path<String>,
) -> Result<Json<DossierScrutinsResponse>, (StatusCode, String)> {
    let scrutins = ListScrutins::new(state.scrutin_repository.as_ref())
        .for_dossier(&uid)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let scrutins: Vec<ScrutinSummaryDto> =
        scrutins.into_iter().map(ScrutinSummaryDto::from).collect();

    Ok(Json(DossierScrutinsResponse {
        count: scrutins.len(),
        scrutins,
        coverage_note: SHOW_OF_HANDS_NOTE,
    }))
}

/// CU-01 — Ingerer les scrutins.
///
/// RM-11: le referentiel doit avoir ete rafraichi avant. Le use case detecte un
/// referentiel vide et remonte l'anomalie plutot que de reconstruire a vide.
/// `POST /api/refresh` enchaine les trois dans l'ordre.
pub async fn refresh_scrutins(
    State(state): State<AppState>,
) -> Result<Json<ScrutinsRefreshResponse>, (StatusCode, String)> {
    let summary = RefreshScrutins::new(
        state.scrutin_source.as_ref(),
        state.scrutin_repository.as_ref(),
        state.actor_repository.as_ref(),
    )
    .execute()
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ScrutinsRefreshResponse::from(summary)))
}
