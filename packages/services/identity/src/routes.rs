use axum::{
    Json, Router,
    extract::{FromRequestParts, Path, Query, State},
    http::{HeaderMap, HeaderValue, header, request::Parts},
    routing::{delete, get, post, put},
};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{AuthError, AuthResult};
use crate::models::{self, Claims};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/auth/login", post(login))
        .route("/api/auth/forgot-password", post(forgot_password))
        .route("/api/auth/reset-password", post(reset_password))
        .route("/api/auth/me", get(me))
        .route("/api/auth/register", post(register))
        .route("/api/auth/refresh", post(refresh))
        .route("/api/auth/revoke-all", post(revoke_all))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/users", get(list_users))
        .route("/api/auth/users/{id}/role", put(update_role))
        .route("/api/auth/users/{id}/toggle", post(toggle_active))
        .route("/api/user/modules", get(list_modules))
        .route("/api/user/my-plan", get(my_plan))
        .route("/api/user/favorites/{module_id}", post(toggle_favorite))
        .route("/api/user/profile", put(update_profile))
        .route("/api/user/password", put(change_password))
        .route("/api/user/preferences", get(get_user_preferences))
        .route("/api/user/preferences", put(update_user_preferences))
        .route("/api/config/branding", get(get_branding))
        .route("/api/auth/my-permissions", get(my_permissions))
        .route("/api/config/branding", put(update_branding))
        .route(
            "/api/corporations",
            get(list_corporations).post(create_corporation),
        )
        .route("/api/corporations/{id}", get(get_corporation))
        .route("/api/corporations/{id}/modules", get(get_corporation_modules))
        .route("/api/schools", get(list_schools).post(create_school))
        .route("/api/schools/{id}", get(get_school).put(update_school))
        .route("/api/schools/{id}/toggle", put(toggle_school))
        .route("/api/legal-representatives", get(list_legal_reps).post(create_legal_rep))
        .route("/api/legal-representatives/{id}", put(update_legal_rep))
        // Internal
        .route("/api/internal/onboarding", post(onboarding))
        .merge(roles_router())
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

fn require_any_role(claims: &Claims, roles: &[&str]) -> Result<(), AuthError> {
    if claims.role == "GerenteGeneral" || roles.contains(&claims.role.as_str()) {
        return Ok(());
    }
    Err(AuthError::Forbidden(format!(
        "Se requiere uno de los roles {:?}, tiene '{}'",
        roles, claims.role
    )))
}

impl FromRequestParts<AppState> for Claims {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AuthError::Unauthorized)?;

        let token_data = jsonwebtoken::decode::<Claims>(
            auth_header,
            &jsonwebtoken::DecodingKey::from_secret(_state.config.jwt_secret.as_bytes()),
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::TokenInvalid("Token inválido".into()),
        })?;

        Ok(token_data.claims)
    }
}

pub(crate) fn generate_token_pair(
    config: &crate::config::Config,
    user_id: Uuid,
    role: &str,
    name: &str,
    email: &str,
    corporation_id: Option<Uuid>,
    school_id: Option<Uuid>,
    admin_type: Option<String>,
) -> Result<(String, Claims), AuthError> {
    let now = chrono::Utc::now();

    let access_claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        name: name.to_string(),
        email: email.to_string(),
        corporation_id: corporation_id.map(|id| id.to_string()),
        school_id: school_id.map(|id| id.to_string()),
        admin_type,
        exp: (now + chrono::Duration::hours(12)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AuthError::Internal(format!("JWT encoding failed: {e}")))?;

    Ok((token, access_claims))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::user::AuthPayload>,
) -> AuthResult<(HeaderMap, Json<Value>)> {
    let user = models::find_by_email(&state.pool, &payload.email)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    if !models::verify_password(&payload.password, &user.password_hash) {
        return Err(AuthError::InvalidCredentials);
    }

    if !user.active {
        return Err(AuthError::Unauthorized);
    }

    let id = user.id;
    let (token, _claims) = generate_token_pair(
        &state.config,
        id,
        &user.role,
        &user.name,
        &user.email,
        user.corporation_id,
        user.school_id,
        user.admin_type.clone(),
    )?;

    let (refresh_token, _) = models::create_refresh_token(&state.pool, id, 7).await?;
    let is_root = user.role == "GerenteGeneral";

    let mut headers = HeaderMap::new();
    if let Ok(cookie) = HeaderValue::from_str(&format!(
        "jwt_token={}; Path=/; SameSite=Lax; Max-Age=43200",
        token
    )) {
        headers.insert(header::SET_COOKIE, cookie);
    }
    if let Ok(marker) = HeaderValue::from_str("session_active=1; Path=/; SameSite=Lax; Max-Age=43200") {
        headers.append(header::SET_COOKIE, marker);
    }

    Ok((headers, Json(json!({
        "token": token,
        "refresh_token": refresh_token,
        "user": {
            "id": user.id,
            "name": user.name,
            "email": user.email,
            "role": user.role,
            "rut": user.rut,
            "corporation_id": user.corporation_id,
            "school_id": user.school_id,
            "admin_type": user.admin_type,
            "is_root": is_root
        }
    }))))
}

async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> AuthResult<Json<Value>> {
    let email = payload.get("email").and_then(|v| v.as_str()).unwrap_or("");
    if email.is_empty() {
        return Err(AuthError::Internal("Email es obligatorio".into()));
    }
    let user = models::find_by_email(&state.pool, email)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    use rand::Rng;
    let token: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(48)
        .map(char::from)
        .collect();
    let token_hash = models::hash_token(&token);

    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
    sqlx::query(
        "INSERT INTO password_reset_tokens (user_id, token_hash, expires_at)
         VALUES ($1, $2, $3)",
    )
    .bind(user.id)
    .bind(&token_hash)
    .bind(expires_at)
    .execute(&state.pool)
    .await?;

    tracing::info!("Password reset token generated for email: {}", email);

    Ok(Json(json!({
        "message": "Si el email está registrado, recibirás un enlace de recuperación",
    })))
}

