use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{AttendanceError, AttendanceResult};

pub use schoolccb_common::auth::Claims;
use schoolccb_common::auth::require_any_role;

#[derive(Deserialize)]
pub struct DateRangeFilter {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub student_id: Option<Uuid>,
    pub course_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct RawAttendance {
    pub id: Uuid,
    pub student_id: Uuid,
    pub course_id: Uuid,
    pub date: NaiveDate,
    pub time: Option<chrono::NaiveTime>,
    pub status: String,
    pub subject: String,
    pub teacher_id: Uuid,
    pub observation: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/attendance",
            get(list_attendance).post(create_attendance),
        )
        .route(
            "/api/attendance/{id}",
            get(get_attendance)
                .put(update_attendance)
                .delete(delete_attendance),
        )
        .route("/api/attendance/bulk", post(bulk_create_attendance))
        .route("/api/attendance/today", get(today_attendance))
        .route("/api/attendance/date/{date}", get(attendance_by_date))
        .route(
            "/api/attendance/student/{student_id}",
            get(attendance_by_student),
        )
        .route(
            "/api/attendance/course/{course_id}/date/{date}",
            get(attendance_by_course_date),
        )
}

async fn list_attendance(
    claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<DateRangeFilter>,
) -> AttendanceResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "attendance",
    )
    .await
    .map_err(|e| AttendanceError::Forbidden(e))?;

    let school_id: Option<Uuid> = claims
        .school_id
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok());

    let corporation_id: Option<Uuid> = claims
        .corporation_id
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok());

    let records = if let (Some(sid), Some(cid)) = (filter.student_id, filter.course_id) {
        if let (Some(from), Some(to)) = (filter.from, filter.to) {
            let (sql, corp_val) = if school_id.is_some() {
                let corp_clause = corporation_id.map(|_| " AND sch.corporation_id = $6").unwrap_or("");
                (format!("SELECT a.id, a.student_id, a.course_id, a.date, a.time, a.status, a.subject, a.teacher_id, a.observation FROM attendance a JOIN schools sch ON sch.id = a.school_id WHERE student_id = $1 AND course_id = $2 AND date >= $3 AND date <= $4 AND a.school_id = $5{} ORDER BY a.date DESC", corp_clause), corporation_id)
            } else {
                ("SELECT id, student_id, course_id, date, time, status, subject, teacher_id, observation FROM attendance WHERE student_id = $1 AND course_id = $2 AND date >= $3 AND date <= $4 ORDER BY date DESC".to_string(), None)
            };
            let mut q = sqlx::query_as::<_, RawAttendance>(&sql).bind(sid).bind(cid).bind(from).bind(to);
            if let Some(sc) = school_id { q = q.bind(sc); }
            if let Some(cc) = corp_val { q = q.bind(cc); }
            q.fetch_all(&state.pool).await?
        } else {
            let (sql, corp_val) = if school_id.is_some() {
                let corp_clause = corporation_id.map(|_| " AND sch.corporation_id = $4").unwrap_or("");
                (format!("SELECT a.id, a.student_id, a.course_id, a.date, a.time, a.status, a.subject, a.teacher_id, a.observation FROM attendance a JOIN schools sch ON sch.id = a.school_id WHERE student_id = $1 AND course_id = $2 AND a.school_id = $3{} ORDER BY a.date DESC", corp_clause), corporation_id)
            } else {
                ("SELECT id, student_id, course_id, date, time, status, subject, teacher_id, observation FROM attendance WHERE student_id = $1 AND course_id = $2 ORDER BY date DESC".to_string(), None)
            };
            let mut q = sqlx::query_as::<_, RawAttendance>(&sql).bind(sid).bind(cid);
            if let Some(sc) = school_id { q = q.bind(sc); }
            if let Some(cc) = corp_val { q = q.bind(cc); }
            q.fetch_all(&state.pool).await?
        }
    } else if let Some(sid) = filter.student_id {
        let (sql, corp_val) = if school_id.is_some() {
            let corp_clause = corporation_id.map(|_| " AND sch.corporation_id = $3").unwrap_or("");
            (format!("SELECT a.id, a.student_id, a.course_id, a.date, a.time, a.status, a.subject, a.teacher_id, a.observation FROM attendance a JOIN schools sch ON sch.id = a.school_id WHERE student_id = $1 AND a.school_id = $2{} ORDER BY a.date DESC", corp_clause), corporation_id)
        } else {
            ("SELECT id, student_id, course_id, date, time, status, subject, teacher_id, observation FROM attendance WHERE student_id = $1 ORDER BY date DESC".to_string(), None)
        };
        let mut q = sqlx::query_as::<_, RawAttendance>(&sql).bind(sid);
        if let Some(sc) = school_id { q = q.bind(sc); }
        if let Some(cc) = corp_val { q = q.bind(cc); }
        q.fetch_all(&state.pool).await?
    } else if let Some(cid) = filter.course_id {
        let (sql, corp_val) = if school_id.is_some() {
            let corp_clause = corporation_id.map(|_| " AND sch.corporation_id = $3").unwrap_or("");
            (format!("SELECT a.id, a.student_id, a.course_id, a.date, a.time, a.status, a.subject, a.teacher_id, a.observation FROM attendance a JOIN schools sch ON sch.id = a.school_id WHERE course_id = $1 AND a.school_id = $2{} ORDER BY a.date DESC", corp_clause), corporation_id)
        } else {
            ("SELECT id, student_id, course_id, date, time, status, subject, teacher_id, observation FROM attendance WHERE course_id = $1 ORDER BY date DESC".to_string(), None)
        };
        let mut q = sqlx::query_as::<_, RawAttendance>(&sql).bind(cid);
        if let Some(sc) = school_id { q = q.bind(sc); }
        if let Some(cc) = corp_val { q = q.bind(cc); }
        q.fetch_all(&state.pool).await?
    } else {
        let (sql, corp_val) = if school_id.is_some() {
            let corp_clause = corporation_id.map(|_| " AND sch.corporation_id = $2").unwrap_or("");
            (format!("SELECT a.id, a.student_id, a.course_id, a.date, a.time, a.status, a.subject, a.teacher_id, a.observation FROM attendance a JOIN schools sch ON sch.id = a.school_id WHERE 1=1 AND a.school_id = $1{} ORDER BY a.date DESC LIMIT 100", corp_clause), corporation_id)
        } else {
            ("SELECT id, student_id, course_id, date, time, status, subject, teacher_id, observation FROM attendance WHERE 1=1 ORDER BY date DESC LIMIT 100".to_string(), None)
        };
        let mut q = sqlx::query_as::<_, RawAttendance>(&sql);
        if let Some(sc) = school_id { q = q.bind(sc); }
        if let Some(cc) = corp_val { q = q.bind(cc); }
        q.fetch_all(&state.pool).await?
    };

    Ok(Json(json!({ "records": records, "total": records.len() })))
}

