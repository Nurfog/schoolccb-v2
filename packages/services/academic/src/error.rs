use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AcademicError {
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

impl IntoResponse for AcademicError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AcademicError::Database(e) => {
                tracing::error!("Database error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error interno del servidor".into(),
                )
            }
            AcademicError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
            AcademicError::Validation(m) => (StatusCode::BAD_REQUEST, m.clone()),
            AcademicError::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
            AcademicError::Unauthorized => (StatusCode::UNAUTHORIZED, "No autorizado".into()),
            AcademicError::Forbidden(m) => (StatusCode::FORBIDDEN, m.clone()),
            AcademicError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m.clone()),
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}

impl From<schoolccb_common::auth::AuthError> for AcademicError {
    fn from(e: schoolccb_common::auth::AuthError) -> Self {
        match e {
            schoolccb_common::auth::AuthError::Unauthorized => AcademicError::Unauthorized,
            schoolccb_common::auth::AuthError::Forbidden(msg) => AcademicError::Forbidden(msg),
            schoolccb_common::auth::AuthError::TokenExpired => AcademicError::Unauthorized,
            schoolccb_common::auth::AuthError::TokenInvalid(_msg) => AcademicError::Unauthorized,
        }
    }
}

pub type AcademicResult<T> = Result<T, AcademicError>;
