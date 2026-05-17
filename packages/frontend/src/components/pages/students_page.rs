use dioxus::prelude::*;

use crate::api::client;
use crate::components::widgets::icon::Icon;
use crate::components::inline_edit::InlineEdit;

fn first_letter(s: &str) -> String {
    s.chars()
        .next()
        .map(|c| c.to_string())
        .unwrap_or_else(|| "?".to_string())
}

#[component]
pub fn StudentsPage() -> Element {
    let mut search_query = use_signal(String::new);
    let mut students = use_resource(|| client::fetch_students(None, None, None));

    let on_search = move |evt: Event<FormData>| {
        let val = evt.value();
        search_query.set(val.clone());
        if val.len() >= 2 {
            students.restart();
        }
    };

    rsx! {
        div { class: "page-header",
            h1 { "Alumnos" }
            p { "Gestión de estudiantes - haga clic en un alumno para ver su ficha 360°" }
        }
        div { class: "page-toolbar",
            div { class: "search-input-wrapper",
                Icon { name: "search", size: 18 }
                input {
                    class: "search-input",
                    placeholder: "Buscar por nombre o RUT...",
                    "aria-label": "Buscar alumnos",
                    value: "{search_query}",
                    oninput: on_search,
                }
            }
        }
        div { class: "data-table-container",
            table { class: "data-table",
                thead {
                    tr {
                        th { scope: "col", "RUT" }
                        th { scope: "col", "Nombre" }
                        th { scope: "col", "Curso" }
                        th { scope: "col", "Sección" }
                        th { scope: "col", "Estado" }
                    }
                }
                tbody {
                    match students() {
                        Some(Ok(data)) => {
                            let rows = data["students"].as_array().cloned().unwrap_or_default();
                            if rows.is_empty() {
                                rsx! { tr { td { colspan: "5", class: "empty-state", "No se encontraron alumnos" } } }
                            } else {
                                rsx! {
                                    for student in rows {
                                        StudentRow { student: student }
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => rsx! {
                            tr { td { colspan: "5", class: "empty-state", "Error: {e}" } }
                        },
                        None => rsx! {
                            tr { td { colspan: "5", class: "empty-state", div { class: "loading-spinner", "Cargando..." } } }
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn StudentRow(student: serde_json::Value) -> Element {
    use crate::route::Route;

    let sid = student["id"].as_str().unwrap_or("").to_string();
    let rut = student["rut"].as_str().unwrap_or("-").to_string();
    let first = student["first_name"].as_str().unwrap_or("").to_string();
    let last = student["last_name"].as_str().unwrap_or("").to_string();
    let grade = student["grade_level"].as_str().unwrap_or("-").to_string();
    let section = student["section"].as_str().unwrap_or("-").to_string();
    let enrolled = student["enrolled"].as_bool().unwrap_or(true);
    let avatar = first_letter(&first);
    let api_base = format!("/api/students/{}", sid);
    let sid_clone = sid.clone();

    rsx! {
        tr {
            td {
                Link { to: Route::StudentDetailPage { student_id: sid.clone() },
                    span { class: "rut-badge", "{rut}" }
                }
            }
            td { class: "cell-name",
                div { class: "avatar-sm", "{avatar}" }
                Link { to: Route::StudentDetailPage { student_id: sid },
                    span { "{first} {last}" }
                }
            }
            td {
                InlineEdit {
                    value: grade.clone(),
                    field: "grade_level".to_string(),
                    entity_id: sid_clone.clone(),
                    api_url: api_base.clone(),
                    input_type: None,
                    options: None,
                }
            }
            td {
                InlineEdit {
                    value: section.clone(),
                    field: "section".to_string(),
                    entity_id: sid_clone.clone(),
                    api_url: api_base.clone(),
                    input_type: None,
                    options: None,
                }
            }
            td {
                if enrolled {
                    span { class: "status-active", "Activo" }
                } else {
                    span { class: "status-inactive", "Inactivo" }
                }
            }
        }
    }
}
