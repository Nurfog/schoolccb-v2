use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post, put},
};
use chrono::{Datelike, NaiveDate, Utc};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::Row;
use uuid::Uuid;

use crate::AppState;
use crate::error::{SisError, SisResult};
use schoolccb_common::auth::{Claims, require_any_role};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/hr/payroll/calculate", post(calculate_payroll_preview))
        .route("/api/hr/payroll", post(create_payroll).get(list_payrolls))
        .route("/api/hr/payroll/{id}", get(get_payroll))
        .route("/api/hr/payroll/export/lre", get(export_lre))
        .route("/api/hr/payroll/export/previred", get(export_previred))
        .route("/api/hr/leave-requests", get(list_all_leave_requests))
        .route("/api/hr/leave-requests/{id}/approve", put(approve_leave_request))
        .route("/api/hr/leave-requests/{id}/notify", post(notify_leave_request))
        .route("/api/hr/employees/{id}/leave-requests", post(create_leave_request).get(list_employee_leave_requests))
        .route("/api/hr/employees/{id}/attendance", get(list_attendance_logs))
        .route("/api/hr/employees/{id}/attendance/summary", get(attendance_summary))
        .route("/api/hr/attendance/sync", post(sync_attendance))
        .route("/api/hr/attendance/{att_id}/modify", put(modify_attendance))
        .route("/api/hr/employees/{id}/pension-fund", get(get_pension_fund).post(set_pension_fund))
        .route("/api/hr/employees/{id}/link-user", post(link_employee_user))
        .route("/api/hr/me", get(my_profile))
        .route("/api/hr/me/payroll", get(my_payroll))
        .route("/api/hr/me/attendance", get(my_attendance))
        .route("/api/hr/me/leave-requests", get(my_leave_requests).post(my_create_leave_request))
        .route("/api/hr/me/documents", get(my_documents).post(my_upload_document))
        .route("/api/hr/complaints", get(list_complaints))
        .route("/api/hr/complaints/submit", post(submit_complaint))
        .route("/api/hr/contract/generate", post(generate_contract))
}

#[derive(Deserialize)]
struct PayrollFilter {
    month: Option<i32>,
    year: Option<i32>,
}

async fn calculate_payroll_preview(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::hr::PayrollPayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    let _employee = sqlx::query_as::<_, schoolccb_common::hr::Employee>(
        "SELECT id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id, created_at, updated_at FROM employees WHERE id = $1",
    ).bind(payload.employee_id).fetch_optional(&state.pool).await?
        .ok_or(SisError::NotFound("Empleado no encontrado".into()))?;

    let contract = sqlx::query_as::<_, schoolccb_common::hr::EmployeeContract>(
        "SELECT id, employee_id, contract_type, salary_base, weekly_hours, ley_karin_signed, start_date, end_date, active, created_at FROM employee_contracts WHERE employee_id = $1 AND active = true",
    ).bind(payload.employee_id).fetch_optional(&state.pool).await?
        .ok_or(SisError::NotFound("Empleado no tiene contrato activo".into()))?;

    let pension = sqlx::query_as::<_, schoolccb_common::hr::EmployeePensionFund>(
        "SELECT id, employee_id, pension_fund, health_system, health_plan_name, health_fixed_amount, created_at FROM employee_pension_funds WHERE employee_id = $1",
    ).bind(payload.employee_id).fetch_optional(&state.pool).await?;

    let (pension_str, health_str, health_fixed) = match pension {
        Some(ref p) => (p.pension_fund.clone(), p.health_system.clone(), p.health_fixed_amount),
        None => ("Provida".into(), "Fonasa".into(), None),
    };
    let pension_fund = schoolccb_common::hr::PensionFund::from_str(&pension_str)
        .unwrap_or(schoolccb_common::hr::PensionFund::Provida);
    let health_system = schoolccb_common::hr::HealthSystem::from_str(&health_str)
        .unwrap_or(schoolccb_common::hr::HealthSystem::Fonasa);

    let calculation = schoolccb_common::hr::calculate_payroll(
        &contract, &payload, &pension_fund, &health_system, health_fixed,
    );

    Ok(Json(json!(calculation)))
}

