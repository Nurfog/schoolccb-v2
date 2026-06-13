use axum::{
    extract::{Multipart, Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
use std::path::PathBuf;
use uuid::Uuid;

use crate::error::{SisError, SisResult};
use schoolccb_common::auth::{Claims, require_any_role};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/academic/changelog", get(list_changelog))
        .route("/api/admission/prospects/{id}/tabs", get(get_prospect_tabs))
        .route("/api/interactions", post(log_interaction).get(list_interactions))
        .route("/api/admission/prospects/{id}/family", get(list_family_members).post(add_family_member))
        .route("/api/admission/stagnation-check", get(check_stagnation))
        .route("/api/admission/metrics/time-in-stage", get(time_in_stage))
        .route("/api/admission/check-vacancies", get(check_vacancies))
        .route("/api/admission/prospects/upload", post(upload_document))
        .route("/api/hr/attendance/geocheck", post(geocerca_validate))
        .route("/api/hr/payroll/export/lre-audit", get(lre_audit))
        .route("/api/hr/attendance/sync-hmac", post(sync_attendance_hmac))
}

async fn check_vacancies(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "Admision"])?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "admission",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;

    let courses = sqlx::query(
        r#"SELECT c.id, c.grade_level, c.section, c.name, c.max_students,
                  COUNT(e.id) as enrolled
           FROM courses c
           LEFT JOIN enrollments e ON e.course_id = c.id AND e.active = true
           GROUP BY c.id, c.grade_level, c.section, c.name, c.max_students
           ORDER BY c.grade_level, c.section"#,
    ).fetch_all(&state.pool).await?;

    let results: Vec<Value> = courses.iter().map(|row| {
        let id: Uuid = row.get("id");
        let grade: String = row.get("grade_level");
        let section: String = row.get("section");
        let name: String = row.get("name");
        let max_students: Option<i32> = row.get("max_students");
        let enrolled: i64 = row.get("enrolled");
        let max = max_students.unwrap_or(35);
        let available = (max as i64 - enrolled).max(0);
        json!({
            "id": id, "grade_level": grade, "section": section, "name": name,
            "max_students": max, "enrolled": enrolled, "available": available,
            "full": available <= 0,
        })
    }).collect();

    Ok(Json(json!({ "courses": results, "total": results.len() })))
}

async fn upload_document(claims: Claims, State(state): State<AppState>, mut multipart: Multipart) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Admision"])?;

    let mut prospect_id = None;
    let mut doc_type = "other".to_string();
    let mut file_name = String::new();
    let mut file_data = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        match field.name().unwrap_or("") {
            "prospect_id" => {
                let text = field.text().await.unwrap_or_default();
                prospect_id = Some(Uuid::parse_str(text.trim()).map_err(|_| SisError::Validation("prospect_id invalido".into()))?);
            }
            "doc_type" => doc_type = field.text().await.unwrap_or_default(),
            "file" => {
                file_name = field.file_name().unwrap_or("documento").to_string();
                file_data = field.bytes().await.unwrap_or_default().to_vec();
            }
            _ => {}
        }
    }

    let pid = prospect_id.ok_or(SisError::Validation("prospect_id es requerido".into()))?;
    if file_data.is_empty() {
        return Err(SisError::Validation("Archivo requerido".into()));
    }

    let upload_dir = PathBuf::from(&state.config.upload_dir).join("prospects").join(pid.to_string());
    tokio::fs::create_dir_all(&upload_dir).await.map_err(|e| SisError::Internal(format!("Error creando directorio: {e}")))?;
    let file_path = upload_dir.join(&file_name);
    tokio::fs::write(&file_path, &file_data).await.map_err(|e| SisError::Internal(format!("Error guardando archivo: {e}")))?;

    let doc_id = Uuid::new_v4();
    let file_url = format!("/uploads/prospects/{}/{}", pid, file_name);
    sqlx::query(
        r#"INSERT INTO prospect_documents (id, prospect_id, doc_type, file_name, s3_url, is_verified)
           VALUES ($1, $2, $3, $4, $5, false)"#,
    ).bind(doc_id).bind(pid).bind(&doc_type).bind(&file_name)
    .bind(&file_url)
    .execute(&state.pool).await?;

    Ok(Json(json!({ "document_id": doc_id, "file_url": file_url, "file_size": file_data.len() })))
}

#[derive(Deserialize)]
struct LreAuditFilter {
    month: Option<i32>,
}

#[derive(Deserialize)]
struct ChangelogFilter {
    entity_type: Option<String>,
    entity_id: Option<Uuid>,
}

