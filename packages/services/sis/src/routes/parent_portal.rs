use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post, put},
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::error::{SisError, SisResult};
use crate::routes::students::Claims;
use crate::AppState;

use std::fs;
use std::path::PathBuf;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/portal/parent/children", get(list_children))
        .route("/api/portal/parent/children/{id}/grades", get(child_grades))
        .route("/api/portal/parent/children/{id}/attendance", get(child_attendance))
        .route("/api/portal/parent/children/{id}/schedule", get(child_schedule))
        .route("/api/portal/parent/children/{id}/annotations", get(child_annotations))
        .route("/api/portal/parent/certificates", get(list_certificates))
        .route("/api/portal/parent/certificates/request", post(request_certificate))
        .route("/api/portal/parent/certificates/{id}/download", get(download_certificate))
        .route("/api/portal/parent/appointments", get(list_appointments).post(create_appointment))
        .route("/api/portal/parent/appointments/{id}", put(cancel_appointment))
        .route("/api/portal/parent/messages", get(list_messages).post(send_message))
        .route("/api/portal/parent/available-slots", get(available_slots))
}

fn require_apoderado(claims: &Claims) -> Result<(), crate::error::SisError> {
    if claims.role == "Apoderado" || claims.role == "GerenteGeneral" {
        Ok(())
    } else {
        Err(crate::error::SisError::Forbidden("Se requiere rol Apoderado".into()))
    }
}

async fn list_children(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    require_apoderado(&claims)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| crate::error::SisError::Unauthorized)?;

    let children = sqlx::query_as::<_, (Uuid, String, String, String, String, String)>(
        "SELECT s.id, s.first_name, s.last_name, s.rut, e.grade_level, e.section
         FROM guardian_relationships gr
         JOIN students s ON s.id = gr.student_id
         LEFT JOIN enrollments e ON e.student_id = s.id AND e.active = true
         WHERE gr.guardian_user_id = $1
         ORDER BY s.last_name, s.first_name",
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|(id, first, last, rut, grade, section)| {
        json!({
            "id": id,
            "name": format!("{first} {last}"),
            "rut": rut,
            "grade_level": grade,
            "section": section,
        })
    })
    .collect::<Vec<_>>();

    Ok(Json(json!({"children": children})))
}

async fn child_grades(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_apoderado(&claims)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| crate::error::SisError::Unauthorized)?;

    // Verify the child belongs to this parent
    let owned = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM guardian_relationships WHERE student_id = $1 AND guardian_user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .fetch_one(&state.pool)
    .await.unwrap_or(0);
    if owned == 0 {
        return Err(crate::error::SisError::NotFound("Alumno no encontrado".into()));
    }

    let grades = sqlx::query_as::<_, (String, String, f64, String)>(
        "SELECT sub.name as subject, g.name as evaluation, g.value, g.date::text
         FROM grades g
         JOIN enrollments e ON e.id = g.enrollment_id AND e.student_id = $1 AND e.active = true
         LEFT JOIN subjects sub ON sub.id = g.subject_id
         WHERE g.value IS NOT NULL
         ORDER BY g.date DESC
         LIMIT 50",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|(sub, eval, val, date)| json!({"subject": sub, "evaluation": eval, "value": val, "date": date}))
    .collect::<Vec<_>>();

    // Get averages per subject
    let averages = sqlx::query_as::<_, (String, f64)>(
        "SELECT sub.name, AVG(g.value)
         FROM grades g
         JOIN enrollments e ON e.id = g.enrollment_id AND e.student_id = $1 AND e.active = true
         LEFT JOIN subjects sub ON sub.id = g.subject_id
         WHERE g.value IS NOT NULL
         GROUP BY sub.name
         ORDER BY sub.name",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|(sub, avg)| json!({"subject": sub, "average": format!("{:.1}", avg)}))
    .collect::<Vec<_>>();

    Ok(Json(json!({"grades": grades, "averages": averages})))
}

async fn child_attendance(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_apoderado(&claims)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| crate::error::SisError::Unauthorized)?;

    let owned = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM guardian_relationships WHERE student_id = $1 AND guardian_user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .fetch_one(&state.pool)
    .await.unwrap_or(0);
    if owned == 0 {
        return Err(crate::error::SisError::NotFound("Alumno no encontrado".into()));
    }

    let monthly: Vec<Value> = sqlx::query_as::<_, (String, i64, i64, i64, i64)>(
        "SELECT to_char(date_trunc('month', date), 'YYYY-MM') as month,
                COUNT(*) as total,
                SUM(CASE WHEN status = 'present' THEN 1 ELSE 0 END) as present,
                SUM(CASE WHEN status = 'absent' THEN 1 ELSE 0 END) as absent,
                SUM(CASE WHEN status = 'late' THEN 1 ELSE 0 END) as late
         FROM attendance_records
         WHERE student_id = $1 AND date >= NOW() - INTERVAL '6 months'
         GROUP BY date_trunc('month', date)
         ORDER BY month",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|(m, total, present, absent, late)| {
        let pct = if total > 0 { (present as f64 / total as f64) * 100.0 } else { 0.0 };
        json!({"month": m, "total": total, "present": present, "absent": absent, "late": late, "percentage": format!("{:.1}", pct)})
    })
    .collect();

    Ok(Json(json!({"attendance": monthly})))
}

