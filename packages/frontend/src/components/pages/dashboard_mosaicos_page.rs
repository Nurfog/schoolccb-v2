use dioxus::prelude::*;
use crate::api::client;
use crate::seo::use_page_title;
use crate::components::widgets::simple_chart::{BarChart, DoughnutChart};

fn att_class(pct: f64) -> &'static str {
    if pct >= 90.0 { "pct-good" }
    else if pct >= 80.0 { "pct-warning" }
    else { "pct-danger" }
}

fn bar_class(pct: f64) -> &'static str {
    if pct >= 90.0 { "good" }
    else if pct >= 80.0 { "warn" }
    else { "bad" }
}

fn sev_class(s: &str) -> &'static str {
    match s {
        "critical" | "high" => "Alto",
        "medium" => "Medio",
        _ => "Bajo",
    }
}

#[component]
pub fn DashboardMosaicosPage() -> Element {
    use_page_title("Dashboard");
    let summary = use_resource(client::fetch_dashboard_summary);
    let attendance_today = use_resource(client::fetch_attendance_today);
    let agenda = use_resource(client::fetch_agenda);
    let att_trends = use_resource(client::fetch_school_attendance_trends);
    let grades_dist = use_resource(client::fetch_school_grades_distribution);
    let finance = use_resource(client::fetch_school_finance_summary);
    let teacher_perf = use_resource(client::fetch_school_teacher_performance);
    let top_alerts = use_resource(client::fetch_school_top_alerts);

    rsx! {
        div { class: "page-header",
            h1 { "Dashboard" }
            p { "Panorama general del colegio" }
        }
        match summary() {
            Some(Ok(data)) => {
                let total = data["total_students"].as_i64().unwrap_or(0);
                let enrolled = data["total_enrolled"].as_i64().unwrap_or(0);
                let teachers = data["total_teachers"].as_i64().unwrap_or(0);
                rsx! {
                    div { class: "mosaicos-grid",
                        Mosaico { title: "Alumnos", value: "{total}", color: "#1a2b3c" }
                        Mosaico { title: "Matriculados", value: "{enrolled}", color: "#22c55e" }
                        Mosaico { title: "Docentes", value: "{teachers}", color: "#3b82f6" }
                    }
                }
            }
            Some(Err(e)) => rsx! { div { class: "alert alert-error", "Error: {e}" } },
            None => rsx! { div { class: "loading-spinner", "Cargando..." } },
        }
        div { class: "dashboard-grid",
            match attendance_today() {
                Some(Ok(data)) => {
                    let pct = data["attendance_percentage"].as_f64().unwrap_or(0.0);
                    let present = data["present"].as_i64().unwrap_or(0);
                    let absent = data["absent"].as_i64().unwrap_or(0);
                    let late = data["late"].as_i64().unwrap_or(0);
                    let justified = data["justified"].as_i64().unwrap_or(0);
                    let total = data["total_students"].as_i64().unwrap_or(0);
                    let a_cls = att_class(pct);
                    let b_cls = bar_class(pct);
                    rsx! {
                        div { class: "widget-card",
                            div { class: "widget-card-header",
                                h3 { "Asistencia Hoy" }
                                span { class: "{a_cls}", "{pct:.1}%" }
                            }
                            div { class: "widget-card-body",
                                div { class: "percentage-bar",
                                    div { class: "fill {b_cls}", style: "width: {pct}%" }
                                }
                                div { class: "percentage-text",
                                    strong { "{present} " } "Presentes · "
                                    strong { "{justified} " } "Justificados · "
                                    strong { "{late} " } "Atrasos · "
                                    strong { "{absent} " } "Ausentes"
                                }
                                div { class: "summary-text", "Total: {total} estudiantes" }
                            }
                        }
                    }
                }
                _ => rsx! {}
            }
            match att_trends() {
                Some(Ok(data)) => {
                    let trends = data["trends"].as_array().cloned().unwrap_or_default();
                    let avg = data["average"].as_str().unwrap_or("0").to_string();
                    if trends.is_empty() {
                        rsx! {}
                    } else {
                        rsx! {
                            div { class: "widget-card",
                                div { class: "widget-card-header",
                                    h3 { "Evolución Asistencia" }
                                    span { "Prom: {avg}%" }
                                }
                                div { class: "widget-card-body",
                                    BarChart {
                                        data: trends,
                                        label_key: "month".to_string(),
                                        value_key: "attendance".to_string(),
                                        height: None,
                                        color: Some("#3b82f6".to_string()),
                                    }
                                }
                            }
                        }
                    }
                }
                _ => rsx! {}
            }
            match grades_dist() {
                Some(Ok(data)) => {
                    let dist = data["distribution"].as_array().cloned().unwrap_or_default();
                    let avg = data["average"].as_str().unwrap_or("0").to_string();
                    if dist.is_empty() {
                        rsx! {}
                    } else {
                        rsx! {
                            div { class: "widget-card",
                                div { class: "widget-card-header",
                                    h3 { "Distribución de Notas" }
                                    span { "Prom: {avg}" }
                                }
                                div { class: "widget-card-body",
                                    DoughnutChart {
                                        data: dist,
                                        label_key: "range".to_string(),
                                        value_key: "count".to_string(),
                                        size: None,
                                    }
                                }
                            }
                        }
                    }
                }
                _ => rsx! {}
            }
            match finance() {
                Some(Ok(data)) => {
                    let revenue = data["monthly_revenue"].as_f64().unwrap_or(0.0);
                    let pending = data["total_pending"].as_f64().unwrap_or(0.0);
                    let collected = data["total_collected"].as_f64().unwrap_or(0.0);
                    let pending_count = data["pending_count"].as_i64().unwrap_or(0);
                    let p_cls = if pending > 0.0 { "warning" } else { "success" };
                    rsx! {
                        div { class: "widget-card",
                            div { class: "widget-card-header",
                                h3 { "Finanzas" }
                                span { "Resumen" }
                            }
                            div { class: "widget-card-body",
                                div { class: "kpi-grid",
                                    div { class: "kpi-item",
                                        div { class: "kpi-value success", "${revenue:.0}" }
                                        div { class: "kpi-label", "Ingresos del Mes" }
                                    }
                                    div { class: "kpi-item",
                                        div { class: "kpi-value info", "${collected:.0}" }
                                        div { class: "kpi-label", "Total Cobrado" }
                                    }
                                    div { class: "kpi-item",
                                        div { class: "kpi-value {p_cls}", "${pending:.0}" }
                                        div { class: "kpi-label", "Pendiente ({pending_count})" }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => rsx! {}
            }
            match top_alerts() {
                Some(Ok(data)) => {
                    let alert_list = data["alerts"].as_array().cloned().unwrap_or_default();
                    let alert_rows: Vec<Element> = alert_list.iter().map(|a| {
                        let name = a["student_name"].as_str().unwrap_or("").to_string();
                        let course = a["course"].as_str().unwrap_or("").to_string();
                        let att_val = a["attendance"].as_str().unwrap_or("0").to_string();
                        let severity = a["severity"].as_str().unwrap_or("").to_string();
                        let sev = sev_class(&severity);
                        rsx! {
                            div { class: "alert-item",
                                div { class: "alert-badge {sev}" }
                                div { class: "alert-info",
                                    div { class: "alert-name", "{name}" }
                                    div { class: "alert-detail", "{course} — {att_val}%" }
                                }
                                span { class: "alert-severity {sev}", "{severity}" }
                            }
                        }
                    }).collect();
                    rsx! {
                        div { class: "widget-card",
                            div { class: "widget-card-header",
                                h3 { "Alertas de Asistencia" }
                                span { "{alert_list.len()} alertas" }
                            }
                            div { class: "widget-card-body",
                                if alert_rows.is_empty() {
                                    div { class: "empty-state", "Sin alertas activas" }
                                } else {
                                    {alert_rows.into_iter()}
                                }
                            }
                        }
                    }
                }
                _ => rsx! {}
            }
            match teacher_perf() {
                Some(Ok(data)) => {
                    let teachers = data["teachers"].as_array().cloned().unwrap_or_default();
                    let t_rows: Vec<Element> = teachers.iter().map(|t| {
                        let name = t["name"].as_str().unwrap_or("-").to_string();
                        let grade = t["avg_grade"].as_str().unwrap_or("-").to_string();
                        let count = t["total_grades"].as_i64().unwrap_or(0);
                        rsx! {
                            tr {
                                td { "{name}" }
                                td { class: "grade-good", "{grade}" }
                                td { "{count}" }
                            }
                        }
                    }).collect();
                    if t_rows.is_empty() {
                        rsx! {}
                    } else {
                        rsx! {
                            div { class: "widget-card",
                                div { class: "widget-card-header",
                                    h3 { "Rendimiento Docente" }
                                    span { "Top docentes" }
                                }
                                div { class: "widget-card-body",
                                    div { class: "data-table-container",
                                        table { class: "data-table",
                                            thead {
                                                tr {
                                                    th { "Docente" }
                                                    th { "Prom. Notas" }
                                                    th { "Notas" }
                                                }
                                            }
                                            tbody { {t_rows.into_iter()} }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => rsx! {}
            }
            match agenda() {
                Some(Ok(data)) => {
                    let events = data["events"].as_array().cloned().unwrap_or_default();
                    let ev_rows: Vec<Element> = events.iter().map(|ev| {
                        let title = ev["title"].as_str().unwrap_or("").to_string();
                        let ev_type = ev["type"].as_str().unwrap_or("").to_string();
                        let date = ev["date"].as_str().unwrap_or("").to_string();
                        rsx! {
                            div { class: "event-item",
                                div { class: "event-date-badge evento",
                                    span { class: "day", "{date}" }
                                    span { class: "month", "..." }
                                }
                                div { class: "event-details",
                                    div { class: "event-title", "{title}" }
                                    div { class: "event-type", "{ev_type}" }
                                }
                            }
                        }
                    }).collect();
                    rsx! {
                        div { class: "widget-card",
                            div { class: "widget-card-header",
                                h3 { "Próximos Eventos" }
                                span { "{events.len()} eventos" }
                            }
                            div { class: "widget-card-body",
                                if ev_rows.is_empty() {
                                    div { class: "empty-state", "Sin eventos programados" }
                                } else {
                                    {ev_rows.into_iter()}
                                }
                            }
                        }
                    }
                }
                _ => rsx! {}
            }
        }
    }
}

#[component]
fn Mosaico(title: String, value: String, color: String) -> Element {
    rsx! {
        div { class: "mosaico-card", style: "border-top: 4px solid {color};",
            div { class: "mosaico-content", style: "text-align: center;",
                div { class: "mosaico-value", "{value}" }
                div { class: "mosaico-title", "{title}" }
            }
        }
    }
}
