use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client;
use crate::seo::use_page_title;

#[component]
pub fn CorporationDashboardPage() -> Element {
    use_page_title("Dashboard Corporativo");
    let summary = use_resource(|| client::fetch_json("/api/corporation/dashboard/summary"));
    let schools = use_resource(|| client::fetch_json("/api/corporation/dashboard/schools"));
    let comparisons = use_resource(|| client::fetch_json("/api/corporation/dashboard/comparisons"));
    let alerts_data = use_resource(|| client::fetch_json("/api/corporation/dashboard/alerts"));
    let trends_data = use_resource(|| client::fetch_json("/api/corporation/dashboard/trends"));
    let license = use_resource(|| client::fetch_json("/api/corporation/dashboard/license"));

    match summary() {
        None => return rsx! { div { class: "loading-spinner", "Cargando dashboard corporativo..." } },
        Some(Err(e)) => return rsx! { div { class: "alert alert-error", "Error al cargar dashboard: {e}" } },
        _ => {}
    }

    let s = match summary() { Some(Ok(ref d)) => Some(d.clone()), _ => None };
    let schools_list: Vec<Value> = match schools() {
        Some(Ok(ref d)) => d["schools"].as_array().cloned().unwrap_or_default(),
        Some(Err(e)) => { log::warn!("Error en colegios: {e}"); vec![] },
        None => vec![],
    };
    let comp_list: Vec<Value> = match comparisons() {
        Some(Ok(ref d)) => d["comparisons"].as_array().cloned().unwrap_or_default(),
        Some(Err(e)) => { log::warn!("Error en comparativas: {e}"); vec![] },
        None => vec![],
    };
    let alerts_list: Vec<Value> = match alerts_data() {
        Some(Ok(ref d)) => d["alerts"].as_array().cloned().unwrap_or_default(),
        Some(Err(e)) => { log::warn!("Error en alertas: {e}"); vec![] },
        None => vec![],
    };
    let lic = match license() { Some(Ok(ref d)) => Some(d.clone()), Some(Err(e)) => { log::warn!("Error en licencia: {e}"); None }, None => None };

    let total_students = s.as_ref().and_then(|d| d["total_students"].as_i64()).unwrap_or(0);
    let total_schools = s.as_ref().and_then(|d| d["total_schools"].as_i64()).unwrap_or(0);
    let total_teachers = s.as_ref().and_then(|d| d["total_teachers"].as_i64()).unwrap_or(0);
    let avg_attendance = s.as_ref().and_then(|d| d["avg_attendance"].as_str()).unwrap_or("0");
    let avg_grades = s.as_ref().and_then(|d| d["avg_grades"].as_str()).unwrap_or("0");
    let monthly_revenue = s.as_ref().and_then(|d| d["monthly_revenue"].as_f64()).unwrap_or(0.0);
    let expiring = s.as_ref().and_then(|d| d["expiring_licenses"].as_i64()).unwrap_or(0);

    let license_name = lic.as_ref().and_then(|d| d["plan_name"].as_str()).unwrap_or("Sin licencia");
    let license_days = lic.as_ref().and_then(|d| d["days_remaining"].as_i64()).unwrap_or(0);

    // Sort comparisons
    let mut sorted = comp_list.clone();
    sorted.sort_by(|a, b| {
        let a_val = a["attendance_pct"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
        let b_val = b["attendance_pct"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
        b_val.partial_cmp(&a_val).unwrap_or(std::cmp::Ordering::Equal)
    });

    let max_students = sorted.iter().filter_map(|s| s["total_students"].as_i64()).max().unwrap_or(1).max(1);

    rsx! {
        div { class: "page-header",
            h1 { "Dashboard Corporativo" }
            p { "Visión global de todos los colegios de tu corporación" }
        }

        div { class: "kpi-grid",
            div { class: "kpi-card",
                div { class: "kpi-value", "{total_schools}" }
                div { class: "kpi-label", "Colegios" }
            }
            div { class: "kpi-card",
                div { class: "kpi-value", "{total_students}" }
                div { class: "kpi-label", "Alumnos" }
            }
            div { class: "kpi-card",
                div { class: "kpi-value", "{total_teachers}" }
                div { class: "kpi-label", "Docentes" }
            }
            div { class: "kpi-card",
                div { class: "kpi-value", "{avg_attendance}%" }
                div { class: "kpi-label", "Asistencia Promedio" }
            }
            div { class: "kpi-card",
                div { class: "kpi-value", "{avg_grades}" }
                div { class: "kpi-label", "Promedio General" }
            }
            div { class: "kpi-card",
                div { class: "kpi-value", "${monthly_revenue}" }
                div { class: "kpi-label", "Ingresos del Mes" }
            }
        }

        div { class: "dashboard-grid",
            div { class: "dashboard-section",
                h3 { "Licencia" }
                div { class: "license-card",
                    div { class: "license-plan", "{license_name}" }
                    if license_days > 0 {
                        div { class: "license-days", "Quedan {license_days} días" }
                    }
                    if expiring > 0 {
                        div { class: "alert alert-warning", "{expiring} licencia(s) próxima(s) a vencer" }
                    }
                }
            }

            div { class: "dashboard-section",
                h3 { "Comparativa entre Colegios" }
                if sorted.is_empty() {
                    div { class: "empty-state", "Sin datos de colegios" }
                } else {
                    div { class: "data-table-container",
                        table { class: "data-table",
                            thead { tr { th { "Colegio" } th { "Alumnos" } th { "Asistencia" } th { "Promedio" } th { "Desempeño" } } }
                            tbody {
                                {sorted.iter().map(|school| {
                                    let name = school["school_name"].as_str().unwrap_or("").to_string();
                                    let students = school["total_students"].as_i64().unwrap_or(0);
                                    let att = school["attendance_pct"].as_str().unwrap_or("0").to_string();
                                    let grade = school["avg_grade"].as_str().unwrap_or("0").to_string();
                                    let pct = students as f64 / max_students as f64 * 100.0;
                                    rsx! {
                                        tr {
                                            td { "{name}" }
                                            td { "{students}" }
                                            td {
                                                div { class: "progress-bar",
                                                    div { class: "progress-fill", style: "width: {att}%; background: #22c55e;", "{att}%" }
                                                }
                                            }
                                            td { "{grade}" }
                                            td {
                                                div { class: "progress-bar",
                                                    div { class: "progress-fill", style: "width: {pct:.0}%; background: #3b82f6;", "{students}" }
                                                }
                                            }
                                        }
                                    }
                                })}
                            }
                        }
                    }
                }
            }

            div { class: "dashboard-section",
                h3 { "Alertas Corporativas" }
                if alerts_list.is_empty() {
                    div { class: "empty-state", "Sin alertas activas" }
                } else {
                    div { class: "alerts-list",
                        {alerts_list.iter().map(|a| {
                            let atype = a["type"].as_str().unwrap_or("").to_string();
                            let severity = a["severity"].as_str().unwrap_or("info").to_string();
                            let msg = a["message"].as_str().unwrap_or("").to_string();
                            let school_name = a["school_name"].as_str().unwrap_or("").to_string();
                            let sev_class = match severity.as_str() {
                                "critical" => "alert alert-error",
                                "high" => "alert alert-warning",
                                _ => "alert alert-info",
                            };
                            rsx! {
                                div { class: "{sev_class}",
                                    b { "{school_name}" } " — {msg}"
                                }
                            }
                        })}
                    }
                }
            }
        }

        div { class: "dashboard-section",
            h3 { "Ranking de Colegios por Asistencia" }
            if sorted.is_empty() {
                div { class: "empty-state", "Sin datos" }
            } else {
                div { class: "ranking-list",
                    {sorted.iter().enumerate().map(|(i, school)| {
                        let name = school["school_name"].as_str().unwrap_or("").to_string();
                        let att = school["attendance_pct"].as_str().unwrap_or("0").to_string();
                        let grade = school["avg_grade"].as_str().unwrap_or("0").to_string();
                        let students = school["total_students"].as_i64().unwrap_or(0);
                        let medals = ["🥇", "🥈", "🥉"];
                        let rank = if i < 3 { medals[i].to_string() } else { format!("#{}", i + 1) };
                        rsx! {
                            div { class: "ranking-row",
                                span { class: "ranking-pos", "{rank}" }
                                span { class: "ranking-name", "{name}" }
                                span { class: "ranking-stat", "Asistencia: {att}%" }
                                span { class: "ranking-stat", "Notas: {grade}" }
                                span { class: "ranking-stat", "Alumnos: {students}" }
                            }
                        }
                    })}
                }
            }
        }
    }
}
