use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::AttendanceResult;
use schoolccb_common::auth::{Claims, require_any_role};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/attendance/alerts", get(alerts))
        .route(
            "/api/attendance/alerts/student/{student_id}",
            get(student_alert),
        )
        .route(
            "/api/attendance/alerts/course/{course_id}",
            get(course_alerts),
        )
}

async fn alerts(claims: Claims, State(state): State<AppState>) -> AttendanceResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;

    let raw = sqlx::query_as::<_, RawAlert>(
        r#"
        WITH recent AS (
            SELECT
                s.id as student_id,
                CONCAT(s.first_name, ' ', s.last_name) as student_name,
                s.rut,
                COUNT(*) FILTER (WHERE a.status = 'Ausente') as total_absences,
                COUNT(*) as total_days,
                s.nee
            FROM students s
            JOIN attendance a ON a.student_id = s.id
            WHERE a.date >= CURRENT_DATE - INTERVAL '30 days'
              AND s.enrolled = true
            GROUP BY s.id, s.first_name, s.last_name, s.rut, s.nee
        )
        SELECT
            student_id, student_name, rut,
            EXTRACT(MONTH FROM CURRENT_DATE)::int as month,
            EXTRACT(YEAR FROM CURRENT_DATE)::int as year,
            CASE WHEN total_days > 0
                THEN ROUND((1.0 - total_absences::float / total_days) * 100, 1)
                ELSE 100.0
            END as attendance_percentage,
            total_absences,
            CASE
                WHEN nee = 'T' OR nee = 'P' THEN 75.0
                ELSE 85.0
            END as threshold,
            CASE
                WHEN nee = 'T' OR nee = 'P' THEN
                    CASE WHEN (1.0 - total_absences::float / GREATEST(total_days, 1)) * 100 < 75 THEN 'Alto'
                         WHEN (1.0 - total_absences::float / GREATEST(total_days, 1)) * 100 < 85 THEN 'Medio'
                         ELSE 'Bajo'
                    END
                ELSE
                    CASE WHEN (1.0 - total_absences::float / GREATEST(total_days, 1)) * 100 < 85 THEN 'Alto'
                         WHEN (1.0 - total_absences::float / GREATEST(total_days, 1)) * 100 < 90 THEN 'Medio'
                         ELSE 'Bajo'
                    END
            END as severity
        FROM recent
        WHERE total_absences > 0
          AND (
              (nee = 'T' OR nee = 'P') AND (1.0 - total_absences::float / GREATEST(total_days, 1)) * 100 < 85
              OR
              (nee != 'T' AND nee != 'P') AND (1.0 - total_absences::float / GREATEST(total_days, 1)) * 100 < 90
          )
        ORDER BY total_absences DESC
        LIMIT 20
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({
        "alerts": raw,
        "total": raw.len(),
        "threshold_general": 85.0,
        "threshold_nee": 75.0
    })))
}

async fn student_alert(
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
        ],
    )?;

    let raw = sqlx::query_as::<_, RawAlert>(
        r#"
        WITH recent AS (
            SELECT
                s.id as student_id,
                CONCAT(s.first_name, ' ', s.last_name) as student_name,
                s.rut,
                COUNT(*) FILTER (WHERE a.status = 'Ausente') as total_absences,
                COUNT(*) as total_days,
                s.nee
            FROM students s
            JOIN attendance a ON a.student_id = s.id
            WHERE a.date >= CURRENT_DATE - INTERVAL '30 days'
              AND s.id = $1
              AND s.enrolled = true
            GROUP BY s.id, s.first_name, s.last_name, s.rut, s.nee
        )
        SELECT
            student_id, student_name, rut,
            EXTRACT(MONTH FROM CURRENT_DATE)::int as month,
            EXTRACT(YEAR FROM CURRENT_DATE)::int as year,
            CASE WHEN total_days > 0
                THEN ROUND((1.0 - total_absences::float / total_days) * 100, 1)
                ELSE 100.0
            END as attendance_percentage,
            total_absences,
            CASE
                WHEN nee = 'T' OR nee = 'P' THEN 75.0
                ELSE 85.0
            END as threshold,
            CASE
                WHEN nee = 'T' OR nee = 'P' THEN
                    CASE WHEN (1.0 - total_absences::float / GREATEST(total_days, 1)) * 100 < 75 THEN 'Alto'
                         WHEN (1.0 - total_absences::float / GREATEST(total_days, 1)) * 100 < 85 THEN 'Medio'
                         ELSE 'Bajo'
                    END
                ELSE
                    CASE WHEN (1.0 - total_absences::float / GREATEST(total_days, 1)) * 100 < 85 THEN 'Alto'
                         WHEN (1.0 - total_absences::float / GREATEST(total_days, 1)) * 100 < 90 THEN 'Medio'
                         ELSE 'Bajo'
                    END
            END as severity
        FROM recent
        "#,
    )
    .bind(student_id)
    .fetch_optional(&state.pool)
    .await?;

    Ok(Json(json!({ "alert": raw })))
}

async fn course_alerts(
    claims: Claims,
    State(state): State<AppState>,
    Path(course_id): Path<Uuid>,
) -> AttendanceResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;

    let raw = sqlx::query_as::<_, RawAlert>(
        r#"
        WITH recent AS (
            SELECT
                s.id as student_id,
                CONCAT(s.first_name, ' ', s.last_name) as student_name,
                s.rut,
                COUNT(*) FILTER (WHERE a.status = 'Ausente') as total_absences,
                COUNT(*) as total_days,
                s.nee
            FROM students s
            JOIN attendance a ON a.student_id = s.id
            JOIN enrollments e ON e.student_id = s.id AND e.course_id = a.course_id
            WHERE a.date >= CURRENT_DATE - INTERVAL '30 days'
              AND e.course_id = $1
              AND s.enrolled = true
              AND e.active = true
            GROUP BY s.id, s.first_name, s.last_name, s.rut, s.nee
        )
        SELECT
            student_id, student_name, rut,
            EXTRACT(MONTH FROM CURRENT_DATE)::int as month,
            EXTRACT(YEAR FROM CURRENT_DATE)::int as year,
            CASE WHEN total_days > 0
                THEN ROUND((1.0 - total_absences::float / total_days) * 100, 1)
                ELSE 100.0
            END as attendance_percentage,
            total_absences,
            CASE
                WHEN nee = 'T' OR nee = 'P' THEN 75.0
                ELSE 85.0
            END as threshold,
            CASE
                WHEN nee = 'T' OR nee = 'P' THEN
                    CASE WHEN (1.0 - total_absences::float / GREATEST(total_days, 1)) * 100 < 75 THEN 'Alto'
                         WHEN (1.0 - total_absences::float / GREATEST(total_days, 1)) * 100 < 85 THEN 'Medio'
                         ELSE 'Bajo'
                    END
                ELSE
                    CASE WHEN (1.0 - total_absences::float / GREATEST(total_days, 1)) * 100 < 85 THEN 'Alto'
                         WHEN (1.0 - total_absences::float / GREATEST(total_days, 1)) * 100 < 90 THEN 'Medio'
                         ELSE 'Bajo'
                    END
            END as severity
        FROM recent
        WHERE total_absences > 0
        ORDER BY total_absences DESC
        LIMIT 20
        "#,
    )
    .bind(course_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({
        "course_id": course_id,
        "alerts": raw,
        "total": raw.len()
    })))
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct RawAlert {
    student_id: Uuid,
    student_name: String,
    rut: String,
    month: i32,
    year: i32,
    attendance_percentage: f64,
    total_absences: i64,
    threshold: f64,
    severity: String,
}