async fn create_payroll(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::hr::PayrollPayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    sqlx::query_as::<_, schoolccb_common::hr::Employee>(
        "SELECT id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id, created_at, updated_at FROM employees WHERE id = $1",
    ).bind(payload.employee_id).fetch_optional(&state.pool).await?
        .ok_or(SisError::NotFound("Empleado no encontrado".into()))?;

    let contract = sqlx::query_as::<_, schoolccb_common::hr::EmployeeContract>(
        "SELECT id, employee_id, contract_type, salary_base, weekly_hours, ley_karin_signed, start_date, end_date, active, created_at FROM employee_contracts WHERE employee_id = $1 AND active = true",
    ).bind(payload.employee_id).fetch_optional(&state.pool).await?
        .ok_or(SisError::NotFound("Empleado no tiene contrato activo".into()))?;

    let pension = sqlx::query_as::<_, schoolccb_common::hr::EmployeePensionFund>(
        "SELECT id, employee_id, pension_fund, health_system, health_plan_name, health_fixed_amount, created_at FROM employee_pension_funds WHERE employee_id = $1",
    ).bind(payload.employee_id).fetch_optional(&state.pool).await?;

    let (pension_str, health_str, health_fixed) = match pension {
        Some(ref p) => (p.pension_fund.clone(), p.health_system.clone(), p.health_fixed_amount),
        None => ("Provida".into(), "Fonasa".into(), None),
    };
    let pension_fund = schoolccb_common::hr::PensionFund::from_str(&pension_str)
        .unwrap_or(schoolccb_common::hr::PensionFund::Provida);
    let health_system = schoolccb_common::hr::HealthSystem::from_str(&health_str)
        .unwrap_or(schoolccb_common::hr::HealthSystem::Fonasa);

    let calc = schoolccb_common::hr::calculate_payroll(
        &contract, &payload, &pension_fund, &health_system, health_fixed,
    );

    let payroll_id = Uuid::new_v4();
    let result = sqlx::query_as::<_, schoolccb_common::hr::Payroll>(
        r#"INSERT INTO payrolls (id, employee_id, month, year, salary_base, gratificacion, non_taxable_earnings, taxable_income, afp_discount, health_discount, unemployment_discount, income_tax, other_deductions, net_salary)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
           RETURNING id, employee_id, month, year, salary_base, gratificacion, non_taxable_earnings, taxable_income, afp_discount, health_discount, unemployment_discount, income_tax, other_deductions, net_salary, lre_exported, previred_exported, created_at, updated_at"#,
    ).bind(payroll_id)
     .bind(payload.employee_id)
     .bind(payload.month).bind(payload.year)
     .bind(calc.salary_base).bind(calc.gratificacion)
     .bind(calc.non_taxable_earnings).bind(calc.taxable_income)
     .bind(calc.afp_discount).bind(calc.health_discount)
     .bind(calc.unemployment_discount).bind(calc.income_tax)
     .bind(calc.other_deductions).bind(calc.net_salary)
    .fetch_one(&state.pool).await?;

    Ok(Json(json!({ "payroll": result, "message": "Liquidacion generada exitosamente" })))
}

async fn list_payrolls(
    claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<PayrollFilter>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "hr",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;

    let now = Utc::now();
    let month = filter.month.unwrap_or(now.month() as i32);
    let year = filter.year.unwrap_or(now.year());

    let rows = sqlx::query(
        r#"SELECT p.*, e.first_name, e.last_name, e.rut
           FROM payrolls p
           JOIN employees e ON p.employee_id = e.id
           WHERE p.month = $1 AND p.year = $2
           ORDER BY e.first_name, e.last_name"#,
    )
    .bind(month)
    .bind(year)
    .fetch_all(&state.pool)
    .await?;

    let payrolls: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.get::<Uuid, _>("id"),
                "employee_id": r.get::<Uuid, _>("employee_id"),
                "employee_name": format!("{} {}", r.get::<String, _>("first_name"), r.get::<String, _>("last_name")),
                "rut": r.get::<String, _>("rut"),
                "month": r.get::<i32, _>("month"),
                "year": r.get::<i32, _>("year"),
                "salary_base": r.get::<f64, _>("salary_base"),
                "gratificacion": r.get::<f64, _>("gratificacion"),
                "non_taxable_earnings": r.get::<f64, _>("non_taxable_earnings"),
                "taxable_income": r.get::<f64, _>("taxable_income"),
                "afp_discount": r.get::<f64, _>("afp_discount"),
                "health_discount": r.get::<f64, _>("health_discount"),
                "unemployment_discount": r.get::<f64, _>("unemployment_discount"),
                "income_tax": r.get::<f64, _>("income_tax"),
                "other_deductions": r.get::<f64, _>("other_deductions"),
                "net_salary": r.get::<f64, _>("net_salary"),
                "lre_exported": r.get::<bool, _>("lre_exported"),
                "previred_exported": r.get::<bool, _>("previred_exported"),
            })
        })
        .collect();

    Ok(Json(json!({ "payrolls": payrolls, "total": payrolls.len() })))
}

