use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum CrmError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Token invalid: {0}")]
    TokenInvalid(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("External service error: {0}")]
    External(String),
}

impl From<String> for CrmError {
    fn from(s: String) -> Self {
        CrmError::External(s)
    }
}

impl IntoResponse for CrmError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            CrmError::Database(e) => {
                tracing::error!("Database error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Error interno del servidor".into())
            }
            CrmError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
            CrmError::Unauthorized => (StatusCode::UNAUTHORIZED, "No autorizado".into()),
            CrmError::Forbidden(m) => (StatusCode::FORBIDDEN, m.clone()),
            CrmError::TokenExpired => (StatusCode::UNAUTHORIZED, "Sesión expirada".into()),
            CrmError::TokenInvalid(m) => (StatusCode::UNAUTHORIZED, m.clone()),
            CrmError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m.clone()),
            CrmError::Validation(m) => (StatusCode::BAD_REQUEST, m.clone()),
            CrmError::External(m) => (StatusCode::SERVICE_UNAVAILABLE, m.clone()),
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}

pub type CrmResult<T> = Result<T, CrmError>;
