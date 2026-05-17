use dioxus::prelude::*;

use crate::api::client;
use crate::seo::use_page_title;

#[component]
pub fn DashboardMosaicosPage() -> Element {
    use_page_title("Dashboard");
    let summary = use_resource(|| async move { client::fetch_dashboard_summary().await });
    let attendance = use_resource(|| async move { client::fetch_attendance_today().await });
    let alerts = use_resource(|| async move { client::fetch_student_alerts().await });
    let agenda = use_resource(|| async move { client::fetch_agenda().await });

    rsx! {
        div { class: "page-header",
            h1 { "Dashboard" }
            p { "Panorama general del colegio" }
        }
        div { class: "mosaicos-grid",
            match summary() {
                Some(Ok(data)) => {
                    let total = data["total_students"].as_i64().unwrap_or(0);
                    let enrolled = data["total_enrolled"].as_i64().unwrap_or(0);
                    let teachers = data["total_teachers"].as_i64().unwrap_or(0);
                    rsx! {
                        Mosaico { title: "Alumnos", value: "{total}", icon: "users", color: "#1a2b3c" }
                        Mosaico { title: "Matriculados", value: "{enrolled}", icon: "check", color: "#22c55e" }
                        Mosaico { title: "Docentes", value: "{teachers}", icon: "book", color: "#3b82f6" }
                    }
                }
                Some(Err(e)) => rsx! { div { class: "alert alert-error", "Error: {e}" } },
                None => rsx! { div { class: "loading-spinner", "Cargando..." } },
            }
            match attendance() {
                Some(Ok(data)) => {
                    let pct = data["percentage"].as_f64().unwrap_or(0.0);
                    rsx! {
                        Mosaico { title: "Asistencia Hoy", value: "{pct:.1}%", icon: "📊", color: "#f59e0b" }
                    }
                }
                Some(Err(e)) => rsx! { div { class: "alert alert-error", "Error: {e}" } },
                None => rsx! { div { class: "loading-spinner", "Cargando..." } },
            }
            match alerts() {
                Some(Ok(data)) => {
                    let count = data["alerts"].as_array().map(|a| a.len()).unwrap_or(0);
                    rsx! {
                        Mosaico { title: "Alertas", value: "{count}", icon: "⚠️", color: "#ef4444" }
                    }
                }
                Some(Err(e)) => rsx! { div { class: "alert alert-error", "Error: {e}" } },
                None => rsx! { div { class: "loading-spinner", "Cargando..." } },
            }
            match agenda() {
                Some(Ok(data)) => {
                    let items = data["agenda"].as_array().map(|a| a.len()).unwrap_or(0);
                    rsx! {
                        Mosaico { title: "Proximos Eventos", value: "{items}", icon: "📅", color: "#8b5cf6" }
                    }
                }
                Some(Err(e)) => rsx! { div { class: "alert alert-error", "Error: {e}" } },
                None => rsx! { div { class: "loading-spinner", "Cargando..." } },
            }
        }
    }
}

#[component]
fn Mosaico(title: String, value: String, icon: String, color: String) -> Element {
    rsx! {
        div { class: "mosaico-card", style: "border-top: 4px solid {color};",
            div { class: "mosaico-icon", "{icon}" }
            div { class: "mosaico-content",
                div { class: "mosaico-value", "{value}" }
                div { class: "mosaico-title", "{title}" }
            }
        }
    }
}
