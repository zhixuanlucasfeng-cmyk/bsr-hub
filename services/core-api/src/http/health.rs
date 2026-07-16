use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;

use crate::{AppState, error::ApiError};

#[derive(Serialize)]
pub struct Health {
    status: &'static str,
}

pub async fn get() -> Json<Health> {
    Json(Health { status: "ok" })
}

#[derive(Serialize)]
pub struct Readiness {
    status: &'static str,
}

pub async fn ready(State(state): State<AppState>) -> Result<Json<Readiness>, ApiError> {
    state.orders.readiness().await.map_err(|_| {
        ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "DATABASE_UNAVAILABLE",
            "Database is temporarily unavailable",
        )
    })?;
    Ok(Json(Readiness { status: "ready" }))
}
