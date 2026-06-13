use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{AcademicError, AcademicResult};
use schoolccb_common::auth::{Claims, require_any_role};

#[derive(Deserialize)]
pub struct GradeFilter {
    pub student_id: Option<Uuid>,
    pub course_subject_id: Option<Uuid>,
    pub semester: Option<i32>,
    pub year: Option<i32>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/grades", get(list_grades).post(create_grade))
        .route(
            "/api/grades/{id}",
            get(get_grade).put(update_grade).delete(delete_grade),
        )
        .route("/api/grades/bulk", post(bulk_create_grades))
        .route(
            "/api/grades/course-subject/{course_subject_id}",
            get(grades_by_course_subject),
        )
        .route(
            "/api/grades/student/{student_id}/{semester}/{year}",
            get(student_grades),
        )
        .route(
            "/api/grades/by-subject/{subject_id}/{year}",
            get(grades_by_subject),
        )
        .route(
            "/api/grades/quick-test",
            post(quick_test),
        )
}

async fn list_grades(
    claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<GradeFilter>,
) -> AcademicResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "grades",
    )
    .await
    .map_err(|e| AcademicError::Forbidden(e))?;

    let school_id: Option<Uuid> = claims
        .school_id
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok());

    let corporation_id: Option<Uuid> = claims
        .corporation_id
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok());

    let grades = if let Some(sid) = filter.student_id {
        if let Some(csid) = filter.course_subject_id {
            if let Some(sem) = filter.semester {
                if let Some(y) = filter.year {
                    let (sql, corp_val) = if school_id.is_some() {
                        let corp_clause = corporation_id.map(|_| " AND sch.corporation_id = $6").unwrap_or("");
                        (format!("SELECT g.id, g.student_id, g.subject, g.grade, g.grade_type, g.semester, g.year, g.date, g.teacher_id, g.observation, g.category_id FROM grades g JOIN schools sch ON sch.id = g.school_id WHERE student_id = $1 AND course_subject_id = $2 AND semester = $3 AND year = $4 AND g.school_id = $5{} ORDER BY g.date DESC", corp_clause), corporation_id)
                    } else {
                        ("SELECT id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id FROM grades WHERE student_id = $1 AND course_subject_id = $2 AND semester = $3 AND year = $4 ORDER BY date DESC".to_string(), None)
                    };
                    let mut q = sqlx::query_as::<_, RawGrade>(&sql).bind(sid).bind(csid).bind(sem).bind(y);
                    if let Some(sc) = school_id { q = q.bind(sc); }
                    if let Some(cc) = corp_val { q = q.bind(cc); }
                    q.fetch_all(&state.pool).await?
                } else {
                    let (sql, corp_val) = if school_id.is_some() {
                        let corp_clause = corporation_id.map(|_| " AND sch.corporation_id = $5").unwrap_or("");
                        (format!("SELECT g.id, g.student_id, g.subject, g.grade, g.grade_type, g.semester, g.year, g.date, g.teacher_id, g.observation, g.category_id FROM grades g JOIN schools sch ON sch.id = g.school_id WHERE student_id = $1 AND course_subject_id = $2 AND semester = $3 AND g.school_id = $4{} ORDER BY g.date DESC", corp_clause), corporation_id)
                    } else {
                        ("SELECT id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id FROM grades WHERE student_id = $1 AND course_subject_id = $2 AND semester = $3 ORDER BY date DESC".to_string(), None)
                    };
                    let mut q = sqlx::query_as::<_, RawGrade>(&sql).bind(sid).bind(csid).bind(sem);
                    if let Some(sc) = school_id { q = q.bind(sc); }
                    if let Some(cc) = corp_val { q = q.bind(cc); }
                    q.fetch_all(&state.pool).await?
                }
            } else {
                let (sql, corp_val) = if school_id.is_some() {
                    let corp_clause = corporation_id.map(|_| " AND sch.corporation_id = $4").unwrap_or("");
                    (format!("SELECT g.id, g.student_id, g.subject, g.grade, g.grade_type, g.semester, g.year, g.date, g.teacher_id, g.observation, g.category_id FROM grades g JOIN schools sch ON sch.id = g.school_id WHERE student_id = $1 AND course_subject_id = $2 AND g.school_id = $3{} ORDER BY g.date DESC", corp_clause), corporation_id)
                } else {
                    ("SELECT id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id FROM grades WHERE student_id = $1 AND course_subject_id = $2 ORDER BY date DESC".to_string(), None)
                };
                let mut q = sqlx::query_as::<_, RawGrade>(&sql).bind(sid).bind(csid);
                if let Some(sc) = school_id { q = q.bind(sc); }
                if let Some(cc) = corp_val { q = q.bind(cc); }
                q.fetch_all(&state.pool).await?
            }
        } else {
            let (sql, corp_val) = if school_id.is_some() {
                let corp_clause = corporation_id.map(|_| " AND sch.corporation_id = $3").unwrap_or("");
                (format!("SELECT g.id, g.student_id, g.subject, g.grade, g.grade_type, g.semester, g.year, g.date, g.teacher_id, g.observation, g.category_id FROM grades g JOIN schools sch ON sch.id = g.school_id WHERE student_id = $1 AND g.school_id = $2{} ORDER BY g.date DESC", corp_clause), corporation_id)
            } else {
                ("SELECT id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id FROM grades WHERE student_id = $1 ORDER BY date DESC".to_string(), None)
            };
            let mut q = sqlx::query_as::<_, RawGrade>(&sql).bind(sid);
            if let Some(sc) = school_id { q = q.bind(sc); }
            if let Some(cc) = corp_val { q = q.bind(cc); }
            q.fetch_all(&state.pool).await?
        }
    } else {
        let (sql, corp_val) = if school_id.is_some() {
            let corp_clause = corporation_id.map(|_| " AND sch.corporation_id = $2").unwrap_or("");
            (format!("SELECT g.id, g.student_id, g.subject, g.grade, g.grade_type, g.semester, g.year, g.date, g.teacher_id, g.observation, g.category_id FROM grades g JOIN schools sch ON sch.id = g.school_id WHERE 1=1 AND g.school_id = $1{} ORDER BY g.date DESC", corp_clause), corporation_id)
        } else {
            ("SELECT id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id FROM grades WHERE 1=1 ORDER BY date DESC".to_string(), None)
        };
        let mut q = sqlx::query_as::<_, RawGrade>(&sql);
        if let Some(sc) = school_id { q = q.bind(sc); }
        if let Some(cc) = corp_val { q = q.bind(cc); }
        q.fetch_all(&state.pool).await?
    };

    Ok(Json(json!({ "grades": grades, "total": grades.len() })))
}

