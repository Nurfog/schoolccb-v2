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
    let tid_for_sched = teacher_id.clone();
    let tid_for_hours = teacher_id.clone();
    let tid_for_duty = teacher_id.clone();

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

    let mut show_sched_form = use_signal(|| false);
    let mut sched_day = use_signal(|| "0".to_string());
    let mut sched_start = use_signal(String::new);
    let mut sched_end = use_signal(String::new);
    let mut sched_type = use_signal(String::new);
    let mut sched_subject = use_signal(String::new);
    let mut saving_sched = use_signal(|| false);

    let mut show_hours_form = use_signal(|| false);
    let mut hours_class = use_signal(|| "".to_string());
    let mut hours_admin = use_signal(|| "".to_string());
    let mut hours_extra = use_signal(|| "".to_string());
    let mut saving_hours = use_signal(|| false);

    let mut show_duty_form = use_signal(|| false);
    let mut editing_duty_id = use_signal(|| None::<String>);
    let mut duty_type = use_signal(String::new);
    let mut duty_desc = use_signal(String::new);
    let mut duty_amount = use_signal(|| "".to_string());
    let mut saving_duty = use_signal(|| false);

    let mut reset_sched_form = move || {
        sched_day.set("0".to_string());
        sched_start.set(String::new());
        sched_end.set(String::new());
        sched_type.set(String::new());
        sched_subject.set(String::new());
        show_sched_form.set(false);
    };

    let mut reset_hours_form = move || {
        hours_class.set("".to_string());
        hours_admin.set("".to_string());
        hours_extra.set("".to_string());
        show_hours_form.set(false);
    };

    let mut reset_duty_form = move || {
        duty_type.set(String::new());
        duty_desc.set(String::new());
        duty_amount.set("".to_string());
        editing_duty_id.set(None);
        show_duty_form.set(false);
    };

    let do_save_sched = move |_| {
        saving_sched.set(true);
        let payload = serde_json::json!({
            "day": sched_day().parse::<i64>().unwrap_or(0),
            "start": sched_start(),
            "end": sched_end(),
            "type": sched_type(),
            "subject": sched_subject(),
        });
        let tid = tid_for_sched.clone();
        let mut schedules = schedules.clone();
        spawn(async move {
            let _ = client::create_teacher_schedule(&tid, &payload).await;
            saving_sched.set(false);
            schedules.restart();
        });
    };

    let do_save_hours = move |_| {
        saving_hours.set(true);
        let payload = serde_json::json!({
            "class": hours_class().parse::<i64>().unwrap_or(0),
            "admin": hours_admin().parse::<i64>().unwrap_or(0),
            "extra": hours_extra().parse::<i64>().unwrap_or(0),
        });
        let tid = tid_for_hours.clone();
        let mut hours = hours.clone();
        spawn(async move {
            let _ = client::set_teacher_hours(&tid, &payload).await;
            saving_hours.set(false);
            hours.restart();
        });
    };

    let do_save_duty = move |_| {
        saving_duty.set(true);
        let payload = serde_json::json!({
            "type": duty_type(),
            "description": duty_desc(),
            "amount": duty_amount().parse::<f64>().unwrap_or(0.0),
        });
        let _is_edit = editing_duty_id().is_some();
        let eid = editing_duty_id();
        let tid = tid_for_duty.clone();
        let mut duties = duties.clone();
        spawn(async move {
            if let Some(ref id) = eid {
                let _ = client::update_extra_duty(id, &payload).await;
            } else {
                let _ = client::create_extra_duty(&tid, &payload).await;
            }
            saving_duty.set(false);
            duties.restart();
        });
    };

    rsx! {
        div { style: "margin-top: 16px;",
            div { class: "tabs-header",
                TabButton { label: "Horarios", active: tab() == 0, onclick: move |_| tab.set(0) }
                TabButton { label: "Horas Contratadas", active: tab() == 1, onclick: move |_| tab.set(1) }
                TabButton { label: "Tareas Extras", active: tab() == 2, onclick: move |_| tab.set(2) }
            }
            if tab() == 0 {
                div { class: "page-toolbar",
                    button { class: "btn btn-primary", onclick: move |_| { reset_sched_form(); show_sched_form.set(true); },
                        "Nuevo Bloque"
                    }
                }
                if show_sched_form() {
                    div { class: "card form-card",
                        h3 { "Nuevo Bloque Horario" }
                        div { class: "form-grid",
                            div { class: "field",
                                label { "Día" }
                                select { class: "form-input", value: "{sched_day}",
                                    onchange: move |e| sched_day.set(e.value()),
                                    option { value: "0", "Lunes" }
                                    option { value: "1", "Martes" }
                                    option { value: "2", "Miércoles" }
                                    option { value: "3", "Jueves" }
                                    option { value: "4", "Viernes" }
                                    option { value: "5", "Sábado" }
                                    option { value: "6", "Domingo" }
                                }
                            }
                            div { class: "field",
                                label { "Inicio" }
                                input { class: "form-input", value: "{sched_start}", placeholder: "08:00",
                                    oninput: move |e| sched_start.set(e.value()),
                                }
                            }
                            div { class: "field",
                                label { "Fin" }
                                input { class: "form-input", value: "{sched_end}", placeholder: "09:30",
                                    oninput: move |e| sched_end.set(e.value()),
                                }
                            }
                            div { class: "field",
                                label { "Tipo" }
                                input { class: "form-input", value: "{sched_type}", placeholder: "Clase",
                                    oninput: move |e| sched_type.set(e.value()),
                                }
                            }
                            div { class: "field",
                                label { "Asignatura" }
                                input { class: "form-input", value: "{sched_subject}", placeholder: "Matemáticas",
                                    oninput: move |e| sched_subject.set(e.value()),
                                }
                            }
                        }
                        div { class: "form-actions",
                            button { class: "btn-secondary", onclick: move |_| reset_sched_form(), "Cancelar" }
                            button { class: "btn-primary", onclick: do_save_sched, disabled: saving_sched(),
                                if saving_sched() { "Guardando..." } else { "Guardar" }
                            }
                        }
                    }
                }
                match schedules() {
                    Some(Ok(data)) => {
                        let list = data["schedules"].as_array().cloned().unwrap_or_default();
                        let schedules_cl = schedules.clone();
                        let rows: Vec<Element> = list.iter().map(|s| {
                            let sid = s["id"].as_str().unwrap_or("").to_string();
                            let day = s["day"].as_i64().unwrap_or(0);
                            let start = s["start"].as_str().unwrap_or("").to_string();
                            let end = s["end"].as_str().unwrap_or("").to_string();
                            let stype = s["type"].as_str().unwrap_or("").to_string();
                            let subject = s["subject"].as_str().unwrap_or("").to_string();
                            let days = ["Lun", "Mar", "Mié", "Jue", "Vie", "Sáb", "Dom"];
                            let day_name = days.get(day as usize).unwrap_or(&"?").to_string();
                            let schedules_r = schedules_cl.clone();
                            let on_delete = move |_| {
                                let id = sid.clone();
                                let mut r = schedules_r.clone();
                                spawn(async move {
                                    let _ = client::delete_teacher_schedule(&id).await;
                                    r.restart();
                                });
                            };
                            rsx! {
                                div { class: "alert-item",
                                    div { class: "alert-info",
                                        div { class: "alert-name", "{day_name} {start}-{end}" }
                                        div { class: "alert-detail", "{subject} ({stype})" }
                                    }
                                    button { class: "btn btn-sm btn-danger", onclick: on_delete, "Eliminar" }
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
                div { class: "page-toolbar",
                    button { class: "btn btn-primary", onclick: move |_| { reset_hours_form(); show_hours_form.set(true); },
                        "Establecer Horas"
                    }
                }
                if show_hours_form() {
                    div { class: "card form-card",
                        h3 { "Establecer Horas Contratadas" }
                        div { class: "form-grid",
                            div { class: "field",
                                label { "Horas Clase" }
                                input { r#type: "number", class: "form-input", value: "{hours_class}", placeholder: "30",
                                    oninput: move |e| hours_class.set(e.value()),
                                }
                            }
                            div { class: "field",
                                label { "Horas Admin" }
                                input { r#type: "number", class: "form-input", value: "{hours_admin}", placeholder: "10",
                                    oninput: move |e| hours_admin.set(e.value()),
                                }
                            }
                            div { class: "field",
                                label { "Horas Extra" }
                                input { r#type: "number", class: "form-input", value: "{hours_extra}", placeholder: "5",
                                    oninput: move |e| hours_extra.set(e.value()),
                                }
                            }
                        }
                        div { class: "form-actions",
                            button { class: "btn-secondary", onclick: move |_| reset_hours_form(), "Cancelar" }
                            button { class: "btn-primary", onclick: do_save_hours, disabled: saving_hours(),
                                if saving_hours() { "Guardando..." } else { "Guardar" }
                            }
                        }
                    }
                }
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
                div { class: "page-toolbar",
                    button { class: "btn btn-primary", onclick: move |_| { reset_duty_form(); show_duty_form.set(true); },
                        "Nueva Tarea Extra"
                    }
                }
                if show_duty_form() {
                    div { class: "card form-card",
                        h3 { if editing_duty_id().is_some() { "Editar Tarea Extra" } else { "Nueva Tarea Extra" } }
                        div { class: "form-grid",
                            div { class: "field",
                                label { "Tipo" }
                                input { class: "form-input", value: "{duty_type}", placeholder: "Reemplazo",
                                    oninput: move |e| duty_type.set(e.value()),
                                }
                            }
                            div { class: "field",
                                label { "Descripción" }
                                input { class: "form-input", value: "{duty_desc}", placeholder: "Reemplazo...",
                                    oninput: move |e| duty_desc.set(e.value()),
                                }
                            }
                            div { class: "field",
                                label { "Monto" }
                                input { r#type: "number", class: "form-input", value: "{duty_amount}", placeholder: "50000",
                                    oninput: move |e| duty_amount.set(e.value()),
                                }
                            }
                        }
                        div { class: "form-actions",
                            button { class: "btn-secondary", onclick: move |_| reset_duty_form(), "Cancelar" }
                            button { class: "btn-primary", onclick: do_save_duty, disabled: saving_duty(),
                                if saving_duty() { "Guardando..." } else { "Guardar" }
                            }
                        }
                    }
                }
                match duties() {
                    Some(Ok(data)) => {
                        let list = data["duties"].as_array().cloned().unwrap_or_default();
                        let duties_cl = duties.clone();
                        let rows: Vec<Element> = list.iter().map(|d| {
                            let did = d["id"].as_str().unwrap_or("").to_string();
                            let did_for_edit = did.clone();
                            let dtype = d["type"].as_str().unwrap_or("").to_string();
                            let dtype_for_edit = dtype.clone();
                            let ddesc = d["description"].as_str().unwrap_or("").to_string();
                            let ddesc_for_edit = ddesc.clone();
                            let amount = d["amount"].as_f64().unwrap_or(0.0);
                            let paid = d["paid"].as_bool().unwrap_or(false);
                            let status = if paid { "(Pagado)".to_string() } else { "(Pendiente)".to_string() };
                            let duties_r = duties_cl.clone();
                            let on_delete = move |_| {
                                let id = did.clone();
                                let mut r = duties_r.clone();
                                spawn(async move {
                                    let _ = client::delete_json(&format!("/api/hr/extra-duties/{}", id)).await;
                                    r.restart();
                                });
                            };
                            let on_edit = move |_| {
                                duty_type.set(dtype_for_edit.clone());
                                duty_desc.set(ddesc_for_edit.clone());
                                duty_amount.set(amount.to_string());
                                editing_duty_id.set(Some(did_for_edit.clone()));
                                show_duty_form.set(true);
                            };
                            rsx! {
                                div { class: "alert-item",
                                    div { class: "alert-info",
                                        div { class: "alert-name", "{dtype} — ${amount:.0}" }
                                        div { class: "alert-detail", "{ddesc} {status}" }
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
