use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::Utc;

use crate::api::dto::{
    CurateBody, DossierDetailDto, DossierDto, DossierPageQuery, DossierPageResponse,
    RecentActivityQuery, RecentDossiersResponse, RefreshQuery, RefreshResponse, RegistryResponse,
    SuggestionsQuery, SuggestionsResponse,
};
use crate::application::use_cases::browse_dossiers::{BrowseDossiers, PageRequest};
use crate::application::use_cases::curate_dossier::CurateDossier;
use crate::application::use_cases::fetch_recent_dossiers::FetchRecentDossiers;
use crate::application::use_cases::get_dossier_detail::GetDossierDetail;
use crate::application::use_cases::refresh_actor_registry::RefreshActorRegistry;
use crate::application::use_cases::refresh_all::RefreshAll;
use crate::application::use_cases::refresh_dossiers::RefreshScope;
use crate::application::use_cases::save_dossier::SaveDossier;
use crate::application::use_cases::suggest_dossiers::SuggestDossiers;
use crate::domain::dossier::DossierUid;
use crate::infrastructure::config;
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

/// Liste paginée de tous les dossiers, du plus récent au plus ancien.
pub async fn browse_dossiers(
    State(state): State<AppState>,
    Query(params): Query<DossierPageQuery>,
) -> Result<Json<DossierPageResponse>, (StatusCode, String)> {
    let request = PageRequest::new(params.page, params.per_page);
    let uc = BrowseDossiers::new(state.dossier_repository.as_ref());

    let page = uc
        .execute(request)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(DossierPageResponse {
        page: request.page(),
        per_page: request.per_page(),
        total: page.total,
        total_pages: page.total.div_euclid(request.per_page())
            + i64::from(page.total.rem_euclid(request.per_page()) > 0),
        dossiers: page.items.into_iter().map(DossierDto::from).collect(),
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
        state.actor_repository.as_ref(),
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
        state.actor_repository.as_ref(),
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

/// Rafraichit le referentiel puis les dossiers, dans cet ordre.
///
/// Par defaut seuls les dossiers dont la source a bouge sont reecrits.
/// `?full=true` force la reecriture complete, necessaire apres un changement
/// de regle de derivation (score, sort, rattachement).
pub async fn refresh_dossiers(
    State(state): State<AppState>,
    Query(query): Query<RefreshQuery>,
) -> Result<Json<RefreshResponse>, (StatusCode, String)> {
    let scope = if query.full {
        RefreshScope::Full
    } else {
        RefreshScope::Incremental
    };

    let uc = RefreshAll::new(
        state.actor_source.as_ref(),
        state.actor_repository.as_ref(),
        state.assembly_source.as_ref(),
        state.dossier_repository.as_ref(),
        state.scrutin_source.as_ref(),
        state.scrutin_repository.as_ref(),
        state.amendment_source.as_ref(),
        state.amendment_repository.as_ref(),
        state.theme_repository.as_ref(),
        state.theme_classifier.as_ref(),
        config::theme_batch_per_refresh(),
        config::amendment_batch_per_refresh(),
    );

    let outcome = uc
        .execute_with(scope, Utc::now().date_naive())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(RefreshResponse::from(outcome)))
}

pub async fn refresh_actor_registry(
    State(state): State<AppState>,
) -> Result<Json<RegistryResponse>, (StatusCode, String)> {
    let uc = RefreshActorRegistry::new(
        state.actor_source.as_ref(),
        state.actor_repository.as_ref(),
    );

    let summary = uc
        .execute()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(RegistryResponse::from(summary)))
}
