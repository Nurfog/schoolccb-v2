use dioxus::prelude::*;
use serde_json::Value;
use crate::api::client;

fn builtin_module_keys() -> Vec<(&'static str, &'static str)> {
    vec![
        ("dashboard", "Dashboard"),
        ("students", "Gestión de Alumnos"),
        ("attendance", "Asistencia"),
        ("grades", "Calificaciones"),
        ("courses", "Cursos"),
        ("enrollments", "Matrículas"),
        ("subjects", "Asignaturas"),
        ("agenda", "Agenda Escolar"),
        ("notifications", "Centro de Mensajería"),
        ("reports", "Reportes"),
        ("finance", "Finanzas"),
        ("users", "Usuarios y Perfiles"),
        ("roles", "Roles y Permisos"),
        ("hr", "Recursos Humanos"),
        ("payroll", "Remuneraciones"),
        ("my-portal", "Mi Portal"),
        ("admission", "Admisiones"),
        ("grade-levels", "Niveles"),
        ("classrooms", "Salas"),
        ("academic-years", "Años Académicos"),
        ("academic-calendar", "Calendario Académico"),
        ("teacher-schedules", "Horarios Docentes"),
        ("parent-portal", "Portal Apoderados"),
        ("student-portal", "Portal Alumnos"),
        ("parent-meetings", "Reuniones Apoderados"),
        ("sige", "SIGE — Exportación MINEDUC"),
        ("complaints", "Ley Karin — Denuncias"),
        ("complementary-subjects", "Asignaturas Complementarias"),
        ("config", "Configuración General"),
        ("audit", "Auditoría"),
        ("corporations", "Corporaciones y Colegios"),
        ("curriculum", "Currículum Nacional"),
        ("sales", "CRM de Ventas"),
        ("b2b-hr", "Recursos Humanos B2B"),
        ("b2b-roles", "Roles y Permisos B2B"),
        ("license-portal", "Portal de Licencias"),
        ("sostenedor", "Panel Sostenedor"),
    ]
}

