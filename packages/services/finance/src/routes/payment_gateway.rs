use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::FinanceResult;
use crate::payment_gateway::PaymentInitRequest;
use schoolccb_common::auth::{Claims, require_any_role};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/finance/payment/init/{fee_id}", get(init_payment))
        .route("/api/finance/payment/return", get(payment_return))
}

#[derive(Deserialize)]
struct ReturnParams {
    token_ws: Option<String>,
    mock: Option<bool>,
}

async fn init_payment(
    claims: Claims,
    State(state): State<AppState>,
    axum::extract::Path(fee_id): axum::extract::Path<Uuid>,
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

    let gateway = state.gateway.ok_or_else(|| {
        crate::error::FinanceError::Internal("Pasarela de pago no configurada".into())
    })?;

    let fee = sqlx::query_as::<_, schoolccb_common::finance::Fee>(
        "SELECT id, student_id, description, amount, due_date, paid, paid_amount, paid_date, created_at FROM fees WHERE id = $1",
    )
    .bind(fee_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(crate::error::FinanceError::NotFound("Cuota no encontrada".into()))?;

    if fee.paid {
        return Err(crate::error::FinanceError::Conflict(
            "La cuota ya está pagada".into(),
        ));
    }

    let req = PaymentInitRequest {
        amount: fee.amount,
        description: fee.description.clone(),
        student_id: fee.student_id.to_string(),
        fee_id: fee.id.to_string(),
        payer_email: None,
    };

    let resp = tokio::task::spawn_blocking(move || {
        gateway.init_transaction(&req)
    })
    .await
    .map_err(|e| crate::error::FinanceError::Internal(e.to_string()))?
    .map_err(crate::error::FinanceError::Internal)?;

    sqlx::query(
        "INSERT INTO payment_transactions (id, fee_id, token, amount, status, gateway_url) VALUES ($1, $2, $3, $4, 'INITIALIZED', $5)",
    )
    .bind(Uuid::new_v4())
    .bind(fee.id)
    .bind(&resp.token)
    .bind(fee.amount)
    .bind(&resp.url)
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({
        "url": resp.url,
        "token": resp.token,
        "gateway": gateway.provider_name(),
    })))
}

async fn payment_return(
    State(state): State<AppState>,
    Query(params): Query<ReturnParams>,
) -> FinanceResult<Json<Value>> {
    let token = params.token_ws.unwrap_or_default();

    if token.is_empty() {
        return Err(crate::error::FinanceError::Validation(
            "Token no proporcionado".into(),
        ));
    }

    let result = if params.mock.unwrap_or(false) {
        crate::payment_gateway::PaymentResult {
            success: true,
            amount: 0.0,
            transaction_id: format!("mock-{}", Uuid::new_v4()),
            authorization_code: "MOCK-000".into(),
            payment_type: "Mock".into(),
        }
    } else {
        let gateway = state.gateway.ok_or_else(|| {
            crate::error::FinanceError::Internal("Pasarela de pago no configurada".into())
        })?;
        let token_clone = token.clone();
        tokio::task::spawn_blocking(move || {
            gateway.confirm_transaction(&token_clone)
        })
        .await
        .map_err(|e| crate::error::FinanceError::Internal(e.to_string()))?
        .map_err(crate::error::FinanceError::Internal)?
    };

    if result.success {
        let tx: Option<(Uuid, f64)> = sqlx::query_as(
            "SELECT fee_id, amount FROM payment_transactions WHERE token = $1 AND status = 'INITIALIZED'",
        )
        .bind(&token)
        .fetch_optional(&state.pool)
        .await?;

        if let Some((fee_id, amount)) = tx {
            sqlx::query("UPDATE payment_transactions SET status = 'CONFIRMED', authorization_code = $1, payment_type = $2 WHERE token = $3")
                .bind(&result.authorization_code)
                .bind(&result.payment_type)
                .bind(&token)
                .execute(&state.pool)
                .await?;

            sqlx::query("UPDATE fees SET paid = true, paid_date = NOW(), paid_amount = $1 WHERE id = $2")
                .bind(amount)
                .bind(fee_id)
                .execute(&state.pool)
                .await?;

            let payment_id = Uuid::new_v4();
            sqlx::query(
                r#"INSERT INTO payments (id, fee_id, student_id, amount, payment_date, payment_method, reference)
                   VALUES ($1, $2, (SELECT student_id FROM fees WHERE id = $2), $3, NOW(), 'Webpay', $4)"#,
            )
            .bind(payment_id)
            .bind(fee_id)
            .bind(amount)
            .bind(&result.transaction_id)
            .execute(&state.pool)
            .await?;
        }

        Ok(Json(json!({
            "success": true,
            "message": "Pago confirmado exitosamente",
            "transaction_id": result.transaction_id,
            "authorization_code": result.authorization_code,
        })))
    } else {
        Ok(Json(json!({
            "success": false,
            "message": "La transacción no fue aprobada",
        })))
    }
}
