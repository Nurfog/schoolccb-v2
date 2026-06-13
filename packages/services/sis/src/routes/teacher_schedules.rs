use axum::{Json, Router, extract::{Path, State}, routing::{get, put}};
use serde_json::{Value, json};
use uuid::Uuid;
use crate::error::SisResult;
use schoolccb_common::auth::{Claims, require_any_role};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/hr/teachers/{id}/schedules", get(list_schedules).post(create_schedule))
        .route("/api/hr/schedules/{id}", put(update_schedule).delete(delete_schedule))
        .route("/api/hr/substitutes", get(list_substitutes).post(create_substitute))
        .route("/api/hr/teachers/{id}/hours", get(get_contract_hours).post(set_contract_hours))
        .route("/api/hr/teachers/{id}/extra-duties", get(list_extra_duties).post(create_extra_duty))
        .route("/api/hr/extra-duties/{id}", put(update_extra_duty).delete(delete_extra_duty))
        .route("/api/hr/interviews", get(list_interviews).post(create_interview))
        .route("/api/hr/interviews/{id}", get(get_interview).put(update_interview).delete(delete_interview))
}

async fn list_schedules(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    let schedules = sqlx::query_as::<_, (Uuid, i32, String, String, String, String)>(
        "SELECT ts.id, ts.day_of_week, ts.start_time::text, ts.end_time::text, ts.schedule_type, COALESCE(sub.name, '')
         FROM teacher_schedules ts LEFT JOIN subjects sub ON sub.id = ts.subject_id
         WHERE ts.teacher_id = $1 ORDER BY ts.day_of_week, ts.start_time"
    ).bind(id).fetch_all(&state.pool).await?.into_iter()
    .map(|(sid, day, st, et, typ, sub)| json!({"id": sid, "day": day, "start": st, "end": et, "type": typ, "subject": sub}))
    .collect::<Vec<_>>();
    Ok(Json(json!({"schedules": schedules})))
}

