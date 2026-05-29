use dioxus::prelude::*;
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
            let sel = selected_id();
            stage_list.iter().map(|stage| {
                let stage_id = stage["id"].as_str().unwrap_or("").to_string();
                let stage_name = stage["name"].as_str().unwrap_or("").to_string();
                let mut card_elements = Vec::new();
                for p in &prospect_list {
                    if p["current_stage_id"].as_str().unwrap_or("") != stage_id { continue; }
                    let pid = p["id"].as_str().unwrap_or("").to_string();
                    let pname = format!("{} {}",
                        p["first_name"].as_str().unwrap_or(""),
                        p["last_name"].as_str().unwrap_or(""),
                    );
                    let prut = p["rut"].as_str().unwrap_or("").to_string();
                    let is_sel = sel.as_deref() == Some(&pid);
                    let pid_clone = pid.clone();
                    card_elements.push(rsx! {
                        div {
                            class: if is_sel { "kanban-card selected" } else { "kanban-card" },
                            onclick: move |_| { selected_id.set(Some(pid_clone.clone())); },
                            div { class: "card-name", "{pname}" }
                            div { class: "card-rut", "{prut}" }
                        }
                    });
                }
                let card_count = card_elements.len();
                rsx! {
                    div { class: "kanban-column", key: "{stage_id}",
                        div { class: "kanban-column-header",
                            h3 { "{stage_name}" }
                            span { class: "kanban-count", "{card_count}" }
                        }
                        div { class: "kanban-cards",
                            {card_elements.into_iter()}
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
                        let rows: Vec<(String, String, String, String, String)> = list.iter().map(|p| {
                            let name = format!("{} {}",
                                p["first_name"].as_str().unwrap_or(""),
                                p["last_name"].as_str().unwrap_or("")
                            );
                            (
                                p["id"].as_str().unwrap_or("").to_string(),
                                name,
                                p["rut"].as_str().unwrap_or("-").to_string(),
                                p["current_stage_name"].as_str().unwrap_or("-").to_string(),
                                p["source"].as_str().unwrap_or("-").to_string(),
                            )
                        }).collect();
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
                                    for (_, name, rut, stage, source) in &rows {
                                        tr {
                                            td { "{name}" }
                                            td { span { class: "rut-badge", "{rut}" } }
                                            td { "{stage}" }
                                            td { "{source}" }
                                            td { "-" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! { div { class: "state-error", "Error: {e}" } },
                None => rsx! { div { class: "empty-state", div { class: "loading-spinner", "Cargando..." } } },
            }
        }
    }
}
