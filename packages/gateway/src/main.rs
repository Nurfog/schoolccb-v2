mod graphql;
mod rate_limit;

use std::env;



use axum::{
    Router,
    extract::{
        Extension, Json, Request, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{any, get},
    body::Body,
};
use futures_util::{SinkExt, StreamExt};
use http_body_util::BodyExt;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use schoolccb_gateway::extract_jwt_from_cookie;

#[derive(Clone)]
struct AppState {
    client: reqwest::Client,
    identity_url: String,
    sis_url: String,
    academic_url: String,
    attendance_url: String,
    notifications_url: String,
    finance_url: String,
    reporting_url: String,
    portal_url: String,
    curriculum_url: String,
    crm_url: String,
    frontend_url: String,
    ollama_url: String,
    auth_rate_limiter: rate_limit::RateLimiter,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    dotenvy::dotenv().ok();

    let state = AppState {
        client: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest Client debe construirse"),

        identity_url: env::var("IDENTITY_URL").unwrap_or_else(|_| "http://localhost:3001".into()),
        sis_url: env::var("SIS_URL").unwrap_or_else(|_| "http://localhost:3002".into()),
        academic_url: env::var("ACADEMIC_URL").unwrap_or_else(|_| "http://localhost:3003".into()),
        attendance_url: env::var("ATTENDANCE_URL")
            .unwrap_or_else(|_| "http://localhost:3004".into()),
        notifications_url: env::var("NOTIFICATIONS_URL")
            .unwrap_or_else(|_| "http://localhost:3005".into()),
        frontend_url: env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:8080".into()),
        finance_url: env::var("FINANCE_URL").unwrap_or_else(|_| "http://localhost:3006".into()),
        reporting_url: env::var("REPORTING_URL").unwrap_or_else(|_| "http://localhost:3007".into()),
        portal_url: env::var("PORTAL_URL").unwrap_or_else(|_| "http://localhost:3010".into()),
        curriculum_url: env::var("CURRICULUM_URL").unwrap_or_else(|_| "http://localhost:3011".into()),
        crm_url: env::var("CRM_URL").unwrap_or_else(|_| "http://localhost:3012".into()),
        ollama_url: env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".into()),
        // Rate limit: 10 requests per minute for auth endpoints
        auth_rate_limiter: rate_limit::RateLimiter::new(10, std::time::Duration::from_secs(60)),
    };
    let frontend_origin = state
        .frontend_url
        .parse::<axum::http::HeaderValue>()
        .expect("FRONTEND_URL debe ser una URL válida");
    let cors = CorsLayer::new()
        .allow_origin(frontend_origin)
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::PUT, axum::http::Method::DELETE, axum::http::Method::OPTIONS])
        .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::AUTHORIZATION, axum::http::header::COOKIE, axum::http::header::ACCEPT])
        .allow_credentials(true);

    let schema = graphql::build_schema(&state.sis_url, &state.academic_url, state.client.clone());

    let app = Router::new()
        .route(
            "/health",
            get(|| async {
                (
                    StatusCode::OK,
                    axum::Json(serde_json::json!({"status": "ok", "service": "gateway"})),
                )
            }),
        )
        // ─── Shared Auth ───
        .route("/api/auth/exchange", any(proxy_portal))
        .route("/api/auth/exchange/{*path}", any(proxy_portal))
        .route("/api/auth/login", any(proxy_identity_ratelimited))
        .route("/api/auth/forgot-password", any(proxy_identity_ratelimited))
        .route("/api/auth/register", any(proxy_identity_ratelimited))
        .route("/api/auth", any(proxy_identity))
        .route("/api/auth/{*path}", any(proxy_identity))
        .route("/api/user", any(proxy_identity))
        .route("/api/user/{*path}", any(proxy_identity))
        .route("/api/roles", any(proxy_identity))
        .route("/api/roles/{*path}", any(proxy_identity))
        .route("/api/permissions", any(proxy_identity))
        .route("/api/permissions/{*path}", any(proxy_identity))
        .route("/api/config", any(proxy_identity))
        .route("/api/config/{*path}", any(proxy_identity))
        .route("/api/users", any(proxy_identity))
        .route("/api/users/{*path}", any(proxy_identity))
        // ─── B2B: Platform Admin (identity) ───
        .route("/b2b/admin", any(proxy_identity))
        .route("/b2b/admin/{*path}", any(proxy_identity))
        .route("/b2b/public", any(proxy_identity_ratelimited))
        .route("/b2b/public/{*path}", any(proxy_identity_ratelimited))
        .route("/b2b/client", any(proxy_identity))
        .route("/b2b/client/{*path}", any(proxy_identity))
        .route("/b2b/corporations", any(proxy_identity))
        .route("/b2b/corporations/{*path}", any(proxy_identity))
        .route("/b2b/schools", any(proxy_identity))
        .route("/b2b/schools/{*path}", any(proxy_identity))
        .route("/b2b/legal-representatives", any(proxy_identity))
        .route("/b2b/legal-representatives/{*path}", any(proxy_identity))
        // ─── B2B: CRM Sales (crm) ───
        .route("/b2b/sales", any(proxy_crm))
        .route("/b2b/sales/{*path}", any(proxy_crm))
        // ─── B2B: Corporation Dashboard (sis) ───
        .route("/b2b/corporation", any(proxy_sis))
        .route("/b2b/corporation/{*path}", any(proxy_sis))
        .route("/api/students", any(proxy_sis))
        .route("/api/students/{*path}", any(proxy_sis))
        .route("/api/courses", any(proxy_sis))
        .route("/api/courses/{*path}", any(proxy_sis))
        .route("/api/enrollments", any(proxy_sis))
        .route("/api/enrollments/{*path}", any(proxy_sis))
        .route("/api/dashboard", any(proxy_sis))
        .route("/api/dashboard/{*path}", any(proxy_sis))
        .route("/api/admission", any(proxy_sis))
        .route("/api/admission/{*path}", any(proxy_sis))
        .route("/api/hr", any(proxy_sis))
        .route("/api/hr/{*path}", any(proxy_sis))
        .route("/api/search", any(proxy_sis))
        .route("/api/search/{*path}", any(proxy_sis))
        .route("/api/grades", any(proxy_academic))
        .route("/api/grades/{*path}", any(proxy_academic))
        .route("/api/academic-years", any(proxy_academic))
        .route("/api/academic-years/{*path}", any(proxy_academic))
        .route("/api/academic/grade-levels", any(proxy_academic))
        .route("/api/academic/grade-levels/{*path}", any(proxy_academic))
        .route("/api/academic/audit-log", any(proxy_academic))
        .route("/api/academic/audit-log/{*path}", any(proxy_academic))
        .route("/api/attendance", any(proxy_attendance))
        .route("/api/attendance/{*path}", any(proxy_attendance))
        .route("/api/communications", any(proxy_notifications))
        .route("/api/communications/{*path}", any(proxy_notifications))
        .route("/api/notifications", any(proxy_notifications))
        .route("/api/notifications/{*path}", any(proxy_notifications))
        .route("/api/finance", any(proxy_finance))
        .route("/api/finance/{*path}", any(proxy_finance))
        .route("/api/reports", any(proxy_reporting))
        .route("/api/reports/{*path}", any(proxy_reporting))
        .route("/api/curriculum", any(proxy_curriculum))
        .route("/api/curriculum/{*path}", any(proxy_curriculum))
        .route("/api/ai/{*path}", any(proxy_ai))
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .layer(Extension(schema))
        .route("/ws", any(ws_proxy))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".into());
    let addr = format!("{host}:{port}");
    tracing::info!("Gateway starting on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind TcpListener to address — is the port already in use?");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install ctrl_c handler");
            tracing::info!("Shutting down gracefully...");
        })
        .await
        .expect("axum::serve failed — the server encountered a fatal error during operation");
}

