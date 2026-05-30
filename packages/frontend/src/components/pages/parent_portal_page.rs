use dioxus::prelude::*;
use crate::api::client;
use crate::seo::use_page_title;

#[component]
pub fn ParentPortalPage() -> Element {
    use_page_title("Portal Apoderado");
    let children = use_resource(client::fetch_parent_children);
    let certificates = use_resource(client::fetch_parent_certificates);

    let mut appointment_key = use_signal(|| 0);
    let appointments = use_resource(move || {
        let _ = appointment_key();
        client::fetch_parent_appointments()
    });

    let mut message_key = use_signal(|| 0);
    let messages = use_resource(move || {
        let _ = message_key();
        client::fetch_parent_messages()
    });

    let mut show_new_appointment = use_signal(|| false);
    let mut appt_type = use_signal(|| String::new());
    let mut appt_date = use_signal(|| String::new());
    let mut appt_notes = use_signal(|| String::new());

    let mut show_new_message = use_signal(|| false);
    let mut msg_teacher = use_signal(|| String::new());
    let mut msg_subject = use_signal(|| String::new());
    let mut msg_body = use_signal(|| String::new());

    let slots = use_resource(client::fetch_available_slots);

    rsx! {
        div { class: "page-header",
            h1 { "Portal del Apoderado" }
            p { "Información académica de tus hijos" }
        }
        match children() {
            Some(Ok(data)) => {
                let list = data["children"].as_array().cloned().unwrap_or_default();
                let cards: Vec<Element> = list.iter().map(|child| {
                    let name = child["name"].as_str().unwrap_or("").to_string();
                    let rut = child["rut"].as_str().unwrap_or("").to_string();
                    let grade = child["grade_level"].as_str().unwrap_or("").to_string();
                    let section = child["section"].as_str().unwrap_or("").to_string();
                    let cid = child["id"].as_str().unwrap_or("").to_string();
                    rsx! {
                        StudentCard { name: "{name}", rut: "{rut}", grade: "{grade}", section: "{section}", child_id: "{cid}" }
                    }
                }).collect();
                rsx! {
                    div { class: "mosaicos-grid", {cards.into_iter()} }
                    if list.is_empty() {
                        div { class: "widget-card", style: "padding: 2rem; text-align: center;",
                            p { "No tienes hijos registrados en el sistema." }
                        }
                    }
                }
            }
            Some(Err(e)) => rsx! { div { class: "alert alert-error", "Error: {e}" } },
            None => rsx! { div { class: "loading-spinner", "Cargando..." } },
        }
        div { class: "dashboard-grid",
            match certificates() {
                Some(Ok(data)) => {
                    let types = data["certificate_types"].as_array().cloned().unwrap_or_default();
                    let my_certs = data["my_certificates"].as_array().cloned().unwrap_or_default();
                    let cert_rows: Vec<Element> = types.iter().map(|ct| {
                        let ct_id = ct["id"].as_str().unwrap_or("").to_string();
                        let ct_name = ct["name"].as_str().unwrap_or("").to_string();
                        rsx! {
                            div { class: "alert-item",
                                div { class: "alert-info",
                                    div { class: "alert-name", "{ct_name}" }
                                }
                                button { class: "btn-primary btn-sm",
                                    onclick: move |_| {
                                        let cid = ct_id.clone();
                                        spawn(async move {
                                            let _ = client::request_certificate(&serde_json::json!({
                                                "certificate_type": cid,
                                                "student_id": ""
                                            })).await;
                                        });
                                    },
                                    "Solicitar"
                                }
                            }
                        }
                    }).collect();
                    let my_cert_rows: Vec<Element> = my_certs.iter().map(|c| {
                        let ctype = c["type"].as_str().unwrap_or("").to_string();
                        let status = c["status"].as_str().unwrap_or("").to_string();
                        let date = c["date"].as_str().unwrap_or("").to_string();
                        rsx! {
                            div { class: "alert-item",
                                div { class: "alert-info",
                                    div { class: "alert-name", "{ctype}" }
                                    div { class: "alert-detail", "{date} — {status}" }
                                }
                            }
                        }
                    }).collect();
                    rsx! {
                        div { class: "widget-card",
                            div { class: "widget-card-header",
                                h3 { "Certificados" }
                                span { "Solicitar" }
                            }
                            div { class: "widget-card-body",
                                {cert_rows.into_iter()}
                                if !my_certs.is_empty() {
                                    h4 { style: "margin-top: 1rem;", "Mis Certificados" }
                                    {my_cert_rows.into_iter()}
                                }
                            }
                        }
                    }
                }
                _ => rsx! {}
            }
            match appointments() {
                Some(Ok(data)) => {
                    let list = data["appointments"].as_array().cloned().unwrap_or_default();
                    let app_rows: Vec<Element> = list.iter().map(|a| {
                        let atype = a["type"].as_str().unwrap_or("").to_string();
                        let reason = a["reason"].as_str().unwrap_or("").to_string();
                        let status = a["status"].as_str().unwrap_or("").to_string();
                        let date = a["date"].as_str().unwrap_or("").to_string();
                        let aid = a["id"].as_str().unwrap_or("").to_string();
                        let can_cancel = status == "pending" || status == "scheduled";
                        rsx! {
                            div { class: "alert-item",
                                div { class: "alert-info",
                                    div { class: "alert-name", "{atype}" }
                                    div { class: "alert-detail", "{reason} — {date} ({status})" }
                                }
                                if can_cancel {
                                    button { class: "btn-danger btn-sm",
                                        onclick: move |_| {
                                            let aid = aid.clone();
                                            spawn(async move {
                                                let _ = client::cancel_parent_appointment(&aid).await;
                                                appointment_key += 1;
                                            });
                                        },
                                        "Cancelar"
                                    }
                                }
                            }
                        }
                    }).collect();
                    let slot_display: Vec<Element> = match slots() {
                        Some(Ok(data)) => {
                            data["available_slots"].as_array().cloned().unwrap_or_default().iter().map(|s| {
                                let label = s["label"].as_str().unwrap_or("").to_string();
                                rsx! {
                                    div { style: "font-size: 0.85rem; padding: 2px 0;", "{label}" }
                                }
                            }).collect()
                        }
                        _ => vec![]
                    };
                    rsx! {
                        div { class: "widget-card",
                            div { class: "widget-card-header",
                                h3 { "Citas Agendadas" }
                                span { "{list.len()} citas" }
                                button { class: "btn-primary btn-sm", onclick: move |_| show_new_appointment.set(!show_new_appointment()),
                                    if show_new_appointment() { "Cancelar" } else { "Nueva Cita" }
                                }
                            }
                            div { class: "widget-card-body",
                                if show_new_appointment() {
                                    div { style: "margin-bottom: 1rem; padding: 0.75rem; background: var(--bg-secondary); border-radius: 8px;",
                                        h4 { "Agendar Nueva Cita" }
                                        input {
                                            placeholder: "Tipo de cita",
                                            value: "{appt_type}",
                                            oninput: move |e| appt_type.set(e.value())
                                        }
                                        input {
                                            placeholder: "Fecha (YYYY-MM-DD)",
                                            value: "{appt_date}",
                                            oninput: move |e| appt_date.set(e.value())
                                        }
                                        input {
                                            placeholder: "Notas / Motivo",
                                            value: "{appt_notes}",
                                            oninput: move |e| appt_notes.set(e.value())
                                        }
                                        if !slot_display.is_empty() {
                                            div { style: "margin-top: 8px;",
                                                p { style: "font-size: 0.85rem; font-weight: 600; margin-bottom: 4px;", "Horarios disponibles:" }
                                                {slot_display.into_iter()}
                                            }
                                        }
                                        button { class: "btn-primary btn-sm", style: "margin-top: 8px;",
                                            onclick: move |_| {
                                                let payload = serde_json::json!({
                                                    "type": appt_type(),
                                                    "date": appt_date(),
                                                    "notes": appt_notes()
                                                });
                                                spawn(async move {
                                                    let _ = client::create_parent_appointment(&payload).await;
                                                    appointment_key += 1;
                                                });
                                                appt_type.set(String::new());
                                                appt_date.set(String::new());
                                                appt_notes.set(String::new());
                                                show_new_appointment.set(false);
                                            },
                                            "Crear Cita"
                                        }
                                    }
                                }
                                if app_rows.is_empty() && !show_new_appointment() {
                                    div { class: "empty-state", "Sin citas agendadas" }
                                } else {
                                    {app_rows.into_iter()}
                                }
                            }
                        }
                    }
                }
                _ => rsx! {}
            }
            match messages() {
                Some(Ok(data)) => {
                    let list = data["messages"].as_array().cloned().unwrap_or_default();
                    let msg_rows: Vec<Element> = list.iter().map(|m| {
                        let teacher = m["teacher"].as_str().unwrap_or("").to_string();
                        let subject = m["subject"].as_str().unwrap_or("").to_string();
                        let msg = m["message"].as_str().unwrap_or("").to_string();
                        let date = m["date"].as_str().unwrap_or("").to_string();
                        rsx! {
                            div { class: "alert-item",
                                div { class: "alert-info",
                                    div { class: "alert-name", "{teacher}: {subject}" }
                                    div { class: "alert-detail", "{msg} — {date}" }
                                }
                            }
                        }
                    }).collect();
                    rsx! {
                        div { class: "widget-card",
                            div { class: "widget-card-header",
                                h3 { "Mensajes con Profesores" }
                                span { "{list.len()} mensajes" }
                                button { class: "btn-primary btn-sm", onclick: move |_| show_new_message.set(!show_new_message()),
                                    if show_new_message() { "Cancelar" } else { "Nuevo Mensaje" }
                                }
                            }
                            div { class: "widget-card-body",
                                if show_new_message() {
                                    div { style: "margin-bottom: 1rem; padding: 0.75rem; background: var(--bg-secondary); border-radius: 8px;",
                                        h4 { "Enviar Mensaje" }
                                        input {
                                            placeholder: "Profesor",
                                            value: "{msg_teacher}",
                                            oninput: move |e| msg_teacher.set(e.value())
                                        }
                                        input {
                                            placeholder: "Asunto",
                                            value: "{msg_subject}",
                                            oninput: move |e| msg_subject.set(e.value())
                                        }
                                        textarea {
                                            placeholder: "Mensaje",
                                            value: "{msg_body}",
                                            oninput: move |e| msg_body.set(e.value()),
                                            rows: 3,
                                        }
                                        button { class: "btn-primary btn-sm", style: "margin-top: 8px;",
                                            onclick: move |_| {
                                                let payload = serde_json::json!({
                                                    "teacher": msg_teacher(),
                                                    "subject": msg_subject(),
                                                    "message": msg_body()
                                                });
                                                spawn(async move {
                                                    let _ = client::send_parent_message(&payload).await;
                                                    message_key += 1;
                                                });
                                                msg_teacher.set(String::new());
                                                msg_subject.set(String::new());
                                                msg_body.set(String::new());
                                                show_new_message.set(false);
                                            },
                                            "Enviar"
                                        }
                                    }
                                }
                                if msg_rows.is_empty() && !show_new_message() {
                                    div { class: "empty-state", "Sin mensajes" }
                                } else {
                                    {msg_rows.into_iter()}
                                }
                            }
                        }
                    }
                }
                _ => rsx! {}
            }
        }
    }
}