async fn get_attendance(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AttendanceResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "attendance",
    )
    .await
    .map_err(|e| AttendanceError::Forbidden(e))?;

    let record = sqlx::query_as::<_, RawAttendance>(
        "SELECT id, student_id, course_id, date, time, status, subject, teacher_id, observation FROM attendance WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AttendanceError::NotFound("Registro de asistencia no encontrado".into()))?;

    Ok(Json(json!({ "record": record })))
}

async fn create_attendance(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::attendance::CreateAttendancePayload>,
) -> AttendanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor"])?;

    if payload.student_id.is_nil() || payload.course_id.is_nil() || payload.teacher_id.is_nil() {
        return Err(AttendanceError::Validation(
            "IDs de estudiante, curso y profesor son obligatorios".into(),
        ));
    }
    if payload.subject.trim().is_empty() {
        return Err(AttendanceError::Validation(
            "La asignatura es obligatoria".into(),
        ));
    }

    let status = schoolccb_common::attendance::AttendanceStatus::from_str(&payload.status)
        .ok_or_else(|| AttendanceError::Validation(format!("Estado de asistencia inválido: {}", payload.status)))?;
    let id = Uuid::new_v4();

    let result = sqlx::query_as::<_, RawAttendance>(
        r#"
        INSERT INTO attendance (id, student_id, course_id, date, time, status, subject, teacher_id, observation)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, student_id, course_id, date, time, status, subject, teacher_id, observation
        "#,
    )
    .bind(id)
    .bind(payload.student_id)
    .bind(payload.course_id)
    .bind(payload.date)
    .bind(payload.time)
    .bind(status.as_str())
    .bind(&payload.subject)
    .bind(payload.teacher_id)
    .bind(&payload.observation)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint() == Some("attendance_student_id_course_id_date_subject_key") {
                return AttendanceError::Conflict("Ya existe un registro de asistencia para este estudiante, curso, fecha y asignatura".into());
            }
        }
        AttendanceError::Database(e)
    })?;

    Ok(Json(json!({ "record": result })))
}

async fn update_attendance(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::attendance::UpdateAttendancePayload>,
) -> AttendanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor"])?;

    let existing = sqlx::query_as::<_, RawAttendance>(
        "SELECT id, student_id, course_id, date, time, status, subject, teacher_id, observation FROM attendance WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AttendanceError::NotFound("Registro de asistencia no encontrado".into()))?;

    let status = match &payload.status {
        Some(s) => schoolccb_common::attendance::AttendanceStatus::from_str(s)
            .ok_or_else(|| AttendanceError::Validation(format!("Estado de asistencia inválido: {s}")))?,
        None => schoolccb_common::attendance::AttendanceStatus::from_str(&existing.status)
            .unwrap_or(schoolccb_common::attendance::AttendanceStatus::Presente),
    };
    let time = payload.time.or(existing.time);
    let observation = payload.observation.or(existing.observation);

    let result = sqlx::query_as::<_, RawAttendance>(
        r#"
        UPDATE attendance SET status = $1, time = $2, observation = $3
        WHERE id = $4
        RETURNING id, student_id, course_id, date, time, status, subject, teacher_id, observation
        "#,
    )
    .bind(status.as_str())
    .bind(time)
    .bind(&observation)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "record": result })))
}

