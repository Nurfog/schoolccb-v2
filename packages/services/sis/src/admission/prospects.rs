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
use crate::routes::students::{Claims, require_any_role};
use crate::workflow::CrmEvent;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/admission/prospects",
            get(list_prospects).post(create_prospect),
        )
        .route(
            "/api/admission/prospects/{id}",
            get(get_prospect)
                .put(update_prospect)
                .delete(delete_prospect),
        )
        .route("/api/admission/prospects/{id}/stage", put(change_stage))
}

#[derive(Deserialize)]
struct ProspectFilter {
    stage_id: Option<Uuid>,
    search: Option<String>,
    assigned_to: Option<Uuid>,
}

async fn list_prospects(
    claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<ProspectFilter>,
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

    let mut sql = "SELECT id, first_name, last_name, rut, email, phone, current_stage_id, assigned_user_id, source, notes, created_at, updated_at FROM prospects".to_string();
    let mut clauses: Vec<String> = vec![];

    if q.stage_id.is_some() {
        clauses.push(format!("current_stage_id = ${}", clauses.len() + 1));
    }
    if q.search.is_some() {
        let n = clauses.len() + 1;
        clauses.push(format!(
            "(first_name ILIKE ${n} OR last_name ILIKE ${n} OR rut ILIKE ${n})"
        ));
    }
    if q.assigned_to.is_some() {
        clauses.push(format!("assigned_user_id = ${}", clauses.len() + 1));
    }
    if !clauses.is_empty() {
        sql.push_str(&format!(" WHERE {}", clauses.join(" AND ")));
    }
    sql.push_str(" ORDER BY created_at DESC LIMIT 100");

    let mut query = sqlx::query_as::<_, schoolccb_common::admission::Prospect>(&sql);
    if let Some(sid) = q.stage_id {
        query = query.bind(sid);
    }
    if let Some(ref s) = q.search {
        let pat = format!("%{}%", s);
        query = query.bind(pat);
    }
    if let Some(uid) = q.assigned_to {
        query = query.bind(uid);
    }

    let prospects = query.fetch_all(&state.pool).await?;
    Ok(Json(
        json!({ "prospects": prospects, "total": prospects.len() }),
    ))
}

async fn get_prospect(
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
    let prospect = sqlx::query_as::<_, schoolccb_common::admission::Prospect>(
        "SELECT id, first_name, last_name, rut, email, phone, current_stage_id, assigned_user_id, source, notes, created_at, updated_at FROM prospects WHERE id = $1",
    ).bind(id).fetch_optional(&state.pool).await?
        .ok_or(SisError::NotFound("Postulante no encontrado".into()))?;

    let activities = sqlx::query_as::<_, schoolccb_common::admission::ProspectActivity>(
        "SELECT id, prospect_id, activity_type, subject, description, scheduled_at, is_completed, created_by, created_at FROM prospect_activities WHERE prospect_id = $1 ORDER BY created_at DESC",
    ).bind(id).fetch_all(&state.pool).await?;

    let documents = sqlx::query_as::<_, schoolccb_common::admission::ProspectDocument>(
        "SELECT id, prospect_id, file_name, s3_url, doc_type, is_verified, uploaded_by, created_at FROM prospect_documents WHERE prospect_id = $1 ORDER BY created_at DESC",
    ).bind(id).fetch_all(&state.pool).await?;

    let reminders = sqlx::query_as::<_, schoolccb_common::admission::ProspectReminder>(
        "SELECT id, prospect_id, reminder_type, title, description, remind_at, is_sent, created_by, created_at FROM prospect_reminders WHERE prospect_id = $1 ORDER BY remind_at ASC",
    ).bind(id).fetch_all(&state.pool).await?;

    Ok(Json(
        json!({ "prospect": prospect, "activities": activities, "documents": documents, "reminders": reminders }),
    ))
}

