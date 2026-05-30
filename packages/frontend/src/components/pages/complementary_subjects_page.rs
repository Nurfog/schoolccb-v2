use dioxus::prelude::*;
use crate::api::client;
use crate::seo::use_page_title;

#[component]
pub fn ComplementarySubjectsPage() -> Element {
    use_page_title("Asignaturas Complementarias");
    let mut course_id = use_signal(String::new);
    let subjects = use_resource(move || {
        let id = course_id();
        async move { client::fetch_complementary_subjects(&id).await }
    });

    let mut show_form = use_signal(|| false);
    let mut editing_id = use_signal(|| None::<String>);
    let mut name = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut max = use_signal(|| "".to_string());
    let mut active = use_signal(|| true);
    let mut saving = use_signal(|| false);

    let mut reset_form = move || {
        name.set(String::new());
        description.set(String::new());
        max.set("".to_string());
        active.set(true);
        editing_id.set(None);
        show_form.set(false);
    };

    let do_save = move |_| {
        saving.set(true);
        let payload = serde_json::json!({
            "name": name(),
            "description": description(),
            "max": max().parse::<i64>().unwrap_or(0),
            "active": active(),
        });
        let _is_edit = editing_id().is_some();
        let eid = editing_id();
        let cid = course_id();
        let mut subjects = subjects.clone();
        spawn(async move {
            if let Some(ref id) = eid {
                let _ = client::update_complementary_subject(id, &payload).await;
            } else {
                let _ = client::create_complementary_subject(&cid, &payload).await;
            }
            saving.set(false);
            subjects.restart();
        });
    };

    rsx! {
        div { class: "page-header",
            h1 { "Asignaturas Complementarias" }
            p { "Talleres, preuniversitarios y actividades extracurriculares" }
        }
        div { class: "widget-card",
            div { class: "widget-card-body",
                div { class: "form-group",
                    label { "ID del Curso:" }
                    input {
                        class: "form-input",
                        value: "{course_id}",
                        oninput: move |e| {
                            course_id.set(e.value());
                            show_form.set(false);
                        },
                        placeholder: "Ingresa el UUID del curso"
                    }
                }
            }
        }
        if !course_id().is_empty() {
            div { class: "page-toolbar",
                button { class: "btn btn-primary", onclick: move |_| { reset_form(); show_form.set(true); },
                    "Nueva Asignatura"
                }
            }
            if show_form() {
                div { class: "card form-card",
                    h3 { if editing_id().is_some() { "Editar Asignatura" } else { "Nueva Asignatura Complementaria" } }
                    div { class: "form-grid",
                        div { class: "field",
                            label { "Nombre" }
                            input { class: "form-input", value: "{name}", placeholder: "Preuniversitario Matemáticas",
                                oninput: move |e| name.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { "Descripción" }
                            input { class: "form-input", value: "{description}", placeholder: "Preuniversitario...",
                                oninput: move |e| description.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { "Cupo Máximo" }
                            input { r#type: "number", class: "form-input", value: "{max}", placeholder: "30",
                                oninput: move |e| max.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { "Activo" }
                            input { r#type: "checkbox", checked: active(),
                                oninput: move |e| active.set(e.checked()),
                            }
                        }
                    }
                    div { class: "form-actions",
                        button { class: "btn-secondary", onclick: move |_| reset_form(), "Cancelar" }
                        button { class: "btn-primary", onclick: do_save, disabled: saving(),
                            if saving() { "Guardando..." } else { "Guardar" }
                        }
                    }
                }
            }
            match subjects() {
                Some(Ok(data)) => {
                    let list = data["subjects"].as_array().cloned().unwrap_or_default();
                    let subjects_cl = subjects.clone();
                    let rows: Vec<Element> = list.iter().map(|s| {
                        let sid = s["id"].as_str().unwrap_or("").to_string();
                        let sid_for_edit = sid.clone();
                        let sname = s["name"].as_str().unwrap_or("").to_string();
                        let sname_for_edit = sname.clone();
                        let sdesc = s["description"].as_str().unwrap_or("").to_string();
                        let sdesc_for_edit = sdesc.clone();
                        let smax = s["max"].as_i64().unwrap_or(0);
                        let sactive = s["active"].as_bool().unwrap_or(false);
                        let status = if sactive { "Activo" } else { "Inactivo" };
                        let subjects_r = subjects_cl.clone();
                        let on_delete = move |_| {
                            let id = sid.clone();
                            let mut r = subjects_r.clone();
                            spawn(async move {
                                let _ = client::delete_complementary_subject(&id).await;
                                r.restart();
                            });
                        };
                        let on_edit = move |_| {
                            name.set(sname_for_edit.clone());
                            description.set(sdesc_for_edit.clone());
                            max.set(smax.to_string());
                            active.set(sactive);
                            editing_id.set(Some(sid_for_edit.clone()));
                            show_form.set(true);
                        };
                        rsx! {
                            div { class: "alert-item",
                                div { class: "alert-info",
                                    div { class: "alert-name", "{sname}" }
                                    div { class: "alert-detail", "{sdesc} — Cupo: {smax} ({status})" }
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
                                h3 { "Asignaturas del Curso" }
                                span { "{list.len()} asignaturas" }
                            }
                            div { class: "widget-card-body",
                                if rows.is_empty() {
                                    div { class: "empty-state", "Sin asignaturas complementarias" }
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
