use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post, put},
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::error::SisResult;
use crate::routes::students::{Claims, require_any_role};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/admission/scholarships", get(list_scholarships).post(create_scholarship))
        .route("/api/admission/scholarships/{id}", put(update_scholarship))
        .route("/api/admission/scholarships/{id}/toggle", put(toggle_scholarship))
        .route("/api/admission/scholarships/{id}/apply", post(apply_scholarship))
        .route("/api/admission/contracts", get(list_contracts).post(create_contract))
        .route("/api/admission/contracts/{id}", get(get_contract))
        .route("/api/admission/contracts/{id}/enroll", post(enroll_student))
        .route("/api/admission/contracts/{id}/pay", post(register_contract_payment))
}

async fn list_scholarships(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Admision", "GerenteGeneral"])?;
    let school_id = claims.school_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    let scholarships = sqlx::query_as::<_, (Uuid, String, String, f64, i32, i32, bool)>(
        "SELECT id, name, description, discount_value, max_beneficiaries, current_beneficiaries, is_active
         FROM admission_scholarships WHERE ($1::uuid IS NULL OR school_id = $1)
         ORDER BY name",
    )
    .bind(school_id)
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|(id, name, desc, val, max, cur, active)| json!({
        "id": id, "name": name, "description": desc, "discount": val,
        "max": max, "current": cur, "active": active,
    }))
    .collect::<Vec<_>>();

    Ok(Json(json!({"scholarships": scholarships})))
}

async fn create_scholarship(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "GerenteGeneral"])?;
    let school_id = claims.school_id.as_deref().and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(crate::error::SisError::Validation("School ID requerido".into()))?;
    let id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO admission_scholarships (id, school_id, name, description, discount_type, discount_value, max_beneficiaries, requirements)
         VALUES ($1, $2, $3, $4, 'percentage', $5, $6, $7)",
    )
    .bind(id)
    .bind(school_id)
    .bind(payload.get("name").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(payload.get("description").and_then(|v| v.as_str()))
    .bind(payload.get("discount").and_then(|v| v.as_f64()).unwrap_or(0.0))
    .bind(payload.get("max_beneficiaries").and_then(|v| v.as_i64()).unwrap_or(0))
    .bind(payload.get("requirements").and_then(|v| v.as_object()).map(|_| payload.get("requirements")))
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"id": id})))
}

async fn update_scholarship(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Value>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "GerenteGeneral"])?;
    sqlx::query(
        "UPDATE admission_scholarships SET name = COALESCE($1, name), description = COALESCE($2, description),
         discount_value = COALESCE($3, discount_value), max_beneficiaries = COALESCE($4, max_beneficiaries),
         updated_at = NOW() WHERE id = $5",
    )
    .bind(payload.get("name").and_then(|v| v.as_str()))
    .bind(payload.get("description").and_then(|v| v.as_str()))
    .bind(payload.get("discount").and_then(|v| v.as_f64()))
    .bind(payload.get("max_beneficiaries").and_then(|v| v.as_i64()).map(|v| v as i32))
    .bind(id)
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"message": "Beca actualizada"})))
}

async fn toggle_scholarship(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "GerenteGeneral"])?;
    sqlx::query("UPDATE admission_scholarships SET is_active = NOT is_active, updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({"message": "Beca toggleada"})))
}

