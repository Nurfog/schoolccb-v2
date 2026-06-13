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
        .route("/api/grades/periods", get(list_periods).post(create_period))
        .route(
            "/api/grades/periods/{id}",
            get(get_period).put(update_period),
        )
        .route("/api/grades/periods/current", get(current_period))
}

async fn list_periods(
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

    let periods = sqlx::query_as::<_, schoolccb_common::academic::AcademicPeriod>(
        "SELECT id, name, year, semester, start_date, end_date, is_active FROM academic_periods ORDER BY year DESC, semester",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "periods": periods })))
}

async fn get_period(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AcademicResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;

    let period = sqlx::query_as::<_, schoolccb_common::academic::AcademicPeriod>(
        "SELECT id, name, year, semester, start_date, end_date, is_active FROM academic_periods WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AcademicError::NotFound("Periodo no encontrado".into()))?;

    Ok(Json(json!({ "period": period })))
}

async fn current_period(
    claims: Claims,
    State(state): State<AppState>,
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

    let period = sqlx::query_as::<_, schoolccb_common::academic::AcademicPeriod>(
        "SELECT id, name, year, semester, start_date, end_date, is_active FROM academic_periods WHERE is_active = true LIMIT 1",
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AcademicError::NotFound("No hay un periodo activo configurado".into()))?;

    Ok(Json(json!({ "period": period })))
}

async fn create_period(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::academic::CreatePeriodPayload>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP"])?;

    if payload.name.trim().is_empty() {
        return Err(AcademicError::Validation(
            "El nombre del periodo es obligatorio".into(),
        ));
    }
    if payload.start_date >= payload.end_date {
        return Err(AcademicError::Validation(
            "La fecha de inicio debe ser anterior a la fecha de término".into(),
        ));
    }

    let id = Uuid::new_v4();
    let result = sqlx::query_as::<_, schoolccb_common::academic::AcademicPeriod>(
        r#"
        INSERT INTO academic_periods (id, name, year, semester, start_date, end_date)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, name, year, semester, start_date, end_date, is_active
        "#,
    )
    .bind(id)
    .bind(&payload.name)
    .bind(payload.year)
    .bind(payload.semester)
    .bind(payload.start_date)
    .bind(payload.end_date)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "period": result })))
}

async fn update_period(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::academic::UpdatePeriodPayload>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP"])?;

    let existing = sqlx::query_as::<_, schoolccb_common::academic::AcademicPeriod>(
        "SELECT id, name, year, semester, start_date, end_date, is_active FROM academic_periods WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AcademicError::NotFound("Periodo no encontrado".into()))?;

    let name = payload.name.unwrap_or(existing.name);
    let start_date = payload.start_date.unwrap_or(existing.start_date);
    let end_date = payload.end_date.unwrap_or(existing.end_date);

    if start_date >= end_date {
        return Err(AcademicError::Validation(
            "La fecha de inicio debe ser anterior a la fecha de término".into(),
        ));
    }

    if payload.is_active == Some(true) {
        sqlx::query("UPDATE academic_periods SET is_active = false WHERE is_active = true")
            .execute(&state.pool)
            .await?;
    }

    let is_active = payload.is_active.unwrap_or(existing.is_active);

    let result = sqlx::query_as::<_, schoolccb_common::academic::AcademicPeriod>(
        r#"
        UPDATE academic_periods SET name = $1, start_date = $2, end_date = $3, is_active = $4
        WHERE id = $5
        RETURNING id, name, year, semester, start_date, end_date, is_active
        "#,
    )
    .bind(&name)
    .bind(start_date)
    .bind(end_date)
    .bind(is_active)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "period": result })))
}
