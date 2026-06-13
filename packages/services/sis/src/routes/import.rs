use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use serde::Serialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{SisError, SisResult};
use schoolccb_common::auth::{Claims, require_any_role};

#[derive(Debug, Serialize)]
struct RowResult {
    row: usize,
    rut: String,
    nombre: String,
    status: &'static str,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct ImportReport {
    total: usize,
    imported: usize,
    skipped: usize,
    errores: usize,
    detalle: Vec<RowResult>,
}

const INSERT_SQL: &str = r#"
    INSERT INTO students
        (id, rut, first_name, last_name, email, phone,
         grade_level, section, cod_nivel, condicion, prioritario, nee,
         diseases, allergies,
         emergency_contact_name, emergency_contact_phone, emergency_contact_relation)
    VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
"#;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/students/import", post(import_csv))
}

async fn import_csv(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> SisResult<(StatusCode, Json<Value>)> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    let csv_content = payload
        .get("csv")
        .and_then(|v| v.as_str())
        .ok_or(SisError::Validation(
            "Se requiere campo 'csv' con el contenido del archivo".into(),
        ))?;

    let col_mapping = payload
        .get("columnas")
        .and_then(|v| v.as_object())
        .map(|m| {
            m.iter()
                .map(|(k, v)| (k.to_uppercase(), v.as_str().unwrap_or(k).to_uppercase()))
                .collect::<std::collections::HashMap<_, _>>()
        });

    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(csv_content.as_bytes());

    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| SisError::Validation(format!("Error en cabeceras CSV: {e}")))?
        .iter()
        .map(|h| {
            let upper = h.to_uppercase();
            col_mapping
                .as_ref()
                .and_then(|m| m.get(&upper))
                .cloned()
                .unwrap_or(upper)
        })
        .collect();

    let mut detalle: Vec<RowResult> = Vec::new();
    let mut imported = 0u32;

    let mut tx = state.pool.begin().await.map_err(SisError::Database)?;

    for (idx, result) in reader.records().enumerate() {
        let row_num = idx + 2;

        let record = match result {
            Ok(r) => r,
            Err(e) => {
                detalle.push(RowResult {
                    row: row_num,
                    rut: String::new(),
                    nombre: String::new(),
                    status: "error",
                    error: Some(format!("Fila malformada: {e}")),
                });
                continue;
            }
        };

        let get = |key: &str| -> &str {
            headers
                .iter()
                .position(|h| h == key)
                .and_then(|i| record.get(i))
                .unwrap_or("")
        };

        let rut_raw = get("RUT");
        let first_name = get("FIRST_NAME");
        let last_name = get("LAST_NAME");
        let email = get("EMAIL");
        let phone = get("PHONE");
        let grade_level = get("GRADE_LEVEL");
        let section = get("SECTION");
        let cod_nivel = get("COD_NIVEL");
        let condicion = get("CONDICION");
        let prioritario = get("PRIORITARIO");
        let nee = get("NEE");
        let diseases = get("DISEASES");
        let allergies = get("ALLERGIES");
        let emerg_name = get("EMERGENCY_CONTACT_NAME");
        let emerg_phone = get("EMERGENCY_CONTACT_PHONE");
        let emerg_rel = get("EMERGENCY_CONTACT_RELATION");

        if rut_raw.is_empty() || first_name.is_empty() || last_name.is_empty() {
            detalle.push(RowResult {
                row: row_num,
                rut: rut_raw.to_string(),
                nombre: format!("{} {}", first_name, last_name),
                status: "error",
                error: Some("RUT, FIRST_NAME y LAST_NAME son obligatorios".into()),
            });
            continue;
        }

        let rut = match schoolccb_common::rut::Rut::new(rut_raw) {
            Ok(r) => r,
            Err(e) => {
                detalle.push(RowResult {
                    row: row_num,
                    rut: rut_raw.to_string(),
                    nombre: format!("{} {}", first_name, last_name),
                    status: "error",
                    error: Some(format!("RUT inválido: {e}")),
                });
                continue;
            }
        };

        let (cond, prio, nee_val) = (
            if condicion.is_empty() {
                "AL"
            } else {
                condicion
            },
            if prioritario.is_empty() {
                "0"
            } else {
                prioritario
            },
            if nee.is_empty() { "N" } else { nee },
        );

        let id = Uuid::new_v4();
        match sqlx::query(INSERT_SQL)
            .bind(id)
            .bind(rut.as_str())
            .bind(first_name)
            .bind(last_name)
            .bind(if email.is_empty() {
                "sin-email@alumno.cl"
            } else {
                email
            })
            .bind(if phone.is_empty() { None } else { Some(phone) })
            .bind(grade_level)
            .bind(section)
            .bind(if cod_nivel.is_empty() {
                None
            } else {
                Some(cod_nivel)
            })
            .bind(cond)
            .bind(prio)
            .bind(nee_val)
            .bind(if diseases.is_empty() {
                None
            } else {
                Some(diseases)
            })
            .bind(if allergies.is_empty() {
                None
            } else {
                Some(allergies)
            })
            .bind(if emerg_name.is_empty() {
                None
            } else {
                Some(emerg_name)
            })
            .bind(if emerg_phone.is_empty() {
                None
            } else {
                Some(emerg_phone)
            })
            .bind(if emerg_rel.is_empty() {
                None
            } else {
                Some(emerg_rel)
            })
            .execute(&mut *tx)
            .await
        {
            Ok(_) => {
                imported += 1;
                detalle.push(RowResult {
                    row: row_num,
                    rut: rut_raw.to_string(),
                    nombre: format!("{} {}", first_name, last_name),
                    status: "imported",
                    error: None,
                });
            }
            Err(e) if matches!(&e, sqlx::Error::Database(d) if d.constraint() == Some("students_rut_key")) =>
            {
                detalle.push(RowResult {
                    row: row_num,
                    rut: rut_raw.to_string(),
                    nombre: format!("{} {}", first_name, last_name),
                    status: "skipped",
                    error: Some("RUT duplicado".into()),
                });
            }
            Err(e) => {
                detalle.push(RowResult {
                    row: row_num,
                    rut: rut_raw.to_string(),
                    nombre: format!("{} {}", first_name, last_name),
                    status: "error",
                    error: Some(format!("Error BD: {e}")),
                });
            }
        }
    }

    tx.commit().await.map_err(SisError::Database)?;

    let total = detalle.len();
    let errores = detalle.iter().filter(|r| r.status == "error").count();
    let skipped = detalle.iter().filter(|r| r.status == "skipped").count();
    let status_code = if errores > 0 {
        StatusCode::BAD_REQUEST
    } else {
        StatusCode::OK
    };

    let report = ImportReport {
        total,
        imported: imported as usize,
        skipped,
        errores,
        detalle,
    };

    Ok((status_code, Json(json!({ "importacion": report }))))
}
