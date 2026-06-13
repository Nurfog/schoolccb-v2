use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{AcademicError, AcademicResult};
use schoolccb_common::auth::{Claims, require_any_role};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/grades/categories",
            get(list_categories).post(create_category),
        )
        .route(
            "/api/grades/categories/{id}",
            get(get_category)
                .put(update_category)
                .delete(delete_category),
        )
        .route(
            "/api/grades/course-subjects/{course_subject_id}/categories",
            get(categories_by_course_subject),
        )
}

async fn list_categories(
    claims: Claims,
    State(state): State<AppState>,
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

    let categories = sqlx::query_as::<_, schoolccb_common::academic::GradeCategory>(
        "SELECT id, course_subject_id, name, weight_percentage, evaluation_count FROM grade_categories ORDER BY name",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "categories": categories })))
}

async fn get_category(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AcademicResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;

    let category = sqlx::query_as::<_, schoolccb_common::academic::GradeCategory>(
        "SELECT id, course_subject_id, name, weight_percentage, evaluation_count FROM grade_categories WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AcademicError::NotFound("Categoría no encontrada".into()))?;

    Ok(Json(json!({ "category": category })))
}

async fn categories_by_course_subject(
    claims: Claims,
    State(state): State<AppState>,
    Path(course_subject_id): Path<Uuid>,
) -> AcademicResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;

    let categories = sqlx::query_as::<_, schoolccb_common::academic::GradeCategory>(
        "SELECT id, course_subject_id, name, weight_percentage, evaluation_count FROM grade_categories WHERE course_subject_id = $1 ORDER BY name",
    )
    .bind(course_subject_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "categories": categories })))
}

async fn create_category(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::academic::CreateCategoryPayload>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor"])?;

    if payload.name.trim().is_empty() {
        return Err(AcademicError::Validation(
            "El nombre de la categoría es obligatorio".into(),
        ));
    }
    if payload.weight_percentage <= 0.0 || payload.weight_percentage > 100.0 {
        return Err(AcademicError::Validation(
            "El porcentaje debe estar entre 0 y 100".into(),
        ));
    }

    let existing: Vec<schoolccb_common::academic::GradeCategory> = sqlx::query_as(
        "SELECT id, course_subject_id, name, weight_percentage, evaluation_count FROM grade_categories WHERE course_subject_id = $1",
    )
    .bind(payload.course_subject_id)
    .fetch_all(&state.pool)
    .await?;

    let total_weight: f64 = existing.iter().map(|c| c.weight_percentage).sum();
    if total_weight + payload.weight_percentage > 100.0 {
        return Err(AcademicError::Validation(format!(
            "La suma de porcentajes superaría 100% (actual: {}%, nuevo: {}%)",
            total_weight, payload.weight_percentage
        )));
    }

    let id = Uuid::new_v4();
    let result = sqlx::query_as::<_, schoolccb_common::academic::GradeCategory>(
        r#"
        INSERT INTO grade_categories (id, course_subject_id, name, weight_percentage, evaluation_count)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, course_subject_id, name, weight_percentage, evaluation_count
        "#,
    )
    .bind(id)
    .bind(payload.course_subject_id)
    .bind(&payload.name)
    .bind(payload.weight_percentage)
    .bind(payload.evaluation_count.unwrap_or(0))
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "category": result })))
}

async fn update_category(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::academic::UpdateCategoryPayload>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor"])?;

    let existing = sqlx::query_as::<_, schoolccb_common::academic::GradeCategory>(
        "SELECT id, course_subject_id, name, weight_percentage, evaluation_count FROM grade_categories WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AcademicError::NotFound("Categoría no encontrada".into()))?;

    let name = payload.name.unwrap_or(existing.name);
    let weight_percentage = payload
        .weight_percentage
        .unwrap_or(existing.weight_percentage);
    let evaluation_count = payload
        .evaluation_count
        .unwrap_or(existing.evaluation_count);

    let result = sqlx::query_as::<_, schoolccb_common::academic::GradeCategory>(
        r#"
        UPDATE grade_categories SET name = $1, weight_percentage = $2, evaluation_count = $3
        WHERE id = $4
        RETURNING id, course_subject_id, name, weight_percentage, evaluation_count
        "#,
    )
    .bind(&name)
    .bind(weight_percentage)
    .bind(evaluation_count)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "category": result })))
}

async fn delete_category(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP"])?;

    let result = sqlx::query("DELETE FROM grade_categories WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AcademicError::NotFound("Categoría no encontrada".into()));
    }

    Ok(Json(
        json!({ "message": "Categoría eliminada correctamente" }),
    ))
}
