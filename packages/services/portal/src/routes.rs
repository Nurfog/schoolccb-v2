use axum::{
    extract::{Path, Query, State},
    http::{
        StatusCode,
        header::{self, HeaderMap, HeaderValue},
    },
    response::{Html, Redirect},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{sync::Arc, time::Instant};
use uuid::Uuid;

use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/robots.txt", get(robots_txt))
        .route("/sitemap.xml", get(sitemap_xml))
        .route("/", get(index))
        .route("/features", get(features))
        .route("/pricing", get(pricing))
        .route("/about", get(about))
        .route("/faq", get(faq_page))
        .route("/contact", get(contact_page))
        .route("/blog", get(blog_index))
        .route("/blog/{slug}", get(blog_post))
        .route("/login", get(login_page))
        .route("/app", get(app_redirect))
        .route("/app/", get(app_redirect))
        .route("/app/{*path}", get(app_redirect))
        .route("/api/auth/login", post(proxy_login))
        .route("/api/auth/exchange", post(exchange_token))
        .route("/api/public/plans", get(public_plans))
        .route("/api/public/features", get(public_features))
        .route("/api/public/contact", post(contact_submit))
}

#[derive(Deserialize)]
struct LoginQuery {
    error: Option<String>,
}

async fn login_page(state: State<Arc<AppState>>, Query(q): Query<LoginQuery>) -> Result<Html<String>, StatusCode> {
    render(
        &state,
        "login.html",
        json!({"title": "Iniciar Sesión — SchoolCBB", "error": q.error, "frontend_url": state.config.frontend_url}),
    )
    .await
}

async fn app_redirect(state: State<Arc<AppState>>) -> Redirect {
    Redirect::to(&format!("{}/login", state.config.frontend_url))
}

async fn proxy_login(
    state: State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let upstream = format!("{}/api/auth/login", state.config.identity_url);
    match state.client.post(&upstream).json(&payload).send().await {
        Ok(resp) => {
            let status = resp.status();
            match resp.json::<Value>().await {
                Ok(mut data) => {
                    if status.is_success() {
                        if let Some(token) = data.get("token").and_then(|v| v.as_str()) {
                            let code = Uuid::new_v4().to_string();
                            let mut store = state.one_time_tokens.write().await;
                            // cleanup expired tokens
                            store.retain(|_, (_, t)| t.elapsed().as_secs() < 60);
                            store.insert(code.clone(), (token.to_string(), Instant::now()));
                            data["one_time_code"] = json!(code);
                        }
                    }
                    Json(data)
                }
                Err(_) => Json(json!({"error": "Error al procesar respuesta del servidor"})),
            }
        }
        Err(_) => Json(json!({"error": "Servicio de autenticación no disponible"})),
    }
}

