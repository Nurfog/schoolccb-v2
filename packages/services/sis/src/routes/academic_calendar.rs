use axum::{Json, Router, extract::{Path, State}, routing::{get, put, delete}};
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

async fn resolve_subject_id(pool: &sqlx::PgPool, name: &str) -> Option<Uuid> {
    sqlx::query_scalar("SELECT id FROM subjects WHERE name ILIKE $1 LIMIT 1")
        .bind(format!("%{}%", name))
        .fetch_optional(pool).await.ok().flatten()
}

async fn resolve_teacher_id(pool: &sqlx::PgPool, name: &str) -> Option<Uuid> {
    sqlx::query_scalar("SELECT id FROM users WHERE name ILIKE $1 AND role = 'Profesor' LIMIT 1")
        .bind(format!("%{}%", name))
        .fetch_optional(pool).await.ok().flatten()
}

async fn create_exam(claims: Claims, State(state): State<AppState>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    let school_id = claims.school_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let id = Uuid::new_v4();

    let subject_id = if let Some(sid) = p.get("subject_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()) {
        Some(sid)
    } else if let Some(name) = p.get("subject").and_then(|v| v.as_str()) {
        resolve_subject_id(&state.pool, name).await
    } else { None };

    let teacher_id = if let Some(tid) = p.get("responsible_teacher_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()) {
        Some(tid)
    } else if let Some(name) = p.get("responsible").and_then(|v| v.as_str()) {
        resolve_teacher_id(&state.pool, name).await
    } else { None };

    sqlx::query("INSERT INTO exam_schedule (id, school_id, course_id, subject_id, title, exam_date, start_time, end_time, responsible_teacher_id, notes)
                 VALUES ($1, $2, $3, $4, $5, $6::date, $7::time, $8::time, $9, $10)")
        .bind(id).bind(school_id)
        .bind(p.get("course_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()))
        .bind(subject_id)
        .bind(p.get("title").and_then(|v| v.as_str()))
        .bind(p.get("date").and_then(|v| v.as_str()))
        .bind(p.get("start_time").or_else(|| p.get("time")).and_then(|v| v.as_str()))
        .bind(p.get("end_time").and_then(|v| v.as_str()))
        .bind(teacher_id)
        .bind(p.get("notes").and_then(|v| v.as_str()))
        .execute(&state.pool).await?;
    Ok(Json(json!({"id": id})))
}

async fn update_exam(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;

    let subject_id = if let Some(sid) = p.get("subject_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()) {
        Some(sid)
    } else if let Some(name) = p.get("subject").and_then(|v| v.as_str()) {
        resolve_subject_id(&state.pool, name).await
    } else { None };

    let teacher_id = if let Some(tid) = p.get("responsible_teacher_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()) {
        Some(tid)
    } else if let Some(name) = p.get("responsible").and_then(|v| v.as_str()) {
        resolve_teacher_id(&state.pool, name).await
    } else { None };

    sqlx::query("UPDATE exam_schedule SET
        title = COALESCE($1, title), exam_date = COALESCE($2::date, exam_date),
        subject_id = COALESCE($3, subject_id), responsible_teacher_id = COALESCE($4, responsible_teacher_id),
        start_time = COALESCE($5::time, start_time), end_time = COALESCE($6::time, end_time),
        notes = COALESCE($7, notes) WHERE id = $8")
        .bind(p.get("title").and_then(|v| v.as_str()))
        .bind(p.get("date").and_then(|v| v.as_str()))
        .bind(subject_id)
        .bind(teacher_id)
        .bind(p.get("start_time").or_else(|| p.get("time")).and_then(|v| v.as_str()))
        .bind(p.get("end_time").and_then(|v| v.as_str()))
        .bind(p.get("notes").and_then(|v| v.as_str()))
        .bind(id).execute(&state.pool).await?;
    Ok(Json(json!({"message": "Examen actualizado"})))
}

async fn delete_exam(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    sqlx::query("DELETE FROM exam_schedule WHERE id = $1").bind(id).execute(&state.pool).await?;
    Ok(Json(json!({"message": "Examen eliminado"})))
}

/// Calcula la fecha del Domingo de Pascua usando el algoritmo de Anonymous Gregorian.
fn easter_sunday(year: i32) -> (i32, u32, u32) {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = ((h + l - 7 * m + 114) % 31) + 1;
    (year, month as u32, day as u32)
}

/// Siembra automática de feriados chilenos para un año dado.
pub async fn seed_holidays(pool: &sqlx::PgPool, year: i32) {
    let easter = easter_sunday(year);
    let easter_date = chrono::NaiveDate::from_ymd_opt(easter.0, easter.1, easter.2);

    let fixed = vec![
        (1, 1, "Año Nuevo", "legal", true),
        (5, 1, "Día del Trabajo", "legal", true),
        (5, 21, "Día de las Glorias Navales", "legal", true),
        (6, 29, "San Pedro y San Pablo", "legal", true),
        (7, 16, "Virgen del Carmen", "legal", true),
        (8, 15, "Asunción de la Virgen", "legal", true),
        (9, 18, "Independencia Nacional", "legal", true),
        (9, 19, "Día de las Glorias del Ejército", "legal", true),
        (10, 12, "Encuentro de Dos Mundos", "legal", true),
        (10, 31, "Día de las Iglesias Evangélicas", "legal", true),
        (11, 1, "Día de Todos los Santos", "legal", true),
        (12, 8, "Inmaculada Concepción", "legal", true),
        (12, 25, "Navidad", "legal", true),
    ];

    let mut holidays: Vec<(chrono::NaiveDate, &str, &str, bool)> = fixed.into_iter()
        .filter_map(|(m, d, name, htype, recurring)| {
            chrono::NaiveDate::from_ymd_opt(year, m, d)
                .map(|date| (date, name, htype, recurring))
        })
        .collect();

    if let Some(e) = easter_date {
        holidays.push((e - chrono::Duration::days(2), "Viernes Santo", "legal", false));
        holidays.push((e - chrono::Duration::days(1), "Sábado Santo", "legal", false));
        holidays.push((e + chrono::Duration::days(60), "Corpus Christi", "legal", false));
    }

    // Mid-winter school break (3rd week of July)
    if let Some(winter_start) = chrono::NaiveDate::from_ymd_opt(year, 7, 14) {
        holidays.push((winter_start, "Vacaciones de Invierno (inicio)", "school", false));
    }

    for (date, name, htype, recurring) in holidays {
        let exists: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM holidays WHERE date = $1 AND name = $2",
        )
        .bind(date)
        .bind(name)
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

        if exists.0 == 0 {
            let id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO holidays (id, date, name, holiday_type, is_recurring) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(id)
            .bind(date)
            .bind(name)
            .bind(htype)
            .bind(recurring)
            .execute(pool)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("No se pudo sembrar feriado {name}: {e}");
                Default::default()
            });
            tracing::info!("Feriado sembrado: {name} ({date})");
        }
    }
}
