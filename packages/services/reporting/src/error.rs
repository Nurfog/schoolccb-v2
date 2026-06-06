use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ReportError {
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

    #[error("{0}")]
    Other(String),
}

impl IntoResponse for ReportError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ReportError::Database(e) => {
                tracing::error!("Database error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error interno del servidor".into(),
                )
            }
            ReportError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
            ReportError::Validation(m) => (StatusCode::BAD_REQUEST, m.clone()),
            ReportError::Unauthorized => (StatusCode::UNAUTHORIZED, "No autorizado".into()),
            ReportError::Forbidden(m) => (StatusCode::FORBIDDEN, m.clone()),
            ReportError::Other(m) => (StatusCode::INTERNAL_SERVER_ERROR, m.clone()),
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}

pub type ReportResult<T> = Result<T, ReportError>;