async fn get_payroll(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    let row = sqlx::query(
        r#"SELECT p.*, e.first_name, e.last_name, e.rut
           FROM payrolls p
           JOIN employees e ON p.employee_id = e.id
           WHERE p.id = $1"#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(SisError::NotFound("Liquidacion no encontrada".into()))?;

    Ok(Json(json!({
        "id": row.get::<Uuid, _>("id"),
        "employee_id": row.get::<Uuid, _>("employee_id"),
        "employee_name": format!("{} {}", row.get::<String, _>("first_name"), row.get::<String, _>("last_name")),
        "rut": row.get::<String, _>("rut"),
        "month": row.get::<i32, _>("month"),
        "year": row.get::<i32, _>("year"),
        "salary_base": row.get::<f64, _>("salary_base"),
        "gratificacion": row.get::<f64, _>("gratificacion"),
        "non_taxable_earnings": row.get::<f64, _>("non_taxable_earnings"),
        "taxable_income": row.get::<f64, _>("taxable_income"),
        "afp_discount": row.get::<f64, _>("afp_discount"),
        "health_discount": row.get::<f64, _>("health_discount"),
        "unemployment_discount": row.get::<f64, _>("unemployment_discount"),
        "income_tax": row.get::<f64, _>("income_tax"),
        "other_deductions": row.get::<f64, _>("other_deductions"),
        "net_salary": row.get::<f64, _>("net_salary"),
        "lre_exported": row.get::<bool, _>("lre_exported"),
        "previred_exported": row.get::<bool, _>("previred_exported"),
    })))
}

async fn export_lre(
    claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<PayrollFilter>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    let now = Utc::now();
    let month = filter.month.unwrap_or(now.month() as i32);
    let year = filter.year.unwrap_or(now.year());

    let rows = sqlx::query(
        r#"SELECT p.*, e.first_name, e.last_name, e.rut
           FROM payrolls p
           JOIN employees e ON p.employee_id = e.id
           WHERE p.month = $1 AND p.year = $2
           ORDER BY e.rut"#,
    )
    .bind(month)
    .bind(year)
    .fetch_all(&state.pool)
    .await?;

    let mut csv = String::from("RUT,Nombre,Sueldo Base,Gratificacion,Imponible,AFP,Salud,Seguro Cesantia,Impuesto,Otros Descuentos,Liquido\n");
    for row in &rows {
        csv.push_str(&format!(
            "{},{},{:.0},{:.0},{:.0},{:.0},{:.0},{:.0},{:.0},{:.0},{:.0}\n",
            row.get::<String, _>("rut"),
            format!("{} {}", row.get::<String, _>("first_name"), row.get::<String, _>("last_name")),
            row.get::<f64, _>("salary_base"),
            row.get::<f64, _>("gratificacion"),
            row.get::<f64, _>("taxable_income"),
            row.get::<f64, _>("afp_discount"),
            row.get::<f64, _>("health_discount"),
            row.get::<f64, _>("unemployment_discount"),
            row.get::<f64, _>("income_tax"),
            row.get::<f64, _>("other_deductions"),
            row.get::<f64, _>("net_salary"),
        ));

        sqlx::query("UPDATE payrolls SET lre_exported = true WHERE id = $1")
            .bind(row.get::<Uuid, _>("id"))
            .execute(&state.pool)
            .await?;
    }

    Ok(Json(json!({ "csv": csv, "count": rows.len() })))
}

async fn export_previred(
    claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<PayrollFilter>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    let now = Utc::now();
    let month = filter.month.unwrap_or(now.month() as i32);
    let year = filter.year.unwrap_or(now.year());

    let rows = sqlx::query(
        r#"SELECT p.*, e.first_name, e.last_name, e.rut
           FROM payrolls p
           JOIN employees e ON p.employee_id = e.id
           WHERE p.month = $1 AND p.year = $2
           ORDER BY e.rut"#,
    )
    .bind(month)
    .bind(year)
    .fetch_all(&state.pool)
    .await?;

    let mut csv = String::from("RUT,Nombre,Sueldo Bruto,AFP,Salud,Seguro Cesantia,Liquido\n");
    for row in &rows {
        csv.push_str(&format!(
            "{},{},{:.0},{:.0},{:.0},{:.0},{:.0}\n",
            row.get::<String, _>("rut"),
            format!("{} {}", row.get::<String, _>("first_name"), row.get::<String, _>("last_name")),
            row.get::<f64, _>("taxable_income"),
            row.get::<f64, _>("afp_discount"),
            row.get::<f64, _>("health_discount"),
            row.get::<f64, _>("unemployment_discount"),
            row.get::<f64, _>("net_salary"),
        ));

        sqlx::query("UPDATE payrolls SET previred_exported = true WHERE id = $1")
            .bind(row.get::<Uuid, _>("id"))
            .execute(&state.pool)
            .await?;
    }

    Ok(Json(json!({ "csv": csv, "count": rows.len() })))
}

#[derive(Deserialize)]
struct LeaveRequestFilter {
    status: Option<String>,
    employee_id: Option<Uuid>,
}

async fn create_leave_request(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::hr::CreateLeavePayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    let leave_id = Uuid::new_v4();
    let result = sqlx::query_as::<_, schoolccb_common::hr::LeaveRequest>(
        r#"INSERT INTO leave_requests (id, employee_id, leave_type, start_date, end_date, reason)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, employee_id, leave_type, start_date, end_date, reason, status, approved_by, approved_at, created_at, updated_at"#,
    ).bind(leave_id).bind(id)
     .bind(&payload.leave_type).bind(payload.start_date)
     .bind(payload.end_date).bind(&payload.reason)
    .fetch_one(&state.pool).await?;

    Ok(Json(json!({ "leave_request": result })))
}

async fn list_employee_leave_requests(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "hr",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;

    let requests = sqlx::query_as::<_, schoolccb_common::hr::LeaveRequest>(
        "SELECT id, employee_id, leave_type, start_date, end_date, reason, status, approved_by, approved_at, created_at, updated_at FROM leave_requests WHERE employee_id = $1 ORDER BY created_at DESC",
    ).bind(id).fetch_all(&state.pool).await?;

    Ok(Json(json!({ "leave_requests": requests })))
}

async fn list_all_leave_requests(
    claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<LeaveRequestFilter>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "hr",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;

    let mut sql = String::from(
        "SELECT lr.*, e.first_name, e.last_name FROM leave_requests lr JOIN employees e ON lr.employee_id = e.id WHERE 1=1",
    );
    let mut params: Vec<String> = vec![];

    if filter.status.is_some() {
        let n = params.len() + 1;
        params.push(format!("lr.status = ${n}"));
    }
    if filter.employee_id.is_some() {
        let n = params.len() + 1;
        params.push(format!("lr.employee_id = ${n}"));
    }

    for p in params {
        sql.push_str(&format!(" AND {}", p));
    }
    sql.push_str(" ORDER BY lr.created_at DESC");

    let mut query = sqlx::query(&sql);
    if let Some(ref status) = filter.status {
        query = query.bind(status);
    }
    if let Some(ref eid) = filter.employee_id {
        query = query.bind(eid);
    }

    let rows = query.fetch_all(&state.pool).await?;
    let requests: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.get::<Uuid, _>("id"),
                "employee_id": r.get::<Uuid, _>("employee_id"),
                "employee_name": format!("{} {}", r.get::<String, _>("first_name"), r.get::<String, _>("last_name")),
                "leave_type": r.get::<String, _>("leave_type"),
                "start_date": r.get::<chrono::NaiveDate, _>("start_date"),
                "end_date": r.get::<chrono::NaiveDate, _>("end_date"),
                "reason": r.get::<Option<String>, _>("reason"),
                "status": r.get::<String, _>("status"),
                "approved_by": r.get::<Option<Uuid>, _>("approved_by"),
                "approved_at": r.get::<Option<chrono::DateTime<Utc>>, _>("approved_at"),
                "created_at": r.get::<chrono::DateTime<Utc>, _>("created_at"),
            })
        })
        .collect();

    Ok(Json(json!({ "leave_requests": requests })))
}

