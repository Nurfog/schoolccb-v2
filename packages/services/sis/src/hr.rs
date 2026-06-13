use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{SisError, SisResult};
use schoolccb_common::auth::{Claims, require_any_role};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/hr/employees",
            get(list_employees).post(create_employee),
        )
        .route(
            "/api/hr/employees/{id}",
            get(get_employee)
                .put(update_employee)
                .delete(deactivate_employee),
        )
        .route(
            "/api/hr/employees/{id}/contracts",
            get(list_contracts).post(create_contract),
        )
}

#[derive(Deserialize)]
struct EmployeeFilter {
    search: Option<String>,
}

async fn list_employees(
    claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<EmployeeFilter>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "hr",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;

    let mut sql = "SELECT id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id, created_at, updated_at FROM employees".to_string();
    let mut clauses: Vec<String> = vec![];

    if let Some(ref _search) = q.search {
        let n = clauses.len() + 1;
        clauses.push(format!(
            "(rut ILIKE ${n} OR first_name ILIKE ${n} OR last_name ILIKE ${n})"
        ));
    }
    if !clauses.is_empty() {
        sql.push_str(&format!(" WHERE {}", clauses.join(" AND ")));
    }
    sql.push_str(" ORDER BY created_at DESC LIMIT 100");

    let mut query = sqlx::query_as::<_, schoolccb_common::hr::Employee>(&sql);
    if let Some(ref s) = q.search {
        let pat = format!("%{}%", s);
        query = query.bind(pat);
    }

    let employees = query.fetch_all(&state.pool).await?;
    Ok(Json(
        json!({ "employees": employees, "total": employees.len() }),
    ))
}

async fn get_employee(
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

    let employee = sqlx::query_as::<_, schoolccb_common::hr::Employee>(
        "SELECT id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id, created_at, updated_at FROM employees WHERE id = $1",
    ).bind(id).fetch_optional(&state.pool).await?
        .ok_or(SisError::NotFound("Funcionario no encontrado".into()))?;

    let contracts = sqlx::query_as::<_, schoolccb_common::hr::EmployeeContract>(
        "SELECT id, employee_id, contract_type, salary_base, weekly_hours, ley_karin_signed, start_date, end_date, active, created_at FROM employee_contracts WHERE employee_id = $1 ORDER BY created_at DESC",
    ).bind(id).fetch_all(&state.pool).await?;

    let documents = sqlx::query_as::<_, schoolccb_common::hr::EmployeeDocument>(
        "SELECT id, employee_id, doc_type, file_name, file_url, created_at FROM employee_documents WHERE employee_id = $1 ORDER BY created_at DESC",
    ).bind(id).fetch_all(&state.pool).await?;

    Ok(Json(
        json!({ "employee": employee, "contracts": contracts, "documents": documents }),
    ))
}

async fn create_employee(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::hr::CreateEmployeePayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    if payload.rut.trim().is_empty()
        || payload.first_name.trim().is_empty()
        || payload.last_name.trim().is_empty()
    {
        return Err(SisError::Validation(
            "RUT, nombre y apellido son obligatorios".into(),
        ));
    }

    let id = Uuid::new_v4();
    let result = sqlx::query_as::<_, schoolccb_common::hr::Employee>(
        r#"INSERT INTO employees (id, rut, first_name, last_name, email, phone, position, category, hire_date)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
           RETURNING id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id, created_at, updated_at"#,
    ).bind(id).bind(&payload.rut).bind(&payload.first_name).bind(&payload.last_name)
    .bind(&payload.email).bind(&payload.phone).bind(&payload.position).bind(&payload.category).bind(payload.hire_date)
    .fetch_one(&state.pool).await?;

    Ok(Json(json!({ "employee": result })))
}

async fn update_employee(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::hr::UpdateEmployeePayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    let current = sqlx::query_as::<_, schoolccb_common::hr::Employee>(
        "SELECT id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id, created_at, updated_at FROM employees WHERE id = $1",
    ).bind(id).fetch_optional(&state.pool).await?
        .ok_or(SisError::NotFound("Funcionario no encontrado".into()))?;

    let result = sqlx::query_as::<_, schoolccb_common::hr::Employee>(
        r#"UPDATE employees SET first_name = $1, last_name = $2, email = $3, phone = $4, position = $5, category = $6, hire_date = $7, vacation_days_available = $8, updated_at = NOW() WHERE id = $9
           RETURNING id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id, created_at, updated_at"#,
    ).bind(payload.first_name.unwrap_or(current.first_name))
    .bind(payload.last_name.unwrap_or(current.last_name))
    .bind(payload.email.or(current.email))
    .bind(payload.phone.or(current.phone))
    .bind(payload.position.or(current.position))
    .bind(payload.category.or(current.category))
    .bind(payload.hire_date.or(current.hire_date))
    .bind(payload.vacation_days_available.unwrap_or(current.vacation_days_available))
    .bind(id)
    .fetch_one(&state.pool).await?;

    Ok(Json(json!({ "employee": result })))
}

async fn deactivate_employee(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    let result = sqlx::query_as::<_, schoolccb_common::hr::Employee>(
        r#"UPDATE employees SET active = false, updated_at = NOW() WHERE id = $1
           RETURNING id, school_id, rut, first_name, last_name, email, phone, position, category, hire_date, vacation_days_available, active, supervisor_id, user_id, created_at, updated_at"#,
    ).bind(id).fetch_optional(&state.pool).await?
        .ok_or(SisError::NotFound("Funcionario no encontrado".into()))?;

    Ok(Json(
        json!({ "employee": result, "message": "Funcionario desactivado" }),
    ))
}

async fn list_contracts(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    let contracts = sqlx::query_as::<_, schoolccb_common::hr::EmployeeContract>(
        "SELECT id, employee_id, contract_type, salary_base, weekly_hours, ley_karin_signed, start_date, end_date, active, created_at FROM employee_contracts WHERE employee_id = $1 ORDER BY created_at DESC",
    ).bind(id).fetch_all(&state.pool).await?;

    Ok(Json(json!({ "contracts": contracts })))
}

async fn create_contract(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::hr::CreateContractPayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    if payload.weekly_hours > 40 {
        return Err(SisError::Validation(
            "Las horas semanales no pueden exceder 40 (Ley 40 Horas)".into(),
        ));
    }

    if payload.salary_base <= 0.0 {
        return Err(SisError::Validation(
            "El salario base debe ser mayor a cero".into(),
        ));
    }

    sqlx::query(
        "UPDATE employee_contracts SET active = false WHERE employee_id = $1 AND active = true",
    )
    .bind(id)
    .execute(&state.pool)
    .await?;

    let contract_id = Uuid::new_v4();
    let result = sqlx::query_as::<_, schoolccb_common::hr::EmployeeContract>(
        r#"INSERT INTO employee_contracts (id, employee_id, contract_type, salary_base, weekly_hours, ley_karin_signed, start_date, end_date)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
           RETURNING id, employee_id, contract_type, salary_base, weekly_hours, ley_karin_signed, start_date, end_date, active, created_at"#,
    ).bind(contract_id).bind(id)
    .bind(&payload.contract_type).bind(payload.salary_base)
    .bind(payload.weekly_hours).bind(payload.ley_karin_signed)
    .bind(payload.start_date).bind(payload.end_date)
    .fetch_one(&state.pool).await?;

    Ok(Json(json!({ "contract": result })))
}
