use dioxus::prelude::*;
use wasm_bindgen::JsCast;

use super::quick_search::QuickSearch;
use crate::api::client;
use crate::components::widgets::icon::Icon;
use crate::route::has_token;

#[component]
pub fn Topbar() -> Element {
    let mut show_search = use_signal(|| false);

    let search_open = show_search;
    let handler_ref = use_signal(|| None::<wasm_bindgen::closure::Closure<dyn FnMut(_)>>);
    use_hook(move || {
        let window = web_sys::window().expect("no window");
        let doc = window.document().expect("no document");
        let mut open = search_open;
        let handler =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
                if (e.meta_key() || e.ctrl_key()) && e.key() == "k" {
                    e.prevent_default();
                    open.set(true);
                }
            }) as Box<dyn FnMut(_)>);
        let _ = doc.add_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref());
        let mut hr = handler_ref.clone();
        hr.set(Some(handler));
        0u32
    });

    let open_search = move |_| {
        show_search.set(true);
    };

    let unread = use_resource(|| async {
        if !has_token() {
            return Ok(serde_json::json!({"unread": 0}));
        }
        client::fetch_json("/api/communications/messages/unread-count").await
    });

    let unread_count: i64 = match unread() {
        Some(Ok(data)) => data["unread"].as_i64().unwrap_or(0),
        _ => 0,
    };

    rsx! {
        header { class: "topbar",
            div { class: "search-bar", onclick: open_search, role: "button", tabindex: "0", "aria-label": "Buscar alumnos y empleados", onkeydown: move |e| { if e.key() == Key::Enter || e.key() == Key::Character(" ".to_string()) { show_search.set(true); } },
                span { class: "search-icon",
                    Icon { name: "search", size: 16 }
                }
                span { class: "search-placeholder", "Buscar alumnos, empleados... (Ctrl+K)" }
                div { class: "search-shortcut",
                    kbd { "Ctrl" }
                    kbd { "K" }
                }
            }
            div { class: "topbar-actions",
                button { class: "notif-btn", onclick: move |_| { let nav = navigator(); nav.push("/notifications"); },
                    Icon { name: "bell" }
                    if unread_count > 0 {
                        div { class: "notif-badge", "{unread_count}" }
                    }
                }
                div { class: "user-avatar", "AD" }
            }
        }
        if show_search() {
            QuickSearch { is_open: show_search }
        }
    }
}
