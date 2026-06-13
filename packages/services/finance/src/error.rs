use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum FinanceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for FinanceError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            FinanceError::Database(e) => {
                tracing::error!("Database error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error interno del servidor".into(),
                )
            }
            FinanceError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
            FinanceError::Validation(m) => (StatusCode::BAD_REQUEST, m.clone()),
            FinanceError::Unauthorized => (StatusCode::UNAUTHORIZED, "No autorizado".into()),
            FinanceError::Forbidden(m) => (StatusCode::FORBIDDEN, m.clone()),
            FinanceError::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
            FinanceError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m.clone()),
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}

impl From<schoolccb_common::auth::AuthError> for FinanceError {
    fn from(e: schoolccb_common::auth::AuthError) -> Self {
        match e {
            schoolccb_common::auth::AuthError::Unauthorized => FinanceError::Unauthorized,
            schoolccb_common::auth::AuthError::Forbidden(msg) => FinanceError::Forbidden(msg),
            schoolccb_common::auth::AuthError::TokenExpired => FinanceError::Unauthorized,
            schoolccb_common::auth::AuthError::TokenInvalid(_msg) => FinanceError::Unauthorized,
        }
    }
}

pub type FinanceResult<T> = Result<T, FinanceError>;
