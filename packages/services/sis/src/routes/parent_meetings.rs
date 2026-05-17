use axum::{Json, Router, extract::{Path, State}, routing::{get, post, put}};
use serde_json::{Value, json};
use uuid::Uuid;
use crate::error::SisResult;
use schoolccb_common::auth::Claims;
use crate::routes::students::require_any_role;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/meetings", get(list_meetings).post(create_meeting))
        .route("/api/meetings/{id}", put(update_meeting))
        .route("/api/meetings/{id}/cancel", post(cancel_meeting))
        .route("/api/meetings/general", get(list_general).post(create_general))
        .route("/api/meetings/general/{id}", put(update_general))
        .route("/api/meetings/general/{id}/minutes", get(get_minutes).post(save_minutes))
}

async fn list_meetings(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor", "GerenteGeneral"])?;
    let school_id = claims.school_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let meetings = sqlx::query_as::<_, (Uuid, String, String, String, String, String)>(
        "SELECT pm.id, s.first_name || ' ' || s.last_name, pm.scheduled_date::text, pm.start_time::text, pm.status, COALESCE(pm.location, '')
         FROM parent_meetings pm JOIN students s ON s.id = pm.student_id
         WHERE ($1::uuid IS NULL OR pm.school_id = $1) ORDER BY pm.scheduled_date DESC LIMIT 50"
    ).bind(school_id).fetch_all(&state.pool).await?.into_iter()
    .map(|(id, s, d, t, st, l)| json!({"id": id, "student": s, "date": d, "time": t, "status": st, "location": l}))
    .collect::<Vec<_>>();
    Ok(Json(json!({"meetings": meetings})))
}

async fn create_meeting(claims: Claims, State(state): State<AppState>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor", "GerenteGeneral"])?;
    let school_id = claims.school_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO parent_meetings (id, school_id, teacher_id, guardian_user_id, student_id, scheduled_date, start_time, end_time, location)
                 VALUES ($1, $2, $3, $4, $5, $6::date, $7::time, $8::time, $9)")
        .bind(id).bind(school_id)
        .bind(p.get("teacher_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()))
        .bind(p.get("guardian_user_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()))
        .bind(p.get("student_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()))
        .bind(p.get("date").and_then(|v| v.as_str()))
        .bind(p.get("start_time").and_then(|v| v.as_str()))
        .bind(p.get("end_time").and_then(|v| v.as_str()))
        .bind(p.get("location").and_then(|v| v.as_str()))
        .execute(&state.pool).await?;
    Ok(Json(json!({"id": id})))
}

async fn update_meeting(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "UTP", "Profesor", "GerenteGeneral"])?;
    sqlx::query("UPDATE parent_meetings SET scheduled_date = COALESCE($1::date, scheduled_date), start_time = COALESCE($2::time, start_time), location = COALESCE($3, location), notes = COALESCE($4, notes), updated_at = NOW() WHERE id = $5")
        .bind(p.get("date").and_then(|v| v.as_str()))
        .bind(p.get("start_time").and_then(|v| v.as_str()))
        .bind(p.get("location").and_then(|v| v.as_str()))
        .bind(p.get("notes").and_then(|v| v.as_str()))
        .bind(id).execute(&state.pool).await?;
    Ok(Json(json!({"message": "Reunión actualizada"})))
}

async fn cancel_meeting(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "UTP", "Profesor", "GerenteGeneral"])?;
    sqlx::query("UPDATE parent_meetings SET status = 'cancelled', updated_at = NOW() WHERE id = $1").bind(id).execute(&state.pool).await?;
    Ok(Json(json!({"message": "Reunión cancelada"})))
}

async fn list_general(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor", "GerenteGeneral"])?;
    let school_id = claims.school_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let meetings = sqlx::query_as::<_, (Uuid, String, String, String, String, String)>(
        "SELECT id, title, meeting_date::text, start_time::text, COALESCE(location, ''), COALESCE(description, '')
         FROM general_meetings WHERE ($1::uuid IS NULL OR school_id = $1) ORDER BY meeting_date DESC LIMIT 50"
    ).bind(school_id).fetch_all(&state.pool).await?.into_iter()
    .map(|(id, t, d, s, l, desc)| json!({"id": id, "title": t, "date": d, "time": s, "location": l, "description": desc}))
    .collect::<Vec<_>>();
    Ok(Json(json!({"meetings": meetings})))
}

async fn create_general(claims: Claims, State(state): State<AppState>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    let school_id = claims.school_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO general_meetings (id, school_id, title, description, meeting_date, start_time, end_time, location, agenda, created_by)
                 VALUES ($1, $2, $3, $4, $5::date, $6::time, $7::time, $8, $9, $10)")
        .bind(id).bind(school_id)
        .bind(p.get("title").and_then(|v| v.as_str()))
        .bind(p.get("description").and_then(|v| v.as_str()))
        .bind(p.get("date").and_then(|v| v.as_str()))
        .bind(p.get("start_time").and_then(|v| v.as_str()))
        .bind(p.get("end_time").and_then(|v| v.as_str()))
        .bind(p.get("location").and_then(|v| v.as_str()))
        .bind(p.get("agenda"))
        .bind(Uuid::parse_str(&claims.sub).ok())
        .execute(&state.pool).await?;
    Ok(Json(json!({"id": id})))
}

async fn update_general(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    sqlx::query("UPDATE general_meetings SET title = COALESCE($1, title), description = COALESCE($2, description), meeting_date = COALESCE($3::date, meeting_date) WHERE id = $4")
        .bind(p.get("title").and_then(|v| v.as_str()))
        .bind(p.get("description").and_then(|v| v.as_str()))
        .bind(p.get("date").and_then(|v| v.as_str()))
        .bind(id).execute(&state.pool).await?;
    Ok(Json(json!({"message": "Reunión actualizada"})))
}

async fn get_minutes(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    let minutes = sqlx::query_as::<_, (Uuid, String, bool, String)>(
        "SELECT id, content, sent_by_email, created_at::text FROM meeting_minutes WHERE meeting_id = $1 ORDER BY created_at DESC LIMIT 1"
    ).bind(id).fetch_optional(&state.pool).await?;
    match minutes {
        Some((mid, content, sent, date)) => Ok(Json(json!({"id": mid, "content": content, "sent": sent, "date": date}))),
        None => Ok(Json(json!({"content": "", "sent": false}))),
    }
}

async fn save_minutes(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    let mid = Uuid::new_v4();
    sqlx::query("INSERT INTO meeting_minutes (id, meeting_id, content, created_by) VALUES ($1, $2, $3, $4)")
        .bind(mid).bind(id)
        .bind(p.get("content").and_then(|v| v.as_str()).unwrap_or(""))
        .bind(Uuid::parse_str(&claims.sub).ok())
        .execute(&state.pool).await?;
    Ok(Json(json!({"id": mid})))
}
