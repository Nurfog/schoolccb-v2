use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post, put},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{AcademicError, AcademicResult};

pub use schoolccb_common::auth::Claims;
use schoolccb_common::auth::require_any_role;

#[derive(Deserialize)]
pub struct SubjectFilter {
    pub search: Option<String>,
    pub level: Option<String>,
}

#[derive(sqlx::FromRow, Serialize)]
struct RawSubject {
    id: Uuid,
    code: String,
    name: String,
    level: Option<String>,
    hours_per_week: i32,
    active: bool,
}

#[derive(sqlx::FromRow, Serialize)]
struct RawSubjectHour {
    level: String,
    hours_per_week: i32,
}

#[derive(Deserialize)]
struct SaveHoursPayload {
    hours: Vec<LevelHour>,
}

#[derive(Deserialize, Serialize)]
struct LevelHour {
    level: String,
    hours_per_week: i32,
}

const PLANS: &[&str] = &["HC", "TP", "Artístico"];

const LEVELS: &[&str] = &[
    "Sala Cuna",
    "Medio Menor",
    "Medio Mayor",
    "Pre-kinder",
    "Kinder",
    "1° Básico",
    "2° Básico",
    "3° Básico",
    "4° Básico",
    "5° Básico",
    "6° Básico",
    "7° Básico",
    "8° Básico",
    "1° Medio",
    "2° Medio",
    "3° Medio HC",
    "4° Medio HC",
    "3° Medio TP",
    "4° Medio TP",
    "3° Medio Artístico",
    "4° Medio Artístico",
];

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/grades/subjects",
            get(list_subjects).post(create_subject),
        )
        .route(
            "/api/grades/subjects/{id}",
            get(get_subject)
                .put(update_subject)
                .delete(deactivate_subject),
        )
        .route("/api/grades/subjects/{id}/hours", put(save_hours))
        .route("/api/grades/subjects/import", post(import_subjects))
        .route("/api/academic/audit-log", get(get_audit_log))
}

async fn list_subjects(
    claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<SubjectFilter>,
) -> AcademicResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "subjects",
    )
    .await
    .map_err(|e| AcademicError::Forbidden(e))?;

    let search_pattern = filter.search.as_ref().map(|q| format!("%{}%", q));

    let raw_subjects = if let (Some(pat), Some(lvl)) = (&search_pattern, &filter.level) {
        sqlx::query_as::<_, RawSubject>(
            "SELECT id, code, name, level, hours_per_week, active FROM subjects WHERE (name ILIKE $1 OR code ILIKE $1) AND level = $2 ORDER BY name",
        )
        .bind(pat).bind(lvl).fetch_all(&state.pool).await?
    } else if let Some(ref pat) = search_pattern {
        sqlx::query_as::<_, RawSubject>(
            "SELECT id, code, name, level, hours_per_week, active FROM subjects WHERE name ILIKE $1 OR code ILIKE $1 ORDER BY name",
        )
        .bind(pat).fetch_all(&state.pool).await?
    } else {
        sqlx::query_as::<_, RawSubject>(
            "SELECT id, code, name, level, hours_per_week, active FROM subjects ORDER BY name",
        )
        .fetch_all(&state.pool)
        .await?
    };

    let mut result = Vec::new();
    for s in raw_subjects {
        let hours: Vec<RawSubjectHour> = sqlx::query_as(
            "SELECT level, hours_per_week FROM subject_hours WHERE subject_id = $1 ORDER BY level",
        )
        .bind(s.id)
        .fetch_all(&state.pool)
        .await?;
        result.push(json!({
            "id": s.id,
            "code": s.code,
            "name": s.name,
            "level": s.level,
            "hours_per_week": s.hours_per_week,
            "active": s.active,
            "hours_by_level": hours,
        }));
    }

    Ok(Json(
        json!({ "subjects": result, "total": result.len(), "levels": LEVELS, "plans": PLANS }),
    ))
}

async fn get_subject(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AcademicResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;

    let s = sqlx::query_as::<_, RawSubject>(
        "SELECT id, code, name, level, hours_per_week, active FROM subjects WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AcademicError::NotFound("Asignatura no encontrada".into()))?;

    let hours: Vec<RawSubjectHour> = sqlx::query_as(
        "SELECT level, hours_per_week FROM subject_hours WHERE subject_id = $1 ORDER BY level",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({
        "subject": {
            "id": s.id, "code": s.code, "name": s.name,
            "level": s.level, "hours_per_week": s.hours_per_week, "active": s.active,
            "hours_by_level": hours,
        }
    })))
}