async fn get_grade(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AcademicResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;

    let grade = sqlx::query_as::<_, RawGrade>(
        "SELECT id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id FROM grades WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AcademicError::NotFound("Calificación no encontrada".into()))?;

    Ok(Json(json!({ "grade": grade })))
}

async fn create_grade(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::academic::CreateGradePayload>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor"])?;

    if !(1.0..=7.0).contains(&payload.grade) {
        return Err(AcademicError::Validation(
            "La nota debe estar entre 1.0 y 7.0".into(),
        ));
    }

    let subject_name = resolve_subject_name(&state.pool, payload.course_subject_id).await?;

    let id = Uuid::new_v4();
    let grade_val = (payload.grade * 10.0).round() / 10.0;

    let result = sqlx::query_as::<_, RawGrade>(
        r#"
        INSERT INTO grades (id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, course_subject_id, category_id, observation)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id
        "#,
    )
    .bind(id)
    .bind(payload.student_id)
    .bind(&subject_name)
    .bind(grade_val)
    .bind(&payload.grade_type)
    .bind(payload.semester)
    .bind(payload.year)
    .bind(payload.date)
    .bind(payload.teacher_id)
    .bind(payload.course_subject_id)
    .bind(payload.category_id)
    .bind(&payload.observation)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "grade": result })))
}

