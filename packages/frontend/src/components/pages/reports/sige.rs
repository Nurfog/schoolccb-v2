use dioxus::prelude::*;

use crate::api::client;

use super::current_year;

fn current_month() -> u32 {
    js_sys::Date::new_0().get_month() + 1
}

#[component]
pub fn SigeReports() -> Element {
    let mut selected_year = use_signal(current_year);
    let mut selected_month = use_signal(current_month);
    let mut export_type = use_signal(|| "students".to_string());
    let mut result = use_signal(|| None::<Result<serde_json::Value, String>>);
    let mut loading = use_signal(|| false);

    let generate_export = move |_| {
        loading.set(true);
        result.set(None);
        let et = export_type();
        let y = selected_year();
        let m = selected_month();
        spawn(async move {
            let res = match et.as_str() {
                "students" => client::fetch_sige_students().await,
                "attendance" => client::fetch_sige_attendance(y, m).await,
                _ => Err("Tipo no válido".to_string()),
            };
            loading.set(false);
            result.set(Some(res));
        });
    };

    rsx! {
        div { class: "report-section",
            div { class: "filter-group",
                label { "Exportar:" }
                select { value: "{export_type}", onchange: move |evt| export_type.set(evt.value()),
                    option { value: "students", "Datos de Estudiantes (SIGE)" }
                    option { value: "attendance", "Asistencia Mensual (SIGE)" }
                }
            }
            div { class: "filter-group",
                label { "Año:" }
                select {
                    value: "{selected_year}",
                    onchange: move |evt| { if let Ok(y) = evt.value().parse() { selected_year.set(y); } },
                    option { value: "2026", "2026" }
                    option { value: "2025", "2025" }
                    option { value: "2024", "2024" }
                }
            }
            {
                if export_type() == "attendance" {
                    rsx! {
                        div { class: "filter-group",
                            label { "Mes:" }
                            select {
                                value: "{selected_month}",
                                onchange: move |evt| { if let Ok(m) = evt.value().parse() { selected_month.set(m); } },
                                option { value: "1", "Enero" }
                                option { value: "2", "Febrero" }
                                option { value: "3", "Marzo" }
                                option { value: "4", "Abril" }
                                option { value: "5", "Mayo" }
                                option { value: "6", "Junio" }
                                option { value: "7", "Julio" }
                                option { value: "8", "Agosto" }
                                option { value: "9", "Septiembre" }
                                option { value: "10", "Octubre" }
                                option { value: "11", "Noviembre" }
                                option { value: "12", "Diciembre" }
                            }
                        }
                    }
                } else { rsx! {} }
            }
            div { class: "form-actions",
                button {
                    class: "btn btn-primary",
                    disabled: loading(),
                    onclick: generate_export,
                    if loading() { "Exportando..." } else { "Exportar" }
                }
            }
            {
                match result() {
                    Some(Ok(j)) => {
                        let total = j["total"].as_i64().unwrap_or(0);
                        let rows = j["rows"].as_array().cloned().unwrap_or_default();
                        rsx! {
                            div { class: "report-result",
                                p { "Total registros: {total}" }
                                table { class: "data-table",
                                    thead {
                                        tr {
                                            { rows.first().map(|first| {
                                                rsx! {
                                                    { first.as_object().map(|obj| {
                                                        rsx! { { obj.keys().map(|k| rsx! { th { "{k}" } }) } }
                                                    })}
                                                }
                                            })}
                                        }
                                    }
                                    tbody {
                                        for row in &rows {
                                            tr {
                                                { row.as_object().map(|obj| {
                                                    rsx! { { obj.values().map(|v| {
                                                        let val = match v {
                                                            serde_json::Value::String(s) => s.clone(),
                                                            serde_json::Value::Number(n) => n.to_string(),
                                                            serde_json::Value::Bool(b) => b.to_string(),
                                                            _ => "".to_string(),
                                                        };
                                                        rsx! { td { "{val}" } }
                                                    }) } }
                                                })}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(e)) => rsx! { div { class: "empty-state", "Error: {e}" } },
                    None => rsx! {},
                }
            }
        }
    }
}
