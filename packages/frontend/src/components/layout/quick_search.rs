use std::rc::Rc;

use dioxus::prelude::*;
use serde_json::Value;
use wasm_bindgen::prelude::*;

use crate::api::client;
use crate::components::widgets::icon::Icon;

#[component]
pub fn QuickSearch(is_open: Signal<bool>) -> Element {
    let mut query = use_signal(|| String::new());
    let debounced_query = use_signal(|| String::new());
    let mut debounce_handle = use_signal(|| None::<i32>);
    let mut results = use_signal(|| Vec::<Value>::new());
    let mut loading = use_signal(|| false);
    let mut selected_idx = use_signal(|| 0i32);

    use_effect(move || {
        let q = query();
        if q != debounced_query() {
            if let Some(h) = debounce_handle.take() {
                web_sys::window().unwrap().clear_timeout_with_handle(h);
            }
            let mut dq = debounced_query.clone();
            let cb = Closure::once(move || {
                dq.set(q);
            });
            let handle = web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    300,
                )
                .unwrap();
            cb.forget();
            debounce_handle.set(Some(handle));
        }
    });

    let _ = use_resource(move || {
        let q = debounced_query();
        async move {
            if q.len() < 2 {
                results.set(Vec::new());
                loading.set(false);
                return;
            }
            loading.set(true);
            match client::fetch_json(&format!("/api/search?q={}", urlencode(&q))).await {
                Ok(data) => {
                    let items = data["results"].as_array().cloned().unwrap_or_default();
                    results.set(items);
                }
                Err(_) => {
                    results.set(Vec::new());
                }
            }
            loading.set(false);
        }
    });

    let close = move |_: Event<MouseData>| {
        is_open.set(false);
        query.set(String::new());
        results.set(Vec::new());
    };

    let on_input = move |evt: Event<FormData>| {
        query.set(evt.value());
        selected_idx.set(0);
    };

    let on_key_down = move |evt: Event<KeyboardData>| {
        let k = evt.key();
        match k {
            Key::Escape => {
                is_open.set(false);
                query.set(String::new());
                results.set(Vec::new());
            }
            Key::ArrowDown => {
                let len = results.read().len() as i32;
                if selected_idx() < len - 1 {
                    selected_idx.set(selected_idx() + 1);
                }
            }
            Key::ArrowUp => {
                if selected_idx() > 0 {
                    selected_idx.set(selected_idx() - 1);
                }
            }
            Key::Enter => {
                let items = results.read().clone();
                if let Some(item) = items.get(selected_idx() as usize) {
                    let entity_type = item["entity_type"].as_str().unwrap_or("");
                    let id = item["id"].as_str().unwrap_or("");
                    query.set(String::new());
                    results.set(Vec::new());
                    is_open.set(false);
                    let nav = navigator();
                    let route = match entity_type {
                        "student" => format!("/students/{}", id),
                        "employee" => format!("/hr/{}", id),
                        _ => format!("/students/{}", id),
                    };
                    nav.push(route);
                }
            }
            _ => {}
        }
    };

    let loading_state = loading();
    let query_text = query();
    let items_list = results.read().clone();
    let selected = selected_idx();

    rsx! {
        div { class: "quick-search-overlay", onclick: close,
            div { class: "quick-search-modal", onclick: |e| e.stop_propagation(),
                div { class: "quick-search-input-wrap",
                    Icon { name: "search" }
                    input {
                        placeholder: "Buscar alumnos, empleados...",
                        value: "{query_text}",
                        autofocus: "true",
                        oninput: on_input,
                        onkeydown: on_key_down,
                    }
                    span { class: "esc-hint", "ESC" }
                }
                div { class: "quick-search-results",
                    if loading_state {
                        div { class: "loading-spinner", "Buscando..." }
                    } else if query_text.len() < 2 {
                        div { class: "quick-search-empty", "Escribe al menos 2 caracteres" }
                    } else {
                        SearchResultsList {
                            items: items_list,
                            selected_idx: selected,
                        }
                    }
                }
            }
        }
    }
}

fn urlencode(s: &str) -> String {
    js_sys::encode_uri_component(s).as_string().unwrap_or_else(|| s.to_string())
}

#[component]
fn SearchResultsList(items: Vec<Value>, selected_idx: i32) -> Element {
    if items.is_empty() {
        return rsx! {
            div { class: "quick-search-empty", "Sin resultados" }
        };
    }

    let rows: Vec<Element> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let item = item.clone();
            let full_name = format!(
                "{} {}",
                item["first_name"].as_str().unwrap_or(""),
                item["last_name"].as_str().unwrap_or("")
            );
            let subtitle = item["subtitle"].as_str().unwrap_or("").to_string();
            let rut = item["rut"].as_str().unwrap_or("").to_string();
            let entity_type = item["entity_type"].as_str().unwrap_or("").to_string();
            let id = item["id"].as_str().unwrap_or("").to_string();
            let initial = item["first_name"]
                .as_str()
                .and_then(|n| n.chars().next())
                .map(|c| c.to_string())
                .unwrap_or_else(|| "?".into());
            let highlighted = i == selected_idx as usize;
            let badge = if entity_type == "employee" {
                "Employee"
            } else {
                "Student"
            };
            let route = if entity_type == "employee" {
                format!("/hr/{}", id)
            } else {
                format!("/students/{}", id)
            };

            let route_rc = Rc::new(route);
            let r1 = route_rc.clone();

            rsx! {
                button {
                    class: "quick-search-result-item",
                    "data-selected": "{highlighted}",
                    onclick: {
                        let r = r1.clone();
                        move |_: Event<MouseData>| {
                            let nav = navigator();
                            nav.push((*r).clone());
                        }
                    },
                    div { class: "avatar", "{initial}" }
                    div { class: "info",
                        div { class: "name", "{full_name}" }
                        div { class: "detail", "{subtitle}" }
                    }
                    div { class: "meta",
                        span { class: "entity-badge", "{badge}" }
                        span { class: "rut-text", "{rut}" }
                    }
                }
            }
        })
        .collect();

    rsx! {
        { rows.into_iter() }
    }
}