async fn update_grade(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::academic::UpdateGradePayload>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor"])?;

    let existing = sqlx::query_as::<_, RawGrade>(
        "SELECT id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id FROM grades WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AcademicError::NotFound("Calificación no encontrada".into()))?;

    let grade_val = match payload.grade {
        Some(g) => {
            if !(1.0..=7.0).contains(&g) {
                return Err(AcademicError::Validation(
                    "La nota debe estar entre 1.0 y 7.0".into(),
                ));
            }
            (g * 10.0).round() / 10.0
        }
        None => existing.grade,
    };
    let grade_type = payload.grade_type.unwrap_or(existing.grade_type);
    let category_id = payload.category_id.or(existing.category_id);
    let observation = payload.observation.or(existing.observation);

    let result = sqlx::query_as::<_, RawGrade>(
        r#"
        UPDATE grades SET grade = $1, grade_type = $2, category_id = $3, observation = $4
        WHERE id = $5
        RETURNING id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id
        "#,
    )
    .bind(grade_val)
    .bind(&grade_type)
    .bind(category_id)
    .bind(&observation)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "grade": result })))
}

async fn delete_grade(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP"])?;

    let result = sqlx::query("DELETE FROM grades WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AcademicError::NotFound("Calificación no encontrada".into()));
    }

    Ok(Json(
        json!({ "message": "Calificación eliminada correctamente" }),
    ))
}

async fn grades_by_course_subject(
    claims: Claims,
    State(state): State<AppState>,
    Path(course_subject_id): Path<Uuid>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor"])?;

    let grades = sqlx::query_as::<_, RawGrade>(
        "SELECT id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id FROM grades WHERE course_subject_id = $1 ORDER BY date DESC",
    )
    .bind(course_subject_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "grades": grades, "total": grades.len() })))
}

async fn bulk_create_grades(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::academic::BulkGradeEntry>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor"])?;

    if payload.grades.is_empty() {
        return Err(AcademicError::Validation(
            "Debe incluir al menos una calificación".into(),
        ));
    }

    let subject_name = resolve_subject_name(&state.pool, payload.course_subject_id).await?;

    let mut imported = 0;
    let mut errors: Vec<Value> = vec![];

    for entry in &payload.grades {
        if !(1.0..=7.0).contains(&entry.grade) {
            errors.push(json!({
                "student_id": entry.student_id,
                "error": "Nota fuera de rango (1.0 - 7.0)"
            }));
            continue;
        }

        let grade_val = (entry.grade * 10.0).round() / 10.0;
        let id = Uuid::new_v4();

        let result = sqlx::query(
            r#"
            INSERT INTO grades (id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, course_subject_id, category_id, observation)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(id)
        .bind(entry.student_id)
        .bind(&subject_name)
        .bind(grade_val)
        .bind(&payload.grade_type)
        .bind(payload.semester)
        .bind(payload.year)
        .bind(payload.date)
        .bind(payload.teacher_id)
        .bind(payload.course_subject_id)
        .bind(payload.category_id)
        .bind(&entry.observation)
        .execute(&state.pool)
        .await;

        match result {
            Ok(_) => imported += 1,
            Err(e) => {
                errors.push(json!({
                    "student_id": entry.student_id,
                    "error": e.to_string()
                }));
            }
        }
    }

    Ok(Json(json!({
        "imported": imported,
        "errors": errors,
        "total": payload.grades.len()
    })))
}

async fn student_grades(
    claims: Claims,
    State(state): State<AppState>,
    Path((student_id, semester, year)): Path<(Uuid, i32, i32)>,
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

    let grades = sqlx::query_as::<_, RawGrade>(
        "SELECT id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id FROM grades WHERE student_id = $1 AND semester = $2 AND year = $3 ORDER BY subject, date",
    )
    .bind(student_id)
    .bind(semester)
    .bind(year)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "grades": grades, "total": grades.len() })))
}

