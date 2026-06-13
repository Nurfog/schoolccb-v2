use axum::{
    extract::State,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::{SisError, SisResult};
use schoolccb_common::auth::{Claims, require_any_role};
use crate::AppState;
use schoolccb_common::rut::Rut;

#[derive(Deserialize)]
pub struct CsvImportPayload {
    pub csv_content: String,
}

pub struct CsvImportResult {
    pub imported: usize,
    pub errors: Vec<String>,
    pub total: usize,
}

pub fn parse_csv_fields(headers: &[String], row: &csv::StringRecord) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for (i, header) in headers.iter().enumerate() {
        if let Some(value) = row.get(i) {
            map.insert(header.to_lowercase(), value.to_string());
        }
    }
    map
}

pub fn get_field(map: &std::collections::HashMap<String, String>, names: &[&str]) -> Option<String> {
    for &name in names {
        if let Some(v) = map.get(name) {
            let trimmed = v.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }
    None
}

pub async fn import_students_csv(claims: Claims, State(state): State<AppState>, Json(payload): Json<CsvImportPayload>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let csv_content = payload.csv_content.as_bytes().to_vec();

    let content = String::from_utf8(csv_content).map_err(|_| SisError::Validation("El archivo debe ser UTF-8 válido".into()))?;
    let mut reader = csv::ReaderBuilder::new().flexible(true).from_reader(content.as_bytes());
    let headers = reader.headers()
        .map_err(|_| SisError::Validation("CSV sin encabezados".into()))?
        .iter().map(|h| h.to_string()).collect::<Vec<_>>();

    let required_headers = vec!["rut", "first_name", "last_name", "grade_level", "section"];
    for req in &required_headers {
        if !headers.iter().any(|h| h.to_lowercase() == *req) {
            return Err(SisError::Validation(format!("Columna requerida '{}' no encontrada. Columnas: {}", req, headers.join(", "))));
        }
    }

    let school_id = claims.school_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let mut result = CsvImportResult { imported: 0, errors: vec![], total: 0 };

    let mut tx = state.pool.begin().await?;

    for (line_num, row) in reader.records().enumerate() {
        result.total += 1;
        let row = match row {
            Ok(r) => r,
            Err(e) => { result.errors.push(format!("Línea {}: error de formato: {}", line_num + 2, e)); continue; }
        };

        let fields = parse_csv_fields(&headers, &row);
        let rut_str = match get_field(&fields, &["rut"]) {
            Some(r) => r,
            None => { result.errors.push(format!("Línea {}: RUT requerido", line_num + 2)); continue; }
        };

        let rut = match Rut::new(&rut_str) {
            Ok(r) => r,
            Err(_) => { result.errors.push(format!("Línea {}: RUT inválido: {}", line_num + 2, rut_str)); continue; }
        };

        let first_name = match get_field(&fields, &["first_name", "nombres", "nombre"]) {
            Some(v) => v,
            None => { result.errors.push(format!("Línea {}: nombre requerido", line_num + 2)); continue; }
        };
        let last_name = match get_field(&fields, &["last_name", "apellidos", "apellido"]) {
            Some(v) => v,
            None => { result.errors.push(format!("Línea {}: apellido requerido", line_num + 2)); continue; }
        };
        let grade_level = match get_field(&fields, &["grade_level", "nivel", "curso"]) {
            Some(v) => v,
            None => { result.errors.push(format!("Línea {}: nivel/curso requerido", line_num + 2)); continue; }
        };
        let section = match get_field(&fields, &["section", "seccion", "letra"]) {
            Some(v) => v,
            None => { result.errors.push(format!("Línea {}: sección requerida", line_num + 2)); continue; }
        };
        let email = get_field(&fields, &["email", "mail"]);
        let phone = get_field(&fields, &["phone", "telefono", "fono"]);
        let condicion = get_field(&fields, &["condicion", "condition"]).unwrap_or_else(|| "AL".to_string());
        let prioritario = get_field(&fields, &["prioritario", "prioridad"]).unwrap_or_else(|| "0".to_string());
        let nee = get_field(&fields, &["nee"]).unwrap_or_else(|| "N".to_string());

        if !["AL", "RE", "TR"].contains(&condicion.as_str()) {
            result.errors.push(format!("Línea {}: condición inválida '{}'. Debe ser AL, RE o TR", line_num + 2, condicion));
            continue;
        }

        let id = Uuid::new_v4();
        let insert_result = sqlx::query(
            r#"INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, condicion, prioritario, nee, school_id)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"#,
        ).bind(id).bind(rut.as_str()).bind(&first_name).bind(&last_name)
        .bind(&email).bind(&phone).bind(&grade_level).bind(&section)
        .bind(&condicion).bind(&prioritario).bind(&nee).bind(school_id)
        .execute(&mut *tx).await;

        match insert_result {
            Ok(_) => result.imported += 1,
            Err(e) => {
                if e.to_string().contains("students_rut_key") {
                    result.errors.push(format!("Línea {}: RUT {} ya existe en el sistema", line_num + 2, rut_str));
                } else {
                    result.errors.push(format!("Línea {}: error BD: {}", line_num + 2, e));
                }
            }
        }
    }

    tx.commit().await?;

    Ok(Json(json!({
        "imported": result.imported,
        "errors": result.errors,
        "total": result.total,
        "message": format!("Importación completada: {} insertados, {} errores de {} totales", result.imported, result.errors.len(), result.total)
    })))
}

pub async fn import_employees_csv(claims: Claims, State(state): State<AppState>, Json(payload): Json<CsvImportPayload>) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    let csv_content = payload.csv_content.as_bytes().to_vec();

    let content = String::from_utf8(csv_content).map_err(|_| SisError::Validation("El archivo debe ser UTF-8 válido".into()))?;
    let mut reader = csv::ReaderBuilder::new().flexible(true).from_reader(content.as_bytes());
    let headers = reader.headers()
        .map_err(|_| SisError::Validation("CSV sin encabezados".into()))?
        .iter().map(|h| h.to_string()).collect::<Vec<_>>();

    let required_headers = vec!["rut", "first_name", "last_name"];
    for req in &required_headers {
        if !headers.iter().any(|h| h.to_lowercase() == *req) {
            return Err(SisError::Validation(format!("Columna requerida '{}' no encontrada", req)));
        }
    }

    let mut result = CsvImportResult { imported: 0, errors: vec![], total: 0 };

    let mut tx = state.pool.begin().await?;

    for (line_num, row) in reader.records().enumerate() {
        result.total += 1;
        let row = match row {
            Ok(r) => r,
            Err(e) => { result.errors.push(format!("Línea {}: error de formato: {}", line_num + 2, e)); continue; }
        };

        let fields = parse_csv_fields(&headers, &row);
        let rut_str = match get_field(&fields, &["rut"]) {
            Some(r) => r,
            None => { result.errors.push(format!("Línea {}: RUT requerido", line_num + 2)); continue; }
        };
        let rut = match Rut::new(&rut_str) {
            Ok(r) => r,
            Err(_) => { result.errors.push(format!("Línea {}: RUT inválido: {}", line_num + 2, rut_str)); continue; }
        };
        let first_name = match get_field(&fields, &["first_name", "nombres", "nombre"]) {
            Some(v) => v,
            None => { result.errors.push(format!("Línea {}: nombre requerido", line_num + 2)); continue; }
        };
        let last_name = match get_field(&fields, &["last_name", "apellidos", "apellido"]) {
            Some(v) => v,
            None => { result.errors.push(format!("Línea {}: apellido requerido", line_num + 2)); continue; }
        };
        let email = get_field(&fields, &["email", "mail"]);
        let phone = get_field(&fields, &["phone", "telefono", "fono"]);
        let position = get_field(&fields, &["position", "cargo", "posicion"]);
        let category = get_field(&fields, &["category", "categoria", "cat"]);

        let id = Uuid::new_v4();
        let insert_result = sqlx::query(
            r#"INSERT INTO employees (id, rut, first_name, last_name, email, phone, position, category)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        ).bind(id).bind(rut.as_str()).bind(&first_name).bind(&last_name)
        .bind(&email).bind(&phone).bind(&position).bind(&category)
        .execute(&mut *tx).await;

        match insert_result {
            Ok(_) => result.imported += 1,
            Err(e) => {
                if e.to_string().contains("employees_rut_key") {
                    result.errors.push(format!("Línea {}: RUT {} ya existe en el sistema", line_num + 2, rut_str));
                } else {
                    result.errors.push(format!("Línea {}: error BD: {}", line_num + 2, e));
                }
            }
        }
    }

    tx.commit().await?;

    Ok(Json(json!({
        "imported": result.imported,
        "errors": result.errors,
        "total": result.total,
        "message": format!("Importación completada: {} insertados, {} errores de {} totales", result.imported, result.errors.len(), result.total)
    })))
}
