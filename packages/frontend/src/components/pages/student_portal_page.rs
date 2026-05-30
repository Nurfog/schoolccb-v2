use dioxus::prelude::*;
use crate::api::client;
use crate::seo::use_page_title;

#[component]
pub fn StudentPortalPage() -> Element {
    use_page_title("Portal Alumno");
    let profile = use_resource(client::fetch_student_profile);
    let grades = use_resource(client::fetch_student_grades);
    let attendance = use_resource(client::fetch_student_attendance);
    let schedule = use_resource(client::fetch_student_schedule);
    let annotations = use_resource(client::fetch_student_annotations);
    let mut appt_key = use_signal(|| 0u32);
    let appointments = use_resource(move || {
        let _ = appt_key();
        client::fetch_student_appointments()
    });

    let mut show_grades = use_signal(|| false);
    let mut show_att = use_signal(|| false);
    let mut show_sch = use_signal(|| false);
    let mut show_ann = use_signal(|| false);
    let mut show_app = use_signal(|| false);

    let mut show_form = use_signal(|| false);
    let mut appt_type = use_signal(String::new);
    let mut appt_date = use_signal(String::new);
    let mut appt_notes = use_signal(String::new);
    let mut submitting = use_signal(|| false);
    let mut error_msg = use_signal(|| None::<String>);

    rsx! {
        div { class: "page-header",
            h1 { "Portal del Alumno" }
            p { "Tus calificaciones, asistencia y más" }
        }
        match profile() {
            Some(Ok(data)) => {
                let name = data["name"].as_str().unwrap_or("").to_string();
                let rut = data["rut"].as_str().unwrap_or("").to_string();
                let grade = data["grade_level"].as_str().unwrap_or("").to_string();
                let school = data["school"].as_str().unwrap_or("").to_string();
                rsx! {
                    div { class: "widget-card",
                        div { class: "widget-card-body",
                            div { class: "student-info",
                                div { class: "student-avatar-lg", "{name.chars().next().unwrap_or('?')}" }
                                div { class: "student-details",
                                    h3 { "{name}" }
                                    p { "RUT: {rut}" }
                                    p { "Curso: {grade}" }
                                    p { "Colegio: {school}" }
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
                                button { class: "btn-primary btn-sm", onclick: move |_| show_app.set(!show_app()),
                                    if show_app() { "Ocultar Citas" } else { "Ver Citas" }
                                }
                            }
                        }
                    }
                }
            }
            Some(Err(e)) => rsx! { div { class: "alert alert-error", "Error: {e}" } },
            None => rsx! { div { class: "loading-spinner", "Cargando..." } },
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
                        let name = g["name"].as_str().unwrap_or("").to_string();
                        let val = g["value"].as_f64().unwrap_or(0.0);
                        let date = g["date"].as_str().unwrap_or("").to_string();
                        rsx! {
                            div { class: "alert-item",
                                div { class: "alert-info",
                                    div { class: "alert-name", "{sub}" }
                                    div { class: "alert-detail", "{name}: {val:.1} — {date}" }
                                }
                            }
                        }
                    }).collect();
                    rsx! {
                        div { class: "widget-card", style: "margin-top: 1rem;",
                            div { class: "widget-card-header",
                                h3 { "Notas y Promedios" }
                            }
                            div { class: "widget-card-body",
                                if !averages.is_empty() {
                                    div { class: "kpi-grid", {avg_rows.into_iter()} }
                                }
                                if !grades_list.is_empty() {
                                    h4 { style: "margin-top: 1rem;", "Últimas Notas" }
                                    {g_rows.into_iter()}
                                }
                            }
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
                        let late = m["late"].as_i64().unwrap_or(0);
                        rsx! {
                            div { class: "alert-item",
                                div { class: "alert-info",
                                    div { class: "alert-name", "{month} — {pct}%" }
                                    div { class: "alert-detail", "Presentes: {present}, Ausentes: {absent}, Atrasos: {late}" }
                                }
                            }
                        }
                    }).collect();
                    rsx! {
                        div { class: "widget-card", style: "margin-top: 1rem;",
                            div { class: "widget-card-header", h3 { "Asistencia" } }
                            div { class: "widget-card-body", {att_rows.into_iter()} }
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
                        div { class: "widget-card", style: "margin-top: 1rem;",
                            div { class: "widget-card-header", h3 { "Horario" } }
                            div { class: "widget-card-body", {sch_rows.into_iter()} }
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
                        rsx! {
                            div { class: "alert-item",
                                div { class: "alert-info",
                                    div { class: "alert-name", "{atype}" }
                                    div { class: "alert-detail", "{desc} — {date}" }
                                }
                            }
                        }
                    }).collect();
                    rsx! {
                        div { class: "widget-card", style: "margin-top: 1rem;",
                            div { class: "widget-card-header", h3 { "Anotaciones" } }
                            div { class: "widget-card-body",
                                if ann_rows.is_empty() {
                                    div { class: "empty-state", "Sin anotaciones" }
                                } else {
                                    {ann_rows.into_iter()}
                                }
                            }
                        }
                    }
                }
                _ => rsx! { div { class: "loading-spinner", "Cargando..." } }
            }
        }
        if show_app() {
            match appointments() {
                Some(Ok(data)) => {
                    let list = data["appointments"].as_array().cloned().unwrap_or_default();
                    let app_rows: Vec<Element> = list.iter().map(|a| {
                        let atype = a["type"].as_str().unwrap_or("").to_string();
                        let reason = a["reason"].as_str().unwrap_or("").to_string();
                        let status = a["status"].as_str().unwrap_or("").to_string();
                        rsx! {
                            div { class: "alert-item",
                                div { class: "alert-info",
                                    div { class: "alert-name", "{atype} — {status}" }
                                    div { class: "alert-detail", "{reason}" }
                                }
                            }
                        }
                    }).collect();
                    rsx! {
                        div { class: "widget-card", style: "margin-top: 1rem;",
                            div { class: "widget-card-header",
                                h3 { "Citas con Apoyo" }
                                button { class: "btn-primary btn-sm", onclick: move |_| show_form.set(!show_form()),
                                    if show_form() { "Cancelar" } else { "Nueva Cita" }
                                }
                            }
                            div { class: "widget-card-body",
                                if app_rows.is_empty() {
                                    div { class: "empty-state", "Sin citas agendadas" }
                                } else {
                                    {app_rows.into_iter()}
                                }
                                if show_form() {
                                    div { style: "margin-top: 1rem; padding-top: 1rem; border-top: 1px solid var(--border-color);",
                                        div { class: "form-group",
                                            label { "Tipo de Cita" }
                                            select { class: "form-input", value: "{appt_type}", oninput: move |e| appt_type.set(e.value()),
                                                option { value: "", "Seleccionar..." }
                                                option { value: "psicologica", "Psicológica" }
                                                option { value: "pedagogica", "Pedagógica" }
                                                option { value: "social", "Social" }
                                                option { value: "otro", "Otro" }
                                            }
                                        }
                                        div { class: "form-group",
                                            label { "Fecha" }
                                            input { class: "form-input", r#type: "date", value: "{appt_date}", oninput: move |e| appt_date.set(e.value()) }
                                        }
                                        div { class: "form-group",
                                            label { "Motivo" }
                                            textarea { class: "form-input", rows: "3", value: "{appt_notes}", oninput: move |e| appt_notes.set(e.value()) }
                                        }
                                        if let Some(msg) = error_msg() {
                                            div { class: "alert alert-error", "{msg}" }
                                        }
                                        div { style: "display: flex; gap: 8px; margin-top: 8px;",
                                            button {
                                                class: "btn-primary btn-sm",
                                                disabled: submitting(),
                                                onclick: move |_| {
                                                    if appt_type().is_empty() || appt_date().is_empty() || appt_notes().is_empty() {
                                                        error_msg.set(Some("Todos los campos son obligatorios".into()));
                                                        return;
                                                    }
                                                    submitting.set(true);
                                                    error_msg.set(None);
                                                    let payload = serde_json::json!({
                                                        "type": appt_type(),
                                                        "date": appt_date(),
                                                        "notes": appt_notes()
                                                    });
                                                    spawn(async move {
                                                        match client::create_student_appointment(&payload).await {
                                                            Ok(_) => {
                                                                submitting.set(false);
                                                                show_form.set(false);
                                                                appt_type.set(String::new());
                                                                appt_date.set(String::new());
                                                                appt_notes.set(String::new());
                                                                appt_key += 1;
                                                            }
                                                            Err(e) => {
                                                                submitting.set(false);
                                                                error_msg.set(Some(e));
                                                            }
                                                        }
                                                    });
                                                },
                                                if submitting() { "Enviando..." } else { "Solicitar Cita" }
                                            }
                                        }
                                    }
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
