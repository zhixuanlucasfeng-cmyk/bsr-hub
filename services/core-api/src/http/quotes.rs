use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    AppState,
    domain::{
        billing::{BillingError, billable_units},
        quote::{FulfillmentMethod, QuoteBreakdown, QuoteError, QuoteInput, calculate_quote},
    },
    error::ApiError,
    ports::order_repository::ReserveError,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct QuoteRequest {
    pub(crate) listing_id: Uuid,
    pub(crate) start_at: Option<String>,
    pub(crate) end_at: Option<String>,
    pub(crate) fulfillment: FulfillmentMethod,
}

pub struct PreparedQuote {
    pub quote: QuoteBreakdown,
    pub start_at: OffsetDateTime,
    pub end_at: OffsetDateTime,
}

pub async fn create(
    State(state): State<AppState>,
    Json(request): Json<QuoteRequest>,
) -> Result<Json<QuoteBreakdown>, ApiError> {
    Ok(Json(prepare_quote(&state, request).await?.quote))
}

pub async fn prepare_quote(
    state: &AppState,
    request: QuoteRequest,
) -> Result<PreparedQuote, ApiError> {
    let (start_at, end_at) = parse_rental_window(request.start_at, request.end_at)?;
    let pricing = state
        .orders
        .pricing(request.listing_id)
        .await
        .map_err(map_repository_error)?;
    if !pricing
        .allowed_fulfillment_methods
        .contains(&request.fulfillment)
    {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "FULFILLMENT_NOT_ALLOWED",
            "Fulfillment method is not available for this listing",
        ));
    }
    let units =
        billable_units(start_at, end_at, pricing.billing_unit).map_err(map_billing_error)?;
    let quote = calculate_quote(
        pricing,
        QuoteInput {
            units,
            fulfillment: request.fulfillment,
        },
    )
    .map_err(map_quote_error)?;
    Ok(PreparedQuote {
        quote,
        start_at,
        end_at,
    })
}

fn parse_rental_window(
    start_at: Option<String>,
    end_at: Option<String>,
) -> Result<(OffsetDateTime, OffsetDateTime), ApiError> {
    let (Some(start_at), Some(end_at)) = (start_at, end_at) else {
        return Err(invalid_rental_window());
    };
    let format = &time::format_description::well_known::Rfc3339;
    let start_at = OffsetDateTime::parse(&start_at, format).map_err(|_| invalid_rental_window())?;
    let end_at = OffsetDateTime::parse(&end_at, format).map_err(|_| invalid_rental_window())?;
    if start_at >= end_at {
        return Err(invalid_rental_window());
    }
    Ok((start_at, end_at))
}

fn invalid_rental_window() -> ApiError {
    ApiError::new(
        StatusCode::UNPROCESSABLE_ENTITY,
        "INVALID_RENTAL_WINDOW",
        "Provide startAt and endAt as valid timestamps with endAt after startAt",
    )
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
        ReserveError::InvalidPricing => ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "INVALID_PRICING_PROFILE",
            "Pricing profile is invalid",
        ),
        ReserveError::PricingNotFound => ApiError::new(
            StatusCode::NOT_FOUND,
            "PRICING_PROFILE_NOT_FOUND",
            "Listing pricing is not configured",
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

fn map_billing_error(_error: BillingError) -> ApiError {
    invalid_rental_window()
}
