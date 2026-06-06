use dioxus::prelude::*;
use crate::api::client;

#[component]
pub fn AuditPage() -> Element {
    let logs = use_resource(|| client::fetch_audit_logs());
    let mut filter_entity = use_signal(|| "".to_string());
    let mut filter_action = use_signal(|| "".to_string());

    rsx! {
        div { class: "page-header",
            h1 { "Auditoría del Sistema" }
            p { "Registro de cambios y acciones realizadas en el sistema" }
        }
        div { class: "page-toolbar",
            div { class: "form-row", style: "gap: 12px; align-items: center;",
                div { class: "form-group", style: "margin-bottom: 0;",
                    label { "Entidad:" }
                    input { class: "form-input", style: "width: 200px;", value: "{filter_entity}",
                        oninput: move |e| filter_entity.set(e.value()),
                        placeholder: "Filtrar por entidad..." }
                }
                div { class: "form-group", style: "margin-bottom: 0;",
                    label { "Acción:" }
                    input { class: "form-input", style: "width: 200px;", value: "{filter_action}",
                        oninput: move |e| filter_action.set(e.value()),
                        placeholder: "Filtrar por acción..." }
                }
                button { class: "btn btn-secondary", onclick: move |_| { filter_entity.set(String::new()); filter_action.set(String::new()); },
                    "Limpiar Filtros" }
            }
        }
        match logs() {
            Some(Ok(data)) => {
                let list = data["logs"].as_array().or_else(|| data["audit_logs"].as_array()).cloned().unwrap_or_default();
                let fe = filter_entity().to_lowercase();
                let fa = filter_action().to_lowercase();
                let filtered: Vec<&serde_json::Value> = list.iter().filter(|log| {
                    let entity = log["entity_type"].as_str().unwrap_or("").to_lowercase();
                    let action = log["action"].as_str().unwrap_or("").to_lowercase();
                    (fe.is_empty() || entity.contains(&fe)) && (fa.is_empty() || action.contains(&fa))
                }).collect();

                let rows: Vec<Element> = filtered.iter().map(|log| {
                    let entity_type = log["entity_type"].as_str().unwrap_or("-").to_string();
                    let entity_id = log["entity_id"].as_str().unwrap_or("-").to_string();
                    let action = log["action"].as_str().unwrap_or("-").to_string();
                    let user_id = log["user_id"].as_str().map(|s| {
                        if s.len() > 8 { format!("{}...", &s[..8]) } else { s.to_string() }
                    }).unwrap_or_else(|| "-".to_string());
                    let created = log["created_at"].as_str().unwrap_or("-").to_string();
                    let has_changes = log["changes"].is_object();
                    rsx! {
                        tr {
                            td { "{entity_type}" }
                            td { "{action}" }
                            td { title: "{entity_id}", "{entity_id}" }
                            td { "{user_id}" }
                            td { if has_changes { span { class: "badge badge-info", "✓" } } else { "-" } }
                            td { "{created}" }
                        }
                    }
                }).collect();
                let empty = filtered.is_empty();
                rsx! {
                    div { class: "data-table-container",
                        table { class: "data-table",
                            thead { tr {
                                th { "Entidad" }
                                th { "Acción" }
                                th { "ID Entidad" }
                                th { "Usuario" }
                                th { "Cambios" }
                                th { "Fecha" }
                            }}
                            tbody { {rows.into_iter()} }
                        }
                        if empty {
                            div { class: "empty-state", "Sin registros de auditoría para los filtros seleccionados" }
                        } else {
                            p { style: "text-align: right; color: #546e7a; font-size: 0.85rem; margin-top: 8px;",
                                "Mostrando {filtered.len()} de {list.len()} registros" }
                        }
                    }
                }
            }
            _ => rsx! { div { class: "loading-spinner", "Cargando registros de auditoría..." } },
        }
    }
}
