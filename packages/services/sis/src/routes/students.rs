use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{SisError, SisResult};

pub use schoolccb_common::auth::Claims;
use schoolccb_common::auth::require_any_role;

#[derive(sqlx::FromRow)]
pub struct RawStudent {
    pub id: Uuid,
    pub rut: String,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub grade_level: String,
    pub section: String,
    pub cod_nivel: Option<String>,
    pub condicion: String,
    pub prioritario: String,
    pub nee: String,
    pub diseases: Option<String>,
    pub allergies: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub emergency_contact_relation: Option<String>,
    pub enrolled: bool,
}

impl RawStudent {
    fn to_student(&self) -> schoolccb_common::student::Student {
        schoolccb_common::student::Student {
            id: self.id,
            rut: schoolccb_common::rut::Rut::new_unchecked(&self.rut),
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            email: self.email.clone(),
            phone: self.phone.clone(),
            grade_level: self.grade_level.clone(),
            section: self.section.clone(),
            cod_nivel: self.cod_nivel.clone(),
            condicion: match self.condicion.as_str() {
                "RE" => schoolccb_common::student::CondicionMatricula::Repitente,
                "TR" => schoolccb_common::student::CondicionMatricula::Trasladado,
                _ => schoolccb_common::student::CondicionMatricula::AlumnoRegular,
            },
            prioritario: match self.prioritario.as_str() {
                "1" => schoolccb_common::student::Prioritario::Si,
                "2" => schoolccb_common::student::Prioritario::Preferente,
                _ => schoolccb_common::student::Prioritario::No,
            },
            nee: match self.nee.as_str() {
                "T" => schoolccb_common::student::NEE::Transitoria,
                "P" => schoolccb_common::student::NEE::Permanente,
                _ => schoolccb_common::student::NEE::No,
            },
            enrolled: self.enrolled,
        }
    }

    fn medical_json(&self) -> Value {
        json!({
            "diseases": self.diseases,
            "allergies": self.allergies,
            "emergency_contact": {
                "name": self.emergency_contact_name,
                "phone": self.emergency_contact_phone,
                "relation": self.emergency_contact_relation
            }
        })
    }
}

#[derive(Deserialize)]
pub struct StudentFilter {
    pub grade_level: Option<String>,
    pub section: Option<String>,
    pub search: Option<String>,
}

const STUDENT_COLUMNS: &str = r#"
    id, rut, first_name, last_name, email, phone,
    grade_level, section, cod_nivel, condicion, prioritario, nee,
    diseases, allergies, emergency_contact_name,
    emergency_contact_phone, emergency_contact_relation, enrolled
"#;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/students", get(list_students).post(create_student))
        .route(
            "/api/students/{id}",
            get(get_student)
                .put(update_student)
                .delete(deactivate_student),
        )
        .route(
            "/api/students/{id}/guardians",
            get(list_guardians).post(add_guardian),
        )
        .route(
            "/api/students/{id}/guardians/{guardian_id}",
            delete(remove_guardian),
        )
        .route(
            "/api/students/annotations",
            post(create_annotation),
        )
        .route(
            "/api/students/{id}/annotations",
            get(list_student_annotations),
        )
}