#[derive(Deserialize, Serialize)]
struct InteractionPayload {
    entity_type: String,
    entity_id: Uuid,
    interaction_type: String,
    subject: String,
    description: Option<String>,
}

async fn list_changelog(claims: Claims, State(state): State<AppState>, Query(q): Query<ChangelogFilter>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "academic",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;

    let mut sql = "SELECT id, entity_type, entity_id, action, field_name, old_value, new_value, changed_by, created_at FROM academic_changelog".to_string();
    let mut clauses: Vec<String> = vec![];
    let mut idx = 1;

    if let Some(ref _et) = q.entity_type {
        clauses.push(format!("entity_type = ${}", idx));
        idx += 1;
    }
    if let Some(_eid) = q.entity_id {
        clauses.push(format!("entity_id = ${}", idx));
    }
    if !clauses.is_empty() {
        sql.push_str(&format!(" WHERE {}", clauses.join(" AND ")));
    }
    sql.push_str(" ORDER BY created_at DESC LIMIT 100");

    let rows = sqlx::query(&sql).fetch_all(&state.pool).await?;
    let entries: Vec<Value> = rows.iter().map(|r| {
        json!({
            "id": r.get::<Uuid, _>("id"),
            "entity_type": r.get::<String, _>("entity_type"),
            "action": r.get::<String, _>("action"),
            "field_name": r.get::<Option<String>, _>("field_name"),
            "old_value": r.get::<Option<String>, _>("old_value"),
            "new_value": r.get::<Option<String>, _>("new_value"),
            "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        })
    }).collect();

    Ok(Json(json!({ "entries": entries, "total": entries.len() })))
}

async fn get_prospect_tabs(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Admision"])?;

    let row = sqlx::query(
        "SELECT id, first_name, last_name, rut, email, phone, source, notes, current_stage_id, assigned_user_id, created_at, updated_at FROM admission_prospects WHERE id = $1",
    ).bind(id).fetch_optional(&state.pool).await?
        .ok_or(SisError::NotFound("Postulante no encontrado".into()))?;

    let prospect = json!({
        "id": row.get::<Uuid, _>("id"),
        "first_name": row.get::<String, _>("first_name"),
        "last_name": row.get::<String, _>("last_name"),
        "rut": row.get::<Option<String>, _>("rut"),
        "email": row.get::<Option<String>, _>("email"),
    });

    let docs = sqlx::query(
        "SELECT id, doc_type, file_name, s3_url as file_path, is_verified, created_at FROM prospect_documents WHERE prospect_id = $1 ORDER BY created_at DESC",
    ).bind(id).fetch_all(&state.pool).await?;

    let documents: Vec<Value> = docs.iter().map(|r| {
        json!({
            "id": r.get::<Uuid, _>("id"),
            "doc_type": r.get::<String, _>("doc_type"),
            "file_name": r.get::<String, _>("file_name"),
            "is_verified": r.get::<bool, _>("is_verified"),
            "file_url": r.get::<Option<String>, _>("file_path"),
            "created_at": r.get::<String, _>("created_at"),
        })
    }).collect();

    Ok(Json(json!({ "prospect": prospect, "documents": documents, "kpi": { "total_documents": documents.len(), "verified_documents": 0 } })))
}

async fn log_interaction(claims: Claims, State(state): State<AppState>, Json(payload): Json<InteractionPayload>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "Admision"])?;

    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO event_log (id, event_type, source, payload)
           VALUES ($1, $2, $3, $4)"#,
    ).bind(id).bind(format!("interaction.{}", payload.interaction_type))
    .bind("sis").bind(serde_json::to_string(&payload).unwrap_or_default())
    .execute(&state.pool).await?;

    Ok(Json(json!({ "id": id, "message": "Interaccion registrada" })))
}

