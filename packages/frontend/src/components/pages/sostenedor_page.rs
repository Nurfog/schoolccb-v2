use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::window;
use crate::api::client;
use crate::components::widgets::simple_chart::BarChart;

fn current_year() -> String {
    js_sys::Date::new_0().get_full_year().to_string()
}

fn export_csv(table_id: &str, filename: &str) {
    if let Some(win) = window() {
        if let Some(doc) = win.document() {
            if let Some(table) = doc.get_element_by_id(table_id) {
                let mut csv = String::from("\u{feff}");
                if let Some(element) = table.dyn_ref::<web_sys::HtmlElement>() {
                    if let Ok(rows) = element.query_selector_all("tr") {
                        for i in 0..rows.length() {
                            if let Some(row) = rows.item(i) {
                                if let Some(row_elem) = row.dyn_ref::<web_sys::HtmlElement>() {
                                    if let Ok(cells) = row_elem.query_selector_all("td, th") {
                                        let mut row_data: Vec<String> = Vec::new();
                                        for j in 0..cells.length() {
                                            if let Some(cell) = cells.item(j) {
                                                if let Some(cell_elem) = cell.dyn_ref::<web_sys::HtmlElement>() {
                                                    let text = cell_elem.text_content().unwrap_or_default();
                                                    row_data.push(format!("\"{}\"", text.replace('"', "\"\"")));
                                                }
                                            }
                                        }
                                        csv.push_str(&row_data.join(","));
                                        csv.push('\n');
                                    }
                                }
                            }
                        }
                    }
                }
                let data_url = format!("data:text/csv;charset=utf-8,{}", js_sys::encode_uri_component(&csv));
                if let Ok(anchor) = doc.create_element("a") {
                    let _ = anchor.set_attribute("href", &data_url);
                    let _ = anchor.set_attribute("download", filename);
                    let _ = anchor.set_attribute("style", "display:none");
                    let _ = doc.body().and_then(|b| b.append_child(&anchor).ok());
                    if let Some(a_elem) = anchor.dyn_ref::<web_sys::HtmlElement>() {
                        let _ = a_elem.click();
                    }
                    let _ = doc.body().and_then(|b| b.remove_child(&anchor).ok());
                }
            }
        }
    }
}

fn export_pdf() {
    if let Some(win) = window() {
        let _ = win.print();
    }
}

