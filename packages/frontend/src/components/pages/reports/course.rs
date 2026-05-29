use dioxus::prelude::*;

use crate::api::client;

use super::current_year;

#[component]
pub fn CourseReports() -> Element {
    let mut selected_year = use_signal(current_year);
    let mut selected_course = use_signal(|| None::<serde_json::Value>);
    let mut search_course = use_signal(String::new);
    let mut result = use_signal(|| None::<Result<serde_json::Value, String>>);
    let mut loading = use_signal(|| false);
    let mut perf_result = use_signal(|| None::<Result<serde_json::Value, String>>);
    let mut perf_loading = use_signal(|| false);
    let courses = use_resource(move || {
        let q = search_course();
        async move {
            client::fetch_json(&format!("/api/courses?search={}", q.replace(' ', "%20"))).await
        }
    });

    let generate_record = move |_| {
        if let Some(ref course) = selected_course() {
            let cid = course["id"].as_str().unwrap_or("").to_string();
            let y = selected_year();
            loading.set(true);
            result.set(None);
            spawn(async move {
                let res = client::fetch_final_record(&cid, y).await;
                loading.set(false);
                result.set(Some(res));
            });
        }
    };

    let generate_performance = move |_| {
        if let Some(ref course) = selected_course() {
            let cid = course["id"].as_str().unwrap_or("").to_string();
            let y = selected_year();
            perf_loading.set(true);
            perf_result.set(None);
            spawn(async move {
                let res = client::fetch_course_performance(&cid, y).await;
                perf_loading.set(false);
                perf_result.set(Some(res));
            });
        }
    };

    let course_elements = match courses() {
        Some(Ok(j)) => {
            let list = j["courses"].as_array().cloned().unwrap_or_default();
            if list.is_empty() {
                None
            } else {
                let mut selected = selected_course.clone();
                let mut search = search_course.clone();
                let course_items: Vec<Element> = list.iter().map(|c| {
                    let cid = c["id"].as_str().unwrap_or("").to_string();
                    let cname = c["name"].as_str().unwrap_or("").to_string();
                    let level = c["grade_level"].as_str().unwrap_or("").to_string();
                    let section = c["section"].as_str().unwrap_or("").to_string();
                    let cinfo = format!("{} - {}", level, section);
                    let cid_clone = cid.clone();
                    let cname_clone = cname.clone();
                    rsx! {
                        div {
                            class: "search-result-item",
                            onclick: move |_| {
                                selected.set(Some(serde_json::json!({
                                    "id": cid_clone.clone(),
                                    "name": cname_clone.clone(),
                                })));
                                search.set(String::new());
                            },
                            span { "{cname}" }
                            span { class: "result-rut", "{cinfo}" }
                        }
                    }
                }).collect();
                Some(rsx! { {course_items.into_iter()} })
            }
        }
        _ => None,
    };

    rsx! {
        div { class: "report-section",
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
            div { class: "student-selector",
                label { "Curso:" }
                {
                    match selected_course() {
                        Some(ref c) => {
                            let cname = c["name"].as_str().unwrap_or("").to_string();
                            rsx! {
                                div { class: "selected-student",
                                    span { "{cname}" }
                                    button { class: "btn-icon", "aria-label": "Cerrar", onclick: move |_| { selected_course.set(None); result.set(None); }, "✕" }
                                }
                            }
                        }
                        None => rsx! {
                            input {
                                class: "search-input",
                                value: "{search_course}",
                                oninput: move |evt| search_course.set(evt.value()),
                                placeholder: "Buscar curso..."
                            }
                        }
                    }
                }
                {
                    if let Some(elements) = course_elements {
                        rsx! { div { class: "search-results", { elements } } }
                    } else { rsx! {} }
                }
            }
            div { class: "form-actions",
                button {
                    class: "btn btn-primary",
                    disabled: selected_course().is_none() || loading(),
                    onclick: generate_record,
                    if loading() { "Generando..." } else { "Generar Acta Final" }
                }
            }
            {
                match result() {
                    Some(Ok(j)) => rsx! { FinalRecordResult { data: j } },
                    Some(Err(e)) => rsx! { div { class: "empty-state", "Error: {e}" } },
                    None => rsx! {},
                }
            }
            div { class: "report-section", style: "margin-top: 20px; padding-top: 20px; border-top: 1px solid var(--border);",
                h3 { "Rendimiento por Curso" }
                div { class: "filter-group",
                    span { style: "font-size: 13px; color: var(--text-secondary);",
                        "Curso: {selected_course().map(|c| c[\"name\"].as_str().unwrap_or(\"\").to_string()).unwrap_or_else(|| \"—\".to_string())} — Año: {selected_year}"
                    }
                }
                div { class: "form-actions",
                    button {
                        class: "btn btn-secondary",
                        disabled: selected_course().is_none() || perf_loading(),
                        onclick: generate_performance,
                        if perf_loading() { "Cargando..." } else { "Ver Rendimiento" }
                    }
                }
                {
                    match perf_result() {
                        Some(Ok(j)) => rsx! { CoursePerformanceResult { data: j } },
                        Some(Err(e)) => rsx! { div { class: "empty-state", "Error: {e}" } },
                        None => rsx! {},
                    }
                }
            }
        }
    }
}

