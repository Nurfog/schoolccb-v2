use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::AcademicResult;
use schoolccb_common::auth::{Claims, require_any_role};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/grades/reports/student/{student_id}/{year}",
            get(student_yearly_report),
        )
        .route(
            "/api/grades/reports/student/{student_id}/{year}/{semester}",
            get(student_semester_report),
        )
        .route(
            "/api/grades/reports/course/{course_id}/{year}",
            get(course_performance),
        )
        .route(
            "/api/grades/reports/promotion/{course_id}/{year}",
            get(promotion_status),
        )
}

async fn student_yearly_report(
    claims: Claims,
    State(state): State<AppState>,
    Path((student_id, year)): Path<(Uuid, i32)>,
) -> AcademicResult<Json<Value>> {
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

    let student_name = get_student_name(&state.pool, student_id).await?;

    let s1 = build_semester_report(&state.pool, student_id, 1, year).await?;
    let s2 = build_semester_report(&state.pool, student_id, 2, year).await?;

    let s1_subjects = s1
        .as_ref()
        .map(|r| &r.subjects)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let all_subjects_s1: Vec<f64> = s1_subjects.iter().map(|s| s.weighted_average).collect();
    let s1_global = if all_subjects_s1.is_empty() {
        0.0
    } else {
        all_subjects_s1.iter().sum::<f64>() / all_subjects_s1.len() as f64
    };

    let final_promotion = if let Some(ref s2_report) = s2 {
        let s2_subjects: Vec<f64> = s2_report
            .subjects
            .iter()
            .map(|s| s.weighted_average)
            .collect();
        let s2_global = if s2_subjects.is_empty() {
            0.0
        } else {
            s2_subjects.iter().sum::<f64>() / s2_subjects.len() as f64
        };
        let yearly_avg = (s1_global + s2_global) / 2.0;
        if yearly_avg >= 4.0 {
            "Promovido"
        } else {
            "Reprobado"
        }
    } else if s1_global >= 4.0 {
        "Pendiente (S2)"
    } else {
        "Riesgo"
    };

    Ok(Json(json!({
        "student_id": student_id,
        "student_name": student_name,
        "year": year,
        "first_semester": s1,
        "second_semester": s2,
        "final_promotion": final_promotion
    })))
}

async fn student_semester_report(
    claims: Claims,
    State(state): State<AppState>,
    Path((student_id, year, semester)): Path<(Uuid, i32, i32)>,
) -> AcademicResult<Json<Value>> {
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

    let student_name = get_student_name(&state.pool, student_id).await?;
    let report = build_semester_report(&state.pool, student_id, semester, year).await?;

    Ok(Json(json!({
        "student_id": student_id,
        "student_name": student_name,
        "year": year,
        "semester": semester,
        "report": report
    })))
}

async fn course_performance(
    claims: Claims,
    State(state): State<AppState>,
    Path((course_id, year)): Path<(Uuid, i32)>,
) -> AcademicResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;

    let course_subjects: Vec<schoolccb_common::academic::CourseSubject> = sqlx::query_as(
        "SELECT id, course_id, subject_id, teacher_id, academic_year, hours_per_week FROM course_subjects WHERE course_id = $1 AND academic_year = $2",
    )
    .bind(course_id)
    .bind(year)
    .fetch_all(&state.pool)
    .await?;

    let students: Vec<(Uuid, String, String)> = sqlx::query_as(
        "SELECT s.id, s.first_name || ' ' || s.last_name, s.rut FROM students s
         JOIN enrollments e ON e.student_id = s.id
         WHERE e.course_id = $1 AND e.year = $2 AND e.active = true AND s.enrolled = true
         ORDER BY s.last_name, s.first_name",
    )
    .bind(course_id)
    .bind(year)
    .fetch_all(&state.pool)
    .await?;

    let mut subject_performance: Vec<Value> = vec![];

    for cs in &course_subjects {
        let subject_info: Option<(String, String)> =
            sqlx::query_as("SELECT code, name FROM subjects WHERE id = $1")
                .bind(cs.subject_id)
                .fetch_optional(&state.pool)
                .await?;

        let subject_code = subject_info
            .as_ref()
            .map(|s| s.0.clone())
            .unwrap_or_default();
        let subject_name = subject_info
            .as_ref()
            .map(|s| s.1.clone())
            .unwrap_or_default();

        let mut student_grades: Vec<Value> = vec![];
        for (sid, sname, srut) in &students {
            let grades: Vec<f64> = sqlx::query_scalar(
                "SELECT grade FROM grades WHERE student_id = $1 AND course_subject_id = $2 AND year = $3 ORDER BY date",
            )
            .bind(sid)
            .bind(cs.id)
            .bind(year)
            .fetch_all(&state.pool)
            .await?;

            let avg = if grades.is_empty() {
                0.0
            } else {
                grades.iter().sum::<f64>() / grades.len() as f64
            };
            student_grades.push(json!({
                "student_id": sid,
                "student_name": sname,
                "rut": srut,
                "grades": grades,
                "average": (avg * 10.0).round() / 10.0
            }));
        }

        subject_performance.push(json!({
            "subject_id": cs.subject_id,
            "subject_code": subject_code,
            "subject_name": subject_name,
            "teacher_id": cs.teacher_id,
            "students": student_grades
        }));
    }

    Ok(Json(json!({
        "course_id": course_id,
        "year": year,
        "subjects": subject_performance,
        "total_students": students.len()
    })))
}