async fn approve_leave_request(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    let status = payload.get("status").and_then(|v| v.as_str()).unwrap_or("Aprobado");
    let approved_by = Uuid::parse_str(&claims.sub).map_err(|_| SisError::Unauthorized)?;

    let current = sqlx::query_as::<_, schoolccb_common::hr::LeaveRequest>(
        "SELECT id, employee_id, leave_type, start_date, end_date, reason, status, approved_by, approved_at, created_at, updated_at FROM leave_requests WHERE id = $1",
    ).bind(id).fetch_optional(&state.pool).await?
        .ok_or(SisError::NotFound("Solicitud no encontrada".into()))?;

    let days = (current.end_date - current.start_date).num_days() as f64 + 1.0;

    if status == "Aprobado" && current.leave_type == "Vacaciones" {
        sqlx::query(
            "UPDATE employees SET vacation_days_available = vacation_days_available - $1 WHERE id = $2",
        )
        .bind(days)
        .bind(current.employee_id)
        .execute(&state.pool)
        .await?;
    }

    let result = sqlx::query_as::<_, schoolccb_common::hr::LeaveRequest>(
        r#"UPDATE leave_requests SET status = $1, approved_by = $2, approved_at = NOW(), updated_at = NOW() WHERE id = $3
           RETURNING id, employee_id, leave_type, start_date, end_date, reason, status, approved_by, approved_at, created_at, updated_at"#,
    ).bind(status).bind(approved_by).bind(id)
    .fetch_one(&state.pool).await?;

    Ok(Json(json!({ "leave_request": result, "message": format!("Solicitud {}", if status == "Aprobado" { "aprobada" } else { "rechazada" }) })))
}

async fn notify_leave_request(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    let request = sqlx::query(
        "SELECT lr.*, e.user_id FROM leave_requests lr JOIN employees e ON lr.employee_id = e.id WHERE lr.id = $1",
    ).bind(id).fetch_optional(&state.pool).await?
        .ok_or(SisError::NotFound("Solicitud no encontrada".into()))?;

    let user_id: Option<Uuid> = request.get("user_id");

    if let Some(uid) = user_id {
        let msg_id = Uuid::new_v4();
        let status: String = request.get("status");
        let leave_type: String = request.get("leave_type");
        sqlx::query(
            "INSERT INTO messages (id, sender_id, recipient_id, subject, body) VALUES ($1, $2, $3, $4, $5)",
        ).bind(msg_id)
         .bind(Uuid::parse_str(&claims.sub).map_err(|_| SisError::Unauthorized)?)
         .bind(uid)
         .bind(format!("Solicitud de {} {}", leave_type, if status == "Aprobado" { "Aprobada" } else { "Rechazada" }))
         .bind(format!("Tu solicitud de {} ha sido {}.", leave_type.to_lowercase(), if status == "Aprobado" { "aprobada" } else { "rechazada" }))
        .execute(&state.pool).await?;
    }

    Ok(Json(json!({ "message": "Notificacion enviada" })))
}

#[derive(Deserialize)]
struct AttendanceFilter {
    start_date: Option<String>,
    end_date: Option<String>,
}

async fn sync_attendance(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::hr::AttendanceLogPayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    let log_id = Uuid::new_v4();
    let result = sqlx::query_as::<_, schoolccb_common::hr::AttendanceLog>(
        r#"INSERT INTO employee_attendance_logs (id, employee_id, timestamp, entry_type, device_id, location_hash, source)
           VALUES ($1, $2, $3, $4, $5, $6, 'api')
           RETURNING id, employee_id, timestamp, entry_type, device_id, location_hash, source, created_at"#,
    ).bind(log_id).bind(payload.employee_id).bind(payload.timestamp)
     .bind(&payload.entry_type).bind(&payload.device_id).bind(&payload.location_hash)
    .fetch_one(&state.pool).await?;

    Ok(Json(json!({ "attendance": result, "message": "Marcacion registrada" })))
}

