use dioxus::prelude::*;

use crate::api::client;
use crate::components::widgets::icon::Icon;

fn current_year() -> i32 {
    js_sys::Date::new_0().get_full_year() as i32
}

fn get_subjects(data: &serde_json::Value) -> Vec<serde_json::Value> {
    data["subjects"].as_array().cloned().unwrap_or_default()
}

fn get_courses(data: &serde_json::Value) -> Vec<serde_json::Value> {
    data["courses"].as_array().cloned().unwrap_or_default()
}

#[component]
pub fn GradesPage() -> Element {
    let mut selected_subject_id = use_signal(String::new);
    let mut selected_year = use_signal(current_year);
    let mut selected_semester = use_signal(|| 0i32);
    let subjects = use_resource(|| client::fetch_subjects());
    let mut grades_data = use_resource(move || {
        let sid = selected_subject_id();
        let y = selected_year();
        async move {
            if sid.is_empty() {
                Err("No subject selected".to_string())
            } else {
                client::fetch_grades_by_subject(&sid, y).await
            }
        }
    });

    let on_subject_change = move |evt: Event<FormData>| {
        selected_subject_id.set(evt.value());
        grades_data.restart();
    };

    let on_year_change = move |evt: Event<FormData>| {
        if let Ok(y) = evt.value().parse::<i32>() {
            selected_year.set(y);
            grades_data.restart();
        }
    };

    let on_semester_change = move |evt: Event<FormData>| {
        if let Ok(s) = evt.value().parse::<i32>() {
            selected_semester.set(s);
        }
    };

    rsx! {
        div { class: "page-header",
            h1 { "Calificaciones" }
            p { "Consulta de notas por asignatura" }
        }
        div { class: "page-toolbar",
            div { class: "filter-group",
                label { "Asignatura:" }
                select { value: "{selected_subject_id}", onchange: on_subject_change,
                    option { value: "", "Seleccione una asignatura..." }
                    {
                        match subjects() {
                            Some(Ok(data)) => {
                                let list = get_subjects(&data);
                                rsx! {
                                    for subject in list {
                                        SubjectOption { subject: subject }
                                    }
                                }
                            }
                            _ => rsx! {},
                        }
                    }
                }
            }
            div { class: "filter-group",
                label { "Año:" }
                select { value: "{selected_year}", onchange: on_year_change,
                    option { value: "2026", "2026" }
                    option { value: "2025", "2025" }
                    option { value: "2024", "2024" }
                }
            }
            div { class: "filter-group",
                label { "Semestre:" }
                select { value: "{selected_semester}", onchange: on_semester_change,
                    option { value: "0", "Ambos" }
                    option { value: "1", "Semestre 1" }
                    option { value: "2", "Semestre 2" }
                }
            }
        }
        div { class: "data-table-container",
            {
                if selected_subject_id().is_empty() {
                    rsx! {
                        div { class: "info-card",
                            div { class: "info-icon",
                                Icon { name: "info", size: 32 }
                            }
                            div { class: "info-text",
                                h3 { "Vista de Calificaciones" }
                                p { "Seleccione una asignatura para ver las calificaciones de los estudiantes." }
                            }
                        }
                    }
                } else {
                    match grades_data() {
                        Some(Ok(json)) => {
                            let subject_name = json["subject_name"].as_str().unwrap_or("");
                            let subject_code = json["subject_code"].as_str().unwrap_or("");
                            let courses = get_courses(&json);
                            let total_courses = json["total_courses"].as_i64().unwrap_or(0);
                            let semester = selected_semester();
                            rsx! {
                                div { class: "summary-cards",
                                    div { class: "summary-card",
                                        span { class: "summary-value", "{subject_code}" }
                                        span { class: "summary-label", "{subject_name}" }
                                    }
                                    div { class: "summary-card",
                                        span { class: "summary-value", "{total_courses}" }
                                        span { class: "summary-label", "Cursos" }
                                    }
                                }
                                for course in courses {
                                    CourseGradesSection { course: course, semester: semester }
                                }
                            }
                        }
                        Some(Err(e)) => rsx! {
                            div { class: "empty-state", "Error: {e}" }
                        },
                        None => rsx! {
                            div { class: "empty-state", div { class: "loading-spinner", "Cargando..." } }
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn SubjectOption(subject: serde_json::Value) -> Element {
    let id = subject["id"].as_str().unwrap_or("").to_string();
    let code = subject["code"].as_str().unwrap_or("").to_string();
    let name = subject["name"].as_str().unwrap_or("").to_string();
    let display = format!("{} ({})", name, code);
    rsx! {
        option { value: "{id}", "{display}" }
    }
}

#[component]
fn CourseGradesSection(course: serde_json::Value, semester: i32) -> Element {
    let course_name = course["course_name"]
        .as_str()
        .unwrap_or("Curso")
        .to_string();
    let students = course["students"].as_array().cloned().unwrap_or_default();
    let total = course["total_students"].as_i64().unwrap_or(0);

    rsx! {
        div { class: "widget-card", style: "margin-top: 16px;",
            div { class: "widget-card-header",
                h3 { "{course_name}" }
                span { "{total} estudiantes" }
            }
            div { class: "widget-card-body",
                table { class: "data-table",
                    thead {
                        tr {
                            th { "Estudiante" }
                            th { "RUT" }
                            if semester == 0 || semester == 1 {
                                th { "Notas S1" }
                                th { "Prom. S1" }
                            }
                            if semester == 0 || semester == 2 {
                                th { "Notas S2" }
                                th { "Prom. S2" }
                            }
                            if semester == 0 {
                                th { "Prom. Final" }
                            }
                        }
                    }
                    tbody {
                        for student in students {
                            StudentGradeRow { student: student, semester: semester }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn StudentGradeRow(student: serde_json::Value, semester: i32) -> Element {
    let name = student["student_name"].as_str().unwrap_or("-").to_string();
    let rut = student["rut"].as_str().unwrap_or("-").to_string();

    let grades_s1_str = student["grades_s1"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|g| format!("{:.1}", g.as_f64().unwrap_or(0.0)))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let avg_s1 = student["average_s1"].as_f64().unwrap_or(0.0);
    let avg_s1_str = if avg_s1 > 0.0 {
        format!("{:.1}", avg_s1)
    } else {
        "-".to_string()
    };
    let avg_s1_class = if avg_s1 >= 4.0 {
        "grade-good"
    } else if avg_s1 > 0.0 {
        "grade-bad"
    } else {
        ""
    };

    let grades_s2_str = student["grades_s2"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|g| format!("{:.1}", g.as_f64().unwrap_or(0.0)))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let avg_s2 = student["average_s2"].as_f64().unwrap_or(0.0);
    let avg_s2_str = if avg_s2 > 0.0 {
        format!("{:.1}", avg_s2)
    } else {
        "-".to_string()
    };
    let avg_s2_class = if avg_s2 >= 4.0 {
        "grade-good"
    } else if avg_s2 > 0.0 {
        "grade-bad"
    } else {
        ""
    };

    let final_avg = student["final_average"].as_f64().unwrap_or(0.0);
    let final_str = if final_avg > 0.0 {
        format!("{:.1}", final_avg)
    } else {
        "-".to_string()
    };
    let final_class = if final_avg >= 4.0 {
        "grade-good"
    } else if final_avg > 0.0 {
        "grade-bad"
    } else {
        ""
    };

    rsx! {
        tr {
            td { class: "cell-name", "{name}" }
            td { "{rut}" }
            if semester == 0 || semester == 1 {
                td { style: "font-size: 0.85em; max-width: 200px; overflow: hidden; text-overflow: ellipsis;", "{grades_s1_str}" }
                td { class: "{avg_s1_class}", style: "font-weight: bold;", "{avg_s1_str}" }
            }
            if semester == 0 || semester == 2 {
                td { style: "font-size: 0.85em; max-width: 200px; overflow: hidden; text-overflow: ellipsis;", "{grades_s2_str}" }
                td { class: "{avg_s2_class}", style: "font-weight: bold;", "{avg_s2_str}" }
            }
            if semester == 0 {
                td { class: "{final_class}", style: "font-weight: bold; font-size: 1.1em;", "{final_str}" }
            }
        }
    }
}
