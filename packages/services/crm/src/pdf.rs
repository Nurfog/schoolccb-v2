use std::fs;
use std::io::BufWriter;
use std::path::Path;

use printpdf::*;

pub fn generate_proposal_pdf(
    output_dir: &str,
    proposal_id: &str,
    client_name: &str,
    client_company: &str,
    client_rut: &str,
    client_email: &str,
    plan_name: &str,
    total_value: f64,
    discount: f64,
    status: &str,
) -> Result<String, String> {
    fs::create_dir_all(output_dir).map_err(|e| format!("Error creando directorio PDF: {e}"))?;

    let (doc, page1, layer1) = PdfDocument::new(
        format!("Propuesta Comercial - {client_name}"),
        Mm(210.0),
        Mm(297.0),
        "Layer 1",
    );

    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();

    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Title
    current_layer.use_text("PROPUESTA COMERCIAL", 18.0, Mm(20.0), Mm(270.0), &font_bold);

    // Client info
    current_layer.use_text(format!("Cliente: {client_name}"), 11.0, Mm(20.0), Mm(250.0), &font);
    current_layer.use_text(format!("Empresa: {client_company}"), 11.0, Mm(20.0), Mm(240.0), &font);
    current_layer.use_text(format!("RUT: {client_rut}"), 11.0, Mm(20.0), Mm(230.0), &font);
    current_layer.use_text(format!("Email: {client_email}"), 11.0, Mm(20.0), Mm(220.0), &font);

    // Separator
    current_layer.use_text("─".repeat(60).as_str(), 8.0, Mm(20.0), Mm(210.0), &font);

    // Plan details
    current_layer.use_text("DETALLE DEL PLAN", 14.0, Mm(20.0), Mm(200.0), &font_bold);
    current_layer.use_text(format!("Plan: {plan_name}"), 11.0, Mm(20.0), Mm(190.0), &font);

    // Pricing
    let discounted = total_value * (1.0 - discount / 100.0);
    current_layer.use_text(format!("Valor mensual: ${total_value:.0}"), 11.0, Mm(20.0), Mm(175.0), &font);

    if discount > 0.0 {
        current_layer.use_text(format!("Descuento: {discount:.0}%"), 11.0, Mm(20.0), Mm(165.0), &font);
        current_layer.use_text(format!("Valor final: ${discounted:.0}"), 13.0, Mm(20.0), Mm(155.0), &font_bold);
    }

    // Status
    let status_text = match status {
        "draft" => "Borrador",
        "sent" => "Enviada al cliente",
        "approved" => "Aprobada",
        "rejected" => "Rechazada",
        _ => status,
    };
    current_layer.use_text(format!("Estado: {status_text}"), 11.0, Mm(20.0), Mm(140.0), &font);

    // Footer
    current_layer.use_text("─".repeat(60).as_str(), 8.0, Mm(20.0), Mm(30.0), &font);
    current_layer.use_text(
        format!("Generado el {} | SchoolCBB", chrono::Local::now().format("%d/%m/%Y %H:%M")),
        8.0, Mm(20.0), Mm(25.0), &font,
    );
    current_layer.use_text("SchoolCBB - Plataforma de Gestión Escolar", 8.0, Mm(20.0), Mm(20.0), &font);

    let filename = format!("proposal_{proposal_id}.pdf");
    let filepath = Path::new(output_dir).join(&filename);

    let file = fs::File::create(&filepath).map_err(|e| format!("Error guardando PDF: {e}"))?;
    doc.save(&mut BufWriter::new(file))
        .map_err(|e| format!("Error escribiendo PDF: {e}"))?;

    Ok(filename)
}
