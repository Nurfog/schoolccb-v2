use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;

use crate::error::{SisError, SisResult};
use schoolccb_common::auth::{Claims, require_any_role};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/search", get(global_search))
}

#[derive(Deserialize)]
struct SearchParams {
    q: String,
}

async fn global_search(claims: Claims, State(state): State<AppState>, Query(params): Query<SearchParams>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"])?;

    let query = params.q.trim();
    if query.len() < 2 {
        return Err(SisError::Validation("La búsqueda debe tener al menos 2 caracteres".into()));
    }

    let pattern = format!("%{}%", query);

    let students = sqlx::query(
        r#"SELECT id::text, rut, first_name, last_name, grade_level, section, 'student' as entity_type
           FROM students
           WHERE rut ILIKE $1 OR first_name ILIKE $1 OR last_name ILIKE $1
           LIMIT 5"#,
    ).bind(&pattern).fetch_all(&state.pool).await?;

    let employees = sqlx::query(
        r#"SELECT id::text, rut, first_name, last_name, position as grade_level, category as section, 'employee' as entity_type
           FROM employees
           WHERE (rut ILIKE $1 OR first_name ILIKE $1 OR last_name ILIKE $1) AND active = true
           LIMIT 5"#,
    ).bind(&pattern).fetch_all(&state.pool).await?;

    let mut results: Vec<Value> = vec![];

    for row in students {
        results.push(json!({
            "id": row.get::<String, _>("id"),
            "rut": row.get::<String, _>("rut"),
            "first_name": row.get::<String, _>("first_name"),
            "last_name": row.get::<String, _>("last_name"),
            "subtitle": format!("{} {}", row.get::<String, _>("grade_level"), row.get::<String, _>("section")),
            "entity_type": "student",
        }));
    }

    for row in employees {
        results.push(json!({
            "id": row.get::<String, _>("id"),
            "rut": row.get::<String, _>("rut"),
            "first_name": row.get::<String, _>("first_name"),
            "last_name": row.get::<String, _>("last_name"),
            "subtitle": row.get::<String, _>("grade_level"),
            "entity_type": "employee",
        }));
    }

    Ok(Json(json!({ "results": results, "total": results.len() })))
}
