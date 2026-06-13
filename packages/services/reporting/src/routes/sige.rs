use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde_json::{Value, json};

use crate::AppState;
use crate::error::{ReportError, ReportResult};
use schoolccb_common::auth::{Claims, require_any_role};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/reports/sige/students", get(sige_students))
        .route(
            "/api/reports/sige/attendance/{year}/{month}",
            get(sige_attendance),
        )
}

async fn sige_students(claims: Claims, State(state): State<AppState>) -> ReportResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "sige",
    )
    .await
    .map_err(|e| ReportError::Forbidden(e))?;

    let rows = sqlx::query_as::<_, SigeStudentRow>(
        r#"
        SELECT rut, first_name as names, last_name,
               grade_level, section,
               COALESCE(cod_nivel, '') as cod_nivel,
               condicion, prioritario, nee
        FROM students WHERE enrolled = true
        ORDER BY grade_level, section, last_name, first_name
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let now = chrono::Utc::now().format("%d/%m/%Y %H:%M").to_string();

    Ok(Json(json!({
        "rows": rows,
        "total": rows.len(),
        "generated_at": now
    })))
}

async fn sige_attendance(
    claims: Claims,
    State(state): State<AppState>,
    Path((year, month)): Path<(i32, u32)>,
) -> ReportResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "sige",
    )
    .await
    .map_err(|e| ReportError::Forbidden(e))?;

    let rows = sqlx::query_as::<_, SigeAttendanceRow>(
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
        "rows": rows_with_pct,
        "total": rows.len(),
        "year": year,
        "month": month
    })))
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
struct SigeStudentRow {
    rut: String,
    names: String,
    last_name: String,
    grade_level: String,
    section: String,
    cod_nivel: String,
    condicion: String,
    prioritario: String,
    nee: String,
}

// SIGE per-subject export gap:
// MINEDUC SIGE format requires per-subject fields TIPO_EVAL and SITUACION_FINAL
// for each student enrolled in a subject:
//   TIPO_EVAL: PA (Promedio Anual), SF (Semestral Final), AN (Anual), EX (Eximido)
//   SITUACION_FINAL: APR (Aprobado), REP (Reprobado), EXM (Eximido)
// No per-subject endpoint exists yet — add one at
// /api/reports/sige/grades/{year}/{semester} when grade data is available.

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
struct SigeAttendanceRow {
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
