use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use schoolccb_common::auth::{Claims, require_any_role};
use crate::AppState;
use crate::error::{SisError, SisResult};

#[derive(Debug, Serialize, sqlx::FromRow)]
struct RawEnrollment {
    id: Uuid,
    student_id: Uuid,
    course_id: Uuid,
    year: i32,
    active: bool,
}

#[derive(Deserialize)]
struct EnrollmentQuery {
    student_id: Option<Uuid>,
    course_id: Option<Uuid>,
    year: Option<i32>,
    active: Option<bool>,
}

#[derive(Deserialize)]
struct CreateEnrollmentPayload {
    student_id: Uuid,
    course_id: Uuid,
    year: i32,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/enrollments",
            get(list_enrollments).post(create_enrollment),
        )
        .route("/api/enrollments/{id}", delete(delete_enrollment))
}

async fn list_enrollments(
    claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<EnrollmentQuery>,
) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Sostenedor", "Administrador", "Director", "UTP", "Profesor"],
    )?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "enrollments",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;

    let mut conditions = Vec::new();
    let mut bind_values: Vec<String> = vec![];

    if let Some(ref sid) = claims.school_id {
        conditions.push(format!("school_id = ${}::uuid", conditions.len() + 1));
        bind_values.push(sid.clone());
    }
    if let Some(sid) = q.student_id {
        conditions.push(format!("student_id = ${}", conditions.len() + 1));
        bind_values.push(sid.to_string());
    }
    if let Some(cid) = q.course_id {
        conditions.push(format!("course_id = ${}", conditions.len() + 1));
        bind_values.push(cid.to_string());
    }
    if let Some(y) = q.year {
        conditions.push(format!("year = ${}", conditions.len() + 1));
        bind_values.push(y.to_string());
    }
    if let Some(a) = q.active {
        conditions.push(format!("active = ${}", conditions.len() + 1));
        bind_values.push(a.to_string());
    }

    let sql = if conditions.is_empty() {
        "SELECT id, student_id, course_id, year, active FROM enrollments ORDER BY year DESC, student_id".to_string()
    } else {
        format!(
            "SELECT id, student_id, course_id, year, active FROM enrollments WHERE {} ORDER BY year DESC, student_id",
            conditions.join(" AND ")
        )
    };

    let mut query = sqlx::query_as::<_, RawEnrollment>(&sql);
    for val in &bind_values {
        query = query.bind(val);
    }

    let enrollments = query.fetch_all(&state.pool).await?;
    Ok(Json(json!({ "enrollments": enrollments })))
}

async fn create_enrollment(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateEnrollmentPayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador", "Director", "UTP"])?;

    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM enrollments WHERE student_id = $1 AND course_id = $2 AND year = $3",
    )
    .bind(payload.student_id)
    .bind(payload.course_id)
    .bind(payload.year)
    .fetch_one(&state.pool)
    .await?;

    if exists > 0 {
        return Err(SisError::Conflict(
            "El alumno ya está matriculado en este curso y año".into(),
        ));
    }

    let school_id = claims.school_id.and_then(|s| Uuid::parse_str(&s).ok());

    let enrollment = sqlx::query_as::<_, RawEnrollment>(
        "INSERT INTO enrollments (student_id, course_id, year, school_id)
         VALUES ($1, $2, $3, $4)
         RETURNING id, student_id, course_id, year, active",
    )
    .bind(payload.student_id)
    .bind(payload.course_id)
    .bind(payload.year)
    .bind(school_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "enrollment": enrollment })))
}

async fn delete_enrollment(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador"])?;

    let result = sqlx::query("DELETE FROM enrollments WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(SisError::NotFound("Matrícula no encontrada".into()));
    }

    Ok(Json(
        json!({ "message": "Matrícula eliminada correctamente" }),
    ))
}