async fn apply_scholarship(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Value>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Admision", "GerenteGeneral"])?;
    let student_id = payload.get("student_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(crate::error::SisError::Validation("student_id requerido".into()))?;

    // Check capacity
    let (max, current): (i32, i32) = sqlx::query_as(
        "SELECT max_beneficiaries, current_beneficiaries FROM admission_scholarships WHERE id = $1 AND is_active = true",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(crate::error::SisError::NotFound("Beca no encontrada".into()))?;

    if max > 0 && current >= max {
        return Err(crate::error::SisError::Validation("La beca ha alcanzado su máximo de beneficiarios".into()));
    }

    // Apply to enrollment contract
    sqlx::query("UPDATE enrollment_contracts SET scholarship_id = $1, updated_at = NOW() WHERE student_id = $2 AND status = 'draft'")
        .bind(id)
        .bind(student_id)
        .execute(&state.pool)
        .await?;

    sqlx::query("UPDATE admission_scholarships SET current_beneficiaries = current_beneficiaries + 1 WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({"message": "Beca aplicada"})))
}

async fn list_contracts(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Admision", "GerenteGeneral"])?;

    let contracts = sqlx::query_as::<_, (Uuid, String, String, String, f64, String)>(
        "SELECT ec.id, s.first_name || ' ' || s.last_name, ec.grade_level, ec.status,
                ec.final_amount, ec.created_at::text
         FROM enrollment_contracts ec
         JOIN students s ON s.id = ec.student_id
         ORDER BY ec.created_at DESC LIMIT 50",
    )
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|(id, name, grade, status, amount, date)| json!({
        "id": id, "student": name, "grade": grade, "status": status, "amount": amount, "date": date,
    }))
    .collect::<Vec<_>>();

    Ok(Json(json!({"contracts": contracts})))
}

async fn create_contract(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Admision", "GerenteGeneral"])?;
    let id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO enrollment_contracts (id, student_id, school_id, grade_level, guardian_user_id,
         total_fee, discount_amount, final_amount, payment_plan, notes)
         VALUES ($1, $2, $3, $4, $5, $6, 0, $7, $8, $9)",
    )
    .bind(id)
    .bind(payload.get("student_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()))
    .bind(payload.get("school_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()))
    .bind(payload.get("grade_level").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(payload.get("guardian_user_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()))
    .bind(payload.get("total_fee").and_then(|v| v.as_f64()).unwrap_or(0.0))
    .bind(payload.get("final_amount").and_then(|v| v.as_f64()).unwrap_or(0.0))
    .bind(payload.get("payment_plan").and_then(|v| v.as_str()).unwrap_or("monthly"))
    .bind(payload.get("notes").and_then(|v| v.as_str()))
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"id": id})))
}

async fn get_contract(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Admision", "GerenteGeneral"])?;

    let contract = sqlx::query_as::<_, (Uuid, String, String, String, f64, f64, f64, String, String)>(
        "SELECT ec.id, s.first_name || ' ' || s.last_name, ec.grade_level, ec.status,
                ec.total_fee, ec.discount_amount, ec.final_amount, ec.payment_plan, COALESCE(ec.notes, '')
         FROM enrollment_contracts ec
         JOIN students s ON s.id = ec.student_id
         WHERE ec.id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(crate::error::SisError::NotFound("Contrato no encontrado".into()))?;

    Ok(Json(json!({
        "id": contract.0, "student": contract.1, "grade": contract.2, "status": contract.3,
        "total_fee": contract.4, "discount": contract.5, "final_amount": contract.6,
        "payment_plan": contract.7, "notes": contract.8,
    })))
}

async fn register_contract_payment(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Value>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Admision", "GerenteGeneral"])?;

    let contract = sqlx::query_as::<_, (Uuid, f64, String)>(
        "SELECT ec.student_id, ec.final_amount, ec.school_id::text
         FROM enrollment_contracts ec WHERE ec.id = $1 AND ec.status = 'draft'",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(crate::error::SisError::NotFound("Contrato no encontrado o ya pagado".into()))?;

    let student_id = contract.0;
    let amount = payload.get("amount").and_then(|v| v.as_f64()).unwrap_or(contract.1);
    let method = payload.get("method").and_then(|v| v.as_str()).unwrap_or("Efectivo");
    let fee_id = Uuid::new_v4();
    let payment_id = Uuid::new_v4();

    // Create fee and mark as paid in one step
    sqlx::query(
        "INSERT INTO fees (id, student_id, description, amount, due_date, paid, paid_date, paid_amount)
         VALUES ($1, $2, 'Matrícula', $3, CURRENT_DATE, true, CURRENT_DATE, $3)",
    )
    .bind(fee_id).bind(student_id).bind(amount)
    .execute(&state.pool).await?;

    sqlx::query(
        "INSERT INTO payments (id, fee_id, student_id, amount, payment_date, payment_method)
         VALUES ($1, $2, $3, $4, CURRENT_DATE, $5)",
    )
    .bind(payment_id).bind(fee_id).bind(student_id).bind(amount).bind(method)
    .execute(&state.pool).await?;

    Ok(Json(json!({"fee_id": fee_id, "payment_id": payment_id, "message": "Pago registrado"})))
}

async fn enroll_student(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Admision", "GerenteGeneral"])?;

    let contract = sqlx::query_as::<_, (Uuid, String, String)>(
        "SELECT ec.student_id, ec.grade_level, ec.school_id::text
         FROM enrollment_contracts ec WHERE ec.id = $1 AND ec.status = 'draft'",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(crate::error::SisError::NotFound("Contrato no encontrado o ya procesado".into()))?;

    // Mark contract as enrolled
    sqlx::query("UPDATE enrollment_contracts SET status = 'enrolled', enrolled_at = NOW(), updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    sqlx::query(
        "UPDATE enrollments SET active = true WHERE student_id = $1",
    )
    .bind(contract.0)
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"message": "Alumno matriculado exitosamente"})))
}
