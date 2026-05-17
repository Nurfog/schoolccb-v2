use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum CurriculumError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden: {0}")]
    Forbidden(String),
}

impl IntoResponse for CurriculumError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            CurriculumError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "No autorizado".to_string())
            }
            CurriculumError::Forbidden(m) => (StatusCode::FORBIDDEN, m.clone()),
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}

pub type CurriculumResult<T> = Result<T, CurriculumError>;