async fn list_attendance_logs(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(filter): Query<AttendanceFilter>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "hr",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;

    let mut sql = String::from(
        "SELECT id, employee_id, timestamp, entry_type, device_id, location_hash, source, created_at FROM employee_attendance_logs WHERE employee_id = $1",
    );
    let mut params: Vec<String> = vec![];

    if filter.start_date.is_some() {
        let n = params.len() + 2;
        params.push(format!("timestamp >= ${n}"));
    }
    if filter.end_date.is_some() {
        let n = params.len() + 2;
        params.push(format!("timestamp <= ${n}"));
    }

    for p in &params {
        sql.push_str(&format!(" AND {}", p));
    }
    sql.push_str(" ORDER BY timestamp DESC");

    let mut query = sqlx::query(&sql).bind(id);
    if let Some(ref sd) = filter.start_date {
        query = query.bind(sd);
    }
    if let Some(ref ed) = filter.end_date {
        query = query.bind(ed);
    }

    let logs = query
        .fetch_all(&state.pool)
        .await?;

    let result: Vec<Value> = logs
        .iter()
        .map(|r| {
            json!({
                "id": r.get::<Uuid, _>("id"),
                "employee_id": r.get::<Uuid, _>("employee_id"),
                "timestamp": r.get::<chrono::NaiveDateTime, _>("timestamp"),
                "entry_type": r.get::<String, _>("entry_type"),
                "device_id": r.get::<Option<String>, _>("device_id"),
                "source": r.get::<String, _>("source"),
            })
        })
        .collect();

    Ok(Json(json!({ "attendance_logs": result, "total": result.len() })))
}

async fn attendance_summary(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    let rows = sqlx::query(
        r#"SELECT DATE(timestamp) as day,
                  MIN(timestamp) as first_entry,
                  MAX(timestamp) as last_exit,
                  COUNT(*) as total_marcas
           FROM employee_attendance_logs
           WHERE employee_id = $1
             AND timestamp >= DATE_TRUNC('month', NOW())
           GROUP BY DATE(timestamp)
           ORDER BY day DESC"#,
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    let mut summaries: Vec<Value> = vec![];
    let mut total_hours_month: f64 = 0.0;
    let mut total_days: i64 = 0;

    for row in &rows {
        let first = row.get::<Option<chrono::NaiveDateTime>, _>("first_entry");
        let last = row.get::<Option<chrono::NaiveDateTime>, _>("last_exit");
        let day_hours = match (first, last) {
            (Some(f), Some(l)) => (l - f).num_minutes() as f64 / 60.0,
            _ => 0.0,
        };
        if day_hours > 0.0 {
            total_hours_month += day_hours;
            total_days += 1;
        }

        summaries.push(json!({
            "date": row.get::<chrono::NaiveDate, _>("day"),
            "first_entry": first,
            "last_exit": last,
            "total_hours": (day_hours * 100.0).round() / 100.0,
            "total_marcas": row.get::<i64, _>("total_marcas"),
        }));
    }

    Ok(Json(json!({
        "daily_summaries": summaries,
        "monthly_total_hours": (total_hours_month * 100.0).round() / 100.0,
        "total_days": total_days,
    })))
}

async fn modify_attendance(
    claims: Claims,
    State(state): State<AppState>,
    Path(att_id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::hr::AttendanceModificationPayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    let original = sqlx::query(
        "SELECT timestamp, entry_type FROM employee_attendance_logs WHERE id = $1",
    )
    .bind(att_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(SisError::NotFound("Marcacion no encontrada".into()))?;

    let orig_ts: chrono::NaiveDateTime = original.get("timestamp");
    let orig_type: String = original.get("entry_type");
    let original_value = format!("{}|{}", orig_ts, orig_type);
    let new_value = format!("{}|{}", payload.new_timestamp, payload.new_entry_type);

    sqlx::query(
        r#"UPDATE employee_attendance_logs SET timestamp = $1, entry_type = $2 WHERE id = $3"#,
    )
    .bind(payload.new_timestamp)
    .bind(&payload.new_entry_type)
    .bind(att_id)
    .execute(&state.pool)
    .await?;

    let mod_id = Uuid::new_v4();
    let mod_user = Uuid::parse_str(&claims.sub).map_err(|_| SisError::Unauthorized)?;
    sqlx::query(
        r#"INSERT INTO employee_attendance_modifications (id, attendance_id, original_value, new_value, reason, modified_by)
           VALUES ($1, $2, $3, $4, $5, $6)"#,
    )
    .bind(mod_id)
    .bind(att_id)
    .bind(&original_value)
    .bind(&new_value)
    .bind(&payload.reason)
    .bind(mod_user)
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({ "message": "Marcacion modificada", "modification_id": mod_id })))
}

async fn set_pension_fund(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    let pension_fund = payload.get("pension_fund").and_then(|v| v.as_str()).unwrap_or("Provida");
    let health_system = payload.get("health_system").and_then(|v| v.as_str()).unwrap_or("Fonasa");
    let health_fixed = payload.get("health_fixed_amount").and_then(|v| v.as_f64());

    sqlx::query(
        r#"INSERT INTO employee_pension_funds (id, employee_id, pension_fund, health_system, health_plan_name, health_fixed_amount)
           VALUES ($1, $2, $3, $4, $5, $6)
           ON CONFLICT (employee_id) DO UPDATE SET pension_fund = $3, health_system = $4, health_plan_name = $5, health_fixed_amount = $6"#,
    ).bind(Uuid::new_v4()).bind(id)
     .bind(pension_fund).bind(health_system)
     .bind(payload.get("health_plan_name").and_then(|v| v.as_str()))
     .bind(health_fixed)
    .execute(&state.pool).await?;

    Ok(Json(json!({ "message": "Configuracion de AFP/Salud guardada" })))
}

async fn get_pension_fund(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    let fund = sqlx::query_as::<_, schoolccb_common::hr::EmployeePensionFund>(
        "SELECT id, employee_id, pension_fund, health_system, health_plan_name, health_fixed_amount, created_at FROM employee_pension_funds WHERE employee_id = $1",
    ).bind(id).fetch_optional(&state.pool).await?;

    Ok(Json(json!({ "pension_fund": fund })))
}

async fn link_employee_user(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    let user_id = payload.get("user_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(SisError::Validation("user_id requerido".into()))?;

    sqlx::query("UPDATE employees SET user_id = $1 WHERE id = $2")
        .bind(user_id)
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({ "message": "Empleado vinculado a usuario" })))
}