async fn promotion_status(
    claims: Claims,
    State(state): State<AppState>,
    Path((course_id, year)): Path<(Uuid, i32)>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let students: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT s.id, s.first_name || ' ' || s.last_name FROM students s
         JOIN enrollments e ON e.student_id = s.id
         WHERE e.course_id = $1 AND e.year = $2 AND e.active = true AND s.enrolled = true
         ORDER BY s.last_name, s.first_name",
    )
    .bind(course_id)
    .bind(year)
    .fetch_all(&state.pool)
    .await?;

    let mut results: Vec<Value> = vec![];

    for (sid, sname) in &students {
        let s1 = build_semester_report(&state.pool, *sid, 1, year).await?;
        let s2 = build_semester_report(&state.pool, *sid, 2, year).await?;

        let empty_s1 = empty_semester(1);
        let s1_ref = s1.as_ref().unwrap_or(&empty_s1);
        let s2_ref = s2.as_ref();

        let failed_subjects: Vec<String> = if let Some(s2_report) = s2_ref {
            s2_report
                .subjects
                .iter()
                .filter(|s| s.weighted_average < 4.0)
                .map(|s| s.subject_name.clone())
                .collect()
        } else {
            s1_ref
                .subjects
                .iter()
                .filter(|s| s.weighted_average < 4.0)
                .map(|s| s.subject_name.clone())
                .collect()
        };

        let s1_avg = subject_average(&s1_ref.subjects);
        let s2_avg = s2_ref.map(|r| subject_average(&r.subjects));

        let promotion = match s2_avg {
            Some(s2a) => {
                let yearly = (s1_avg + s2a) / 2.0;
                evaluate_promotion_decreto67(&failed_subjects, yearly)
            }
            None => "Pendiente (S2)".to_string(),
        };

        results.push(json!({
            "student_id": sid,
            "student_name": sname,
            "semester1_average": (s1_avg * 10.0).round() / 10.0,
            "semester2_average": s2_avg.map(|a| (a * 10.0).round() / 10.0),
            "failed_subjects": failed_subjects,
            "promotion": promotion
        }));
    }

    let promoted = results
        .iter()
        .filter(|r| r["promotion"] == "Promovido")
        .count();
    let failed = results
        .iter()
        .filter(|r| r["promotion"] == "Reprobado")
        .count();
    let pending = results.len() - promoted - failed;

    Ok(Json(json!({
        "course_id": course_id,
        "year": year,
        "students": results,
        "summary": {
            "total": results.len(),
            "promovidos": promoted,
            "reprobados": failed,
            "pendientes": pending
        }
    })))
}

fn subject_average(subjects: &[schoolccb_common::academic::WeightedSubjectAverage]) -> f64 {
    if subjects.is_empty() {
        return 0.0;
    }
    let sum: f64 = subjects.iter().map(|s| s.weighted_average).sum();
    sum / subjects.len() as f64
}

fn evaluate_promotion_decreto67(failed_subjects: &[String], _yearly_avg: f64) -> String {
    match failed_subjects.len() {
        0 => "Promovido".to_string(),
        1 => "Promovido (Decreto 67 - 1 reprobada)".to_string(),
        2 => "Promovido (Decreto 67 - 2 reprobadas)".to_string(),
        _ => "Reprobado".to_string(),
    }
}