async fn create_subject(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::academic::CreateSubjectPayload>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    if payload.code.trim().is_empty() || payload.name.trim().is_empty() {
        return Err(AcademicError::Validation(
            "Código y nombre son obligatorios".into(),
        ));
    }

    let id = Uuid::new_v4();
    let result = sqlx::query_as::<_, RawSubject>(
        "INSERT INTO subjects (id, code, name, level, hours_per_week) VALUES ($1, $2, $3, $4, $5)
         RETURNING id, code, name, level, hours_per_week, active",
    )
    .bind(id)
    .bind(&payload.code)
    .bind(&payload.name)
    .bind(&payload.level)
    .bind(payload.hours_per_week.unwrap_or(0))
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint() == Some("subjects_code_key") {
                return AcademicError::Conflict("El código de asignatura ya existe".into());
            }
        }
        AcademicError::Database(e)
    })?;

    let user_id = Uuid::parse_str(&claims.sub).ok();
    schoolccb_common::audit::log(
        &state.pool,
        &schoolccb_common::audit::AuditEntry {
            entity_type: "subject".into(),
            entity_id: id,
            action: "created".into(),
            user_id,
            changes: Some(serde_json::json!({
                "code": &payload.code, "name": &payload.name,
                "level": &payload.level, "hours_per_week": payload.hours_per_week,
            })),
        },
    )
    .await;

    Ok(Json(json!({ "subject": result })))
}

async fn update_subject(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::academic::UpdateSubjectPayload>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let existing = sqlx::query_as::<_, RawSubject>(
        "SELECT id, code, name, level, hours_per_week, active FROM subjects WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AcademicError::NotFound("Asignatura no encontrada".into()))?;

    let code = payload.code.unwrap_or_else(|| existing.code.clone());
    let name = payload.name.unwrap_or_else(|| existing.name.clone());
    let level = payload.level.clone().or_else(|| existing.level.clone());
    let hours_per_week = payload.hours_per_week.unwrap_or(existing.hours_per_week);

    let result = sqlx::query_as::<_, RawSubject>(
        "UPDATE subjects SET code = $1, name = $2, level = $3, hours_per_week = $4 WHERE id = $5
         RETURNING id, code, name, level, hours_per_week, active",
    )
    .bind(&code)
    .bind(&name)
    .bind(&level)
    .bind(hours_per_week)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    let user_id = Uuid::parse_str(&claims.sub).ok();
    let mut changes = serde_json::json!({});
    if code != existing.code {
        changes["code"] = serde_json::json!(code);
    }
    if name != existing.name {
        changes["name"] = serde_json::json!(name);
    }
    if level != existing.level {
        changes["level"] = serde_json::json!(level);
    }
    if hours_per_week != existing.hours_per_week {
        changes["hours_per_week"] = serde_json::json!(hours_per_week);
    }
    if !changes.as_object().map(|o| o.is_empty()).unwrap_or(true) {
        schoolccb_common::audit::log(
            &state.pool,
            &schoolccb_common::audit::AuditEntry {
                entity_type: "subject".into(),
                entity_id: id,
                action: "updated".into(),
                user_id,
                changes: Some(changes),
            },
        )
        .await;
    }

    Ok(Json(json!({ "subject": result })))
}

async fn deactivate_subject(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    let result = sqlx::query("UPDATE subjects SET active = false WHERE id = $1 AND active = true")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AcademicError::NotFound(
            "Asignatura no encontrada o ya desactivada".into(),
        ));
    }

    sqlx::query("DELETE FROM subject_hours WHERE subject_id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    let user_id = Uuid::parse_str(&claims.sub).ok();
    schoolccb_common::audit::log(
        &state.pool,
        &schoolccb_common::audit::AuditEntry {
            entity_type: "subject".into(),
            entity_id: id,
            action: "deactivated".into(),
            user_id,
            changes: None,
        },
    )
    .await;

    Ok(Json(
        json!({ "message": "Asignatura desactivada correctamente" }),
    ))
}

