use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{SisError, SisResult};
use schoolccb_common::auth::{Claims, require_any_role};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/admission/stages", get(list_stages).post(create_stage))
        .route(
            "/api/admission/stages/{id}",
            get(get_stage).put(update_stage).delete(delete_stage),
        )
}

pub async fn seed_pipeline_stages(pool: &sqlx::PgPool) {
    let stages = vec![
        ("Primer Contacto", 0, false),
        ("Tour Escolar", 1, false),
        ("Evaluación", 2, false),
        ("Documentación", 3, false),
        ("Aceptado", 4, false),
        ("Matriculado", 5, true),
    ];

    for (name, order, is_final) in stages {
        let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM pipeline_stages WHERE name = $1")
            .bind(name)
            .fetch_one(pool)
            .await
            .unwrap_or((0,));

        if exists.0 == 0 {
            let id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO pipeline_stages (id, name, sort_order, is_final) VALUES ($1, $2, $3, $4)",
            )
            .bind(id).bind(name).bind(order).bind(is_final)
            .execute(pool).await.unwrap_or_else(|e| {
                tracing::warn!("Could not seed stage {name}: {e}");
                Default::default()
            });
        }
    }
}

async fn list_stages(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Admision"],
    )?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "admission",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;
    let stages = sqlx::query_as::<_, schoolccb_common::admission::PipelineStage>(
        "SELECT id, name, sort_order, is_final, created_at FROM pipeline_stages ORDER BY sort_order",
    ).fetch_all(&state.pool).await?;
    Ok(Json(json!({ "stages": stages })))
}

async fn get_stage(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Admision"],
    )?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "admission",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;
    let stage = sqlx::query_as::<_, schoolccb_common::admission::PipelineStage>(
        "SELECT id, name, sort_order, is_final, created_at FROM pipeline_stages WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(SisError::NotFound("Etapa no encontrada".into()))?;
    Ok(Json(json!({ "stage": stage })))
}

async fn create_stage(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::admission::CreateStagePayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;
    if payload.name.trim().is_empty() {
        return Err(SisError::Validation("Nombre obligatorio".into()));
    }
    let id = Uuid::new_v4();
    let result = sqlx::query_as::<_, schoolccb_common::admission::PipelineStage>(
        "INSERT INTO pipeline_stages (id, name, sort_order, is_final) VALUES ($1, $2, $3, $4)
         RETURNING id, name, sort_order, is_final, created_at",
    )
    .bind(id)
    .bind(&payload.name)
    .bind(payload.sort_order.unwrap_or(0))
    .bind(payload.is_final.unwrap_or(false))
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(json!({ "stage": result })))
}

async fn update_stage(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::admission::UpdateStagePayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;
    let current = sqlx::query_as::<_, schoolccb_common::admission::PipelineStage>(
        "SELECT id, name, sort_order, is_final, created_at FROM pipeline_stages WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(SisError::NotFound("Etapa no encontrada".into()))?;
    let name = payload.name.unwrap_or(current.name);
    let sort_order = payload.sort_order.unwrap_or(current.sort_order);
    let is_final = payload.is_final.unwrap_or(current.is_final);
    let result = sqlx::query_as::<_, schoolccb_common::admission::PipelineStage>(
        "UPDATE pipeline_stages SET name = $1, sort_order = $2, is_final = $3 WHERE id = $4
         RETURNING id, name, sort_order, is_final, created_at",
    )
    .bind(&name)
    .bind(sort_order)
    .bind(is_final)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(json!({ "stage": result })))
}

async fn delete_stage(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;
    sqlx::query("DELETE FROM pipeline_stages WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({ "message": "Etapa eliminada" })))
}