async fn my_profile(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| SisError::Unauthorized)?;

    let employee = sqlx::query_as::<_, schoolccb_common::hr::Employee>(
        "SELECT id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id, created_at, updated_at FROM employees WHERE user_id = $1",
    ).bind(user_id).fetch_optional(&state.pool).await?
        .ok_or(SisError::NotFound("Perfil laboral no encontrado. Consulte a RRHH.".into()))?;

    let contract = sqlx::query_as::<_, schoolccb_common::hr::EmployeeContract>(
        "SELECT id, employee_id, contract_type, salary_base, weekly_hours, ley_karin_signed, start_date, end_date, active, created_at FROM employee_contracts WHERE employee_id = $1 AND active = true",
    ).bind(employee.id).fetch_optional(&state.pool).await?;

    let supervisor = match employee.supervisor_id {
        Some(sup_id) => {
            sqlx::query_as::<_, schoolccb_common::hr::Employee>(
                "SELECT id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id, created_at, updated_at FROM employees WHERE id = $1",
            ).bind(sup_id).fetch_optional(&state.pool).await?
        }
        None => None,
    };

    Ok(Json(json!({
        "employee": employee,
        "contract": contract,
        "supervisor": supervisor,
    })))
}

async fn my_payroll(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| SisError::Unauthorized)?;

    let emp = sqlx::query(
        "SELECT id FROM employees WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(SisError::NotFound("Perfil laboral no encontrado".into()))?;

    let emp_id: Uuid = emp.get("id");

    let rows = sqlx::query(
        r#"SELECT id, employee_id, month, year, salary_base, gratificacion, non_taxable_earnings, taxable_income, afp_discount, health_discount, unemployment_discount, income_tax, other_deductions, net_salary, lre_exported, previred_exported, created_at, updated_at
           FROM payrolls WHERE employee_id = $1
           ORDER BY year DESC, month DESC LIMIT 12"#,
    ).bind(emp_id).fetch_all(&state.pool).await?;

    let payrolls: Vec<Value> = rows.iter().map(|r| {
        json!({
            "id": r.get::<Uuid, _>("id"),
            "month": r.get::<i32, _>("month"),
            "year": r.get::<i32, _>("year"),
            "salary_base": r.get::<f64, _>("salary_base"),
            "gratificacion": r.get::<f64, _>("gratificacion"),
            "taxable_income": r.get::<f64, _>("taxable_income"),
            "afp_discount": r.get::<f64, _>("afp_discount"),
            "health_discount": r.get::<f64, _>("health_discount"),
            "net_salary": r.get::<f64, _>("net_salary"),
            "created_at": r.get::<chrono::DateTime<Utc>, _>("created_at"),
        })
    }).collect();

    Ok(Json(json!({ "payrolls": payrolls })))
}

async fn my_attendance(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| SisError::Unauthorized)?;

    let emp = sqlx::query("SELECT id FROM employees WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await?;

    let emp_id = match emp {
        Some(e) => e.get::<Uuid, _>("id"),
        None => {
            return Ok(Json(json!({ "attendance_logs": [], "total": 0 })));
        }
    };

    let logs = sqlx::query(
        r#"SELECT id, employee_id, timestamp, entry_type, device_id, source, created_at
           FROM employee_attendance_logs
           WHERE employee_id = $1
             AND timestamp >= DATE_TRUNC('month', NOW())
           ORDER BY timestamp DESC
           LIMIT 100"#,
    )
    .bind(emp_id)
    .fetch_all(&state.pool)
    .await?;

    let result: Vec<Value> = logs
        .iter()
        .map(|r| {
            json!({
                "id": r.get::<Uuid, _>("id"),
                "timestamp": r.get::<chrono::NaiveDateTime, _>("timestamp"),
                "entry_type": r.get::<String, _>("entry_type"),
                "device_id": r.get::<Option<String>, _>("device_id"),
            })
        })
        .collect();

    Ok(Json(json!({ "attendance_logs": result, "total": result.len() })))
}

