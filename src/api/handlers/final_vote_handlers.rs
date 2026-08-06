use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::api::final_vote_dto::{FinalVoteListQuery, FinalVoteListResponse};
use crate::application::use_cases::browse_final_votes::{
    BrowseFinalVotes, BrowseFinalVotesCommand, BrowseFinalVotesError,
};
use crate::AppState;

/// CU-07 — Votes sur l'ensemble d'un texte, groupe par groupe.
pub async fn list_final_votes(
    State(state): State<AppState>,
    Query(params): Query<FinalVoteListQuery>,
) -> Result<Json<FinalVoteListResponse>, (StatusCode, String)> {
    let command: BrowseFinalVotesCommand = params.into();
    let offset = command.offset.unwrap_or(0).max(0);

    let view = BrowseFinalVotes::new(state.final_vote_repository.as_ref())
        .execute(command)
        .await
        .map_err(status_of)?;

    Ok(Json(FinalVoteListResponse::from((view, offset))))
}

/// Une demande mal formee est une erreur du visiteur, pas du serveur: le
/// distinguer evite de faire passer un sigle inconnu pour une panne.
fn status_of(error: BrowseFinalVotesError) -> (StatusCode, String) {
    let status = match error {
        BrowseFinalVotesError::Repository(_) => StatusCode::INTERNAL_SERVER_ERROR,
        _ => StatusCode::BAD_REQUEST,
    };
    (status, error.to_string())
}
