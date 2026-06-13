use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{SisError, SisResult};
use schoolccb_common::auth::{Claims, require_any_role};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/admission/classrooms",
            get(list_classrooms).post(create_classroom),
        )
        .route(
            "/api/admission/classrooms/{id}",
            get(get_classroom)
                .put(update_classroom)
                .delete(delete_classroom),
        )
        .route("/api/admission/classrooms/{id}/availability", get(classroom_availability))
        .route("/api/admission/vacancy-check", get(vacancy_check))
}

async fn list_classrooms(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
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
    let rooms = sqlx::query_as::<_, schoolccb_common::admission::Classroom>(
        "SELECT id, name, capacity, location, active, created_at FROM classrooms ORDER BY name",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(json!({ "classrooms": rooms })))
}

async fn get_classroom(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
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
    let room = sqlx::query_as::<_, schoolccb_common::admission::Classroom>(
        "SELECT id, name, capacity, location, active, created_at FROM classrooms WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(crate::error::SisError::NotFound(
        "Sala no encontrada".into(),
    ))?;
    Ok(Json(json!({ "classroom": room })))
}

async fn create_classroom(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::admission::CreateClassroomPayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;
    if payload.name.trim().is_empty() {
        return Err(crate::error::SisError::Validation(
            "Nombre obligatorio".into(),
        ));
    }
    let id = Uuid::new_v4();
    let result = sqlx::query_as::<_, schoolccb_common::admission::Classroom>(
        "INSERT INTO classrooms (id, name, capacity, location) VALUES ($1, $2, $3, $4)
         RETURNING id, name, capacity, location, active, created_at",
    )
    .bind(id)
    .bind(&payload.name)
    .bind(payload.capacity)
    .bind(&payload.location)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(json!({ "classroom": result })))
}

async fn update_classroom(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::admission::UpdateClassroomPayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;
    let current = sqlx::query_as::<_, schoolccb_common::admission::Classroom>(
        "SELECT id, name, capacity, location, active, created_at FROM classrooms WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(crate::error::SisError::NotFound(
        "Sala no encontrada".into(),
    ))?;
    let name = payload.name.unwrap_or(current.name);
    let capacity = payload.capacity.unwrap_or(current.capacity);
    let location = payload.location.or(current.location);
    let active = payload.active.unwrap_or(current.active);
    let result = sqlx::query_as::<_, schoolccb_common::admission::Classroom>(
        "UPDATE classrooms SET name = $1, capacity = $2, location = $3, active = $4 WHERE id = $5
         RETURNING id, name, capacity, location, active, created_at",
    )
    .bind(&name)
    .bind(capacity)
    .bind(&location)
    .bind(active)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(json!({ "classroom": result })))
}

async fn delete_classroom(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;
    sqlx::query("DELETE FROM classrooms WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({ "message": "Sala eliminada" })))
}

async fn classroom_availability(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Admision"],
    )?;

    let room = sqlx::query_as::<_, (Uuid, String, i32, Option<String>, bool)>(
        "SELECT id, name, capacity, location, active FROM classrooms WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(crate::error::SisError::NotFound("Sala no encontrada".into()))?;

    let enrolled: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::bigint FROM enrollments e
         JOIN courses c ON c.id = e.course_id
         WHERE c.classroom_id = $1 AND e.active = true
         AND e.year = EXTRACT(YEAR FROM NOW())::int",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    let capacity = room.2 as i64;
    let available = (capacity - enrolled.0).max(0);

    Ok(Json(json!({
        "classroom": {
            "id": room.0,
            "name": room.1,
            "capacity": capacity,
            "location": room.3,
            "active": room.4,
            "enrolled": enrolled.0,
            "available": available,
            "usage_pct": if capacity > 0 {
                format!("{:.0}%", (enrolled.0 as f64 / capacity as f64) * 100.0)
            } else { "0%".to_string() },
        }
    })))
}

async fn vacancy_check(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Admision"],
    )?;

    let results = sqlx::query_as::<_, (String, i64, i64)>(
        r#"
        SELECT c.grade_level, COALESCE(SUM(cl.capacity), 0)::bigint as total_capacity,
               (SELECT COUNT(*) FROM enrollments e2
                JOIN courses c2 ON c2.id = e2.course_id
                WHERE c2.grade_level = c.grade_level AND e2.active = true AND e2.year = EXTRACT(YEAR FROM NOW())::int) as enrolled
        FROM courses c
        LEFT JOIN classrooms cl ON cl.id = c.classroom_id
        GROUP BY c.grade_level
        ORDER BY c.grade_level
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let vacancies: Vec<Value> = results
        .into_iter()
        .map(|(level, cap, enrolled)| {
            json!({
                "grade_level": level,
                "total_capacity": cap,
                "enrolled_count": enrolled,
                "available": (cap - enrolled).max(0),
            })
        })
        .collect();

    Ok(Json(json!({ "vacancies": vacancies })))
}
