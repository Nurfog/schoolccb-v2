use crate::api::client;
use crate::components::widgets::admission_metrics::AdmissionMetricsWidget;
use dioxus::prelude::*;

mod detail;
mod pipeline;

#[component]
pub fn AdmissionPage() -> Element {
    let stages = use_resource(|| client::fetch_pipeline_stages());
    let mut prospects = use_resource(|| client::fetch_prospects());
    let selected_id = use_signal(|| None::<String>);
    let prospect_detail = use_resource(move || {
        let sid = selected_id();
        async move {
            match sid {
                Some(id) => client::fetch_prospect(&id).await,
                None => Err("none".to_string()),
            }
        }
    });
    let mut show_new = use_signal(|| false);
    let mut first_name = use_signal(String::new);
    let mut last_name = use_signal(String::new);
    let mut rut = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut source = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let mut saving = use_signal(|| false);
    let editing_prospect = use_signal(|| false);
    let mut view_mode = use_signal(|| "table".to_string());
    let edit_first_name = use_signal(String::new);
    let edit_last_name = use_signal(String::new);
    let edit_rut = use_signal(String::new);
    let edit_email = use_signal(String::new);
    let edit_phone = use_signal(String::new);
    let edit_source = use_signal(String::new);
    let edit_notes = use_signal(String::new);
    let vacancies = use_resource(|| client::check_vacancies());

    let do_create = move |_| {
        saving.set(true);
        let payload = serde_json::json!({
            "first_name": first_name(),
            "last_name": last_name(),
            "rut": rut(),
            "email": email(),
            "phone": phone(),
            "source": source(),
            "notes": notes(),
        });
        spawn(async move {
            let _ = client::create_prospect(&payload).await;
            saving.set(false);
            show_new.set(false);
            first_name.set(String::new());
            last_name.set(String::new());
            rut.set(String::new());
            email.set(String::new());
            phone.set(String::new());
            source.set(String::new());
            notes.set(String::new());
            prospects.restart();
        });
    };

    rsx! {
        div { class: "page-header",
            h1 { "Admisiones" }
            p { "Pipeline de postulantes - gestione el ciclo de admisión" }
        }
        div { class: "page-toolbar",
            button { class: "btn btn-primary", onclick: move |_| show_new.set(!show_new()), if show_new() { "Cancelar" } else { "Nuevo Postulante" } }
            button { class: "btn btn-secondary", onclick: move |_| view_mode.set(if view_mode() == "kanban" { "table".to_string() } else { "kanban".to_string() }),
                if view_mode() == "kanban" { "Vista Tabla" } else { "Vista Kanban" }
            }
        }
        {
            if show_new() {
                rsx! {
                    div { class: "form-card",
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Nombres:" }
                                input { class: "form-input", value: "{first_name}", oninput: move |e| first_name.set(e.value()), placeholder: "Juan" }
                            }
                            div { class: "form-group",
                                label { "Apellidos:" }
                                input { class: "form-input", value: "{last_name}", oninput: move |e| last_name.set(e.value()), placeholder: "Pérez" }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "RUT:" }
                                input { class: "form-input", value: "{rut}", oninput: move |e| rut.set(e.value()), placeholder: "12.345.678-9" }
                            }
                            div { class: "form-group",
                                label { "Email:" }
                                input { class: "form-input", value: "{email}", oninput: move |e| email.set(e.value()), placeholder: "juan@ejemplo.cl" }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Teléfono:" }
                                input { class: "form-input", value: "{phone}", oninput: move |e| phone.set(e.value()), placeholder: "+56 9 1234 5678" }
                            }
                            div { class: "form-group",
                                label { "Origen:" }
                                select { class: "form-input", value: "{source}", oninput: move |e| source.set(e.value()),
                                    option { value: "", "Seleccionar..." }
                                    option { value: "web", "Sitio Web" }
                                    option { value: "referido", "Referido" }
                                    option { value: "red_social", "Red Social" }
                                    option { value: "feria", "Feria Educativa" }
                                    option { value: "otro", "Otro" }
                                }
                            }
                        }
                        div { class: "form-actions",
                            button { class: "btn btn-primary", disabled: saving(), onclick: do_create, if saving() { "Guardando..." } else { "Crear Postulante" } }
                            button { class: "btn", onclick: move |_| show_new.set(false), "Cancelar" }
                        }
                    }
                }
            } else { rsx! {} }
        }
        div { class: "dashboard-grid", AdmissionMetricsWidget {} }
        { if view_mode() == "kanban" {
            rsx! {
                pipeline::KanbanBoard {
                    stages: stages,
                    prospects: prospects,
                    selected_id: selected_id,
                }
            }
        } else {
            rsx! { pipeline::ProspectTable { prospects: prospects } }
        }}
        div { class: "vacancy-section",
            h3 { "Disponibilidad por Nivel" }
            {
                match vacancies() {
                    Some(Ok(j)) => {
                        let list = j["vacancies"].as_array().cloned().unwrap_or_default();
                        let rows: Vec<(String, i64, i64, i64)> = list.iter().map(|v| {
                            (v["grade_level"].as_str().unwrap_or("").to_string(),
                             v["total_capacity"].as_i64().unwrap_or(0),
                             v["enrolled_count"].as_i64().unwrap_or(0),
                             v["available"].as_i64().unwrap_or(0))
                        }).collect();
                        rsx! {
                            div { class: "vacancy-grid",
                                for (level, cap, enrolled, avail) in &rows {
                                    div { class: "vacancy-card",
                                        div { class: "vacancy-level", "{level}" }
                                        div { class: "vacancy-numbers",
                                            span { "Cupos: {cap}  |  " }
                                            span { "Matriculados: {enrolled}  |  " }
                                            span { class: if *avail > 0 { "vacancy-ok" } else { "vacancy-full" }, "Disponibles: {avail}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => rsx! { div { class: "loading-spinner", "Cargando disponibilidad..." } },
                }
            }
        }

        detail::ProspectDetailModal {
            prospect_detail: prospect_detail,
            stages: stages,
            prospects: prospects,
            selected_id: selected_id,
            editing_prospect: editing_prospect,
            edit_first_name: edit_first_name,
            edit_last_name: edit_last_name,
            edit_rut: edit_rut,
            edit_email: edit_email,
            edit_phone: edit_phone,
            edit_source: edit_source,
            edit_notes: edit_notes,
            saving: saving,
        }
    }
}