async fn delete_attendance(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AttendanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP"])?;

    let result = sqlx::query("DELETE FROM attendance WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AttendanceError::NotFound(
            "Registro de asistencia no encontrado".into(),
        ));
    }

    Ok(Json(
        json!({ "message": "Registro de asistencia eliminado correctamente" }),
    ))
}

async fn bulk_create_attendance(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::attendance::BulkAttendanceEntry>,
) -> AttendanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor"])?;

    if payload.records.is_empty() {
        return Err(AttendanceError::Validation(
            "Debe incluir al menos un registro de asistencia".into(),
        ));
    }

    let mut imported = 0;
    let mut errors: Vec<Value> = vec![];

    for record in &payload.records {
        let Some(status) = schoolccb_common::attendance::AttendanceStatus::from_str(&record.status) else {
            errors.push(json!({
                "student_id": record.student_id,
                "error": format!("Estado de asistencia inválido: {}", record.status),
            }));
            continue;
        };
        let id = Uuid::new_v4();

        let result = sqlx::query(
            r#"
            INSERT INTO attendance (id, student_id, course_id, date, time, status, subject, teacher_id, observation)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (student_id, course_id, date, subject)
            DO UPDATE SET status = EXCLUDED.status, time = EXCLUDED.time, observation = EXCLUDED.observation
            "#,
        )
        .bind(id)
        .bind(record.student_id)
        .bind(payload.course_id)
        .bind(payload.date)
        .bind(payload.time)
        .bind(status.as_str())
        .bind(&payload.subject)
        .bind(payload.teacher_id)
        .bind(&record.observation)
        .execute(&state.pool)
        .await;

        match result {
            Ok(_) => imported += 1,
            Err(e) => {
                errors.push(json!({
                    "student_id": record.student_id,
                    "error": e.to_string()
                }));
            }
        }
    }

    Ok(Json(json!({
        "imported": imported,
        "errors": errors,
        "total": payload.records.len()
    })))
}

async fn today_attendance(
    claims: Claims,
    State(state): State<AppState>,
) -> AttendanceResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "attendance",
    )
    .await
    .map_err(|e| AttendanceError::Forbidden(e))?;

    let today = chrono::Utc::now().date_naive();
    let records = sqlx::query_as::<_, RawAttendance>(
        "SELECT id, student_id, course_id, date, time, status, subject, teacher_id, observation
         FROM attendance WHERE date = $1 ORDER BY subject, student_id",
    )
    .bind(today)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(
        json!({ "date": today.to_string(), "records": records, "total": records.len() }),
    ))
}

async fn attendance_by_date(
    claims: Claims,
    State(state): State<AppState>,
    Path(date): Path<NaiveDate>,
) -> AttendanceResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;

    let records = sqlx::query_as::<_, RawAttendance>(
        "SELECT id, student_id, course_id, date, time, status, subject, teacher_id, observation
         FROM attendance WHERE date = $1 ORDER BY subject, student_id",
    )
    .bind(date)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(
        json!({ "date": date.to_string(), "records": records, "total": records.len() }),
    ))
}

async fn attendance_by_student(
    claims: Claims,
    State(state): State<AppState>,
    Path(student_id): Path<Uuid>,
) -> AttendanceResult<Json<Value>> {
    require_any_role(
        &claims,
        &[
            "Administrador",
            "Sostenedor",
            "Director",
            "UTP",
            "Profesor",
            "Apoderado",
            "Alumno",
        ],
    )?;

    let records = sqlx::query_as::<_, RawAttendance>(
        "SELECT id, student_id, course_id, date, time, status, subject, teacher_id, observation
         FROM attendance WHERE student_id = $1 ORDER BY date DESC",
    )
    .bind(student_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "records": records, "total": records.len() })))
}

async fn attendance_by_course_date(
    claims: Claims,
    State(state): State<AppState>,
    Path((course_id, date)): Path<(Uuid, NaiveDate)>,
) -> AttendanceResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;

    let records = sqlx::query_as::<_, RawAttendance>(
        "SELECT id, student_id, course_id, date, time, status, subject, teacher_id, observation
         FROM attendance WHERE course_id = $1 AND date = $2 ORDER BY student_id",
    )
    .bind(course_id)
    .bind(date)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "records": records, "total": records.len() })))
}
