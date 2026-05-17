use dioxus::prelude::*;

use crate::api::client;

fn current_year() -> i32 {
    js_sys::Date::new_0().get_full_year() as i32
}

#[component]
pub fn IndividualReports() -> Element {
    let mut search_query = use_signal(String::new);
    let mut selected_student = use_signal(|| None::<serde_json::Value>);
    let mut selected_year = use_signal(current_year);
    let mut report_type = use_signal(|| "certificate".to_string());
    let mut result = use_signal(|| None::<Result<serde_json::Value, String>>);
    let mut loading = use_signal(|| false);
    let search_results = use_resource(move || {
        let q = search_query();
        async move {
            if q.len() < 2 {
                Ok(serde_json::json!({"students": []}))
            } else {
                client::search_students(&q).await
            }
        }
    });

    let on_search = move |evt: Event<FormData>| {
        search_query.set(evt.value());
    };

    let clear_student = move |_| {
        selected_student.set(None);
        result.set(None);
    };

    let generate_report = move |_| {
        if let Some(ref student) = selected_student() {
            let sid = student["id"].as_str().unwrap_or("").to_string();
            let y = selected_year();
            let rt = report_type();
            loading.set(true);
            result.set(None);
            spawn(async move {
                let res = match rt.as_str() {
                    "certificate" => client::fetch_student_certificate(&sid).await,
                    "concentration" => client::fetch_student_concentration(&sid, y).await,
                    _ => Err("Tipo no válido".to_string()),
                };
                loading.set(false);
                result.set(Some(res));
            });
        }
    };

    let mut select_student_fn = move |s: serde_json::Value| {
        selected_student.set(Some(s));
        search_query.set(String::new());
    };

    let search_data: Option<Vec<serde_json::Value>> = match search_results() {
        Some(Ok(j)) => {
            let list = j["students"].as_array().cloned().unwrap_or_default();
            if !list.is_empty() && search_query().len() >= 2 {
                Some(list)
            } else {
                None
            }
        }
        _ => None,
    };

    rsx! {
        div { class: "report-section",
            div { class: "filter-group",
                label { "Tipo de Reporte:" }
                select { value: "{report_type}", onchange: move |evt| report_type.set(evt.value()),
                    option { value: "certificate", "Certificado Alumno Regular" }
                    option { value: "concentration", "Concentración de Notas" }
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
            div { class: "student-selector",
                label { "Estudiante:" }
                {
                    match selected_student() {
                        Some(ref s) => {
                            let sname = format!("{} {}",
                                s["first_name"].as_str().unwrap_or(""),
                                s["last_name"].as_str().unwrap_or("")
                            );
                            let srut = s["rut"].as_str().unwrap_or("").to_string();
                            rsx! {
                                div { class: "selected-student",
                                    span { "{sname} ({srut})" }
                                    button { class: "btn-icon", "aria-label": "Limpiar seleccion", onclick: clear_student, "✕" }
                                }
                            }
                        }
                        None => rsx! {
                            input {
                                class: "search-input",
                                value: "{search_query}",
                                oninput: on_search,
                                placeholder: "Buscar estudiante por nombre o RUT..."
                            }
                        }
                    }
                }
                {
                    match search_data {
                        Some(ref list) => {
                            rsx! { div { class: "search-results",
                                for s in list {
                                    let sid = s["id"].as_str().unwrap_or("").to_string();
                                    let sname = format!("{} {}",
                                        s["first_name"].as_str().unwrap_or(""),
                                        s["last_name"].as_str().unwrap_or("")
                                    );
                                    let srut = s["rut"].as_str().unwrap_or("").to_string();
                                    rsx! {
                                        div {
                                            class: "search-result-item",
                                            onclick: move |_| {
                                                let sv = serde_json::json!({
                                                    "id": sid.clone(),
                                                    "first_name": sname.clone(),
                                                    "rut": srut.clone(),
                                                });
                                                select_student_fn(sv);
                                            },
                                            span { "{sname}" }
                                            span { class: "result-rut", "{srut}" }
                                        }
                                    }
                                }
                            } }
                        }
                        None => rsx! {},
                    }
                }
            }
            div { class: "form-actions",
                button {
                    class: "btn btn-primary",
                    disabled: selected_student().is_none() || loading(),
                    onclick: generate_report,
                    if loading() { "Generando..." } else { "Generar Reporte" }
                }
            }
            {
                match result() {
                    Some(Ok(j)) => {
                        match report_type().as_str() {
                            "certificate" => rsx! { CertificateResult { data: j } },
                            "concentration" => rsx! { ConcentrationResult { data: j } },
                            _ => rsx! {},
                        }
                    }
                    Some(Err(e)) => rsx! { div { class: "empty-state", "Error: {e}" } },
                    None => rsx! {},
                }
            }
        }
    }
}

#[component]
pub fn CertificateResult(data: serde_json::Value) -> Element {
    let name = data["certificate"]["student_name"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let rut = data["certificate"]["rut"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let grade_level = data["certificate"]["grade_level"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let section = data["certificate"]["section"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let year = data["certificate"]["year"].as_i64().unwrap_or(0);
    let status = data["certificate"]["enrollment_status"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let issued_at = data["certificate"]["issued_at"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let issuer = data["certificate"]["issuer_name"]
        .as_str()
        .unwrap_or("")
        .to_string();

    rsx! {
        div { class: "report-result certificate",
            div { class: "certificate-header",
                h2 { "Certificado Alumno Regular" }
            }
            div { class: "certificate-body",
                p { "El/la estudiante {name}, Rut {rut}, se encuentra matriculado(a) en {grade_level} {section}, durante el año académico {year}." }
                p { "Estado: {status}" }
                hr {}
                p { class: "certificate-meta", "Emitido el {issued_at} por {issuer}" }
            }
        }
    }
}

#[component]
pub fn ConcentrationResult(data: serde_json::Value) -> Element {
    let student_name = data["concentration"]["student_name"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let rut = data["concentration"]["rut"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let year = data["concentration"]["year"].as_i64().unwrap_or(0);
    let final_avg = data["concentration"]["final_average"]
        .as_f64()
        .unwrap_or(0.0);
    let final_prom = data["concentration"]["final_promotion"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let semesters = data["concentration"]["semesters"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let semester_cards: Vec<(i64, f64, Vec<(String, i64, String, f64, f64, f64)>)> = semesters
        .iter()
        .map(|sem| {
            let sem_num = sem["semester"].as_i64().unwrap_or(0);
            let global = sem["global_average"].as_f64().unwrap_or(0.0);
            let subjects = sem["subjects"].as_array().cloned().unwrap_or_default();
            let subject_rows: Vec<(String, i64, String, f64, f64, f64)> = subjects
                .iter()
                .map(|s| {
                    let sname = s["subject_name"].as_str().unwrap_or("-").to_string();
                    let gcount = s["grades_count"].as_i64().unwrap_or(0);
                    let avg = s["average"].as_f64().unwrap_or(0.0);
                    let min_g = s["min_grade"].as_f64().unwrap_or(0.0);
                    let max_g = s["max_grade"].as_f64().unwrap_or(0.0);
                    let avg_class = if avg >= 4.0 {
                        "grade-good".to_string()
                    } else {
                        "grade-bad".to_string()
                    };
                    (sname, gcount, avg_class, avg, min_g, max_g)
                })
                .collect();
            (sem_num, global, subject_rows)
        })
        .collect();

    rsx! {
        div { class: "report-result",
            h2 { "Concentración de Notas" }
            p { "{student_name} - {rut} - Año {year}" }
            {
                semester_cards.iter().map(|(sem_num, global, subject_rows)| {
                    rsx! {
                        div { class: "widget-card", style: "margin-top: 12px;",
                            div { class: "widget-card-header",
                                h3 { "Semestre {sem_num}" }
                                span { "Promedio: {global:.1}" }
                            }
                            table { class: "data-table",
                                thead { tr {
                                    th { "Asignatura" }
                                    th { "Notas" }
                                    th { "Promedio" }
                                    th { "Mín" }
                                    th { "Máx" }
                                }}
                                tbody { for (sname, gcount, avg_class, avg, min_g, max_g) in subject_rows {
                                    tr {
                                        td { "{sname}" }
                                        td { "{gcount}" }
                                        td { class: "{avg_class}", "{avg:.1}" }
                                        td { "{min_g:.1}" }
                                        td { "{max_g:.1}" }
                                    }
                                }}
                            }
                        }
                    }
                })
            }
            div { class: "promotion-banner",
                span { class: "promotion-text", "Promedio Final: {final_avg:.1} - {final_prom}" }
            }
        }
    }
}
