use dioxus::prelude::*;
use serde_json::json;

use crate::api::client;

const MONTHS: &[(&str, &str)] = &[
    ("1", "Enero"), ("2", "Febrero"), ("3", "Marzo"), ("4", "Abril"),
    ("5", "Mayo"), ("6", "Junio"), ("7", "Julio"), ("8", "Agosto"),
    ("9", "Septiembre"), ("10", "Octubre"), ("11", "Noviembre"), ("12", "Diciembre"),
];

fn first_letter(s: &str) -> String {
    s.chars().next().map(|c| c.to_string()).unwrap_or_else(|| "?".to_string())
}

#[component]
pub fn PayrollPage() -> Element {
    let now_year = js_sys::Date::new_0().get_full_year() as i32;
    let now_month = js_sys::Date::new_0().get_month() + 1;
    let mut selected_month = use_signal(|| now_month);
    let mut selected_year = use_signal(|| now_year);
    let mut show_generate = use_signal(|| false);
    let mut gen_employee_id = use_signal(String::new);
    let mut gen_non_taxable = use_signal(|| "0".to_string());
    let mut gen_other_deductions = use_signal(|| "0".to_string());
    let mut generating = use_signal(|| false);
    let mut calc_result = use_signal(|| None::<serde_json::Value>);
    let mut exported = use_signal(|| String::new());

    let mut payrolls = use_resource(move || {
        let m = selected_month();
        let y = selected_year();
        async move { client::fetch_json(&format!("/api/hr/payroll?month={}&year={}", m, y)).await }
    });

    let employees = use_resource(|| async move { client::fetch_json("/api/hr/employees").await });

    let do_calculate = move |_| {
        if gen_employee_id().trim().is_empty() { return; }
        generating.set(true);
        let payload = json!({
            "employee_id": gen_employee_id(),
            "month": selected_month(),
            "year": selected_year(),
            "non_taxable_earnings": gen_non_taxable().parse::<f64>().unwrap_or(0.0),
            "other_deductions": gen_other_deductions().parse::<f64>().unwrap_or(0.0),
        });
        spawn(async move {
            let result = client::post_json("/api/hr/payroll/calculate", &payload).await;
            calc_result.set(result.ok());
            generating.set(false);
        });
    };

    let do_generate = move |_| {
        if gen_employee_id().trim().is_empty() { return; }
        generating.set(true);
        let payload = json!({
            "employee_id": gen_employee_id(),
            "month": selected_month(),
            "year": selected_year(),
            "non_taxable_earnings": gen_non_taxable().parse::<f64>().unwrap_or(0.0),
            "other_deductions": gen_other_deductions().parse::<f64>().unwrap_or(0.0),
        });
        spawn(async move {
            let result = client::post_json("/api/hr/payroll", &payload).await;
            generating.set(false);
            if result.is_ok() {
                show_generate.set(false);
                gen_employee_id.set(String::new());
                gen_non_taxable.set("0".to_string());
                gen_other_deductions.set("0".to_string());
                calc_result.set(None);
                payrolls.restart();
            }
        });
    };

    let do_export_lre = move |_| {
        let m = selected_month();
        let y = selected_year();
        spawn(async move {
            match client::fetch_json(&format!("/api/hr/payroll/export/lre?month={}&year={}", m, y)).await {
                Ok(data) => {
                    exported.set(format!("LRE exportado - {} registros", data["count"]));
                    payrolls.restart();
                }
                Err(e) => exported.set(format!("Error: {}", e)),
            }
        });
    };

    let do_export_previred = move |_| {
        let m = selected_month();
        let y = selected_year();
        spawn(async move {
            match client::fetch_json(&format!("/api/hr/payroll/export/previred?month={}&year={}", m, y)).await {
                Ok(data) => {
                    exported.set(format!("Previred exportado - {} registros", data["count"]));
                    payrolls.restart();
                }
                Err(e) => exported.set(format!("Error: {}", e)),
            }
        });
    };

    rsx! {
        div { class: "page-header",
            h1 { "Remuneraciones" }
            p { "Liquidaciones de sueldo con descuentos legales (AFP, Salud, Impuesto), LRE y exportación Previred" }
        }
        div { class: "page-toolbar",
            div { class: "filter-group",
                label { "Mes:" }
                select { class: "form-input", value: "{selected_month}", onchange: move |e| selected_month.set(e.value().parse().unwrap_or(now_month)),
                    {MONTHS.iter().map(|(v, l)| rsx! { option { value: "{v}", "{l}" } })}
                }
            }
            div { class: "filter-group",
                label { "Año:" }
                input { class: "form-input", value: "{selected_year}", oninput: move |e| selected_year.set(e.value().parse().unwrap_or(now_year)), r#type: "number", style: "width: 100px;" }
            }
            button { class: "btn btn-primary", onclick: move |_| show_generate.set(!show_generate()),
                if show_generate() { "Cancelar" } else { "Nueva Liquidación" }
            }
            button { class: "btn btn-secondary", onclick: do_export_lre, "Exportar LRE" }
            button { class: "btn btn-secondary", onclick: do_export_previred, "Exportar Previred" }
        }
        { if !exported().is_empty() {
            rsx! { div { class: "alert alert-success", "{exported}" } }
        } else { rsx! {} }}
        { if show_generate() {
            rsx! {
                div { class: "form-card",
                    div { class: "form-card-header",
                        h3 { "Generar Liquidación" }
                        span { class: "form-card-badge", "{selected_month()}/{selected_year()}" }
                    }
                    div { class: "form-section",
                        div { class: "form-section-title", "Datos del Cálculo" }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Empleado" }
                                select { class: "form-input", value: "{gen_employee_id}", onchange: move |e| gen_employee_id.set(e.value()),
                                    option { value: "", "Seleccionar empleado..." }
                                    { match employees() {
                                        Some(Ok(data)) => {
                                            let list = data["employees"].as_array().cloned().unwrap_or_default();
                                            let opts: Vec<Element> = list.iter().map(|emp| {
                                                let eid = emp["id"].as_str().unwrap_or("").to_string();
                                                let name = format!("{} {} - {}",
                                                    emp["first_name"].as_str().unwrap_or(""),
                                                    emp["last_name"].as_str().unwrap_or(""),
                                                    emp["rut"].as_str().unwrap_or("")
                                                );
                                                rsx! { option { value: "{eid}", "{name}" } }
                                            }).collect();
                                            rsx! { { opts.into_iter() } }
                                        }
                                        _ => rsx! {}
                                    }}
                                }
                            }
                            div { class: "form-group",
                                label { "Movilización / Colación" }
                                input { class: "form-input", value: "{gen_non_taxable}", oninput: move |e| gen_non_taxable.set(e.value()), r#type: "number", min: "0", placeholder: "0" }
                            }
                            div { class: "form-group",
                                label { "Otros Descuentos" }
                                input { class: "form-input", value: "{gen_other_deductions}", oninput: move |e| gen_other_deductions.set(e.value()), r#type: "number", min: "0", placeholder: "0" }
                            }
                        }
                    }
                    div { class: "form-actions",
                        button { class: "btn btn-secondary", disabled: generating(), onclick: do_calculate, "Calcular Vista Previa" }
                        button { class: "btn btn-primary", disabled: generating() || gen_employee_id().trim().is_empty(), onclick: do_generate,
                            if generating() { "Procesando..." } else { "Generar Liquidación" }
                        }
                    }
                    { if let Some(ref calc) = calc_result() {
                        let net_salary = calc["net_salary"].as_f64().unwrap_or(0.0);
                        let breakdown_rows: Vec<Element> = calc["breakdown"].as_array().cloned().unwrap_or_default().iter().map(|item| {
                            let concept = item["concept"].as_str().unwrap_or("").to_string();
                            let amount = item["amount"].as_f64().unwrap_or(0.0);
                            let cat = item["category"].as_str().unwrap_or("").to_string();
                            let cls = if amount < 0.0 { "amount-negative" } else { "amount-positive" };
                            rsx! { tr {
                                td { "{concept}" }
                                td { class: "{cls}", "${amount:.0}" }
                                td { "{cat}" }
                            }}
                        }).collect();
                        rsx! {
                            div { class: "calc-result",
                                h3 { "Vista Previa de Liquidación" }
                                table { class: "data-table",
                                    thead { tr { th { "Concepto" } th { "Monto" } th { "Categoría" } } }
                                    tbody { { breakdown_rows.into_iter() } }
                                    tfoot { tr {
                                        td { strong { "Sueldo Líquido" } }
                                        td { strong { "${net_salary:.0}" } }
                                        td { "" }
                                    }}
                                }
                            }
                        }
                    } else { rsx! {} }}
                }
            }
        } else { rsx! {} }}
        div { class: "data-table-container",
            match payrolls() {
                Some(Ok(data)) => {
                    let list = data["payrolls"].as_array().cloned().unwrap_or_default();
                    if list.is_empty() {
                        rsx! { div { class: "empty-state", "Sin liquidaciones para este período" } }
                    } else {
                        let rows: Vec<Element> = list.iter().map(|p| {
                            let name = p["employee_name"].as_str().unwrap_or("").to_string();
                            let rut = p["rut"].as_str().unwrap_or("").to_string();
                            let sb = p["salary_base"].as_f64().unwrap_or(0.0);
                            let taxable = p["taxable_income"].as_f64().unwrap_or(0.0);
                            let afp = p["afp_discount"].as_f64().unwrap_or(0.0);
                            let health = p["health_discount"].as_f64().unwrap_or(0.0);
                            let net = p["net_salary"].as_f64().unwrap_or(0.0);
                            let lre = p["lre_exported"].as_bool().unwrap_or(false);
                            let prev = p["previred_exported"].as_bool().unwrap_or(false);
                            let avatar = first_letter(&name);
                            rsx! {
                                tr {
                                    td { div { class: "employee-cell",
                                        div { class: "emp-avatar-small", "{avatar}" }
                                        span { "{name}" }
                                    }}
                                    td { span { class: "rut-badge", "{rut}" } }
                                    td { "${sb:.0}" }
                                    td { "${taxable:.0}" }
                                    td { "${afp:.0}" }
                                    td { "${health:.0}" }
                                    td { strong { style: "color: var(--success);", "${net:.0}" } }
                                    td {
                                        if lre { span { class: "badge badge-success", "LRE" } }
                                        else { span { class: "badge badge-warning", "-" } }
                                        if prev { span { class: "badge badge-success", "PREV" } }
                                    }
                                }
                            }
                        }).collect();
                        rsx! {
                            table { class: "data-table",
                                thead { tr {
                                    th { "Empleado" }
                                    th { "RUT" }
                                    th { "Sueldo Base" }
                                    th { "Imponible" }
                                    th { "AFP" }
                                    th { "Salud" }
                                    th { "Líquido" }
                                    th { "Exportado" }
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
