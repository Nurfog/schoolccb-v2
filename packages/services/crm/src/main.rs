mod config;
mod email;
mod error;
mod models;
mod pdf;
mod routes;
mod signature;

use std::sync::Arc;

use axum::Router;
use schoolccb_common::auth::JwtSecret;
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use config::Config;
use email::Mailer;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub client: reqwest::Client,
    pub signature_config: Arc<signature::SignatureConfig>,
    pub mailer: Mailer,
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
    let client = reqwest::Client::new();

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    tracing::info!("CRM Service connected to database");

    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        tracing::warn!("SQLx migrations skipped: {e}");
    }

    let signature_config = Arc::new(signature::SignatureConfig::from_env());
    let mailer = email::Mailer::new(pool.clone(), config.clone());

    let state = AppState {
        pool: pool.clone(),
        config: config.clone(),
        client,
        signature_config,
        mailer,
    };

    let cors = if std::env::var("CORS_ENABLED").as_deref() == Ok("true") {
        let origins = std::env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000,http://localhost:8080".into());
        let mut layer = CorsLayer::new()
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any)
            .allow_credentials(true);
        for origin in origins.split(',') {
            if let Ok(val) = origin.trim().parse::<axum::http::HeaderValue>() {
                layer = layer.allow_origin(val);
            }
        }
        Some(layer)
    } else {
        None
    };

    let mut app = Router::new()
        .merge(routes::router())
        .layer(axum::middleware::from_fn(inject_jwt_secret))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    if let Some(cors) = cors {
        app = app.layer(cors);
    }

    let addr = config.addr();
    tracing::info!("CRM Service starting on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
