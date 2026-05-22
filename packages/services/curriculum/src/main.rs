mod routes;

use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

pub struct AppState {
    pub kb: Vec<KbEntry>,
}

#[derive(Clone)]
pub struct KbEntry {
    pub text: String,
    pub fuente: String,
    pub nivel: String,
    pub asignatura: String,
}

fn normalize_nivel(s: &str) -> String {
    match s {
        "sala_cuna" | "sc_sala_cuna" => "Sala Cuna".into(),
        "nivel_medio" | "nm_nivel_medio" => "Nivel Medio".into(),
        "nivel_transicion" | "nt_nivel_transicion" => "Nivel Transición".into(),
        "general" => "General".into(),
        _ => {
            let s = s.replace('_', " ");
            let s = s.replace("basico", "Básico");
            let s = s.replace("medio", "Medio");
            let parts: Vec<&str> = s.splitn(2, ' ').collect();
            if parts.len() == 2 {
                let num = parts[0];
                let rest = parts[1];
                if num.chars().all(|c| c.is_ascii_digit()) {
                    format!("{num}° {rest}")
                } else {
                    s
                }
            } else {
                s
            }
        }
    }
}

fn normalize_asignatura(s: &str) -> String {
    let s = s.replace('_', " ");
    s.split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            c.next().map(|f| f.to_uppercase().to_string() + c.as_str()).unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
        .replace("Cs ", "Cs. ")
        .replace("Y C", "y C")
}

fn parse_dir_name(dir_name: &str) -> (String, String, String) {
    let parts: Vec<&str> = dir_name.splitn(3, '-').collect();
    let categoria = if !parts.is_empty() { parts[0] } else { "" };
    let nivel = if parts.len() > 1 { parts[1] } else { "" };
    let asignatura = if parts.len() > 2 { parts[2] } else { "" };

    let nivel = match categoria {
        "Educacion_Parvularia" => normalize_nivel(nivel),
        "EPJA" => {
            let nivel = nivel.replace('_', " ").replace("basica", "Básica").replace("media", "Media");
            let parts: Vec<&str> = nivel.splitn(2, ' ').collect();
            if parts.len() == 2 && parts[0].starts_with("nivel") {
                format!("Nivel {} {}", &parts[0][5..], parts[1])
            } else {
                nivel
            }
        }
        "Lengua_Indigena_7_8" => "7°-8° Básico".to_string(),
        "Pueblos_Originarios_1_6" => normalize_nivel(nivel),
        _ => {
            if nivel.is_empty() {
                "General".into()
            } else {
                normalize_nivel(nivel)
            }
        }
    };

    let asignatura = if asignatura.is_empty() {
        normalize_asignatura(categoria)
    } else {
        normalize_asignatura(asignatura)
    };

    (categoria.to_string(), nivel, asignatura)
}

fn load_kb(kb_dir: &str) -> Vec<KbEntry> {
    let mut entries = Vec::new();

    let dir = match std::fs::read_dir(kb_dir) {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!("Could not read KB directory {kb_dir}: {e}");
            return entries;
        }
    };

    for entry in dir.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let dir_name = entry.file_name().to_string_lossy().to_string();
        let kb_path = entry.path().join("kb.json");
        if !kb_path.exists() {
            continue;
        }
        let content = match std::fs::read_to_string(&kb_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Could not read {kb_path:?}: {e}");
                continue;
            }
        };
        let kb: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("Could not parse {kb_path:?}: {e}");
                continue;
            }
        };

        let fuente = kb["source"].as_str().unwrap_or("").to_string();
        let (nivel, asignatura) = if fuente.contains('/') {
            let parts: Vec<&str> = fuente.split('/').collect();
            if parts.len() >= 4 {
                (normalize_nivel(parts[2]), normalize_asignatura(parts[3]))
            } else if parts.len() == 3 {
                (normalize_nivel(parts[1]), normalize_asignatura(parts[2]))
            } else {
                (parse_dir_name(&dir_name).1, parse_dir_name(&dir_name).2)
            }
        } else {
            (parse_dir_name(&dir_name).1, parse_dir_name(&dir_name).2)
        };

        if let Some(chunks) = kb["chunks"].as_array() {
            for chunk in chunks {
                let text = chunk["text"].as_str().unwrap_or("").to_string();
                if text.is_empty() {
                    continue;
                }
                let chunk_source = chunk["source"]
                    .as_str()
                    .unwrap_or(&fuente)
                    .to_string();
                entries.push(KbEntry {
                    text,
                    fuente: chunk_source,
                    nivel: nivel.clone(),
                    asignatura: asignatura.clone(),
                });
            }
        }
    }

    tracing::info!("Loaded {} KB entries from {kb_dir}", entries.len());
    entries
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    dotenvy::dotenv().ok();

    let kb_dir = std::env::var("CURRICULUM_KB_DIR")
        .unwrap_or_else(|_| ".agents/skills/cn".to_string());

    let kb = load_kb(&kb_dir);

    let state = Arc::new(AppState { kb });

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(routes::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3011".into());
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let addr = format!("{host}:{port}");
    tracing::info!("Curriculum Service starting on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
