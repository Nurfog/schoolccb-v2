use dioxus::prelude::*;
use serde_json::json;

use crate::api::client;

fn first_letter(s: &str) -> String {
    s.chars().next().map(|c| c.to_string()).unwrap_or_else(|| "?".to_string())
}

const AFP_LIST: &[&str] = &["Capital", "Cuprum", "Habitat", "Modelo", "Planvital", "Provida", "Uno"];
const CONTRACT_TYPES: &[&str] = &["Indefinido", "PlazoFijo", "Honorarios", "PrestacionServicios"];

#[component]
pub fn HrPage() -> Element {
    let mut employees = use_resource(|| client::fetch_json("/api/hr/employees"));
    let mut search = use_signal(String::new);
    let mut show_form = use_signal(|| false);

    let mut rut = use_signal(String::new);
    let mut first_name = use_signal(String::new);
    let mut last_name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut position = use_signal(String::new);
    let mut category = use_signal(|| "Docente".to_string());
    let mut hire_date = use_signal(String::new);

    let mut pension_fund = use_signal(|| "Provida".to_string());
    let mut health_system = use_signal(|| "Fonasa".to_string());
    let mut health_plan = use_signal(String::new);
    let mut health_amount = use_signal(|| "0".to_string());

    let mut contract_type = use_signal(|| "Indefinido".to_string());
    let mut salary_base = use_signal(|| "0".to_string());
    let mut weekly_hours = use_signal(|| "40".to_string());
    let mut ley_karin = use_signal(|| false);

    let mut saving = use_signal(|| false);

    let do_create = move |_| {
        if rut().trim().is_empty() || first_name().trim().is_empty() || last_name().trim().is_empty() {
            return;
        }
        saving.set(true);
        let payload = json!({
            "rut": rut(),
            "first_name": first_name(),
            "last_name": last_name(),
            "email": email(),
            "phone": phone(),
            "position": position(),
            "category": category(),
            "hire_date": hire_date(),
        });
        let p_fund = pension_fund();
        let h_sys = health_system();
        let h_plan = health_plan();
        let h_amt = health_amount();
        let c_type = contract_type();
        let s_base = salary_base();
        let w_hours = weekly_hours();
        let l_karin = ley_karin();
        spawn(async move {
            if let Ok(resp) = client::post_json("/api/hr/employees", &payload).await {
                let emp_id = resp["id"].as_str().unwrap_or("");
                if !emp_id.is_empty() {
                    let pf_payload = json!({
                        "pension_fund": p_fund,
                        "health_system": h_sys,
                        "health_plan_name": if h_sys == "Isapre" { h_plan } else { String::new() },
                        "health_fixed_amount": if h_sys == "Isapre" { h_amt.parse::<f64>().unwrap_or(0.0) } else { 0.0 },
                    });
                    let _ = client::post_json(&format!("/api/hr/employees/{}/pension-fund", emp_id), &pf_payload).await;

                    let contract_payload = json!({
                        "contract_type": c_type,
                        "salary_base": s_base.parse::<f64>().unwrap_or(0.0),
                        "weekly_hours": w_hours.parse::<i32>().unwrap_or(40),
                        "ley_karin_signed": l_karin,
                        "start_date": hire_date(),
                    });
                    let _ = client::post_json(&format!("/api/hr/employees/{}/contracts", emp_id), &contract_payload).await;
                }
            }
            saving.set(false);
            show_form.set(false);
            rut.set(String::new()); first_name.set(String::new()); last_name.set(String::new());
            email.set(String::new()); phone.set(String::new()); position.set(String::new());
            pension_fund.set("Provida".to_string()); health_system.set("Fonasa".to_string());
            health_plan.set(String::new()); health_amount.set("0".to_string());
            contract_type.set("Indefinido".to_string()); salary_base.set("0".to_string());
            weekly_hours.set("40".to_string()); ley_karin.set(false);
            employees.restart();
        });
    };

    let do_search = move |e: FormEvent| {
        let q = e.value();
        search.set(q.clone());
        if q.len() >= 2 || q.is_empty() {
            employees.restart();
        }
    };

    rsx! {
        div { class: "page-header",
            h1 { "Recursos Humanos" }
            p { "Gestión de empleados, contratación con Ley Karin, previsión y remuneraciones" }
        }
        div { class: "page-toolbar",
            input { class: "search-input", value: "{search}", oninput: do_search, placeholder: "Buscar por RUT, nombre..." }
            button { class: "btn btn-primary", onclick: move |_| show_form.set(!show_form()),
                if show_form() { "Cancelar" } else { "Nuevo Empleado" }
            }
        }
        {
            if show_form() {
                rsx! {
                    div { class: "form-card",
                        div { class: "form-card-header",
                            h3 { "Nueva Contratación" }
                            span { class: "form-card-badge", "Ley Karin" }
                        }
                        div { class: "form-section",
                            div { class: "form-section-title", "Datos Personales" }
                            div { class: "form-row",
                                div { class: "form-group",
                                    label { "RUT *" }
                                    input { class: "form-input", value: "{rut}", oninput: move |e| rut.set(e.value()), placeholder: "12.345.678-9" }
                                }
                                div { class: "form-group",
                                    label { "Categoría *" }
                                    select { class: "form-input", value: "{category}", onchange: move |e| category.set(e.value()),
                                        option { value: "Docente", "Docente" }
                                        option { value: "Directivo", "Directivo" }
                                        option { value: "Administrativo", "Administrativo" }
                                        option { value: "Asistente", "Asistente" }
                                        option { value: "AsistenteEducacion", "Asistente Educ." }
                                        option { value: "Enfermeria", "Enfermería" }
                                        option { value: "Psicologia", "Psicología" }
                                        option { value: "Psicopedagogia", "Psicopedagogía" }
                                        option { value: "Auxiliar", "Auxiliar" }
                                        option { value: "Otro", "Otro" }
                                    }
                                }
                            }
                            div { class: "form-row",
                                div { class: "form-group",
                                    label { "Nombres *" }
                                    input { class: "form-input", value: "{first_name}", oninput: move |e| first_name.set(e.value()), placeholder: "Juan" }
                                }
                                div { class: "form-group",
                                    label { "Apellidos *" }
                                    input { class: "form-input", value: "{last_name}", oninput: move |e| last_name.set(e.value()), placeholder: "Pérez" }
                                }
                            }
                            div { class: "form-row",
                                div { class: "form-group",
                                    label { "Email" }
                                    input { class: "form-input", value: "{email}", oninput: move |e| email.set(e.value()), placeholder: "juan@colegio.cl" }
                                }
                                div { class: "form-group",
                                    label { "Teléfono" }
                                    input { class: "form-input", value: "{phone}", oninput: move |e| phone.set(e.value()), placeholder: "+56 9 1234 5678" }
                                }
                            }
                            div { class: "form-row",
                                div { class: "form-group",
                                    label { "Cargo" }
                                    input { class: "form-input", value: "{position}", oninput: move |e| position.set(e.value()), placeholder: "Profesor de Matemáticas" }
                                }
                                div { class: "form-group",
                                    label { "Fecha de Contratación" }
                                    input { class: "form-input", value: "{hire_date}", oninput: move |e| hire_date.set(e.value()), r#type: "date" }
                                }
                            }
                        }
                        div { class: "form-section",
                            div { class: "form-section-title", "Previsión y Salud (Leyes Sociales)" }
                            div { class: "form-row",
                                div { class: "form-group",
                                    label { "AFP" }
                                    select { class: "form-input", value: "{pension_fund}", onchange: move |e| pension_fund.set(e.value()),
                                        {AFP_LIST.iter().map(|afp| rsx! {
                                            option { value: "{afp}", "{afp}" }
                                        })}
                                    }
                                }
                                div { class: "form-group",
                                    label { "Sistema de Salud" }
                                    select { class: "form-input", value: "{health_system}", onchange: move |e| {
                                        let v = e.value();
                                        health_system.set(v.clone());
                                        if v == "Fonasa" { health_amount.set("0".to_string()); }
                                    },
                                        option { value: "Fonasa", "Fonasa (7%)" }
                                        option { value: "Isapre", "Isapre" }
                                    }
                                }
                            }
                            { if health_system() == "Isapre" {
                                rsx! {
                                    div { class: "form-row",
                                        div { class: "form-group",
                                            label { "Plan de Salud" }
                                            input { class: "form-input", value: "{health_plan}", oninput: move |e| health_plan.set(e.value()), placeholder: "Ej: Más Salud" }
                                        }
                                        div { class: "form-group",
                                            label { "Valor UF (monto fijo)" }
                                            input { class: "form-input", value: "{health_amount}", oninput: move |e| health_amount.set(e.value()), r#type: "number", min: "0", placeholder: "0" }
                                        }
                                    }
                                }
                            } else { rsx! {} }}
                        }
                        div { class: "form-section",
                            div { class: "form-section-title", "Contrato Inicial" }
                            div { class: "form-row",
                                div { class: "form-group",
                                    label { "Tipo de Contrato" }
                                    select { class: "form-input", value: "{contract_type}", onchange: move |e| contract_type.set(e.value()),
                                        {CONTRACT_TYPES.iter().map(|ct| rsx! {
                                            option { value: "{ct}", "{ct}" }
                                        })}
                                    }
                                }
                                div { class: "form-group",
                                    label { "Sueldo Base $" }
                                    input { class: "form-input", value: "{salary_base}", oninput: move |e| salary_base.set(e.value()), r#type: "number", min: "0", placeholder: "500000" }
                                }
                                div { class: "form-group",
                                    label { "Horas Semanales" }
                                    input { class: "form-input", value: "{weekly_hours}", oninput: move |e| weekly_hours.set(e.value()), r#type: "number", min: "1", max: "45", placeholder: "40" }
                                }
                            }
                            div { class: "form-row",
                                label { class: "checkbox-label",
                                    input { r#type: "checkbox", checked: ley_karin, oninput: move |_| ley_karin.set(!ley_karin()) }
                                    " Firmar declaración Ley Karin (Ley 21.643)"
                                }
                            }
                        }
                        div { class: "form-actions",
                            button { class: "btn btn-secondary", onclick: move |_| show_form.set(false), "Cancelar" }
                            button { class: "btn btn-primary", disabled: saving() || rut().trim().is_empty() || first_name().trim().is_empty() || last_name().trim().is_empty(), onclick: do_create,
                                if saving() { "Guardando..." } else { "Crear Empleado y Contrato" }
                            }
                        }
                    }
                }
            } else { rsx! {} }
        }
        div { class: "data-table-container",
            match employees() {
                Some(Ok(data)) => {
                    let list = data["employees"].as_array().cloned().unwrap_or_default();
                    if list.is_empty() {
                        rsx! { div { class: "empty-state", "No hay empleados registrados" } }
                    } else {
                        let rows: Vec<Element> = list.iter().map(|emp| {
                            let id = emp["id"].as_str().unwrap_or("").to_string();
                            let rut = emp["rut"].as_str().unwrap_or("").to_string();
                            let name = format!("{} {}",
                                emp["first_name"].as_str().unwrap_or(""),
                                emp["last_name"].as_str().unwrap_or("")
                            );
                            let cat = emp["category"].as_str().unwrap_or("—").to_string();
                            let pos = emp["position"].as_str().unwrap_or("—").to_string();
                            let active = emp["active"].as_bool().unwrap_or(true);
                            let avatar = first_letter(&name);
                            rsx! {
                                tr { class: "clickable-row", onclick: move |_| {
                                    let nav = navigator();
                                    nav.push(format!("/hr/{}", id));
                                },
                                    td { div { class: "employee-cell",
                                        div { class: "emp-avatar-small", "{avatar}" }
                                        span { class: "rut-badge", "{rut}" }
                                    }}
                                    td { "{name}" }
                                    td { span { class: "role-badge", "{cat}" } }
                                    td { "{pos}" }
                                    td {
                                        if active { span { class: "status-active", "Activo" } }
                                        else { span { class: "status-inactive", "Inactivo" } }
                                    }
                                }
                            }
                        }).collect();
                        rsx! {
                            table { class: "data-table",
                                thead { tr {
                                    th { "RUT" }
                                    th { "Nombre" }
                                    th { "Categoría" }
                                    th { "Cargo" }
                                    th { "Estado" }
                                }}
                                tbody { { rows.into_iter() } }
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! { div { class: "empty-state", "Error: {e}" } },
                None => rsx! { div { class: "loading-spinner", "Cargando..." } },
            }
        }
    }
}
