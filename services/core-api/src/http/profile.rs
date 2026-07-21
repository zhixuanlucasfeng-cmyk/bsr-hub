use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use serde::Deserialize;

use crate::{
    AppState,
    error::ApiError,
    ports::profile_repository::{ProfileError, ProfilePatch, UserProfile},
};

use super::auth_extract::require_auth;

pub async fn bootstrap(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<UserProfile>), ApiError> {
    let user = require_auth(&state, &headers).await?;
    let profile = state
        .profiles
        .bootstrap(user.user_id)
        .await
        .map_err(map_profile_error)?;
    Ok((StatusCode::OK, Json(profile)))
}

pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UserProfile>, ApiError> {
    let user = require_auth(&state, &headers).await?;
    let profile = state
        .profiles
        .get_by_auth_user_id(user.user_id)
        .await
        .map_err(map_profile_error)?;
    Ok(Json(profile))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<Option<String>>,
}

pub async fn update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfile>, ApiError> {
    let user = require_auth(&state, &headers).await?;
    let profile = state
        .profiles
        .update(
            user.user_id,
            ProfilePatch {
                display_name: request.display_name,
                avatar_url: request.avatar_url,
            },
        )
        .await
        .map_err(map_profile_error)?;
    Ok(Json(profile))
}

fn map_profile_error(error: ProfileError) -> ApiError {
    match error {
        ProfileError::NotFound => ApiError::new(
            StatusCode::NOT_FOUND,
            "PROFILE_NOT_FOUND",
            "Profile was not found",
        ),
        ProfileError::Invalid => ApiError::new(
            StatusCode::BAD_REQUEST,
            "INVALID_PROFILE",
            "Profile details are invalid",
        ),
        ProfileError::Unavailable => ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "PROFILE_UNAVAILABLE",
            "Profile service is temporarily unavailable",
        ),
    }
}