async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> AuthResult<Json<Value>> {
    let token = payload.get("token").and_then(|v| v.as_str()).unwrap_or("");
    let new_password = payload
        .get("new_password")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if token.is_empty() || new_password.len() < 6 {
        return Err(AuthError::Internal(
            "Token inválido o contraseña muy corta (mín. 6 caracteres)".into(),
        ));
    }

    let token_hash = models::hash_token(token);
    let reset = sqlx::query_as::<_, (Uuid, Uuid, bool)>(
        "SELECT id, user_id, used FROM password_reset_tokens
         WHERE token_hash = $1 AND expires_at > NOW() AND used = false",
    )
    .bind(&token_hash)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AuthError::Internal("Token inválido o expirado".into()))?;

    sqlx::query("UPDATE password_reset_tokens SET used = true WHERE id = $1")
        .bind(reset.0)
        .execute(&state.pool)
        .await?;

    models::change_password(&state.pool, reset.1, new_password).await?;

    Ok(Json(
        json!({ "message": "Contraseña actualizada correctamente" }),
    ))
}

async fn me(claims: Claims, State(state): State<AppState>) -> AuthResult<Json<Value>> {
    let id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError::TokenInvalid("Invalid user ID in token".into()))?;

    let user = models::find_by_id(&state.pool, id)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    let is_root = user.role == "GerenteGeneral";

    Ok(Json(json!({
        "user": {
            "id": user.id,
            "name": user.name,
            "email": user.email,
            "role": user.role,
            "rut": user.rut,
            "corporation_id": user.corporation_id,
            "school_id": user.school_id,
            "is_root": is_root
        }
    })))
}

async fn my_permissions(claims: Claims, State(state): State<AppState>) -> AuthResult<Json<Value>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError::TokenInvalid("Invalid user ID".into()))?;

    let role_ids: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT role_id FROM user_roles WHERE user_id = $1
         UNION
         SELECT id FROM roles WHERE name = $2",
    )
    .bind(user_id)
    .bind(&claims.role)
    .fetch_all(&state.pool)
    .await?;

    let is_full_access = claims.role == "GerenteGeneral" || claims.role == "Sostenedor";

    let perms: Vec<PermEntry> = if is_full_access {
        sqlx::query_as::<_, PermEntry>(
            "SELECT pd.id, pd.module, pd.resource, true as can_write FROM permission_definitions pd ORDER BY pd.module, pd.resource",
        )
        .fetch_all(&state.pool).await?
    } else if role_ids.is_empty() {
        vec![]
    } else {
        let ids: Vec<Uuid> = role_ids.iter().map(|r| r.0).collect();
        let placeholders: Vec<String> = ids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("${}", i + 1))
            .collect();
        let sql = format!(
            "SELECT pd.id, pd.module, pd.resource,
                    bool_or(rp.can_create OR rp.can_update) as can_write
             FROM role_permissions rp
             JOIN permission_definitions pd ON pd.id = rp.permission_id
             WHERE rp.role_id IN ({})
             GROUP BY pd.id, pd.module, pd.resource
             ORDER BY pd.module, pd.resource",
            placeholders.join(",")
        );
        let mut query = sqlx::query_as::<_, PermEntry>(&sql);
        for rid in &ids {
            query = query.bind(rid);
        }
        query.fetch_all(&state.pool).await?
    };

    let mut modules: Vec<Value> = Vec::new();
    let mut i = 0;
    while i < perms.len() {
        let module_name = perms[i].module.clone();
        let has_write = perms[i..]
            .iter()
            .take_while(|p| p.module == module_name)
            .any(|p| p.can_write);
        modules.push(json!({ "module": module_name, "write": has_write }));
        i += perms[i..]
            .iter()
            .take_while(|p| p.module == module_name)
            .count();
    }

    Ok(Json(json!({
        "permissions": perms,
        "modules": modules,
        "can_assign_roles": is_full_access || perms.iter().any(|p| p.module == "Users" && p.can_write),
    })))
}

async fn register(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::user::RegisterPayload>,
) -> AuthResult<Json<Value>> {
    let is_root = claims.role == "GerenteGeneral";
    if !is_root {
        require_any_role(&claims, &["Administrador", "Sostenedor"])?;
        if payload.role == "Administrador" || payload.role == "Sostenedor" {
            require_role(&claims, "Sostenedor")?;
        }
    }

    if payload.rut.trim().is_empty() || payload.name.trim().is_empty() {
        return Err(AuthError::Internal("RUT y nombre son obligatorios".into()));
    }

    let valid_roles: &[&str] = if is_root {
        &["Sostenedor", "Administrador", "Director", "UTP", "Profesor", "Apoderado", "Alumno"]
    } else {
        &["Director", "UTP", "Administrador", "Profesor", "Apoderado", "Alumno"]
    };
    if !valid_roles.contains(&payload.role.as_str()) {
        return Err(AuthError::Internal(format!("Rol inválido: {}", payload.role)));
    }

    let corporation_id = payload
        .corporation_id
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok());
    let school_id = payload
        .school_id
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok());

    if is_root && payload.role == "Sostenedor" && corporation_id.is_none() {
        return Err(AuthError::Internal("Debe seleccionar una corporación para el rol Sostenedor".into()));
    }

    let admin_type = payload.admin_type.as_deref();
    let user = models::insert_user(
        &state.pool,
        &payload.rut,
        &payload.name,
        &payload.email,
        &payload.password,
        &payload.role,
        corporation_id,
        school_id,
        admin_type,
    )
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint() == Some("users_email_key") {
                return AuthError::Internal("El email ya está registrado".into());
            }
            if db_err.constraint() == Some("users_rut_key") {
                return AuthError::Internal("El RUT ya está registrado".into());
            }
        }
        AuthError::Database(e)
    })?;

    Ok(Json(json!({
        "user": {
            "id": user.id,
            "name": user.name,
            "email": user.email,
            "role": user.role,
            "rut": user.rut,
            "corporation_id": user.corporation_id,
            "school_id": user.school_id
        }
    })))
}

