use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{AcademicError, AcademicResult};
use schoolccb_common::auth::{Claims, require_any_role};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/academic-years", get(list_years).post(create_year))
        .route(
            "/api/academic-years/{id}",
            get(get_year).put(update_year).delete(delete_year),
        )
        .route("/api/academic-years/{id}/activate", post(activate_year))
        .route("/api/academic-years/clone", post(clone_year))
}

async fn list_years(claims: Claims, State(state): State<AppState>) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "academic-years",
    )
    .await
    .map_err(|e| AcademicError::Forbidden(e))?;

    let years = sqlx::query_as::<_, schoolccb_common::academic::AcademicYear>(
        "SELECT id, year, name, is_active, created_at FROM academic_years ORDER BY year DESC",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "years": years, "total": years.len() })))
}

async fn get_year(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let year = sqlx::query_as::<_, schoolccb_common::academic::AcademicYear>(
        "SELECT id, year, name, is_active, created_at FROM academic_years WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AcademicError::NotFound(
        "Año académico no encontrado".into(),
    ))?;

    Ok(Json(json!({ "year": year })))
}

async fn create_year(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::academic::CreateAcademicYearPayload>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    if payload.name.trim().is_empty() {
        return Err(AcademicError::Validation("El nombre es obligatorio".into()));
    }

    if payload.is_active == Some(true) {
        sqlx::query("UPDATE academic_years SET is_active = false")
            .execute(&state.pool)
            .await?;
    }

    let id = Uuid::new_v4();
    let result = sqlx::query_as::<_, schoolccb_common::academic::AcademicYear>(
        r#"
        INSERT INTO academic_years (id, year, name, is_active)
        VALUES ($1, $2, $3, $4)
        RETURNING id, year, name, is_active, created_at
        "#,
    )
    .bind(id)
    .bind(payload.year)
    .bind(&payload.name)
    .bind(payload.is_active.unwrap_or(false))
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "year": result })))
}

async fn update_year(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::academic::UpdateAcademicYearPayload>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    let current = sqlx::query_as::<_, schoolccb_common::academic::AcademicYear>(
        "SELECT id, year, name, is_active, created_at FROM academic_years WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AcademicError::NotFound(
        "Año académico no encontrado".into(),
    ))?;

    if payload.is_active == Some(true) {
        sqlx::query("UPDATE academic_years SET is_active = false")
            .execute(&state.pool)
            .await?;
    }

    let name = payload.name.unwrap_or(current.name);
    let is_active = payload.is_active.unwrap_or(current.is_active);

    let result = sqlx::query_as::<_, schoolccb_common::academic::AcademicYear>(
        "UPDATE academic_years SET name = $1, is_active = $2 WHERE id = $3
         RETURNING id, year, name, is_active, created_at",
    )
    .bind(&name)
    .bind(is_active)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "year": result })))
}

async fn delete_year(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    sqlx::query("DELETE FROM academic_years WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({ "message": "Año académico eliminado" })))
}

async fn activate_year(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    sqlx::query("UPDATE academic_years SET is_active = false")
        .execute(&state.pool)
        .await?;

    let result = sqlx::query_as::<_, schoolccb_common::academic::AcademicYear>(
        "UPDATE academic_years SET is_active = true WHERE id = $1
         RETURNING id, year, name, is_active, created_at",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "year": result })))
}

async fn clone_year(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::academic::CloneYearPayload>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    if payload.from_year == payload.to_year {
        return Err(AcademicError::Validation(
            "Los años deben ser diferentes".into(),
        ));
    }

    // Verify from_year exists
    let from_exists: Option<(i32,)> =
        sqlx::query_as("SELECT year FROM academic_years WHERE year = $1")
            .bind(payload.from_year)
            .fetch_optional(&state.pool)
            .await?;

    if from_exists.is_none() {
        return Err(AcademicError::NotFound(
            format!("El año origen {} no está configurado", payload.from_year).into(),
        ));
    }

    // Ensure to_year academic_year record exists
    let to_exists: Option<(i32,)> =
        sqlx::query_as("SELECT year FROM academic_years WHERE year = $1")
            .bind(payload.to_year)
            .fetch_optional(&state.pool)
            .await?;

    if to_exists.is_none() {
        let id = Uuid::new_v4();
        let name = payload
            .to_year_name
            .clone()
            .unwrap_or_else(|| format!("Año Escolar {}", payload.to_year));
        sqlx::query(
            "INSERT INTO academic_years (id, year, name, is_active) VALUES ($1, $2, $3, false)",
        )
        .bind(id)
        .bind(payload.to_year)
        .bind(&name)
        .execute(&state.pool)
        .await?;
    }

    // Clone course_subjects: copy all from from_year to to_year
    let cs_to_clone: Vec<(Uuid, Uuid, Uuid, i32)> = sqlx::query_as(
        r#"
        SELECT DISTINCT cs.course_id, cs.subject_id, cs.teacher_id, cs.hours_per_week
        FROM course_subjects cs
        WHERE cs.academic_year = $1
        "#,
    )
    .bind(payload.from_year)
    .fetch_all(&state.pool)
    .await?;

    let mut cloned = 0u32;
    let mut skipped = 0u32;

    for (course_id, subject_id, teacher_id, hours_per_week) in &cs_to_clone {
        let exists: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM course_subjects WHERE course_id = $1 AND subject_id = $2 AND academic_year = $3",
        )
        .bind(course_id)
        .bind(subject_id)
        .bind(payload.to_year)
        .fetch_one(&state.pool)
        .await?;

        if exists.0 > 0 {
            skipped += 1;
            continue;
        }

        let new_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO course_subjects (id, course_id, subject_id, teacher_id, academic_year, hours_per_week)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(new_id)
        .bind(course_id)
        .bind(subject_id)
        .bind(teacher_id)
        .bind(payload.to_year)
        .bind(hours_per_week)
        .execute(&state.pool)
        .await?;

        cloned += 1;
    }

    Ok(Json(json!({
        "message": format!("Clonación completada: {} cursos-asignaturas creados, {} omitidos (ya existían)", cloned, skipped),
        "from_year": payload.from_year,
        "to_year": payload.to_year,
        "cloned": cloned,
        "skipped": skipped,
    })))
}
