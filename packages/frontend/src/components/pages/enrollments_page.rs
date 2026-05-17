use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client;
use crate::components::widgets::icon::Icon;
use crate::components::widgets::searchable_select::SearchableSelect;

#[component]
pub fn EnrollmentsPage() -> Element {
    let mut enrollments = use_resource(|| client::fetch_json("/api/enrollments"));
    let mut student_id = use_signal(|| "".to_string());
    let mut course_id = use_signal(|| "".to_string());
    let mut year = use_signal(|| chrono_now_year());
    let mut show_form = use_signal(|| false);
    let mut saving = use_signal(|| false);

    let mut reset_form = move || {
        student_id.set("".to_string());
        course_id.set("".to_string());
        year.set(chrono_now_year());
        show_form.set(false);
    };

    let do_enroll = move |_| {
        saving.set(true);
        let payload = serde_json::json!({
            "student_id": student_id(),
            "course_id": course_id(),
            "year": year(),
        });
        spawn(async move {
            let _ = client::post_json("/api/enrollments", &payload).await;
            saving.set(false);
            reset_form();
            enrollments.restart();
        });
    };

    let do_delete = move |id: String| {
        if !web_sys::window().unwrap().confirm_with_message("¿Estás seguro?").unwrap_or(false) {
            return;
        }
        spawn(async move {
            let _ = client::delete_json(&format!("/api/enrollments/{}", id))
                .await;
            enrollments.restart();
        });
    };

    rsx! {
        div { class: "page-header",
            h1 { "Matrículas" }
            p { "Gestión de matrículas de alumnos en cursos" }
        }
        div { class: "page-toolbar",
            button { class: "btn btn-primary", onclick: move |_| { reset_form(); show_form.set(true); },
                "Nueva Matrícula"
            }
        }
        if show_form() {
            div { class: "card form-card",
                h3 { "Nueva Matrícula" }
                div { class: "form-grid",
                    div { class: "field",
                        label { "Alumno" }
                        SearchableSelect {
                            fetch_url: "/api/students".to_string(),
                            results_key: "students".to_string(),
                            label_key: "first_name".to_string(),
                            value_key: "id".to_string(),
                            placeholder: "Buscar alumno por nombre...",
                            on_select: move |id| student_id.set(id),
                        }
                    }
                    div { class: "field",
                        label { "Curso" }
                        SearchableSelect {
                            fetch_url: "/api/courses".to_string(),
                            results_key: "courses".to_string(),
                            label_key: "name".to_string(),
                            value_key: "id".to_string(),
                            placeholder: "Buscar curso...",
                            on_select: move |id| course_id.set(id),
                        }
                    }
                    div { class: "field",
                        label { "Año Académico" }
                        input { class: "form-input", r#type: "number", value: "{year}",
                            oninput: move |e| year.set(e.value().parse().unwrap_or(chrono_now_year())),
                        }
                    }
                }
                div { class: "form-actions",
                    button { class: "btn-secondary", onclick: move |_| reset_form(), "Cancelar" }
                    button { class: "btn-primary", onclick: do_enroll, disabled: saving(),
                        if saving() { "Matriculando..." } else { "Matricular" }
                    }
                }
            }
        }
        div { class: "data-table-container",
            table { class: "data-table",
                thead {
                    tr {
                        th { "Alumno ID" }
                        th { "Curso ID" }
                        th { "Año" }
                        th { "Estado" }
                        th { "Acciones" }
                    }
                }
                tbody {
                    match enrollments() {
                        Some(Ok(data)) => {
                            let list = data["enrollments"].as_array().cloned().unwrap_or_default();
                            if list.is_empty() {
                                rsx! { tr { td { colspan: "5", class: "empty-state", "No hay matrículas registradas" } } }
                            } else {
                                rsx! {
                                    for e in list {
                                        EnrollmentRow { enrollment: e, on_delete: do_delete.clone() }
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => rsx! { tr { td { colspan: "5", class: "empty-state", "Error: {e}" } } },
                        None => rsx! { tr { td { colspan: "5", class: "empty-state", div { class: "loading-spinner", "Cargando..." } } } },
                    }
                }
            }
        }
    }
}

fn chrono_now_year() -> i32 {
    let date = js_sys::Date::new_0();
    date.get_full_year() as i32
}

#[component]
fn EnrollmentRow(enrollment: Value, on_delete: EventHandler<String>) -> Element {
    let id = enrollment["id"].as_str().unwrap_or("").to_string();
    let student_id = enrollment["student_id"].as_str().unwrap_or("").to_string();
    let course_id = enrollment["course_id"].as_str().unwrap_or("").to_string();
    let year = enrollment["year"].as_i64().unwrap_or(0);
    let active = enrollment["active"].as_bool().unwrap_or(false);

    rsx! {
        tr {
            td { class: "cell-mono", "{&student_id[..8]}..." }
            td { class: "cell-mono", "{&course_id[..8]}..." }
            td { "{year}" }
            td {
                if active {
                    span { class: "status-active", "Activa" }
                } else {
                    span { class: "status-inactive", "Inactiva" }
                }
            }
            td { class: "cell-actions",
                button { class: "btn-icon btn-icon-danger", onclick: move |_| on_delete.call(id.clone()),
                    Icon { name: "trash", size: 16 }
                }
            }
        }
    }
}
