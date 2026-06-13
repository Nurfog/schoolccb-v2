use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{AttendanceError, AttendanceResult};
use schoolccb_common::auth::{Claims, require_any_role};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/attendance/monthly/{year}/{month}",
            get(monthly_report),
        )
        .route(
            "/api/attendance/monthly/{year}/{month}/{course_id}",
            get(monthly_report_by_course),
        )
        .route(
            "/api/attendance/student/{student_id}/summary/{year}",
            get(student_yearly_summary),
        )
        .route(
            "/api/attendance/report/export/{year}/{month}",
            get(export_supereduc),
        )
}

async fn monthly_report(
    claims: Claims,
    State(state): State<AppState>,
    Path((year, month)): Path<(i32, u32)>,
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

    let summary = get_monthly_summary(&state.pool, year, month).await?;

    let total_students = summary.len();
    let total_below_85 = summary
        .iter()
        .filter(|s| s.attendance_percentage() < 85.0)
        .count();
    let total_below_75 = summary
        .iter()
        .filter(|s| s.attendance_percentage() < 75.0)
        .count();

    Ok(Json(json!({
        "year": year,
        "month": month,
        "students": summary,
        "summary": {
            "total_students": total_students,
            "below_general_threshold": total_below_85,
            "below_nee_threshold": total_below_75
        }
    })))
}

async fn monthly_report_by_course(
    claims: Claims,
    State(state): State<AppState>,
    Path((year, month, course_id)): Path<(i32, u32, Uuid)>,
) -> AttendanceResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;

    let summary = get_monthly_summary_by_course(&state.pool, year, month, course_id).await?;

    Ok(Json(json!({
        "year": year,
        "month": month,
        "course_id": course_id,
        "students": summary,
        "total": summary.len()
    })))
}

async fn student_yearly_summary(
    claims: Claims,
    State(state): State<AppState>,
    Path((student_id, year)): Path<(Uuid, i32)>,
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

    let mut months = vec![];
    let mut total_days = 0i32;
    let mut present = 0i32;
    let mut absent = 0i32;
    let mut late = 0i32;
    let mut justified = 0i32;

    for month in 1..=12u32 {
        let records = sqlx::query_as::<_, (i32, i32, i32, i32, i32)>(
            r#"
            SELECT
                COUNT(*)::int as total_days,
                COUNT(*) FILTER (WHERE status = 'Presente')::int as present,
                COUNT(*) FILTER (WHERE status = 'Ausente')::int as absent,
                COUNT(*) FILTER (WHERE status = 'Atraso')::int as late,
                COUNT(*) FILTER (WHERE status = 'Justificado')::int as justified
            FROM attendance
            WHERE student_id = $1 AND EXTRACT(YEAR FROM date) = $2 AND EXTRACT(MONTH FROM date) = $3
            "#,
        )
        .bind(student_id)
        .bind(year)
        .bind(month as i32)
        .fetch_one(&state.pool)
        .await?;

        if records.0 > 0 {
            total_days += records.0;
            present += records.1;
            absent += records.2;
            late += records.3;
            justified += records.4;

            months.push(serde_json::json!({
                "month": month,
                "total_days": records.0,
                "present": records.1,
                "absent": records.2,
                "late": records.3,
                "justified": records.4,
                "percentage": if records.0 > 0 {
                    (records.1 as f64 / records.0 as f64 * 100.0 * 10.0).round() / 10.0
                } else { 100.0 }
            }));
        }
    }

    let attendance_pct = if total_days > 0 {
        (present as f64 / total_days as f64 * 100.0 * 10.0).round() / 10.0
    } else {
        100.0
    };

    Ok(Json(json!({
        "student_id": student_id,
        "year": year,
        "months": months,
        "total_days": total_days,
        "present": present,
        "absent": absent,
        "late": late,
        "justified": justified,
        "attendance_percentage": attendance_pct,
        "is_below_general_threshold": attendance_pct < 85.0,
        "is_below_nee_threshold": attendance_pct < 75.0
    })))
}

