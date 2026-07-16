use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    AppState,
    domain::order_state::{OrderAction, OrderState},
    error::ApiError,
    ports::{
        order_repository::CreateOrder,
        payment_gateway::{CheckoutRequest, PaymentError},
    },
};

use super::quotes::{QuoteRequest, map_repository_error, prepare_quote};

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
    Json(request): Json<QuoteRequest>,
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
    let listing_id = request.listing_id;
    let fulfillment = request.fulfillment;
    let prepared = prepare_quote(&state, request).await?;
    let quote = prepared.quote;
    let reserved = state
        .orders
        .reserve(CreateOrder {
            listing_id,
            buyer_id: user.user_id,
            start_at: Some(prepared.start_at),
            end_at: Some(prepared.end_at),
            fulfillment,
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
