use dioxus::prelude::*;
use crate::api::client;
use crate::seo::use_page_title;

#[component]
pub fn ParentMeetingsPage() -> Element {
    use_page_title("Reuniones Apoderados");
    let meetings = use_resource(client::fetch_meetings);
    let general = use_resource(client::fetch_general_meetings);

    let mut tab = use_signal(|| 0u32);

    let mut show_create = use_signal(|| false);
    let mut meeting_student = use_signal(String::new);
    let mut meeting_date = use_signal(String::new);
    let mut meeting_time = use_signal(String::new);
    let mut meeting_location = use_signal(String::new);
    let mut meeting_reason = use_signal(String::new);
    let mut saving = use_signal(|| false);

    let mut show_create_general = use_signal(|| false);
    let mut general_title = use_signal(String::new);
    let mut general_date = use_signal(String::new);
    let mut general_time = use_signal(String::new);
    let mut general_location = use_signal(String::new);
    let mut general_description = use_signal(String::new);
    let mut saving_general = use_signal(|| false);

    let mut reset_form = move || {
        meeting_student.set(String::new());
        meeting_date.set(String::new());
        meeting_time.set(String::new());
        meeting_location.set(String::new());
        meeting_reason.set(String::new());
        show_create.set(false);
    };

    let mut reset_general_form = move || {
        general_title.set(String::new());
        general_date.set(String::new());
        general_time.set(String::new());
        general_location.set(String::new());
        general_description.set(String::new());
        show_create_general.set(false);
    };

    let do_create = move |_| {
        saving.set(true);
        let payload = serde_json::json!({
            "student": meeting_student(),
            "date": meeting_date(),
            "time": meeting_time(),
            "location": meeting_location(),
            "reason": meeting_reason(),
        });
        let mut meetings = meetings.clone();
        spawn(async move {
            let _ = client::create_meeting(&payload).await;
            saving.set(false);
            meetings.restart();
        });
    };

    let do_create_general = move |_| {
        saving_general.set(true);
        let payload = serde_json::json!({
            "title": general_title(),
            "date": general_date(),
            "time": general_time(),
            "location": general_location(),
            "description": general_description(),
        });
        let mut general = general.clone();
        spawn(async move {
            let _ = client::create_general_meeting(&payload).await;
            saving_general.set(false);
            general.restart();
        });
    };

    rsx! {
        div { class: "page-header",
            h1 { "Reuniones de Apoderados" }
            p { "Reuniones individuales y generales con minutas" }
        }
        div { class: "tabs-header",
            TabButton { label: "Reuniones Individuales", active: tab() == 0, onclick: move |_| tab.set(0) }
            TabButton { label: "Reuniones Generales", active: tab() == 1, onclick: move |_| tab.set(1) }
            TabButton { label: "Minutas", active: tab() == 2, onclick: move |_| tab.set(2) }
        }
        if tab() == 0 {
            div { class: "page-toolbar",
                button { class: "btn btn-primary", onclick: move |_| { reset_form(); show_create.set(true); },
                    "Nueva Reunión"
                }
            }
            if show_create() {
                div { class: "card form-card",
                    h3 { "Nueva Reunión Individual" }
                    div { class: "form-grid",
                        div { class: "field",
                            label { "Estudiante" }
                            input { class: "form-input", value: "{meeting_student}", placeholder: "Nombre del estudiante",
                                oninput: move |e| meeting_student.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { "Fecha" }
                            input { class: "form-input", value: "{meeting_date}", placeholder: "2025-03-15",
                                oninput: move |e| meeting_date.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { "Hora" }
                            input { class: "form-input", value: "{meeting_time}", placeholder: "10:00",
                                oninput: move |e| meeting_time.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { "Lugar" }
                            input { class: "form-input", value: "{meeting_location}", placeholder: "Oficina",
                                oninput: move |e| meeting_location.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { "Motivo" }
                            input { class: "form-input", value: "{meeting_reason}", placeholder: "Rendimiento académico",
                                oninput: move |e| meeting_reason.set(e.value()),
                            }
                        }
                    }
                    div { class: "form-actions",
                        button { class: "btn-secondary", onclick: move |_| reset_form(), "Cancelar" }
                        button { class: "btn-primary", onclick: do_create, disabled: saving(),
                            if saving() { "Creando..." } else { "Crear" }
                        }
                    }
                }
            }
            match meetings() {
                Some(Ok(data)) => {
                    let list = data["meetings"].as_array().cloned().unwrap_or_default();
                    let meetings_cl = meetings.clone();
                    let rows: Vec<Element> = list.iter().map(|m| {
                        let mid = m["id"].as_str().unwrap_or("").to_string();
                        let student = m["student"].as_str().unwrap_or("").to_string();
                        let date = m["date"].as_str().unwrap_or("").to_string();
                        let time = m["time"].as_str().unwrap_or("").to_string();
                        let status = m["status"].as_str().unwrap_or("").to_string();
                        let location = m["location"].as_str().unwrap_or("").to_string();
                        let meetings_r = meetings_cl.clone();
                        let on_cancel = move |_| {
                            let id = mid.clone();
                            let mut r = meetings_r.clone();
                            spawn(async move {
                                let _ = client::cancel_meeting(&id).await;
                                r.restart();
                            });
                        };
                        rsx! {
                            div { class: "alert-item",
                                div { class: "alert-info",
                                    div { class: "alert-name", "{student}" }
                                    div { class: "alert-detail", "{date} {time} — {location} ({status})" }
                                }
                                if status != "cancelada" {
                                    button { class: "btn btn-sm btn-danger", onclick: on_cancel, "Cancelar" }
                                }
                            }
                        }
                    }).collect();
                    rsx! {
                        div { class: "widget-card",
                            div { class: "widget-card-header",
                                h3 { "Reuniones con Apoderados" }
                                span { "{list.len()} reuniones" }
                            }
                            div { class: "widget-card-body",
                                if rows.is_empty() {
                                    div { class: "empty-state", "Sin reuniones agendadas" }
                                } else {
                                    {rows.into_iter()}
                                }
                            }
                        }
                    }
                }
                _ => rsx! { div { class: "loading-spinner", "Cargando..." } }
            }
        }
        if tab() == 1 {
            div { class: "page-toolbar",
                button { class: "btn btn-primary", onclick: move |_| { reset_general_form(); show_create_general.set(true); },
                    "Nueva Reunión General"
                }
            }
            if show_create_general() {
                div { class: "card form-card",
                    h3 { "Nueva Reunión General" }
                    div { class: "form-grid",
                        div { class: "field",
                            label { "Título" }
                            input { class: "form-input", value: "{general_title}", placeholder: "Reunión de Apoderados",
                                oninput: move |e| general_title.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { "Fecha" }
                            input { class: "form-input", value: "{general_date}", placeholder: "2025-03-15",
                                oninput: move |e| general_date.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { "Hora" }
                            input { class: "form-input", value: "{general_time}", placeholder: "18:00",
                                oninput: move |e| general_time.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { "Lugar" }
                            input { class: "form-input", value: "{general_location}", placeholder: "Salón Auditorio",
                                oninput: move |e| general_location.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { "Descripción" }
                            input { class: "form-input", value: "{general_description}", placeholder: "Información...",
                                oninput: move |e| general_description.set(e.value()),
                            }
                        }
                    }
                    div { class: "form-actions",
                        button { class: "btn-secondary", onclick: move |_| reset_general_form(), "Cancelar" }
                        button { class: "btn-primary", onclick: do_create_general, disabled: saving_general(),
                            if saving_general() { "Creando..." } else { "Crear" }
                        }
                    }
                }
            }
            match general() {
                Some(Ok(data)) => {
                    let list = data["meetings"].as_array().cloned().unwrap_or_default();
                    let rows: Vec<Element> = list.iter().map(|m| {
                        let title = m["title"].as_str().unwrap_or("").to_string();
                        let date = m["date"].as_str().unwrap_or("").to_string();
                        let time = m["time"].as_str().unwrap_or("").to_string();
                        let location = m["location"].as_str().unwrap_or("").to_string();
                        let desc = m["description"].as_str().unwrap_or("").to_string();
                        rsx! {
                            div { class: "alert-item",
                                div { class: "alert-info",
                                    div { class: "alert-name", "{title}" }
                                    div { class: "alert-detail", "{date} {time} — {location}" }
                                    p { style: "font-size: 13px; color: var(--text-secondary); margin-top: 2px;", "{desc}" }
                                }
                            }
                        }
                    }).collect();
                    rsx! {
                        div { class: "widget-card",
                            div { class: "widget-card-header",
                                h3 { "Reuniones Generales" }
                                span { "{list.len()} reuniones" }
                            }
                            div { class: "widget-card-body",
                                if rows.is_empty() {
                                    div { class: "empty-state", "Sin reuniones generales agendadas" }
                                } else {
                                    {rows.into_iter()}
                                }
                            }
                        }
                    }
                }
                _ => rsx! { div { class: "loading-spinner", "Cargando..." } }
            }
        }
        if tab() == 2 {
            match general() {
                Some(Ok(data)) => {
                    let list = data["meetings"].as_array().cloned().unwrap_or_default();
                    let rows: Vec<Element> = list.iter().map(|m| {
                        let mid = m["id"].as_str().unwrap_or("").to_string();
                        let title = m["title"].as_str().unwrap_or("").to_string();
                        let date = m["date"].as_str().unwrap_or("").to_string();
                        rsx! {
                            MinutesCard { meeting_id: "{mid}", title: "{title}", date: "{date}" }
                        }
                    }).collect();
                    rsx! {
                        div { class: "widget-card",
                            div { class: "widget-card-header",
                                h3 { "Minutas de Reuniones" }
                            }
                            div { class: "widget-card-body",
                                if rows.is_empty() {
                                    div { class: "empty-state", "Sin reuniones generales" }
                                } else {
                                    {rows.into_iter()}
                                }
                            }
                        }
                    }
                }
                _ => rsx! { div { class: "loading-spinner", "Cargando..." } }
            }
        }
    }
}

#[component]
fn TabButton(label: String, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let cls = if active { "tab tab-active" } else { "tab" };
    rsx! {
        button { class: "{cls}", onclick: move |ev| onclick.call(ev), "{label}" }
    }
}

#[component]
fn MinutesCard(meeting_id: String, title: String, date: String) -> Element {
    let mid = meeting_id.clone();
    let minutes = use_resource(move || {
        let id = mid.clone();
        async move { client::fetch_meeting_minutes(&id).await }
    });
    let mut content = use_signal(String::new);
    let mut saved = use_signal(|| false);

    rsx! {
        div { class: "widget-card", style: "margin-bottom: 8px;",
            div { class: "widget-card-body",
                h4 { "{title} — {date}" }
                match minutes() {
                    Some(Ok(data)) => {
                        let existing = data["content"].as_str().unwrap_or("").to_string();
                        if content().is_empty() && !existing.is_empty() {
                            content.set(existing);
                        }
                        rsx! {
                            textarea {
                                class: "form-input",
                                style: "width: 100%; min-height: 100px;",
                                value: "{content}",
                                oninput: move |e| content.set(e.value()),
                                placeholder: "Escribe la minuta aquí..."
                            }
                            div { style: "margin-top: 8px;",
                                button {
                                    class: "btn-primary btn-sm",
                                    onclick: move |_| {
                                        let c = content();
                                        let id = meeting_id.clone();
                                        spawn(async move {
                                            let _ = client::save_meeting_minutes(&id, &serde_json::json!({"content": c})).await;
                                            saved.set(true);
                                        });
                                    },
                                    "Guardar Minuta"
                                }
                                if saved() {
                                    span { style: "color: var(--success); margin-left: 8px; font-size: 13px;", "Guardado" }
                                }
                            }
                        }
                    }
                    _ => rsx! { div { class: "loading-spinner", "Cargando..." } }
                }
            }
        }
    }
}