#[component]
pub fn B2bLicensePlansPage() -> Element {
    let mut plans = use_resource(|| client::admin_list_plans());
    let mut show_create = use_signal(|| false);
    let mut editing_id = use_signal(|| None::<String>);
    let mut plan_name = use_signal(String::new);
    let mut plan_desc = use_signal(String::new);
    let mut plan_monthly = use_signal(|| "0".to_string());
    let mut plan_yearly = use_signal(|| "0".to_string());
    let mut plan_featured = use_signal(|| false);
    let mut plan_sort = use_signal(|| "0".to_string());
    let mut plan_active = use_signal(|| true);
    let mut plan_show_portal = use_signal(|| true);
    let mut plan_modules: Signal<Vec<(String, String, bool)>> = use_signal(|| {
        builtin_module_keys().iter().map(|(k, n)| (k.to_string(), n.to_string(), false)).collect()
    });
    let mut saving = use_signal(|| false);
    let mut msg = use_signal(|| None::<String>);

    let mut reset_form = move || {
        plan_name.set(String::new());
        plan_desc.set(String::new());
        plan_monthly.set("0".to_string());
        plan_yearly.set("0".to_string());
        plan_featured.set(false);
        plan_sort.set("0".to_string());
        plan_active.set(true);
        plan_show_portal.set(true);
        plan_modules.set(
            builtin_module_keys().iter().map(|(k, n)| (k.to_string(), n.to_string(), false)).collect()
        );
        editing_id.set(None);
        show_create.set(false);
        msg.set(None);
    };

    let start_edit = move |id: String| {
        let id_c = id.clone();
        spawn(async move {
            match client::admin_get_plan(&id_c).await {
                Ok(data) => {
                    plan_name.set(data["name"].as_str().unwrap_or("").to_string());
                    plan_desc.set(data["description"].as_str().unwrap_or("").to_string());
                    plan_monthly.set(format!("{:.0}", data["price_monthly"].as_f64().unwrap_or(0.0)));
                    plan_yearly.set(format!("{:.0}", data["price_yearly"].as_f64().unwrap_or(0.0)));
                    plan_featured.set(data["featured"].as_bool().unwrap_or(false));
                    plan_sort.set(data["sort_order"].as_i64().unwrap_or(0).to_string());
                    plan_active.set(data["active"].as_bool().unwrap_or(true));
                    plan_show_portal.set(data["show_in_portal"].as_bool().unwrap_or(true));
                    let existing: std::collections::HashMap<String, bool> = data["modules"].as_array().map(|arr| {
                        arr.iter().filter_map(|m| {
                            let k = m["module_key"].as_str()?;
                            let inc = m["included"].as_bool().unwrap_or(false);
                            Some((k.to_string(), inc))
                        }).collect()
                    }).unwrap_or_default();
                    plan_modules.set(
                        builtin_module_keys().iter().map(|(k, n)| {
                            let inc = existing.get(*k).copied().unwrap_or(false);
                            (k.to_string(), n.to_string(), inc)
                        }).collect()
                    );
                    editing_id.set(Some(id_c));
                    show_create.set(true);
                    msg.set(None);
                }
                Err(e) => msg.set(Some(format!("Error: {e}"))),
            }
        });
    };

    let do_save = move |_| {
        saving.set(true);
        msg.set(None);
        let modules: Vec<Value> = plan_modules().iter().map(|(k, n, inc)| {
            serde_json::json!({"module_key": k, "module_name": n, "included": inc})
        }).collect();
        let payload = serde_json::json!({
            "name": plan_name(),
            "description": plan_desc(),
            "price_monthly": plan_monthly().parse::<f64>().unwrap_or(0.0),
            "price_yearly": plan_yearly().parse::<f64>().unwrap_or(0.0),
            "featured": plan_featured(),
            "sort_order": plan_sort().parse::<i32>().unwrap_or(0),
            "is_custom": false,
            "show_in_portal": plan_show_portal(),
            "modules": modules,
        });
        let is_edit = editing_id().is_some();
        spawn(async move {
            let result = if is_edit {
                let id = editing_id().unwrap_or_default();
                let plan_result = client::admin_update_plan(&id, &payload).await;
                let mod_result = client::admin_set_plan_modules(&id, &payload).await;
                plan_result.and(mod_result)
            } else {
                client::admin_create_plan(&payload).await
            };
            match result {
                Ok(_) => { reset_form(); plans.restart(); }
                Err(e) => msg.set(Some(format!("Error: {e}"))),
            }
            saving.set(false);
        });
    };

    let do_delete = move |id: String| {
        if !web_sys::window().unwrap().confirm_with_message("¿Desactivar este plan? Los planes solo pueden eliminarse si están inactivos.").unwrap_or(false) {
            return;
        }
        spawn(async move {
            let _ = client::admin_update_plan(&id, &serde_json::json!({"active": false})).await;
            plans.restart();
        });
    };

    let mut toggle_module = move |idx: usize| {
        let mut mods = plan_modules();
        if let Some(m) = mods.get_mut(idx) {
            m.2 = !m.2;
        }
        plan_modules.set(mods);
    };

    rsx! {
        div { class: "page-header",
            h1 { "Planes de Licencia" }
            p { "Construcción y administración de planes de licenciamiento" }
        }
        div { class: "page-toolbar",
            button { class: "btn btn-primary", onclick: move |_| if show_create() { reset_form(); } else { reset_form(); show_create.set(true); },
                if show_create() { "Cancelar" } else { "Nuevo Plan" }
            }
        }
        {
            if show_create() {
                rsx! {
                    div { class: "form-card",
                        if let Some(ref m) = msg() {
                            div { class: "alert alert-info", "{m}" }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Nombre:" }
                                input { class: "form-input", value: "{plan_name}", oninput: move |e| plan_name.set(e.value()), placeholder: "Plan Básico" }
                            }
                            div { class: "form-group",
                                label { "Orden:" }
                                input { class: "form-input", value: "{plan_sort}", oninput: move |e| plan_sort.set(e.value()), type: "number", min: "0" }
                            }
                        }
                        div { class: "form-group",
                            label { "Descripción:" }
                            input { class: "form-input", value: "{plan_desc}", oninput: move |e| plan_desc.set(e.value()), placeholder: "Plan ideal para..." }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Precio Mensual (CLP):" }
                                input { class: "form-input", value: "{plan_monthly}", oninput: move |e| plan_monthly.set(e.value()), type: "number", min: "0" }
                            }
                            div { class: "form-group",
                                label { "Precio Anual (CLP):" }
                                input { class: "form-input", value: "{plan_yearly}", oninput: move |e| plan_yearly.set(e.value()), type: "number", min: "0" }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { class: "checkbox-label",
                                    input { class: "checkbox", r#type: "checkbox", checked: plan_featured, oninput: move |_| plan_featured.set(!plan_featured()) }
                                    span { " Destacado" }
                                }
                            }
                            div { class: "form-group",
                                label { class: "checkbox-label",
                                    input { class: "checkbox", r#type: "checkbox", checked: plan_show_portal, oninput: move |_| plan_show_portal.set(!plan_show_portal()) }
                                    span { " Mostrar en portal público" }
                                }
                            }
                            div { class: "form-group",
                                label { class: "checkbox-label",
                                    input { class: "checkbox", r#type: "checkbox", checked: plan_active, oninput: move |_| plan_active.set(!plan_active()) }
                                    span { " Activo" }
                                }
                            }
                        }
                        div { class: "form-section", h4 { "Módulos del Plan" }
                            div { class: "module-grid",
                                for (i, (key, name, included)) in plan_modules().iter().enumerate() {
                                    div { class: "module-check-item",
                                        label { class: "checkbox-label",
                                            input {
                                                class: "checkbox",
                                                r#type: "checkbox",
                                                checked: *included,
                                                oninput: move |_| toggle_module(i),
                                            }
                                            span { "{name}" }
                                            span { class: "module-key", "({key})" }
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "form-actions",
                            button { class: "btn btn-primary", disabled: saving(), onclick: do_save,
                                if saving() { "Guardando..." } else if editing_id().is_some() { "Actualizar Plan" } else { "Crear Plan" }
                            }
                            button { class: "btn", onclick: move |_| reset_form(), "Cancelar" }
                        }
                    }
                }
            } else { rsx! {} }
        }
        div { class: "data-table-container",
            match plans() {
                Some(Ok(data)) => {
                    let list: Vec<Value> = data["plans"].as_array().cloned().unwrap_or_default();
                    let rows: Vec<(String, String, String, f64, f64, bool, bool, i32, usize, usize, String, String)> = list.iter().map(|p| {
                        let modules = p["modules"].as_array().cloned().unwrap_or_default();
                        let mod_count = modules.iter().filter(|m| m["included"].as_bool().unwrap_or(false)).count();
                        let total_mods = modules.len();
                        let active = p["active"].as_bool().unwrap_or(true);
                        let status_cls = if active { "badge badge-success".to_string() } else { "badge badge-danger".to_string() };
                        let status_label = if active { "Activo".to_string() } else { "Inactivo".to_string() };
                        (p["id"].as_str().unwrap_or("").to_string(),
                         p["name"].as_str().unwrap_or("").to_string(),
                         p["description"].as_str().unwrap_or("").to_string(),
                         p["price_monthly"].as_f64().unwrap_or(0.0),
                         p["price_yearly"].as_f64().unwrap_or(0.0),
                         p["featured"].as_bool().unwrap_or(false),
                         active,
                         p["sort_order"].as_i64().unwrap_or(0) as i32,
                         mod_count,
                         total_mods,
                         status_cls,
                         status_label)
                    }).collect();
                    rsx! {
                        table { class: "data-table",
                            thead { tr {
                                th { "Orden" }
                                th { "Nombre" }
                                th { "Precio Mensual" }
                                th { "Precio Anual" }
                                th { "Módulos" }
                                th { "Estado" }
                                th { "Acciones" }
                            }}
                            tbody { for (id, name, desc, pm, py, featured, _active, sort, mod_count, total_mods, status_cls, status_label) in &rows {
                                tr {
                                    td { "{sort}" }
                                    td {
                                        strong { "{name}" }
                                        if !desc.is_empty() { br {} span { class: "text-muted", "{desc}" } }
                                        if *featured { span { class: "badge badge-warning", style: "margin-left: 4px;", "Destacado" } }
                                    }
                                    td { "${pm:.0}" }
                                    td { "${py:.0}" }
                                    td { "{mod_count}/{total_mods} módulos" }
                                    td { span { class: "{status_cls}", "{status_label}" } }
                                    td {
                                        button { class: "btn btn-sm", onclick: { let i = id.clone(); move |_| start_edit(i.clone()) }, "Editar" }
                                        button { class: "btn btn-sm btn-danger", style: "margin-left: 4px;", onclick: { let i = id.clone(); move |_| do_delete(i.clone()) }, "Desactivar" }
                                    }
                                }
                            }}
                        }
                        if rows.is_empty() {
                            div { class: "empty-state", "Sin planes configurados" }
                        }
                    }
                }
                Some(Err(e)) => rsx! { div { class: "empty-state", "Error: {e}" } },
                None => rsx! { div { class: "empty-state", div { class: "loading-spinner", "Cargando..." } } },
            }
        }
    }
}