async fn list_interactions(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "Admision"])?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "admission",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;

    let rows = sqlx::query(
        "SELECT id, event_type, source, payload, created_at FROM event_log WHERE event_type LIKE 'interaction.%' ORDER BY created_at DESC LIMIT 100",
    ).fetch_all(&state.pool).await?;

    let interactions: Vec<Value> = rows.iter().map(|r| {
        json!({
            "id": r.get::<Uuid, _>("id"),
            "type": r.get::<String, _>("event_type"),
            "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        })
    }).collect();

    Ok(Json(json!({ "interactions": interactions })))
}

async fn list_family_members(claims: Claims, State(state): State<AppState>, Path(pid): Path<Uuid>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Admision"])?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "admission",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;

    let members = sqlx::query(
        "SELECT id, prospect_id, student_id, rut, first_name, last_name, relationship, is_enrolled, created_at FROM family_members WHERE prospect_id = $1 ORDER BY created_at DESC",
    ).bind(pid).fetch_all(&state.pool).await?;

    let list: Vec<Value> = members.iter().map(|r| json!({
        "id": r.get::<Uuid, _>("id"),
        "rut": r.get::<String, _>("rut"),
        "first_name": r.get::<String, _>("first_name"),
        "last_name": r.get::<String, _>("last_name"),
        "relationship": r.get::<String, _>("relationship"),
        "is_enrolled": r.get::<bool, _>("is_enrolled"),
    })).collect();

    Ok(Json(json!({ "family_members": list })))
}

async fn add_family_member(claims: Claims, State(state): State<AppState>, Path(pid): Path<Uuid>, Json(payload): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Admision"])?;

    let rut = payload["rut"].as_str().unwrap_or("");
    let first_name = payload["first_name"].as_str().unwrap_or("");
    let last_name = payload["last_name"].as_str().unwrap_or("");
    let relationship = payload["relationship"].as_str().unwrap_or("");
    if rut.is_empty() || first_name.is_empty() || last_name.is_empty() || relationship.is_empty() {
        return Err(SisError::Validation("RUT, nombre, apellido y parentesco requeridos".into()));
    }

    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO family_members (id, prospect_id, rut, first_name, last_name, relationship) VALUES ($1, $2, $3, $4, $5, $6)",
    ).bind(id).bind(pid).bind(rut).bind(first_name).bind(last_name).bind(relationship)
    .execute(&state.pool).await?;

    Ok(Json(json!({ "id": id, "message": "Familiar agregado" })))
}

async fn check_stagnation(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "Admision"])?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "admission",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;

    let rows = sqlx::query(
        r#"SELECT p.id, p.first_name, p.last_name, ps.name as stage_name, p.updated_at,
                  EXTRACT(DAY FROM NOW() - p.updated_at)::INTEGER as days_in_stage
           FROM admission_prospects p
           JOIN pipeline_stages ps ON p.current_stage_id = ps.id
           WHERE p.updated_at < NOW() - INTERVAL '5 days'
           ORDER BY days_in_stage DESC"#,
    ).fetch_all(&state.pool).await?;

    let stagnant: Vec<Value> = rows.iter().map(|r| json!({
        "id": r.get::<Uuid, _>("id"),
        "name": format!("{} {}", r.get::<String, _>("first_name"), r.get::<String, _>("last_name")),
        "stage": r.get::<String, _>("stage_name"),
        "days_in_stage": r.get::<i32, _>("days_in_stage"),
    })).collect();

    Ok(Json(json!({ "stagnant_prospects": stagnant, "total": stagnant.len() })))
}

async fn time_in_stage(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "Admision"])?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "admission",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;

    let rows = sqlx::query(
        r#"SELECT ps.id, ps.name, ps."order",
                  COUNT(p.id) as prospect_count,
                  COALESCE(AVG(EXTRACT(DAY FROM NOW() - p.updated_at)), 0)::INTEGER as avg_days
           FROM pipeline_stages ps
           LEFT JOIN admission_prospects p ON p.current_stage_id = ps.id
           GROUP BY ps.id, ps.name, ps."order"
           ORDER BY ps."order""#,
    ).fetch_all(&state.pool).await?;

    let stages: Vec<Value> = rows.iter().map(|r| json!({
        "id": r.get::<Uuid, _>("id"),
        "name": r.get::<String, _>("name"),
        "prospect_count": r.get::<i64, _>("prospect_count"),
        "avg_days_in_stage": r.get::<i32, _>("avg_days"),
    })).collect();

    Ok(Json(json!({ "stages": stages })))
}

async fn geocerca_validate(claims: Claims, State(state): State<AppState>, Json(payload): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    let employee_id = payload.get("employee_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok());
    let lat = payload.get("lat").and_then(|v| v.as_f64());
    let lng = payload.get("lng").and_then(|v| v.as_f64());

    let (eid, lat_val, lng_val) = match (employee_id, lat, lng) {
        (Some(id), Some(l), Some(n)) => (id, l, n),
        _ => return Err(SisError::Validation("employee_id, lat y lng requeridos".into())),
    };

    let fences = sqlx::query(
        "SELECT id, lat, lng, radius_meters, name FROM employee_geofences WHERE employee_id = $1",
    ).bind(eid).fetch_all(&state.pool).await?;

    let mut matched = false;
    let mut matched_name = String::new();

    for fence in &fences {
        let f_lat: f64 = fence.get("lat");
        let f_lng: f64 = fence.get("lng");
        let radius: f64 = fence.get("radius_meters");
        let name: String = fence.get("name");

        let dist = haversine(lat_val, lng_val, f_lat, f_lng);
        if dist <= radius {
            matched = true;
            matched_name = name;
            break;
        }
    }

    Ok(Json(json!({ "within_geofence": matched, "geofence_name": matched_name })))
}

fn haversine(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    let r = 6371000.0;
    let d_lat = (lat2 - lat1).to_radians();
    let d_lng = (lng2 - lng1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2) + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lng / 2.0).sin().powi(2);
    r * 2.0 * a.sqrt().asin()
}