async fn child_schedule(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_apoderado(&claims)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| crate::error::SisError::Unauthorized)?;

    let owned = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM guardian_relationships WHERE student_id = $1 AND guardian_user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .fetch_one(&state.pool)
    .await.unwrap_or(0);
    if owned == 0 {
        return Err(crate::error::SisError::NotFound("Alumno no encontrado".into()));
    }

    let schedule = sqlx::query_as::<_, (String, String, String)>(
        "SELECT sub.name, cs.day_of_week, cs.time_slot
         FROM course_subjects cs
         JOIN subjects sub ON sub.id = cs.subject_id
         JOIN enrollments e ON e.course_id = cs.course_id AND e.student_id = $1 AND e.active = true
         ORDER BY cs.day_of_week, cs.time_slot",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await.unwrap_or_default()
    .into_iter()
    .map(|(sub, day, time)| json!({"subject": sub, "day": day, "time": time}))
    .collect::<Vec<_>>();

    Ok(Json(json!({"schedule": schedule})))
}

async fn child_annotations(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_apoderado(&claims)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| crate::error::SisError::Unauthorized)?;

    let owned = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM guardian_relationships WHERE student_id = $1 AND guardian_user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .fetch_one(&state.pool)
    .await.unwrap_or(0);
    if owned == 0 {
        return Err(crate::error::SisError::NotFound("Alumno no encontrado".into()));
    }

    let annotations = sqlx::query_as::<_, (String, String, String, String, Option<String>)>(
        "SELECT a.annotation_type, a.description, a.severity, a.created_at::text, u.name
         FROM student_annotations a
         LEFT JOIN users u ON u.id = a.created_by
         WHERE a.student_id = $1
         ORDER BY a.created_at DESC
         LIMIT 20",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await.unwrap_or_default()
    .into_iter()
    .map(|(t, desc, sev, date, teacher)| json!({"type": t, "description": desc, "severity": sev, "date": date, "teacher": teacher}))
    .collect::<Vec<_>>();

    Ok(Json(json!({"annotations": annotations})))
}

