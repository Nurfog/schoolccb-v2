use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum NotifError {
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
}

impl IntoResponse for NotifError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            NotifError::Database(e) => {
                tracing::error!("Database error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error interno del servidor".into(),
                )
            }
            NotifError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
            NotifError::Validation(m) => (StatusCode::BAD_REQUEST, m.clone()),
            NotifError::Unauthorized => (StatusCode::UNAUTHORIZED, "No autorizado".into()),
            NotifError::Forbidden(m) => (StatusCode::FORBIDDEN, m.clone()),
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}

impl From<schoolccb_common::auth::AuthError> for NotifError {
    fn from(e: schoolccb_common::auth::AuthError) -> Self {
        match e {
            schoolccb_common::auth::AuthError::Unauthorized => NotifError::Unauthorized,
            schoolccb_common::auth::AuthError::Forbidden(msg) => NotifError::Forbidden(msg),
            schoolccb_common::auth::AuthError::TokenExpired => NotifError::Unauthorized,
            schoolccb_common::auth::AuthError::TokenInvalid(_msg) => NotifError::Unauthorized,
        }
    }
}

pub type NotifResult<T> = Result<T, NotifError>;
