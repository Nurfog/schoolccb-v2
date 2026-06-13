use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, put},
};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{SisError, SisResult};
use schoolccb_common::auth::{Claims, require_any_role};

#[derive(Deserialize)]
struct FieldDefinitionPayload {
    entity_type: String,
    field_name: String,
    field_type: Option<String>,
    is_required: Option<bool>,
    options: Option<Value>,
    sort_order: Option<i32>,
}

#[derive(Deserialize)]
struct SaveValuesPayload {
    values: Vec<FieldValuePayload>,
}

#[derive(Deserialize)]
struct FieldValuePayload {
    field_definition_id: Uuid,
    value: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/admission/custom-fields/definitions",
            get(list_definitions).post(create_definition),
        )
        .route(
            "/api/admission/custom-fields/definitions/{id}",
            put(update_definition).delete(delete_definition),
        )
        .route(
            "/api/admission/custom-fields/values/{entity_id}",
            get(get_values).put(save_values),
        )
}

#[derive(Deserialize)]
struct ListDefinitionsFilter {
    entity_type: Option<String>,
}

async fn list_definitions(
    claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<ListDefinitionsFilter>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "admission",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;

    let rows = match q.entity_type {
        Some(ref et) => sqlx::query_as::<_, CustomFieldRow>(
            "SELECT id, entity_type, field_name, field_type, is_required, options, sort_order, active, created_at FROM custom_field_definitions WHERE active = true AND entity_type = $1 ORDER BY sort_order",
        )
        .bind(et)
        .fetch_all(&state.pool).await?,
        None => sqlx::query_as::<_, CustomFieldRow>(
            "SELECT id, entity_type, field_name, field_type, is_required, options, sort_order, active, created_at FROM custom_field_definitions WHERE active = true ORDER BY entity_type, sort_order",
        )
        .fetch_all(&state.pool).await?,
    };

    Ok(Json(json!({ "definitions": rows })))
}

async fn create_definition(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<FieldDefinitionPayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    let id = Uuid::new_v4();
    let field_type = payload.field_type.unwrap_or_else(|| "text".into());
    let is_required = payload.is_required.unwrap_or(false);
    let sort_order = payload.sort_order.unwrap_or(0);

    sqlx::query(
        r#"INSERT INTO custom_field_definitions (id, entity_type, field_name, field_type, is_required, options, sort_order)
           VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
    )
    .bind(id)
    .bind(&payload.entity_type)
    .bind(&payload.field_name)
    .bind(&field_type)
    .bind(is_required)
    .bind(&payload.options)
    .bind(sort_order)
    .execute(&state.pool).await?;

    Ok(Json(json!({ "id": id })))
}

async fn update_definition(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<FieldDefinitionPayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    sqlx::query(
        r#"UPDATE custom_field_definitions SET field_name = $1, field_type = $2, is_required = $3, options = $4, sort_order = $5 WHERE id = $6"#,
    )
    .bind(&payload.field_name)
    .bind(payload.field_type.unwrap_or_else(|| "text".into()))
    .bind(payload.is_required.unwrap_or(false))
    .bind(&payload.options)
    .bind(payload.sort_order.unwrap_or(0))
    .bind(id)
    .execute(&state.pool).await?;

    Ok(Json(json!({ "message": "Campo actualizado" })))
}

async fn delete_definition(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    sqlx::query("DELETE FROM custom_field_definitions WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({ "message": "Campo eliminado" })))
}

async fn get_values(
    claims: Claims,
    State(state): State<AppState>,
    Path(entity_id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Admision"],
    )?;

    let values = sqlx::query_as::<_, (Uuid, Option<String>)>(
        "SELECT field_definition_id, value FROM custom_field_values WHERE entity_id = $1",
    )
    .bind(entity_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(
        json!({ "values": values.into_iter().map(|(fid, val)| json!({"field_definition_id": fid, "value": val})).collect::<Vec<_>>() }),
    ))
}

async fn save_values(
    claims: Claims,
    State(state): State<AppState>,
    Path(entity_id): Path<Uuid>,
    Json(payload): Json<SaveValuesPayload>,
) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Admision"],
    )?;

    for fv in &payload.values {
        let existing: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM custom_field_values WHERE field_definition_id = $1 AND entity_id = $2",
        )
        .bind(fv.field_definition_id)
        .bind(entity_id)
        .fetch_one(&state.pool).await?;

        if existing.0 > 0 {
            sqlx::query(
                "UPDATE custom_field_values SET value = $1, updated_at = NOW() WHERE field_definition_id = $2 AND entity_id = $3",
            )
            .bind(&fv.value)
            .bind(fv.field_definition_id)
            .bind(entity_id)
            .execute(&state.pool).await?;
        } else {
            sqlx::query(
                "INSERT INTO custom_field_values (id, field_definition_id, entity_id, value) VALUES ($1, $2, $3, $4)",
            )
            .bind(Uuid::new_v4())
            .bind(fv.field_definition_id)
            .bind(entity_id)
            .bind(&fv.value)
            .execute(&state.pool).await?;
        }
    }

    Ok(Json(json!({ "message": "Valores guardados" })))
}

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
struct CustomFieldRow {
    id: Uuid,
    entity_type: String,
    field_name: String,
    field_type: String,
    is_required: bool,
    options: Option<Value>,
    sort_order: i32,
    active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}