async fn list_certificates(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    require_apoderado(&claims)?;

    let types = vec![
        json!({"id": "alumno_regular", "name": "Certificado de Alumno Regular", "cost": 0, "description": "Certifica que el alumno se encuentra matriculado"}),
        json!({"id": "notas", "name": "Certificado de Notas", "cost": 0, "description": "Historial académico con calificaciones"}),
        json!({"id": "asistencia", "name": "Certificado de Asistencia", "cost": 0, "description": "Registro de asistencia del alumno"}),
        json!({"id": "conducta", "name": "Certificado de Conducta", "cost": 0, "description": "Certificado de buena conducta"}),
    ];

    let certificates = sqlx::query_as::<_, (Uuid, String, String, String)>(
        "SELECT c.id, c.certificate_type, c.status, c.created_at::text
         FROM portal_certificates c
         WHERE c.requested_by = $1
         ORDER BY c.created_at DESC
         LIMIT 20",
    )
    .bind(Uuid::parse_str(&claims.sub).unwrap_or_default())
    .fetch_all(&state.pool)
    .await.unwrap_or_default()
    .into_iter()
    .map(|(id, ctype, status, date)| json!({"id": id, "type": ctype, "status": status, "date": date}))
    .collect::<Vec<_>>();

    Ok(Json(json!({"certificate_types": types, "my_certificates": certificates})))
}

async fn request_certificate(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> SisResult<Json<Value>> {
    require_apoderado(&claims)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| SisError::Unauthorized)?;
    let cert_type = payload.get("certificate_type").and_then(|v| v.as_str()).unwrap_or("");
    let student_id = payload.get("student_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(SisError::Validation("student_id requerido".into()))?;

    if cert_type.is_empty() {
        return Err(SisError::Validation("Tipo de certificado requerido".into()));
    }

    // Generate PDF
    let student_info = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT s.first_name || ' ' || s.last_name, s.rut, COALESCE(e.grade_level, ''), COALESCE(sch.name, '')
         FROM students s
         LEFT JOIN enrollments e ON e.student_id = s.id AND e.active = true
         LEFT JOIN schools sch ON sch.id = e.school_id
         WHERE s.id = $1",
    )
    .bind(student_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(SisError::NotFound("Alumno no encontrado".into()))?;

    let output_dir = std::env::var("PDF_OUTPUT_DIR").unwrap_or_else(|_| "/tmp/certificates".into());
    let dir = PathBuf::from(&output_dir);
    let _ = fs::create_dir_all(&dir);
    let filename = format!("{}_{}_{}.txt", cert_type, student_id, chrono::Utc::now().format("%Y%m%d"));

    // Generate text-based certificate (PDF generation requires fixing dep conflict)
    let content = format!(
        "Certificado: {}\nAlumno: {}\nRUT: {}\nCurso: {}\nColegio: {}\nFecha: {}\n",
        cert_type, student_info.0, student_info.1, student_info.2, student_info.3,
        chrono::Utc::now().format("%d/%m/%Y")
    );
    let _ = fs::write(dir.join(&filename), &content);

    let cert_id = Uuid::new_v4();
    let file_url = format!("/certificates/{filename}");
    sqlx::query(
        "INSERT INTO portal_certificates (id, certificate_type, student_id, requested_by, status, file_url)
         VALUES ($1, $2, $3, $4, 'issued', $5)",
    )
    .bind(cert_id)
    .bind(cert_type)
    .bind(student_id)
    .bind(user_id)
    .bind(&file_url)
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"id": cert_id, "file_url": file_url, "status": "issued"})))
}

async fn download_certificate(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_apoderado(&claims)?;
    let cert = sqlx::query_as::<_, (String,)>(
        "SELECT file_url FROM portal_certificates WHERE id = $1 AND requested_by = $2",
    )
    .bind(id)
    .bind(Uuid::parse_str(&claims.sub).unwrap_or_default())
    .fetch_optional(&state.pool)
    .await?
    .ok_or(SisError::NotFound("Certificado no encontrado".into()))?;

    Ok(Json(json!({"file_url": cert.0})))
}

// ─── Appointments ───

async fn list_appointments(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    require_apoderado(&claims)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| SisError::Unauthorized)?;

    let appointments = sqlx::query_as::<_, (Uuid, String, String, String, String)>(
        "SELECT id, appointment_type, reason, status, preferred_date::text
         FROM support_appointments
         WHERE requested_by = $1
         ORDER BY created_at DESC
         LIMIT 20",
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await.unwrap_or_default()
    .into_iter()
    .map(|(id, atype, reason, status, date)| json!({"id": id, "type": atype, "reason": reason, "status": status, "date": date}))
    .collect::<Vec<_>>();

    Ok(Json(json!({"appointments": appointments})))
}