async fn create_prospect(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::admission::CreateProspectPayload>,
) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Admision"],
    )?;
    if payload.first_name.trim().is_empty() || payload.last_name.trim().is_empty() {
        return Err(SisError::Validation(
            "Nombre y apellido obligatorios".into(),
        ));
    }

    let id = Uuid::new_v4();
    let first_stage: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM pipeline_stages ORDER BY sort_order LIMIT 1")
            .fetch_optional(&state.pool)
            .await?;

    let result = sqlx::query_as::<_, schoolccb_common::admission::Prospect>(
        r#"INSERT INTO prospects (id, first_name, last_name, rut, email, phone, current_stage_id, source, notes)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
           RETURNING id, first_name, last_name, rut, email, phone, current_stage_id, assigned_user_id, source, notes, created_at, updated_at"#,
    ).bind(id).bind(&payload.first_name).bind(&payload.last_name)
    .bind(&payload.rut).bind(&payload.email).bind(&payload.phone)
    .bind(first_stage.map(|s| s.0)).bind(&payload.source).bind(&payload.notes)
    .fetch_one(&state.pool).await?;

    Ok(Json(json!({ "prospect": result })))
}

async fn update_prospect(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::admission::UpdateProspectPayload>,
) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Admision"],
    )?;
    let current = sqlx::query_as::<_, schoolccb_common::admission::Prospect>(
        "SELECT id, first_name, last_name, rut, email, phone, current_stage_id, assigned_user_id, source, notes, created_at, updated_at FROM prospects WHERE id = $1",
    ).bind(id).fetch_optional(&state.pool).await?
        .ok_or(SisError::NotFound("Postulante no encontrado".into()))?;

    let result = sqlx::query_as::<_, schoolccb_common::admission::Prospect>(
        r#"UPDATE prospects SET first_name = $1, last_name = $2, rut = $3, email = $4, phone = $5, source = $6, notes = $7, updated_at = NOW() WHERE id = $8
           RETURNING id, first_name, last_name, rut, email, phone, current_stage_id, assigned_user_id, source, notes, created_at, updated_at"#,
    ).bind(payload.first_name.unwrap_or(current.first_name))
    .bind(payload.last_name.unwrap_or(current.last_name))
    .bind(payload.rut.or(current.rut))
    .bind(payload.email.or(current.email))
    .bind(payload.phone.or(current.phone))
    .bind(payload.source.or(current.source))
    .bind(payload.notes.or(current.notes))
    .bind(id)
    .fetch_one(&state.pool).await?;

    Ok(Json(json!({ "prospect": result })))
}

async fn delete_prospect(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;
    sqlx::query("DELETE FROM prospects WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({ "message": "Postulante eliminado" })))
}

async fn change_stage(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ChangeStagePayload>,
) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Admision"],
    )?;

    let old_stage: Option<Uuid> = sqlx::query_as::<_, (Option<Uuid>,)>(
        "SELECT current_stage_id FROM prospects WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .and_then(|r| r.0);

    let stage_info: Option<(String, bool)> =
        sqlx::query_as("SELECT name, is_final FROM pipeline_stages WHERE id = $1")
            .bind(payload.stage_id)
            .fetch_optional(&state.pool)
            .await?;

    let (stage_name, is_final) =
        stage_info.ok_or_else(|| SisError::Validation("La etapa no existe".into()))?;

    if is_final {
        let rut: Option<String> =
            sqlx::query_as::<_, (Option<String>,)>("SELECT rut FROM prospects WHERE id = $1")
                .bind(id)
                .fetch_optional(&state.pool)
                .await?
                .and_then(|r| r.0)
                .filter(|r| !r.is_empty());
        if rut.is_none() {
            return Err(SisError::Validation(
                "El postulante debe tener RUT para ser matriculado".into(),
            ));
        }
    }

    let result = sqlx::query_as::<_, schoolccb_common::admission::Prospect>(
        r#"UPDATE prospects SET current_stage_id = $1, updated_at = NOW() WHERE id = $2
           RETURNING id, first_name, last_name, rut, email, phone, current_stage_id, assigned_user_id, source, notes, created_at, updated_at"#,
    ).bind(payload.stage_id).bind(id).fetch_one(&state.pool).await?;

    let user_id = Uuid::parse_str(&claims.sub).ok();
    let event = CrmEvent::StageChanged {
        prospect_id: id,
        from_stage_id: old_stage,
        to_stage_id: payload.stage_id,
        to_stage_name: stage_name,
        triggered_by: user_id,
    };

    let wf = state.workflow.clone();
    tokio::spawn(async move {
        wf.process(event).await;
    });

    Ok(Json(json!({ "prospect": result })))
}

#[derive(Deserialize)]
struct ChangeStagePayload {
    stage_id: Uuid,
}
