use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    AppState,
    domain::quote::{QuoteBreakdown, QuoteError, QuoteInput, calculate_quote},
    error::ApiError,
    ports::order_repository::ReserveError,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    listing_id: Uuid,
    units: i64,
    wants_delivery: bool,
}

pub async fn create(
    State(state): State<AppState>,
    Json(request): Json<QuoteRequest>,
) -> Result<Json<QuoteBreakdown>, ApiError> {
    let pricing = state
        .orders
        .pricing(request.listing_id)
        .await
        .map_err(map_repository_error)?;
    let quote = calculate_quote(
        pricing,
        QuoteInput {
            units: request.units,
            wants_delivery: request.wants_delivery,
        },
    )
    .map_err(map_quote_error)?;
    Ok(Json(quote))
}

pub(crate) fn map_repository_error(error: ReserveError) -> ApiError {
    match error {
        ReserveError::NotFound => ApiError::new(
            StatusCode::NOT_FOUND,
            "LISTING_NOT_FOUND",
            "Listing not found",
        ),
        ReserveError::Unavailable => ApiError::new(
            StatusCode::CONFLICT,
            "LISTING_UNAVAILABLE",
            "Listing is not available for those dates",
        ),
        ReserveError::SelfTransaction => ApiError::new(
            StatusCode::CONFLICT,
            "SELF_TRANSACTION",
            "You cannot transact with your own listing",
        ),
        ReserveError::Forbidden => ApiError::new(
            StatusCode::FORBIDDEN,
            "ACTION_FORBIDDEN",
            "You cannot perform that action",
        ),
        ReserveError::InvalidTransition => ApiError::new(
            StatusCode::CONFLICT,
            "INVALID_ORDER_TRANSITION",
            "That order action is not allowed in its current state",
        ),
        ReserveError::Database(_) => ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "SERVICE_UNAVAILABLE",
            "Please try again later",
        ),
    }
}

pub(crate) fn map_quote_error(_error: QuoteError) -> ApiError {
    ApiError::new(
        StatusCode::UNPROCESSABLE_ENTITY,
        "INVALID_QUOTE_INPUT",
        "Quote input is invalid",
    )
}