async fn create_appointment(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> SisResult<Json<Value>> {
    require_apoderado(&claims)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| SisError::Unauthorized)?;
    let id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO support_appointments (id, requested_by, appointment_type, reason, preferred_date, preferred_time)
         VALUES ($1, $2, $3, $4, $5::date, $6::time)",
    )
    .bind(id)
    .bind(user_id)
    .bind(payload.get("type").and_then(|v| v.as_str()).unwrap_or("general"))
    .bind(payload.get("reason").or_else(|| payload.get("notes")).and_then(|v| v.as_str()))
    .bind(payload.get("date").and_then(|v| v.as_str()))
    .bind(payload.get("time").and_then(|v| v.as_str()))
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"id": id})))
}

async fn cancel_appointment(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_apoderado(&claims)?;
    sqlx::query("UPDATE support_appointments SET status = 'cancelled', updated_at = NOW() WHERE id = $1 AND requested_by = $2")
        .bind(id)
        .bind(Uuid::parse_str(&claims.sub).unwrap_or_default())
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({"message": "Cita cancelada"})))
}

// ─── Messages ───

async fn list_messages(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    require_apoderado(&claims)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| SisError::Unauthorized)?;

    let messages = sqlx::query_as::<_, (Uuid, String, String, String, bool, String)>(
        "SELECT pm.id, u.name as teacher, pm.subject, pm.message, pm.is_read, pm.created_at::text
         FROM parent_messages pm
         JOIN users u ON u.id = pm.teacher_id
         WHERE pm.parent_id = $1
         ORDER BY pm.created_at DESC
         LIMIT 50",
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await.unwrap_or_default()
    .into_iter()
    .map(|(id, teacher, subject, msg, read, date)| json!({"id": id, "teacher": teacher, "subject": subject, "message": msg, "is_read": read, "date": date}))
    .collect::<Vec<_>>();

    Ok(Json(json!({"messages": messages})))
}

async fn send_message(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> SisResult<Json<Value>> {
    require_apoderado(&claims)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| SisError::Unauthorized)?;
    let id = Uuid::new_v4();

    // Lookup teacher_id: support UUID directly or name string
    let teacher_id = if let Some(tid) = payload.get("teacher_id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()) {
        Some(tid)
    } else if let Some(name) = payload.get("teacher").and_then(|v| v.as_str()) {
        sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM users WHERE name ILIKE $1 AND role = 'Profesor' LIMIT 1",
        )
        .bind(format!("%{}%", name))
        .fetch_optional(&state.pool)
        .await?
    } else {
        None
    };

    let student_id = payload.get("student_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());

    sqlx::query(
        "INSERT INTO parent_messages (id, parent_id, teacher_id, student_id, subject, message)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(user_id)
    .bind(teacher_id)
    .bind(student_id)
    .bind(payload.get("subject").and_then(|v| v.as_str()).unwrap_or("Sin asunto"))
    .bind(payload.get("message").and_then(|v| v.as_str()).unwrap_or(""))
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"id": id})))
}

async fn available_slots(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    require_apoderado(&claims)?;

    let slots = sqlx::query_as::<_, (Uuid, String, i32, String, String)>(
        "SELECT ts.id, u.name as teacher, ts.day_of_week, ts.start_time::text, ts.end_time::text
         FROM teacher_available_slots ts
         JOIN users u ON u.id = ts.teacher_id
         WHERE ts.is_booked = false
         ORDER BY ts.day_of_week, ts.start_time",
    )
    .fetch_all(&state.pool)
    .await.unwrap_or_default()
    .into_iter()
    .map(|(id, teacher, day, start, end)| json!({"id": id, "teacher": teacher, "day": day, "start": start, "end": end}))
    .collect::<Vec<_>>();

    Ok(Json(json!({"available_slots": slots})))
}
