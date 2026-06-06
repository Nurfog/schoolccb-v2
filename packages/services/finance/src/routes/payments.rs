use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::FinanceResult;
use crate::routes::fees::{Claims, require_any_role};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/finance/payments",
            get(list_payments).post(create_payment),
        )
        .route("/api/finance/payments/{id}", get(get_payment))
        .route(
            "/api/finance/payments/student/{student_id}",
            get(payments_by_student),
        )
}

async fn list_payments(
    claims: Claims,
    State(state): State<AppState>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let payments = sqlx::query_as::<_, schoolccb_common::finance::Payment>(
        "SELECT id, fee_id, student_id, amount, payment_date, payment_method, reference, created_at FROM payments ORDER BY payment_date DESC LIMIT 100",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(
        json!({ "payments": payments, "total": payments.len() }),
    ))
}

async fn get_payment(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let payment = sqlx::query_as::<_, schoolccb_common::finance::Payment>(
        "SELECT id, fee_id, student_id, amount, payment_date, payment_method, reference, created_at FROM payments WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(crate::error::FinanceError::NotFound("Pago no encontrado".into()))?;

    Ok(Json(json!({ "payment": payment })))
}

async fn create_payment(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::finance::CreatePaymentPayload>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let id = Uuid::new_v4();
    let payment_date = payload
        .payment_date
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    schoolccb_common::audit::log(&state.pool, &schoolccb_common::audit::AuditEntry {
        entity_type: "payment".into(),
        entity_id: id,
        action: "create".into(),
        user_id: Some(uuid::Uuid::parse_str(&claims.sub).unwrap_or_default()),
        changes: Some(serde_json::json!({"fee_id": payload.fee_id, "student_id": payload.student_id, "amount": payload.amount, "method": payload.payment_method})),
    }).await;

    let result = sqlx::query_as::<_, schoolccb_common::finance::Payment>(
        r#"
        INSERT INTO payments (id, fee_id, student_id, amount, payment_date, payment_method, reference)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, fee_id, student_id, amount, payment_date, payment_method, reference, created_at
        "#,
    )
    .bind(id)
    .bind(payload.fee_id)
    .bind(payload.student_id)
    .bind(payload.amount)
    .bind(payment_date)
    .bind(&payload.payment_method)
    .bind(&payload.reference)
    .fetch_one(&state.pool)
    .await?;

    sqlx::query("UPDATE fees SET paid = true, paid_date = $1, paid_amount = $2 WHERE id = $3")
        .bind(payment_date)
        .bind(payload.amount)
        .bind(payload.fee_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({ "payment": result })))
}

async fn payments_by_student(
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

    let payments = sqlx::query_as::<_, schoolccb_common::finance::Payment>(
        "SELECT id, fee_id, student_id, amount, payment_date, payment_method, reference, created_at FROM payments WHERE student_id = $1 ORDER BY payment_date DESC",
    )
    .bind(student_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "payments": payments })))
}
