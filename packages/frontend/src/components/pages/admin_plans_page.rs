use dioxus::prelude::*;
use serde_json::{Value, json};

use crate::api::client;

const MODULE_DEFS: &[(&str, &str, &[&str])] = &[
    ("dashboard", "Dashboard", &[]),
    ("students", "Gestión de Alumnos", &["view", "create", "edit", "delete", "import", "export"]),
    ("attendance", "Asistencia", &["records", "reports", "alerts", "modify"]),
    ("grades", "Calificaciones", &["view", "create", "edit", "delete", "periods", "categories", "reports"]),
    ("hr", "Recursos Humanos", &["employees", "contracts", "documents", "leaves"]),
    ("payroll", "Remuneraciones", &["view", "calculate", "export"]),
    ("finance", "Finanzas", &["fees", "payments", "scholarships"]),
    ("admission", "Admisión CRM", &["prospects", "stages", "documents", "activities", "classrooms", "metrics"]),
    ("reports", "Reportes", &["certificates", "concentrations", "final-records", "sige"]),
    ("notifications", "Centro de Mensajería", &["send", "view", "manage"]),
    ("sige", "SIGE / MINEDUC", &["export"]),
    ("complaints", "Ley Karin - Denuncias", &["view", "manage", "resolve"]),
    ("users", "Usuarios y Perfiles", &["view", "create", "edit", "delete"]),
    ("roles", "Roles y Permisos", &["view", "create", "edit", "delete", "assign"]),
    ("config", "Configuración", &["branding", "preferences", "general"]),
    ("corporations", "Multi-colegio", &["view", "create", "edit", "toggle"]),
    ("courses", "Cursos", &["view", "create", "edit", "delete"]),
    ("enrollments", "Matrículas", &["view", "create", "edit", "delete", "manage"]),
    ("subjects", "Asignaturas", &["view", "create", "edit", "delete"]),
    ("grade-levels", "Niveles", &["view", "create", "edit", "delete"]),
    ("academic-years", "Años Académicos", &["view", "create", "edit", "delete", "activate"]),
    ("classrooms", "Salas", &["view", "create", "edit", "delete"]),
    ("agenda", "Agenda Escolar", &["events", "view", "manage"]),
    ("audit", "Auditoría", &["view", "export"]),
    ("my-portal", "Portal Auto-consulta", &["view"]),
];