#[component]
pub fn SostenedorPage() -> Element {
    let summary = use_resource(client::fetch_corp_dashboard_summary);
    let schools = use_resource(client::fetch_corp_dashboard_schools);
    let comparisons = use_resource(client::fetch_corp_dashboard_comparisons);
    let trends = use_resource(client::fetch_corp_dashboard_trends);
    let alerts = use_resource(client::fetch_corp_dashboard_alerts);
    let license = use_resource(client::fetch_corp_license);
    let mut selected_year = use_signal(current_year);

    rsx! {
        div { class: "page-header",
            h1 { "Panel del Sostenedor" }
            p { "Visión global de tu corporación" }
        }
        div { class: "page-toolbar", style: "display: flex; gap: 12px; align-items: center; margin-bottom: 16px;",
            div { class: "filter-group",
                label { "Año:" }
                select { class: "form-input", value: "{selected_year}", oninput: move |e| selected_year.set(e.value()),
                    option { value: "2026", "2026" }
                    option { value: "2025", "2025" }
                    option { value: "2024", "2024" }
                    option { value: "2023", "2023" }
                }
            }
            button { class: "btn btn-secondary", onclick: move |_| export_csv("schools-table", "colegios.csv"), "Exportar CSV" }
            button { class: "btn btn-secondary", onclick: move |_| export_pdf(), "Exportar PDF" }
        }
        match summary() {
            Some(Ok(data)) => {
                let schools = data["total_schools"].as_i64().unwrap_or(0);
                let students = data["total_students"].as_i64().unwrap_or(0);
                let teachers = data["total_teachers"].as_i64().unwrap_or(0);
                let employees = data["total_employees"].as_i64().unwrap_or(0);
                let attendance = data["avg_attendance"].as_str().unwrap_or("0").to_string();
                let avg_grade = data["avg_grades"].as_str().unwrap_or("0").to_string();
                let active_lic = data["active_licenses"].as_i64().unwrap_or(0);
                let expiring = data["expiring_licenses"].as_i64().unwrap_or(0);
                let revenue = data["monthly_revenue"].as_f64().unwrap_or(0.0);
                let exp_color = if expiring > 0 { "danger" } else { "success" };
                rsx! {
                    div { class: "kpi-grid-wide",
                        KpiCard { label: "Colegios", value: "{schools}", color: "primary" }
                        KpiCard { label: "Alumnos", value: "{students}", color: "info" }
                        KpiCard { label: "Docentes", value: "{teachers}", color: "success" }
                        KpiCard { label: "Empleados", value: "{employees}", color: "secondary" }
                        KpiCard { label: "Asistencia Prom.", value: "{attendance}%", color: "info" }
                        KpiCard { label: "Prom. Notas", value: "{avg_grade}", color: "info" }
                        KpiCard { label: "Licencias Activas", value: "{active_lic}", color: "success" }
                        KpiCard { label: "Próximas a Vencer", value: "{expiring}", color: "{exp_color}" }
                        KpiCard { label: "Ingresos del Mes", value: "${revenue:.0}", color: "primary" }
                    }
                }
            }
            Some(Err(e)) => rsx! { div { class: "alert alert-error", "Error: {e}" } },
            None => rsx! { div { class: "loading-spinner", "Cargando..." } },
        }
        div { class: "dashboard-grid",
            match license() {
                Some(Ok(data)) => {
                    let plan = data["plan_name"].as_str().unwrap_or("-");
                    let status = data["status"].as_str().unwrap_or("-");
                    let days = data["days_remaining"].as_i64().unwrap_or(0);
                    let modules = data["modules"].as_array().cloned().unwrap_or_default();
                    let enabled = modules.iter().filter(|m| m["included"].as_bool().unwrap_or(false)).count();
                    rsx! {
                        div { class: "widget-card",
                            div { class: "widget-card-header",
                                h3 { "Plan y Licencia" }
                                span { class: "status-active", "{status}" }
                            }
                            div { class: "widget-card-body",
                                div { class: "license-plan-name", "{plan}" }
                                div { class: "license-days", "Días restantes: ", strong { "{days}" } }
                                div { class: "license-modules", "Módulos: {enabled}/{modules.len()}" }
                            }
                        }
                    }
                }
                _ => rsx! {}
            }
            match schools() {
                Some(Ok(data)) => {
                    let school_list = data["schools"].as_array().cloned().unwrap_or_default();
                    let rows: Vec<Element> = school_list.iter().map(|school| {
                        let name = school["name"].as_str().unwrap_or("-").to_string();
                        let students = school["students"].as_i64().unwrap_or(0);
                        let teachers = school["teachers"].as_i64().unwrap_or(0);
                        let att = school["attendance"].as_str().unwrap_or("100").to_string();
                        let att_cls = att.parse::<f64>().map(|v| {
                            if v >= 90.0 { "pct-good" } else if v >= 80.0 { "pct-warning" } else { "pct-danger" }
                        }).unwrap_or("");
                        let grade = school["avg_grade"].as_str().unwrap_or("0").to_string();
                        let g_cls = grade.parse::<f64>().map(|v| {
                            if v >= 4.0 { "grade-good" } else { "grade-bad" }
                        }).unwrap_or("");
                        rsx! {
                            tr {
                                td { "{name}" }
                                td { "{students}" }
                                td { "{teachers}" }
                                td { class: "{att_cls}", "{att}%" }
                                td { class: "{g_cls}", "{grade}" }
                            }
                        }
                    }).collect();
                    rsx! {
                        div { class: "widget-card",
                            div { class: "widget-card-header",
                                h3 { "Colegios" }
                                span { "{school_list.len()} colegios" }
                            }
                            div { class: "widget-card-body",
                                div { class: "data-table-container",
                                    table { id: "schools-table", class: "data-table",
                                        thead {
                                            tr {
                                                th { "Colegio" }
                                                th { "Alumnos" }
                                                th { "Docentes" }
                                                th { "Asistencia" }
                                                th { "Prom. Notas" }
                                            }
                                        }
                                        tbody { {rows.into_iter()} }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => rsx! {}
            }
            match comparisons() {
                Some(Ok(data)) => {
                    let comps = data["comparisons"].as_array().cloned().unwrap_or_default();
                    if comps.is_empty() {
                        rsx! {}
                    } else {
                        rsx! {
                            div { class: "widget-card",
                                div { class: "widget-card-header",
                                    h3 { "Comparativa de Asistencia" }
                                    span { "Últimos 30 días" }
                                }
                                div { class: "widget-card-body",
                                    BarChart {
                                        data: comps.clone(),
                                        label_key: "school_name".to_string(),
                                        value_key: "attendance_pct".to_string(),
                                        height: None,
                                        color: None,
                                    }
                                }
                            }
                            div { class: "widget-card",
                                div { class: "widget-card-header",
                                    h3 { "Comparativa de Notas" }
                                }
                                div { class: "widget-card-body",
                                    BarChart {
                                        data: comps,
                                        label_key: "school_name".to_string(),
                                        value_key: "avg_grade".to_string(),
                                        height: None,
                                        color: Some("#22c55e".to_string()),
                                    }
                                }
                            }
                        }
                    }
                }
                _ => rsx! {}
            }
            match trends() {
                Some(Ok(data)) => {
                    let enrollment_trend = data["enrollment_trend"].as_array().cloned().unwrap_or_default();
                    let attendance_trend = data["attendance_trend"].as_array().cloned().unwrap_or_default();
                    let enrollment_chart = if enrollment_trend.is_empty() {
                        rsx! {}
                    } else {
                        rsx! {
                            div { class: "widget-card",
                                div { class: "widget-card-header",
                                    h3 { "Evolución Matrícula" }
                                    span { "Últimos 12 meses" }
                                }
                                div { class: "widget-card-body",
                                    BarChart {
                                        data: enrollment_trend,
                                        label_key: "month".to_string(),
                                        value_key: "enrollments".to_string(),
                                        height: None,
                                        color: Some("#8b5cf6".to_string()),
                                    }
                                }
                            }
                        }
                    };
                    let attendance_chart = if attendance_trend.is_empty() {
                        rsx! {}
                    } else {
                        rsx! {
                            div { class: "widget-card",
                                div { class: "widget-card-header",
                                    h3 { "Evolución Asistencia" }
                                    span { "Últimos 12 meses" }
                                }
                                div { class: "widget-card-body",
                                    BarChart {
                                        data: attendance_trend,
                                        label_key: "month".to_string(),
                                        value_key: "attendance".to_string(),
                                        height: None,
                                        color: Some("#f59e0b".to_string()),
                                    }
                                }
                            }
                        }
                    };
                    rsx! { {enrollment_chart} {attendance_chart} }
                }
                _ => rsx! {}
            }
            match alerts() {
                Some(Ok(data)) => {
                    let alert_list = data["alerts"].as_array().cloned().unwrap_or_default();
                    let alert_rows: Vec<Element> = alert_list.iter().map(|alert| {
                        let msg = alert["message"].as_str().unwrap_or("").to_string();
                        let school = alert["school_name"].as_str().unwrap_or("").to_string();
                        let severity = alert["severity"].as_str().unwrap_or("").to_string();
                        let sev = match severity.as_str() {
                            "critical" | "high" => "Alto",
                            "medium" => "Medio",
                            _ => "Bajo",
                        };
                        rsx! {
                            div { class: "alert-item",
                                div { class: "alert-badge {sev}" }
                                div { class: "alert-info",
                                    div { class: "alert-name", "{msg}" }
                                    div { class: "alert-detail", "{school}" }
                                }
                                span { class: "alert-severity {sev}", "{severity}" }
                            }
                        }
                    }).collect();
                    rsx! {
                        div { class: "widget-card",
                            div { class: "widget-card-header",
                                h3 { "Alertas" }
                                span { "{alert_list.len()} alertas" }
                            }
                            div { class: "widget-card-body",
                                {alert_rows.into_iter()}
                            }
                        }
                    }
                }
                _ => rsx! {}
            }
        }
    }
}

#[component]
fn KpiCard(label: String, value: String, color: String) -> Element {
    rsx! {
        div { class: "widget-card", style: "padding: 16px; text-align: center;",
            div { class: "kpi-value {color}", "{value}" }
            div { class: "kpi-label", "{label}" }
        }
    }
}