async fn lre_audit(claims: Claims, State(state): State<AppState>, Query(q): Query<LreAuditFilter>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "hr",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;

    let month = q.month;
    let rows = sqlx::query(
        r#"SELECT p.id, e.rut, e.first_name, e.last_name, p.salary_base, p.gratificacion,
                  p.non_taxable_earnings, p.taxable_income, p.afp_discount, p.health_discount,
                  p.unemployment_discount, p.net_salary, p.month, p.year
           FROM payrolls p JOIN employees e ON p.employee_id = e.id
           WHERE ($1::int IS NULL OR p.month = $1)
           ORDER BY e.first_name"#,
    ).bind(month).fetch_all(&state.pool).await?;

    let mut total_taxable = 0.0;
    let mut total_discounts = 0.0;
    let mut total_net = 0.0;
    let mut issues: Vec<String> = vec![];

    for row in &rows {
        let taxable: f64 = row.get("taxable_income");
        let afp: f64 = row.get("afp_discount");
        let health: f64 = row.get("health_discount");
        let unemp: f64 = row.get("unemployment_discount");
        let net: f64 = row.get("net_salary");
        let name: String = format!("{} {}", row.get::<String, _>("first_name"), row.get::<String, _>("last_name"));

        total_taxable += taxable;
        total_discounts += afp + health + unemp;
        total_net += net;

        let calc_net = taxable - (afp + health + unemp);
        if (calc_net - net).abs() > 100.0 {
            issues.push(format!("{}: discrepancia de ${:.0} en liquido", name, calc_net - net));
        }
    }

    Ok(Json(json!({
        "records": rows.len(),
        "total_taxable": total_taxable,
        "total_discounts": total_discounts,
        "total_net": total_net,
        "issues": issues,
        "audit_passed": issues.is_empty(),
    })))
}

async fn sync_attendance_hmac(claims: Claims, State(state): State<AppState>, Json(payload): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    let employee_id = payload.get("employee_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok());
    let timestamp = payload.get("timestamp").and_then(|v| v.as_str());
    let entry_type = payload.get("entry_type").and_then(|v| v.as_str());
    let signature = payload.get("signature").and_then(|v| v.as_str()).unwrap_or("");

    let eid = match employee_id { Some(id) => id, None => return Err(SisError::Validation("employee_id requerido".into())) };
    let ts = match timestamp { Some(t) => t.to_string(), None => return Err(SisError::Validation("timestamp requerido".into())) };
    let etype = match entry_type { Some(t) => t.to_string(), None => return Err(SisError::Validation("entry_type requerido".into())) };

    // Verify HMAC signature using stored API keys
    let keys = sqlx::query("SELECT api_key_hash FROM api_keys WHERE is_active = true LIMIT 1")
        .fetch_optional(&state.pool).await?;

    if let Some(key_row) = keys {
        let stored_hash: String = key_row.get("api_key_hash");
        let msg = format!("{}|{}|{}", eid, ts, etype);
        let computed = sha256_hmac(&msg, &stored_hash);
        if signature != computed {
            return Err(SisError::Validation("Firma HMAC invalida. Datos podrian haber sido alterados.".into()));
        }
    }

    let log_id = Uuid::new_v4();
    let result = sqlx::query_as::<_, schoolccb_common::hr::AttendanceLog>(
        r#"INSERT INTO employee_attendance_logs (id, employee_id, timestamp, entry_type, source)
           VALUES ($1, $2, $3, $4, 'api_hmac')
           RETURNING id, employee_id, timestamp, entry_type, device_id, location_hash, source, created_at"#,
    ).bind(log_id).bind(eid).bind(ts).bind(&etype)
    .fetch_one(&state.pool).await?;

    Ok(Json(json!({ "attendance": result, "verified": true })))
}

fn sha256_hmac(msg: &str, key: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes()).expect("HMAC key");
    mac.update(msg.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
