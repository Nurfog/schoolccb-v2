use dioxus::prelude::*;

use crate::api::client;

#[component]
pub fn KanbanBoard(
    stages: Resource<Result<serde_json::Value, String>>,
    prospects: Resource<Result<serde_json::Value, String>>,
    selected_id: Signal<Option<String>>,
) -> Element {
    let columns: Vec<Element> = match (stages(), prospects()) {
        (Some(Ok(sj)), Some(Ok(pj))) => {
            let stage_list = sj["stages"].as_array().cloned().unwrap_or_default();
            let prospect_list = pj["prospects"].as_array().cloned().unwrap_or_default();
            stage_list.iter().map(|stage| {
                let stage_id = stage["id"].as_str().unwrap_or("").to_string();
                let stage_name = stage["name"].as_str().unwrap_or("").to_string();
                let cards: Vec<(&str, String, String)> = prospect_list.iter()
                    .filter(|p| p["current_stage_id"].as_str().unwrap_or("") == stage_id)
                    .map(|p| {
                        let pid = p["id"].as_str().unwrap_or("");
                        let pname = format!("{} {}",
                            p["first_name"].as_str().unwrap_or(""),
                            p["last_name"].as_str().unwrap_or(""),
                        );
                        let prut = p["rut"].as_str().unwrap_or("").to_string();
                        (pid, pname, prut)
                    })
                    .collect();
                let card_count = cards.len();
                let sel = selected_id();
                rsx! {
                    div { class: "kanban-column", key: "{stage_id}",
                        div { class: "kanban-column-header",
                            h3 { "{stage_name}" }
                            span { class: "kanban-count", "{card_count}" }
                        }
                        div { class: "kanban-cards",
                            for (pid, pname, prut) in &cards {
                                let pid_s = pid.to_string();
                                let is_sel = sel.as_deref() == Some(pid);
                                rsx! {
                                    div {
                                        class: "kanban-card",
                                        class: if is_sel { "selected" } else { "" },
                                        onclick: move |_| { selected_id.set(Some(pid_s.clone())); },
                                        div { class: "card-name", "{pname}" }
                                        div { class: "card-rut", "{prut}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }).collect()
        }
        _ => vec![],
    };

    rsx! {
        div { class: "kanban-board",
            {
                if columns.is_empty() && stages().is_some() {
                    rsx! { div { class: "empty-state", "No hay postulantes en ninguna etapa" } }
                } else {
                    rsx! { { columns.into_iter() } }
                }
            }
        }
    }
}

#[component]
pub fn ProspectTable(prospects: Resource<Result<serde_json::Value, String>>) -> Element {
    rsx! {
        div { class: "data-table-container",
            match prospects() {
                Some(Ok(data)) => {
                    let list = data["prospects"].as_array().cloned().unwrap_or_default();
                    if list.is_empty() {
                        rsx! { div { class: "empty-state", "No hay postulantes" } }
                    } else {
                        rsx! {
                            table { class: "data-table",
                                thead { tr {
                                    th { "Nombre" }
                                    th { "RUT" }
                                    th { "Etapa" }
                                    th { "Origen" }
                                    th { "Creado" }
                                }}
                                tbody {
                                    for p in &list {
                                        let _pid = p["id"].as_str().unwrap_or("").to_string();
                                        let name = format!("{} {}",
                                            p["first_name"].as_str().unwrap_or(""),
                                            p["last_name"].as_str().unwrap_or("")
                                        );
                                        let rut = p["rut"].as_str().unwrap_or("-").to_string();
                                        let stage = p["current_stage_name"].as_str().unwrap_or("-").to_string();
                                        let source = p["source"].as_str().unwrap_or("-").to_string();
                                        let date = p["created_at"].as_str().unwrap_or("").to_string();
                                        rsx! {
                                            tr { class: "clickable-row", onclick: move |_| {
                                            },
                                                td { "{name}" }
                                                td { "{rut}" }
                                                td { span { class: "role-badge", "{stage}" } }
                                                td { "{source}" }
                                                td { "{date}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => rsx! { div { class: "empty-state", "Cargando..." } },
            }
        }
    }
}
