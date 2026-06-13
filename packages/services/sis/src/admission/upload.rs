use axum::{
    Json, Router,
    extract::{Multipart, State},
    routing::post,
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::SisResult;
use schoolccb_common::auth::{Claims, require_any_role};

pub fn router() -> Router<AppState> {
    Router::new().route("/api/admission/documents/upload", post(upload_document))
}

async fn upload_document(
    claims: Claims,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Admision"],
    )?;

    let user_id = Uuid::parse_str(&claims.sub).ok();
    let mut prospect_id = None;
    let mut doc_type = "other".to_string();
    let mut file_name = String::new();
    let mut file_data = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "prospect_id" => {
                prospect_id = Some(
                    Uuid::parse_str(field.text().await.unwrap_or_default().trim()).map_err(
                        |_| crate::error::SisError::Validation("prospect_id inválido".into()),
                    )?,
                );
            }
            "doc_type" => {
                doc_type = field.text().await.unwrap_or_default();
            }
            "file" => {
                file_name = field.file_name().unwrap_or("documento").to_string();
                file_data = field.bytes().await.unwrap_or_default().to_vec();
            }
            _ => {}
        }
    }

    let pid = prospect_id
        .ok_or_else(|| crate::error::SisError::Validation("prospect_id es obligatorio".into()))?;

    if file_data.is_empty() {
        return Err(crate::error::SisError::Validation(
            "Debe adjuntar un archivo".into(),
        ));
    }

    // Create uploads directory structure
    let upload_dir = std::path::Path::new("uploads")
        .join("prospects")
        .join(pid.to_string());
    tokio::fs::create_dir_all(&upload_dir).await.map_err(|e| {
        crate::error::SisError::Internal(format!("Error al crear directorio: {}", e))
    })?;

    // Save file
    let file_path = upload_dir.join(&file_name);
    tokio::fs::write(&file_path, &file_data)
        .await
        .map_err(|e| {
            crate::error::SisError::Internal(format!("Error al guardar archivo: {}", e))
        })?;

    let s3_url = Some(format!("/uploads/prospects/{}/{}", pid, file_name));

    let doc_id = Uuid::new_v4();
    let result = sqlx::query_as::<_, schoolccb_common::admission::ProspectDocument>(
        r#"INSERT INTO prospect_documents (id, prospect_id, file_name, s3_url, doc_type, uploaded_by)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, prospect_id, file_name, s3_url, doc_type, is_verified, uploaded_by, created_at"#,
    )
    .bind(doc_id)
    .bind(pid)
    .bind(&file_name)
    .bind(&s3_url)
    .bind(&doc_type)
    .bind(user_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "document": result })))
}