async fn onboarding(
    State(state): State<AppState>,
    parts: axum::http::request::Parts,
    Json(payload): Json<schoolccb_common::school::OnboardingPayload>,
) -> AuthResult<Json<Value>> {
    // 1. Validation of internal secret
    let secret = parts.headers.get("X-Internal-Secret")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AuthError::Unauthorized)?;

    if secret != state.config.internal_api_secret {
        return Err(AuthError::Unauthorized);
    }
    let mut tx = state.pool.begin().await?;

    // 1. Create Corporation
    let corp_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO corporations (id, name, rut, active) VALUES ($1, $2, $3, true)",
    )
    .bind(corp_id)
    .bind(&payload.corporation_name)
    .bind(&payload.corporation_rut)
    .execute(&mut *tx)
    .await?;

    // 2. Create Default School
    let school_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO schools (id, corporation_id, name, active) VALUES ($1, $2, $3, true)",
    )
    .bind(school_id)
    .bind(corp_id)
    .bind(&payload.school_name)
    .execute(&mut *tx)
    .await?;

    // 4. Create Admin User (Sostenedor)
    let temp_password = format!("C-{}", &Uuid::new_v4().to_string()[..8]);
    let hash = models::hash_password(&temp_password);
    let user_id = Uuid::new_v4();
    
    sqlx::query(
        "INSERT INTO users (id, rut, name, email, password_hash, role, active, corporation_id, school_id, admin_type)
         VALUES ($1, $2, $3, $4, $5, 'Sostenedor', true, $6, $7, 'global')",
    )
    .bind(user_id)
    .bind(&payload.admin_rut)
    .bind(&payload.admin_name)
    .bind(&payload.admin_email)
    .bind(&hash)
    .bind(corp_id)
    .bind(school_id)
    .execute(&mut *tx)
    .await?;

    // 5. Activate Modules (if provided)
    if let Some(modules) = payload.modules {
        for module_id in modules {
            sqlx::query(
                "INSERT INTO corporation_modules (corporation_id, module_id, active)
                 VALUES ($1, $2, true)
                 ON CONFLICT (corporation_id, module_id) DO UPDATE SET active = true",
            )
            .bind(corp_id)
            .bind(module_id)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;

    Ok(Json(json!({
        "message": "Onboarding completado",
        "corporation_id": corp_id,
        "school_id": school_id,
        "user_id": user_id,
        "admin_email": payload.admin_email,
        "temp_password": temp_password
    })))
}

async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::user::RefreshPayload>,
) -> AuthResult<Json<Value>> {
    let stored = models::find_refresh_token(&state.pool, &payload.refresh_token)
        .await?
        .ok_or(AuthError::TokenInvalid(
            "Refresh token inválido o expirado".into(),
        ))?;

    models::revoke_refresh_token(&state.pool, stored.id).await?;

    let user = models::find_by_id(&state.pool, stored.user_id)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    if !user.active {
        return Err(AuthError::Unauthorized);
    }

    let (token, _claims) = generate_token_pair(
        &state.config,
        user.id,
        &user.role,
        &user.name,
        &user.email,
        user.corporation_id,
        user.school_id,
        user.admin_type.clone(),
    )?;

    let (new_refresh_token, _) = models::create_refresh_token(&state.pool, user.id, 7).await?;
    let is_root = user.role == "GerenteGeneral";

    Ok(Json(json!({
        "token": token,
        "refresh_token": new_refresh_token,
        "user": {
            "id": user.id,
            "name": user.name,
            "email": user.email,
            "role": user.role,
            "rut": user.rut,
            "corporation_id": user.corporation_id,
            "school_id": user.school_id,
            "admin_type": user.admin_type,
            "is_root": is_root
        }
    })))
}

async fn revoke_all(claims: Claims, State(state): State<AppState>) -> AuthResult<Json<Value>> {
    require_role(&claims, "Administrador")?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError::TokenInvalid("Invalid user ID".into()))?;
    models::revoke_all_user_tokens(&state.pool, user_id).await?;
    Ok(Json(json!({ "message": "All sessions revoked" })))
}

async fn logout(claims: Claims, State(state): State<AppState>) -> AuthResult<(HeaderMap, Json<Value>)> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError::TokenInvalid("Invalid user ID".into()))?;
    models::revoke_all_user_tokens(&state.pool, user_id).await?;

    let mut headers = HeaderMap::new();
    if let Ok(cookie) = HeaderValue::from_str("jwt_token=; HttpOnly; Path=/; Max-Age=0") {
        headers.insert(header::SET_COOKIE, cookie);
    }
    if let Ok(marker) = HeaderValue::from_str("session_active=; Path=/; Max-Age=0") {
        headers.append(header::SET_COOKIE, marker);
    }

    Ok((headers, Json(json!({ "message": "Sesión cerrada correctamente" }))))
}

#[derive(Serialize, sqlx::FromRow)]
struct UserListItem {
    id: Uuid,
    rut: String,
    name: String,
    email: String,
    role: String,
    active: bool,
    corporation_id: Option<Uuid>,
    school_id: Option<Uuid>,
    admin_type: Option<String>,
}

#[derive(Deserialize)]
struct UserQuery {
    search: Option<String>,
}

async fn list_users(
    claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<UserQuery>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;
    let search = q.search.as_deref().unwrap_or("");
    let users = if search.is_empty() {
        sqlx::query_as::<_, UserListItem>(
            "SELECT id, rut, name, email, role, active, corporation_id, school_id, admin_type FROM users ORDER BY name",
        )
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, UserListItem>(
            "SELECT id, rut, name, email, role, active, corporation_id, school_id, admin_type FROM users
             WHERE name ILIKE $1 OR email ILIKE $1 ORDER BY name",
        )
        .bind(format!("%{}%", search))
        .fetch_all(&state.pool)
        .await?
    };
    Ok(Json(json!({ "users": users })))
}

async fn update_role(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Value>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;
    let new_role = payload
        .get("role")
        .and_then(|v| v.as_str())
        .ok_or(AuthError::Internal("role es requerido".into()))?;
    if ![
        "Sostenedor",
        "Director",
        "UTP",
        "Administrador",
        "Profesor",
        "Apoderado",
        "Alumno",
    ]
    .contains(&new_role)
    {
        return Err(AuthError::Internal("Rol inválido".into()));
    }
    let user = sqlx::query_as::<_, UserListItem>(
        "UPDATE users SET role = $1 WHERE id = $2 RETURNING id, rut, name, email, role, active, corporation_id, school_id",
    )
    .bind(new_role)
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AuthError::UserNotFound)?;
    Ok(Json(json!({ "user": user })))
}

