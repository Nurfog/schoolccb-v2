use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client;
use crate::components::widgets::icon::Icon;

#[component]
pub fn ModuleManager() -> Element {
    let fav_ver = use_context::<Signal<u32>>();
    let modules = use_resource(move || async move {
        let _ = fav_ver();
        client::fetch_json("/api/user/modules").await
    });
    let mut search = use_signal(String::new);

    rsx! {
        div { class: "module-manager",
            div { class: "module-search",
                input {
                    class: "module-search-input",
                    placeholder: "Buscar módulos...",
                    value: "{search}",
                    oninput: move |e| search.set(e.value()),
                }
            }
            div { class: "module-list",
                match modules() {
                    Some(Ok(data)) => {
                        let list: Vec<Value> = data["modules"].as_array().cloned().unwrap_or_default()
                            .into_iter().filter(|m| {
                                let q = search().to_lowercase();
                                q.is_empty() || m["name"].as_str().unwrap_or("").to_lowercase().contains(&q)
                            }).collect();
                        rsx! {
                            for m in list {
                                ModuleTile { module: m }
                            }
                        }
                    }
                    Some(Err(e)) => rsx! { p { class: "empty-state", "Error: {e}" } },
                    None => rsx! { div { class: "loading-spinner", "Cargando..." } },
                }
            }
        }
    }
}

#[component]
fn ModuleTile(module: Value) -> Element {
    let id = module["id"].as_str().unwrap_or("").to_string();
    let name = module["name"].as_str().unwrap_or("").to_string();
    let icon = module["icon"].as_str().unwrap_or("dashboard").to_string();
    let route = module["route"].as_str().unwrap_or("/").to_string();
    let is_fav = module["is_favorite"].as_bool().unwrap_or(false);
    let mut fav_ver = use_context::<Signal<u32>>();

    let tile_cls = format!("tile-icon {}", icon);
    let star_cls = if is_fav { "active" } else { "" };

    let do_toggle = move |evt: Event<MouseData>| {
        evt.prevent_default();
        let mid = id.clone();
        let new_fav = !is_fav;
        spawn(async move {
            let _ = client::post_json(
                &format!("/api/user/favorites/{}", mid),
                &serde_json::json!({ "module_id": mid, "favorite": new_fav }),
            )
            .await;
            fav_ver += 1;
        });
    };

    rsx! {
        a { class: "module-tile", href: "{route}",
            div { class: "{tile_cls}",
                Icon { name: "{icon}" }
            }
            span { class: "tile-name", "{name}" }
            div { class: "tile-star {star_cls}", onclick: do_toggle,
                Icon { name: "star" }
            }
        }
    }
}
