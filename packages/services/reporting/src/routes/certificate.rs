use axum::{
    Json, Router,
    body::Body,
    extract::{Path, State},
    http::{Response, header},
    response::IntoResponse,
    routing::get,
};
use printpdf::*;
use serde::Serialize;
use serde_json::{Value, json};
use std::io::BufWriter;
use uuid::Uuid;

use crate::AppState;
use crate::error::{ReportError, ReportResult};

pub use schoolccb_common::auth::Claims;
use schoolccb_common::auth::require_any_role;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/reports/certificate/student/{student_id}",
            get(certificate_student_json),
        )
        .route(
            "/api/reports/certificate/student/{student_id}/pdf",
            get(certificate_student_pdf),
        )
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct StudentRow {
    id: Uuid,
    student_name: String,
    rut: String,
    grade_level: String,
    section: String,
    enrolled: bool,
}

async fn certificate_student_json(
    claims: Claims,
    State(state): State<AppState>,
    Path(student_id): Path<Uuid>,
) -> ReportResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor", "Apoderado"],
    )?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "reports",
    )
    .await
    .map_err(|e| ReportError::Forbidden(e))?;

    let student = sqlx::query_as::<_, StudentRow>(
        r#"SELECT s.id, CONCAT(s.first_name, ' ', s.last_name) as student_name, s.rut,
           s.grade_level, s.section, s.enrolled FROM students s WHERE s.id = $1"#,
    )
    .bind(student_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ReportError::NotFound("Estudiante no encontrado".into()))?;

    if !student.enrolled {
        return Err(ReportError::Validation("El estudiante no se encuentra matriculado".into()));
    }

    let year: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(year)::int FROM enrollments WHERE student_id = $1 AND active = true",
    )
    .bind(student_id)
    .fetch_one(&state.pool)
    .await?;

    let year = year.unwrap_or_else(|| {
        chrono::Utc::now().format("%Y").to_string().parse::<i32>().unwrap_or(2025)
    });

    let cert = schoolccb_common::reporting::CertificateRegular {
        student_id: student.id,
        student_name: student.student_name,
        rut: student.rut,
        grade_level: student.grade_level,
        section: student.section,
        year,
        enrollment_status: "Matriculado".to_string(),
        issued_at: chrono::Utc::now().format("%d/%m/%Y %H:%M").to_string(),
        issuer_name: claims.name.clone(),
    };

    Ok(Json(json!({ "certificate": cert })))
}

async fn certificate_student_pdf(
    claims: Claims,
    State(state): State<AppState>,
    Path(student_id): Path<Uuid>,
) -> ReportResult<Response<Body>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor", "Apoderado"],
    )?;
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "reports",
    )
    .await
    .map_err(|e| ReportError::Forbidden(e))?;

    let student = sqlx::query_as::<_, StudentRow>(
        r#"SELECT s.id, CONCAT(s.first_name, ' ', s.last_name) as student_name, s.rut,
           s.grade_level, s.section, s.enrolled FROM students s WHERE s.id = $1"#,
    )
    .bind(student_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ReportError::NotFound("Estudiante no encontrado".into()))?;

    if !student.enrolled {
        return Err(ReportError::Validation("El estudiante no se encuentra matriculado".into()));
    }

    let (doc, page1, layer1) = PdfDocument::new(
        format!("Certificado - {}", student.student_name),
        Mm(210.0),
        Mm(297.0),
        "Layer 1",
    );

    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();
    let layer = doc.get_page(page1).get_layer(layer1);

    let issued_at = chrono::Utc::now().format("%d/%m/%Y").to_string();

    layer.use_text("CERTIFICADO DE ALUMNO REGULAR", 16.0, Mm(20.0), Mm(270.0), &font_bold);
    layer.use_text("─".repeat(60).as_str(), 8.0, Mm(20.0), Mm(260.0), &font);
    layer.use_text("El establecimiento certifica que:", 11.0, Mm(20.0), Mm(240.0), &font);
    layer.use_text(&student.student_name, 13.0, Mm(20.0), Mm(225.0), &font_bold);
    layer.use_text(format!("RUT: {}", student.rut), 11.0, Mm(20.0), Mm(215.0), &font);
    layer.use_text(format!("Curso: {} - {}", student.grade_level, student.section), 11.0, Mm(20.0), Mm(205.0), &font);
    layer.use_text("Se encuentra matriculado(a) en este establecimiento, asistiendo regularmente a clases.", 11.0, Mm(20.0), Mm(185.0), &font);
    layer.use_text("─".repeat(60).as_str(), 8.0, Mm(20.0), Mm(50.0), &font);
    layer.use_text(format!("Emitido el {issued_at}"), 9.0, Mm(20.0), Mm(40.0), &font);
    layer.use_text(format!("Emitido por: {}", claims.name), 8.0, Mm(20.0), Mm(30.0), &font);
    layer.use_text("SchoolCBB - Plataforma de Gestión Escolar", 8.0, Mm(20.0), Mm(20.0), &font);

    let mut buf = Vec::new();
    doc.save(&mut BufWriter::new(&mut buf))
        .map_err(|e| ReportError::Other(format!("Error generando PDF: {e}")))?;

    let filename = format!("certificado_{}.pdf", student.student_name.replace(' ', "_"));

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"{}\"", filename)),
    ];

    Ok((headers, Body::from(buf)).into_response())
}
