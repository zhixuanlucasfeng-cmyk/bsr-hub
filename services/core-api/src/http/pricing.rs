use std::collections::HashSet;

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    AppState,
    domain::{
        pricing::{PricingCategoryInput, PricingError, recommend},
        quote::FulfillmentMethod,
    },
    error::ApiError,
    ports::order_repository::SavePricingProfile,
};

use super::quotes::map_repository_error;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavePricingRequest {
    attributes: PricingCategoryInput,
    seller_adjustment_cents: i64,
    allowed_fulfillment_methods: Vec<FulfillmentMethod>,
}

pub async fn save(
    State(state): State<AppState>,
    Path(listing_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<SavePricingRequest>,
) -> Result<Json<crate::ports::order_repository::StoredPricingProfile>, ApiError> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| !value.is_empty())
        .ok_or_else(ApiError::auth_required)?;
    let user = state
        .auth
        .verify(token)
        .await
        .map_err(|_| ApiError::auth_required())?;
    validate_fulfillment(&request.attributes, &request.allowed_fulfillment_methods)?;
    let recommendation = recommend(request.attributes.clone()).map_err(map_pricing_error)?;
    let final_unit_price_cents = recommendation
        .final_price_cents(request.seller_adjustment_cents)
        .map_err(map_pricing_error)?;
    let stored = state
        .orders
        .save_pricing_profile(SavePricingProfile {
            listing_id,
            owner_id: user.user_id,
            attributes: request.attributes,
            recommendation,
            seller_adjustment_cents: request.seller_adjustment_cents,
            final_unit_price_cents,
            allowed_fulfillment_methods: request.allowed_fulfillment_methods,
        })
        .await
        .map_err(map_repository_error)?;
    Ok(Json(stored))
}

fn validate_fulfillment(
    attributes: &PricingCategoryInput,
    methods: &[FulfillmentMethod],
) -> Result<(), ApiError> {
    if methods.is_empty() || methods.iter().copied().collect::<HashSet<_>>().len() != methods.len()
    {
        return Err(invalid_fulfillment());
    }
    let valid = match attributes {
        PricingCategoryInput::Ps5(_) => methods
            .iter()
            .all(|method| *method != FulfillmentMethod::OnSite),
        PricingCategoryInput::Workspace(_) => methods
            .iter()
            .all(|method| *method == FulfillmentMethod::OnSite),
    };
    if !valid {
        return Err(invalid_fulfillment());
    }
    Ok(())
}

fn invalid_fulfillment() -> ApiError {
    ApiError::new(
        StatusCode::UNPROCESSABLE_ENTITY,
        "FULFILLMENT_NOT_ALLOWED",
        "Fulfillment is not valid for this pricing category",
    )
}

fn map_pricing_error(error: PricingError) -> ApiError {
    let code = match error {
        PricingError::AdjustmentOutOfRange => "SELLER_ADJUSTMENT_OUT_OF_RANGE",
        PricingError::InvalidAttributes | PricingError::NotOperational | PricingError::Overflow => {
            "INVALID_PRICING_ATTRIBUTES"
        }
    };
    ApiError::new(
        StatusCode::UNPROCESSABLE_ENTITY,
        code,
        "Pricing input is invalid",
    )
}
