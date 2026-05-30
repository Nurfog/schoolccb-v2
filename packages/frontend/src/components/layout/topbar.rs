use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::window;

use super::quick_search::QuickSearch;
use crate::api::client;
use crate::components::widgets::icon::Icon;
use crate::route::has_token;

const ROUTE_LABELS: &[(&str, &str, &str)] = &[
    ("dashboard", "Dashboard", "🏠"),
    ("sostenedor", "Portal Sostenedor", "🏢"),
    ("students", "Alumnos", "👤"),
    ("attendance", "Asistencia", "✅"),
    ("grades", "Calificaciones", "📊"),
    ("notifications", "Notificaciones", "🔔"),
    ("reports", "Reportes", "📄"),
    ("finance", "Finanzas", "💰"),
    ("users", "Usuarios", "👥"),
    ("courses", "Cursos", "📚"),
    ("enrollments", "Matrículas", "📝"),
    ("subjects", "Asignaturas", "📖"),
    ("config", "Configuración", "⚙️"),
    ("admission", "Admisión", "🎓"),
    ("hr", "RRHH", "👔"),
    ("import", "Importar", "📥"),
    ("corporations", "Corporaciones", "🏛️"),
    ("agenda", "Agenda", "📅"),
    ("academic-years", "Años Académicos", "📆"),
    ("academic-calendar", "Calendario Académico", "🗓️"),
    ("audit", "Auditoría", "📋"),
    ("grade-levels", "Niveles", "📐"),
    ("roles", "Roles", "🔐"),
    ("classrooms", "Salas", "🚪"),
    ("payroll", "Remuneraciones", "💵"),
    ("license-portal", "Portal Licencia", "🔑"),
    ("my-portal", "Mi Portal", "👤"),
    ("parent-portal", "Portal Apoderado", "👨‍👩‍👧"),
    ("parent-meetings", "Reuniones", "🤝"),
    ("student-portal", "Portal Alumno", "🧑‍🎓"),
    ("teacher-schedules", "Horarios", "⏰"),
    ("sige", "SIGE", "📡"),
    ("complaints", "Denuncias", "⚠️"),
    ("complementary-subjects", "Complementarias", "➕"),
    ("curriculum", "Currículum Nacional", "📗"),
    ("sales", "CRM Ventas", "💼"),
];

fn get_breadcrumbs(path: &str) -> Vec<(String, String)> {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let mut crumbs = vec![("Dashboard".to_string(), "/dashboard".to_string())];
    if segments.is_empty() {
        return crumbs;
    }
    let current = segments[0];
    let label = ROUTE_LABELS
        .iter()
        .find(|(route, _, _)| *route == current)
        .map(|(_, label, _)| label.to_string())
        .unwrap_or_else(|| {
            current.replace('-', " ")
                .split(' ')
                .map(|w| {
                    let mut c = w.chars();
                    c.next().map(|f| f.to_uppercase().to_string() + c.as_str()).unwrap_or_default()
                })
                .collect::<Vec<_>>()
                .join(" ")
        });
    crumbs.push((label, format!("/{}", current)));
    crumbs
}

fn current_path() -> String {
    window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_default()
}

#[component]
pub fn Topbar() -> Element {
    let mut show_search = use_signal(|| false);
    let path = use_signal(|| get_breadcrumbs(&current_path()));

    {
        let mut p = path.clone();
        use_effect(move || {
            let new_path = current_path();
            let crumbs = get_breadcrumbs(&new_path);
            p.set(crumbs);
        });
    }

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

    let nav = use_navigator();

    let crumbs: Vec<(String, String)> = path();
    let crumb_len = crumbs.len();

    rsx! {
        header { class: "topbar",
            div { class: "topbar-breadcrumb",
                nav { class: "breadcrumb", aria_label: "Navegación",
                    {crumbs.into_iter().enumerate().map(move |(i, (label, route))| {
                        let is_last = i == crumb_len - 1;
                        let item_class = if is_last { "breadcrumb-item active" } else { "breadcrumb-item" };
                        let r = route.clone();
                        rsx! {
                            div { class: "{item_class}",
                                if i > 0 {
                                    span { class: "breadcrumb-separator", "/" }
                                }
                                if !is_last {
                                    a {
                                        href: "{r}",
                                        onclick: move |ev| {
                                            ev.prevent_default();
                                            nav.push(r.as_str());
                                        },
                                        "{label}"
                                    }
                                } else {
                                    span { "{label}" }
                                }
                            }
                        }
                    })}
                }
            }
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
