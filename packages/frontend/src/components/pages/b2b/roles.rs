use dioxus::prelude::*;
use crate::api::client;

#[component]
pub fn B2bRolesPage() -> Element {
    let roles = use_resource(|| client::fetch_json("/api/roles"));

    rsx! {
        div { class: "page-header",
            h1 { "Roles y Permisos B2B" }
            p { "Administración de roles del área corporativa y comercial" }
        }
        div { class: "data-table-container",
            match roles() {
                Some(Ok(data)) => {
                    let list: Vec<serde_json::Value> = data["roles"].as_array().cloned().unwrap_or_default();
                    let rows: Vec<(String, String, String)> = list.iter().map(|role| {
                        (
                            role["name"].as_str().unwrap_or("-").to_string(),
                            role["description"].as_str().unwrap_or("").to_string(),
                            role["role_type"].as_str().unwrap_or("b2b").to_string(),
                        )
                    }).collect();
                    rsx! {
                        table { class: "data-table",
                            thead { tr {
                                th { "Nombre" }
                                th { "Descripción" }
                                th { "Tipo" }
                            }}
                            tbody { for (name, desc, role_type) in &rows {
                                tr {
                                    td { "{name}" }
                                    td { "{desc}" }
                                    td { "{role_type}" }
                                }
                            }}
                        }
                        if rows.is_empty() {
                            div { class: "empty-state", "Sin roles configurados" }
                        }
                    }
                }
                Some(Err(e)) => rsx! { div { class: "empty-state", "Error: {e}" } },
                None => rsx! { div { class: "empty-state", div { class: "loading-spinner", "Cargando..." } } },
            }
        }
    }
}
