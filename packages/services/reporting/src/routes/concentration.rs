use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{ReportError, ReportResult};
use schoolccb_common::auth::{Claims, require_any_role};

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/api/reports/concentration/{student_id}/{year}",
        get(concentration),
    )
}

async fn concentration(
    claims: Claims,
    State(state): State<AppState>,
    Path((student_id, year)): Path<(Uuid, i32)>,
) -> ReportResult<Json<Value>> {
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
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "reports",
    )
    .await
    .map_err(|e| ReportError::Forbidden(e))?;

    let student = sqlx::query_as::<_, (String, String)>(
        "SELECT CONCAT(first_name, ' ', last_name), rut FROM students WHERE id = $1",
    )
    .bind(student_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(crate::error::ReportError::NotFound(
        "Estudiante no encontrado".into(),
    ))?;

    let s1 = build_semester_concentration(&state.pool, student_id, 1, year).await?;
    let s2 = build_semester_concentration(&state.pool, student_id, 2, year).await?;

    let s1_avg = semester_global(&s1);
    let s2_avg = semester_global(&s2);
    let final_avg = if s1_avg > 0.0 && s2_avg > 0.0 {
        (s1_avg + s2_avg) / 2.0
    } else if s1_avg > 0.0 {
        s1_avg
    } else {
        s2_avg
    };

    let all_failed: Vec<String> = s1
        .iter()
        .chain(s2.iter())
        .filter(|s| s.average < 4.0)
        .map(|s| s.subject_name.clone())
        .collect();

    let promotion = match all_failed.len() {
        0 => "Promovido",
        1 | 2 => "Promovido (Decreto 67)",
        _ => "Reprobado",
    };

    Ok(Json(json!({
        "concentration": {
            "student_id": student_id,
            "student_name": student.0,
            "rut": student.1,
            "year": year,
            "semesters": [
                {
                    "semester": 1,
                    "subjects": &s1,
                    "global_average": (s1_avg * 10.0).round() / 10.0
                },
                {
                    "semester": 2,
                    "subjects": &s2,
                    "global_average": (s2_avg * 10.0).round() / 10.0
                }
            ],
            "final_promotion": promotion,
            "final_average": (final_avg * 10.0).round() / 10.0
        }
    })))
}

async fn build_semester_concentration(
    pool: &sqlx::PgPool,
    student_id: Uuid,
    semester: i32,
    year: i32,
) -> Result<Vec<SubjectConcRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, SubjectConcRow>(
        r#"
        SELECT
            COALESCE(s.name, g.subject) as subject_name,
            COALESCE(s.code, '') as subject_code,
            COUNT(*)::int as grades_count,
            ROUND(AVG(g.grade)::numeric, 1)::float8 as average,
            ROUND(MIN(g.grade)::numeric, 1)::float8 as min_grade,
            ROUND(MAX(g.grade)::numeric, 1)::float8 as max_grade
        FROM grades g
        LEFT JOIN course_subjects cs ON cs.id = g.course_subject_id
        LEFT JOIN subjects s ON s.id = cs.subject_id
        WHERE g.student_id = $1 AND g.semester = $2 AND g.year = $3
        GROUP BY s.name, s.code, g.subject
        ORDER BY s.name, g.subject
        "#,
    )
    .bind(student_id)
    .bind(semester)
    .bind(year)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

fn semester_global(subjects: &[SubjectConcRow]) -> f64 {
    if subjects.is_empty() {
        return 0.0;
    }
    subjects.iter().map(|s| s.average).sum::<f64>() / subjects.len() as f64
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
struct SubjectConcRow {
    subject_name: String,
    subject_code: String,
    grades_count: i32,
    average: f64,
    min_grade: f64,
    max_grade: f64,
}