async fn toggle_active(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;
    let user = sqlx::query_as::<_, UserListItem>(
        "UPDATE users SET active = NOT active WHERE id = $1 RETURNING id, rut, name, email, role, active, corporation_id, school_id",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AuthError::UserNotFound)?;
    Ok(Json(json!({ "user": user })))
}

async fn update_profile(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> AuthResult<Json<Value>> {
    let id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError::TokenInvalid("Invalid user ID".into()))?;
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let email = payload.get("email").and_then(|v| v.as_str()).unwrap_or("");
    if name.trim().is_empty() || email.trim().is_empty() {
        return Err(AuthError::Internal(
            "Nombre y email son obligatorios".into(),
        ));
    }
    let user = models::update_user_profile(&state.pool, id, name, email)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.constraint() == Some("users_email_key") {
                    return AuthError::Internal("El email ya está registrado".into());
                }
            }
            AuthError::Database(e)
        })?;
    Ok(Json(json!({
        "user": {
            "id": user.id,
            "name": user.name,
            "email": user.email,
            "role": user.role,
            "rut": user.rut,
            "corporation_id": user.corporation_id,
            "school_id": user.school_id
        }
    })))
}

async fn change_password(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> AuthResult<Json<Value>> {
    let id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError::TokenInvalid("Invalid user ID".into()))?;
    let current_password = payload
        .get("current_password")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let new_password = payload
        .get("new_password")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if new_password.len() < 6 {
        return Err(AuthError::Internal(
            "La nueva contraseña debe tener al menos 6 caracteres".into(),
        ));
    }
    let user = models::find_by_id(&state.pool, id)
        .await?
        .ok_or(AuthError::UserNotFound)?;
    if !models::verify_password(current_password, &user.password_hash) {
        return Err(AuthError::Internal(
            "La contraseña actual no es correcta".into(),
        ));
    }
    models::change_password(&state.pool, id, new_password).await?;
    Ok(Json(
        json!({ "message": "Contraseña actualizada correctamente" }),
    ))
}

async fn get_user_preferences(
    claims: Claims,
    State(state): State<AppState>,
) -> AuthResult<Json<Value>> {
    let id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError::TokenInvalid("Invalid user ID".into()))?;
    let prefs = models::get_preferences(&state.pool, id).await?;
    Ok(Json(json!({
        "show_module_manager": prefs.show_module_manager
    })))
}

async fn update_user_preferences(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> AuthResult<Json<Value>> {
    let id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError::TokenInvalid("Invalid user ID".into()))?;
    let show = payload
        .get("show_module_manager")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let prefs = models::update_preferences(&state.pool, id, show).await?;
    Ok(Json(json!({
        "show_module_manager": prefs.show_module_manager
    })))
}

async fn get_branding(claims: Claims, State(state): State<AppState>) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador"])?;
    let corp_id = claims.corporation_id.and_then(|s| s.parse::<Uuid>().ok());
    let config = models::get_branding(&state.pool, corp_id).await?;
    if let Some(c) = config {
        Ok(Json(json!({
            "school_name": c.school_name,
            "school_logo_url": c.school_logo_url,
            "primary_color": c.primary_color,
            "secondary_color": c.secondary_color
        })))
    } else {
        Ok(Json(json!({
            "school_name": "",
            "school_logo_url": "",
            "primary_color": "#1A2B3C",
            "secondary_color": "#243B4F"
        })))
    }
}