async fn my_leave_requests(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| SisError::Unauthorized)?;

    let emp = sqlx::query("SELECT id FROM employees WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(SisError::NotFound("Perfil laboral no encontrado".into()))?;

    let emp_id: Uuid = emp.get("id");

    let requests = sqlx::query_as::<_, schoolccb_common::hr::LeaveRequest>(
        "SELECT id, employee_id, leave_type, start_date, end_date, reason, status, approved_by, approved_at, created_at, updated_at FROM leave_requests WHERE employee_id = $1 ORDER BY created_at DESC",
    ).bind(emp_id).fetch_all(&state.pool).await?;

    Ok(Json(json!({ "leave_requests": requests })))
}

async fn my_create_leave_request(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> SisResult<Json<Value>> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| SisError::Unauthorized)?;

    let emp = sqlx::query("SELECT id, supervisor_id FROM employees WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(SisError::NotFound("Perfil laboral no encontrado".into()))?;

    let emp_id: Uuid = emp.get("id");
    let supervisor_id: Option<Uuid> = emp.get("supervisor_id");

    let leave_type = payload.get("leave_type").and_then(|v| v.as_str()).unwrap_or("Vacaciones");
    let start_date = payload.get("start_date").and_then(|v| v.as_str()).unwrap_or("");
    let end_date = payload.get("end_date").and_then(|v| v.as_str()).unwrap_or("");
    let reason = payload.get("reason").and_then(|v| v.as_str());

    let sd = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
        .map_err(|_| SisError::Validation("start_date invalido, use YYYY-MM-DD".into()))?;
    let ed = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
        .map_err(|_| SisError::Validation("end_date invalido, use YYYY-MM-DD".into()))?;

    let leave_id = Uuid::new_v4();
    let result = sqlx::query_as::<_, schoolccb_common::hr::LeaveRequest>(
        r#"INSERT INTO leave_requests (id, employee_id, leave_type, start_date, end_date, reason)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, employee_id, leave_type, start_date, end_date, reason, status, approved_by, approved_at, created_at, updated_at"#,
    ).bind(leave_id).bind(emp_id)
     .bind(leave_type).bind(sd).bind(ed).bind(reason)
    .fetch_one(&state.pool).await?;

    if let Some(sup_id) = supervisor_id {
        if let Some(sup_user_id) = sqlx::query_scalar::<_, Option<Uuid>>(
            "SELECT user_id FROM employees WHERE id = $1",
        ).bind(sup_id).fetch_optional(&state.pool).await?.flatten() {
            let msg_id = Uuid::new_v4();
            let emp_name: String = sqlx::query_scalar(
                "SELECT first_name || ' ' || last_name FROM employees WHERE id = $1",
            ).bind(emp_id).fetch_one(&state.pool).await?;

            sqlx::query(
                "INSERT INTO messages (id, sender_id, recipient_id, subject, body) VALUES ($1, $2, $3, $4, $5)",
            ).bind(msg_id).bind(Uuid::parse_str(&claims.sub).map_err(|_| SisError::Unauthorized)?).bind(sup_user_id)
             .bind(format!("Nueva solicitud de {} de {}", leave_type, emp_name))
             .bind(format!("{} ha solicitado {} del {} al {}. Motivo: {}",
                 emp_name, leave_type.to_lowercase(), start_date, end_date, reason.unwrap_or("Sin motivo")))
            .execute(&state.pool).await?;
        }
    }

    Ok(Json(json!({ "leave_request": result, "message": "Solicitud creada. Se notificara a tu supervisor." })))
}

async fn my_documents(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| SisError::Unauthorized)?;

    let emp = sqlx::query("SELECT id FROM employees WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(SisError::NotFound("Perfil laboral no encontrado".into()))?;

    let emp_id: Uuid = emp.get("id");

    let docs = sqlx::query_as::<_, schoolccb_common::hr::EmployeeDocument>(
        "SELECT id, employee_id, doc_type, file_name, file_url, created_at FROM employee_documents WHERE employee_id = $1 ORDER BY created_at DESC",
    ).bind(emp_id).fetch_all(&state.pool).await?;

    Ok(Json(json!({ "documents": docs })))
}

async fn my_upload_document(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> SisResult<Json<Value>> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| SisError::Unauthorized)?;

    let emp = sqlx::query("SELECT id FROM employees WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(SisError::NotFound("Perfil laboral no encontrado".into()))?;

    let emp_id: Uuid = emp.get("id");

    let doc_type = payload.get("doc_type").and_then(|v| v.as_str()).unwrap_or("certificado");
    let file_name = payload.get("file_name").and_then(|v| v.as_str()).unwrap_or("documento");

    let doc_id = Uuid::new_v4();
    let result = sqlx::query_as::<_, schoolccb_common::hr::EmployeeDocument>(
        r#"INSERT INTO employee_documents (id, employee_id, doc_type, file_name)
           VALUES ($1, $2, $3, $4)
           RETURNING id, employee_id, doc_type, file_name, file_url, created_at"#,
    ).bind(doc_id).bind(emp_id).bind(doc_type).bind(file_name)
    .fetch_one(&state.pool).await?;

    Ok(Json(json!({ "document": result })))
}

async fn list_complaints(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "hr",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;

    let complaints = sqlx::query_as::<_, schoolccb_common::hr::Complaint>(
        "SELECT id, complainant_name, complainant_email, accused_rut, complaint_type, description, status, resolution, created_at, updated_at FROM complaints ORDER BY created_at DESC",
    ).fetch_all(&state.pool).await?;

    Ok(Json(json!({ "complaints": complaints })))
}

