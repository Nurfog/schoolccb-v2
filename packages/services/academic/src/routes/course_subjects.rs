use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use schoolccb_common::auth::{Claims, require_any_role};
use crate::AppState;
use crate::error::{AcademicError, AcademicResult};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/grades/course-subjects/{course_id}/{year}",
            get(list_course_subjects),
        )
        .route("/api/grades/course-subjects", post(assign_course_subject))
        .route(
            "/api/grades/course-subjects/{id}",
            delete(remove_course_subject),
        )
}

#[derive(sqlx::FromRow, Serialize)]
struct CourseSubjectRow {
    id: Uuid,
    course_id: Uuid,
    subject_id: Uuid,
    teacher_id: Uuid,
    academic_year: i32,
    hours_per_week: i32,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
struct AssignPayload {
    course_id: Uuid,
    subject_id: Uuid,
    teacher_id: Uuid,
    academic_year: i32,
    hours_per_week: Option<i32>,
}

async fn list_course_subjects(
    claims: Claims,
    State(state): State<AppState>,
    Path((course_id, year)): Path<(Uuid, i32)>,
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

    let rows = sqlx::query_as::<_, CourseSubjectRow>(
        r#"SELECT cs.id, cs.course_id, cs.subject_id, cs.teacher_id,
                  cs.academic_year, cs.hours_per_week, cs.created_at
           FROM course_subjects cs
           WHERE cs.course_id = $1 AND cs.academic_year = $2
           ORDER BY cs.created_at"#,
    )
    .bind(course_id)
    .bind(year)
    .fetch_all(&state.pool)
    .await?;

    let mut result = Vec::new();
    for r in rows {
        let subject =
            sqlx::query_as::<_, (String, String)>("SELECT code, name FROM subjects WHERE id = $1")
                .bind(r.subject_id)
                .fetch_optional(&state.pool)
                .await?;

        let teacher_name: Option<String> =
            sqlx::query_scalar("SELECT name FROM users WHERE id = $1")
                .bind(r.teacher_id)
                .fetch_optional(&state.pool)
                .await?;

        result.push(json!({
            "id": r.id,
            "course_id": r.course_id,
            "subject_id": r.subject_id,
            "subject_code": subject.as_ref().map(|s| s.0.clone()),
            "subject_name": subject.as_ref().map(|s| s.1.clone()),
            "teacher_id": r.teacher_id,
            "teacher_name": teacher_name,
            "academic_year": r.academic_year,
            "hours_per_week": r.hours_per_week,
        }));
    }

    Ok(Json(json!({ "course_subjects": result })))
}

async fn assign_course_subject(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<AssignPayload>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let id = Uuid::new_v4();
    let hours = payload.hours_per_week.unwrap_or(0);

    let result = sqlx::query_as::<_, CourseSubjectRow>(
        r#"INSERT INTO course_subjects (id, course_id, subject_id, teacher_id, academic_year, hours_per_week)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, course_id, subject_id, teacher_id, academic_year, hours_per_week, created_at"#,
    )
    .bind(id)
    .bind(payload.course_id)
    .bind(payload.subject_id)
    .bind(payload.teacher_id)
    .bind(payload.academic_year)
    .bind(hours)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint() == Some("course_subjects_course_id_subject_id_academic_year_key") {
                return crate::error::AcademicError::Conflict(
                    "Esta asignatura ya está asignada al curso para el año académico".into(),
                );
            }
        }
        crate::error::AcademicError::Database(e)
    })?;

    let user_id = Uuid::parse_str(&claims.sub).ok();
    schoolccb_common::audit::log(
        &state.pool,
        &schoolccb_common::audit::AuditEntry {
            entity_type: "course_subject".into(),
            entity_id: id,
            action: "assigned".into(),
            user_id,
            changes: Some(serde_json::json!({
                "course_id": payload.course_id,
                "subject_id": payload.subject_id,
                "teacher_id": payload.teacher_id,
                "academic_year": payload.academic_year,
                "hours_per_week": hours,
            })),
        },
    )
    .await;

    Ok(Json(json!({ "course_subject": result })))
}

async fn remove_course_subject(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let existing = sqlx::query_as::<_, CourseSubjectRow>(
        r#"SELECT id, course_id, subject_id, teacher_id, academic_year, hours_per_week, created_at
           FROM course_subjects WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(crate::error::AcademicError::NotFound(
        "Asignación de asignatura no encontrada".into(),
    ))?;

    sqlx::query("DELETE FROM course_subjects WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    let user_id = Uuid::parse_str(&claims.sub).ok();
    schoolccb_common::audit::log(
        &state.pool,
        &schoolccb_common::audit::AuditEntry {
            entity_type: "course_subject".into(),
            entity_id: id,
            action: "removed".into(),
            user_id,
            changes: Some(serde_json::json!({
                "course_id": existing.course_id,
                "subject_id": existing.subject_id,
                "academic_year": existing.academic_year,
            })),
        },
    )
    .await;

    Ok(Json(
        json!({ "message": "Asignatura removida del curso correctamente" }),
    ))
}