async fn update_branding(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> AuthResult<Json<Value>> {
    if claims.role != "GerenteGeneral" {
        require_role(&claims, "Sostenedor")?;
    }
    let corp_id = claims.corporation_id.and_then(|s| s.parse::<Uuid>().ok());
    let school_name = payload
        .get("school_name")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let school_logo_url = payload
        .get("school_logo_url")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let primary_color = payload
        .get("primary_color")
        .and_then(|v| v.as_str())
        .unwrap_or("#1A2B3C");
    let secondary_color = payload
        .get("secondary_color")
        .and_then(|v| v.as_str())
        .unwrap_or("#243B4F");

    let is_global = claims.admin_type.as_deref() == Some("global") || claims.role == "Sostenedor";
    if is_global {
        tracing::info!(
            "Global branding updated for corporation {:?} by user {} (role={}, admin_type={:?})",
            corp_id, claims.sub, claims.role, claims.admin_type
        );
    }

    let config = models::upsert_branding(
        &state.pool, corp_id,
        school_name,
        school_logo_url,
        primary_color,
        secondary_color,
    )
    .await?;
    Ok(Json(json!({
        "school_name": config.school_name,
        "school_logo_url": config.school_logo_url,
        "primary_color": config.primary_color,
        "secondary_color": config.secondary_color
    })))
}

fn builtin_modules() -> Vec<schoolccb_common::modules::Module> {
    vec![
        schoolccb_common::modules::Module {
            id: "students".into(),
            name: "Gestión de Alumnos".into(),
            icon: "students".into(),
            category: "Académico".into(),
            route: "/students".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "attendance".into(),
            name: "Asistencia".into(),
            icon: "attendance".into(),
            category: "Académico".into(),
            route: "/attendance".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "grades".into(),
            name: "Calificaciones".into(),
            icon: "grades".into(),
            category: "Académico".into(),
            route: "/grades".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "agenda".into(),
            name: "Agenda Escolar".into(),
            icon: "agenda".into(),
            category: "Comunicaciones".into(),
            route: "/agenda".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "notifications".into(),
            name: "Centro de Mensajería".into(),
            icon: "notifications".into(),
            category: "Comunicaciones".into(),
            route: "/notifications".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "reports".into(),
            name: "Reportes".into(),
            icon: "reports".into(),
            category: "Administración".into(),
            route: "/reports".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "finance".into(),
            name: "Finanzas".into(),
            icon: "config".into(),
            category: "Administración".into(),
            route: "/finance".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "users".into(),
            name: "Usuarios y Perfiles".into(),
            icon: "users".into(),
            category: "Sistema".into(),
            route: "/users".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "courses".into(),
            name: "Cursos".into(),
            icon: "book".into(),
            category: "Académico".into(),
            route: "/courses".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "enrollments".into(),
            name: "Matrículas".into(),
            icon: "clipboard".into(),
            category: "Académico".into(),
            route: "/enrollments".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "subjects".into(),
            name: "Asignaturas".into(),
            icon: "book".into(),
            category: "Académico".into(),
            route: "/subjects".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "academic-years".into(),
            name: "Años Académicos".into(),
            icon: "calendar".into(),
            category: "Administración".into(),
            route: "/academic-years".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "admission".into(),
            name: "Admisiones".into(),
            icon: "users".into(),
            category: "Administración".into(),
            route: "/admission".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "grade-levels".into(),
            name: "Niveles".into(),
            icon: "book".into(),
            category: "Académico".into(),
            route: "/grade-levels".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "classrooms".into(),
            name: "Salas".into(),
            icon: "home".into(),
            category: "Administración".into(),
            route: "/classrooms".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "audit".into(),
            name: "Auditoría".into(),
            icon: "file-text".into(),
            category: "Sistema".into(),
            route: "/audit".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "roles".into(),
            name: "Roles y Permisos".into(),
            icon: "users".into(),
            category: "Sistema".into(),
            route: "/roles".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "corporations".into(),
            name: "Corporaciones y Colegios".into(),
            icon: "home".into(),
            category: "Sistema".into(),
            route: "/corporations".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "hr".into(),
            name: "Recursos Humanos".into(),
            icon: "users".into(),
            category: "Administración".into(),
            route: "/hr".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "payroll".into(),
            name: "Remuneraciones".into(),
            icon: "dollar".into(),
            category: "Administración".into(),
            route: "/payroll".into(),
            parent: Some("hr".into()),
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "my-portal".into(),
            name: "Mi Portal (Auto-consulta)".into(),
            icon: "user".into(),
            category: "Administración".into(),
            route: "/my-portal".into(),
            parent: Some("hr".into()),
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "sige".into(),
            name: "SIGE — Exportación MINEDUC".into(),
            icon: "file-text".into(),
            category: "Administración".into(),
            route: "/sige".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "complaints".into(),
            name: "Ley Karin — Denuncias".into(),
            icon: "shield".into(),
            category: "Administración".into(),
            route: "/complaints".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "academic-calendar".into(),
            name: "Calendario Académico".into(),
            icon: "calendar".into(),
            category: "Académico".into(),
            route: "/academic-calendar".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "teacher-schedules".into(),
            name: "Horarios Docentes".into(),
            icon: "clock".into(),
            category: "RRHH".into(),
            route: "/teacher-schedules".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "parent-portal".into(),
            name: "Portal Apoderados".into(),
            icon: "users".into(),
            category: "Portales".into(),
            route: "/parent-portal".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "student-portal".into(),
            name: "Portal Alumnos".into(),
            icon: "users".into(),
            category: "Portales".into(),
            route: "/student-portal".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "parent-meetings".into(),
            name: "Reuniones Apoderados".into(),
            icon: "calendar".into(),
            category: "Comunicaciones".into(),
            route: "/parent-meetings".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "complementary-subjects".into(),
            name: "Asignaturas Complementarias".into(),
            icon: "book".into(),
            category: "Académico".into(),
            route: "/complementary-subjects".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "config".into(),
            name: "Configuración General".into(),
            icon: "settings".into(),
            category: "Sistema".into(),
            route: "/config".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "sostenedor".into(),
            name: "Panel Sostenedor".into(),
            icon: "home".into(),
            category: "Corporación".into(),
            route: "/sostenedor".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "curriculum".into(),
            name: "Currículum Nacional".into(),
            icon: "book".into(),
            category: "Académico".into(),
            route: "/curriculum".into(),
            parent: None,
            is_favorite: false,
        },
        schoolccb_common::modules::Module {
            id: "license-portal".into(),
            name: "Portal de Licencias".into(),
            icon: "key".into(),
            category: "Sistema".into(),
            route: "/license-portal".into(),
            parent: None,
            is_favorite: false,
        },
    ]
}

#[derive(Deserialize)]
struct ModulesQuery {
    filter_by_license: Option<bool>,
}

async fn list_modules(
    claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<ModulesQuery>,
) -> AuthResult<Json<Value>> {
    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AuthError::TokenInvalid("Invalid user".into()))?;
    let favs: Vec<(String,)> =
        sqlx::query_as("SELECT module_id FROM user_favorites WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&state.pool)
            .await?;
    let fav_set: std::collections::HashSet<String> = favs.into_iter().map(|r| r.0).collect();

    let all_modules = if claims.role == "GerenteGeneral" {
        vec![
            schoolccb_common::modules::Module {
                id: "sales".into(),
                name: "CRM de Ventas".into(),
                icon: "trending-up".into(),
                category: "Gestión".into(),
                route: "/sales".into(),
                parent: None,
                is_favorite: false,
            },
            schoolccb_common::modules::Module {
                id: "corporations".into(),
                name: "Corporaciones".into(),
                icon: "home".into(),
                category: "Configuración".into(),
                route: "/corporations".into(),
                parent: None,
                is_favorite: false,
            },
        ]
    } else if claims.role == "AdminGlobal" {
        // AdminGlobal: solo configuración de colegios/corporaciones, sin datos académicos
        vec![
            schoolccb_common::modules::Module {
                id: "corporations".into(),
                name: "Corporaciones y Colegios".into(),
                icon: "home".into(),
                category: "Configuración".into(),
                route: "/corporations".into(),
                parent: None, is_favorite: false,
            },
            schoolccb_common::modules::Module {
                id: "users".into(),
                name: "Usuarios y Perfiles".into(),
                icon: "users".into(),
                category: "Configuración".into(),
                route: "/users".into(),
                parent: None, is_favorite: false,
            },
            schoolccb_common::modules::Module {
                id: "roles".into(),
                name: "Roles y Permisos".into(),
                icon: "users".into(),
                category: "Configuración".into(),
                route: "/roles".into(),
                parent: None, is_favorite: false,
            },
            schoolccb_common::modules::Module {
                id: "config".into(),
                name: "Configuración General".into(),
                icon: "settings".into(),
                category: "Configuración".into(),
                route: "/config".into(),
                parent: None, is_favorite: false,
            },
            schoolccb_common::modules::Module {
                id: "audit".into(),
                name: "Auditoría".into(),
                icon: "file-text".into(),
                category: "Sistema".into(),
                route: "/audit".into(),
                parent: None, is_favorite: false,
            },
            schoolccb_common::modules::Module {
                id: "license-portal".into(),
                name: "Portal de Licencias".into(),
                icon: "key".into(),
                category: "Sistema".into(),
                route: "/license-portal".into(),
                parent: None, is_favorite: false,
            },
        ]
    } else {
        let filter = q.filter_by_license.unwrap_or(true);
        let bm = builtin_modules();
        if filter {
            let corp_id = claims.corporation_id.as_ref()
                .and_then(|s| Uuid::parse_str(s).ok());
            let allowed: std::collections::HashSet<String> = if let Some(cid) = corp_id {
                let keys: Vec<(String,)> = sqlx::query_as(
                    "SELECT DISTINCT pm.module_key
                     FROM corporation_licenses cl
                     JOIN plan_modules pm ON pm.plan_id = cl.plan_id
                     WHERE cl.corporation_id = $1 AND cl.status = 'active'
                       AND pm.included = true"
                )
                .bind(cid)
                .fetch_all(&state.pool)
                .await
                .unwrap_or_default();
                keys.into_iter().map(|(k,)| k).collect()
            } else {
                bm.iter().map(|m| m.id.clone()).collect()
            };
            bm.into_iter().filter(|m| allowed.contains(&m.id)).collect()
        } else {
            bm
        }
    };
    let modules: Vec<schoolccb_common::modules::Module> = all_modules
        .into_iter()
        .map(|m| schoolccb_common::modules::Module {
            is_favorite: fav_set.contains(&m.id),
            ..m
        })
        .collect();
    Ok(Json(json!({ "modules": modules })))
}

async fn my_plan(claims: Claims, State(state): State<AppState>) -> AuthResult<Json<Value>> {
    let corp_id = claims.corporation_id.as_ref()
        .and_then(|s| Uuid::parse_str(s).ok());
    match corp_id {
        Some(cid) => {
            let plan: Option<(Uuid, String, String, Option<chrono::NaiveDate>, String)> = sqlx::query_as(
                "SELECT lp.id, lp.name, cl.status, cl.end_date, cl.notes
                 FROM corporation_licenses cl
                 JOIN license_plans lp ON lp.id = cl.plan_id
                 WHERE cl.corporation_id = $1 AND cl.status = 'active'
                 ORDER BY cl.created_at DESC LIMIT 1"
            )
            .bind(cid)
            .fetch_optional(&state.pool)
            .await?;

            match plan {
                Some((pid, pname, status, end_date, notes)) => {
                    let modules: Vec<Value> = sqlx::query_as::<_, (String, String, bool)>(
                        "SELECT module_key, module_name, included FROM plan_modules WHERE plan_id = $1 ORDER BY module_key"
                    )
                    .bind(pid)
                    .fetch_all(&state.pool)
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(k, n, inc)| json!({"key": k, "name": n, "included": inc}))
                    .collect();

                    Ok(Json(json!({
                        "plan": {"id": pid, "name": pname, "status": status, "end_date": end_date, "notes": notes},
                        "modules": modules,
                    })))
                }
                None => Ok(Json(json!({"plan": null, "modules": []}))),
            }
        }
        None => Ok(Json(json!({"plan": null, "modules": []}))),
    }
}

