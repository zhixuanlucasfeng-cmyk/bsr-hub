use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

#[derive(Serialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorBody {
    code: &'static str,
    message: String,
    request_id: Uuid,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }

    pub fn auth_required() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "AUTH_REQUIRED",
            "Sign in to continue",
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = ErrorEnvelope {
            error: ErrorBody {
                code: self.code,
                message: self.message,
                request_id: Uuid::new_v4(),
            },
        };
        (self.status, Json(body)).into_response()
    }
}