async fn submit_complaint(
    _claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::hr::CreateComplaintPayload>,
) -> SisResult<Json<Value>> {
    let complaint_id = Uuid::new_v4();
    let result = sqlx::query_as::<_, schoolccb_common::hr::Complaint>(
        r#"INSERT INTO complaints (id, complainant_name, complainant_email, accused_rut, complaint_type, description)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, complainant_name, complainant_email, accused_rut, complaint_type, description, status, resolution, created_at, updated_at"#,
    ).bind(complaint_id)
     .bind(&payload.complainant_name)
     .bind(&payload.complainant_email)
     .bind(&payload.accused_rut)
     .bind(&payload.complaint_type)
     .bind(&payload.description)
    .fetch_one(&state.pool).await?;

    Ok(Json(json!({ "complaint": result, "message": "Denuncia recibida. Será revisada por RRHH." })))
}

#[derive(Deserialize)]
struct GenerateContractPayload {
    employee_id: Uuid,
    contract_type: String,
    salary_base: f64,
    weekly_hours: i32,
    start_date: NaiveDate,
    ley_karin_signed: bool,
}

async fn generate_contract(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<GenerateContractPayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    let employee = sqlx::query_as::<_, schoolccb_common::hr::Employee>(
        "SELECT id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id, created_at, updated_at FROM employees WHERE id = $1",
    ).bind(payload.employee_id).fetch_optional(&state.pool).await?
        .ok_or(SisError::NotFound("Empleado no encontrado".into()))?;

    let school_name = sqlx::query_scalar::<_, String>(
        "SELECT name FROM schools WHERE id = (SELECT school_id FROM employees WHERE id = $1)",
    ).bind(payload.employee_id).fetch_optional(&state.pool).await?
        .unwrap_or_else(|| "El Establecimiento".to_string());

    let position = employee.position.as_deref().unwrap_or("El cargo");
    let full_name = format!("{} {}", employee.first_name, employee.last_name);
    let rut = &employee.rut;
    let salary_str = format!("${:.0}", payload.salary_base);
    let contract_type_str = match payload.contract_type.as_str() {
        "Indefinido" => "plazo indefinido",
        "PlazoFijo" => "plazo fijo",
        "Honorarios" => "honorarios",
        "PrestacionServicios" => "prestación de servicios",
        _ => &payload.contract_type,
    };
    let karin_clause = if payload.ley_karin_signed {
        "El empleador y el trabajador declaran haber tomado conocimiento de la Ley N\u{00b0} 21.643 (Ley Karin) sobre prevenci\u{00f3}n, investigaci\u{00f3}n y sanci\u{00f3}n del acoso laboral, sexual y violencia en el trabajo, y se comprometen a dar estricto cumplimiento a sus disposiciones durante toda la vigencia de la relaci\u{00f3}n laboral."
    } else {
        "Pendiente de firma de declaraci\u{00f3}n Ley Karin (Ley 21.643)."
    };

    let duration_clause = if payload.contract_type == "PlazoFijo" {
        format!("El contrato se extiende hasta el [fecha de t\u{00e9}rmino].")
    } else {
        "El contrato es de plazo indefinido.".to_string()
    };

    let start_fmt = payload.start_date.format("%d/%m/%Y").to_string();

    let contract_text = format!(
        r#"CONTRATO DE TRABAJO

En {school}, a {start}, comparecen:

Por una parte, el empleador {school}, representado legalmente por quien corresponda, y

Por la otra, don(ña) {name}, RUN {rut}, domiciliado en [Dirección del trabajador],

Los comparecientes, mayores de edad, acuerdan celebrar el siguiente contrato de trabajo:

PRIMERO: NATURALEZA DEL CONTRATO.- El empleador contrata y el trabajador se obliga a prestar servicios bajo subordinación y dependencia, en calidad de {ctype}.

SEGUNDO: CARGO Y FUNCIONES.- El trabajador desempeñará el cargo de {pos}, debiendo cumplir las funciones propias del cargo y las que el empleador le encomiende dentro del giro del establecimiento educacional.

TERCERO: REMUNERACIÓN.- El empleador pagará al trabajador una remuneración mensual de {salary} (pesos chilenos), la que será pagada por mensualidades vencidas.

CUARTO: JORNADA DE TRABAJO.- La jornada ordinaria de trabajo será de {hours} horas semanales, distribuidas de lunes a viernes, en los horarios que determine el empleador.

QUINTO: DURACIÓN DEL CONTRATO.- El presente contrato rige desde el {start}. {duration}

SEXTO: LEY KARIN (Ley 21.643).- {karin}

SÉPTIMO: DOMICILIO.- Las partes fijan su domicilio en la ciudad de [Ciudad], comuna de [Comuna], y se someten a la jurisdicción de sus tribunales ordinarios de justicia.

FIRMAS:

_____________________________
{name}
RUN: {rut}

_____________________________
El Empleador
{school}

LUGAR Y FECHA: {school}, {start}
"#,
        school = school_name,
        start = start_fmt,
        name = full_name,
        rut = rut,
        ctype = contract_type_str,
        pos = position,
        salary = salary_str,
        hours = payload.weekly_hours,
        duration = duration_clause,
        karin = karin_clause,
    );

    Ok(Json(json!({ "contract_text": contract_text })))
}
