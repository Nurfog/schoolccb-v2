use dioxus::prelude::*;
use crate::api::client;

#[component]
pub fn B2bHrPage() -> Element {
    let agents = use_resource(|| client::fetch_json("/b2b/sales/agents"));
    let mut search = use_signal(String::new);

    rsx! {
        div { class: "page-header",
            h1 { "Equipo B2B" }
            p { "Gestión del equipo comercial y corporativo" }
        }
        div { class: "page-toolbar",
            div { class: "filter-group",
                input {
                    class: "form-input",
                    placeholder: "Buscar por nombre o email...",
                    value: "{search}",
                    oninput: move |e| search.set(e.value()),
                }
            }
        }
        div { class: "data-table-container",
            match agents() {
                Some(Ok(data)) => {
                    let list: Vec<serde_json::Value> = data["agents"].as_array().cloned().unwrap_or_default()
                        .into_iter().filter(|entry| {
                            let q = search().to_lowercase();
                            if q.is_empty() { return true; }
                            let user = &entry["user"];
                            user["name"].as_str().unwrap_or("").to_lowercase().contains(&q)
                                || user["email"].as_str().unwrap_or("").to_lowercase().contains(&q)
                        }).collect();
                    let rows: Vec<(String, String, String, f64, f64, bool)> = list.iter().map(|entry| {
                        let agent = &entry["agent"];
                        let user = &entry["user"];
                        (
                            user["name"].as_str().unwrap_or("-").to_string(),
                            user["email"].as_str().unwrap_or("-").to_string(),
                            user["role"].as_str().unwrap_or("-").to_string(),
                            agent["quota_monthly"].as_f64().unwrap_or(0.0),
                            agent["commission_rate"].as_f64().unwrap_or(0.0),
                            agent["active"].as_bool().unwrap_or(false),
                        )
                    }).collect();
                    rsx! {
                        table { class: "data-table",
                            thead { tr {
                                th { "Nombre" }
                                th { "Email" }
                                th { "Rol" }
                                th { "Cuota Mensual" }
                                th { "Comisión" }
                                th { "Estado" }
                            }}
                            tbody { for (name, email, role, monthly, commission, active) in &rows {
                                tr {
                                    td { "{name}" }
                                    td { "{email}" }
                                    td { "{role}" }
                                    td { "${monthly:.0}" }
                                    td { "{commission:.0}%" }
                                    td { if *active { span { class: "grade-good", "Activo" } } else { span { class: "grade-bad", "Inactivo" } } }
                                }
                            }}
                        }
                        if rows.is_empty() {
                            div { class: "empty-state", "Sin miembros en el equipo" }
                        }
                    }
                }
                Some(Err(e)) => rsx! { div { class: "empty-state", "Error: {e}" } },
                None => rsx! { div { class: "empty-state", div { class: "loading-spinner", "Cargando..." } } },
            }
        }
    }
}