async fn build_semester_report(
    pool: &sqlx::PgPool,
    student_id: Uuid,
    semester: i32,
    year: i32,
) -> Result<Option<schoolccb_common::academic::SemesterReport>, sqlx::Error> {
    let course_subjects: Vec<(Uuid, Uuid)> = sqlx::query_as(
        r#"
        SELECT DISTINCT cs.id, cs.subject_id FROM course_subjects cs
        JOIN enrollments e ON e.course_id = cs.course_id
        WHERE e.student_id = $1 AND cs.academic_year = $2 AND e.active = true
        "#,
    )
    .bind(student_id)
    .bind(year)
    .fetch_all(pool)
    .await?;

    if course_subjects.is_empty() {
        return Ok(None);
    }

    let mut subjects: Vec<schoolccb_common::academic::WeightedSubjectAverage> = vec![];

    for (cs_id, subject_id) in &course_subjects {
        let subject_info: (String, String) =
            sqlx::query_as("SELECT code, name FROM subjects WHERE id = $1")
                .bind(subject_id)
                .fetch_one(pool)
                .await?;

        let categories: Vec<schoolccb_common::academic::GradeCategory> = sqlx::query_as(
            "SELECT id, course_subject_id, name, weight_percentage, evaluation_count FROM grade_categories WHERE course_subject_id = $1",
        )
        .bind(cs_id)
        .fetch_all(pool)
        .await?;

        let mut category_breakdowns: Vec<schoolccb_common::academic::CategoryBreakdown> = vec![];
        let mut all_grades: Vec<f64> = vec![];
        let mut min_grade = 7.0f64;
        let mut max_grade = 1.0f64;

        for cat in &categories {
            let cat_grades: Vec<f64> = sqlx::query_scalar(
                "SELECT grade FROM grades WHERE student_id = $1 AND course_subject_id = $2 AND category_id = $3 AND semester = $4 AND year = $5 ORDER BY date",
            )
            .bind(student_id)
            .bind(cs_id)
            .bind(cat.id)
            .bind(semester)
            .bind(year)
            .fetch_all(pool)
            .await?;

            let cat_avg = if cat_grades.is_empty() {
                0.0
            } else {
                cat_grades.iter().sum::<f64>() / cat_grades.len() as f64
            };

            for &g in &cat_grades {
                all_grades.push(g);
                if g < min_grade {
                    min_grade = g;
                }
                if g > max_grade {
                    max_grade = g;
                }
            }

            category_breakdowns.push(schoolccb_common::academic::CategoryBreakdown {
                category_name: cat.name.clone(),
                weight: cat.weight_percentage,
                grades: cat_grades,
                category_average: (cat_avg * 10.0).round() / 10.0,
                weighted_contribution: ((cat_avg * cat.weight_percentage / 100.0) * 10.0).round()
                    / 10.0,
            });
        }

        if categories.is_empty() {
            let raw_grades: Vec<f64> = sqlx::query_scalar(
                "SELECT grade FROM grades WHERE student_id = $1 AND course_subject_id = $2 AND semester = $3 AND year = $4 ORDER BY date",
            )
            .bind(student_id)
            .bind(cs_id)
            .bind(semester)
            .bind(year)
            .fetch_all(pool)
            .await?;

            for &g in &raw_grades {
                all_grades.push(g);
                if g < min_grade {
                    min_grade = g;
                }
                if g > max_grade {
                    max_grade = g;
                }
            }
        }

        let grades_count = all_grades.len() as i32;
        let simple_avg = if all_grades.is_empty() {
            0.0
        } else {
            all_grades.iter().sum::<f64>() / all_grades.len() as f64
        };

        let weighted_avg: f64 = if categories.is_empty() {
            simple_avg
        } else {
            let weighted_sum: f64 = category_breakdowns
                .iter()
                .map(|c| c.weighted_contribution)
                .sum();
            if weighted_sum == 0.0 {
                simple_avg
            } else {
                weighted_sum
            }
        };

        subjects.push(schoolccb_common::academic::WeightedSubjectAverage {
            subject_name: subject_info.1,
            subject_code: subject_info.0,
            categories: category_breakdowns,
            weighted_average: (weighted_avg * 10.0).round() / 10.0,
            simple_average: (simple_avg * 10.0).round() / 10.0,
            grades_count,
            min_grade: if grades_count == 0 { 0.0 } else { min_grade },
            max_grade: if grades_count == 0 { 0.0 } else { max_grade },
        });
    }

    let global_avg = subject_average(&subjects);
    let failed_subjects: Vec<&schoolccb_common::academic::WeightedSubjectAverage> = subjects
        .iter()
        .filter(|s| s.weighted_average < 4.0)
        .collect();
    let is_promoted = failed_subjects.is_empty();
    let has_min_grades = subjects.iter().all(|s| s.grades_count >= 2);

    Ok(Some(schoolccb_common::academic::SemesterReport {
        semester,
        subjects,
        global_average: (global_avg * 10.0).round() / 10.0,
        is_promoted,
        has_minimum_grades: has_min_grades,
    }))
}

fn empty_semester(semester: i32) -> schoolccb_common::academic::SemesterReport {
    schoolccb_common::academic::SemesterReport {
        semester,
        subjects: vec![],
        global_average: 0.0,
        is_promoted: false,
        has_minimum_grades: false,
    }
}

async fn get_student_name(pool: &sqlx::PgPool, student_id: Uuid) -> Result<String, sqlx::Error> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT first_name || ' ' || last_name FROM students WHERE id = $1")
            .bind(student_id)
            .fetch_optional(pool)
            .await?;

    Ok(row
        .map(|r| r.0)
        .unwrap_or_else(|| "Desconocido".to_string()))
}