#[component]
fn StudentCard(name: String, rut: String, grade: String, section: String, child_id: String) -> Element {
    let cid1 = child_id.clone();
    let cid2 = child_id.clone();
    let cid3 = child_id.clone();
    let grades = use_resource(move || {
        let id = cid1.clone();
        async move { client::fetch_child_grades(&id).await }
    });
    let attendance = use_resource(move || {
        let id = cid2.clone();
        async move { client::fetch_child_attendance(&id).await }
    });
    let schedule = use_resource(move || {
        let id = cid3.clone();
        async move { client::fetch_child_schedule(&id).await }
    });
    let annotations = use_resource(move || {
        let id = child_id.clone();
        async move { client::fetch_child_annotations(&id).await }
    });

    let mut show_grades = use_signal(|| false);
    let mut show_att = use_signal(|| false);
    let mut show_sch = use_signal(|| false);
    let mut show_ann = use_signal(|| false);

    rsx! {
        div { class: "widget-card",
            div { class: "widget-card-body",
                div { class: "student-info",
                    div { class: "student-avatar-lg", "{name.chars().next().unwrap_or('?')}" }
                    div { class: "student-details",
                        h3 { "{name}" }
                        p { "RUT: {rut}" }
                        p { "Curso: {grade} {section}" }
                    }
                }
                div { style: "display: flex; gap: 8px; margin-top: 12px; flex-wrap: wrap;",
                    button { class: "btn-primary btn-sm", onclick: move |_| show_grades.set(!show_grades()),
                        if show_grades() { "Ocultar Notas" } else { "Ver Notas" }
                    }
                    button { class: "btn-primary btn-sm", onclick: move |_| show_att.set(!show_att()),
                        if show_att() { "Ocultar Asistencia" } else { "Ver Asistencia" }
                    }
                    button { class: "btn-primary btn-sm", onclick: move |_| show_sch.set(!show_sch()),
                        if show_sch() { "Ocultar Horario" } else { "Ver Horario" }
                    }
                    button { class: "btn-primary btn-sm", onclick: move |_| show_ann.set(!show_ann()),
                        if show_ann() { "Ocultar Anotaciones" } else { "Ver Anotaciones" }
                    }
                }
                if show_grades() {
                    match grades() {
                        Some(Ok(data)) => {
                            let averages = data["averages"].as_array().cloned().unwrap_or_default();
                            let grades_list = data["grades"].as_array().cloned().unwrap_or_default();
                            let avg_rows: Vec<Element> = averages.iter().map(|avg| {
                                let sub = avg["subject"].as_str().unwrap_or("").to_string();
                                let val = avg["average"].as_str().unwrap_or("0").to_string();
                                rsx! {
                                    div { class: "kpi-item",
                                        div { class: "kpi-value", "{val}" }
                                        div { class: "kpi-label", "{sub}" }
                                    }
                                }
                            }).collect();
                            let g_rows: Vec<Element> = grades_list.iter().map(|g| {
                                let sub = g["subject"].as_str().unwrap_or("").to_string();
                                let eval_name = g["evaluation"].as_str().unwrap_or("").to_string();
                                let val = g["value"].as_f64().unwrap_or(0.0);
                                let date = g["date"].as_str().unwrap_or("").to_string();
                                rsx! {
                                    div { class: "alert-item",
                                        div { class: "alert-info",
                                            div { class: "alert-name", "{sub} — {eval_name}" }
                                            div { class: "alert-detail", "Nota: {val:.1} — {date}" }
                                        }
                                    }
                                }
                            }).collect();
                            rsx! {
                                div { style: "margin-top: 12px;",
                                    h4 { "Promedios por Asignatura" }
                                    div { class: "kpi-grid", {avg_rows.into_iter()} }
                                    h4 { "Últimas Notas" }
                                    {g_rows.into_iter()}
                                }
                            }
                        }
                        _ => rsx! { div { class: "loading-spinner", "Cargando..." } }
                    }
                }
                if show_att() {
                    match attendance() {
                        Some(Ok(data)) => {
                            let monthly = data["attendance"].as_array().cloned().unwrap_or_default();
                            let att_rows: Vec<Element> = monthly.iter().map(|m| {
                                let month = m["month"].as_str().unwrap_or("").to_string();
                                let pct = m["percentage"].as_str().unwrap_or("0").to_string();
                                let present = m["present"].as_i64().unwrap_or(0);
                                let absent = m["absent"].as_i64().unwrap_or(0);
                                rsx! {
                                    div { class: "alert-item",
                                        div { class: "alert-info",
                                            div { class: "alert-name", "{month}" }
                                            div { class: "alert-detail", "{pct}% — {present} presentes, {absent} ausentes" }
                                        }
                                    }
                                }
                            }).collect();
                            rsx! {
                                div { style: "margin-top: 12px;",
                                    h4 { "Asistencia Últimos 6 Meses" }
                                    {att_rows.into_iter()}
                                }
                            }
                        }
                        _ => rsx! { div { class: "loading-spinner", "Cargando..." } }
                    }
                }
                if show_sch() {
                    match schedule() {
                        Some(Ok(data)) => {
                            let list = data["schedule"].as_array().cloned().unwrap_or_default();
                            let sch_rows: Vec<Element> = list.iter().map(|s| {
                                let sub = s["subject"].as_str().unwrap_or("").to_string();
                                let day = s["day"].as_str().unwrap_or("").to_string();
                                let time = s["time"].as_str().unwrap_or("").to_string();
                                rsx! {
                                    div { class: "alert-item",
                                        div { class: "alert-info",
                                            div { class: "alert-name", "{sub}" }
                                            div { class: "alert-detail", "{day} — {time}" }
                                        }
                                    }
                                }
                            }).collect();
                            rsx! {
                                div { style: "margin-top: 12px;",
                                    h4 { "Horario" }
                                    {sch_rows.into_iter()}
                                }
                            }
                        }
                        _ => rsx! { div { class: "loading-spinner", "Cargando..." } }
                    }
                }
                if show_ann() {
                    match annotations() {
                        Some(Ok(data)) => {
                            let list = data["annotations"].as_array().cloned().unwrap_or_default();
                            let ann_rows: Vec<Element> = list.iter().map(|a| {
                                let atype = a["type"].as_str().unwrap_or("").to_string();
                                let desc = a["description"].as_str().unwrap_or("").to_string();
                                let date = a["date"].as_str().unwrap_or("").to_string();
                                let teacher = a["teacher"].as_str().unwrap_or("").to_string();
                                rsx! {
                                    div { class: "alert-item",
                                        div { class: "alert-info",
                                            div { class: "alert-name", "{atype}" }
                                            div { class: "alert-detail", "{desc} — {date} por {teacher}" }
                                        }
                                    }
                                }
                            }).collect();
                            rsx! {
                                div { style: "margin-top: 12px;",
                                    h4 { "Anotaciones" }
                                    if ann_rows.is_empty() {
                                        div { class: "empty-state", "Sin anotaciones" }
                                    } else {
                                        {ann_rows.into_iter()}
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => rsx! { div { class: "alert alert-error", "Error: {e}" } },
                        None => rsx! { div { class: "loading-spinner", "Cargando..." } }
                    }
                }
            }
        }
    }
}
