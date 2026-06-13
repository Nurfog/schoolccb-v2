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
            "/api/academic/grade-levels",
            get(list_levels).post(create_level),
        )
        .route(
            "/api/academic/grade-levels/{id}",
            get(get_level).put(update_level).delete(delete_level),
        )
}

pub async fn seed_grade_levels(pool: &sqlx::PgPool) {
    let default_levels = vec![
        ("SALA_CUNA", "Sala Cuna", None, 0),
        ("MEDIO_MENOR", "Medio Menor", None, 1),
        ("MEDIO_MAYOR", "Medio Mayor", None, 2),
        ("PREKINDER", "Pre-kinder", None, 3),
        ("KINDER", "Kinder", None, 4),
        ("1_BASICO", "1° Básico", None, 5),
        ("2_BASICO", "2° Básico", None, 6),
        ("3_BASICO", "3° Básico", None, 7),
        ("4_BASICO", "4° Básico", None, 8),
        ("5_BASICO", "5° Básico", None, 9),
        ("6_BASICO", "6° Básico", None, 10),
        ("7_BASICO", "7° Básico", None, 11),
        ("8_BASICO", "8° Básico", None, 12),
        ("1_MEDIO", "1° Medio", None, 13),
        ("2_MEDIO", "2° Medio", None, 14),
        ("3_MEDIO_HC", "3° Medio HC", Some("HC"), 15),
        ("4_MEDIO_HC", "4° Medio HC", Some("HC"), 16),
        ("3_MEDIO_TP", "3° Medio TP", Some("TP"), 17),
        ("4_MEDIO_TP", "4° Medio TP", Some("TP"), 18),
        ("3_MEDIO_ART", "3° Medio Artístico", Some("Artístico"), 19),
        ("4_MEDIO_ART", "4° Medio Artístico", Some("Artístico"), 20),
    ];

    for (code, name, plan, order) in default_levels {
        let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM grade_levels WHERE code = $1")
            .bind(code)
            .fetch_one(pool)
            .await
            .unwrap_or((0,));

        if exists.0 == 0 {
            let id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO grade_levels (id, code, name, plan, sort_order, active)
                 VALUES ($1, $2, $3, $4, $5, true)",
            )
            .bind(id)
            .bind(code)
            .bind(name)
            .bind(plan)
            .bind(order)
            .execute(pool)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("Could not seed grade level {code}: {e}");
                Default::default()
            });
            tracing::info!("Seeded grade level: {code} - {name}");
        }
    }
}

async fn list_levels(claims: Claims, State(state): State<AppState>) -> AcademicResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "academic",
    )
    .await
    .map_err(|e| AcademicError::Forbidden(e))?;

    let levels = sqlx::query_as::<_, schoolccb_common::academic::GradeLevel>(
        "SELECT id, code, name, plan, sort_order, active, created_at FROM grade_levels ORDER BY sort_order",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "levels": levels })))
}

async fn get_level(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let level = sqlx::query_as::<_, schoolccb_common::academic::GradeLevel>(
        "SELECT id, code, name, plan, sort_order, active, created_at FROM grade_levels WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AcademicError::NotFound("Nivel no encontrado".into()))?;

    Ok(Json(json!({ "level": level })))
}

async fn create_level(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::academic::CreateGradeLevelPayload>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    if payload.code.trim().is_empty() || payload.name.trim().is_empty() {
        return Err(AcademicError::Validation(
            "Código y nombre son obligatorios".into(),
        ));
    }

    let id = Uuid::new_v4();
    let result = sqlx::query_as::<_, schoolccb_common::academic::GradeLevel>(
        r#"
        INSERT INTO grade_levels (id, code, name, plan, sort_order, active)
        VALUES ($1, $2, $3, $4, $5, true)
        RETURNING id, code, name, plan, sort_order, active, created_at
        "#,
    )
    .bind(id)
    .bind(&payload.code)
    .bind(&payload.name)
    .bind(&payload.plan)
    .bind(payload.sort_order.unwrap_or(0))
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "level": result })))
}

async fn update_level(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::academic::UpdateGradeLevelPayload>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    let current = sqlx::query_as::<_, schoolccb_common::academic::GradeLevel>(
        "SELECT id, code, name, plan, sort_order, active, created_at FROM grade_levels WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AcademicError::NotFound("Nivel no encontrado".into()))?;

    let name = payload.name.unwrap_or(current.name);
    let plan = payload.plan.or(current.plan);
    let sort_order = payload.sort_order.unwrap_or(current.sort_order);
    let active = payload.active.unwrap_or(current.active);

    let result = sqlx::query_as::<_, schoolccb_common::academic::GradeLevel>(
        "UPDATE grade_levels SET name = $1, plan = $2, sort_order = $3, active = $4 WHERE id = $5
         RETURNING id, code, name, plan, sort_order, active, created_at",
    )
    .bind(&name)
    .bind(&plan)
    .bind(sort_order)
    .bind(active)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "level": result })))
}

async fn delete_level(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    sqlx::query("DELETE FROM grade_levels WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({ "message": "Nivel eliminado correctamente" })))
}
