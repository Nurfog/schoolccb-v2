use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{FinanceError, FinanceResult};

pub use schoolccb_common::auth::Claims;
use schoolccb_common::auth::require_any_role;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/finance/fees", get(list_fees).post(create_fee))
        .route(
            "/api/finance/fees/{id}",
            get(get_fee).put(update_fee).delete(delete_fee),
        )
        .route(
            "/api/finance/fees/student/{student_id}",
            get(fees_by_student),
        )
}

async fn list_fees(claims: Claims, State(state): State<AppState>) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "finance",
    )
    .await
    .map_err(|e| FinanceError::Forbidden(e))?;

    let school_id: Option<Uuid> = claims
        .school_id
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok());

    let corporation_id: Option<Uuid> = claims
        .corporation_id
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok());

    let (sql, corp_val) = if let Some(_sc) = school_id {
        let corp_clause = corporation_id.map(|_| " AND sch.corporation_id = $2").unwrap_or("");
        (format!("SELECT f.id, f.student_id, f.description, f.amount, f.due_date, f.paid, f.paid_date, f.paid_amount, f.created_at FROM fees f JOIN schools sch ON sch.id = f.school_id WHERE f.school_id = $1{} ORDER BY f.due_date DESC LIMIT 100", corp_clause), corporation_id)
    } else {
        ("SELECT id, student_id, description, amount, due_date, paid, paid_date, paid_amount, created_at FROM fees ORDER BY due_date DESC LIMIT 100".to_string(), None)
    };
    let mut q = sqlx::query_as::<_, schoolccb_common::finance::Fee>(&sql);
    if let Some(sc) = school_id { q = q.bind(sc); }
    if let Some(cc) = corp_val { q = q.bind(cc); }
    let fees = q.fetch_all(&state.pool).await?;

    Ok(Json(json!({ "fees": fees, "total": fees.len() })))
}

async fn get_fee(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "finance",
    )
    .await
    .map_err(|e| FinanceError::Forbidden(e))?;

    let fee = sqlx::query_as::<_, schoolccb_common::finance::Fee>(
        "SELECT id, student_id, description, amount, due_date, paid, paid_date, paid_amount, created_at FROM fees WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(FinanceError::NotFound("Cuota no encontrada".into()))?;

    Ok(Json(json!({ "fee": fee })))
}

async fn create_fee(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::finance::CreateFeePayload>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    if payload.description.trim().is_empty() || payload.amount <= 0.0 {
        return Err(FinanceError::Validation(
            "Descripción y monto válido son obligatorios".into(),
        ));
    }

    let school_id = claims.school_id.and_then(|s| Uuid::parse_str(&s).ok());

    let id = Uuid::new_v4();
    let result = sqlx::query_as::<_, schoolccb_common::finance::Fee>(
        r#"
        INSERT INTO fees (id, student_id, description, amount, due_date, school_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, student_id, description, amount, due_date, paid, paid_date, paid_amount, created_at
        "#,
    )
    .bind(id)
    .bind(payload.student_id)
    .bind(&payload.description)
    .bind(payload.amount)
    .bind(payload.due_date)
    .bind(school_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "fee": result })))
}

async fn update_fee(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let paid = payload.get("paid").and_then(|v| v.as_bool());
    let paid_amount = payload.get("paid_amount").and_then(|v| v.as_f64());

    if let Some(true) = paid {
        let paid_date = chrono::Utc::now().date_naive();
        sqlx::query(
            "UPDATE fees SET paid = true, paid_date = $1, paid_amount = COALESCE($2, amount) WHERE id = $3",
        )
        .bind(paid_date)
        .bind(paid_amount)
        .bind(id)
        .execute(&state.pool)
        .await?;
    }

    let fee = sqlx::query_as::<_, schoolccb_common::finance::Fee>(
        "SELECT id, student_id, description, amount, due_date, paid, paid_date, paid_amount, created_at FROM fees WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(FinanceError::NotFound("Cuota no encontrada".into()))?;

    Ok(Json(json!({ "fee": fee })))
}

async fn delete_fee(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    sqlx::query("DELETE FROM fees WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({ "message": "Cuota eliminada correctamente" })))
}

async fn fees_by_student(
    claims: Claims,
    State(state): State<AppState>,
    Path(student_id): Path<Uuid>,
) -> FinanceResult<Json<Value>> {
    require_any_role(
        &claims,
        &[
            "Administrador",
            "Sostenedor",
            "Director",
            "UTP",
            "Apoderado",
        ],
    )?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "finance",
    )
    .await
    .map_err(|e| FinanceError::Forbidden(e))?;

    let fees = sqlx::query_as::<_, schoolccb_common::finance::Fee>(
        "SELECT id, student_id, description, amount, due_date, paid, paid_date, paid_amount, created_at FROM fees WHERE student_id = $1 ORDER BY due_date",
    )
    .bind(student_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "fees": fees })))
}
