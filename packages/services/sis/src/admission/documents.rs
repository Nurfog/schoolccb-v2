use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{SisError, SisResult};
use schoolccb_common::auth::{Claims, require_any_role};
use crate::workflow::CrmEvent;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/admission/documents",
            get(list_documents).post(create_document),
        )
        .route(
            "/api/admission/documents/{id}",
            get(get_document).delete(delete_document),
        )
        .route(
            "/api/admission/documents/{id}/verify",
            post(verify_document),
        )
}

async fn list_documents(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
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
    let docs = sqlx::query_as::<_, schoolccb_common::admission::ProspectDocument>(
        "SELECT id, prospect_id, file_name, s3_url, doc_type, is_verified, uploaded_by, created_at FROM prospect_documents ORDER BY created_at DESC LIMIT 200",
    ).fetch_all(&state.pool).await?;
    Ok(Json(json!({ "documents": docs })))
}

async fn get_document(
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
    let doc = sqlx::query_as::<_, schoolccb_common::admission::ProspectDocument>(
        "SELECT id, prospect_id, file_name, s3_url, doc_type, is_verified, uploaded_by, created_at FROM prospect_documents WHERE id = $1",
    ).bind(id).fetch_optional(&state.pool).await?
        .ok_or(crate::error::SisError::NotFound("Documento no encontrado".into()))?;
    Ok(Json(json!({ "document": doc })))
}

async fn create_document(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::admission::CreateDocumentPayload>,
) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Admision"],
    )?;
    let id = Uuid::new_v4();
    let user_id = Uuid::parse_str(&claims.sub).ok();
    let result = sqlx::query_as::<_, schoolccb_common::admission::ProspectDocument>(
        r#"INSERT INTO prospect_documents (id, prospect_id, file_name, doc_type, uploaded_by)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, prospect_id, file_name, s3_url, doc_type, is_verified, uploaded_by, created_at"#,
    ).bind(id).bind(payload.prospect_id).bind(&payload.file_name).bind(&payload.doc_type).bind(user_id)
    .fetch_one(&state.pool).await?;

    let event = CrmEvent::DocumentUploaded {
        prospect_id: payload.prospect_id,
        document_id: id,
        doc_type: payload.doc_type.clone(),
        uploaded_by: user_id,
    };
    let wf = state.workflow.clone();
    tokio::spawn(async move {
        wf.process(event).await;
    });

    Ok(Json(json!({ "document": result })))
}

async fn verify_document(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;
    let result = sqlx::query_as::<_, schoolccb_common::admission::ProspectDocument>(
        "UPDATE prospect_documents SET is_verified = true WHERE id = $1
         RETURNING id, prospect_id, file_name, s3_url, doc_type, is_verified, uploaded_by, created_at",
    ).bind(id).fetch_one(&state.pool).await?;

    let event = CrmEvent::DocumentVerified {
        document_id: id,
        prospect_id: result.prospect_id,
    };
    let wf = state.workflow.clone();
    tokio::spawn(async move {
        wf.process(event).await;
    });

    Ok(Json(json!({ "document": result })))
}

async fn delete_document(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;
    sqlx::query("DELETE FROM prospect_documents WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({ "message": "Documento eliminado" })))
}
