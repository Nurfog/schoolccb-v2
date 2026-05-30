use dioxus::prelude::*;
use crate::api::client;
use crate::seo::use_page_title;

#[component]
pub fn ParentMeetingsPage() -> Element {
    use_page_title("Reuniones Apoderados");
    let meetings = use_resource(client::fetch_meetings);
    let general = use_resource(client::fetch_general_meetings);

    let mut tab = use_signal(|| 0u32);

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
            match meetings() {
                Some(Ok(data)) => {
                    let list = data["meetings"].as_array().cloned().unwrap_or_default();
                    let rows: Vec<Element> = list.iter().map(|m| {
                        let student = m["student"].as_str().unwrap_or("").to_string();
                        let date = m["date"].as_str().unwrap_or("").to_string();
                        let time = m["time"].as_str().unwrap_or("").to_string();
                        let status = m["status"].as_str().unwrap_or("").to_string();
                        let location = m["location"].as_str().unwrap_or("").to_string();
                        rsx! {
                            div { class: "alert-item",
                                div { class: "alert-info",
                                    div { class: "alert-name", "{student}" }
                                    div { class: "alert-detail", "{date} {time} — {location} ({status})" }
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
