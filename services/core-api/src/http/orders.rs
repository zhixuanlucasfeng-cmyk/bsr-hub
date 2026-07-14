use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    AppState,
    domain::{
        order_state::{OrderAction, OrderState},
        quote::{FulfillmentMethod, QuoteInput, calculate_quote},
    },
    error::ApiError,
    ports::{
        order_repository::CreateOrder,
        payment_gateway::{CheckoutRequest, PaymentError},
    },
};

use super::quotes::{map_quote_error, map_repository_error};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderRequest {
    listing_id: Uuid,
    units: i64,
    wants_delivery: bool,
    start_at: Option<String>,
    end_at: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderResponse {
    order_id: Uuid,
    reservation_expires_at: time::OffsetDateTime,
    total_cents: i64,
    checkout_url: String,
}

pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateOrderRequest>,
) -> Result<(StatusCode, Json<CreateOrderResponse>), ApiError> {
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
    let (start_at, end_at) = parse_rental_window(request.start_at, request.end_at)?;
    let pricing = state
        .orders
        .pricing(request.listing_id)
        .await
        .map_err(map_repository_error)?;
    let quote = calculate_quote(
        pricing,
        QuoteInput {
            units: request.units,
            fulfillment: if request.wants_delivery {
                FulfillmentMethod::Delivery
            } else {
                FulfillmentMethod::Pickup
            },
        },
    )
    .map_err(map_quote_error)?;
    let reserved = state
        .orders
        .reserve(CreateOrder {
            listing_id: request.listing_id,
            buyer_id: user.user_id,
            start_at,
            end_at,
            quote: quote.clone(),
        })
        .await
        .map_err(map_repository_error)?;
    let checkout = state
        .payments
        .create_checkout(CheckoutRequest {
            order_id: reserved.order_id,
            amount_cents: quote.total_cents,
        })
        .await
        .map_err(map_payment_error)?;
    Ok((
        StatusCode::CREATED,
        Json(CreateOrderResponse {
            order_id: reserved.order_id,
            reservation_expires_at: reserved.expires_at,
            total_cents: quote.total_cents,
            checkout_url: checkout.checkout_url,
        }),
    ))
}

fn parse_rental_window(
    start_at: Option<String>,
    end_at: Option<String>,
) -> Result<(Option<time::OffsetDateTime>, Option<time::OffsetDateTime>), ApiError> {
    match (start_at, end_at) {
        (None, None) => Ok((None, None)),
        (Some(start), Some(end)) => {
            let format = &time::format_description::well_known::Rfc3339;
            let start =
                time::OffsetDateTime::parse(&start, format).map_err(|_| invalid_rental_window())?;
            let end =
                time::OffsetDateTime::parse(&end, format).map_err(|_| invalid_rental_window())?;
            if start >= end {
                return Err(invalid_rental_window());
            }
            Ok((Some(start), Some(end)))
        }
        _ => Err(invalid_rental_window()),
    }
}

fn invalid_rental_window() -> ApiError {
    ApiError::new(
        StatusCode::UNPROCESSABLE_ENTITY,
        "INVALID_RENTAL_WINDOW",
        "Provide both startAt and endAt as valid timestamps",
    )
}

fn map_payment_error(_error: PaymentError) -> ApiError {
    ApiError::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "PAYMENT_UNAVAILABLE",
        "Payment setup is temporarily unavailable",
    )
}

#[derive(Deserialize)]
pub struct TransitionRequest {
    action: OrderAction,
}

#[derive(Serialize)]
pub struct TransitionResponse {
    status: OrderState,
}

pub async fn transition(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<TransitionRequest>,
) -> Result<Json<TransitionResponse>, ApiError> {
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
    let status = state
        .orders
        .transition(order_id, user.user_id, request.action)
        .await
        .map_err(map_repository_error)?;
    Ok(Json(TransitionResponse { status }))
}
