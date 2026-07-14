use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
};

use crate::{AppState, adapters::stripe::verify_webhook, error::ApiError};

pub async fn receive(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    let signature = headers
        .get("stripe-signature")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::BAD_REQUEST,
                "INVALID_WEBHOOK",
                "Invalid webhook",
            )
        })?;
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let event = verify_webhook(
        &body,
        signature,
        state.stripe_webhook_secret.as_bytes(),
        now,
    )
    .map_err(|_| {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            "INVALID_WEBHOOK",
            "Invalid webhook",
        )
    })?;
    if let Some(payment) = event.verified_payment() {
        state
            .orders
            .apply_payment_event(payment)
            .await
            .map_err(super::quotes::map_repository_error)?;
    }
    Ok(StatusCode::OK)
}
