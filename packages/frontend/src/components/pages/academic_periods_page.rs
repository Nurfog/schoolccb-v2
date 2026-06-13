use dioxus::prelude::*;
use crate::api::client;

#[component]
pub fn AcademicPeriodsPage() -> Element {
    let mut periods = use_resource(|| client::fetch_academic_periods());
    let mut show_form = use_signal(|| false);
    let mut editing_id = use_signal(|| None::<String>);
    let now_year = js_sys::Date::new_0().get_full_year() as i32;
    let mut name = use_signal(|| String::new());
    let mut year = use_signal(|| now_year);
    let mut semester = use_signal(|| 1);
    let mut start_date = use_signal(|| String::new());
    let mut end_date = use_signal(|| String::new());
    let mut is_active = use_signal(|| false);
    let mut saving = use_signal(|| false);

    let mut reset_form = move || {
        name.set(String::new());
        year.set(now_year);
        semester.set(1);
        start_date.set(String::new());
        end_date.set(String::new());
        is_active.set(false);
        editing_id.set(None);
        show_form.set(false);
    };

    let do_save = move |_| {
        saving.set(true);
        let payload = serde_json::json!({
            "name": name(),
            "year": year(),
            "semester": semester(),
            "start_date": start_date(),
            "end_date": end_date(),
            "is_active": is_active(),
        });
        let eid = editing_id();
        spawn(async move {
            if let Some(ref id) = eid {
                let _ = client::update_academic_period(id, &payload).await;
            } else {
                let _ = client::create_academic_period(&payload).await;
            }
            saving.set(false);
            reset_form();
            periods.restart();
        });
    };

    let do_activate = move |id: String| {
        spawn(async move {
            let payload = serde_json::json!({"is_active": true});
            let _ = client::update_academic_period(&id, &payload).await;
            periods.restart();
        });
    };

    let mut do_edit = move |p: serde_json::Value| {
        name.set(p["name"].as_str().unwrap_or("").to_string());
        year.set(p["year"].as_i64().unwrap_or(now_year as i64) as i32);
        semester.set(p["semester"].as_i64().unwrap_or(1) as i32);
        start_date.set(p["start_date"].as_str().unwrap_or("").to_string());
        end_date.set(p["end_date"].as_str().unwrap_or("").to_string());
        is_active.set(p["is_active"].as_bool().unwrap_or(false));
        editing_id.set(Some(p["id"].as_str().unwrap_or("").to_string()));
        show_form.set(true);
    };

    rsx! {
        div { class: "page-header",
            h1 { "Períodos Académicos" }
            p { "Gestión de semestres y períodos académicos del año escolar" }
        }
        div { class: "page-toolbar",
            button { class: "btn btn-primary", onclick: move |_| { reset_form(); show_form.set(!show_form()); },
                if show_form() { "Cancelar" } else { "Nuevo Período" }
            }
        }
        {
            if show_form() {
                rsx! {
                    div { class: "form-card",
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Nombre:" }
                                input { class: "form-input", value: "{name}", oninput: move |e| name.set(e.value()), placeholder: "Semestre 1" }
                            }
                            div { class: "form-group",
                                label { "Año:" }
                                input { class: "form-input", value: "{year}", oninput: move |e| year.set(e.value().parse::<i32>().unwrap_or(now_year)), type: "number" }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Semestre:" }
                                select { class: "form-input", value: "{semester}", oninput: move |e| semester.set(e.value().parse::<i32>().unwrap_or(1)),
                                    option { value: "1", "Semestre 1" }
                                    option { value: "2", "Semestre 2" }
                                }
                            }
                            div { class: "form-group",
                                label { style: "display: flex; align-items: center; gap: 8px; margin-top: 24px;",
                                    input { type: "checkbox", checked: is_active(), oninput: move |e| is_active.set(e.checked()) }
                                    "Activo"
                                }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Fecha Inicio:" }
                                input { class: "form-input", value: "{start_date}", oninput: move |e| start_date.set(e.value()), type: "date" }
                            }
                            div { class: "form-group",
                                label { "Fecha Término:" }
                                input { class: "form-input", value: "{end_date}", oninput: move |e| end_date.set(e.value()), type: "date" }
                            }
                        }
                        div { class: "form-actions",
                            button { class: "btn btn-primary", disabled: saving(), onclick: do_save,
                                if editing_id().is_some() { if saving() { "Guardando..." } else { "Actualizar" } }
                                else { if saving() { "Guardando..." } else { "Crear Período" } }
                            }
                            button { class: "btn", onclick: move |_| reset_form(), "Cancelar" }
                        }
                    }
                }
            } else { rsx! {} }
        }
        match periods() {
            Some(Ok(data)) => {
                let list = data["periods"].as_array().cloned().unwrap_or_default();
                let rows: Vec<Element> = list.iter().map(|p| {
                    let pid = p["id"].as_str().unwrap_or("").to_string();
                    let pname = p["name"].as_str().unwrap_or("-").to_string();
                    let pyear = p["year"].as_i64().unwrap_or(0);
                    let psem = format!("Semestre {}", p["semester"].as_i64().unwrap_or(1));
                    let pstart = p["start_date"].as_str().unwrap_or("-").to_string();
                    let pend = p["end_date"].as_str().unwrap_or("-").to_string();
                    let pact = p["is_active"].as_bool().unwrap_or(false);
                    rsx! {
                        tr {
                            td { "{pname}" }
                            td { "{pyear}" }
                            td { "{psem}" }
                            td { "{pstart}" }
                            td { "{pend}" }
                            td { if pact { span { class: "grade-good", "✓ Activo" } } else { span { class: "grade-bad", "✗ Inactivo" } } }
                            td {
                                button { class: "btn btn-sm", onclick: { let p = p.clone(); move |_| do_edit(p.clone()) }, "Editar" }
                                if !pact {
                                    button { class: "btn btn-sm btn-primary", style: "margin-left: 4px;", onclick: { let id = pid.clone(); move |_| do_activate(id.clone()) }, "Activar" }
                                }
                            }
                        }
                    }
                }).collect();
                let empty = list.is_empty();
                rsx! {
                    div { class: "data-table-container",
                        table { class: "data-table",
                            thead { tr {
                                th { "Nombre" }
                                th { "Año" }
                                th { "Período" }
                                th { "Inicio" }
                                th { "Término" }
                                th { "Estado" }
                                th { "Acciones" }
                            }}
                            tbody { {rows.into_iter()} }
                        }
                        if empty {
                            div { class: "empty-state", "Sin períodos académicos configurados. Cree el primer período para comenzar." }
                        }
                    }
                }
            }
            _ => rsx! { div { class: "loading-spinner", "Cargando períodos..." } },
        }
    }
}