async fn ws_proxy(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    let upstream = state.notifications_url.clone();
    ws.on_upgrade(move |socket| handle_ws_proxy(socket, upstream))
}

async fn graphql_playground() -> impl IntoResponse {
    axum::response::Html(
        r#"<!DOCTYPE html>
<html><head><title>SchoolCBB GraphQL</title>
<script src="https://cdn.jsdelivr.net/npm/graphql-playground-react/build/static/js/middleware.js"></script>
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/graphql-playground-react/build/static/css/index.css"/>
</head><body><div id="root"></div>
<script>window.addEventListener('load',function(){GraphQLPlayground.init(document.getElementById('root'),{endpoint:'/graphql'})})</script>
</body></html>"#,
    )
}

async fn graphql_handler(
    Extension(schema): Extension<graphql::AppSchema>,
    headers: axum::http::HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let query = body.get("query").and_then(|v| v.as_str()).unwrap_or("");
    let vars = body.get("variables");

    let mut req = async_graphql::Request::new(query);
    let operation_name = body.get("operationName").and_then(|v| v.as_str());
    if let Some(name) = operation_name {
        req = req.operation_name(name);
    }
    if let Some(v) = vars {
        if let Some(obj) = v.as_object() {
            let vars = async_graphql::Variables::from_json(serde_json::Value::Object(obj.clone()));
            req = req.variables(vars);
        }
    }
    if let Some(auth) = headers.get("Authorization").and_then(|v| v.to_str().ok()) {
        req = req.data(auth.to_string());
    }

    let resp = schema.execute(req).await;
    let mut output = serde_json::Map::new();
    output.insert("data".into(), serde_json::to_value(&resp.data).unwrap_or_default());
    if !resp.errors.is_empty() {
        let errors: Vec<serde_json::Value> = resp
            .errors
            .iter()
            .map(|e| serde_json::json!({"message": e.message}))
            .collect();
        output.insert("errors".into(), serde_json::Value::Array(errors));
    }
    Json(serde_json::Value::Object(output))
}

