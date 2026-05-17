use dioxus::prelude::*;
use serde_json::Value;
use wasm_bindgen::prelude::*;

use crate::api::client;
use crate::components::widgets::icon::Icon;

#[component]
pub fn SearchableSelect(
    fetch_url: String,
    results_key: String,
    label_key: String,
    value_key: String,
    placeholder: String,
    on_select: EventHandler<String>,
    initial_label: Option<String>,
) -> Element {
    let mut query = use_signal(|| String::new());
    let debounced_query = use_signal(|| String::new());
    let mut debounce_handle = use_signal(|| None::<i32>);
    let mut is_open = use_signal(|| false);
    let init_val = initial_label.clone().unwrap_or_default();
    let mut selected_label = use_signal(|| init_val);
    let init_some = initial_label.is_some();
    let mut has_selected = use_signal(|| init_some);

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

    let results = use_resource(move || {
        let q = debounced_query();
        let url = fetch_url.clone();
        let rk = results_key.clone();
        async move {
            if q.len() < 1 {
                return Vec::<Value>::new();
            }
            let separator = if url.contains('?') { "&" } else { "?" };
            match client::fetch_json(&format!("{}{}search={}", url, separator, q)).await {
                Ok(data) => data[&rk].as_array().cloned().unwrap_or_default(),
                Err(_) => Vec::new(),
            }
        }
    });

    let items = results().unwrap_or_default();
    let display_val = if has_selected() {
        selected_label()
    } else {
        query()
    };

    let f_label_key = label_key.clone();
    let f_value_key = value_key.clone();

    let rendered_items: Vec<_> = items
        .iter()
        .map(|item| {
            let label = item
                .get(&f_label_key)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let value = item
                .get(&f_value_key)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let label_c = label.clone();
            let value_c = value.clone();
            let key = value.clone();
            rsx! {
                div {
                    class: "searchable-select-item",
                    key: "{key}",
                    onmousedown: move |_| {
                        selected_label.set(label_c.clone());
                        has_selected.set(true);
                        is_open.set(false);
                        query.set("".to_string());
                        on_select.call(value_c.clone());
                    },
                    "{label}"
                }
            }
        })
        .collect();

    rsx! {
        div { class: "searchable-select",
            input {
                class: "form-input",
                r#type: "text",
                placeholder: "{placeholder}",
                value: "{display_val}",
                oninput: move |evt: FormEvent| {
                    query.set(evt.value());
                    is_open.set(true);
                    has_selected.set(false);
                },
                onfocus: move |_| if !has_selected() { is_open.set(true); },
                onblur: move |_| is_open.set(false),
                autocomplete: "off",
            }
            if has_selected() {
                span {
                    class: "searchable-select-clear",
                    onclick: move |_| {
                        selected_label.set("".to_string());
                        has_selected.set(false);
                        on_select.call("".to_string());
                    },
                    Icon { name: "x", size: 14 }
                }
            }
            if is_open() && query().len() >= 1 {
                div { class: "searchable-select-dropdown",
                    if items.is_empty() {
                        div { class: "searchable-select-empty", "Sin resultados" }
                    } else {
                        {rendered_items.into_iter()}
                    }
                }
            }
        }
    }
}
