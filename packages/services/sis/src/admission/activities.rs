use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{SisError, SisResult};
use schoolccb_common::auth::{Claims, require_any_role};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/admission/activities",
            get(list_activities).post(create_activity),
        )
        .route(
            "/api/admission/activities/{id}",
            get(get_activity)
                .put(update_activity)
                .delete(delete_activity),
        )
}

async fn list_activities(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Admision"],
    )?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "admission",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;
    let activities = sqlx::query_as::<_, schoolccb_common::admission::ProspectActivity>(
        "SELECT id, prospect_id, activity_type, subject, description, scheduled_at, is_completed, created_by, created_at FROM prospect_activities ORDER BY created_at DESC LIMIT 200",
    ).fetch_all(&state.pool).await?;
    Ok(Json(json!({ "activities": activities })))
}

async fn get_activity(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Admision"],
    )?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "admission",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;
    let activity = sqlx::query_as::<_, schoolccb_common::admission::ProspectActivity>(
        "SELECT id, prospect_id, activity_type, subject, description, scheduled_at, is_completed, created_by, created_at FROM prospect_activities WHERE id = $1",
    ).bind(id).fetch_optional(&state.pool).await?
        .ok_or(crate::error::SisError::NotFound("Actividad no encontrada".into()))?;
    Ok(Json(json!({ "activity": activity })))
}

async fn create_activity(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::admission::CreateActivityPayload>,
) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Admision"],
    )?;
    let id = Uuid::new_v4();
    let user_id = Uuid::parse_str(&claims.sub).ok();
    let result = sqlx::query_as::<_, schoolccb_common::admission::ProspectActivity>(
        r#"INSERT INTO prospect_activities (id, prospect_id, activity_type, subject, description, scheduled_at, created_by)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id, prospect_id, activity_type, subject, description, scheduled_at, is_completed, created_by, created_at"#,
    ).bind(id).bind(payload.prospect_id).bind(&payload.activity_type).bind(&payload.subject)
    .bind(&payload.description).bind(payload.scheduled_at).bind(user_id)
    .fetch_one(&state.pool).await?;
    Ok(Json(json!({ "activity": result })))
}

async fn update_activity(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Admision"],
    )?;
    let result = sqlx::query_as::<_, schoolccb_common::admission::ProspectActivity>(
        "UPDATE prospect_activities SET is_completed = $1 WHERE id = $2
         RETURNING id, prospect_id, activity_type, subject, description, scheduled_at, is_completed, created_by, created_at",
    ).bind(payload.get("is_completed").and_then(|v| v.as_bool()).unwrap_or(true)).bind(id)
    .fetch_one(&state.pool).await?;
    Ok(Json(json!({ "activity": result })))
}

async fn delete_activity(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;
    sqlx::query("DELETE FROM prospect_activities WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({ "message": "Actividad eliminada" })))
}
