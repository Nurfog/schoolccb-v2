mod config;
mod error;
mod grpc;
mod routes;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use schoolccb_common::auth::JwtSecret;
use sqlx::PgPool;
use tonic::transport::Server;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
}

async fn inject_jwt_secret(
    mut req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let secret = req
        .extensions()
        .get::<AppState>()
        .map(|s| s.config.jwt_secret.clone());
    if let Some(secret) = secret {
        req.extensions_mut().insert(JwtSecret(secret));
    }
    next.run(req).await
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    dotenvy::dotenv().ok();
    let config = Arc::new(Config::from_env());

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    tracing::info!("Academic Service connected to database");
    schoolccb_common::db_schema::run(&pool).await;
    seed_subjects(&pool).await;
    routes::grade_levels::seed_grade_levels(&pool).await;

    let state = AppState {
        pool,
        config: config.clone(),
    };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(routes::router())
        .layer(axum::middleware::from_fn(inject_jwt_secret))
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    let rest_addr = config.addr();
    let grpc_addr = format!("{}:{}", config.host, config.port + 100);
    let pool_clone = state.pool.clone();

    tokio::spawn(async move {
        tracing::info!("Academic gRPC starting on {grpc_addr}");
        let grpc = grpc::AcademicGrpc { pool: pool_clone };
        Server::builder()
            .add_service(schoolccb_proto::academic_service_server::AcademicServiceServer::new(grpc))
            .serve(grpc_addr.parse().expect("dirección gRPC inválida"))
            .await
            .unwrap();
    });

    tracing::info!("Academic REST starting on {rest_addr}");
    let listener = tokio::net::TcpListener::bind(&rest_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

struct SubjectSeed<'a> {
    code: &'a str,
    name: &'a str,
    hours: Vec<(&'a str, Option<&'a str>, i32)>,
}

async fn seed_subjects(pool: &sqlx::PgPool) {
    let subjects = vec![
        // ─── Lenguaje ──────────────────────────────────────────────
        SubjectSeed {
            code: "LEN01",
            name: "Lenguaje y Comunicación",
            hours: vec![
                ("1° Básico", None, 8),
                ("2° Básico", None, 8),
                ("3° Básico", None, 8),
                ("4° Básico", None, 8),
                ("5° Básico", None, 6),
                ("6° Básico", None, 6),
            ],
        },
        SubjectSeed {
            code: "LLE07",
            name: "Lengua y Literatura",
            hours: vec![
                ("7° Básico", None, 6),
                ("8° Básico", None, 6),
                ("1° Medio", None, 6),
                ("2° Medio", None, 6),
                ("3° Medio", Some("HC"), 4),
                ("4° Medio", Some("HC"), 4),
                ("3° Medio", Some("TP"), 3),
                ("4° Medio", Some("TP"), 3),
                ("3° Medio", Some("Artístico"), 4),
                ("4° Medio", Some("Artístico"), 4),
            ],
        },
        // ─── Matemática ────────────────────────────────────────────
        SubjectSeed {
            code: "MAT01",
            name: "Matemática",
            hours: vec![
                ("1° Básico", None, 8),
                ("2° Básico", None, 8),
                ("3° Básico", None, 8),
                ("4° Básico", None, 8),
                ("5° Básico", None, 6),
                ("6° Básico", None, 6),
                ("7° Básico", None, 6),
                ("8° Básico", None, 6),
                ("1° Medio", None, 6),
                ("2° Medio", None, 6),
                ("3° Medio", Some("HC"), 5),
                ("4° Medio", Some("HC"), 5),
                ("3° Medio", Some("TP"), 4),
                ("4° Medio", Some("TP"), 4),
                ("3° Medio", Some("Artístico"), 4),
                ("4° Medio", Some("Artístico"), 4),
            ],
        },
        // ─── Ciencias ──────────────────────────────────────────────
        SubjectSeed {
            code: "CIE01",
            name: "Ciencias Naturales",
            hours: vec![
                ("1° Básico", None, 4),
                ("2° Básico", None, 4),
                ("3° Básico", None, 4),
                ("4° Básico", None, 4),
                ("5° Básico", None, 5),
                ("6° Básico", None, 5),
                ("7° Básico", None, 5),
                ("8° Básico", None, 5),
                ("1° Medio", None, 4),
                ("2° Medio", None, 4),
            ],
        },
        SubjectSeed {
            code: "FIS01",
            name: "Física",
            hours: vec![
                ("3° Medio", Some("HC"), 5),
                ("3° Medio", Some("TP"), 3),
                ("3° Medio", Some("Artístico"), 3),
            ],
        },
        SubjectSeed {
            code: "QUI01",
            name: "Química",
            hours: vec![
                ("3° Medio", Some("HC"), 5),
                ("3° Medio", Some("TP"), 3),
                ("3° Medio", Some("Artístico"), 3),
            ],
        },
        SubjectSeed {
            code: "BIO01",
            name: "Biología Celular y Molecular",
            hours: vec![("3° Medio", Some("HC"), 5)],
        },
        SubjectSeed {
            code: "CSA01",
            name: "Ciencias de la Salud",
            hours: vec![("3° Medio", Some("HC"), 3)],
        },
        // ─── Historia ──────────────────────────────────────────────
        SubjectSeed {
            code: "HIS01",
            name: "Historia, Geografía y Cs. Sociales",
            hours: vec![
                ("1° Básico", None, 4),
                ("2° Básico", None, 4),
                ("3° Básico", None, 4),
                ("4° Básico", None, 4),
                ("5° Básico", None, 4),
                ("6° Básico", None, 4),
                ("7° Básico", None, 5),
                ("8° Básico", None, 5),
                ("1° Medio", None, 5),
                ("2° Medio", None, 5),
            ],
        },
        SubjectSeed {
            code: "ECO01",
            name: "Economía y Sociedad",
            hours: vec![("3° Medio", Some("HC"), 4), ("3° Medio", Some("TP"), 2)],
        },
        // ─── Inglés ────────────────────────────────────────────────
        SubjectSeed {
            code: "ING01",
            name: "Inglés",
            hours: vec![
                ("1° Básico", None, 2),
                ("2° Básico", None, 2),
                ("3° Básico", None, 2),
                ("4° Básico", None, 2),
                ("5° Básico", None, 3),
                ("6° Básico", None, 3),
                ("7° Básico", None, 3),
                ("8° Básico", None, 3),
                ("1° Medio", None, 4),
                ("2° Medio", None, 4),
                ("3° Medio", Some("HC"), 4),
                ("4° Medio", Some("HC"), 4),
                ("3° Medio", Some("TP"), 3),
                ("4° Medio", Some("TP"), 3),
                ("3° Medio", Some("Artístico"), 3),
                ("4° Medio", Some("Artístico"), 3),
            ],
        },
        // ─── Artes ─────────────────────────────────────────────────
        SubjectSeed {
            code: "ART01",
            name: "Artes Visuales",
            hours: vec![
                ("1° Básico", None, 2),
                ("2° Básico", None, 2),
                ("3° Básico", None, 2),
                ("4° Básico", None, 2),
                ("5° Básico", None, 2),
                ("6° Básico", None, 2),
                ("7° Básico", None, 2),
                ("8° Básico", None, 2),
                ("1° Medio", None, 2),
                ("2° Medio", None, 2),
                ("3° Medio", Some("Artístico"), 6),
            ],
        },
        SubjectSeed {
            code: "MUS01",
            name: "Música",
            hours: vec![
                ("1° Básico", None, 2),
                ("2° Básico", None, 2),
                ("3° Básico", None, 2),
                ("4° Básico", None, 2),
                ("5° Básico", None, 2),
                ("6° Básico", None, 2),
                ("7° Básico", None, 2),
                ("8° Básico", None, 2),
                ("1° Medio", None, 2),
            ],
        },
        SubjectSeed {
            code: "AVI01",
            name: "Artes Visuales Audiovisuales",
            hours: vec![
                ("3° Medio", Some("HC"), 4),
                ("3° Medio", Some("Artístico"), 6),
            ],
        },
        SubjectSeed {
            code: "DAN01",
            name: "Danza",
            hours: vec![("3° Medio", Some("Artístico"), 6)],
        },
        SubjectSeed {
            code: "TEA01",
            name: "Teatro",
            hours: vec![
                ("3° Medio", Some("HC"), 4),
                ("3° Medio", Some("Artístico"), 6),
            ],
        },
        SubjectSeed {
            code: "ITE01",
            name: "Interpretación y Creación en Teatro",
            hours: vec![("3° Medio", Some("HC"), 4)],
        },
        // ─── Educación Física ──────────────────────────────────────
        SubjectSeed {
            code: "EFI01",
            name: "Educación Física y Salud",
            hours: vec![
                ("1° Básico", None, 3),
                ("2° Básico", None, 3),
                ("3° Básico", None, 3),
                ("4° Básico", None, 3),
                ("5° Básico", None, 3),
                ("6° Básico", None, 3),
                ("7° Básico", None, 3),
                ("8° Básico", None, 3),
                ("1° Medio", None, 2),
                ("2° Medio", None, 2),
                ("3° Medio", Some("HC"), 2),
                ("4° Medio", Some("HC"), 2),
                ("3° Medio", Some("TP"), 2),
                ("4° Medio", Some("TP"), 2),
                ("3° Medio", Some("Artístico"), 2),
                ("4° Medio", Some("Artístico"), 2),
            ],
        },
        // ─── Tecnología ────────────────────────────────────────────
        SubjectSeed {
            code: "TEC01",
            name: "Tecnología",
            hours: vec![
                ("1° Básico", None, 1),
                ("2° Básico", None, 1),
                ("3° Básico", None, 1),
                ("4° Básico", None, 1),
                ("5° Básico", None, 1),
                ("6° Básico", None, 1),
                ("7° Básico", None, 2),
                ("8° Básico", None, 2),
                ("1° Medio", None, 2),
            ],
        },
        // ─── Religión ──────────────────────────────────────────────
        SubjectSeed {
            code: "REL01",
            name: "Religión",
            hours: vec![
                ("1° Básico", None, 2),
                ("2° Básico", None, 2),
                ("3° Básico", None, 2),
                ("4° Básico", None, 2),
                ("5° Básico", None, 2),
                ("6° Básico", None, 2),
                ("7° Básico", None, 2),
            ],
        },
        // ─── Orientación ───────────────────────────────────────────
        SubjectSeed {
            code: "ORI01",
            name: "Orientación",
            hours: vec![
                ("1° Básico", None, 1),
                ("2° Básico", None, 1),
                ("3° Básico", None, 1),
                ("4° Básico", None, 1),
                ("5° Básico", None, 1),
                ("6° Básico", None, 1),
                ("7° Básico", None, 1),
                ("8° Básico", None, 1),
                ("1° Medio", None, 1),
                ("2° Medio", None, 1),
            ],
        },
        // ─── Filosofía ─────────────────────────────────────────────
        SubjectSeed {
            code: "FIL01",
            name: "Filosofía",
            hours: vec![
                ("3° Medio", Some("HC"), 4),
                ("4° Medio", Some("HC"), 4),
                ("3° Medio", Some("Artístico"), 3),
            ],
        },
        // ─── Ciudadanía ────────────────────────────────────────────
        SubjectSeed {
            code: "CIU01",
            name: "Educación Ciudadana",
            hours: vec![
                ("3° Medio", Some("HC"), 3),
                ("4° Medio", Some("HC"), 3),
                ("3° Medio", Some("TP"), 2),
            ],
        },
        // ─── Matemática Avanzada (HC) ──────────────────────────────
        SubjectSeed {
            code: "LDI01",
            name: "Límites, Derivadas e Integrales",
            hours: vec![("3° Medio", Some("HC"), 4)],
        },
        // ─── Formación Diferenciada TP ─────────────────────────────
        SubjectSeed {
            code: "TPG01",
            name: "Formación Diferenciada TP",
            hours: vec![("3° Medio", Some("TP"), 18), ("4° Medio", Some("TP"), 20)],
        },
        // ─── Formación Diferenciada Artístico ──────────────────────
        SubjectSeed {
            code: "FAR01",
            name: "Formación Diferenciada Artístico",
            hours: vec![
                ("3° Medio", Some("Artístico"), 12),
                ("4° Medio", Some("Artístico"), 14),
            ],
        },
        // ─── Parvularia (Nivel Transición) ──────────────────────────
        SubjectSeed {
            code: "LEN-PRV",
            name: "Lenguaje Verbal",
            hours: vec![("Nivel Transición", None, 6)],
        },
        SubjectSeed {
            code: "MAT-PRV",
            name: "Pensamiento Matemático",
            hours: vec![("Nivel Transición", None, 4)],
        },
        SubjectSeed {
            code: "CIE-PRV",
            name: "Exploración del Entorno",
            hours: vec![("Nivel Transición", None, 3)],
        },
        SubjectSeed {
            code: "SOC-PRV",
            name: "Identidad y Convivencia",
            hours: vec![("Nivel Transición", None, 3)],
        },
        SubjectSeed {
            code: "ART-PRV",
            name: "Expresión Artística",
            hours: vec![("Nivel Transición", None, 2)],
        },
        SubjectSeed {
            code: "EFI-PRV",
            name: "Corporeidad y Movimiento",
            hours: vec![("Nivel Transición", None, 3)],
        },
        // ─── Lengua Indígena ────────────────────────────────────────
        SubjectSeed {
            code: "LIN01",
            name: "Lengua Indígena (Mapuzugun)",
            hours: vec![
                ("1° Básico", None, 2),
                ("2° Básico", None, 2),
                ("3° Básico", None, 2),
                ("4° Básico", None, 2),
                ("5° Básico", None, 2),
                ("6° Básico", None, 2),
            ],
        },
        // ─── Inglés Propuesta (1°-6° Básico) ────────────────────────
        SubjectSeed {
            code: "ING-P01",
            name: "Inglés (Propuesta 1° Básico)",
            hours: vec![("1° Básico", None, 2)],
        },
        SubjectSeed {
            code: "ING-P02",
            name: "Inglés (Propuesta 2° Básico)",
            hours: vec![("2° Básico", None, 2)],
        },
        SubjectSeed {
            code: "ING-P03",
            name: "Inglés (Propuesta 3° Básico)",
            hours: vec![("3° Básico", None, 3)],
        },
        SubjectSeed {
            code: "ING-P04",
            name: "Inglés (Propuesta 4° Básico)",
            hours: vec![("4° Básico", None, 3)],
        },
        SubjectSeed {
            code: "ING-P05",
            name: "Inglés (Propuesta 5° Básico)",
            hours: vec![("5° Básico", None, 3)],
        },
        SubjectSeed {
            code: "ING-P06",
            name: "Inglés (Propuesta 6° Básico)",
            hours: vec![("6° Básico", None, 3)],
        },
    ];

    for s in &subjects {
        let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM subjects WHERE code = $1")
            .bind(s.code)
            .fetch_one(pool)
            .await
            .unwrap_or((0,));

        if exists.0 == 0 {
            let id = uuid::Uuid::new_v4();
            let result = sqlx::query(
                "INSERT INTO subjects (id, code, name, level, hours_per_week, active)
                 VALUES ($1, $2, $3, NULL, 0, true)",
            )
            .bind(id)
            .bind(s.code)
            .bind(s.name)
            .execute(pool)
            .await;

            match result {
                Ok(_) => {
                    tracing::info!("Seeded subject: {} - {}", s.code, s.name);
                    // Seed hours per level/plan
                    for (level, plan, hours) in &s.hours {
                        let level_key = match plan {
                            Some(p) => format!("{} {}", level, p),
                            None => level.to_string(),
                        };
                        sqlx::query(
                            r#"INSERT INTO subject_hours (id, subject_id, level, hours_per_week)
                               VALUES ($1, $2, $3, $4)
                               ON CONFLICT (subject_id, level) DO NOTHING"#,
                        )
                        .bind(uuid::Uuid::new_v4())
                        .bind(id)
                        .bind(&level_key)
                        .bind(hours)
                        .execute(pool)
                        .await
                        .unwrap_or_else(|_| {
                            tracing::warn!("Could not seed hours for {} at {}", s.code, level);
                            Default::default()
                        });
                    }
                }
                Err(e) => {
                    tracing::warn!("Could not seed subject {}: {e}", s.code);
                }
            }
        }
    }
}
