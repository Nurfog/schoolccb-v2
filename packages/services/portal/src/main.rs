mod blog;
mod config;
mod routes;

use axum::routing::get;
use axum::Router;
use minijinja::Environment;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

pub struct AppState {
    pub config: Arc<config::Config>,
    pub templates: Environment<'static>,
    pub client: reqwest::Client,
    pub one_time_tokens: RwLock<HashMap<String, (String, std::time::Instant)>>,
    pub blog_posts: Vec<blog::BlogPost>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    dotenvy::dotenv().ok();
    let config = Arc::new(config::Config::from_env());

    let blog_posts = blog::load_blog_posts();
    tracing::info!("Loaded {} blog posts", blog_posts.len());

    let mut templates = Environment::new();
    templates
        .add_template("base.html", include_str!("../templates/base.html"))
        .expect("base template");
    templates
        .add_template("index.html", include_str!("../templates/index.html"))
        .expect("index template");
    templates
        .add_template("features.html", include_str!("../templates/features.html"))
        .expect("features template");
    templates
        .add_template("pricing.html", include_str!("../templates/pricing.html"))
        .expect("pricing template");
    templates
        .add_template("about.html", include_str!("../templates/about.html"))
        .expect("about template");
    templates
        .add_template("contact.html", include_str!("../templates/contact.html"))
        .expect("contact template");
    templates
        .add_template("login.html", include_str!("../templates/login.html"))
        .expect("login template");
    templates
        .add_template("blog/index.html", include_str!("../templates/blog/index.html"))
        .expect("blog/index template");
    templates
        .add_template("blog/post.html", include_str!("../templates/blog/post.html"))
        .expect("blog/post template");

    let state = Arc::new(AppState {
        config,
        templates,
        client: reqwest::Client::new(),
        one_time_tokens: RwLock::new(HashMap::new()),
        blog_posts,
    });

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(routes::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = config::Config::from_env().addr();
    tracing::info!("Portal Service starting on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
