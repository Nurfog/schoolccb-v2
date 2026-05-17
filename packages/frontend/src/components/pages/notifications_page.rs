use dioxus::prelude::*;

use crate::api::client;
use crate::components::widgets::icon::Icon;

fn format_date(date_str: &str) -> String {
    if date_str.len() >= 10 {
        let day = &date_str[8..10];
        let month = &date_str[5..7];
        let year = &date_str[0..4];
        format!("{}/{}/{}", day, month, year)
    } else {
        date_str.to_string()
    }
}

#[component]
pub fn NotificationsPage() -> Element {
    let mut messages = use_resource(|| client::fetch_json("/api/communications/messages"));
    let mut show_compose = use_signal(|| false);

    rsx! {
        div { class: "page-header",
            h1 { "Mensajes" }
            p { "Bandeja de mensajes y comunicaciones" }
        }
        div { class: "page-toolbar",
            button {
                class: "btn-primary",
                onclick: move |_| show_compose.set(true),
                "Nuevo Mensaje"
            }
        }
        if show_compose() {
            ComposeModal { is_open: show_compose, on_sent: move || messages.restart() }
        }
        div { class: "data-table-container",
            table { class: "data-table",
                thead {
                    tr {
                        th { "Asunto" }
                        th { "Estado" }
                        th { "Fecha" }
                    }
                }
                tbody {
                    match messages() {
                        Some(Ok(data)) => {
                            let list = data["messages"].as_array().cloned().unwrap_or_default();
                            if list.is_empty() {
                                rsx! { tr { td { colspan: "3", class: "empty-state", "No hay mensajes" } } }
                            } else {
                                rsx! {
                                    for msg in list {
                                        MessageRow { msg: msg }
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => rsx! { tr { td { colspan: "3", class: "empty-state", "Error: {e}" } } },
                        None => rsx! { tr { td { colspan: "3", class: "empty-state", div { class: "loading-spinner", "Cargando..." } } } },
                    }
                }
            }
        }
    }
}

#[component]
fn MessageRow(msg: serde_json::Value) -> Element {
    let subject = msg["subject"].as_str().unwrap_or("Sin asunto").to_string();
    let is_read = msg["read"].as_bool().unwrap_or(true);
    let created = msg["created_at"].as_str().unwrap_or("").to_string();
    let row_class = if !is_read { "unread" } else { "" };

    rsx! {
        tr { class: "{row_class}",
            td { class: "cell-name",
                if !is_read {
                    span { class: "unread-dot" }
                }
                span { "{subject}" }
            }
            td {
                if is_read {
                    span { class: "status-inactive", "Leído" }
                } else {
                    span { class: "status-active", "Nuevo" }
                }
            }
            td { "{format_date(&created)}" }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum AudienceType {
    User,
    Course,
    AllStudents,
    AllTeachers,
    AllStaff,
}

#[component]
fn ComposeModal(is_open: Signal<bool>, on_sent: EventHandler) -> Element {
    let mut audience_type = use_signal(|| AudienceType::AllStudents);
    let mut audience_id = use_signal(|| "".to_string());
    let mut subject = use_signal(|| "".to_string());
    let mut body = use_signal(|| "".to_string());
    let mut sending = use_signal(|| false);

    let do_send = move |_| {
        if subject().is_empty() || body().is_empty() {
            return;
        }
        sending.set(true);
        let audience = match audience_type() {
            AudienceType::User => serde_json::json!({ "type": "User", "id": audience_id() }),
            AudienceType::Course => serde_json::json!({ "type": "Course", "id": audience_id() }),
            AudienceType::AllStudents => serde_json::json!({ "type": "AllStudents" }),
            AudienceType::AllTeachers => serde_json::json!({ "type": "AllTeachers" }),
            AudienceType::AllStaff => serde_json::json!({ "type": "AllStaff" }),
        };
        let payload = serde_json::json!({
            "audience": audience,
            "subject": subject(),
            "body": body(),
        });
        spawn(async move {
            let _ = client::post_json("/api/communications/messages", &payload).await;
            sending.set(false);
            is_open.set(false);
            on_sent.call(());
        });
    };

    let needs_id = audience_type() == AudienceType::User || audience_type() == AudienceType::Course;

    rsx! {
        div { class: "quick-search-overlay",
            div { class: "quick-search-modal compose-modal",
                div { class: "quick-search-header",
                    h3 { "Nuevo Mensaje" }
                    button { class: "close-btn", onclick: move |_| is_open.set(false),
                        Icon { name: "x", size: 20 }
                    }
                }
                div { class: "compose-body",
                    div { class: "field",
                        label { "Destinatarios" }
                        select { class: "form-input", value: "{audience_type():?}", oninput: move |e| {
                            let v = e.value();
                            audience_type.set(if v == "User" { AudienceType::User }
                                else if v == "Course" { AudienceType::Course }
                                else if v == "AllTeachers" { AudienceType::AllTeachers }
                                else if v == "AllStaff" { AudienceType::AllStaff }
                                else { AudienceType::AllStudents });
                        },
                            option { value: "AllStudents", "Todos los Alumnos" }
                            option { value: "AllTeachers", "Todos los Profesores" }
                            option { value: "AllStaff", "Todo el Personal" }
                            option { value: "User", "Usuario específico" }
                            option { value: "Course", "Curso específico" }
                        }
                    }
                    if needs_id {
                        div { class: "field",
                            label { if audience_type() == AudienceType::User { "ID del Usuario" } else { "ID del Curso" } }
                            input { class: "form-input", value: "{audience_id}", placeholder: if audience_type() == AudienceType::User { "UUID del usuario" } else { "UUID del curso" },
                                oninput: move |evt| audience_id.set(evt.value()),
                            }
                        }
                    }
                    div { class: "field",
                        label { "Asunto" }
                        input { class: "form-input", value: "{subject}", placeholder: "Asunto del mensaje",
                            oninput: move |evt| subject.set(evt.value()),
                        }
                    }
                    div { class: "field",
                        label { "Mensaje" }
                        textarea { class: "form-input compose-textarea", value: "{body}", placeholder: "Escriba su mensaje...",
                            rows: "5",
                            oninput: move |evt| body.set(evt.value()),
                        }
                    }
                    button { class: "btn-primary", onclick: do_send, disabled: sending(),
                        if sending() { "Enviando..." } else { "Enviar Mensaje" }
                    }
                }
            }
        }
    }
}