async fn grades_by_subject(
    claims: Claims,
    State(state): State<AppState>,
    Path((subject_id, year)): Path<(Uuid, i32)>,
) -> AcademicResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;

    let subject_info: (String, String) =
        sqlx::query_as("SELECT code, name FROM subjects WHERE id = $1 AND active = true")
            .bind(subject_id)
            .fetch_optional(&state.pool)
            .await?
            .ok_or(AcademicError::NotFound("Asignatura no encontrada".into()))?;

    let course_subjects: Vec<(Uuid, Uuid, String)> = sqlx::query_as(
        r#"
        SELECT cs.id, cs.course_id, c.name as course_name
        FROM course_subjects cs
        JOIN courses c ON c.id = cs.course_id
        WHERE cs.subject_id = $1 AND cs.academic_year = $2
        ORDER BY c.name
        "#,
    )
    .bind(subject_id)
    .bind(year)
    .fetch_all(&state.pool)
    .await?;

    let mut courses_data: Vec<Value> = vec![];

    for (cs_id, course_id, course_name) in &course_subjects {
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

        let mut students_data: Vec<Value> = vec![];

        for (sid, sname, srut) in &students {
            let grades_s1: Vec<f64> = sqlx::query_scalar(
                "SELECT grade FROM grades WHERE student_id = $1 AND course_subject_id = $2 AND semester = 1 AND year = $3 ORDER BY date",
            )
            .bind(sid)
            .bind(cs_id)
            .bind(year)
            .fetch_all(&state.pool)
            .await?;

            let grades_s2: Vec<f64> = sqlx::query_scalar(
                "SELECT grade FROM grades WHERE student_id = $1 AND course_subject_id = $2 AND semester = 2 AND year = $3 ORDER BY date",
            )
            .bind(sid)
            .bind(cs_id)
            .bind(year)
            .fetch_all(&state.pool)
            .await?;

            let avg_s1 = if grades_s1.is_empty() {
                0.0
            } else {
                grades_s1.iter().sum::<f64>() / grades_s1.len() as f64
            };
            let avg_s2 = if grades_s2.is_empty() {
                0.0
            } else {
                grades_s2.iter().sum::<f64>() / grades_s2.len() as f64
            };

            students_data.push(json!({
                "student_id": sid,
                "student_name": sname,
                "rut": srut,
                "grades_s1": grades_s1,
                "average_s1": (avg_s1 * 10.0).round() / 10.0,
                "grades_s2": grades_s2,
                "average_s2": (avg_s2 * 10.0).round() / 10.0,
                "final_average": ((avg_s1 + avg_s2) / 2.0 * 10.0).round() / 10.0,
            }));
        }

        courses_data.push(json!({
            "course_id": course_id,
            "course_name": course_name,
            "students": students_data,
            "total_students": students_data.len(),
        }));
    }

    Ok(Json(json!({
        "subject_id": subject_id,
        "subject_code": subject_info.0,
        "subject_name": subject_info.1,
        "year": year,
        "courses": courses_data,
        "total_courses": courses_data.len(),
    })))
}

async fn resolve_subject_name(
    pool: &sqlx::PgPool,
    course_subject_id: Uuid,
) -> Result<String, AcademicError> {
    let row: (String,) = sqlx::query_as(
        r#"
        SELECT s.name FROM subjects s
        JOIN course_subjects cs ON cs.subject_id = s.id
        WHERE cs.id = $1
        "#,
    )
    .bind(course_subject_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AcademicError::Validation(
        "La asignatura del curso no existe".into(),
    ))?;

    Ok(row.0)
}

async fn quick_test(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor"])?;

    let student_id = payload.get("student_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AcademicError::Validation("student_id requerido".into()))?;
    let course_subject_id = payload.get("course_subject_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AcademicError::Validation("course_subject_id requerido".into()))?;
    let grade_val = payload.get("grade").and_then(|v| v.as_f64())
        .ok_or(AcademicError::Validation("grade requerido".into()))?;
    let teacher_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AcademicError::Unauthorized)?;

    let subject = resolve_subject_name(&state.pool, course_subject_id).await?;
    let id = Uuid::new_v4();
    let today = chrono::Utc::now().date_naive();

    sqlx::query(
        "INSERT INTO grades (id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation)
         VALUES ($1, $2, $3, $4, 'ControlSorpresa', 0, $5, $6, $7, $8)",
    )
    .bind(id)
    .bind(student_id)
    .bind(&subject)
    .bind(grade_val)
    .bind(today.year())
    .bind(today)
    .bind(teacher_id)
    .bind(payload.get("observation").and_then(|v| v.as_str()))
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"id": id, "grade": grade_val, "grade_type": "ControlSorpresa", "subject": subject})))
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
struct RawGrade {
    pub id: Uuid,
    pub student_id: Uuid,
    pub subject: String,
    pub grade: f64,
    pub grade_type: String,
    pub semester: i32,
    pub year: i32,
    pub date: chrono::NaiveDate,
    pub teacher_id: Uuid,
    pub observation: Option<String>,
    pub category_id: Option<Uuid>,
}
