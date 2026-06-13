use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

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
}

impl From<schoolccb_common::auth::AuthError> for AuthError {
    fn from(e: schoolccb_common::auth::AuthError) -> Self {
        match e {
            schoolccb_common::auth::AuthError::Unauthorized => AuthError::Unauthorized,
            schoolccb_common::auth::AuthError::Forbidden(msg) => AuthError::Forbidden(msg),
            schoolccb_common::auth::AuthError::TokenExpired => AuthError::TokenExpired,
            schoolccb_common::auth::AuthError::TokenInvalid(msg) => AuthError::TokenInvalid(msg),
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AuthError::Database(e) => {
                tracing::error!("Database error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error interno del servidor".into(),
                )
            }
            AuthError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Credenciales inválidas".into())
            }
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, "Usuario no encontrado".into()),
            AuthError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
            AuthError::Unauthorized => (StatusCode::UNAUTHORIZED, "No autorizado".into()),
            AuthError::Forbidden(m) => (StatusCode::FORBIDDEN, m.clone()),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "Sesión expirada".into()),
            AuthError::TokenInvalid(m) => (StatusCode::UNAUTHORIZED, m.clone()),
            AuthError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m.clone()),
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}

pub type AuthResult<T> = Result<T, AuthError>;
