use axum::http::request::Parts;
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub name: String,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
    pub school_id: Option<String>,
    pub corporation_id: Option<String>,
    pub admin_type: Option<String>,
}

impl<S> axum::extract::FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AuthError::Unauthorized)?;

        // Note: jwt_secret must be passed via extensions or state in real apps
        // This is a simplified version - in production, extract from state
        let jwt_secret = parts
            .extensions
            .get::<JwtSecret>()
            .map(|s| s.0.as_str())
            .unwrap_or("fallback-secret-only-for-development");

        let token_data = decode_token(auth_header, jwt_secret)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::TokenInvalid("Token inválido".into()),
            })?;

        Ok(token_data)
    }
}

#[derive(Clone)]
pub struct JwtSecret(pub String);

#[derive(Debug)]
pub enum AuthError {
    Unauthorized,
    Forbidden(String),
    TokenExpired,
    TokenInvalid(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::Unauthorized => write!(f, "No autenticado"),
            AuthError::Forbidden(msg) => write!(f, "{}", msg),
            AuthError::TokenExpired => write!(f, "Token expirado"),
            AuthError::TokenInvalid(msg) => write!(f, "{}", msg),
        }
    }
}

impl axum::response::IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AuthError::Unauthorized => (
                axum::http::StatusCode::UNAUTHORIZED,
                "No autenticado".to_string(),
            ),
            AuthError::Forbidden(msg) => (axum::http::StatusCode::FORBIDDEN, msg),
            AuthError::TokenExpired => (
                axum::http::StatusCode::UNAUTHORIZED,
                "Token expirado".to_string(),
            ),
            AuthError::TokenInvalid(msg) => (
                axum::http::StatusCode::UNAUTHORIZED,
                msg,
            ),
        };

        let body = axum::Json(serde_json::json!({ "error": message }));
        (status, body).into_response()
    }
}

/// Extrae el token Bearer del header Authorization.
pub fn extract_bearer(parts: &Parts) -> Option<&str> {
    parts
        .headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}

/// Decodifica y valida un JWT.
pub fn decode_token(token: &str, jwt_secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

pub fn has_role(role: &str, allowed: &[&str]) -> bool {
    role == "GerenteGeneral" || allowed.contains(&role)
}

pub fn require_role(claims: &Claims, required: &str) -> Result<(), AuthError> {
    if claims.role == "GerenteGeneral" || claims.role == required {
        return Ok(());
    }
    Err(AuthError::Forbidden(format!(
        "Se requiere rol '{}', tiene '{}'",
        required, claims.role
    )))
}

pub fn require_any_role(claims: &Claims, roles: &[&str]) -> Result<(), AuthError> {
    if claims.role == "GerenteGeneral" || roles.contains(&claims.role.as_str()) {
        return Ok(());
    }
    Err(AuthError::Forbidden(format!(
        "Se requiere uno de los roles {:?}, tiene '{}'",
        roles, claims.role
    )))
}

pub fn forbidden_msg(roles: &[&str], current: &str) -> String {
    format!(
        "Se requiere uno de los roles {:?}, tiene '{}'",
        roles, current
    )
}
