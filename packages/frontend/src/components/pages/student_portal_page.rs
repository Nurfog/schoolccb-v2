use dioxus::prelude::*;
use crate::api::client;
use crate::seo::use_page_title;

#[component]
pub fn StudentPortalPage() -> Element {
    use_page_title("Mi Portal");
    let profile = use_resource(|| client::fetch_json("/api/portal/student/profile"));
    let grades = use_resource(|| client::fetch_json("/api/portal/student/grades"));
    let attendance = use_resource(|| client::fetch_json("/api/portal/student/attendance"));
    let schedule = use_resource(|| client::fetch_json("/api/portal/student/schedule"));
    let annotations = use_resource(|| client::fetch_json("/api/portal/student/annotations"));

    let p = match profile() { Some(Ok(ref d)) => Some(d.clone()), _ => None };
    let name = p.as_ref().and_then(|d| d["name"].as_str()).unwrap_or("");
    let grade = p.as_ref().and_then(|d| d["grade_level"].as_str()).unwrap_or("");

    rsx! {
        div { class: "page-header",
            h1 { "Mi Portal" }
            p { "Bienvenido, {name}" }
        }

        div { class: "kpi-grid",
            div { class: "kpi-card",
                div { class: "kpi-value", "{grade}" }
                div { class: "kpi-label", "Curso" }
            }
        }

        div { class: "dashboard-grid",
            div { class: "dashboard-section",
                h3 { "Mis Calificaciones" }
                match grades() {
                    Some(Ok(ref d)) => {
                        let averages = d["averages"].as_array().cloned().unwrap_or_default();
                        let grades_list = d["grades"].as_array().cloned().unwrap_or_default();
                        rsx! {
                            if !averages.is_empty() {
                                div { class: "data-table-container",
                                    table { class: "data-table",
                                        thead { tr { th { "Asignatura" } th { "Promedio" } } }
                                        tbody { {averages.iter().map(|a| {
                                            let s = a["subject"].as_str().unwrap_or("").to_string();
                                            let v = a["average"].as_str().unwrap_or("0").to_string();
                                            rsx! { tr { td { "{s}" } td { b { "{v}" } } } }
                                        })} }
                                    }
                                }
                            }
                            if !grades_list.is_empty() {
                                h4 { "Últimas Notas" }
                                div { class: "data-table-container",
                                    table { class: "data-table",
                                        thead { tr { th { "Asignatura" } th { "Nota" } th { "Fecha" } } }
                                        tbody { {grades_list.iter().take(10).map(|g| {
                                            let s = g["subject"].as_str().unwrap_or("").to_string();
                                            let v = g["value"].as_f64().unwrap_or(0.0);
                                            let d = g["date"].as_str().unwrap_or("").to_string();
                                            rsx! { tr { td { "{s}" } td { "{v}" } td { "{d}" } } }
                                        })} }
                                    }
                                }
                            }
                        }
                    }
                    _ => rsx! { div { class: "loading-spinner", "Cargando..." } },
                }
            }

            div { class: "dashboard-section",
                h3 { "Asistencia" }
                match attendance() {
                    Some(Ok(ref d)) => {
                        let months = d["attendance"].as_array().cloned().unwrap_or_default();
                        rsx! {
                            div { class: "data-table-container",
                                table { class: "data-table",
                                    thead { tr { th { "Mes" } th { "Asistencia" } th { "Faltas" } } }
                                    tbody { {months.iter().map(|m| {
                                        let mo = m["month"].as_str().unwrap_or("").to_string();
                                        let pct = m["percentage"].as_str().unwrap_or("0").to_string();
                                        let abs = m["absent"].as_i64().unwrap_or(0);
                                        rsx! { tr { td { "{mo}" } td { "{pct}%" } td { "{abs}" } } }
                                    })} }
                                }
                            }
                        }
                    }
                    _ => rsx! { div { class: "loading-spinner", "Cargando..." } },
                }
            }
        }

        div { class: "dashboard-grid",
            div { class: "dashboard-section",
                h3 { "Horario" }
                match schedule() {
                    Some(Ok(ref d)) => {
                        let items = d["schedule"].as_array().cloned().unwrap_or_default();
                        rsx! {
                            div { class: "data-table-container",
                                table { class: "data-table",
                                    thead { tr { th { "Día" } th { "Hora" } th { "Asignatura" } } }
                                    tbody { {items.iter().map(|s| {
                                        let day = s["day"].as_str().unwrap_or("").to_string();
                                        let time = s["time"].as_str().unwrap_or("").to_string();
                                        let sub = s["subject"].as_str().unwrap_or("").to_string();
                                        rsx! { tr { td { "{day}" } td { "{time}" } td { "{sub}" } } }
                                    })} }
                                }
                            }
                        }
                    }
                    _ => rsx! { div { class: "loading-spinner", "Cargando..." } },
                }
            }

            div { class: "dashboard-section",
                h3 { "Anotaciones" }
                match annotations() {
                    Some(Ok(ref d)) => {
                        let items = d["annotations"].as_array().cloned().unwrap_or_default();
                        rsx! {
                            if items.is_empty() {
                                div { class: "empty-state", "Sin anotaciones" }
                            } else {
                                for a in items {
                                    let atype = a["type"].as_str().unwrap_or("").to_string();
                                    let desc = a["description"].as_str().unwrap_or("").to_string();
                                    let date = a["date"].as_str().unwrap_or("").to_string();
                                    let cls = if atype == "positive" { "alert alert-success" } else { "alert alert-warning" };
                                    rsx! { div { class: "{cls}", b { "{atype}" } " — {desc} ({date})" } }
                                }
                            }
                        }
                    }
                    _ => rsx! { div { class: "loading-spinner", "Cargando..." } },
                }
            }
        }
    }
}
