use axum::{
    Json, Router,
    extract::State,
    routing::get,
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{AuthError, AuthResult};
use crate::models::Claims;

pub fn client_router() -> Router<AppState> {
    Router::new()
        .route("/api/client/license", get(client_license))
        .route("/api/client/payments", get(client_payments))
        .route("/api/client/billing-info", get(client_billing_info).put(client_update_billing))
}

fn require_corporation_user(claims: &Claims) -> Result<(), AuthError> {
    if claims.corporation_id.is_some() {
        return Ok(());
    }
    Err(AuthError::Forbidden(
        "Se requiere una corporación asociada".into(),
    ))
}

fn corporation_id(claims: &Claims) -> Result<Uuid, AuthError> {
    claims
        .corporation_id
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AuthError::Forbidden(
            "No tienes una corporación asociada".into(),
        ))
}

async fn client_license(
    claims: Claims,
    State(state): State<AppState>,
) -> AuthResult<Json<Value>> {
    require_corporation_user(&claims)?;
    let corp_id = corporation_id(&claims)?;

    let license = sqlx::query_as::<_, (Uuid, String, Option<chrono::NaiveDate>, Option<chrono::NaiveDate>, String, bool)>(
        "SELECT cl.id, lp.name, cl.start_date, cl.end_date, cl.status, cl.auto_renew
         FROM corporation_licenses cl
         JOIN license_plans lp ON lp.id = cl.plan_id
         WHERE cl.corporation_id = $1 AND cl.status = 'active'
         LIMIT 1",
    )
    .bind(corp_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AuthError::NotFound("No tienes una licencia activa".into()))?;

    let days_remaining: i64 = license.2
        .map(|end| (end - chrono::Utc::now().date_naive()).num_days())
        .unwrap_or(0);

    let modules: Vec<Value> = sqlx::query_as::<_, (String, String, bool)>(
        "SELECT pm.module_key, pm.module_name, pm.included
         FROM plan_modules pm
         JOIN corporation_licenses cl ON cl.plan_id = pm.plan_id
         WHERE cl.id = $1
         ORDER BY pm.module_key",
    )
    .bind(license.0)
    .fetch_all(&state.pool)
    .await.unwrap_or_default()
    .into_iter()
    .map(|(k, n, inc)| json!({"key": k, "name": n, "included": inc}))
    .collect();

    Ok(Json(json!({
        "license": {
            "id": license.0,
            "plan_name": license.1,
            "start_date": license.2,
            "end_date": license.3,
            "status": license.4,
            "auto_renew": license.5,
            "days_remaining": days_remaining,
        },
        "modules": modules,
    })))
}

async fn client_payments(
    claims: Claims,
    State(state): State<AppState>,
) -> AuthResult<Json<Value>> {
    require_corporation_user(&claims)?;
    let corp_id = corporation_id(&claims)?;

    let payments: Vec<Value> = sqlx::query_as::<_, (Uuid, f64, String, String, Option<String>, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::NaiveDate>, Option<chrono::NaiveDate>)>(
        "SELECT lp.id, lp.amount, lp.currency, lp.payment_method, lp.transaction_id, lp.paid_at, lp.period_start, lp.period_end
         FROM license_payments lp
         JOIN corporation_licenses cl ON cl.id = lp.corporation_license_id
         WHERE cl.corporation_id = $1 AND lp.status = 'completed'
         ORDER BY lp.paid_at DESC",
    )
    .bind(corp_id)
    .fetch_all(&state.pool)
    .await.unwrap_or_default()
    .into_iter()
    .map(|(id, amount, currency, method, tx_id, paid, ps, pe)| {
        json!({"id": id, "amount": amount, "currency": currency, "payment_method": method, "transaction_id": tx_id, "paid_at": paid, "period_start": ps, "period_end": pe})
    })
    .collect();

    Ok(Json(json!({"payments": payments})))
}

async fn client_billing_info(
    claims: Claims,
    State(state): State<AppState>,
) -> AuthResult<Json<Value>> {
    require_corporation_user(&claims)?;
    let corp_id = corporation_id(&claims)?;

    let info = sqlx::query_as::<_, (Option<String>, Option<String>, Option<String>)>(
        "SELECT rut, name, logo_url FROM corporations WHERE id = $1",
    )
    .bind(corp_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AuthError::NotFound("Corporación no encontrada".into()))?;

    Ok(Json(json!({
        "rut": info.0,
        "business_name": info.1,
        "logo_url": info.2,
    })))
}

async fn client_update_billing(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> AuthResult<Json<Value>> {
    require_corporation_user(&claims)?;
    let corp_id = corporation_id(&claims)?;

    if let Some(name) = payload.get("business_name").and_then(|v| v.as_str()) {
        if !name.is_empty() {
            sqlx::query("UPDATE corporations SET name = $1 WHERE id = $2")
                .bind(name)
                .bind(corp_id)
                .execute(&state.pool)
                .await?;
        }
    }

    Ok(Json(json!({"message": "Datos de facturación actualizados"})))
}
