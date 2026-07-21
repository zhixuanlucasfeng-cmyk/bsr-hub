use axum::http::{HeaderMap, header::AUTHORIZATION};

use crate::{AppState, auth::AuthUser, error::ApiError};

pub async fn require_auth(state: &AppState, headers: &HeaderMap) -> Result<AuthUser, ApiError> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| !value.is_empty())
        .ok_or_else(ApiError::auth_required)?;

    state
        .auth
        .verify(token)
        .await
        .map_err(|_| ApiError::auth_required())
}
