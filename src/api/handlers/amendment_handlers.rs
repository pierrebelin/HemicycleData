use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::api::amendment_dto::{AmendmentsRefreshResponse, DossierAmendmentsResponse};
use crate::application::ports::amendment_repository::AmendmentPageRequest;
use crate::application::use_cases::browse_dossier_amendments::BrowseDossierAmendments;
use crate::application::use_cases::refresh_amendments::RefreshAmendments;
use crate::infrastructure::config;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct AmendmentPageQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// CU-02 — Amendements d'un dossier.
///
/// Route fille plutot qu'un gonflement de `DossierDetailDto`: un dossier
/// budgetaire porte des milliers d'amendements, et la page dossier doit
/// s'afficher sans les attendre. Meme forme que `/api/dossiers/{uid}/scrutins`.
pub async fn get_dossier_amendments(
    State(state): State<AppState>,
    Path(uid): Path<String>,
    Query(query): Query<AmendmentPageQuery>,
) -> Result<Json<DossierAmendmentsResponse>, (StatusCode, String)> {
    let page = AmendmentPageRequest::new(query.limit, query.offset);

    let amendments = BrowseDossierAmendments::new(
        state.amendment_repository.as_ref(),
        state.actor_repository.as_ref(),
    )
    .execute(&uid, &page)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(DossierAmendmentsResponse::new(
        amendments,
        page.offset,
        page.limit,
    )))
}

/// CU-01 — Ingestion des amendements.
pub async fn refresh_amendments(
    State(state): State<AppState>,
) -> Result<Json<AmendmentsRefreshResponse>, (StatusCode, String)> {
    let summary = RefreshAmendments::new(
        state.amendment_source.as_ref(),
        state.amendment_repository.as_ref(),
        state.actor_repository.as_ref(),
        config::amendment_batch_per_refresh(),
    )
    .execute()
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AmendmentsRefreshResponse::from(summary)))
}
