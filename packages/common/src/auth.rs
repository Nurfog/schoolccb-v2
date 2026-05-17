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

pub fn forbidden_msg(roles: &[&str], current: &str) -> String {
    format!(
        "Se requiere uno de los roles {:?}, tiene '{}'",
        roles, current
    )
}
