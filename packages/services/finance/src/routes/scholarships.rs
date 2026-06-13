use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::FinanceResult;
use schoolccb_common::auth::{Claims, require_any_role};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/finance/student_scholarships",
            get(list_student_scholarships).post(create_scholarship),
        )
        .route(
            "/api/finance/student_scholarships/{id}",
            get(get_scholarship)
                .put(approve_scholarship)
                .delete(delete_scholarship),
        )
        .route(
            "/api/finance/student_scholarships/student/{student_id}",
            get(student_scholarships_by_student),
        )
}

async fn list_student_scholarships(
    claims: Claims,
    State(state): State<AppState>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let student_scholarships = sqlx::query_as::<_, schoolccb_common::finance::Scholarship>(
        "SELECT id, student_id, name, discount_percentage, approved, approved_by, valid_from, valid_until, created_at FROM student_scholarships ORDER BY name",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(
        json!({ "student_scholarships": student_scholarships, "total": student_scholarships.len() }),
    ))
}

async fn get_scholarship(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let scholarship = sqlx::query_as::<_, schoolccb_common::finance::Scholarship>(
        "SELECT id, student_id, name, discount_percentage, approved, approved_by, valid_from, valid_until, created_at FROM student_scholarships WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(crate::error::FinanceError::NotFound("Beca no encontrada".into()))?;

    Ok(Json(json!({ "scholarship": scholarship })))
}

async fn create_scholarship(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::finance::CreateScholarshipPayload>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    if payload.name.trim().is_empty()
        || payload.discount_percentage <= 0.0
        || payload.discount_percentage > 100.0
    {
        return Err(crate::error::FinanceError::Validation(
            "Nombre y porcentaje válido (1-100) son obligatorios".into(),
        ));
    }

    let id = Uuid::new_v4();
    let result = sqlx::query_as::<_, schoolccb_common::finance::Scholarship>(
        r#"
        INSERT INTO student_scholarships (id, student_id, name, discount_percentage, valid_from, valid_until)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, student_id, name, discount_percentage, approved, approved_by, valid_from, valid_until, created_at
        "#,
    )
    .bind(id)
    .bind(payload.student_id)
    .bind(&payload.name)
    .bind(payload.discount_percentage)
    .bind(payload.valid_from)
    .bind(payload.valid_until)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "scholarship": result })))
}

async fn approve_scholarship(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    let approver_id: Uuid = claims
        .sub
        .parse()
        .map_err(|_| crate::error::FinanceError::Unauthorized)?;

    let result = sqlx::query_as::<_, schoolccb_common::finance::Scholarship>(
        r#"
        UPDATE student_scholarships SET approved = true, approved_by = $1 WHERE id = $2
        RETURNING id, student_id, name, discount_percentage, approved, approved_by, valid_from, valid_until, created_at
        "#,
    )
    .bind(approver_id)
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(crate::error::FinanceError::NotFound("Beca no encontrada".into()))?;

    Ok(Json(json!({ "scholarship": result })))
}

async fn delete_scholarship(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    sqlx::query("DELETE FROM student_scholarships WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({ "message": "Beca eliminada correctamente" })))
}

async fn student_scholarships_by_student(
    claims: Claims,
    State(state): State<AppState>,
    Path(student_id): Path<Uuid>,
) -> FinanceResult<Json<Value>> {
    require_any_role(
        &claims,
        &[
            "Administrador",
            "Sostenedor",
            "Director",
            "UTP",
            "Apoderado",
        ],
    )?;

    let student_scholarships = sqlx::query_as::<_, schoolccb_common::finance::Scholarship>(
        "SELECT id, student_id, name, discount_percentage, approved, approved_by, valid_from, valid_until, created_at FROM student_scholarships WHERE student_id = $1 ORDER BY valid_from DESC",
    )
    .bind(student_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "student_scholarships": student_scholarships })))
}
