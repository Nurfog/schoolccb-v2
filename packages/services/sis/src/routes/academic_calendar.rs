use axum::{Json, Router, extract::{Path, State}, routing::{get, post, put, delete}};
use serde_json::{Value, json};
use uuid::Uuid;
use crate::error::SisResult;
use crate::routes::students::{require_any_role, Claims};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/academic/calendar", get(list_events).post(create_event))
        .route("/api/academic/calendar/{id}", put(update_event).delete(delete_event))
        .route("/api/academic/holidays", get(list_holidays).post(create_holiday))
        .route("/api/academic/holidays/{id}", delete(delete_holiday))
        .route("/api/academic/exams", get(list_exams).post(create_exam))
        .route("/api/academic/exams/{id}", put(update_exam).delete(delete_exam))
}

async fn list_events(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor", "GerenteGeneral"])?;
    let school_id = claims.school_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let events = sqlx::query_as::<_, (Uuid, String, String, String, String, String, Option<String>)>(
        "SELECT id, title, event_type, event_date::text, start_time::text, COALESCE(color, '#3B82F6'), description
         FROM academic_calendar WHERE ($1::uuid IS NULL OR school_id = $1)
         ORDER BY event_date DESC LIMIT 100",
    ).bind(school_id).fetch_all(&state.pool).await?.into_iter()
    .map(|(id, t, ty, d, s, c, desc)| json!({"id": id, "title": t, "type": ty, "date": d, "time": s, "color": c, "description": desc}))
    .collect::<Vec<_>>();
    Ok(Json(json!({"events": events})))
}

async fn create_event(claims: Claims, State(state): State<AppState>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    let school_id = claims.school_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO academic_calendar (id, school_id, title, description, event_type, event_date, start_time, end_time, color, created_by)
                 VALUES ($1, $2, $3, $4, $5, $6::date, $7::time, $8::time, $9, $10)")
        .bind(id).bind(school_id)
        .bind(p.get("title").and_then(|v| v.as_str()))
        .bind(p.get("description").and_then(|v| v.as_str()))
        .bind(p.get("event_type").and_then(|v| v.as_str()).unwrap_or("event"))
        .bind(p.get("date").and_then(|v| v.as_str()))
        .bind(p.get("start_time").and_then(|v| v.as_str()))
        .bind(p.get("end_time").and_then(|v| v.as_str()))
        .bind(p.get("color").and_then(|v| v.as_str()))
        .bind(Uuid::parse_str(&claims.sub).ok())
        .execute(&state.pool).await?;
    Ok(Json(json!({"id": id})))
}

async fn update_event(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    sqlx::query("UPDATE academic_calendar SET title = COALESCE($1, title), description = COALESCE($2, description), event_date = COALESCE($3::date, event_date), start_time = COALESCE($4::time, start_time), color = COALESCE($5, color) WHERE id = $6")
        .bind(p.get("title").and_then(|v| v.as_str()))
        .bind(p.get("description").and_then(|v| v.as_str()))
        .bind(p.get("date").and_then(|v| v.as_str()))
        .bind(p.get("start_time").and_then(|v| v.as_str()))
        .bind(p.get("color").and_then(|v| v.as_str()))
        .bind(id).execute(&state.pool).await?;
    Ok(Json(json!({"message": "Evento actualizado"})))
}

async fn delete_event(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    sqlx::query("DELETE FROM academic_calendar WHERE id = $1").bind(id).execute(&state.pool).await?;
    Ok(Json(json!({"message": "Evento eliminado"})))
}

async fn list_holidays(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    let school_id = claims.school_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let holidays = sqlx::query_as::<_, (Uuid, String, String, String)>(
        "SELECT id, date::text, name, holiday_type FROM holidays WHERE ($1::uuid IS NULL OR school_id = $1) ORDER BY date DESC"
    ).bind(school_id).fetch_all(&state.pool).await?.into_iter()
    .map(|(id, d, n, t)| json!({"id": id, "date": d, "name": n, "type": t})).collect::<Vec<_>>();
    Ok(Json(json!({"holidays": holidays})))
}

async fn create_holiday(claims: Claims, State(state): State<AppState>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "GerenteGeneral"])?;
    let id = Uuid::new_v4();
    let school_id = claims.school_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    sqlx::query("INSERT INTO holidays (id, school_id, date, name, holiday_type) VALUES ($1, $2, $3::date, $4, $5)")
        .bind(id).bind(school_id)
        .bind(p.get("date").and_then(|v| v.as_str()))
        .bind(p.get("name").and_then(|v| v.as_str()))
        .bind(p.get("type").and_then(|v| v.as_str()).unwrap_or("legal"))
        .execute(&state.pool).await?;
    Ok(Json(json!({"id": id})))
}

async fn delete_holiday(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "GerenteGeneral"])?;
    sqlx::query("DELETE FROM holidays WHERE id = $1").bind(id).execute(&state.pool).await?;
    Ok(Json(json!({"message": "Feriado eliminado"})))
}

async fn list_exams(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor", "GerenteGeneral"])?;
    let school_id = claims.school_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let exams = sqlx::query_as::<_, (Uuid, String, String, String, String, String)>(
        "SELECT e.id, e.title, COALESCE(sub.name, ''), e.exam_date::text, e.start_time::text, COALESCE(u.name, '')
         FROM exam_schedule e LEFT JOIN subjects sub ON sub.id = e.subject_id LEFT JOIN users u ON u.id = e.responsible_teacher_id
         WHERE ($1::uuid IS NULL OR e.school_id = $1) ORDER BY e.exam_date DESC LIMIT 100"
    ).bind(school_id).fetch_all(&state.pool).await?.into_iter()
    .map(|(id, t, s, d, ti, r)| json!({"id": id, "title": t, "subject": s, "date": d, "time": ti, "responsible": r}))
    .collect::<Vec<_>>();
    Ok(Json(json!({"exams": exams})))
}

async fn create_exam(claims: Claims, State(state): State<AppState>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    let school_id = claims.school_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO exam_schedule (id, school_id, course_id, subject_id, title, exam_date, start_time, end_time, responsible_teacher_id, notes)
                 VALUES ($1, $2, $3, $4, $5, $6::date, $7::time, $8::time, $9, $10)")
        .bind(id).bind(school_id)
        .bind(p.get("course_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()))
        .bind(p.get("subject_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()))
        .bind(p.get("title").and_then(|v| v.as_str()))
        .bind(p.get("date").and_then(|v| v.as_str()))
        .bind(p.get("start_time").and_then(|v| v.as_str()))
        .bind(p.get("end_time").and_then(|v| v.as_str()))
        .bind(p.get("responsible_teacher_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()))
        .bind(p.get("notes").and_then(|v| v.as_str()))
        .execute(&state.pool).await?;
    Ok(Json(json!({"id": id})))
}

async fn update_exam(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    sqlx::query("UPDATE exam_schedule SET title = COALESCE($1, title), exam_date = COALESCE($2::date, exam_date), notes = COALESCE($3, notes) WHERE id = $4")
        .bind(p.get("title").and_then(|v| v.as_str()))
        .bind(p.get("date").and_then(|v| v.as_str()))
        .bind(p.get("notes").and_then(|v| v.as_str()))
        .bind(id).execute(&state.pool).await?;
    Ok(Json(json!({"message": "Examen actualizado"})))
}

async fn delete_exam(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    sqlx::query("DELETE FROM exam_schedule WHERE id = $1").bind(id).execute(&state.pool).await?;
    Ok(Json(json!({"message": "Examen eliminado"})))
}