async fn handle_ws_proxy(client_ws: WebSocket, upstream_url: String) {
    let ws_url = upstream_url
        .replace("http://", "ws://")
        .replace("https://", "wss://");
    let ws_url = format!("{}/ws", ws_url.trim_end_matches('/'));

    match tokio_tungstenite::connect_async(&ws_url).await {
        Ok((upstream_ws, _)) => {
            let (mut client_sender, mut client_receiver) = client_ws.split();
            let (mut upstream_sender, mut upstream_receiver) = upstream_ws.split();

            let c2u = tokio::spawn(async move {
                while let Some(Ok(msg)) = client_receiver.next().await {
                    let upstream_msg = match msg {
                        Message::Text(t) => tungstenite::Message::Text(t.to_string()),
                        Message::Binary(b) => tungstenite::Message::Binary(b.to_vec()),
                        Message::Ping(p) => tungstenite::Message::Ping(p.to_vec()),
                        Message::Pong(p) => tungstenite::Message::Pong(p.to_vec()),
                        Message::Close(c) => {
                            let ws_close = c.map(|f| {
                                let code: u16 = f.code.into();
                                tungstenite::protocol::CloseFrame {
                                    code: tungstenite::protocol::frame::coding::CloseCode::from(code),
                                    reason: std::borrow::Cow::Owned(f.reason.to_string()),
                                }
                            });
                            let _ = upstream_sender.send(tungstenite::Message::Close(ws_close)).await;
                            break;
                        }
                    };
                    if upstream_sender.send(upstream_msg).await.is_err() {
                        break;
                    }
                }
            });

            let u2c = tokio::spawn(async move {
                while let Some(Ok(msg)) = upstream_receiver.next().await {
                    let client_msg = match msg {
                        tungstenite::Message::Text(t) => Message::Text(t.into()),
                        tungstenite::Message::Binary(b) => Message::Binary(axum::body::Bytes::from(b)),
                        tungstenite::Message::Ping(p) => Message::Ping(axum::body::Bytes::from(p)),
                        tungstenite::Message::Pong(p) => Message::Pong(axum::body::Bytes::from(p)),
                        tungstenite::Message::Close(c) => {
                            let axum_close = c.map(|f| axum::extract::ws::CloseFrame {
                                code: f.code.into(),
                                reason: f.reason.to_string().into(),
                            });
                            let _ = client_sender.send(Message::Close(axum_close)).await;
                            break;
                        }
                        _ => continue,
                    };
                    if client_sender.send(client_msg).await.is_err() {
                        break;
                    }
                }
            });

            tokio::select! {
                _ = c2u => {},
                _ = u2c => {},
            }
        }
        Err(e) => {
            tracing::error!("WebSocket proxy: failed to connect to upstream {ws_url}: {e}");
        }
    }
}

macro_rules! proxy_handler {
    ($name:ident, $service:expr) => {
        async fn $name(state: State<AppState>, req: Request) -> Response {
            proxy_request(&state, $service, req).await
        }
    };
}

proxy_handler!(proxy_identity, "identity");
proxy_handler!(proxy_sis, "sis");
proxy_handler!(proxy_academic, "academic");
proxy_handler!(proxy_attendance, "attendance");
proxy_handler!(proxy_notifications, "notifications");
proxy_handler!(proxy_finance, "finance");
proxy_handler!(proxy_reporting, "reporting");
proxy_handler!(proxy_portal, "portal");
proxy_handler!(proxy_curriculum, "curriculum");
proxy_handler!(proxy_crm, "crm");

async fn proxy_identity_ratelimited(
    State(state): State<AppState>,
    req: Request,
) -> Response {
    // Extract client IP from X-Forwarded-For or connect info
    let ip = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .and_then(|v| v.trim().parse::<std::net::IpAddr>().ok())
        .or_else(|| {
            req.extensions()
                .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
                .map(|ci| ci.0.ip())
        })
        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));

    if !state.auth_rate_limiter.check(ip).await {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error": "Demasiadas solicitudes. Intenta nuevamente en un minuto."
            })),
        )
            .into_response();
    }

    proxy_request(&state, "identity", req).await
}