async fn create_schedule(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    let sid = Uuid::new_v4();
    sqlx::query("INSERT INTO teacher_schedules (id, teacher_id, day_of_week, start_time, end_time, schedule_type, subject_id, course_id, room)
                 VALUES ($1, $2, $3, $4::time, $5::time, $6, $7, $8, $9)")
        .bind(sid).bind(id)
        .bind(p.get("day").and_then(|v| v.as_i64()).unwrap_or(0) as i32)
        .bind(p.get("start").and_then(|v| v.as_str()))
        .bind(p.get("end").and_then(|v| v.as_str()))
        .bind(p.get("type").and_then(|v| v.as_str()).unwrap_or("class"))
        .bind(p.get("subject_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()))
        .bind(p.get("course_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()))
        .bind(p.get("room").and_then(|v| v.as_str()))
        .execute(&state.pool).await?;
    Ok(Json(json!({"id": sid})))
}
async fn update_schedule(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    sqlx::query("UPDATE teacher_schedules SET day_of_week = COALESCE($1, day_of_week), start_time = COALESCE($2::time, start_time), end_time = COALESCE($3::time, end_time), room = COALESCE($4, room), updated_at = NOW() WHERE id = $5")
        .bind(p.get("day").and_then(|v| v.as_i64()).map(|v| v as i32))
        .bind(p.get("start").and_then(|v| v.as_str()))
        .bind(p.get("end").and_then(|v| v.as_str()))
        .bind(p.get("room").and_then(|v| v.as_str()))
        .bind(id).execute(&state.pool).await?;
    Ok(Json(json!({"message": "Horario actualizado"})))
}
async fn delete_schedule(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    sqlx::query("DELETE FROM teacher_schedules WHERE id = $1").bind(id).execute(&state.pool).await?;
    Ok(Json(json!({"message": "Horario eliminado"})))
}

async fn list_substitutes(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    let subs = sqlx::query_as::<_, (Uuid, String, String, String, String)>(
        "SELECT ss.id, u1.name, u2.name, ss.schedule_date::text, COALESCE(ss.reason, '')
         FROM substitute_schedule ss JOIN users u1 ON u1.id = ss.original_teacher_id JOIN users u2 ON u2.id = ss.substitute_teacher_id
         ORDER BY ss.schedule_date DESC LIMIT 50"
    ).fetch_all(&state.pool).await?.into_iter()
    .map(|(id, orig, sub, date, reason)| json!({"id": id, "original": orig, "substitute": sub, "date": date, "reason": reason}))
    .collect::<Vec<_>>();
    Ok(Json(json!({"substitutes": subs})))
}

async fn create_substitute(claims: Claims, State(state): State<AppState>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO substitute_schedule (id, original_teacher_id, substitute_teacher_id, schedule_date, reason, approved_by) VALUES ($1, $2, $3, $4::date, $5, $6)")
        .bind(id)
        .bind(p.get("original_teacher_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()))
        .bind(p.get("substitute_teacher_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()))
        .bind(p.get("date").and_then(|v| v.as_str()))
        .bind(p.get("reason").and_then(|v| v.as_str()))
        .bind(Uuid::parse_str(&claims.sub).ok())
        .execute(&state.pool).await?;
    Ok(Json(json!({"id": id})))
}

async fn get_contract_hours(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "GerenteGeneral"])?;
    let hours = sqlx::query_as::<_, (i32, i32, i32, i32)>(
        "SELECT total_hours, class_hours, admin_hours, extra_hours FROM teacher_contract_hours WHERE teacher_id = $1"
    ).bind(id).fetch_optional(&state.pool).await?.unwrap_or((0, 0, 0, 0));
    Ok(Json(json!({"total": hours.0, "class": hours.1, "admin": hours.2, "extra": hours.3})))
}
async fn set_contract_hours(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "GerenteGeneral"])?;
    let total = p.get("total_hours").or_else(|| p.get("total")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let class_h = p.get("class_hours").or_else(|| p.get("class")).and_then(|v| v.as_i64()).map(|v| v as i32);
    let admin_h = p.get("admin_hours").or_else(|| p.get("admin")).and_then(|v| v.as_i64()).map(|v| v as i32);
    sqlx::query("INSERT INTO teacher_contract_hours (teacher_id, total_hours, class_hours, admin_hours) VALUES ($1, $2, $3, $4)
                 ON CONFLICT (teacher_id, academic_year_id) DO UPDATE SET total_hours = $2, class_hours = COALESCE($3, teacher_contract_hours.class_hours), admin_hours = COALESCE($4, teacher_contract_hours.admin_hours), updated_at = NOW()")
        .bind(id).bind(total)
        .bind(class_h)
        .bind(admin_h)
        .execute(&state.pool).await?;
    Ok(Json(json!({"message": "Horas asignadas"})))
}

async fn list_extra_duties(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "GerenteGeneral"])?;
    let duties = sqlx::query_as::<_, (Uuid, String, String, f64, bool)>(
        "SELECT id, duty_type, COALESCE(description, ''), extra_amount, is_paid FROM extra_duties WHERE teacher_id = $1 ORDER BY created_at DESC"
    ).bind(id).fetch_all(&state.pool).await?.into_iter()
    .map(|(did, dt, desc, amt, paid)| json!({"id": did, "type": dt, "description": desc, "amount": amt, "paid": paid}))
    .collect::<Vec<_>>();
    Ok(Json(json!({"duties": duties})))
}
async fn create_extra_duty(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "GerenteGeneral"])?;
    let did = Uuid::new_v4();
    sqlx::query("INSERT INTO extra_duties (id, teacher_id, duty_type, description, extra_amount, period, approved_by) VALUES ($1, $2, $3, $4, $5, $6, $7)")
        .bind(did).bind(id)
        .bind(p.get("type").and_then(|v| v.as_str()).unwrap_or("other"))
        .bind(p.get("description").and_then(|v| v.as_str()))
        .bind(p.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0))
        .bind(p.get("period").and_then(|v| v.as_str()))
        .bind(Uuid::parse_str(&claims.sub).ok())
        .execute(&state.pool).await?;
    Ok(Json(json!({"id": did})))
}
async fn update_extra_duty(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "GerenteGeneral"])?;
    sqlx::query("UPDATE extra_duties SET is_paid = COALESCE($1, is_paid), description = COALESCE($2, description) WHERE id = $3")
        .bind(p.get("is_paid").and_then(|v| v.as_bool()))
        .bind(p.get("description").and_then(|v| v.as_str()))
        .bind(id).execute(&state.pool).await?;
    Ok(Json(json!({"message": "Tarea extra actualizada"})))
}
async fn delete_extra_duty(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "GerenteGeneral"])?;
    sqlx::query("DELETE FROM extra_duties WHERE id = $1").bind(id).execute(&state.pool).await?;
    Ok(Json(json!({"message": "Tarea extra eliminada"})))
}

