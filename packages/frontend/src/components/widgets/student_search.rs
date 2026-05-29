use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client;

#[component]
pub fn StudentSearchSelect(
    on_select: EventHandler<String>,
    reset_key: Option<String>,
) -> Element {
    let mut query = use_signal(String::new);
    let mut is_open = use_signal(|| false);
    let mut selected_id = use_signal(|| String::new());
    let mut selected_name = use_signal(|| String::new());
    let _ = &reset_key;

    let results = use_resource(move || {
        let q = query();
        async move {
            if q.len() < 2 {
                Vec::<Value>::new()
            } else {
                match client::search_students(&q).await {
                    Ok(data) => data["students"].as_array().cloned().unwrap_or_default(),
                    Err(_) => Vec::new(),
                }
            }
        }
    });

    let items = results().unwrap_or_default();

    let show_results = is_open() && query().len() >= 2 && selected_id().is_empty();

    rsx! {
        div { class: "student-search-select",
            if !selected_id().is_empty() {
                div { class: "selected-student",
                    span { "{selected_name}" }
                    button {
                        class: "btn-icon",
                        "aria-label": "Limpiar selección",
                        onclick: move |_| {
                            selected_id.set(String::new());
                            selected_name.set(String::new());
                            query.set(String::new());
                            on_select.call(String::new());
                        },
                        svg { role: "presentation", view_box: "0 0 24 24", width: "14", height: "14",
                            line { x1: "18", y1: "6", x2: "6", y2: "18", stroke: "currentColor", "stroke-width": "2", "stroke-linecap": "round" }
                            line { x1: "6", y1: "6", x2: "18", y2: "18", stroke: "currentColor", "stroke-width": "2", "stroke-linecap": "round" }
                        }
                    }
                }
            } else {
                input {
                    class: "form-input",
                    placeholder: "Buscar estudiante por nombre o RUT...",
                    value: "{query}",
                    autocomplete: "off",
                    oninput: move |evt| {
                        query.set(evt.value());
                        is_open.set(true);
                    },
                    onfocus: move |_| is_open.set(true),
                    onblur: move |_| is_open.set(false),
                }
            }
            if show_results {
                div { class: "search-results",
                    if items.is_empty() {
                        div { class: "searchable-select-empty", "Sin resultados" }
                    } else {
                        {items.iter().map(|s| {
                            let sid = s["id"].as_str().unwrap_or("").to_string();
                            let sname = format!("{} {}",
                                s["first_name"].as_str().unwrap_or(""),
                                s["last_name"].as_str().unwrap_or("")
                            );
                            let srut = s["rut"].as_str().unwrap_or("").to_string();
                            let sid_clone = sid.clone();
                            let sname_clone = sname.clone();
                            rsx! {
                                div {
                                    class: "searchable-select-item",
                                    onmousedown: move |_| {
                                        selected_id.set(sid_clone.clone());
                                        selected_name.set(sname_clone.clone());
                                        is_open.set(false);
                                        query.set(String::new());
                                        on_select.call(sid_clone.clone());
                                    },
                                    div { class: "search-result-name", "{sname}" }
                                    div { class: "search-result-rut", "{srut}" }
                                }
                            }
                        })}
                    }
                }
            }
        }
    }
}