async fn toggle_favorite(
    claims: Claims,
    State(state): State<AppState>,
    Path(module_id): Path<String>,
    Json(payload): Json<schoolccb_common::modules::FavoriteToggle>,
) -> AuthResult<Json<Value>> {
    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AuthError::TokenInvalid("Invalid user".into()))?;
    if payload.favorite {
        sqlx::query("INSERT INTO user_favorites (user_id, module_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(user_id).bind(&module_id).execute(&state.pool).await?;
    } else {
        sqlx::query("DELETE FROM user_favorites WHERE user_id = $1 AND module_id = $2")
            .bind(user_id)
            .bind(&module_id)
            .execute(&state.pool)
            .await?;
    }
    Ok(Json(
        json!({ "module_id": module_id, "favorite": payload.favorite }),
    ))
}

// ─── Corporations & Schools ───

#[derive(Deserialize)]
struct SchoolQuery {
    corporation_id: Option<Uuid>,
}

async fn list_corporations(
    claims: Claims,
    State(state): State<AppState>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador"])?;

    let corporations = if claims.role == "GerenteGeneral" {
        sqlx::query_as::<_, schoolccb_common::school::Corporation>(
            "SELECT id, name, rut, logo_url, legal_representative_name, legal_representative_rut, legal_representative_email, settings, active, created_at FROM corporations ORDER BY name",
        )
        .fetch_all(&state.pool)
        .await?
    } else if let Some(cid) = &claims.corporation_id {
        let cid: Uuid = cid.parse().map_err(|_| AuthError::Internal("ID de corporación inválido".into()))?;
        sqlx::query_as::<_, schoolccb_common::school::Corporation>(
            "SELECT id, name, rut, logo_url, legal_representative_name, legal_representative_rut, legal_representative_email, settings, active, created_at FROM corporations WHERE id = $1 ORDER BY name",
        )
        .bind(cid)
        .fetch_all(&state.pool)
        .await?
    } else {
        return Err(AuthError::Forbidden("Sin corporación asignada".into()));
    };

    Ok(Json(json!({ "corporations": corporations })))
}

