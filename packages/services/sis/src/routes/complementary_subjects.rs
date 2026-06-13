use axum::{Json, Router, extract::{Path, State}, routing::{get, post, put}};
use serde_json::{Value, json};
use uuid::Uuid;
use crate::error::SisResult;
use schoolccb_common::auth::{Claims, require_any_role};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/courses/{id}/complementary-subjects", get(list_complementary).post(create_complementary))
        .route("/api/complementary-subjects/{id}", put(update_complementary).delete(delete_complementary))
        .route("/api/portal/parent/complementary-subjects", get(list_available))
        .route("/api/portal/parent/enroll-complementary", post(enroll_complementary))
}

async fn list_complementary(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    let subjects = sqlx::query_as::<_, (Uuid, String, String, i32, bool)>(
        "SELECT id, name, COALESCE(description, ''), max_students, is_active FROM complementary_subjects WHERE course_id = $1 ORDER BY name"
    ).bind(id).fetch_all(&state.pool).await?.into_iter()
    .map(|(sid, n, d, m, a)| json!({"id": sid, "name": n, "description": d, "max": m, "active": a}))
    .collect::<Vec<_>>();
    Ok(Json(json!({"subjects": subjects})))
}

async fn create_complementary(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    let sid = Uuid::new_v4();
    let is_active = p.get("is_active").or_else(|| p.get("active")).and_then(|v| v.as_bool()).unwrap_or(true);
    sqlx::query("INSERT INTO complementary_subjects (id, school_id, course_id, name, description, max_students, teacher_id, is_active) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
        .bind(sid)
        .bind(claims.school_id.as_deref().and_then(|s| Uuid::parse_str(s).ok()))
        .bind(id)
        .bind(p.get("name").and_then(|v| v.as_str()))
        .bind(p.get("description").and_then(|v| v.as_str()))
        .bind(p.get("max_students").and_then(|v| v.as_i64()).unwrap_or(0) as i32)
        .bind(p.get("teacher_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()))
        .bind(is_active)
        .execute(&state.pool).await?;
    Ok(Json(json!({"id": sid})))
}

async fn update_complementary(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    sqlx::query("UPDATE complementary_subjects SET name = COALESCE($1, name), description = COALESCE($2, description), max_students = COALESCE($3, max_students), is_active = COALESCE($4, is_active), updated_at = NOW() WHERE id = $5")
        .bind(p.get("name").and_then(|v| v.as_str()))
        .bind(p.get("description").and_then(|v| v.as_str()))
        .bind(p.get("max_students").and_then(|v| v.as_i64()).map(|v| v as i32))
        .bind(p.get("is_active").or_else(|| p.get("active")).and_then(|v| v.as_bool()))
        .bind(id).execute(&state.pool).await?;
    Ok(Json(json!({"message": "Asignatura actualizada"})))
}

async fn delete_complementary(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    sqlx::query("DELETE FROM complementary_subjects WHERE id = $1").bind(id).execute(&state.pool).await?;
    Ok(Json(json!({"message": "Asignatura eliminada"})))
}

async fn list_available(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Apoderado", "Alumno", "GerenteGeneral"])?;
    let subjects = sqlx::query_as::<_, (Uuid, String, String, i32)>(
        "SELECT id, name, COALESCE(description, ''), max_students FROM complementary_subjects WHERE is_active = true ORDER BY name"
    ).fetch_all(&state.pool).await?.into_iter()
    .map(|(id, n, d, m)| json!({"id": id, "name": n, "description": d, "max": m}))
    .collect::<Vec<_>>();
    Ok(Json(json!({"subjects": subjects})))
}

async fn enroll_complementary(claims: Claims, State(state): State<AppState>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Apoderado", "Alumno", "GerenteGeneral"])?;
    let subject_id = p.get("subject_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(crate::error::SisError::Validation("subject_id requerido".into()))?;

    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| crate::error::SisError::Unauthorized)?;

    // Check capacity
    let (max, enrolled): (i32, i64) = sqlx::query_as(
        "SELECT cs.max_students, (SELECT COUNT(*) FROM complementary_subject_enrollments WHERE subject_id = $1)
         FROM complementary_subjects cs WHERE cs.id = $1 AND cs.is_active = true",
    )
    .bind(subject_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(crate::error::SisError::NotFound("Asignatura no encontrada".into()))?;

    if max > 0 && enrolled >= max as i64 {
        return Err(crate::error::SisError::Validation("La asignatura ha alcanzado su cupo máximo".into()));
    }

    let eid = Uuid::new_v4();
    sqlx::query("INSERT INTO complementary_subject_enrollments (id, subject_id, student_id) VALUES ($1, $2, $3)")
        .bind(eid).bind(subject_id).bind(user_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({"id": eid, "message": "Inscripción exitosa"})))
}