async fn list_students(
    claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<StudentFilter>,
) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "students",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;

    let mut conditions = vec!["s.enrolled = true".to_string()];
    let mut bind_values: Vec<String> = vec![];

    if let Some(ref sid) = claims.school_id {
        conditions.push(format!("s.school_id = ${}::uuid", conditions.len() + 1));
        bind_values.push(sid.clone());
        if let Some(ref cid) = claims.corporation_id {
            conditions.push(format!("sch.corporation_id = ${}::uuid", conditions.len() + 1));
            bind_values.push(cid.clone());
        }
    }

    if let Some(ref gl) = filter.grade_level {
        conditions.push(format!("s.grade_level = ${}", conditions.len() + 1));
        bind_values.push(gl.clone());
    }
    if let Some(ref sec) = filter.section {
        conditions.push(format!("s.section = ${}", conditions.len() + 1));
        bind_values.push(sec.clone());
    }
    if let Some(ref q) = filter.search {
        conditions.push(format!(
            "(s.first_name ILIKE ${n} OR s.last_name ILIKE ${n} OR s.rut ILIKE ${n})",
            n = conditions.len() + 1
        ));
        bind_values.push(format!("%{}%", q));
    }

    let from_clause = if claims.school_id.is_some() {
        "FROM students s JOIN schools sch ON sch.id = s.school_id"
    } else {
        "FROM students s"
    };

    let sql = format!(
        "SELECT {} {} WHERE {} ORDER BY s.last_name, s.first_name",
        STUDENT_COLUMNS,
        from_clause,
        conditions.join(" AND ")
    );

    let mut query = sqlx::query_as::<_, RawStudent>(&sql);
    for val in &bind_values {
        query = query.bind(val);
    }

    let raw = query.fetch_all(&state.pool).await?;

    let students: Vec<Value> = raw.iter().map(|r| json!(r.to_student())).collect();

    Ok(Json(
        json!({ "students": students, "total": students.len() }),
    ))
}

async fn get_student(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &[
            "Administrador",
            "Sostenedor",
            "Director",
            "UTP",
            "Profesor",
            "Apoderado",
        ],
    )?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "students",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;

    let raw = sqlx::query_as::<_, RawStudent>(&format!(
        "SELECT {} FROM students WHERE id = $1",
        STUDENT_COLUMNS
    ))
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(SisError::NotFound("Alumno no encontrado".into()))?;

    Ok(Json(json!({
        "student": raw.to_student(),
        "medical": raw.medical_json()
    })))
}

async fn create_student(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::student::CreateStudentPayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let rut = schoolccb_common::rut::Rut::new(&payload.rut)
        .map_err(|e| SisError::Validation(format!("RUT inválido: {e}")))?;

    if payload.first_name.trim().is_empty() || payload.last_name.trim().is_empty() {
        return Err(SisError::Validation(
            "Nombre y apellido son obligatorios".into(),
        ));
    }
    if payload.grade_level.trim().is_empty() || payload.section.trim().is_empty() {
        return Err(SisError::Validation(
            "Curso y sección son obligatorios".into(),
        ));
    }

    let condicion = payload.condicion.as_deref().unwrap_or("AL");
    let prioritario = payload.prioritario.as_deref().unwrap_or("0");
    let nee = payload.nee.as_deref().unwrap_or("N");

    if !["AL", "RE", "TR"].contains(&condicion) {
        return Err(SisError::Validation(
            "Condición inválida: use AL, RE o TR".into(),
        ));
    }
    if !["0", "1", "2"].contains(&prioritario) {
        return Err(SisError::Validation(
            "Prioritario inválido: use 0, 1 o 2".into(),
        ));
    }
    if !["N", "T", "P"].contains(&nee) {
        return Err(SisError::Validation("NEE inválido: use N, T o P".into()));
    }

    let school_id = claims.school_id.and_then(|s| Uuid::parse_str(&s).ok());

    let id = Uuid::new_v4();
    let result = sqlx::query_as::<_, RawStudent>(
        r#"
        INSERT INTO students (id, rut, first_name, last_name, email, phone,
                              grade_level, section, cod_nivel, condicion, prioritario, nee,
                              diseases, allergies,
                              emergency_contact_name, emergency_contact_phone, emergency_contact_relation,
                              school_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
        RETURNING id, rut, first_name, last_name, email, phone,
                  grade_level, section, cod_nivel, condicion, prioritario, nee,
                  diseases, allergies, emergency_contact_name,
                  emergency_contact_phone, emergency_contact_relation, enrolled
        "#,
    )
    .bind(id)
    .bind(rut.as_str())
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(payload.email.unwrap_or_default())
    .bind(&payload.phone)
    .bind(&payload.grade_level)
    .bind(&payload.section)
    .bind(&payload.cod_nivel)
    .bind(condicion)
    .bind(prioritario)
    .bind(nee)
    .bind(&payload.diseases)
    .bind(&payload.allergies)
    .bind(&payload.emergency_contact_name)
    .bind(&payload.emergency_contact_phone)
    .bind(&payload.emergency_contact_relation)
    .bind(school_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint() == Some("students_rut_key") {
                return SisError::Conflict("El RUT ya está registrado".into());
            }
        }
        SisError::Database(e)
    })?;

    Ok(Json(json!({ "student": result.to_student() })))
}

