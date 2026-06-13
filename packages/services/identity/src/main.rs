mod admin;
mod client;
mod config;
mod error;
mod models;
mod routes;
mod scheduler;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
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

    tracing::info!("Identity Service connected to database");
    schoolccb_common::db_schema::run(&pool).await;

    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        tracing::warn!("SQLx migrations skipped: {e}");
    }

    models::seed_admin(&pool)
        .await
        .expect("Failed to seed admin user");
    tracing::info!("Admin user seeded (check .env for credentials)");

    models::seed_roles(&pool).await;
    models::seed_permission_definitions(&pool).await;
    tracing::info!("Roles and permissions seeded");

    models::seed_gerente_general(&pool).await;
    models::seed_license_plans(&pool).await;

    models::seed_default_school(&pool).await;

    let state = AppState {
        pool: pool.clone(),
        config: config.clone(),
    };

    scheduler::start(pool.clone()).await;

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
        .route("/health", get(|| async { "ok" }))
        .merge(routes::router())
        .merge(client::client_router())
        .merge(admin::admin_router())
        .layer(TraceLayer::new_for_http())
        .layer(axum::extract::Extension(schoolccb_common::auth::JwtSecret(config.jwt_secret.clone())))
        .with_state(state);

    if let Some(cors) = cors {
        app = app.layer(cors);
    }

    let addr = config.addr();
    tracing::info!("Identity Service starting on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
