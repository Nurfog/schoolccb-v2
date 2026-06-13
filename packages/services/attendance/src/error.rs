use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AttendanceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AttendanceError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AttendanceError::Database(e) => {
                tracing::error!("Database error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error interno del servidor".into(),
                )
            }
            AttendanceError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
            AttendanceError::Validation(m) => (StatusCode::BAD_REQUEST, m.clone()),
            AttendanceError::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
            AttendanceError::Unauthorized => (StatusCode::UNAUTHORIZED, "No autorizado".into()),
            AttendanceError::Forbidden(m) => (StatusCode::FORBIDDEN, m.clone()),
            AttendanceError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m.clone()),
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}

impl From<schoolccb_common::auth::AuthError> for AttendanceError {
    fn from(e: schoolccb_common::auth::AuthError) -> Self {
        match e {
            schoolccb_common::auth::AuthError::Unauthorized => AttendanceError::Unauthorized,
            schoolccb_common::auth::AuthError::Forbidden(msg) => AttendanceError::Forbidden(msg),
            schoolccb_common::auth::AuthError::TokenExpired => AttendanceError::Unauthorized,
            schoolccb_common::auth::AuthError::TokenInvalid(_msg) => AttendanceError::Unauthorized,
        }
    }
}

pub type AttendanceResult<T> = Result<T, AttendanceError>;