// ─── Interview Process ───

async fn list_interviews(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "GerenteGeneral", "DirectorRRHH"])?;
    let school_id = claims.school_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let interviews = sqlx::query_as::<_, (Uuid, String, String, String, String, String, String)>(
        "SELECT id, candidate_name, position, interview_date::text, result, status, created_at::text
         FROM interview_process WHERE ($1::uuid IS NULL OR school_id = $1)
         ORDER BY created_at DESC"
    ).bind(school_id).fetch_all(&state.pool).await?.into_iter()
    .map(|(id, name, pos, date, result, status, created)| json!({
        "id": id, "candidate": name, "position": pos, "date": date,
        "result": result, "status": status, "created_at": created,
    }))
    .collect::<Vec<_>>();
    Ok(Json(json!({"interviews": interviews})))
}

async fn get_interview(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "GerenteGeneral", "DirectorRRHH"])?;
    let interview = sqlx::query_as::<_, (Uuid, String, Option<String>, Option<String>, String, Option<Uuid>, Option<String>, String, Option<String>, String)>(
        "SELECT id, candidate_name, candidate_email, candidate_phone, position, interviewer_id,
                interview_date::text, result, notes, status
         FROM interview_process WHERE id = $1"
    ).bind(id).fetch_optional(&state.pool).await?
    .ok_or(crate::error::SisError::NotFound("Entrevista no encontrada".into()))?;
    Ok(Json(json!({"interview": {
        "id": interview.0, "candidate": interview.1, "email": interview.2,
        "phone": interview.3, "position": interview.4, "interviewer_id": interview.5,
        "date": interview.6, "result": interview.7, "notes": interview.8, "status": interview.9,
    }})))
}

async fn create_interview(claims: Claims, State(state): State<AppState>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "GerenteGeneral", "DirectorRRHH"])?;
    let school_id = claims.school_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let user_id = claims.sub.parse::<Uuid>().ok();
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO interview_process (id, candidate_name, candidate_email, candidate_phone, position,
         interviewer_id, interview_date, result, notes, status, school_id, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7::timestamptz, 'pending', $8, 'pendiente', $9, $10)"
    )
    .bind(id)
    .bind(p.get("candidate_name").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(p.get("candidate_email").and_then(|v| v.as_str()))
    .bind(p.get("candidate_phone").and_then(|v| v.as_str()))
    .bind(p.get("position").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(p.get("interviewer_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()))
    .bind(p.get("interview_date").and_then(|v| v.as_str()))
    .bind(p.get("notes").and_then(|v| v.as_str()))
    .bind(school_id)
    .bind(user_id)
    .execute(&state.pool).await?;
    Ok(Json(json!({"id": id})))
}

async fn update_interview(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>, Json(p): Json<Value>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "GerenteGeneral", "DirectorRRHH"])?;
    sqlx::query(
        "UPDATE interview_process SET
         result = COALESCE($1, result), notes = COALESCE($2, notes),
         status = COALESCE($3, status), interview_date = COALESCE($4::timestamptz, interview_date),
         interviewer_id = COALESCE($5, interviewer_id), updated_at = NOW()
         WHERE id = $6"
    )
    .bind(p.get("result").and_then(|v| v.as_str()))
    .bind(p.get("notes").and_then(|v| v.as_str()))
    .bind(p.get("status").and_then(|v| v.as_str()))
    .bind(p.get("interview_date").and_then(|v| v.as_str()))
    .bind(p.get("interviewer_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()))
    .bind(id)
    .execute(&state.pool).await?;
    Ok(Json(json!({"message": "Entrevista actualizada"})))
}

async fn delete_interview(claims: Claims, State(state): State<AppState>, Path(id): Path<Uuid>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "GerenteGeneral", "DirectorRRHH"])?;
    sqlx::query("DELETE FROM interview_process WHERE id = $1").bind(id).execute(&state.pool).await?;
    Ok(Json(json!({"message": "Entrevista eliminada"})))
}
