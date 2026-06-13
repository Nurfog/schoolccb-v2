mod config;
mod email;
mod error;
mod push;
mod ws;

use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use schoolccb_common::auth::JwtSecret;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub ws_hub: Arc<ws::hub::WsHub>,
    pub mailer: email::Mailer,
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

    tracing::info!("Notifications Service connected to database");
    schoolccb_common::db_schema::run(&pool).await;

    let mailer = email::Mailer::new(
        pool.clone(),
        config.smtp_host.clone(),
        config.smtp_port,
        config.smtp_user.clone(),
        config.smtp_pass.clone(),
        config.from_address.clone(),
        config.from_name.clone(),
    );

    let ws_hub = Arc::new(ws::hub::WsHub::new());

    // Background email queue processor
    let mailer_clone = mailer.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            mailer_clone.process_email_queue().await;
        }
    });

    let state = AppState {
        pool,
        config: config.clone(),
        ws_hub,
        mailer,
    };

    let addr = config.addr();

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(ws::routes::router())
        .merge(push::router())
        .layer(axum::middleware::from_fn(inject_jwt_secret))
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    tracing::info!("Notifications Service starting on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
