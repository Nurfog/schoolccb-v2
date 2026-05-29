use dioxus::prelude::*;

use crate::api::client;

#[component]
pub fn AdminSystemPage() -> Element {
    let health = use_resource(|| client::admin_system_health());
    let activity = use_resource(|| client::admin_activity_log());

    rsx! {
        div { class: "page-header",
            h1 { "Sistema" }
            p { "Monitoreo y salud de los servicios" }
        }
        div { class: "section-card",
            h3 { "Estado de Servicios" }
            match health() {
                Some(Ok(data)) => {
                    let services = data["services"].as_object().cloned().unwrap_or_default();
                    if services.is_empty() {
                        rsx! { p { class: "empty-state", "Sin informaci\u{00f3}n de servicios" } }
                    } else {
                        rsx! {
                            div { class: "health-grid",
                                {services.into_iter().map(|(name, status)| {
                                    let ok = status.as_str().map(|s| s == "ok" || s == "healthy").unwrap_or(false);
                                    let cls = if ok { "health-ok" } else { "health-fail" };
                                    let label = if ok { "✅" } else { "❌" };
                                    rsx! {
                                        div { key: "{name}", class: "health-item {cls}",
                                            span { class: "health-icon", "{label}" }
                                            span { class: "health-name", "{name}" }
                                        }
                                    }
                                })}
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! { p { class: "state-error", "Error: {e}" } },
                None => rsx! { div { class: "loading-spinner", "Cargando..." } },
            }
        }
        div { class: "section-card",
            h3 { "Actividad Reciente" }
            match activity() {
                Some(Ok(data)) => {
                    let list = data["activity_log"].as_array().cloned().unwrap_or_default();
                    if list.is_empty() {
                        rsx! { p { class: "empty-state", "Sin actividad registrada" } }
                    } else {
                        rsx! {
                            table { class: "data-table",
                                thead {
                                    tr {
                                        th { "Fecha" } th { "Admin" } th { "Acci\u{00f3}n" } th { "Entidad" }
                                    }
                                }
                                tbody {
                                    {list.into_iter().map(|a| {
                                        let admin = a["admin"].as_str().unwrap_or("—").to_string();
                                        let action = a["action"].as_str().unwrap_or("—").to_string();
                                        let entity = a["entity_type"].as_str().unwrap_or("—").to_string();
                                        let created = a["created_at"].as_str().unwrap_or("—").to_string();
                                        rsx! {
                                            tr {
                                                td { "{created}" } td { "{admin}" }
                                                td { span { class: "badge badge-info", "{action}" } }
                                                td { "{entity}" }
                                            }
                                        }
                                    })}
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! { p { class: "state-error", "Error: {e}" } },
                None => rsx! { div { class: "loading-spinner", "Cargando..." } },
            }
        }
    }
}
