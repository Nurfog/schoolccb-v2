use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{SisError, SisResult};
use schoolccb_common::auth::{Claims, require_any_role};

#[derive(Deserialize)]
struct ReminderQuery {
    prospect_id: Option<Uuid>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/admission/reminders",
            get(list_reminders).post(create_reminder),
        )
        .route(
            "/api/admission/reminders/{id}",
            get(get_reminder)
                .put(update_reminder)
                .delete(delete_reminder),
        )
}

async fn list_reminders(claims: Claims, State(state): State<AppState>, Query(q): Query<ReminderQuery>) -> SisResult<Json<Value>> {
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
    let reminders = if let Some(pid) = q.prospect_id {
        sqlx::query_as::<_, schoolccb_common::admission::ProspectReminder>(
            "SELECT id, prospect_id, reminder_type, title, description, remind_at, is_sent, created_by, created_at FROM prospect_reminders WHERE prospect_id = $1 ORDER BY remind_at ASC",
        ).bind(pid).fetch_all(&state.pool).await?
    } else {
        sqlx::query_as::<_, schoolccb_common::admission::ProspectReminder>(
            "SELECT id, prospect_id, reminder_type, title, description, remind_at, is_sent, created_by, created_at FROM prospect_reminders ORDER BY remind_at ASC LIMIT 200",
        ).fetch_all(&state.pool).await?
    };
    Ok(Json(json!({ "reminders": reminders })))
}

async fn get_reminder(
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
    let reminder = sqlx::query_as::<_, schoolccb_common::admission::ProspectReminder>(
        "SELECT id, prospect_id, reminder_type, title, description, remind_at, is_sent, created_by, created_at FROM prospect_reminders WHERE id = $1",
    ).bind(id).fetch_optional(&state.pool).await?
        .ok_or(SisError::NotFound("Recordatorio no encontrado".into()))?;
    Ok(Json(json!({ "reminder": reminder })))
}

async fn create_reminder(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::admission::CreateReminderPayload>,
) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Admision"],
    )?;
    let id = Uuid::new_v4();
    let user_id = Uuid::parse_str(&claims.sub).ok();
    let result = sqlx::query_as::<_, schoolccb_common::admission::ProspectReminder>(
        r#"INSERT INTO prospect_reminders (id, prospect_id, reminder_type, title, description, remind_at, created_by)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id, prospect_id, reminder_type, title, description, remind_at, is_sent, created_by, created_at"#,
    ).bind(id).bind(payload.prospect_id).bind(&payload.reminder_type).bind(&payload.title)
    .bind(&payload.description).bind(payload.remind_at).bind(user_id)
    .fetch_one(&state.pool).await?;
    Ok(Json(json!({ "reminder": result })))
}

async fn update_reminder(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Admision"],
    )?;
    let result = sqlx::query_as::<_, schoolccb_common::admission::ProspectReminder>(
        "UPDATE prospect_reminders SET is_sent = $1, remind_at = COALESCE($2, remind_at), title = COALESCE($3, title) WHERE id = $4
         RETURNING id, prospect_id, reminder_type, title, description, remind_at, is_sent, created_by, created_at",
    ).bind(payload.get("is_sent").and_then(|v| v.as_bool()).unwrap_or(false))
    .bind(payload.get("remind_at").and_then(|v| v.as_str()).and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&chrono::Utc))))
    .bind(payload.get("title").and_then(|v| v.as_str()))
    .bind(id)
    .fetch_one(&state.pool).await?;
    Ok(Json(json!({ "reminder": result })))
}

async fn delete_reminder(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;
    sqlx::query("DELETE FROM prospect_reminders WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({ "message": "Recordatorio eliminado" })))
}
