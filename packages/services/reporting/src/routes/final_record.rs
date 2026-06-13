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
        "/api/reports/final-record/{course_id}/{year}",
        get(final_record),
    )
}

async fn final_record(
    claims: Claims,
    State(state): State<AppState>,
    Path((course_id, year)): Path<(Uuid, i32)>,
) -> ReportResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "reports",
    )
    .await
    .map_err(|e| ReportError::Forbidden(e))?;

    let course = sqlx::query_as::<_, (String, String, String)>(
        "SELECT name, grade_level, section FROM courses WHERE id = $1",
    )
    .bind(course_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(crate::error::ReportError::NotFound(
        "Curso no encontrado".into(),
    ))?;

    let subject_list: Vec<(Uuid, String, String)> = sqlx::query_as(
        r#"
        SELECT cs.id, COALESCE(s.code, ''), COALESCE(s.name, '')
        FROM course_subjects cs
        JOIN subjects s ON s.id = cs.subject_id
        WHERE cs.course_id = $1 AND cs.academic_year = $2
        ORDER BY s.name
        "#,
    )
    .bind(course_id)
    .bind(year)
    .fetch_all(&state.pool)
    .await?;

    let students: Vec<(Uuid, String, String)> = sqlx::query_as(
        "SELECT s.id, CONCAT(s.first_name, ' ', s.last_name), s.rut FROM students s
         JOIN enrollments e ON e.student_id = s.id
         WHERE e.course_id = $1 AND e.year = $2 AND e.active = true AND s.enrolled = true
         ORDER BY s.last_name, s.first_name",
    )
    .bind(course_id)
    .bind(year)
    .fetch_all(&state.pool)
    .await?;

    let mut student_results: Vec<Value> = vec![];
    let mut total_promoted = 0usize;
    let mut total_failed = 0usize;

    for (sid, sname, srut) in &students {
        let mut subjects: Vec<Value> = vec![];

        for (cs_id, subj_code, subj_name) in &subject_list {
            let s1_avg: Option<f64> = sqlx::query_scalar(
                "SELECT ROUND(AVG(grade)::numeric, 1)::float8 FROM grades WHERE student_id = $1 AND course_subject_id = $2 AND semester = 1 AND year = $3",
            )
            .bind(sid).bind(cs_id).bind(year)
            .fetch_one(&state.pool)
            .await?;

            let s2_avg: Option<f64> = sqlx::query_scalar(
                "SELECT ROUND(AVG(grade)::numeric, 1)::float8 FROM grades WHERE student_id = $1 AND course_subject_id = $2 AND semester = 2 AND year = $3",
            )
            .bind(sid).bind(cs_id).bind(year)
            .fetch_one(&state.pool)
            .await?;

            let s1 = s1_avg.unwrap_or(0.0);
            let s2 = s2_avg.unwrap_or(0.0);
            let final_avg = if s1 > 0.0 && s2 > 0.0 {
                (s1 + s2) / 2.0
            } else if s1 > 0.0 {
                s1
            } else {
                s2
            };

            subjects.push(json!({
                "subject_name": subj_name,
                "subject_code": subj_code,
                "semester1_avg": s1,
                "semester2_avg": s2,
                "final_avg": (final_avg * 10.0).round() / 10.0,
            }));
        }

        let all_avgs: Vec<f64> = subjects
            .iter()
            .filter_map(|s| s["final_avg"].as_f64())
            .collect();
        let final_avg = if all_avgs.is_empty() {
            0.0
        } else {
            all_avgs.iter().sum::<f64>() / all_avgs.len() as f64
        };

        let failed_count = subjects
            .iter()
            .filter(|s| s["final_avg"].as_f64().unwrap_or(0.0) < 4.0)
            .count();

        let promotion = match failed_count {
            0 => "Promovido",
            1 | 2 => "Promovido (Decreto 67)",
            _ => "Reprobado",
        };

        if promotion.starts_with("Promovido") {
            total_promoted += 1;
        } else {
            total_failed += 1;
        }

        student_results.push(json!({
            "student_id": sid,
            "student_name": sname,
            "rut": srut,
            "subjects": subjects,
            "final_average": (final_avg * 10.0).round() / 10.0,
            "promotion": promotion,
        }));
    }

    let total = student_results.len();
    let promotion_rate = if total > 0 {
        (total_promoted as f64 / total as f64 * 100.0 * 10.0).round() / 10.0
    } else {
        0.0
    };

    Ok(Json(json!({
        "final_record": {
            "course_id": course_id,
            "course_name": course.0,
            "grade_level": course.1,
            "section": course.2,
            "year": year,
            "students": student_results,
            "summary": {
                "total_students": total,
                "promoted": total_promoted,
                "failed": total_failed,
                "average_promotion_rate": promotion_rate,
            }
        }
    })))
}