async fn proxy_ai(State(state): State<AppState>, req: Request) -> Response {
    let base_url = &state.ollama_url;
    let path = req.uri().path().strip_prefix("/api/ai").unwrap_or("");
    let query = req
        .uri()
        .query()
        .map(|q| format!("?{q}"))
        .unwrap_or_default();
    let upstream_url = format!("{base_url}{path}{query}");

    let method = req.method().clone();

    let mut req_headers: Vec<(String, String)> = Vec::new();
    for (k, v) in req.headers().iter() {
        let key = k.to_string();
        if key == "host" {
            continue;
        }
        req_headers.push((key, v.to_str().unwrap_or("").to_string()));
    }

    let body_bytes = req
        .into_body()
        .collect()
        .await
        .unwrap_or_default()
        .to_bytes();

    let mut upstream_req = state
        .client
        .request(method, &upstream_url)
        .body(body_bytes);

    for (key, value) in &req_headers {
        upstream_req = upstream_req.header(key.as_str(), value.as_str());
    }

    match upstream_req.send().await {
        Ok(resp) => {
            let status = resp.status();
            let resp_headers = resp.headers().clone();
            let stream = resp.bytes_stream().map(|r| r.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>));
            let body = Body::from_stream(stream);
            let mut response = Response::new(body);
            *response.status_mut() = status;
            for (key, value) in resp_headers.iter() {
                if key != "host" && key != "transfer-encoding" {
                    response.headers_mut().insert(key, value.clone());
                }
            }
            response
        }
        Err(e) => {
            tracing::error!("AI proxy error: {e}");
            (StatusCode::BAD_GATEWAY, format!("AI service unavailable: {e}")).into_response()
        }
    }
}

async fn proxy_request(state: &AppState, service: &str, req: Request) -> Response {
    let base_url = match service {
        "identity" => &state.identity_url,
        "sis" => &state.sis_url,
        "academic" => &state.academic_url,
        "attendance" => &state.attendance_url,
        "notifications" => &state.notifications_url,
        "finance" => &state.finance_url,
        "reporting" => &state.reporting_url,
        "portal" => &state.portal_url,
        "curriculum" => &state.curriculum_url,
        "crm" => &state.crm_url,
        _ => return (StatusCode::BAD_REQUEST, "Unknown service").into_response(),
    };

    let path = req.uri().path();
    let query = req
        .uri()
        .query()
        .map(|q| format!("?{q}"))
        .unwrap_or_default();
    let upstream_url = format!("{base_url}{path}{query}");

    let method = req.method().clone();
    let mut has_auth = false;
    let mut req_headers: Vec<(String, String)> = Vec::new();

    for (k, v) in req.headers().iter() {
        let key = k.to_string();
        if key == "host" {
            continue;
        }
        if key.eq_ignore_ascii_case("authorization") {
            has_auth = true;
        }
        req_headers.push((key, v.to_str().unwrap_or("").to_string()));
    }

    // Fallback: si no hay Authorization header pero hay cookie jwt_token, agregarlo
    if !has_auth {
        if let Some(jwt) = extract_jwt_from_cookie(req.headers()) {
            req_headers.push(("Authorization".into(), format!("Bearer {jwt}")));
        }
    }

    let body_bytes = req
        .into_body()
        .collect()
        .await
        .unwrap_or_default()
        .to_bytes();

    let mut upstream_req = state
        .client
        .request(method, &upstream_url)
        .body(body_bytes);

    for (key, value) in &req_headers {
        upstream_req = upstream_req.header(key.as_str(), value.as_str());
    }

    match upstream_req.send().await {
        Ok(resp) => {
            let status = resp.status();
            let resp_headers = resp.headers().clone();
            let body = resp.bytes().await.unwrap_or_default();
            let mut response = Response::new(axum::body::Body::from(body));
            *response.status_mut() = status;
            for (key, value) in resp_headers.iter() {
                if key != "host" && key != "transfer-encoding" {
                    if key == "set-cookie" {
                        response.headers_mut().append(key, value.clone());
                    } else {
                        response.headers_mut().insert(key, value.clone());
                    }
                }
            }
            response
        }
        Err(e) => {
            tracing::error!("Proxy error to {service}: {e}");
            (
                StatusCode::BAD_GATEWAY,
                format!("Service {service} unavailable"),
            )
                .into_response()
        }
    }
}
