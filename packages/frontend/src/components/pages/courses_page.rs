use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client;
use crate::components::widgets::icon::Icon;
use crate::components::widgets::searchable_select::SearchableSelect;

fn needs_plan(level: &str) -> bool {
    level == "3° Medio" || level == "4° Medio"
}

#[component]
pub fn CoursesPage() -> Element {
    let mut courses = use_resource(|| client::fetch_json("/api/courses"));
    let mut name = use_signal(|| "".to_string());
    let mut grade_level = use_signal(|| "".to_string());
    let mut plan = use_signal(|| "".to_string());
    let mut section = use_signal(|| "".to_string());
    let mut teacher_id = use_signal(|| "".to_string());
    let mut editing_id = use_signal(|| None::<String>);
    let mut show_form = use_signal(|| false);
    let mut saving = use_signal(|| false);

    let mut subjects_modal_course = use_signal(|| None::<Value>);
    let mut subjects_modal_open = use_signal(|| false);

    let mut reset_form = move || {
        name.set("".to_string());
        grade_level.set("".to_string());
        plan.set("".to_string());
        section.set("".to_string());
        teacher_id.set("".to_string());
        editing_id.set(None);
        show_form.set(false);
    };

    let do_save = move |_| {
        saving.set(true);
        let mut payload = serde_json::json!({
            "name": name(),
            "grade_level": grade_level(),
            "section": section(),
            "teacher_id": teacher_id(),
        });
        if needs_plan(&grade_level()) && !plan().is_empty() {
            payload["plan"] = serde_json::json!(plan());
        }
        let is_edit = editing_id().is_some();
        let endpoint = if let Some(ref id) = editing_id() {
            format!("/api/courses/{}", id)
        } else {
            "/api/courses".to_string()
        };
        spawn(async move {
            if is_edit {
                let _ = client::put_json(&endpoint, &payload).await;
            } else {
                let _ = client::post_json(&endpoint, &payload).await;
            }
            saving.set(false);
            reset_form();
            courses.restart();
        });
    };

    let do_delete = move |id: String| {
        if !web_sys::window().unwrap().confirm_with_message("¿Estás seguro?").unwrap_or(false) {
            return;
        }
        spawn(async move {
            let _ = client::delete_json(&format!("/api/courses/{}", id)).await;
            courses.restart();
        });
    };

    let do_edit = move |c: Value| {
        name.set(c["name"].as_str().unwrap_or("").to_string());
        grade_level.set(c["grade_level"].as_str().unwrap_or("").to_string());
        plan.set(c["plan"].as_str().unwrap_or("").to_string());
        section.set(c["section"].as_str().unwrap_or("").to_string());
        teacher_id.set(c["teacher_id"].as_str().unwrap_or("").to_string());
        editing_id.set(c["id"].as_str().map(|s| s.to_string()));
        show_form.set(true);
    };

    rsx! {
        div { class: "page-header",
            h1 { "Cursos" }
            p { "Gestión de cursos y asignación de profesores jefe" }
        }
        div { class: "page-toolbar",
            button { class: "btn btn-primary", onclick: move |_| { reset_form(); show_form.set(true); },
                "Nuevo Curso"
            }
        }
        if show_form() {
            div { class: "card form-card",
                h3 { if editing_id().is_some() { "Editar Curso" } else { "Nuevo Curso" } }
                div { class: "form-grid",
                    div { class: "field",
                        label { r#for: "course-name", "Nombre del Curso" }
                        input { id: "course-name", class: "form-input", placeholder: "1° Medio A", value: "{name}",
                            oninput: move |e| name.set(e.value()),
                        }
                    }
                    div { class: "field",
                        label { r#for: "course-level", "Nivel" }
                        select { id: "course-level", class: "form-input", value: "{grade_level}",
                            onchange: move |e| {
                                grade_level.set(e.value());
                                plan.set(String::new());
                            },
                            option { value: "", "Seleccionar..." }
                            option { value: "Sala Cuna", "Sala Cuna" }
                            option { value: "Medio Menor", "Medio Menor" }
                            option { value: "Medio Mayor", "Medio Mayor" }
                            option { value: "Pre-kinder", "Pre-kinder" }
                            option { value: "Kinder", "Kinder" }
                            option { value: "1° Básico", "1° Básico" }
                            option { value: "2° Básico", "2° Básico" }
                            option { value: "3° Básico", "3° Básico" }
                            option { value: "4° Básico", "4° Básico" }
                            option { value: "5° Básico", "5° Básico" }
                            option { value: "6° Básico", "6° Básico" }
                            option { value: "7° Básico", "7° Básico" }
                            option { value: "8° Básico", "8° Básico" }
                            option { value: "1° Medio", "1° Medio" }
                            option { value: "2° Medio", "2° Medio" }
                            option { value: "3° Medio", "3° Medio" }
                            option { value: "4° Medio", "4° Medio" }
                        }
                    }
                    {
                        if needs_plan(&grade_level()) {
                            rsx! {
                                div { class: "field",
                                    label { r#for: "course-plan", "Plan" }
                                    select { id: "course-plan", class: "form-input", value: "{plan}",
                                        onchange: move |e| plan.set(e.value()),
                                        option { value: "", "Seleccionar..." }
                                        option { value: "HC", "Científico-Humanista" }
                                        option { value: "TP", "Técnico-Profesional" }
                                        option { value: "Artístico", "Artístico" }
                                    }
                                }
                            }
                        } else { rsx! {} }
                    }
                    div { class: "field",
                        label { r#for: "course-section", "Sección" }
                        input { id: "course-section", class: "form-input", placeholder: "A", value: "{section}",
                            oninput: move |e| section.set(e.value()),
                        }
                    }
                    div { class: "field",
                        label { "Profesor Jefe" }
                        SearchableSelect {
                            fetch_url: "/api/auth/users".to_string(),
                            results_key: "users".to_string(),
                            label_key: "name".to_string(),
                            value_key: "id".to_string(),
                            placeholder: "Buscar profesor...",
                            on_select: move |id| teacher_id.set(id),
                        }
                    }
                }
                div { class: "form-actions",
                    button { class: "btn-secondary", onclick: move |_| reset_form(), "Cancelar" }
                    button { class: "btn-primary", onclick: do_save, disabled: saving(),
                        if saving() { "Guardando..." } else { "Guardar" }
                    }
                }
            }
        }
        div { class: "data-table-container",
            table { class: "data-table",
                thead {
                    tr {
                        th { scope: "col", "Nombre" }
                        th { scope: "col", "Nivel" }
                        th { scope: "col", "Plan" }
                        th { scope: "col", "Sección" }
                        th { scope: "col", "Profesor ID" }
                        th { scope: "col", "Acciones" }
                    }
                }
                tbody {
                    match courses() {
                        Some(Ok(data)) => {
                            let list = data["courses"].as_array().cloned().unwrap_or_default();
                            if list.is_empty() {
                                rsx! { tr { td { colspan: "6", class: "empty-state", "No hay cursos configurados" } } }
                            } else {
                                rsx! {
                                    for c in &list {
                                        CourseRow {
                                            course: c.clone(),
                                            on_edit: do_edit.clone(),
                                            on_delete: do_delete.clone(),
                                            on_manage_subjects: {
                                                let c = c.clone();
                                                move |_| {
                                                    subjects_modal_course.set(Some(c.clone()));
                                                    subjects_modal_open.set(true);
                                                }
                                            },
                                        }
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => rsx! { tr { td { colspan: "6", class: "empty-state", "Error: {e}" } } },
                        None => rsx! { tr { td { colspan: "6", class: "empty-state", div { class: "loading-spinner", "Cargando..." } } } },
                    }
                }
            }
        }
        if subjects_modal_open() {
            if let Some(ref course) = subjects_modal_course() {
                CourseSubjectsModal {
                    course: course.clone(),
                    on_close: move || {
                        subjects_modal_open.set(false);
                        subjects_modal_course.set(None);
                    },
                }
            }
        }
    }
}

#[component]
fn CourseRow(
    course: Value,
    on_edit: EventHandler<Value>,
    on_delete: EventHandler<String>,
    on_manage_subjects: EventHandler<()>,
) -> Element {
    let id = course["id"].as_str().unwrap_or("").to_string();
    let name = course["name"].as_str().unwrap_or("").to_string();
    let grade_level = course["grade_level"].as_str().unwrap_or("").to_string();
    let plan = course["plan"].as_str().unwrap_or("").to_string();
    let section = course["section"].as_str().unwrap_or("").to_string();
    let teacher_id = course["teacher_id"].as_str().unwrap_or("").to_string();
    let plan_display = if plan.is_empty() {
        "-".to_string()
    } else {
        plan.clone()
    };

    rsx! {
        tr {
            td { class: "cell-name", "{name}" }
            td { "{grade_level}" }
            td { "{plan_display}" }
            td { "{section}" }
            td { class: "cell-mono", "{&teacher_id[..8]}..." }
            td { class: "cell-actions",
                button { class: "btn-icon", title: "Editar",
                    onclick: move |_| on_edit.call(course.clone()),
                    Icon { name: "edit", size: 16 }
                }
                button { class: "btn-icon", title: "Asignaturas",
                    onclick: move |_| on_manage_subjects.call(()),
                    Icon { name: "list", size: 16 }
                }
                button { class: "btn-icon btn-icon-danger", title: "Eliminar",
                    onclick: move |_| on_delete.call(id.clone()),
                    Icon { name: "trash", size: 16 }
                }
            }
        }
    }
}

#[component]
fn CourseSubjectsModal(course: Value, on_close: EventHandler<()>) -> Element {
    let course_name = course["name"].as_str().unwrap_or("").to_string();

    let cid = course["id"].as_str().unwrap_or("").to_string();
    let cid_clone = cid.clone();
    let subjects = use_resource(move || {
        let cid = cid_clone.clone();
        async move { client::fetch_json(&format!("/api/grades/course-subjects/{}/{}", cid, js_sys::Date::new_0().get_full_year())).await }
    });

    let mut assign_subject_id = use_signal(|| "".to_string());
    let mut assign_teacher_id = use_signal(|| "".to_string());
    let mut assign_hours = use_signal(|| "".to_string());
    let mut assign_year = use_signal(|| js_sys::Date::new_0().get_full_year().to_string());
    let mut saving = use_signal(|| false);

    let subject_rows = match &subjects() {
        Some(Ok(data)) => data["course_subjects"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|cs| {
                        let subj_name = cs
                            .get("subject_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("-")
                            .to_string();
                        let subj_code = cs
                            .get("subject_code")
                            .and_then(|v| v.as_str())
                            .unwrap_or("-")
                            .to_string();
                        let teacher_name = cs
                            .get("teacher_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("-")
                            .to_string();
                        let hpw = cs
                            .get("hours_per_week")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        let year = cs
                            .get("academic_year")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        let cs_id = cs
                            .get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        (subj_name, subj_code, teacher_name, hpw, year, cs_id)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        _ => vec![],
    };

    let course_id = cid.clone();
    let do_assign = move |_| {
        saving.set(true);
        let payload = serde_json::json!({
            "course_id": course_id,
            "subject_id": assign_subject_id(),
            "teacher_id": assign_teacher_id(),
            "academic_year": assign_year().parse::<i32>().unwrap_or_else(|_| js_sys::Date::new_0().get_full_year() as i32),
            "hours_per_week": assign_hours().parse::<i32>().ok(),
        });
        let mut subjects = subjects.clone();
        spawn(async move {
            let _ = client::post_json("/api/grades/course-subjects", &payload).await;
            saving.set(false);
            assign_subject_id.set("".to_string());
            assign_teacher_id.set("".to_string());
            assign_hours.set("".to_string());
            subjects.restart();
        });
    };

    let subjects_rc = subjects.clone();

    rsx! {
        div { class: "modal-overlay", onclick: move |_| on_close.call(()),
            div { class: "modal-content", role: "dialog", "aria-modal": "true", onclick: move |e| e.stop_propagation(),
                div { class: "modal-header",
                    h2 { "Asignaturas - {course_name}" }
                    button { class: "btn-icon", onclick: move |_| on_close.call(()),
                        Icon { name: "x", size: 20 }
                    }
                }
                div { class: "modal-body",
                    div { class: "form-row",
                        div { class: "field",
                            label { "Asignatura" }
                            SearchableSelect {
                                fetch_url: "/api/grades/subjects".to_string(),
                                results_key: "subjects".to_string(),
                                label_key: "name".to_string(),
                                value_key: "id".to_string(),
                                placeholder: "Buscar asignatura...",
                                on_select: move |id| assign_subject_id.set(id),
                            }
                        }
                        div { class: "field",
                            label { "Profesor" }
                            SearchableSelect {
                                fetch_url: "/api/auth/users".to_string(),
                                results_key: "users".to_string(),
                                label_key: "name".to_string(),
                                value_key: "id".to_string(),
                                placeholder: "Buscar profesor...",
                                on_select: move |id| assign_teacher_id.set(id),
                            }
                        }
                        div { class: "field",
                            label { r#for: "modal-hours", "Horas" }
                            input { id: "modal-hours", class: "form-input", placeholder: "4", value: "{assign_hours}",
                                oninput: move |e| assign_hours.set(e.value()),
                            }
                        }
                        div { class: "field",
                            label { r#for: "modal-year", "Año" }
                            input { id: "modal-year", class: "form-input", value: "{assign_year}",
                                oninput: move |e| assign_year.set(e.value()),
                            }
                        }
                    }
                    button { class: "btn-primary", onclick: do_assign, disabled: saving(),
                        if saving() { "Asignando..." } else { "Asignar Asignatura" }
                    }
                    hr {}
                    h4 { "Asignaturas asignadas" }
                    if subject_rows.is_empty() {
                        p { class: "empty-state", "No hay asignaturas asignadas a este curso" }
                    } else {
                        table { class: "data-table",
                            thead {
                                tr {
                                    th { scope: "col", "Asignatura" }
                                    th { scope: "col", "Código" }
                                    th { scope: "col", "Profesor" }
                                    th { scope: "col", "Horas" }
                                    th { scope: "col", "Año" }
                                    th { scope: "col", "Acciones" }
                                }
                            }
                            tbody {
                                for row in &subject_rows {
                                    tr {
                                        td { "{row.0}" }
                                        td { "{row.1}" }
                                        td { "{row.2}" }
                                        td { "{row.3}" }
                                        td { "{row.4}" }
                                        td {
                                            button {
                                                class: "btn-icon btn-icon-danger",
                                                title: "Remover",
                                                onclick: {
                                                    let id = row.5.clone();
                                                    let mut subjects = subjects_rc.clone();
                                                    move |_| {
                                                        if !web_sys::window().unwrap().confirm_with_message("¿Estás seguro?").unwrap_or(false) { return; }
                                                        let id = id.clone();
                                                        spawn(async move {
                                                            let _ = client::delete_json(&format!("/api/grades/course-subjects/{}", id)).await;
                                                            subjects.restart();
                                                        });
                                                    }
                                                },
                                                Icon { name: "trash", size: 16 }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
