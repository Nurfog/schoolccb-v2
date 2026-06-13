use axum::{Json, Router, extract::State, routing::get};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{SisError, SisResult};
use schoolccb_common::auth::{Claims, require_any_role};

pub fn router() -> Router<AppState> {
    Router::new().route("/api/admission/metrics", get(admission_metrics))
}

async fn admission_metrics(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Admision"],
    )?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "admission",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;

    let total_prospects: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM prospects")
        .fetch_one(&state.pool)
        .await?;

    let prospects_by_stage: Vec<(Uuid, String, i64)> = sqlx::query_as(
        r#"SELECT ps.id, ps.name, COUNT(p.id) as cnt
           FROM pipeline_stages ps
           LEFT JOIN prospects p ON p.current_stage_id = ps.id
           GROUP BY ps.id, ps.name, ps.sort_order
           ORDER BY ps.sort_order"#,
    )
    .fetch_all(&state.pool)
    .await?;

    let total_finalized: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM prospects p
           JOIN pipeline_stages ps ON p.current_stage_id = ps.id
           WHERE ps.is_final = true"#,
    )
    .fetch_one(&state.pool)
    .await?;

    let conversion_rate = if total_prospects.0 > 0 {
        (total_finalized.0 as f64 / total_prospects.0 as f64) * 100.0
    } else {
        0.0
    };

    let prospects_by_source: Vec<(Option<String>, i64)> = sqlx::query_as(
        r#"SELECT source, COUNT(*) as cnt
           FROM prospects
           GROUP BY source
           ORDER BY cnt DESC"#,
    )
    .fetch_all(&state.pool)
    .await?;

    let recent_activity: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM prospect_activities WHERE created_at > NOW() - INTERVAL '7 days'",
    )
    .fetch_one(&state.pool)
    .await?;

    let stages_with_counts: Vec<Value> = prospects_by_stage
        .iter()
        .map(|(id, name, count)| {
            json!({
                "stage_id": id,
                "stage_name": name,
                "count": count,
            })
        })
        .collect();

    let sources: Vec<Value> = prospects_by_source
        .iter()
        .map(|(source, count)| {
            json!({
                "source": source.as_deref().unwrap_or("sin_origen"),
                "count": count,
            })
        })
        .collect();

    Ok(Json(json!({
        "total_prospects": total_prospects.0,
        "total_finalized": total_finalized.0,
        "conversion_rate": format!("{:.1}", conversion_rate),
        "stages": stages_with_counts,
        "sources": sources,
        "recent_activities_7d": recent_activity.0,
    })))
}
