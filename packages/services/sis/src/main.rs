mod admission;
mod config;
mod error;
mod extras;
mod hr;
mod hr_extended;
mod routes;
mod search;
mod workflow;

use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use schoolccb_common::auth::JwtSecret;
use schoolccb_common::event_bus::BroadcastBus;
use sqlx::PgPool;
use tokio::sync::broadcast;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use config::Config;
use workflow::WorkflowEngine;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub workflow: Arc<WorkflowEngine>,
    pub event_bus: Arc<BroadcastBus>,
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

    tracing::info!("SIS Service connected to database");
    schoolccb_common::db_schema::run(&pool).await;
    admission::seed_pipeline_stages(&pool).await;
    for year in [2025, 2026, 2027] {
        routes::academic_calendar::seed_holidays(&pool, year).await;
    }

    let event_bus = Arc::new(BroadcastBus::new(256));

    let finance_grpc = std::env::var("FINANCE_GRPC_URL").ok();
    let workflow = Arc::new(
        WorkflowEngine::with_grpc(pool.clone(), finance_grpc, "sis".into())
            .with_event_bus(event_bus.clone()),
    );
    let bus_rx = event_bus.subscribe();
    let _pool_for_bus = pool.clone();
    tokio::spawn(async move {
        let mut rx = bus_rx;
        loop {
            match rx.recv().await {
                Ok(event) => {
                    tracing::info!(
                        "[EventBus] {} from {}: {:?}",
                        event.event_type,
                        event.source,
                        event.payload
                    );
                    let _ = sqlx::query(
                        "INSERT INTO event_log (id, event_type, payload) VALUES ($1, $2, $3)",
                    )
                    .bind(uuid::Uuid::new_v4())
                    .bind(format!("bus.{}", event.event_type))
                    .bind(&event.payload)
                    .execute(&_pool_for_bus)
                    .await;
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("EventBus lagged by {} messages", n);
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    let state = AppState {
        pool,
        config: config.clone(),
        workflow,
        event_bus,
    };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(routes::router())
        .merge(admission::router())
        .merge(hr::router())
        .merge(hr_extended::router())
        .merge(extras::router())
        .merge(search::router())
        .layer(axum::middleware::from_fn(inject_jwt_secret))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = config.addr();
    tracing::info!("SIS Service starting on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
