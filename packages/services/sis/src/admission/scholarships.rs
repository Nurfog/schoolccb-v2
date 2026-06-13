use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post, put},
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::error::SisResult;
use schoolccb_common::auth::{Claims, require_any_role};
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

    // Get contract to recalculate amounts
    let contract_data = sqlx::query_as::<_, (f64,)>(
        "SELECT total_fee FROM enrollment_contracts WHERE student_id = $1 AND status = 'draft' LIMIT 1",
    )
    .bind(student_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(crate::error::SisError::NotFound("No hay un contrato en borrador para este estudiante".into()))?;

    let total_fee = contract_data.0;
    let discount_val: f64 = sqlx::query_as::<_, (f64,)>(
        "SELECT discount_value FROM admission_scholarships WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(crate::error::SisError::NotFound("Beca no encontrada".into()))?
    .0;

    let discount_amount = total_fee * discount_val / 100.0;
    let final_amount = total_fee - discount_amount;

    // Apply to enrollment contract with recalculated amounts
    sqlx::query(
        "UPDATE enrollment_contracts SET scholarship_id = $1, discount_amount = $2, final_amount = $3,
         updated_at = NOW() WHERE student_id = $4 AND status = 'draft'",
    )
    .bind(id)
    .bind(discount_amount)
    .bind(final_amount)
    .bind(student_id)
    .execute(&state.pool)
    .await?;

    sqlx::query("UPDATE admission_scholarships SET current_beneficiaries = current_beneficiaries + 1 WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({"message": "Beca aplicada", "discount": discount_amount, "final_amount": final_amount})))
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

    let total_fee = payload.get("total_fee").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let final_amount = payload.get("final_amount").and_then(|v| v.as_f64()).unwrap_or(total_fee);
    let discount_amount = payload.get("discount_amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let scholarship_id = payload.get("scholarship_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok());

    // If scholarship_id is provided but no discount, auto-calculate
    let (final_discount, final_total) = if scholarship_id.is_some() && discount_amount == 0.0 {
        let discount_val: Option<f64> = sqlx::query_scalar(
            "SELECT discount_value FROM admission_scholarships WHERE id = $1 AND is_active = true",
        )
        .bind(scholarship_id)
        .fetch_optional(&state.pool)
        .await?;

        match discount_val {
            Some(dv) => {
                let d = total_fee * dv / 100.0;
                (d, total_fee - d)
            }
            None => (discount_amount, final_amount),
        }
    } else {
        (discount_amount, final_amount)
    };

    sqlx::query(
        "INSERT INTO enrollment_contracts (id, student_id, school_id, grade_level, guardian_user_id,
         scholarship_id, total_fee, discount_amount, final_amount, payment_plan, notes)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(id)
    .bind(payload.get("student_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()))
    .bind(payload.get("school_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()))
    .bind(payload.get("grade_level").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(payload.get("guardian_user_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()))
    .bind(scholarship_id)
    .bind(total_fee)
    .bind(final_discount)
    .bind(final_total)
    .bind(payload.get("payment_plan").and_then(|v| v.as_str()).unwrap_or("monthly"))
    .bind(payload.get("notes").and_then(|v| v.as_str()))
    .execute(&state.pool)
    .await?;

    schoolccb_common::audit::log(&state.pool, &schoolccb_common::audit::AuditEntry {
        entity_type: "enrollment_contract".into(),
        entity_id: id,
        action: "create".into(),
        user_id: Some(Uuid::parse_str(&claims.sub).unwrap_or_default()),
        changes: Some(serde_json::json!({"total_fee": total_fee, "final_amount": final_total, "has_scholarship": scholarship_id.is_some()})),
    }).await;

    Ok(Json(json!({"id": id, "discount_amount": final_discount, "final_amount": final_total})))
}

async fn get_contract(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Admision", "GerenteGeneral"])?;

    let contract = sqlx::query_as::<_, (Uuid, String, String, String, f64, f64, f64, String, String, Option<Uuid>, Option<String>)>(
        "SELECT ec.id, s.first_name || ' ' || s.last_name, ec.grade_level, ec.status,
                ec.total_fee, ec.discount_amount, ec.final_amount, ec.payment_plan, COALESCE(ec.notes, ''),
                ec.scholarship_id, as2.name
         FROM enrollment_contracts ec
         JOIN students s ON s.id = ec.student_id
         LEFT JOIN admission_scholarships as2 ON as2.id = ec.scholarship_id
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
        "scholarship_id": contract.9, "scholarship_name": contract.10,
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

    // Update contract status to paid
    sqlx::query(
        "UPDATE enrollment_contracts SET status = 'paid', updated_at = NOW() WHERE id = $1 AND status = 'draft'",
    )
    .bind(id)
    .execute(&state.pool).await?;

    schoolccb_common::audit::log(&state.pool, &schoolccb_common::audit::AuditEntry {
        entity_type: "enrollment_payment".into(),
        entity_id: id,
        action: "pay".into(),
        user_id: Some(Uuid::parse_str(&claims.sub).unwrap_or_default()),
        changes: Some(serde_json::json!({"student_id": student_id, "amount": amount, "method": method})),
    }).await;

    // If Webpay, return gateway URL to redirect
    let resp = json!({"fee_id": fee_id, "payment_id": payment_id, "message": "Pago registrado"});
    if method == "Webpay" {
        let gateway = json!({"gateway_url": format!("/api/finance/payment/init/{}", fee_id)});
        Ok(Json(json!({"fee_id": fee_id, "payment_id": payment_id, "message": "Redirigiendo a Webpay...", "gateway_url": gateway["gateway_url"]})))
    } else {
        Ok(Json(resp))
    }
}

async fn enroll_student(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Admision", "GerenteGeneral"])?;

    let contract = sqlx::query_as::<_, (Uuid, String, String, String)>(
        "SELECT ec.student_id, ec.grade_level, ec.school_id::text, ec.status
         FROM enrollment_contracts ec WHERE ec.id = $1 AND ec.status IN ('draft', 'paid')",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(crate::error::SisError::NotFound("Contrato no encontrado o ya procesado".into()))?;

    let status = &contract.3;
    if status == "draft" {
        return Err(crate::error::SisError::Validation(
            "El contrato debe estar pagado antes de matricular. Registre el pago primero.".into(),
        ));
    }

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

    schoolccb_common::audit::log(&state.pool, &schoolccb_common::audit::AuditEntry {
        entity_type: "enrollment".into(),
        entity_id: id,
        action: "enroll".into(),
        user_id: Some(Uuid::parse_str(&claims.sub).unwrap_or_default()),
        changes: Some(serde_json::json!({"student_id": contract.0, "grade": contract.1})),
    }).await;

    Ok(Json(json!({"message": "Alumno matriculado exitosamente"})))
}