async fn get_corporation(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador"])?;

    if claims.role != "GerenteGeneral" {
        if let Some(cid) = &claims.corporation_id {
            let user_cid: Uuid = cid.parse().map_err(|_| AuthError::Internal("ID inválido".into()))?;
            if user_cid != id {
                return Err(AuthError::Forbidden("No tienes acceso a esta corporación".into()));
            }
        }
    }

    let corp = sqlx::query_as::<_, schoolccb_common::school::Corporation>(
        "SELECT id, name, rut, logo_url, legal_representative_name, legal_representative_rut, legal_representative_email, settings, active, created_at FROM corporations WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AuthError::NotFound("Corporación no encontrada".into()))?;

    Ok(Json(json!({ "corporation": corp })))
}

async fn get_corporation_modules(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador"])?;

    if claims.role != "GerenteGeneral" {
        if let Some(cid) = &claims.corporation_id {
            let user_cid: Uuid = cid.parse().map_err(|_| AuthError::Internal("ID inválido".into()))?;
            if user_cid != id {
                return Err(AuthError::Forbidden("No tienes acceso a esta corporación".into()));
            }
        }
    }

    let plan_modules = sqlx::query_as::<_, (String, String, bool, serde_json::Value)>(
        "SELECT pm.module_key, pm.module_name, pm.included, pm.sub_modules
         FROM corporation_licenses cl
         JOIN license_plans lp ON lp.id = cl.plan_id
         JOIN plan_modules pm ON pm.plan_id = lp.id
         WHERE cl.corporation_id = $1 AND cl.status = 'active'
         ORDER BY pm.module_key",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    let overrides: std::collections::HashMap<String, bool> = sqlx::query_as::<_, (String, bool)>(
        "SELECT module_key, enabled FROM corporation_module_overrides WHERE corporation_id = $1",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await.unwrap_or_default()
    .into_iter()
    .collect();

    let modules: Vec<Value> = plan_modules
        .into_iter()
        .map(|(key, name, included, sub)| {
            let enabled = overrides.get(&key).copied().unwrap_or(included);
            json!({
                "module_key": key,
                "module_name": name,
                "included": included,
                "enabled": enabled,
                "sub_modules": sub,
            })
        })
        .collect();

    Ok(Json(json!({ "modules": modules })))
}

async fn create_corporation(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::school::CreateCorporationPayload>,
) -> AuthResult<Json<Value>> {
    require_role(&claims, "Sostenedor")?;

    let id = Uuid::new_v4();
    let corp = sqlx::query_as::<_, schoolccb_common::school::Corporation>(
        "INSERT INTO corporations (id, name, rut, logo_url, legal_representative_name, legal_representative_rut, legal_representative_email)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id, name, rut, logo_url, legal_representative_name, legal_representative_rut, legal_representative_email, settings, active, created_at",
    )
    .bind(id)
    .bind(&payload.name)
    .bind(&payload.rut)
    .bind(&payload.logo_url)
    .bind(&payload.legal_representative_name)
    .bind(&payload.legal_representative_rut)
    .bind(&payload.legal_representative_email)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "corporation": corp })))
}

async fn list_schools(
    claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<SchoolQuery>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador"])?;

    let schools = if let Some(corp_id) = q.corporation_id {
        sqlx::query_as::<_, schoolccb_common::school::School>(
            "SELECT id, corporation_id, name, address, phone, logo_url, active, created_at FROM schools WHERE corporation_id = $1 ORDER BY name",
        )
        .bind(corp_id)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, schoolccb_common::school::School>(
            "SELECT id, corporation_id, name, address, phone, logo_url, active, created_at FROM schools ORDER BY name",
        )
        .fetch_all(&state.pool)
        .await?
    };

    Ok(Json(json!({ "schools": schools })))
}

async fn create_school(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::school::CreateSchoolPayload>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador"])?;

    let id = Uuid::new_v4();
    let school = sqlx::query_as::<_, schoolccb_common::school::School>(
        "INSERT INTO schools (id, corporation_id, name, address, phone) VALUES ($1, $2, $3, $4, $5)
         RETURNING id, corporation_id, name, address, phone, logo_url, active, created_at",
    )
    .bind(id)
    .bind(payload.corporation_id)
    .bind(&payload.name)
    .bind(&payload.address)
    .bind(&payload.phone)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "school": school })))
}

async fn get_school(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador"])?;

    let school = sqlx::query_as::<_, schoolccb_common::school::School>(
        "SELECT id, corporation_id, name, address, phone, logo_url, active, created_at FROM schools WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AuthError::NotFound("Colegio no encontrado".into()))?;

    Ok(Json(json!({ "school": school })))
}

async fn update_school(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::school::UpdateSchoolPayload>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador"])?;

    sqlx::query(
        "UPDATE schools SET
            name = COALESCE($1, name),
            address = COALESCE($2, address),
            phone = COALESCE($3, phone),
            logo_url = COALESCE($4, logo_url)
         WHERE id = $5",
    )
    .bind(&payload.name)
    .bind(&payload.address)
    .bind(&payload.phone)
    .bind(&payload.logo_url)
    .bind(id)
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"message": "Colegio actualizado"})))
}

async fn toggle_school(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador"])?;

    let school = sqlx::query_as::<_, schoolccb_common::school::School>(
        "UPDATE schools SET active = NOT active WHERE id = $1 RETURNING id, corporation_id, name, address, phone, logo_url, active, created_at",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AuthError::UserNotFound)?; // not a user but same error pattern

    Ok(Json(json!({ "school": school })))
}

// ─── Legal Representatives ───

async fn list_legal_reps(
    claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<LegalRepQuery>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador"])?;

    let corp_id = q.corporation_id.or_else(|| claims.corporation_id.as_ref().and_then(|s| s.parse::<Uuid>().ok()));
    let school_id = q.school_id.or_else(|| claims.school_id.as_ref().and_then(|s| s.parse::<Uuid>().ok()));

    let reps = if let Some(sid) = school_id {
        sqlx::query_as::<_, schoolccb_common::school::LegalRepresentative>(
            "SELECT id, corporation_id, school_id, rut, first_name, last_name, email, phone, address, active, created_at, updated_at
             FROM legal_representatives WHERE school_id = $1 ORDER BY last_name, first_name",
        )
        .bind(sid)
        .fetch_all(&state.pool).await.unwrap_or_default()
    } else if let Some(cid) = corp_id {
        sqlx::query_as::<_, schoolccb_common::school::LegalRepresentative>(
            "SELECT id, corporation_id, school_id, rut, first_name, last_name, email, phone, address, active, created_at, updated_at
             FROM legal_representatives WHERE corporation_id = $1 ORDER BY last_name, first_name",
        )
        .bind(cid)
        .fetch_all(&state.pool).await.unwrap_or_default()
    } else {
        vec![]
    };

    Ok(Json(json!({"legal_representatives": reps})))
}

async fn create_legal_rep(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::school::CreateLegalRepPayload>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador"])?;

    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO legal_representatives (id, corporation_id, school_id, rut, first_name, last_name, email, phone, address)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(id)
    .bind(payload.corporation_id)
    .bind(payload.school_id)
    .bind(&payload.rut)
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&payload.address)
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"id": id, "message": "Representante legal creado"})))
}

