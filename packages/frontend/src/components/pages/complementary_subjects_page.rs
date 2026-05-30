use dioxus::prelude::*;
use crate::api::client;
use crate::seo::use_page_title;

#[component]
pub fn ComplementarySubjectsPage() -> Element {
    use_page_title("Asignaturas Complementarias");
    let mut course_id = use_signal(String::new);
    let subjects = use_resource(move || {
        let id = course_id();
        async move { client::fetch_complementary_subjects(&id).await }
    });

    rsx! {
        div { class: "page-header",
            h1 { "Asignaturas Complementarias" }
            p { "Talleres, preuniversitarios y actividades extracurriculares" }
        }
        div { class: "widget-card",
            div { class: "widget-card-body",
                div { class: "form-group",
                    label { "ID del Curso:" }
                    input {
                        class: "form-input",
                        value: "{course_id}",
                        oninput: move |e| course_id.set(e.value()),
                        placeholder: "Ingresa el UUID del curso"
                    }
                }
            }
        }
        if !course_id().is_empty() {
            match subjects() {
                Some(Ok(data)) => {
                    let list = data["subjects"].as_array().cloned().unwrap_or_default();
                    let rows: Vec<Element> = list.iter().map(|s| {
                        let name = s["name"].as_str().unwrap_or("").to_string();
                        let desc = s["description"].as_str().unwrap_or("").to_string();
                        let max = s["max"].as_i64().unwrap_or(0);
                        let active = s["active"].as_bool().unwrap_or(false);
                        rsx! {
                            div { class: "alert-item",
                                div { class: "alert-info",
                                    div { class: "alert-name", "{name}" }
                                    div { class: "alert-detail", "{desc} — Cupo: {max}" }
                                }
                            }
                        }
                    }).collect();
                    rsx! {
                        div { class: "widget-card",
                            div { class: "widget-card-header",
                                h3 { "Asignaturas del Curso" }
                                span { "{list.len()} asignaturas" }
                            }
                            div { class: "widget-card-body",
                                if rows.is_empty() {
                                    div { class: "empty-state", "Sin asignaturas complementarias" }
                                } else {
                                    {rows.into_iter()}
                                }
                            }
                        }
                    }
                }
                _ => rsx! { div { class: "loading-spinner", "Cargando..." } }
            }
        }
    }
}