async fn update_student(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::student::UpdateStudentPayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let existing = sqlx::query_as::<_, RawStudent>(&format!(
        "SELECT {} FROM students WHERE id = $1",
        STUDENT_COLUMNS
    ))
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(SisError::NotFound("Alumno no encontrado".into()))?;

    let first_name = payload.first_name.unwrap_or(existing.first_name);
    let last_name = payload.last_name.unwrap_or(existing.last_name);
    let email = payload.email.or(existing.email);
    let phone = payload.phone.or(existing.phone);
    let grade_level = payload.grade_level.unwrap_or(existing.grade_level);
    let section = payload.section.unwrap_or(existing.section);
    let cod_nivel = payload.cod_nivel.or(existing.cod_nivel);

    let condicion = payload
        .condicion
        .as_deref()
        .unwrap_or(&existing.condicion)
        .to_string();
    if !["AL", "RE", "TR"].contains(&condicion.as_str()) {
        return Err(SisError::Validation(
            "Condición inválida: use AL, RE o TR".into(),
        ));
    }

    let prioritario = payload
        .prioritario
        .as_deref()
        .unwrap_or(&existing.prioritario)
        .to_string();
    if !["0", "1", "2"].contains(&prioritario.as_str()) {
        return Err(SisError::Validation(
            "Prioritario inválido: use 0, 1 o 2".into(),
        ));
    }

    let nee = payload.nee.as_deref().unwrap_or(&existing.nee).to_string();
    if !["N", "T", "P"].contains(&nee.as_str()) {
        return Err(SisError::Validation("NEE inválido: use N, T o P".into()));
    }

    let diseases = payload.diseases.or(existing.diseases);
    let allergies = payload.allergies.or(existing.allergies);
    let emergency_contact_name = payload
        .emergency_contact_name
        .or(existing.emergency_contact_name);
    let emergency_contact_phone = payload
        .emergency_contact_phone
        .or(existing.emergency_contact_phone);
    let emergency_contact_relation = payload
        .emergency_contact_relation
        .or(existing.emergency_contact_relation);

    let result = sqlx::query_as::<_, RawStudent>(
        r#"
        UPDATE students SET
            first_name = $1, last_name = $2, email = $3, phone = $4,
            grade_level = $5, section = $6, cod_nivel = $7,
            condicion = $8, prioritario = $9, nee = $10,
            diseases = $11, allergies = $12,
            emergency_contact_name = $13, emergency_contact_phone = $14,
            emergency_contact_relation = $15,
            updated_at = NOW()
        WHERE id = $16
        RETURNING id, rut, first_name, last_name, email, phone,
                  grade_level, section, cod_nivel, condicion, prioritario, nee,
                  diseases, allergies, emergency_contact_name,
                  emergency_contact_phone, emergency_contact_relation, enrolled
        "#,
    )
    .bind(&first_name)
    .bind(&last_name)
    .bind(&email)
    .bind(&phone)
    .bind(&grade_level)
    .bind(&section)
    .bind(&cod_nivel)
    .bind(&condicion)
    .bind(&prioritario)
    .bind(&nee)
    .bind(&diseases)
    .bind(&allergies)
    .bind(&emergency_contact_name)
    .bind(&emergency_contact_phone)
    .bind(&emergency_contact_relation)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "student": result.to_student() })))
}

async fn deactivate_student(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    let result = sqlx::query(
        "UPDATE students SET enrolled = false, updated_at = NOW() WHERE id = $1 AND enrolled = true",
    )
    .bind(id)
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(SisError::NotFound(
            "Alumno no encontrado o ya desactivado".into(),
        ));
    }

    Ok(Json(
        json!({ "message": "Alumno desactivado correctamente" }),
    ))
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
struct RawGuardian {
    id: Uuid,
    student_id: Uuid,
    guardian_user_id: Uuid,
    relationship: String,
    authorized_pickup: bool,
    receives_notifications: bool,
    guardian_name: String,
    guardian_rut: String,
}

