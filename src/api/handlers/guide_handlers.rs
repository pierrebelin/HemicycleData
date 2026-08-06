use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::api::guide_dto::DatasetResponse;
use crate::application::use_cases::describe_dataset::DescribeDataset;
use crate::AppState;

/// Chiffres du guide de lecture (page « Comprendre »).
pub async fn get_dataset(
    State(state): State<AppState>,
) -> Result<Json<DatasetResponse>, (StatusCode, String)> {
    let overview = DescribeDataset::new(
        state.scrutin_repository.as_ref(),
        state.theme_repository.as_ref(),
    )
    .execute()
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(DatasetResponse::from(overview)))
}
