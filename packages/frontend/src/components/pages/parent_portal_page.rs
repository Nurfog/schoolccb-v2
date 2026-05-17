use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client;
use crate::seo::use_page_title;

#[component]
pub fn ParentPortalPage() -> Element {
    use_page_title("Portal Apoderado");
    let children = use_resource(|| client::fetch_json("/api/portal/parent/children"));
    let mut selected_child = use_signal(|| None::<String>);

    let children_list: Vec<Value> = match children() {
        Some(Ok(ref d)) => d["children"].as_array().cloned().unwrap_or_default(),
        _ => vec![],
    };

    let grades = use_resource(move || {
        let cid = selected_child();
        async move {
            match cid {
                Some(id) => client::fetch_json(&format!("/api/portal/parent/children/{}/grades", id)).await,
                None => Err("none".to_string()),
            }
        }
    });

    let attendance = use_resource(move || {
        let cid = selected_child();
        async move {
            match cid {
                Some(id) => client::fetch_json(&format!("/api/portal/parent/children/{}/attendance", id)).await,
                None => Err("none".to_string()),
            }
        }
    });

    let appointments = use_resource(|| client::fetch_json("/api/portal/parent/appointments"));
    let certs = use_resource(|| client::fetch_json("/api/portal/parent/certificates"));
    let messages = use_resource(|| client::fetch_json("/api/portal/parent/messages"));
    let slots = use_resource(|| client::fetch_json("/api/portal/parent/available-slots"));

    let active_child = selected_child();
    let has_child = active_child.is_some();

    let mut show_appt = use_signal(|| false);
    let mut appt_type = use_signal(|| "general".to_string());
    let mut appt_reason = use_signal(String::new);
    let mut appt_date = use_signal(String::new);

    let mut show_msg = use_signal(|| false);
    let mut msg_teacher = use_signal(String::new);
    let mut msg_subject = use_signal(String::new);
    let mut msg_body = use_signal(String::new);

    let mut show_cert = use_signal(|| false);
    let mut cert_type = use_signal(|| "alumno_regular".to_string());

    rsx! {
        div { class: "page-header",
            h1 { "Portal Apoderado" }
            p { "Consulta la información académica de tus hijos" }
        }

        if children_list.is_empty() {
            div { class: "empty-state", "No tienes hijos vinculados a tu cuenta" }
        } else {
            div { class: "children-selector",
                {children_list.iter().map(|c| {
                    let cid = c["id"].as_str().unwrap_or("").to_string();
                    let name = c["name"].as_str().unwrap_or("").to_string();
                    let grade = c["grade_level"].as_str().unwrap_or("").to_string();
                    let section = c["section"].as_str().unwrap_or("").to_string();
                    let is_active = selected_child() == Some(cid.clone());
                    let cls = if is_active { "child-card active" } else { "child-card" };
                    rsx! {
                        div { class: "{cls}", onclick: move |_| selected_child.set(Some(cid.clone())),
                            div { class: "child-name", "{name}" }
                            div { class: "child-grade", "{grade} {section}" }
                        }
                    }
                })}
            }

            if has_child {
                div { class: "dashboard-grid",
                    div { class: "dashboard-section",
                        h3 { "Calificaciones" }
                        match grades() {
                            Some(Ok(ref d)) => {
                                let averages = d["averages"].as_array().cloned().unwrap_or_default();
                                let grades_list = d["grades"].as_array().cloned().unwrap_or_default();
                                rsx! {
                                    if !averages.is_empty() {
                                        div { class: "data-table-container",
                                            table { class: "data-table",
                                                thead { tr { th { "Asignatura" } th { "Promedio" } } }
                                                tbody {
                                                    {averages.iter().map(|a| {
                                                        let sub = a["subject"].as_str().unwrap_or("").to_string();
                                                        let avg = a["average"].as_str().unwrap_or("0").to_string();
                                                        rsx! { tr { td { "{sub}" } td { b { "{avg}" } } } }
                                                    })}
                                                }
                                            }
                                        }
                                    }
                                    if !grades_list.is_empty() {
                                        h4 { "Últimas Calificaciones" }
                                        div { class: "data-table-container",
                                            table { class: "data-table",
                                                thead { tr { th { "Asignatura" } th { "Evaluación" } th { "Nota" } th { "Fecha" } } }
                                                tbody {
                                                    {grades_list.iter().take(10).map(|g| {
                                                        let sub = g["subject"].as_str().unwrap_or("").to_string();
                                                        let eval = g["evaluation"].as_str().unwrap_or("").to_string();
                                                        let val = g["value"].as_f64().unwrap_or(0.0);
                                                        let date = g["date"].as_str().unwrap_or("").to_string();
                                                        rsx! { tr { td { "{sub}" } td { "{eval}" } td { "{val}" } td { "{date}" } } }
                                                    })}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Some(Err(_)) => rsx! { div { class: "empty-state", "Error al cargar notas" } },
                            None => rsx! { div { class: "loading-spinner", "Cargando..." } },
                        }
                    }

                    div { class: "dashboard-section",
                        h3 { "Asistencia" }
                        match attendance() {
                            Some(Ok(ref d)) => {
                                let months = d["attendance"].as_array().cloned().unwrap_or_default();
                                rsx! {
                                    div { class: "data-table-container",
                                        table { class: "data-table",
                                            thead { tr { th { "Mes" } th { "Asistencia" } th { "Inasistencias" } th { "Atrasos" } } }
                                            tbody {
                                                {months.iter().map(|m| {
                                                    let month = m["month"].as_str().unwrap_or("").to_string();
                                                    let pct = m["percentage"].as_str().unwrap_or("0").to_string();
                                                    let absent = m["absent"].as_i64().unwrap_or(0);
                                                    let late = m["late"].as_i64().unwrap_or(0);
                                                    rsx! {
                                                        tr {
                                                            td { "{month}" }
                                                            td { b { "{pct}%" } }
                                                            td { "{absent}" }
                                                            td { "{late}" }
                                                        }
                                                    }
                                                })}
                                            }
                                        }
                                    }
                                }
                            }
                            Some(Err(_)) => rsx! { div { class: "empty-state", "Error al cargar asistencia" } },
                            None => rsx! { div { class: "loading-spinner", "Cargando..." } },
                        }
                    }
                }

                div { class: "dashboard-section",
                    h3 { "Certificados" }
                    div { class: "action-buttons",
                        button { class: "btn btn-primary", onclick: move |_| show_cert.set(!show_cert()),
                            if show_cert() { "Cancelar" } else { "Solicitar Certificado" }
                        }
                    }
                    if show_cert() {
                        div { class: "form-card",
                            select { class: "form-input",
                                onchange: move |e| cert_type.set(e.value()),
                                option { value: "alumno_regular", "Alumno Regular" }
                                option { value: "notas", "Certificado de Notas" }
                                option { value: "asistencia", "Certificado de Asistencia" }
                                option { value: "conducta", "Certificado de Conducta" }
                            }
                            button { class: "btn btn-primary", onclick: move |_| {
                                let sid = selected_child().unwrap_or_default();
                                let ct = cert_type();
                                spawn(async move {
                                    let payload = json!({"certificate_type": ct, "student_id": sid});
                                    let _ = client::post_json("/api/portal/parent/certificates/request", &payload).await;
                                    show_cert.set(false);
                                    certs.restart();
                                });
                            }, "Solicitar" }
                        }
                    }
                    match certs() {
                        Some(Ok(ref d)) => {
                            let my_certs = d["my_certificates"].as_array().cloned().unwrap_or_default();
                            if !my_certs.is_empty() {
                                div { class: "data-table-container",
                                    table { class: "data-table",
                                        thead { tr { th { "Tipo" } th { "Estado" } th { "Fecha" } } }
                                        tbody { {my_certs.iter().map(|c| {
                                            let ct = c["type"].as_str().unwrap_or("").to_string();
                                            let st = c["status"].as_str().unwrap_or("").to_string();
                                            let dt = c["date"].as_str().unwrap_or("").to_string();
                                            rsx! { tr { td { "{ct}" } td { span { class: "badge badge-{st}", "{st}" } } td { "{dt}" } } }
                                        })} }
                                    }
                                }
                            }
                        }
                        _ => rsx! {}
                    }
                }

                div { class: "dashboard-section",
                    h3 { "Citas" }
                    div { class: "action-buttons",
                        button { class: "btn", onclick: move |_| show_appt.set(!show_appt()),
                            if show_appt() { "Cancelar" } else { "Agendar Cita" }
                        }
                    }
                    if show_appt() {
                        div { class: "form-card",
                            div { class: "form-group",
                                label { "Tipo" }
                                select { class: "form-input", onchange: move |e| appt_type.set(e.value()),
                                    option { value: "general", "General" }
                                    option { value: "psicologia", "Psicología" }
                                    option { value: "enfermeria", "Enfermería" }
                                    option { value: "asistente_social", "Asistente Social" }
                                }
                            }
                            div { class: "form-group",
                                label { "Motivo" }
                                textarea { class: "form-input", rows: 2, value: "{appt_reason}", oninput: move |e| appt_reason.set(e.value()) }
                            }
                            div { class: "form-group",
                                label { "Fecha preferida" }
                                input { class: "form-input", r#type: "date", value: "{appt_date}", oninput: move |e| appt_date.set(e.value()) }
                            }
                            button { class: "btn btn-primary", onclick: move |_| {
                                let payload = json!({"type": appt_type(), "reason": appt_reason(), "date": appt_date()});
                                spawn(async move {
                                    let _ = client::post_json("/api/portal/parent/appointments", &payload).await;
                                    show_appt.set(false);
                                    appointments.restart();
                                });
                            }, "Agendar" }
                        }
                    }
                    match appointments() {
                        Some(Ok(ref d)) => {
                            let list = d["appointments"].as_array().cloned().unwrap_or_default();
                            if !list.is_empty() {
                                div { class: "data-table-container",
                                    table { class: "data-table",
                                        thead { tr { th { "Tipo" } th { "Estado" } th { "Fecha" } } }
                                        tbody { {list.iter().map(|a| {
                                            let t = a["type"].as_str().unwrap_or("").to_string();
                                            let s = a["status"].as_str().unwrap_or("").to_string();
                                            let d = a["date"].as_str().unwrap_or("").to_string();
                                            rsx! { tr { td { "{t}" } td { span { class: "badge badge-{s}", "{s}" } } td { "{d}" } } }
                                        })} }
                                    }
                                }
                            } else { div { class: "empty-state", "Sin citas agendadas" } }
                        }
                        _ => rsx! {}
                    }
                }
            }
        }
    }
}