async fn update_legal_rep(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::school::UpdateLegalRepPayload>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador"])?;

    sqlx::query(
        "UPDATE legal_representatives SET
            rut = COALESCE($1, rut),
            first_name = COALESCE($2, first_name),
            last_name = COALESCE($3, last_name),
            email = COALESCE($4, email),
            phone = COALESCE($5, phone),
            address = COALESCE($6, address),
            active = COALESCE($7, active),
            updated_at = NOW()
         WHERE id = $8",
    )
    .bind(&payload.rut)
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&payload.address)
    .bind(payload.active)
    .bind(id)
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"message": "Representante legal actualizado"})))
}

#[derive(Deserialize)]
struct LegalRepQuery {
    corporation_id: Option<Uuid>,
    school_id: Option<Uuid>,
}

// ─── Roles & Permissions ───

async fn list_roles(claims: Claims, State(state): State<AppState>) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    let roles = sqlx::query_as::<_, schoolccb_common::roles::RoleRow>(
        "SELECT id, name, description, is_system, created_at FROM roles ORDER BY name",
    )
    .fetch_all(&state.pool)
    .await?;

    let mut result = Vec::new();
    for role in roles {
        let perms: Vec<schoolccb_common::roles::ResourcePermission> = sqlx::query_as::<_, PermJoin>(
            r#"SELECT pd.id, pd.module, pd.resource, rp.can_create, rp.can_read, rp.can_update, rp.can_delete
               FROM role_permissions rp
               JOIN permission_definitions pd ON pd.id = rp.permission_id
               WHERE rp.role_id = $1
               ORDER BY pd.module, pd.resource"#,
        )
        .bind(role.id)
        .fetch_all(&state.pool).await?
        .into_iter()
        .map(|p| schoolccb_common::roles::ResourcePermission {
            permission_id: p.id,
            module: p.module,
            resource: p.resource,
            can_create: p.can_create,
            can_read: p.can_read,
            can_update: p.can_update,
            can_delete: p.can_delete,
        })
        .collect();

        result.push(serde_json::json!({
            "id": role.id,
            "name": role.name,
            "description": role.description,
            "is_system": role.is_system,
            "permissions": perms,
        }));
    }

    Ok(Json(json!({ "roles": result })))
}

async fn create_role(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::roles::CreateRolePayload>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO roles (id, name, description) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(&payload.name)
        .bind(&payload.description)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({ "id": id })))
}

async fn delete_role(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    let is_system: (bool,) = sqlx::query_as("SELECT is_system FROM roles WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AuthError::NotFound("Rol no encontrado".into()))?;

    if is_system.0 {
        return Err(AuthError::Forbidden(
            "No se puede eliminar un rol del sistema".into(),
        ));
    }

    sqlx::query("DELETE FROM roles WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({ "message": "Rol eliminado" })))
}

async fn update_role_permissions(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::roles::UpdatePermissionsPayload>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    for perm in &payload.permissions {
        let existing: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM role_permissions WHERE role_id = $1 AND permission_id = $2",
        )
        .bind(id)
        .bind(perm.permission_id)
        .fetch_one(&state.pool)
        .await?;

        if existing.0 > 0 {
            sqlx::query(
                "UPDATE role_permissions SET can_create = $1, can_read = $2, can_update = $3, can_delete = $4 WHERE role_id = $5 AND permission_id = $6",
            )
            .bind(perm.can_create).bind(perm.can_read).bind(perm.can_update).bind(perm.can_delete)
            .bind(id).bind(perm.permission_id)
            .execute(&state.pool).await?;
        } else {
            sqlx::query(
                "INSERT INTO role_permissions (id, role_id, permission_id, can_create, can_read, can_update, can_delete) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(Uuid::new_v4()).bind(id).bind(perm.permission_id)
            .bind(perm.can_create).bind(perm.can_read).bind(perm.can_update).bind(perm.can_delete)
            .execute(&state.pool).await?;
        }
    }

    Ok(Json(json!({ "message": "Permisos actualizados" })))
}

async fn list_permission_definitions(
    claims: Claims,
    State(state): State<AppState>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    let defs = sqlx::query_as::<_, schoolccb_common::roles::PermissionDef>(
        "SELECT id, module, resource, label, created_at FROM permission_definitions ORDER BY module, resource",
    )
    .fetch_all(&state.pool).await?;

    let defs_json: Vec<Value> = defs
        .into_iter()
        .map(|d| {
            json!({
                "id": d.id,
                "module": d.module,
                "resource": d.resource,
                "label": d.label,
                "created_at": d.created_at,
            })
        })
        .collect();
    Ok(Json(json!({ "definitions": defs_json })))
}

async fn assign_role_to_user(
    claims: Claims,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::roles::AssignRolePayload>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    sqlx::query(
        "INSERT INTO user_roles (id, user_id, role_id) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(payload.role_id)
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({ "message": "Rol asignado" })))
}

async fn remove_role_from_user(
    claims: Claims,
    State(state): State<AppState>,
    Path((user_id, role_id)): Path<(Uuid, Uuid)>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    sqlx::query("DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2")
        .bind(user_id)
        .bind(role_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({ "message": "Rol removido" })))
}

async fn list_user_roles(
    claims: Claims,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    let assigned: Vec<(Uuid,)> =
        sqlx::query_as("SELECT role_id FROM user_roles WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&state.pool)
            .await?;

    let ids: Vec<String> = assigned.into_iter().map(|r| r.0.to_string()).collect();
    Ok(Json(json!({ "role_ids": ids })))
}

// Re-export for router
pub fn roles_router() -> Router<AppState> {
    Router::new()
        .route("/api/roles", get(list_roles).post(create_role))
        .route("/api/roles/{id}", delete(delete_role))
        .route("/api/roles/{id}/permissions", put(update_role_permissions))
        .route(
            "/api/permissions/definitions",
            get(list_permission_definitions),
        )
        .route(
            "/api/users/{user_id}/roles",
            get(list_user_roles).post(assign_role_to_user),
        )
        .route(
            "/api/users/{user_id}/roles/{role_id}",
            delete(remove_role_from_user),
        )
}

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
struct PermEntry {
    id: Uuid,
    module: String,
    resource: String,
    can_write: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct PermJoin {
    id: Uuid,
    module: String,
    resource: String,
    can_create: bool,
    can_read: bool,
    can_update: bool,
    can_delete: bool,
}