async fn list_guardians(
    claims: Claims,
    State(state): State<AppState>,
    Path(student_id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;

    let guardians = sqlx::query_as::<_, RawGuardian>(
        r#"
        SELECT gr.id, gr.student_id, gr.guardian_user_id, gr.relationship,
               gr.authorized_pickup, gr.receives_notifications,
               u.name as guardian_name, u.rut as guardian_rut
        FROM guardian_relationships gr
        JOIN users u ON u.id = gr.guardian_user_id
        WHERE gr.student_id = $1
        ORDER BY gr.created_at
        "#,
    )
    .bind(student_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "guardians": guardians })))
}

async fn add_guardian(
    claims: Claims,
    State(state): State<AppState>,
    Path(student_id): Path<Uuid>,
    Json(payload): Json<Value>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let guardian_user_id = payload
        .get("guardian_user_id")
        .and_then(|v| v.as_str())
        .and_then(|v| Uuid::parse_str(v).ok())
        .ok_or(SisError::Validation(
            "guardian_user_id es requerido (UUID válido)".into(),
        ))?;

    let relationship = payload
        .get("relationship")
        .and_then(|v| v.as_str())
        .ok_or(SisError::Validation("relationship es requerido".into()))?;

    let authorized_pickup = payload
        .get("authorized_pickup")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let receives_notifications = payload
        .get("receives_notifications")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    sqlx::query(
        r#"
        INSERT INTO guardian_relationships (id, student_id, guardian_user_id, relationship, authorized_pickup, receives_notifications)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(student_id)
    .bind(guardian_user_id)
    .bind(relationship)
    .bind(authorized_pickup)
    .bind(receives_notifications)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint() == Some("guardian_relationships_student_id_guardian_user_id_key") {
                return SisError::Conflict("El apoderado ya está vinculado a este alumno".into());
            }
            if db_err.constraint() == Some("guardian_relationships_guardian_user_id_fkey") {
                return SisError::Validation("El usuario apoderado no existe".into());
            }
        }
        SisError::Database(e)
    })?;

    Ok(Json(
        json!({ "message": "Apoderado vinculado correctamente" }),
    ))
}

async fn remove_guardian(
    claims: Claims,
    State(state): State<AppState>,
    Path((student_id, guardian_id)): Path<(Uuid, Uuid)>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let result = sqlx::query(
        "DELETE FROM guardian_relationships WHERE student_id = $1 AND guardian_user_id = $2",
    )
    .bind(student_id)
    .bind(guardian_id)
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(SisError::NotFound("Vínculo no encontrado".into()));
    }

    Ok(Json(
        json!({ "message": "Apoderado desvinculado correctamente" }),
    ))
}

async fn create_annotation(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::student::CreateAnnotationPayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor"])?;

    let id = Uuid::new_v4();
    let user_id = Uuid::parse_str(&claims.sub).ok();

    sqlx::query(
        "INSERT INTO student_annotations (id, student_id, annotation_type, description, severity, created_by)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(payload.student_id)
    .bind(&payload.annotation_type)
    .bind(&payload.description)
    .bind(&payload.severity)
    .bind(user_id)
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"id": id, "message": "Anotación creada"})))
}

async fn list_student_annotations(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"])?;

    let annotations = sqlx::query_as::<_, (String, String, String, String, Option<String>)>(
        "SELECT a.annotation_type, a.description, a.severity, a.created_at::text, u.name
         FROM student_annotations a
         LEFT JOIN users u ON u.id = a.created_by
         WHERE a.student_id = $1
         ORDER BY a.created_at DESC LIMIT 50",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await.unwrap_or_default()
    .into_iter()
    .map(|(t, d, s, c, teacher)| json!({"type": t, "description": d, "severity": s, "date": c, "teacher": teacher}))
    .collect::<Vec<_>>();

    Ok(Json(json!({"annotations": annotations})))
}