async fn export_supereduc(
    claims: Claims,
    State(state): State<AppState>,
    Path((year, month)): Path<(i32, u32)>,
) -> AttendanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "attendance",
    )
    .await
    .map_err(|e| AttendanceError::Forbidden(e))?;

    let rows = sqlx::query_as::<_, SupereducRow>(
        r#"
        SELECT
            s.rut,
            CONCAT(s.first_name, ' ', s.last_name) as student_name,
            s.grade_level,
            s.section,
            COUNT(*)::int as total_days,
            COUNT(*) FILTER (WHERE a.status = 'Presente')::int as present,
            COUNT(*) FILTER (WHERE a.status = 'Ausente')::int as absent,
            COUNT(*) FILTER (WHERE a.status = 'Atraso')::int as late,
            COUNT(*) FILTER (WHERE a.status = 'Justificado')::int as justified
        FROM students s
        JOIN attendance a ON a.student_id = s.id
        WHERE EXTRACT(YEAR FROM a.date) = $1
          AND EXTRACT(MONTH FROM a.date) = $2
          AND s.enrolled = true
        GROUP BY s.id, s.rut, s.first_name, s.last_name, s.grade_level, s.section
        ORDER BY s.grade_level, s.section, s.last_name, s.first_name
        "#,
    )
    .bind(year)
    .bind(month as i32)
    .fetch_all(&state.pool)
    .await?;

    let rows_with_pct: Vec<Value> = rows
        .iter()
        .map(|r| {
            let pct = if r.total_days > 0 {
                (r.present as f64 / r.total_days as f64 * 100.0 * 10.0).round() / 10.0
            } else {
                100.0
            };
            json!({
                "rut": r.rut,
                "student_name": r.student_name,
                "grade_level": r.grade_level,
                "section": r.section,
                "total_days": r.total_days,
                "present": r.present,
                "absent": r.absent,
                "late": r.late,
                "justified": r.justified,
                "attendance_percentage": pct
            })
        })
        .collect();

    Ok(Json(json!({
        "year": year,
        "month": month,
        "rows": rows_with_pct,
        "total": rows.len(),
        "format": "SUPEREDUC"
    })))
}

async fn get_monthly_summary(
    pool: &sqlx::PgPool,
    year: i32,
    month: u32,
) -> Result<Vec<MonthlySummaryRow>, sqlx::Error> {
    let raw = sqlx::query_as::<_, MonthlySummaryRow>(
        r#"
        SELECT
            s.id as student_id,
            CONCAT(s.first_name, ' ', s.last_name) as student_name,
            s.rut,
            COUNT(*)::int as total_days,
            COUNT(*) FILTER (WHERE a.status = 'Presente')::int as present,
            COUNT(*) FILTER (WHERE a.status = 'Ausente')::int as absent,
            COUNT(*) FILTER (WHERE a.status = 'Atraso')::int as late,
            COUNT(*) FILTER (WHERE a.status = 'Justificado')::int as justified
        FROM students s
        JOIN attendance a ON a.student_id = s.id
        WHERE EXTRACT(YEAR FROM a.date) = $1
          AND EXTRACT(MONTH FROM a.date) = $2
          AND s.enrolled = true
        GROUP BY s.id, s.first_name, s.last_name, s.rut
        ORDER BY s.last_name, s.first_name
        "#,
    )
    .bind(year)
    .bind(month as i32)
    .fetch_all(pool)
    .await?;

    Ok(raw)
}

async fn get_monthly_summary_by_course(
    pool: &sqlx::PgPool,
    year: i32,
    month: u32,
    course_id: Uuid,
) -> Result<Vec<MonthlySummaryRow>, sqlx::Error> {
    let raw = sqlx::query_as::<_, MonthlySummaryRow>(
        r#"
        SELECT
            s.id as student_id,
            CONCAT(s.first_name, ' ', s.last_name) as student_name,
            s.rut,
            COUNT(*)::int as total_days,
            COUNT(*) FILTER (WHERE a.status = 'Presente')::int as present,
            COUNT(*) FILTER (WHERE a.status = 'Ausente')::int as absent,
            COUNT(*) FILTER (WHERE a.status = 'Atraso')::int as late,
            COUNT(*) FILTER (WHERE a.status = 'Justificado')::int as justified
        FROM students s
        JOIN attendance a ON a.student_id = s.id
        JOIN enrollments e ON e.student_id = s.id AND e.course_id = a.course_id
        WHERE EXTRACT(YEAR FROM a.date) = $1
          AND EXTRACT(MONTH FROM a.date) = $2
          AND a.course_id = $3
          AND s.enrolled = true
          AND e.active = true
        GROUP BY s.id, s.first_name, s.last_name, s.rut
        ORDER BY s.last_name, s.first_name
        "#,
    )
    .bind(year)
    .bind(month as i32)
    .bind(course_id)
    .fetch_all(pool)
    .await?;

    Ok(raw)
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
struct MonthlySummaryRow {
    student_id: Uuid,
    student_name: String,
    rut: String,
    total_days: i32,
    present: i32,
    absent: i32,
    late: i32,
    justified: i32,
}

impl MonthlySummaryRow {
    fn attendance_percentage(&self) -> f64 {
        if self.total_days == 0 {
            return 100.0;
        }
        (self.present as f64 / self.total_days as f64) * 100.0
    }
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
struct SupereducRow {
    rut: String,
    student_name: String,
    grade_level: String,
    section: String,
    total_days: i32,
    present: i32,
    absent: i32,
    late: i32,
    justified: i32,
}
