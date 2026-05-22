use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client;

fn jwt_claims() -> Option<Value> {
    let window = web_sys::window()?;
    let doc = window.document()?;
    let cookie = js_sys::Reflect::get(&doc, &wasm_bindgen::JsValue::from_str("cookie"))
        .ok()
        .and_then(|v| v.as_string())?;
    let token = cookie.split(';').find_map(|c| {
        let c = c.trim();
        c.strip_prefix("jwt_token=").map(|v| v.to_string())
    })?;
    let parts: Vec<&str> = token.split('.').collect();
    let payload_b64 = parts.get(1)?;
    let decoded = window.atob(payload_b64).ok()?;
    serde_json::from_str(&decoded).ok()
}

fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

fn role_label(role: &str) -> &'static str {
    match role {
        "Root" => "Root Admin",
        "Sostenedor" => "Sostenedor",
        "Administrador" => "Admin",
        "Director" => "Director",
        "UTP" => "UTP",
        "Profesor" => "Profesor",
        "Apoderado" => "Apoderado",
        "Alumno" => "Alumno",
        _ => "Usuario",
    }
}

#[component]
pub fn Sidebar() -> Element {
    let claims = use_signal(jwt_claims);
    let user_name = claims()
        .as_ref()
        .and_then(|c| c["name"].as_str())
        .unwrap_or("Usuario")
        .to_string();
    let user_role = claims()
        .as_ref()
        .and_then(|c| c["role"].as_str())
        .unwrap_or("")
        .to_string();
    let user_initials = initials(&user_name);

    let fav_ver = use_context::<Signal<u32>>();
    let modules = use_resource(move || async move {
        let _ = fav_ver();
        client::fetch_json("/api/user/modules").await
    });

    let mut sidebar_open = use_signal(|| false);
    let toggle_sidebar = move |_| sidebar_open.set(!sidebar_open());
    let close_sidebar = move |_| sidebar_open.set(false);

    let sidebar_class = if sidebar_open() { "sidebar open" } else { "sidebar" };

    let current_path = web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_default();
    let is_active = move |path: &str| current_path == path;

    rsx! {
        button { class: "sidebar-toggle", "aria-label": "Abrir menu de navegacion", "aria-expanded": "{sidebar_open()}", onclick: toggle_sidebar,
            svg { role: "presentation", view_box: "0 0 24 24", width: "24", height: "24",
                path { d: "M3 12h18M3 6h18M3 18h18" }
            }
        }
        { if sidebar_open() {
            rsx! { div { class: "sidebar-overlay", onclick: close_sidebar, role: "presentation" } }
        } else { rsx! {} }}
        nav { class: "{sidebar_class}", role: "navigation", aria_label: "Navegación principal",
            div { class: "sidebar-header",
                img { class: "logo-img", src: "/logo-white.svg", alt: "SchoolCBB", width: "140", height: "36" }
            }

            div { class: "sidebar-nav",
                div { class: "sidebar-user",
                    div { class: "user-avatar-sidebar", "{user_initials}" }
                    div { class: "user-info",
                        span { class: "user-name", "{user_name}" }
                        span { class: "user-role", "{role_label(&user_role)}" }
                    }
                }

                a { class: "nav-item", href: "/dashboard", aria_current: is_active("/dashboard").then_some("page"),
                    span { class: "icon",
                        svg { role: "presentation", view_box: "0 0 24 24",
                            rect { x: "3", y: "3", width: "7", height: "7", rx: "1" }
                            rect { x: "14", y: "3", width: "7", height: "7", rx: "1" }
                            rect { x: "3", y: "14", width: "7", height: "7", rx: "1" }
                            rect { x: "14", y: "14", width: "7", height: "7", rx: "1" }
                        }
                    }
                    span { class: "label", "Dashboard" }
                }

                {if user_role == "Root" {
                    rsx! {
                        a { class: "nav-item", href: "/root", aria_current: is_active("/root").then_some("page"),
                            span { class: "icon",
                                svg { role: "presentation", view_box: "0 0 24 24",
                                    path { d: "M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z" }
                                }
                            }
                            span { class: "label", "Panel Root" }
                        }
                    }
                } else { rsx! {} }}

                div { class: "nav-section-label", "Acceso Rápido"}
                div { class: "sidebar-favs",
                    match modules() {
                        Some(Ok(data)) => {
                            let all = data["modules"].as_array().cloned().unwrap_or_default();
                            let fav_ids: std::collections::HashSet<String> = all.iter()
                                .filter(|m| m["is_favorite"].as_bool().unwrap_or(false))
                                .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                                .collect();
                            let parents: Vec<&Value> = all.iter().filter(|m| {
                                let id = m["id"].as_str().unwrap_or("");
                                fav_ids.contains(id) && m["parent"].is_null()
                            }).collect();
                            let children: Vec<&Value> = all.iter().filter(|m| {
                                let id = m["id"].as_str().unwrap_or("");
                                fav_ids.contains(id) && m["parent"].is_string()
                            }).collect();
                            if parents.is_empty() && children.is_empty() {
                                rsx! { p { class: "empty-hint", "Sin favoritos" } }
                            } else {
                                let items: Vec<_> = parents.into_iter().map(|p| {
                                    let pid = p["id"].as_str().unwrap_or("").to_string();
                                    let pname = p["name"].as_str().unwrap_or("").to_string();
                                    let proute = p["route"].as_str().unwrap_or("/").to_string();
                                    let sub: Vec<&Value> = children.iter().filter(|c| {
                                        c["parent"].as_str() == Some(&pid)
                                    }).copied().collect();
                                    if sub.is_empty() {
                                        rsx! {
                                            a { key: "{pid}", class: "fav-link", href: "{proute}",
                                                span { class: "fav-dot" }
                                                span { "{pname}" }
                                            }
                                        }
                                    } else {
                                        let sub_items: Vec<_> = sub.into_iter().map(|c| {
                                            let cid = c["id"].as_str().unwrap_or("").to_string();
                                            let cname = c["name"].as_str().unwrap_or("").to_string();
                                            let croute = c["route"].as_str().unwrap_or("/").to_string();
                                            rsx! {
                                                a { key: "{cid}", class: "fav-sub-link", href: "{croute}",
                                                    span { "{cname}" }
                                                }
                                            }
                                        }).collect();
                                        rsx! {
                                            div { key: "{pid}", class: "fav-group",
                                                a { class: "fav-link", href: "{proute}",
                                                    span { class: "fav-dot" }
                                                    span { "{pname}" }
                                                }
                                                div { class: "fav-submenu",
                                                    {sub_items.into_iter()}
                                                }
                                            }
                                        }
                                    }
                                }).collect();
                                rsx! { {items.into_iter()} }
                            }
                        }
                        Some(Err(e)) => rsx! { p { class: "empty-hint", "Error: {e}" } },
                        None => rsx! { p { class: "empty-hint", "Cargando..." } },
                    }
                }

            }

            div { class: "sidebar-footer",
                div { class: "nav-section-label", "Sistema"}

                {if user_role == "Sostenedor" {
                    rsx! {
                        a { class: "nav-item config-item", href: "/sostenedor", aria_current: is_active("/sostenedor").then_some("page"),
                            span { class: "icon",
                                svg { role: "presentation", view_box: "0 0 24 24",
                                    path { d: "M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" }
                                    polyline { points: "9 22 9 12 15 12 15 22" }
                                }
                            }
                            span { class: "label", "Panel Sostenedor" }
                        }
                    }
                } else { rsx! {} }}

                {if user_role != "Root" {
                    rsx! {
                        a { class: "nav-item config-item", href: "/curriculum", aria_current: is_active("/curriculum").then_some("page"),
                            span { class: "icon",
                                svg { role: "presentation", view_box: "0 0 24 24",
                                    path { d: "M4 19.5A2.5 2.5 0 0 1 6.5 17H20" }
                                    path { d: "M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" }
                                    line { x1: "8", y1: "7", x2: "16", y2: "7" }
                                    line { x1: "8", y1: "11", x2: "14", y2: "11" }
                                }
                            }
                            span { class: "label", "Currículum Nacional" }
                        }
                    }
                } else { rsx! {} }}

                a { class: "nav-item config-item", href: "/", aria_current: is_active("/").then_some("page"),
                    span { class: "icon",
                        svg { role: "presentation", view_box: "0 0 24 24",
                            polygon { points: "12 2 2 7 12 12 22 7 12 2" }
                            polyline { points: "2 12 12 17 22 12" }
                            polyline { points: "2 17 12 22 22 17" }
                        }
                    }
                    span { class: "label", "Module Manager" }
                }

                a { class: "nav-item config-item", href: "/config", aria_current: is_active("/config").then_some("page"),
                    span { class: "icon",
                        svg { role: "presentation", view_box: "0 0 24 24",
                            path { d: "M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z" }
                        }
                    }
                    span { class: "label", "Configuración" }
                }



                button { class: "nav-item logout", onclick: move |_| {
                        let nav = navigator();
                        spawn(async move {
                            let _ = client::logout().await;
                            client::remove_token();
                            nav.replace("/login");
                        });
                    },
                    span { class: "icon",
                        svg { role: "presentation", view_box: "0 0 24 24",
                            path { d: "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" }
                            polyline { points: "16 17 21 12 16 7" }
                            line { x1: "21", y1: "12", x2: "9", y2: "12" }
                        }
                    }
                    span { class: "label", "Cerrar Sesión" }
                }
            }
        }
    }
}