async fn save_hours(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<SaveHoursPayload>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM subjects WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await?;

    if exists.0 == 0 {
        return Err(AcademicError::NotFound("Asignatura no encontrada".into()));
    }

    sqlx::query("DELETE FROM subject_hours WHERE subject_id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    for lh in &payload.hours {
        sqlx::query(
            "INSERT INTO subject_hours (subject_id, level, hours_per_week) VALUES ($1, $2, $3)",
        )
        .bind(id)
        .bind(&lh.level)
        .bind(lh.hours_per_week)
        .execute(&state.pool)
        .await?;
    }

    let hours: Vec<RawSubjectHour> = sqlx::query_as(
        "SELECT level, hours_per_week FROM subject_hours WHERE subject_id = $1 ORDER BY level",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    let user_id = Uuid::parse_str(&claims.sub).ok();
    schoolccb_common::audit::log(
        &state.pool,
        &schoolccb_common::audit::AuditEntry {
            entity_type: "subject_hours".into(),
            entity_id: id,
            action: "hours_updated".into(),
            user_id,
            changes: Some(serde_json::json!({ "hours": &payload.hours })),
        },
    )
    .await;

    Ok(Json(json!({ "hours": hours })))
}

#[derive(Deserialize, Serialize)]
struct ImportSubjectsPayload {
    subjects: Vec<CreateSubjectRow>,
}

#[derive(Deserialize, Serialize)]
struct CreateSubjectRow {
    code: String,
    name: String,
    level: Option<String>,
    hours_per_week: Option<i32>,
    hours_by_level: Option<Vec<ImportHourRow>>,
}

#[derive(Deserialize, Serialize)]
struct ImportHourRow {
    level: String,
    hours_per_week: i32,
}

async fn import_subjects(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<ImportSubjectsPayload>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let mut imported = 0u32;
    let mut skipped = 0u32;
    let mut errors: Vec<Value> = vec![];

    for row in &payload.subjects {
        if row.code.trim().is_empty() || row.name.trim().is_empty() {
            errors.push(json!({ "code": &row.code, "error": "Código y nombre son obligatorios" }));
            continue;
        }

        let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM subjects WHERE code = $1")
            .bind(&row.code)
            .fetch_one(&state.pool)
            .await
            .unwrap_or((0,));

        if exists.0 > 0 {
            skipped += 1;
            continue;
        }

        let id = Uuid::new_v4();
        let result = sqlx::query(
            "INSERT INTO subjects (id, code, name, level, hours_per_week) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(id)
        .bind(&row.code)
        .bind(&row.name)
        .bind(&row.level)
        .bind(row.hours_per_week.unwrap_or(0))
        .execute(&state.pool)
        .await;

        match result {
            Ok(_) => {
                imported += 1;

                if let Some(ref hours) = row.hours_by_level {
                    for h in hours {
                        let _ = sqlx::query(
                            "INSERT INTO subject_hours (id, subject_id, level, hours_per_week) VALUES ($1, $2, $3, $4)
                             ON CONFLICT (subject_id, level) DO NOTHING",
                        )
                        .bind(Uuid::new_v4())
                        .bind(id)
                        .bind(&h.level)
                        .bind(h.hours_per_week)
                        .execute(&state.pool)
                        .await;
                    }
                }

                let user_id = Uuid::parse_str(&claims.sub).ok();
                schoolccb_common::audit::log(
                    &state.pool,
                    &schoolccb_common::audit::AuditEntry {
                        entity_type: "subject".into(),
                        entity_id: id,
                        action: "bulk_imported".into(),
                        user_id,
                        changes: Some(json!({ "code": &row.code, "name": &row.name })),
                    },
                )
                .await;
            }
            Err(e) => {
                errors.push(json!({ "code": &row.code, "error": e.to_string() }));
            }
        }
    }

    Ok(Json(json!({
        "imported": imported,
        "skipped": skipped,
        "errors": errors,
        "total": payload.subjects.len(),
    })))
}

async fn get_audit_log(
    claims: Claims,
    State(state): State<AppState>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    #[derive(sqlx::FromRow, Serialize)]
    struct AuditRow {
        id: Uuid,
        entity_type: String,
        entity_id: Uuid,
        action: String,
        user_id: Option<Uuid>,
        changes: Option<serde_json::Value>,
        created_at: chrono::NaiveDateTime,
    }

    let logs = sqlx::query_as::<_, AuditRow>(
        "SELECT id, entity_type, entity_id, action, user_id, changes, created_at
         FROM audit_log ORDER BY created_at DESC LIMIT 200",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "audit_logs": logs })))
}
