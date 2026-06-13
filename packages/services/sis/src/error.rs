use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum SisError {
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

impl IntoResponse for SisError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            SisError::Database(e) => {
                tracing::error!("Database error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error interno del servidor".into(),
                )
            }
            SisError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
            SisError::Validation(m) => (StatusCode::BAD_REQUEST, m.clone()),
            SisError::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
            SisError::Unauthorized => (StatusCode::UNAUTHORIZED, "No autorizado".into()),
            SisError::Forbidden(m) => (StatusCode::FORBIDDEN, m.clone()),
            SisError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m.clone()),
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}

impl From<schoolccb_common::auth::AuthError> for SisError {
    fn from(e: schoolccb_common::auth::AuthError) -> Self {
        match e {
            schoolccb_common::auth::AuthError::Unauthorized => SisError::Unauthorized,
            schoolccb_common::auth::AuthError::Forbidden(msg) => SisError::Forbidden(msg),
            schoolccb_common::auth::AuthError::TokenExpired => SisError::Unauthorized,
            schoolccb_common::auth::AuthError::TokenInvalid(_msg) => SisError::Unauthorized,
        }
    }
}

pub type SisResult<T> = Result<T, SisError>;
