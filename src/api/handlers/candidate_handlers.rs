use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::api::candidate_dto::{CandidateComparisonResponse, CandidateListQuery};
use crate::application::use_cases::browse_candidates::{
    BrowseCandidates, BrowseCandidatesCommand, BrowseCandidatesError,
};
use crate::AppState;

/// Candidatures 2027, programmes attribués et groupes associés avec source.
pub async fn list_candidates(
    State(state): State<AppState>,
    Query(params): Query<CandidateListQuery>,
) -> Result<Json<CandidateComparisonResponse>, (StatusCode, String)> {
    let command: BrowseCandidatesCommand = params.into();
    let view = BrowseCandidates::new(state.candidate_repository.as_ref())
        .execute(command)
        .await
        .map_err(status_of)?;
    Ok(Json(view.into()))
}

fn status_of(error: BrowseCandidatesError) -> (StatusCode, String) {
    let status = match error {
        BrowseCandidatesError::Repository(_) => StatusCode::INTERNAL_SERVER_ERROR,
        _ => StatusCode::BAD_REQUEST,
    };
    (status, error.to_string())
}
