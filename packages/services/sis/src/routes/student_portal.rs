use axum::{
    Json, Router,
    extract::State,
    routing::get,
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::error::{SisError, SisResult};
use crate::routes::students::Claims;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/portal/student/grades", get(my_grades))
        .route("/api/portal/student/attendance", get(my_attendance))
        .route("/api/portal/student/schedule", get(my_schedule))
        .route("/api/portal/student/annotations", get(my_annotations))
        .route("/api/portal/student/appointments", get(my_appointments).post(create_my_appointment))
        .route("/api/portal/student/profile", get(my_profile))
}

fn require_alumno(claims: &Claims) -> Result<(), SisError> {
    if claims.role == "Alumno" || claims.role == "GerenteGeneral" {
        Ok(())
    } else {
        Err(SisError::Forbidden("Se requiere rol Alumno".into()))
    }
}

fn student_id_from_claims(claims: &Claims) -> Result<Uuid, SisError> {
    // The sub in JWT is the user_id. Students have a matching record in students table.
    Uuid::parse_str(&claims.sub).map_err(|_| SisError::Unauthorized)
}

async fn my_grades(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    require_alumno(&claims)?;
    let sid = student_id_from_claims(&claims)?;

    let averages = sqlx::query_as::<_, (String, f64)>(
        "SELECT sub.name, AVG(g.value)
         FROM grades g
         JOIN enrollments e ON e.id = g.enrollment_id AND e.student_id = $1 AND e.active = true
         LEFT JOIN subjects sub ON sub.id = g.subject_id
         WHERE g.value IS NOT NULL
         GROUP BY sub.name ORDER BY sub.name",
    )
    .bind(sid)
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|(s, a)| json!({"subject": s, "average": format!("{:.1}", a)}))
    .collect::<Vec<_>>();

    let grades = sqlx::query_as::<_, (String, String, f64, String)>(
        "SELECT sub.name, g.name, g.value, g.date::text
         FROM grades g
         JOIN enrollments e ON e.id = g.enrollment_id AND e.student_id = $1 AND e.active = true
         LEFT JOIN subjects sub ON sub.id = g.subject_id
         WHERE g.value IS NOT NULL
         ORDER BY g.date DESC LIMIT 30",
    )
    .bind(sid)
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|(s, n, v, d)| json!({"subject": s, "name": n, "value": v, "date": d}))
    .collect::<Vec<_>>();

    Ok(Json(json!({"averages": averages, "grades": grades})))
}

async fn my_attendance(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    require_alumno(&claims)?;
    let sid = student_id_from_claims(&claims)?;

    let monthly = sqlx::query_as::<_, (String, i64, i64, i64, i64)>(
        "SELECT to_char(date_trunc('month', date), 'YYYY-MM'),
                COUNT(*), SUM(CASE WHEN status = 'present' THEN 1 ELSE 0 END),
                SUM(CASE WHEN status = 'absent' THEN 1 ELSE 0 END),
                SUM(CASE WHEN status = 'late' THEN 1 ELSE 0 END)
         FROM attendance_records WHERE student_id = $1 AND date >= NOW() - INTERVAL '6 months'
         GROUP BY date_trunc('month', date) ORDER BY month",
    )
    .bind(sid)
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|(m, t, p, a, l)| json!({"month": m, "total": t, "present": p, "absent": a, "late": l,
        "percentage": if t > 0 { format!("{:.1}", (p as f64 / t as f64) * 100.0) } else { "0".into() } }))
    .collect::<Vec<_>>();

    Ok(Json(json!({"attendance": monthly})))
}

async fn my_schedule(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    require_alumno(&claims)?;
    let sid = student_id_from_claims(&claims)?;

    let schedule = sqlx::query_as::<_, (String, String, String)>(
        "SELECT sub.name, cs.day_of_week, cs.time_slot
         FROM course_subjects cs
         JOIN subjects sub ON sub.id = cs.subject_id
         JOIN enrollments e ON e.course_id = cs.course_id AND e.student_id = $1 AND e.active = true
         ORDER BY cs.day_of_week, cs.time_slot",
    )
    .bind(sid)
    .fetch_all(&state.pool)
    .await.unwrap_or_default()
    .into_iter()
    .map(|(s, d, t)| json!({"subject": s, "day": d, "time": t}))
    .collect::<Vec<_>>();

    Ok(Json(json!({"schedule": schedule})))
}

async fn my_annotations(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    require_alumno(&claims)?;
    let sid = student_id_from_claims(&claims)?;

    let annotations = sqlx::query_as::<_, (String, String, String, String, Option<String>)>(
        "SELECT a.annotation_type, a.description, a.severity, a.created_at::text, u.name
         FROM student_annotations a
         LEFT JOIN users u ON u.id = a.created_by
         WHERE a.student_id = $1
         ORDER BY a.created_at DESC LIMIT 20",
    )
    .bind(sid)
    .fetch_all(&state.pool)
    .await.unwrap_or_default()
    .into_iter()
    .map(|(t, d, s, c, teacher)| json!({"type": t, "description": d, "severity": s, "date": c, "teacher": teacher}))
    .collect::<Vec<_>>();

    Ok(Json(json!({"annotations": annotations})))
}

async fn my_profile(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    require_alumno(&claims)?;
    let sid = student_id_from_claims(&claims)?;

    let profile = sqlx::query_as::<_, (String, String, String, String, String)>(
        "SELECT s.first_name || ' ' || s.last_name, s.rut, COALESCE(e.grade_level, ''), COALESCE(e.section, ''), COALESCE(sch.name, '')
         FROM students s
         LEFT JOIN enrollments e ON e.student_id = s.id AND e.active = true
         LEFT JOIN schools sch ON sch.id = e.school_id
         WHERE s.id = $1",
    )
    .bind(sid)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(SisError::NotFound("Estudiante no encontrado".into()))?;

    Ok(Json(json!({"name": profile.0, "rut": profile.1, "grade_level": profile.2, "section": profile.3, "school": profile.4})))
}

async fn my_appointments(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    require_alumno(&claims)?;
    let sid = student_id_from_claims(&claims)?;

    let appointments = sqlx::query_as::<_, (Uuid, String, String, String)>(
        "SELECT id, appointment_type, reason, status
         FROM support_appointments WHERE student_id = $1 OR requested_by = $1
         ORDER BY created_at DESC LIMIT 10",
    )
    .bind(sid)
    .fetch_all(&state.pool)
    .await.unwrap_or_default()
    .into_iter()
    .map(|(id, t, r, s)| json!({"id": id, "type": t, "reason": r, "status": s}))
    .collect::<Vec<_>>();

    Ok(Json(json!({"appointments": appointments})))
}

async fn create_my_appointment(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> SisResult<Json<Value>> {
    require_alumno(&claims)?;
    let sid = student_id_from_claims(&claims)?;
    let id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO support_appointments (id, student_id, requested_by, appointment_type, reason, preferred_date, preferred_time)
         VALUES ($1, $2, $3, $4, $5, $6::date, $7::time)",
    )
    .bind(id)
    .bind(sid)
    .bind(sid)
    .bind(payload.get("type").and_then(|v| v.as_str()).unwrap_or("general"))
    .bind(payload.get("reason").and_then(|v| v.as_str()))
    .bind(payload.get("date").and_then(|v| v.as_str()))
    .bind(payload.get("time").and_then(|v| v.as_str()))
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"id": id})))
}