#[component]
pub fn CoursePerformanceResult(data: serde_json::Value) -> Element {
    let cp = &data["course_performance"];
    let course_name = cp["course_name"].as_str().unwrap_or("").to_string();
    let year = cp["year"].as_i64().unwrap_or(0);
    let subjects = cp["subjects"].as_array().cloned().unwrap_or_default();

    let rows: Vec<(String, f64, i64, f64, f64, String)> = subjects.iter().map(|s| {
        let sname = s["subject_name"].as_str().unwrap_or("-").to_string();
        let avg = s["average_grade"].as_f64().unwrap_or(0.0);
        let count = s["grades_count"].as_i64().or_else(|| s["student_count"].as_i64()).unwrap_or(0);
        let min_g = s["min_grade"].as_f64().unwrap_or(0.0);
        let max_g = s["max_grade"].as_f64().unwrap_or(0.0);
        let avg_class = if avg >= 4.0 { "grade-good".to_string() } else { "grade-bad".to_string() };
        (sname, avg, count, min_g, max_g, avg_class)
    }).collect();

    rsx! {
        div { class: "report-result",
            p { "Rendimiento de {course_name} - Año {year}" }
            table { class: "data-table", style: "margin-top: 12px;",
                thead { tr {
                    th { "Asignatura" }
                    th { "Promedio" }
                    th { "Notas" }
                    th { "Mín" }
                    th { "Máx" }
                }}
                tbody { for (sname, avg, count, min_g, max_g, avg_class) in &rows {
                    tr {
                        td { "{sname}" }
                        td { class: "{avg_class}", "{avg:.1}" }
                        td { "{count}" }
                        td { "{min_g:.1}" }
                        td { "{max_g:.1}" }
                    }
                }}
            }
            {
                let pass_count = cp["pass_count"].as_i64();
                let fail_count = cp["fail_count"].as_i64();
                match (pass_count, fail_count) {
                    (Some(p), Some(f)) => rsx! {
                        div { class: "summary-cards", style: "margin-top: 12px;",
                            div { class: "summary-card",
                                span { class: "summary-value", "{p}" }
                                span { class: "summary-label", "Aprobados" }
                            }
                            div { class: "summary-card danger",
                                span { class: "summary-value", "{f}" }
                                span { class: "summary-label", "Reprobados" }
                            }
                        }
                    },
                    _ => rsx! {},
                }
            }
        }
    }
}

#[component]
pub fn FinalRecordResult(data: serde_json::Value) -> Element {
    let record = &data["final_record"];
    let course_name = record["course_name"].as_str().unwrap_or("").to_string();
    let year = record["year"].as_i64().unwrap_or(0);
    let promoted = record["summary"]["promoted"].as_i64().unwrap_or(0);
    let failed = record["summary"]["failed"].as_i64().unwrap_or(0);
    let rate = record["summary"]["average_promotion_rate"]
        .as_f64()
        .unwrap_or(0.0);
    let students = record["students"].as_array().cloned().unwrap_or_default();

    let student_rows: Vec<(String, String, f64, String, String)> = students
        .iter()
        .map(|s| {
            let name = s["student_name"].as_str().unwrap_or("-").to_string();
            let rut = s["rut"].as_str().unwrap_or("-").to_string();
            let avg = s["final_average"].as_f64().unwrap_or(0.0);
            let prom = s["promotion"].as_str().unwrap_or("").to_string();
            let prom_class = if prom == "Reprobado" {
                "grade-bad".to_string()
            } else {
                "grade-good".to_string()
            };
            (name, rut, avg, prom, prom_class)
        })
        .collect();

    rsx! {
        div { class: "report-result",
            div { class: "summary-cards",
                div { class: "summary-card",
                    span { class: "summary-value", "{course_name}" }
                    span { class: "summary-label", "Curso - {year}" }
                }
                div { class: "summary-card",
                    span { class: "summary-value", "{promoted}" }
                    span { class: "summary-label", "Promovidos" }
                }
                div { class: "summary-card",
                    span { class: "summary-value", "{failed}" }
                    span { class: "summary-label", "Reprobados" }
                }
                div { class: "summary-card",
                    span { class: "summary-value", "{rate:.1}%" }
                    span { class: "summary-label", "Tasa de Promoción" }
                }
            }
            table { class: "data-table", style: "margin-top: 16px;",
                thead { tr {
                    th { "Estudiante" }
                    th { "RUT" }
                    th { "Prom. Final" }
                    th { "Promoción" }
                }}
                tbody { for (name, rut, avg, prom, prom_class) in &student_rows {
                    tr {
                        td { class: "cell-name", "{name}" }
                        td { "{rut}" }
                        td { "{avg:.1}" }
                        td { class: "{prom_class}", "{prom}" }
                    }
                }}
            }
        }
    }
}