#[component]
pub fn AdminPlansPage() -> Element {
    let mut active_tab = use_signal(|| "plans".to_string());
    let mut plans = use_resource(|| client::admin_list_plans());
    let mut show_form = use_signal(|| false);
    let mut edit_id = use_signal(|| None::<String>);
    let mut name = use_signal(String::new);
    let mut desc = use_signal(String::new);
    let mut price_m = use_signal(|| "".to_string());
    let mut price_y = use_signal(|| "".to_string());
    let mut featured = use_signal(|| false);
    let mut is_custom = use_signal(|| false);
    let mut show_in_portal = use_signal(|| true);
    let mut saving = use_signal(|| false);
    let mut mods = use_signal(|| vec![false; MODULE_DEFS.len()]);
    let mut sub_mods = use_signal(|| {
        MODULE_DEFS.iter().map(|(_, _, subs)| vec![false; subs.len()]).collect::<Vec<_>>()
    });
    let expanded = use_signal(|| vec![false; MODULE_DEFS.len()]);

    let mut open_edit = move |p: Value| {
        edit_id.set(p["id"].as_str().map(|s| s.to_string()));
        name.set(p["name"].as_str().unwrap_or("").to_string());
        desc.set(p["description"].as_str().unwrap_or("").to_string());
        price_m.set(p["price_monthly"].as_f64().unwrap_or(0.0).to_string());
        price_y.set(p["price_yearly"].as_f64().unwrap_or(0.0).to_string());
        featured.set(p["featured"].as_bool().unwrap_or(false));
        is_custom.set(p["is_custom"].as_bool().unwrap_or(false));
        show_in_portal.set(p["show_in_portal"].as_bool().unwrap_or(true));

        let mut new_mods = vec![false; MODULE_DEFS.len()];
        let mut new_sub_mods: Vec<Vec<bool>> = MODULE_DEFS.iter().map(|(_, _, subs)| vec![false; subs.len()]).collect();
        if let Some(modules) = p["modules"].as_array() {
            for m in modules {
                if let Some(mk) = m["module_key"].as_str() {
                    if let Some(idx) = MODULE_DEFS.iter().position(|(k, _, _)| *k == mk) {
                        new_mods[idx] = true;
                        if let Some(subs) = m["sub_modules"].as_array() {
                            for sv in subs {
                                if let Some(sk) = sv.as_str() {
                                    if let Some(si) = MODULE_DEFS[idx].2.iter().position(|s| *s == sk) {
                                        new_sub_mods[idx][si] = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        mods.set(new_mods);
        sub_mods.set(new_sub_mods);
        show_form.set(true);
    };

    let do_save = move |_| {
        let n = name();
        let d = desc();
        let pm = price_m();
        let py = price_y();
        let f = featured();
        let ic = is_custom();
        let sp = show_in_portal();
        let eid = edit_id();
        let mvals = mods();
        let smvals = sub_mods();
        if n.trim().is_empty() { return; }
        saving.set(true);
        spawn(async move {
            let payload = json!({
                "name": n, "description": d,
                "price_monthly": pm.parse::<f64>().unwrap_or(0.0),
                "price_yearly": py.parse::<f64>().unwrap_or(0.0),
                "featured": f,
                "is_custom": ic,
                "show_in_portal": sp,
            });
            let result = match &eid {
                Some(id) => client::admin_update_plan(id, &payload).await,
                None => client::admin_create_plan(&payload).await,
            };
            if let Ok(data) = result {
                let pid = eid.as_deref().unwrap_or_else(|| data["id"].as_str().unwrap_or(""));
                if !pid.is_empty() {
                    let enabled: Vec<Value> = MODULE_DEFS.iter().enumerate()
                        .filter(|(i, _)| mvals[*i])
                        .map(|(i, (k, n, subs))| {
                            let selected: Vec<&str> = subs.iter().enumerate()
                                .filter(|(si, _)| smvals[i].get(*si).copied().unwrap_or(false))
                                .map(|(_, s)| *s)
                                .collect();
                            let mut obj = json!({"module_key": k, "module_name": n});
                            if !selected.is_empty() {
                                obj["sub_modules"] = json!(selected);
                            }
                            obj
                        })
                        .collect();
                    let _ = client::admin_set_plan_modules(pid, &json!({"modules": enabled})).await;
                }
            }
            saving.set(false);
            show_form.set(false);
            edit_id.set(None);
            name.set(String::new());
            desc.set(String::new());
            price_m.set("".to_string());
            price_y.set("".to_string());
            featured.set(false);
            is_custom.set(false);
            show_in_portal.set(true);
            plans.restart();
        });
    };

    let do_delete = move |id: String| {
        if !web_sys::window().unwrap().confirm_with_message("¿Estás seguro?").unwrap_or(false) {
            return;
        }
        spawn(async move {
            let _ = client::admin_delete_plan(&id).await;
            plans.restart();
        });
    };

    rsx! {
        div { class: "page-header",
            h1 { "Planes y Licencias" }
            p { "Configuración de planes de precios, módulos y licencias asignadas" }
        }
        div { class: "tab-bar",
            button {
                class: if active_tab() == "plans" { "tab active" } else { "tab" },
                onclick: move |_| active_tab.set("plans".to_string()),
                "Planes"
            }
            button {
                class: if active_tab() == "licenses" { "tab active" } else { "tab" },
                onclick: move |_| active_tab.set("licenses".to_string()),
                "Licencias"
            }
        }
        div { class: "tab-content",
            match active_tab().as_str() {
                "plans" => rsx! {
                    div { class: "page-toolbar",
                        button { class: "btn btn-primary", onclick: move |_| { show_form.set(!show_form()); },
                            if show_form() { "Cancelar" } else { "Nuevo Plan" }
                        }
                    }
                    {if show_form() {
                        rsx! {
                            div { class: "form-card",
                                div { class: "form-card-header",
                                    h3 { if edit_id().is_some() { "Editar Plan" } else { "Nuevo Plan" } }
                                    span { class: "form-card-badge", if edit_id().is_some() { "Editando" } else { "Creación" } }
                                }
                                div { class: "form-section",
                                    div { class: "form-section-title", "Información Básica" }
                                    div { class: "form-row",
                                        div { class: "form-group",
                                            label { "Nombre del plan" }
                                            input { class: "form-input", value: "{name}", oninput: move |e| name.set(e.value()), placeholder: "Ej: Básico, Profesional..." }
                                        }
                                        div { class: "form-group",
                                            label { "Descripción" }
                                            input { class: "form-input", value: "{desc}", oninput: move |e| desc.set(e.value()), placeholder: "Breve descripción del plan" }
                                        }
                                    }
                                }
                                div { class: "form-section",
                                    div { class: "form-section-title", "Precios" }
                                    div { class: "form-row",
                                        div { class: "form-group",
                                            label { "Precio mensual" }
                                            input { class: "form-input", r#type: "number", step: "any", value: "{price_m}", oninput: move |e| price_m.set(e.value()), placeholder: "0" }
                                        }
                                        div { class: "form-group",
                                            label { "Precio anual" }
                                            input { class: "form-input", r#type: "number", step: "any", value: "{price_y}", oninput: move |e| price_y.set(e.value()), placeholder: "0" }
                                        }
                                    }
                                }
                                div { class: "form-section",
                                    div { class: "form-section-title", "Visibilidad" }
                                    div { class: "checkbox-row",
                                        label { class: "checkbox-label",
                                            input { r#type: "checkbox", checked: featured, oninput: move |_| featured.set(!featured()) }
                                            " Destacado"
                                        }
                                        label { class: "checkbox-label",
                                            input { r#type: "checkbox", checked: is_custom, oninput: move |_| {
                                                let new_val = !is_custom();
                                                is_custom.set(new_val);
                                                if new_val { show_in_portal.set(false); }
                                            }}
                                            " Personalizado"
                                        }
                                        label { class: "checkbox-label",
                                            input { r#type: "checkbox", checked: show_in_portal, oninput: move |_| show_in_portal.set(!show_in_portal()) }
                                            " Mostrar en portal"
                                        }
                                    }
                                }
                                div { class: "form-section",
                                    div { class: "form-section-title", "Módulos incluidos" }
                                    {plan_modules_section(mods, sub_mods, expanded)}
                                }
                                div { class: "form-actions",
                                    button { class: "btn btn-secondary", onclick: move |_| show_form.set(false), "Cancelar" }
                                    button { class: "btn btn-primary", disabled: saving(), onclick: do_save,
                                        if saving() { "Guardando..." } else { "Guardar Plan" }
                                    }
                                }
                            }
                        }
                    } else { rsx! {} }}
                    div { class: "data-table-container",
                        match plans() {
                            Some(Ok(data)) => {
                                let list = data["plans"].as_array().cloned().unwrap_or_default();
                                if list.is_empty() {
                                    rsx! { p { class: "empty-state", "No hay planes configurados" } }
                                } else {
                                    rsx! {
                                        table { class: "data-table",
                                            thead {
                                                tr {
                                                    th { "Nombre" } th { "Descripción" } th { "Mensual" } th { "Anual" }
                                                    th { "Destacado" } th { "Custom" } th { "Portal" } th { "Activo" } th { "Acciones" }
                                                }
                                            }
                                            tbody {
                                                {list.into_iter().map(|p| {
                                                    let pid = p["id"].as_str().unwrap_or("").to_string();
                                                    let pname = p["name"].as_str().unwrap_or("").to_string();
                                                    let pdesc = p["description"].as_str().unwrap_or("").to_string();
                                                    let pmonthly = p["price_monthly"].as_f64().unwrap_or(0.0);
                                                    let pyearly = p["price_yearly"].as_f64().unwrap_or(0.0);
                                                    let pfeat = p["featured"].as_bool().unwrap_or(false);
                                                    let pcust = p["is_custom"].as_bool().unwrap_or(false);
                                                    let pportal = p["show_in_portal"].as_bool().unwrap_or(true);
                                                    let pact = p["active"].as_bool().unwrap_or(true);
                                                    rsx! {
                                                        tr { key: "{pid}",
                                                            td { "{pname}" } td { "{pdesc}" }
                                                            td { "${pmonthly:.0}" } td { "${pyearly:.0}" }
                                                            td { if pfeat { "⭐" } else { "—" } }
                                                            td { if pcust { span { class: "badge badge-info", "Custom" } } else { "—" } }
                                                            td { if pportal { span { class: "badge badge-success", "Sí" } } else { span { class: "badge badge-warning", "No" } } }
                                                            td { if pact { span { class: "badge badge-success", "Activo" } } else { span { class: "badge badge-warning", "Inactivo" } } }
                                                            td {
                                                                button { class: "btn btn-sm", onclick: { let p = p.clone(); move |_| open_edit(p.clone()) }, "Editar" }
                                                                button { class: "btn btn-sm btn-danger", onclick: { let pid = pid.clone(); move |_| do_delete(pid.clone()) }, "Eliminar" }
                                                            }
                                                        }
                                                    }
                                                })}
                                            }
                                        }
                                    }
                                }
                            }
                            Some(Err(e)) => rsx! { p { class: "state-error", "Error: {e}" } },
                            None => rsx! { div { class: "loading-spinner", "Cargando..." } },
                        }
                    }
                },
                "licenses" => rsx! { LicenseManager {} },
                _ => rsx! {},
            }
        }
    }
}

fn plan_modules_section(
    mods: Signal<Vec<bool>>,
    sub_mods: Signal<Vec<Vec<bool>>>,
    expanded: Signal<Vec<bool>>,
) -> Element {
    let tree_items: Vec<Element> = MODULE_DEFS.iter().enumerate().map(|(i, (key, label, subs))| {
        let has_subs = !subs.is_empty();
        let child_items: Vec<Element> = if has_subs {
            subs.iter().enumerate().map(|(j, sub)| {
                rsx! {
                    label { key: "{key}-{sub}", class: "tree-checkbox sub-module",
                        input { r#type: "checkbox", checked: sub_mods()[i][j], oninput: move |_| { let mut sm = sub_mods; sm.with_mut(|s| s[i][j] = !s[i][j]); } }
                        span { "{sub}" }
                    }
                }
            }).collect()
        } else {
            vec![]
        };
        let show_children = has_subs && expanded()[i];
        rsx! {
            div { key: "{key}", class: "module-tree-item",
                div { class: "module-tree-row",
                    if has_subs {
                        button { class: "tree-toggle", onclick: move |_| { let mut e = expanded; e.with_mut(|e| e[i] = !e[i]); }, "aria-label": if expanded()[i] { "Contraer" } else { "Expandir" },
                            svg { role: "presentation", view_box: "0 0 24 24", width: "16", height: "16",
                                if expanded()[i] {
                                    path { d: "M6 9l6 6 6-6", fill: "none", stroke: "currentColor", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                                } else {
                                    path { d: "M9 18l6-6-6-6", fill: "none", stroke: "currentColor", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round" }
                                }
                            }
                        }
                    } else {
                        div { class: "tree-spacer" }
                    }
                    label { class: "tree-checkbox",
                        input { r#type: "checkbox", checked: mods()[i], oninput: move |_| {
                            let mut m = mods;
                            let mut sm = sub_mods;
                            let current = m()[i];
                            m.with_mut(|m| m[i] = !current);
                            if current {
                                sm.with_mut(|s| { if i < s.len() { s[i].iter_mut().for_each(|x| *x = false); } });
                            } else {
                                sm.with_mut(|s| { if i < s.len() { s[i].iter_mut().for_each(|x| *x = true); } });
                            }
                        } }
                        span { "{label}" }
                    }
                }
                if show_children {
                    div { class: "module-tree-children",
                        {child_items.into_iter()}
                    }
                }
            }
        }
    }).collect();

    rsx! {
        div { class: "form-group",
            label { "Módulos incluidos:" }
            div { class: "module-tree",
                {tree_items.into_iter()}
            }
        }
    }
}

#[component]
fn LicenseManager() -> Element {
    let mut licenses = use_resource(|| client::admin_list_licenses());
    let mut toggling = use_signal(|| None::<String>);

    let mut do_toggle = move |id: String, current_active: bool| {
        let msg = if current_active {
            "¿Desactivar esta licencia?"
        } else {
            "¿Activar esta licencia?"
        };
        if !web_sys::window().unwrap().confirm_with_message(msg).unwrap_or(false) {
            return;
        }
        toggling.set(Some(id.clone()));
        spawn(async move {
            let _ = client::admin_update_license_status(&id, &serde_json::json!({ "active": !current_active })).await;
            toggling.set(None);
            licenses.restart();
        });
    };

    rsx! {
        div { class: "page-toolbar",
            button { class: "btn btn-secondary", onclick: move |_| licenses.restart(), "Recargar" }
        }
        div { class: "data-table-container",
            match licenses() {
                Some(Ok(data)) => {
                    let list = data["licenses"].as_array().cloned().unwrap_or_default();
                    if list.is_empty() {
                        rsx! { p { class: "empty-state", "No hay licencias asignadas" } }
                    } else {
                        rsx! {
                            table { class: "data-table",
                                thead {
                                    tr {
                                        th { "Escuela" }
                                        th { "Plan" }
                                        th { "Vence" }
                                        th { "Máx. Alumnos" }
                                        th { "Estado" }
                                        th { "Acciones" }
                                    }
                                }
                                tbody {
                                    {list.into_iter().map(|lic| {
                                        let lid = lic["id"].as_str().unwrap_or("").to_string();
                                        let school = lic["school_name"].as_str().unwrap_or("—").to_string();
                                        let plan = lic["plan_name"].as_str().unwrap_or("—").to_string();
                                        let expires = lic["expires_at"].as_str().unwrap_or("—").to_string();
                                        let max_students = lic["max_students"].as_i64().unwrap_or(0);
                                        let active = lic["active"].as_bool().unwrap_or(false);
                                        let is_toggling = toggling() == Some(lid.clone());
                                        rsx! {
                                            tr { key: "{lid}",
                                                td { "{school}" }
                                                td { "{plan}" }
                                                td { "{expires}" }
                                                td { "{max_students}" }
                                                td {
                                                    if active {
                                                        span { class: "badge badge-success", "Activa" }
                                                    } else {
                                                        span { class: "badge badge-warning", "Inactiva" }
                                                    }
                                                }
                                                td {
                                                    button {
                                                        class: if active { "btn btn-sm btn-secondary" } else { "btn btn-sm btn-primary" },
                                                        disabled: is_toggling,
                                                        onclick: { let lid = lid.clone(); move |_| do_toggle(lid.clone(), active) },
                                                        if is_toggling { "..." } else if active { "Desactivar" } else { "Activar" }
                                                    }
                                                }
                                            }
                                        }
                                    })}
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! { p { class: "state-error", "Error: {e}" } },
                None => rsx! { div { class: "loading-spinner", "Cargando..." } },
            }
        }
    }
}
