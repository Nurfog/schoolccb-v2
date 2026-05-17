use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client;
use crate::components::widgets::icon::Icon;

#[component]
pub fn SubjectsPage() -> Element {
    let mut subjects = use_resource(|| client::fetch_json("/api/grades/subjects"));
    let mut code = use_signal(|| "".to_string());
    let mut name = use_signal(|| "".to_string());
    let mut editing_id = use_signal(|| None::<String>);
    let mut show_form = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut expanded_id = use_signal(|| None::<String>);
    let mut hours_data = use_signal(|| Vec::<Value>::new());
    let mut saving_hours = use_signal(|| false);
    let mut show_import = use_signal(|| false);
    let mut import_text = use_signal(|| String::new());
    let mut import_result = use_signal(|| None::<String>);
    let mut importing = use_signal(|| false);
    let mut editing_cell = use_signal(|| None::<(String, String)>);
    let mut edit_value = use_signal(|| String::new());

    let mut reset = move || {
        code.set("".to_string());
        name.set("".to_string());
        editing_id.set(None);
        show_form.set(false);
    };

    let table_rows = match subjects() {
        Some(Ok(data)) => {
            let list = data["subjects"].as_array().cloned().unwrap_or_default();
            if list.is_empty() {
                vec![
                    rsx! { tr { td { colspan: "4", class: "empty-state", "No hay asignaturas" } } },
                ]
            } else {
                let mut rows = Vec::new();
                for s in &list {
                    let sid = s["id"].as_str().unwrap_or("").to_string();
                    let code_v = s["code"].as_str().unwrap_or("").to_string();
                    let name_v = s["name"].as_str().unwrap_or("").to_string();
                    let active = s["active"].as_bool().unwrap_or(true);
                    let is_expanded = expanded_id() == Some(sid.clone());
                    let hours = s["hours_by_level"].as_array().cloned().unwrap_or_default();

                    let sv = s.clone();
                    let sid_e = sid.clone();
                    let sid_h = sid.clone();

                    let is_editing_code = editing_cell() == Some((sid.clone(), "code".to_string()));
                    let is_editing_name = editing_cell() == Some((sid.clone(), "name".to_string()));

                    rows.push(rsx! {
                        tr { key: "{sid}",
                            td { class: "cell-mono",
                                ondoubleclick: { let s = sid.clone(); let cv = code_v.clone(); move |_| { editing_cell.set(Some((s.clone(), "code".to_string()))); edit_value.set(cv.clone()); } },
                                {
                                    if is_editing_code {
                                        rsx! {
                                            input {
                                                class: "inline-edit-input",
                                                value: "{edit_value}",
                                                oninput: move |e| edit_value.set(e.value()),
                                                onblur: move |_| { editing_cell.set(None); },
                                            }
                                        }
                                    } else {
                                        rsx! { span { "{code_v}" } }
                                    }
                                }
                            }
                            td { class: "cell-name",
                                ondoubleclick: { let s = sid.clone(); let nv = name_v.clone(); move |_| { editing_cell.set(Some((s.clone(), "name".to_string()))); edit_value.set(nv.clone()); } },
                                {
                                    if is_editing_name {
                                        rsx! {
                                            input {
                                                class: "inline-edit-input",
                                                value: "{edit_value}",
                                                oninput: move |e| edit_value.set(e.value()),
                                                onblur: move |_| { editing_cell.set(None); },
                                            }
                                        }
                                    } else {
                                        rsx! { span { "{name_v}" } }
                                    }
                                }
                            }
                            td { if active { span { class: "status-active", "Activo" } } else { span { class: "status-inactive", "Inactivo" } } }
                            td { class: "cell-actions",
                                button { class: "btn-icon", onclick: move |_| {
                                    code.set(sv["code"].as_str().unwrap_or("").to_string());
                                    name.set(sv["name"].as_str().unwrap_or("").to_string());
                                    editing_id.set(sv["id"].as_str().map(|s| s.to_string()));
                                    show_form.set(true);
                                },
                                    Icon { name: "edit", size: 16 }
                                }
                                button { class: "btn-icon", onclick: move |_| {
                                    if expanded_id() == Some(sid_e.clone()) { expanded_id.set(None); }
                                    else { expanded_id.set(Some(sid_e.clone())); hours_data.set(hours.clone()); }
                                },
                                    Icon { name: "chevron-down", size: 16 }
                                }
                                if active {
                                    button { class: "btn-icon btn-icon-danger", onclick: {
                                        let sid = sid_h.clone();
                                        move |_| {
                                            if !web_sys::window().unwrap().confirm_with_message("¿Estás seguro?").unwrap_or(false) { return; }
                                            let id = sid.clone();
                                            spawn(async move { let _ = client::delete_json(&format!("/api/grades/subjects/{}", id)).await; subjects.restart(); });
                                        }
                                    },
                                        Icon { name: "trash", size: 16 }
                                    }
                                }
                            }
                        }
                    });

                    if is_expanded {
                        let mut hour_fields = Vec::new();
                        for (i, h) in hours_data().iter().enumerate() {
                            let lvl = h["level"].as_str().unwrap_or("").to_string();
                            let val = h["hours_per_week"].as_i64().unwrap_or(0).to_string();
                            hour_fields.push(rsx! {
                                div { class: "hours-field", key: "{sid_h}-{lvl}",
                                    label { "{lvl}" }
                                    input {
                                        class: "hours-input",
                                        r#type: "number", min: "0", max: "10",
                                        value: "{val}",
                                        oninput: move |e: FormEvent| {
                                            let idx = i;
                                            let mut h = hours_data();
                                            if let Some(entry) = h.get_mut(idx) {
                                                if let Some(obj) = entry.as_object_mut() {
                                                    obj.insert("hours_per_week".to_string(), serde_json::Value::String(e.value()));
                                                }
                                            }
                                            hours_data.set(h);
                                        },
                                    }
                                }
                            });
                        }
                        rows.push(rsx! {
                            tr { key: "{sid_h}-hours",
                                td { colspan: "4", class: "hours-cell",
                                    div { class: "hours-editor",
                                        div { class: "hours-grid", {hour_fields.into_iter()} }
                                    }
                                }
                            }
                        });
                    }
                }
                rows
            }
        }
        Some(Err(e)) => {
            vec![rsx! { tr { td { colspan: "4", class: "empty-state", "Error: {e}" } } }]
        }
        None => vec![
            rsx! { tr { td { colspan: "4", class: "empty-state", div { class: "loading-spinner", "Cargando..." } } } },
        ],
    };

    rsx! {
        div { class: "page-header",
            h1 { "Asignaturas" }
            p { "Catálogo de asignaturas y configuración de horas por nivel" }
        }
        div { class: "page-toolbar",
            button { class: "btn btn-primary", onclick: move |_| { reset(); show_form.set(true); }, "Nueva Asignatura" }
            button { class: "btn", onclick: move |_| { show_import.set(!show_import()); import_result.set(None); }, "Importar CSV" }
        }
        {
            if show_import() {
                rsx! {
                    div { class: "form-card",
                        h3 { "Importar Asignaturas desde CSV" }
                        p { style: "font-size: 0.9em; color: var(--text-secondary);", "Pegue los datos en formato: código, nombre, nivel, horas_semana. Una fila por asignatura." }
                        textarea {
                            class: "form-input",
                            style: "width: 100%; min-height: 120px; font-family: monospace;",
                            value: "{import_text}",
                            placeholder: "MAT01,Matemática,1° Básico,8\nLEN01,Lenguaje,1° Básico,8\nCIE01,Ciencias,1° Básico,4",
                            oninput: move |e| import_text.set(e.value()),
                        }
                        div { class: "form-actions",
                            button { class: "btn btn-primary", disabled: importing(),
                                onclick: move |_| {
                                    importing.set(true);
                                    import_result.set(None);
                                    let rows: Vec<Value> = import_text().lines()
                                        .filter(|l| !l.trim().is_empty())
                                        .filter_map(|l| {
                                            let parts: Vec<&str> = l.split(',').map(|s| s.trim()).collect();
                                            if parts.len() < 2 { return None; }
                                            Some(serde_json::json!({
                                                "code": parts[0],
                                                "name": parts[1],
                                                "level": parts.get(2).map(|s| s.to_string()),
                                                "hours_per_week": parts.get(3).and_then(|s| s.parse::<i32>().ok()),
                                            }))
                                        })
                                        .collect();
                                    let payload = serde_json::json!({ "subjects": rows });
                                    spawn(async move {
                                        let res = client::import_subjects(&payload).await;
                                        importing.set(false);
                                        import_result.set(Some(match res {
                                            Ok(j) => format!("Importadas: {} | Omitidas: {} | Errores: {}",
                                                j["imported"].as_i64().unwrap_or(0),
                                                j["skipped"].as_i64().unwrap_or(0),
                                                j["errors"].as_array().map(|a| a.len()).unwrap_or(0),
                                            ),
                                            Err(e) => format!("Error: {}", e),
                                        }));
                                        subjects.restart();
                                    });
                                },
                                if importing() { "Importando..." } else { "Importar" }
                            }
                            button { class: "btn", onclick: move |_| { show_import.set(false); import_text.set(String::new()); }, "Cancelar" }
                        }
                        {
                            match import_result() {
                                Some(msg) => rsx! { div { class: "info-card", p { "{msg}" } } },
                                None => rsx! {},
                            }
                        }
                    }
                }
            } else { rsx! {} }
        }
        if show_form() {
            div { class: "card form-card",
                h3 { if editing_id().is_some() { "Editar Asignatura" } else { "Nueva Asignatura" } }
                div { class: "form-grid",
                    div { class: "field", label { "Código SIGE" } input { class: "form-input", placeholder: "MAT01", value: "{code}", oninput: move |e| code.set(e.value()) } }
                    div { class: "field", label { "Nombre" } input { class: "form-input", placeholder: "Matemática", value: "{name}", oninput: move |e| name.set(e.value()) } }
                }
                div { class: "form-actions",
                    button { class: "btn-secondary", onclick: move |_| reset(), "Cancelar" }
                    button { class: "btn-primary", onclick: move |_| {
                        saving.set(true);
                        let payload = serde_json::json!({ "code": code(), "name": name(), "level": serde_json::Value::Null, "hours_per_week": 0 });
                        let is_edit = editing_id().is_some();
                        let endpoint = if let Some(ref id) = editing_id() { format!("/api/grades/subjects/{}", id) } else { "/api/grades/subjects".to_string() };
                        spawn(async move {
                            if is_edit { let _ = client::put_json(&endpoint, &payload).await; }
                            else { let _ = client::post_json(&endpoint, &payload).await; }
                            saving.set(false); reset(); subjects.restart();
                        });
                    }, disabled: saving(),
                        if saving() { "Guardando..." } else { "Guardar" }
                    }
                }
            }
        }
        div { class: "data-table-container",
            table { class: "data-table",
                thead { tr { th { "Código" } th { "Nombre" } th { "Estado" } th { "Acciones" } } }
                tbody { {table_rows.into_iter()} }
            }
        }
        if expanded_id().is_some() {
            div { class: "page-toolbar", style: "margin-top: 8px",
                button { class: "btn-primary", onclick: move |_| {
                    saving_hours.set(true);
                    let sid = expanded_id();
                    let raw = hours_data();
                    let hours: Vec<Value> = raw.iter().map(|h| {
                        let lvl = h["level"].as_str().unwrap_or("").to_string();
                        let hrs = h["hours_per_week"].as_str().and_then(|s| s.parse::<i32>().ok()).or_else(|| h["hours_per_week"].as_i64().map(|v| v as i32)).unwrap_or(0);
                        serde_json::json!({ "level": lvl, "hours_per_week": hrs })
                    }).collect();
                    spawn(async move {
                        if let Some(id) = sid {
                            let _ = client::put_json(&format!("/api/grades/subjects/{}/hours", id), &serde_json::json!({ "hours": hours })).await;
                        }
                        saving_hours.set(false); subjects.restart();
                    });
                }, disabled: saving_hours(),
                    if saving_hours() { "Guardando..." } else { "Guardar Horas por Nivel" }
                }
            }
        }
    }
}
