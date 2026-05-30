use dioxus::prelude::*;
use crate::api::client;
use crate::seo::use_page_title;

#[component]
pub fn TeacherSchedulesPage() -> Element {
    use_page_title("Horarios Docentes");
    let mut teacher_id = use_signal(String::new);

    rsx! {
        div { class: "page-header",
            h1 { "Horarios Docentes" }
            p { "Gestión de horarios, horas contratadas y tareas extras" }
        }
        div { class: "widget-card",
            div { class: "widget-card-body",
                div { class: "form-group",
                    label { "ID del Docente:" }
                    input {
                        class: "form-input",
                        value: "{teacher_id}",
                        oninput: move |e| teacher_id.set(e.value()),
                        placeholder: "Ingresa el UUID del docente"
                    }
                }
            }
        }
        {render_teacher_details(teacher_id())}
    }
}

fn render_teacher_details(tid: String) -> Element {
    if tid.is_empty() {
        rsx! {}
    } else {
        rsx! { TeacherDetails { teacher_id: tid } }
    }
}

#[component]
fn TeacherDetails(teacher_id: String) -> Element {
    let tid1 = teacher_id.clone();
    let tid2 = teacher_id.clone();
    let schedules = use_resource(move || {
        let id = tid1.clone();
        async move { client::fetch_teacher_schedules(&id).await }
    });
    let hours = use_resource(move || {
        let id = tid2.clone();
        async move { client::fetch_teacher_hours(&id).await }
    });
    let duties = use_resource(move || {
        let id = teacher_id.clone();
        async move { client::fetch_extra_duties(&id).await }
    });

    let mut tab = use_signal(|| 0u32);

    rsx! {
        div { style: "margin-top: 16px;",
            div { class: "tabs-header",
                TabButton { label: "Horarios", active: tab() == 0, onclick: move |_| tab.set(0) }
                TabButton { label: "Horas Contratadas", active: tab() == 1, onclick: move |_| tab.set(1) }
                TabButton { label: "Tareas Extras", active: tab() == 2, onclick: move |_| tab.set(2) }
            }
            if tab() == 0 {
                match schedules() {
                    Some(Ok(data)) => {
                        let list = data["schedules"].as_array().cloned().unwrap_or_default();
                        let rows: Vec<Element> = list.iter().map(|s| {
                            let day = s["day"].as_i64().unwrap_or(0);
                            let start = s["start"].as_str().unwrap_or("").to_string();
                            let end = s["end"].as_str().unwrap_or("").to_string();
                            let stype = s["type"].as_str().unwrap_or("").to_string();
                            let subject = s["subject"].as_str().unwrap_or("").to_string();
                            let days = ["Lun", "Mar", "Mié", "Jue", "Vie", "Sáb", "Dom"];
                            let day_name = days.get(day as usize).unwrap_or(&"?").to_string();
                            rsx! {
                                div { class: "alert-item",
                                    div { class: "alert-info",
                                        div { class: "alert-name", "{day_name} {start}-{end}" }
                                        div { class: "alert-detail", "{subject} ({stype})" }
                                    }
                                }
                            }
                        }).collect();
                        rsx! {
                            div { class: "widget-card",
                                div { class: "widget-card-header",
                                    h3 { "Horario Semanal" }
                                    span { "{list.len()} bloques" }
                                }
                                div { class: "widget-card-body",
                                    if rows.is_empty() {
                                        div { class: "empty-state", "Sin horarios registrados" }
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
                match hours() {
                    Some(Ok(data)) => {
                        let total = data["total"].as_i64().unwrap_or(0);
                        let class_h = data["class"].as_i64().unwrap_or(0);
                        let admin_h = data["admin"].as_i64().unwrap_or(0);
                        let extra = data["extra"].as_i64().unwrap_or(0);
                        rsx! {
                            div { class: "kpi-grid",
                                div { class: "kpi-item",
                                    div { class: "kpi-value primary", "{total}" }
                                    div { class: "kpi-label", "Total Horas" }
                                }
                                div { class: "kpi-item",
                                    div { class: "kpi-value info", "{class_h}" }
                                    div { class: "kpi-label", "Horas Clase" }
                                }
                                div { class: "kpi-item",
                                    div { class: "kpi-value warning", "{admin_h}" }
                                    div { class: "kpi-label", "Horas Admin" }
                                }
                                div { class: "kpi-item",
                                    div { class: "kpi-value success", "{extra}" }
                                    div { class: "kpi-label", "Horas Extra" }
                                }
                            }
                        }
                    }
                    _ => rsx! { div { class: "loading-spinner", "Cargando..." } }
                }
            }
            if tab() == 2 {
                match duties() {
                    Some(Ok(data)) => {
                        let list = data["duties"].as_array().cloned().unwrap_or_default();
                        let rows: Vec<Element> = list.iter().map(|d| {
                            let dtype = d["type"].as_str().unwrap_or("").to_string();
                            let desc = d["description"].as_str().unwrap_or("").to_string();
                            let amount = d["amount"].as_f64().unwrap_or(0.0);
                            let paid = d["paid"].as_bool().unwrap_or(false);
                            let status = if paid { "(Pagado)".to_string() } else { "(Pendiente)".to_string() };
                            rsx! {
                                div { class: "alert-item",
                                    div { class: "alert-info",
                                        div { class: "alert-name", "{dtype} — ${amount:.0}" }
                                        div { class: "alert-detail", "{desc} {status}" }
                                    }
                                }
                            }
                        }).collect();
                        rsx! {
                            div { class: "widget-card",
                                div { class: "widget-card-header",
                                    h3 { "Tareas Extras" }
                                    span { "{list.len()} tareas" }
                                }
                                div { class: "widget-card-body",
                                    if rows.is_empty() {
                                        div { class: "empty-state", "Sin tareas extras" }
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
}

#[component]
fn TabButton(label: String, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let cls = if active { "tab tab-active" } else { "tab" };
    rsx! {
        button { class: "{cls}", onclick: move |ev| onclick.call(ev), "{label}" }
    }
}
