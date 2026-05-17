use std::fs;
use std::io::BufWriter;
use std::path::Path;

use printpdf::*;

pub fn generate_certificate(
    output_dir: &str,
    cert_type: &str,
    student_name: &str,
    student_rut: &str,
    grade_level: &str,
    school_name: &str,
    issue_date: &str,
) -> Result<String, String> {
    fs::create_dir_all(output_dir).map_err(|e| format!("Error creando directorio: {e}"))?;

    let (doc, page1, layer1) = PdfDocument::new(
        format!("Certificado - {student_name}"),
        Mm(210.0),
        Mm(297.0),
        "Layer 1",
    );

    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();

    let layer = doc.get_page(page1).get_layer(layer1);

    let title = match cert_type {
        "alumno_regular" => "CERTIFICADO DE ALUMNO REGULAR",
        "notas" => "CERTIFICADO DE NOTAS",
        "asistencia" => "CERTIFICADO DE ASISTENCIA",
        "conducta" => "CERTIFICADO DE CONDUCTA",
        _ => "CERTIFICADO",
    };

    layer.use_text(title, 16.0, Mm(20.0), Mm(270.0), &font_bold);
    layer.use_text("─".repeat(60).as_str(), 8.0, Mm(20.0), Mm(260.0), &font);

    layer.use_text(format!("El {school_name} certifica que:"), 11.0, Mm(20.0), Mm(240.0), &font);
    layer.use_text(student_name.to_string(), 13.0, Mm(20.0), Mm(225.0), &font_bold);
    layer.use_text(format!("RUT: {student_rut}"), 11.0, Mm(20.0), Mm(215.0), &font);
    layer.use_text(format!("Curso: {grade_level}"), 11.0, Mm(20.0), Mm(205.0), &font);

    let body = match cert_type {
        "alumno_regular" => "Se encuentra matriculado(a) en este establecimiento, asistiendo regularmente a clases.",
        "notas" => "Ha obtenido las calificaciones registradas en su historial académico.",
        "asistencia" => "Presenta el registro de asistencia según los datos del establecimiento.",
        "conducta" => "Mantiene una conducta conforme a las normas del establecimiento.",
        _ => "Se extiende el presente certificado para los fines que estime conveniente.",
    };

    layer.use_text(body, 11.0, Mm(20.0), Mm(185.0), &font);

    layer.use_text("─".repeat(60).as_str(), 8.0, Mm(20.0), Mm(50.0), &font);
    layer.use_text(format!("Emitido el {issue_date}"), 9.0, Mm(20.0), Mm(40.0), &font);
    layer.use_text("SchoolCBB - Plataforma de Gestión Escolar", 8.0, Mm(20.0), Mm(30.0), &font);

    let filename = format!("certificate_{cert_type}_{}.pdf", student_name.replace(' ', "_"));
    let filepath = Path::new(output_dir).join(&filename);
    let file = fs::File::create(&filepath).map_err(|e| format!("Error guardando PDF: {e}"))?;
    doc.save(&mut BufWriter::new(file))
        .map_err(|e| format!("Error escribiendo PDF: {e}"))?;

    Ok(filename)
}
