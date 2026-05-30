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

    let mut show_event_form = use_signal(|| false);
    let mut editing_event_id = use_signal(|| None::<String>);
    let mut event_title = use_signal(String::new);
    let mut event_type = use_signal(String::new);
    let mut event_date = use_signal(String::new);
    let mut event_time = use_signal(String::new);
    let mut event_description = use_signal(String::new);
    let mut saving_event = use_signal(|| false);

    let mut show_holiday_form = use_signal(|| false);
    let mut holiday_name = use_signal(String::new);
    let mut holiday_date = use_signal(String::new);
    let mut holiday_type = use_signal(String::new);
    let mut saving_holiday = use_signal(|| false);

    let mut show_exam_form = use_signal(|| false);
    let mut editing_exam_id = use_signal(|| None::<String>);
    let mut exam_title = use_signal(String::new);
    let mut exam_subject = use_signal(String::new);
    let mut exam_date = use_signal(String::new);
    let mut exam_time = use_signal(String::new);
    let mut exam_responsible = use_signal(String::new);
    let mut saving_exam = use_signal(|| false);

    let mut reset_event_form = move || {
        event_title.set(String::new());
        event_type.set(String::new());
        event_date.set(String::new());
        event_time.set(String::new());
        event_description.set(String::new());
        editing_event_id.set(None);
        show_event_form.set(false);
    };

    let mut reset_holiday_form = move || {
        holiday_name.set(String::new());
        holiday_date.set(String::new());
        holiday_type.set(String::new());
        show_holiday_form.set(false);
    };

    let mut reset_exam_form = move || {
        exam_title.set(String::new());
        exam_subject.set(String::new());
        exam_date.set(String::new());
        exam_time.set(String::new());
        exam_responsible.set(String::new());
        editing_exam_id.set(None);
        show_exam_form.set(false);
    };

    let do_save_event = move |_| {
        saving_event.set(true);
        let payload = serde_json::json!({
            "title": event_title(),
            "type": event_type(),
            "date": event_date(),
            "time": event_time(),
            "description": event_description(),
        });
        let _is_edit = editing_event_id().is_some();
        let eid = editing_event_id();
        let mut events = events.clone();
        spawn(async move {
            if let Some(ref id) = eid {
                let _ = client::update_calendar_event(id, &payload).await;
            } else {
                let _ = client::create_calendar_event(&payload).await;
            }
            saving_event.set(false);
            events.restart();
        });
    };

    let do_save_holiday = move |_| {
        saving_holiday.set(true);
        let payload = serde_json::json!({
            "name": holiday_name(),
            "date": holiday_date(),
            "type": holiday_type(),
        });
        let mut holidays = holidays.clone();
        spawn(async move {
            let _ = client::create_holiday(&payload).await;
            saving_holiday.set(false);
            holidays.restart();
        });
    };

    let do_save_exam = move |_| {
        saving_exam.set(true);
        let payload = serde_json::json!({
            "title": exam_title(),
            "subject": exam_subject(),
            "date": exam_date(),
            "time": exam_time(),
            "responsible": exam_responsible(),
        });
        let _is_edit = editing_exam_id().is_some();
        let eid = editing_exam_id();
        let mut exams = exams.clone();
        spawn(async move {
            if let Some(ref id) = eid {
                let _ = client::update_exam(id, &payload).await;
            } else {
                let _ = client::create_exam(&payload).await;
            }
            saving_exam.set(false);
            exams.restart();
        });
    };

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
            div { class: "page-toolbar",
                button { class: "btn btn-primary", onclick: move |_| { reset_event_form(); show_event_form.set(true); },
                    "Nuevo Evento"
                }
            }
            if show_event_form() {
                div { class: "card form-card",
                    h3 { if editing_event_id().is_some() { "Editar Evento" } else { "Nuevo Evento" } }
                    div { class: "form-grid",
                        div { class: "field",
                            label { "Título" }
                            input { class: "form-input", value: "{event_title}", placeholder: "Día del Patrimonio",
                                oninput: move |e| event_title.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { "Tipo" }
                            input { class: "form-input", value: "{event_type}", placeholder: "cultural",
                                oninput: move |e| event_type.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { "Fecha" }
                            input { class: "form-input", value: "{event_date}", placeholder: "2025-05-30",
                                oninput: move |e| event_date.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { "Hora" }
                            input { class: "form-input", value: "{event_time}", placeholder: "10:00",
                                oninput: move |e| event_time.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { "Descripción" }
                            input { class: "form-input", value: "{event_description}", placeholder: "Descripción...",
                                oninput: move |e| event_description.set(e.value()),
                            }
                        }
                    }
                    div { class: "form-actions",
                        button { class: "btn-secondary", onclick: move |_| reset_event_form(), "Cancelar" }
                        button { class: "btn-primary", onclick: do_save_event, disabled: saving_event(),
                            if saving_event() { "Guardando..." } else { "Guardar" }
                        }
                    }
                }
            }
            match events() {
                Some(Ok(data)) => {
                    let list = data["events"].as_array().cloned().unwrap_or_default();
                    let events_cl = events.clone();
                    let rows: Vec<Element> = list.iter().map(|e| {
                        let eid = e["id"].as_str().unwrap_or("").to_string();
                        let eid_for_edit = eid.clone();
                        let title = e["title"].as_str().unwrap_or("").to_string();
                        let title_for_edit = title.clone();
                        let etype = e["type"].as_str().unwrap_or("").to_string();
                        let etype_for_edit = etype.clone();
                        let date = e["date"].as_str().unwrap_or("").to_string();
                        let date_for_edit = date.clone();
                        let time = e["time"].as_str().unwrap_or("").to_string();
                        let time_for_edit = time.clone();
                        let desc = e["description"].as_str().unwrap_or("").to_string();
                        let desc_for_edit = desc.clone();
                        let events_r = events_cl.clone();
                        let on_delete = move |_| {
                            let id = eid.clone();
                            let mut r = events_r.clone();
                            spawn(async move {
                                let _ = client::delete_calendar_event(&id).await;
                                r.restart();
                            });
                        };
                        let on_edit = move |_| {
                            event_title.set(title_for_edit.clone());
                            event_type.set(etype_for_edit.clone());
                            event_date.set(date_for_edit.clone());
                            event_time.set(time_for_edit.clone());
                            event_description.set(desc_for_edit.clone());
                            editing_event_id.set(Some(eid_for_edit.clone()));
                            show_event_form.set(true);
                        };
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
                                div { style: "display: flex; align-items: center; gap: 8px;",
                                    button { class: "btn btn-sm", onclick: on_edit, "Editar" }
                                    button { class: "btn btn-sm btn-danger", onclick: on_delete, "Eliminar" }
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
            div { class: "page-toolbar",
                button { class: "btn btn-primary", onclick: move |_| { reset_holiday_form(); show_holiday_form.set(true); },
                    "Nuevo Feriado"
                }
            }
            if show_holiday_form() {
                div { class: "card form-card",
                    h3 { "Nuevo Feriado" }
                    div { class: "form-grid",
                        div { class: "field",
                            label { "Nombre" }
                            input { class: "form-input", value: "{holiday_name}", placeholder: "Fiestas Patrias",
                                oninput: move |e| holiday_name.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { "Fecha" }
                            input { class: "form-input", value: "{holiday_date}", placeholder: "2025-09-18",
                                oninput: move |e| holiday_date.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { "Tipo" }
                            input { class: "form-input", value: "{holiday_type}", placeholder: "civil",
                                oninput: move |e| holiday_type.set(e.value()),
                            }
                        }
                    }
                    div { class: "form-actions",
                        button { class: "btn-secondary", onclick: move |_| reset_holiday_form(), "Cancelar" }
                        button { class: "btn-primary", onclick: do_save_holiday, disabled: saving_holiday(),
                            if saving_holiday() { "Guardando..." } else { "Guardar" }
                        }
                    }
                }
            }
            match holidays() {
                Some(Ok(data)) => {
                    let list = data["holidays"].as_array().cloned().unwrap_or_default();
                    let holidays_cl = holidays.clone();
                    let rows: Vec<Element> = list.iter().map(|h| {
                        let hid = h["id"].as_str().unwrap_or("").to_string();
                        let name = h["name"].as_str().unwrap_or("").to_string();
                        let date = h["date"].as_str().unwrap_or("").to_string();
                        let htype = h["type"].as_str().unwrap_or("").to_string();
                        let holidays_r = holidays_cl.clone();
                        let on_delete = move |_| {
                            let id = hid.clone();
                            let mut r = holidays_r.clone();
                            spawn(async move {
                                let _ = client::delete_holiday(&id).await;
                                r.restart();
                            });
                        };
                        rsx! {
                            div { class: "alert-item",
                                div { class: "alert-info",
                                    div { class: "alert-name", "{name}" }
                                    div { class: "alert-detail", "{date} — {htype}" }
                                }
                                button { class: "btn btn-sm btn-danger", onclick: on_delete, "Eliminar" }
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
            div { class: "page-toolbar",
                button { class: "btn btn-primary", onclick: move |_| { reset_exam_form(); show_exam_form.set(true); },
                    "Nueva Prueba"
                }
            }
            if show_exam_form() {
                div { class: "card form-card",
                    h3 { if editing_exam_id().is_some() { "Editar Prueba" } else { "Nueva Prueba" } }
                    div { class: "form-grid",
                        div { class: "field",
                            label { "Título" }
                            input { class: "form-input", value: "{exam_title}", placeholder: "Prueba Unidad 1",
                                oninput: move |e| exam_title.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { "Asignatura" }
                            input { class: "form-input", value: "{exam_subject}", placeholder: "Matemáticas",
                                oninput: move |e| exam_subject.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { "Fecha" }
                            input { class: "form-input", value: "{exam_date}", placeholder: "2025-06-15",
                                oninput: move |e| exam_date.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { "Hora" }
                            input { class: "form-input", value: "{exam_time}", placeholder: "10:00",
                                oninput: move |e| exam_time.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { "Responsable" }
                            input { class: "form-input", value: "{exam_responsible}", placeholder: "Prof. Pérez",
                                oninput: move |e| exam_responsible.set(e.value()),
                            }
                        }
                    }
                    div { class: "form-actions",
                        button { class: "btn-secondary", onclick: move |_| reset_exam_form(), "Cancelar" }
                        button { class: "btn-primary", onclick: do_save_exam, disabled: saving_exam(),
                            if saving_exam() { "Guardando..." } else { "Guardar" }
                        }
                    }
                }
            }
            match exams() {
                Some(Ok(data)) => {
                    let list = data["exams"].as_array().cloned().unwrap_or_default();
                    let exams_cl = exams.clone();
                    let rows: Vec<Element> = list.iter().map(|e| {
                        let eid = e["id"].as_str().unwrap_or("").to_string();
                        let eid_for_edit = eid.clone();
                        let title = e["title"].as_str().unwrap_or("").to_string();
                        let title_for_edit = title.clone();
                        let subject = e["subject"].as_str().unwrap_or("").to_string();
                        let subject_for_edit = subject.clone();
                        let date = e["date"].as_str().unwrap_or("").to_string();
                        let date_for_edit = date.clone();
                        let time = e["time"].as_str().unwrap_or("").to_string();
                        let time_for_edit = time.clone();
                        let responsible = e["responsible"].as_str().unwrap_or("").to_string();
                        let resp_for_edit = responsible.clone();
                        let exams_r = exams_cl.clone();
                        let on_delete = move |_| {
                            let id = eid.clone();
                            let mut r = exams_r.clone();
                            spawn(async move {
                                let _ = client::delete_exam(&id).await;
                                r.restart();
                            });
                        };
                        let on_edit = move |_| {
                            exam_title.set(title_for_edit.clone());
                            exam_subject.set(subject_for_edit.clone());
                            exam_date.set(date_for_edit.clone());
                            exam_time.set(time_for_edit.clone());
                            exam_responsible.set(resp_for_edit.clone());
                            editing_exam_id.set(Some(eid_for_edit.clone()));
                            show_exam_form.set(true);
                        };
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
                                div { style: "display: flex; align-items: center; gap: 8px;",
                                    button { class: "btn btn-sm", onclick: on_edit, "Editar" }
                                    button { class: "btn btn-sm btn-danger", onclick: on_delete, "Eliminar" }
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
