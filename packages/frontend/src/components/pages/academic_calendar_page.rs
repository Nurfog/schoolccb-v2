use dioxus::prelude::*;
use crate::api::client;
use crate::seo::use_page_title;

#[component]
pub fn AcademicCalendarPage() -> Element {
    use_page_title("Calendario Académico");
    let events = use_resource(client::fetch_calendar_events);
    let holidays = use_resource(client::fetch_holidays);
    let exams = use_resource(client::fetch_exams);

    let mut tab = use_signal(|| 0u32);

    rsx! {
        div { class: "page-header",
            h1 { "Calendario Académico" }
            p { "Eventos, feriados y calendario de pruebas" }
        }
        div { class: "tabs-header",
            TabButton { label: "Eventos", active: tab() == 0, onclick: move |_| tab.set(0) }
            TabButton { label: "Feriados", active: tab() == 1, onclick: move |_| tab.set(1) }
            TabButton { label: "Calendario de Pruebas", active: tab() == 2, onclick: move |_| tab.set(2) }
        }
        if tab() == 0 {
            match events() {
                Some(Ok(data)) => {
                    let list = data["events"].as_array().cloned().unwrap_or_default();
                    let rows: Vec<Element> = list.iter().map(|e| {
                        let title = e["title"].as_str().unwrap_or("").to_string();
                        let etype = e["type"].as_str().unwrap_or("").to_string();
                        let date = e["date"].as_str().unwrap_or("").to_string();
                        let time = e["time"].as_str().unwrap_or("").to_string();
                        let desc = e["description"].as_str().unwrap_or("").to_string();
                        rsx! {
                            div { class: "event-item",
                                div { class: "event-date-badge evento",
                                    span { class: "day", "{date}" }
                                    span { class: "month", "..." }
                                }
                                div { class: "event-details",
                                    div { class: "event-title", "{title}" }
                                    div { class: "event-type", "{etype} — {time}" }
                                    p { style: "font-size: 13px; color: var(--text-secondary); margin-top: 4px;", "{desc}" }
                                }
                            }
                        }
                    }).collect();
                    rsx! {
                        div { class: "widget-card",
                            div { class: "widget-card-header",
                                h3 { "Eventos Académicos" }
                                span { "{list.len()} eventos" }
                            }
                            div { class: "widget-card-body",
                                if rows.is_empty() {
                                    div { class: "empty-state", "Sin eventos registrados" }
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
            match holidays() {
                Some(Ok(data)) => {
                    let list = data["holidays"].as_array().cloned().unwrap_or_default();
                    let rows: Vec<Element> = list.iter().map(|h| {
                        let name = h["name"].as_str().unwrap_or("").to_string();
                        let date = h["date"].as_str().unwrap_or("").to_string();
                        let htype = h["type"].as_str().unwrap_or("").to_string();
                        rsx! {
                            div { class: "alert-item",
                                div { class: "alert-info",
                                    div { class: "alert-name", "{name}" }
                                    div { class: "alert-detail", "{date} — {htype}" }
                                }
                            }
                        }
                    }).collect();
                    rsx! {
                        div { class: "widget-card",
                            div { class: "widget-card-header",
                                h3 { "Feriados" }
                                span { "{list.len()} feriados" }
                            }
                            div { class: "widget-card-body",
                                if rows.is_empty() {
                                    div { class: "empty-state", "Sin feriados registrados" }
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
            match exams() {
                Some(Ok(data)) => {
                    let list = data["exams"].as_array().cloned().unwrap_or_default();
                    let rows: Vec<Element> = list.iter().map(|e| {
                        let title = e["title"].as_str().unwrap_or("").to_string();
                        let subject = e["subject"].as_str().unwrap_or("").to_string();
                        let date = e["date"].as_str().unwrap_or("").to_string();
                        let time = e["time"].as_str().unwrap_or("").to_string();
                        let responsible = e["responsible"].as_str().unwrap_or("").to_string();
                        rsx! {
                            div { class: "event-item",
                                div { class: "event-date-badge evaluacion",
                                    span { class: "day", "{date}" }
                                    span { class: "month", "..." }
                                }
                                div { class: "event-details",
                                    div { class: "event-title", "{title}" }
                                    div { class: "event-type", "{subject} — {time}" }
                                    p { style: "font-size: 13px; color: var(--text-secondary); margin-top: 4px;", "Responsable: {responsible}" }
                                }
                            }
                        }
                    }).collect();
                    rsx! {
                        div { class: "widget-card",
                            div { class: "widget-card-header",
                                h3 { "Calendario de Pruebas" }
                                span { "{list.len()} pruebas" }
                            }
                            div { class: "widget-card-body",
                                if rows.is_empty() {
                                    div { class: "empty-state", "Sin pruebas registradas" }
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