async fn exchange_token(
    state: State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> (HeaderMap, Json<Value>) {
    let code = payload.get("code").and_then(|v| v.as_str()).unwrap_or("");
    if code.is_empty() {
        return (HeaderMap::new(), Json(json!({"error": "Código requerido"})));
    }
    let mut store = state.one_time_tokens.write().await;
    match store.remove(code) {
        Some((jwt, created)) => {
            if created.elapsed().as_secs() > 60 {
                return (HeaderMap::new(), Json(json!({"error": "Código expirado"})));
            }
            let mut headers = HeaderMap::new();
            if let Ok(cookie) = HeaderValue::from_str(&format!(
                "jwt_token={}; Path=/; SameSite=Lax; Max-Age=43200",
                jwt
            )) {
                headers.insert(header::SET_COOKIE, cookie);
            }
            if let Ok(marker) =
                HeaderValue::from_str("session_active=1; Path=/; SameSite=Lax; Max-Age=43200")
            {
                headers.append(header::SET_COOKIE, marker);
            }
            (headers, Json(json!({"token": jwt})))
        }
        None => (HeaderMap::new(), Json(json!({"error": "Código inválido o ya utilizado"}))),
    }
}

async fn render(
    state: &AppState,
    template: &str,
    ctx: Value,
) -> Result<Html<String>, StatusCode> {
    let tmpl = state.templates.get_template(template).map_err(|e| {
        tracing::error!("Template {template} not found: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let mut context = ctx;
    if let Some(obj) = context.as_object_mut() {
        let path = template.replace(".html", "").replace("index", "");
        obj.insert("canonical_url".to_string(), json!(format!("https://schoolccb.cl/{}", path)));
    }
    let rendered = tmpl.render(context).map_err(|e| {
        tracing::error!("Template {template} render error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Html(rendered))
}

async fn robots_txt() -> String {
    "User-agent: *\nAllow: /\nDisallow: /api/\nDisallow: /app/\nDisallow: /login\n\nSitemap: https://schoolccb.cl/sitemap.xml".to_string()
}

async fn sitemap_xml(state: State<Arc<AppState>>) -> ([(axum::http::header::HeaderName, &'static str); 1], String) {
    let mut urls = vec![
        r#"  <url><loc>https://schoolccb.cl/</loc><priority>1.0</priority></url>"#.to_string(),
        r#"  <url><loc>https://schoolccb.cl/features</loc><priority>0.8</priority></url>"#.to_string(),
        r#"  <url><loc>https://schoolccb.cl/pricing</loc><priority>0.8</priority></url>"#.to_string(),
        r#"  <url><loc>https://schoolccb.cl/about</loc><priority>0.6</priority></url>"#.to_string(),
        r#"  <url><loc>https://schoolccb.cl/contact</loc><priority>0.7</priority></url>"#.to_string(),
        r#"  <url><loc>https://schoolccb.cl/faq</loc><priority>0.6</priority></url>"#.to_string(),
        r#"  <url><loc>https://schoolccb.cl/blog</loc><priority>0.8</priority></url>"#.to_string(),
    ];
    for post in &state.blog_posts {
        urls.push(format!(
            r#"  <url><loc>https://schoolccb.cl/blog/{}</loc><priority>0.6</priority></url>"#,
            post.slug
        ));
    }
    (
        [(axum::http::header::CONTENT_TYPE, "application/xml")],
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
{body}
</urlset>"#,
            body = urls.join("\n")
        ),
    )
}

async fn index(state: State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    render(&state, "index.html", json!({
        "title": "SchoolCBB — Gestión Escolar Inteligente",
        "is_home": true
    })).await
}

async fn features(state: State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    let features = fetch_features(&state).await.unwrap_or_default();
    render(&state, "features.html", json!({"title": "Características — SchoolCBB", "features": features})).await
}

async fn fetch_features(state: &AppState) -> Result<Vec<Value>, ()> {
    let resp = state
        .client
        .get(format!("{}/api/public/features", state.config.identity_url))
        .send()
        .await
        .map_err(|e| tracing::error!("Failed to fetch features: {e}"))?;
    let body: Value = resp.json().await.map_err(|e| {
        tracing::error!("Failed to parse features response: {e}");
    })?;
    Ok(body["features"].as_array().cloned().unwrap_or_default())
}

async fn pricing(state: State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    let plans = fetch_plans(&state).await.unwrap_or_default();
    render(
        &state,
        "pricing.html",
        json!({"title": "Planes y Precios — SchoolCBB", "plans": plans}),
    )
    .await
}

async fn about(state: State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    render(&state, "about.html", json!({"title": "Sobre Nosotros — SchoolCBB"})).await
}

async fn faq_page(state: State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    render(&state, "faq.html", json!({"title": "Preguntas Frecuentes — SchoolCBB"})).await
}

async fn contact_page(state: State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    render(&state, "contact.html", json!({"title": "Contacto — SchoolCBB"})).await
}

async fn blog_index(state: State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    let posts: Vec<Value> = state
        .blog_posts
        .iter()
        .map(|p| {
            json!({
                "slug": p.slug,
                "title": p.title,
                "date": p.date.format("%d/%m/%Y").to_string(),
                "excerpt": p.excerpt,
                "author": p.author,
            })
        })
        .collect();
    render(
        &state,
        "blog/index.html",
        json!({"title": "Blog — SchoolCBB", "posts": posts}),
    )
    .await
}

async fn blog_post(
    state: State<Arc<AppState>>,
    Path(slug): axum::extract::Path<String>,
) -> Result<Html<String>, StatusCode> {
    let post = state.blog_posts.iter().find(|p| p.slug == slug);
    match post {
        Some(p) => {
            render(
                &state,
                "blog/post.html",
                json!({
                    "title": format!("{} — SchoolCBB", p.title),
                    "post": {
                        "title": p.title,
                        "date": p.date.format("%d/%m/%Y").to_string(),
                        "author": p.author,
                        "html": p.html,
                    }
                }),
            )
            .await
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn fetch_plans(state: &AppState) -> Result<Vec<Value>, ()> {
    let resp = state
        .client
        .get(format!("{}/api/public/plans", state.config.identity_url))
        .send()
        .await
        .map_err(|e| tracing::error!("Failed to fetch plans: {e}"))?;
    let body: Value = resp.json().await.map_err(|e| {
        tracing::error!("Failed to parse plans response: {e}");
    })?;
    Ok(body["plans"].as_array().cloned().unwrap_or_default())
}

async fn public_plans(state: State<Arc<AppState>>) -> Json<Value> {
    let plans = fetch_plans(&state).await.unwrap_or_default();
    Json(json!({"plans": plans}))
}

async fn public_features() -> Json<Value> {
    Json(json!({
        "features": [
            {"key": "dashboard", "name": "Dashboard", "description": "Panel de control con KPIs y gráficos en tiempo real"},
            {"key": "students", "name": "Gestión de Alumnos", "description": "Fichas completas con datos personales, académicos y de salud"},
            {"key": "courses", "name": "Cursos", "description": "Configuración de cursos, niveles y asignación de profesores"},
            {"key": "attendance", "name": "Asistencia", "description": "Registro de asistencia diaria por bloque con reportes"},
            {"key": "academic", "name": "Notas / Académico", "description": "Evaluaciones, calificaciones y libretas con Decreto 67"},
            {"key": "hr", "name": "Recursos Humanos", "description": "Fichas empleados, contratos, vacaciones y ausencias"},
            {"key": "finance", "name": "Finanzas", "description": "Gestión de cobros, pagos, becas y pasarela de pago"},
            {"key": "admission", "name": "Admisión CRM", "description": "Pipeline de postulantes con kanban y workflow automático"},
            {"key": "reports", "name": "Reportes", "description": "Certificados, concentraciones de notas y actas finales"},
            {"key": "communications", "name": "Comunicaciones", "description": "Mensajería interna, notificaciones y agenda escolar"},
            {"key": "sige", "name": "SIGE / MINEDUC", "description": "Exportación de datos según formato oficial del Ministerio"},
            {"key": "multi-school", "name": "Multi-colegio", "description": "Gestión centralizada de múltiples colegios desde una plataforma"},
            {"key": "api", "name": "API Pública", "description": "API REST para integración con sistemas externos"},
            {"key": "compliance", "name": "Cumplimiento Legal", "description": "LRE, Previred, Ley Karin y normativa chilena"},
        ]
    }))
}

async fn contact_submit(state: State<Arc<AppState>>, Json(payload): Json<Value>) -> Json<Value> {
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let email = payload.get("email").and_then(|v| v.as_str()).unwrap_or("");
    let company = payload.get("company").and_then(|v| v.as_str()).unwrap_or("");
    let message = payload.get("message").and_then(|v| v.as_str()).unwrap_or("");
    
    tracing::info!("Portal contact: {name} <{email}> ({company}): {message}");

    // Split name into first and last
    let mut parts = name.split_whitespace();
    let first_name = parts.next().unwrap_or("").to_string();
    let last_name = parts.collect::<Vec<&str>>().join(" ");

    let crm_payload = json!({
        "first_name": first_name,
        "last_name": if last_name.is_empty() { "-" } else { &last_name },
        "email": email,
        "company": company,
        "notes": message,
        "source": "web"
    });

    let crm_url = format!("{}/api/public/sales/prospects", state.config.crm_url);
    match state.client.post(&crm_url).json(&crm_payload).send().await {
        Ok(resp) if resp.status().is_success() => {
            Json(json!({"message": "Mensaje recibido. Te contactaremos pronto."}))
        }
        _ => {
            // Fallback: still log it even if CRM is down
            tracing::error!("Failed to create prospect in CRM for {}", email);
            Json(json!({"message": "Mensaje recibido. Te contactaremos pronto."}))
        }
    }
}
