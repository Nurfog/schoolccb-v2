use crate::api::client;
use dioxus::prelude::*;

#[component]
pub fn InterviewProcessPage() -> Element {
    let mut interviews = use_resource(|| client::fetch_interviews());
    let mut candidate_name = use_signal(String::new);
    let mut position = use_signal(String::new);
    let mut interview_date = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let mut show_form = use_signal(|| false);
    let mut saving = use_signal(|| false);

    let reset_form = move || {
        candidate_name.set(String::new());
        position.set(String::new());
        interview_date.set(String::new());
        notes.set(String::new());
        show_form.set(false);
    };

    let do_save = move |_| {
        saving.set(true);
        let payload = serde_json::json!({
            "candidate_name": candidate_name(),
            "position": position(),
            "interview_date": interview_date(),
            "notes": notes(),
        });
        spawn(async move {
            let _ = client::create_interview(&payload).await;
            saving.set(false);
            reset_form();
            interviews.restart();
        });
    };

    let do_update = move |id: String, result: &str| {
        let payload = serde_json::json!({"result": result, "status": if result == "hired" { "contratado" } else if result == "rejected" { "rechazado" } else { "pendiente" }});
        let id_c = id.clone();
        spawn(async move {
            let _ = client::update_interview(&id_c, &payload).await;
            interviews.restart();
        });
    };

    rsx! {
        div { class: "page-header", h1 { "Proceso de Entrevistas" } p { "Selección de personal docente y administrativo" } }
        div { class: "page-toolbar", button { class: "btn btn-primary", onclick: move |_| { reset_form(); show_form.set(true); }, "Nueva Entrevista" } }
        {
            if show_form() {
                rsx! {
                    div { class: "form-card",
                        div { class: "form-row",
                            div { class: "form-group", label { "Candidato:" } input { class: "form-input", value: "{candidate_name}", oninput: move |e| candidate_name.set(e.value()) } }
                            div { class: "form-group", label { "Cargo:" } input { class: "form-input", value: "{position}", oninput: move |e| position.set(e.value()) } }
                        }
                        div { class: "form-row",
                            div { class: "form-group", label { "Fecha Entrevista:" } input { class: "form-input", value: "{interview_date}", oninput: move |e| interview_date.set(e.value()), type: "datetime-local" } }
                            div { class: "form-group" }
                        }
                        div { class: "form-group", label { "Notas:" } textarea { class: "form-input", value: "{notes}", oninput: move |e| notes.set(e.value()), rows: "3" } }
                        div { class: "form-actions",
                            button { class: "btn btn-primary", disabled: saving(), onclick: do_save, if saving() { "Guardando..." } else { "Guardar" } }
                            button { class: "btn", onclick: move |_| reset_form(), "Cancelar" }
                        }
                    }
                }
            } else { rsx! {} }
        }
        div { class: "data-table-container",
            match interviews() {
                Some(Ok(j)) => {
                    let rows: Vec<(String, String, String, String, String, String)> = j["interviews"].as_array().map(|arr| arr.iter().map(|r| {
                        (r["id"].as_str().unwrap_or("").to_string(), r["candidate"].as_str().unwrap_or("").to_string(), r["position"].as_str().unwrap_or("").to_string(), r["date"].as_str().unwrap_or("").to_string(), r["result"].as_str().unwrap_or("pending").to_string(), r["status"].as_str().unwrap_or("").to_string())
                    }).collect()).unwrap_or_default();
                    rsx! {
                        table { class: "data-table",
                            thead { tr { th { "Candidato" } th { "Cargo" } th { "Fecha" } th { "Resultado" } th { "Estado" } th { "Acciones" } } }
                            tbody { for (id, name, pos, date, result, status) in &rows {
                                tr {
                                    td { "{name}" }
                                    td { "{pos}" }
                                    td { "{date}" }
                                    td {
                                        if result == "passed" { span { class: "grade-good", "Aprobado" } }
                                        else if result == "hired" { span { class: "grade-good", "Contratado" } }
                                        else if result == "rejected" { span { class: "grade-bad", "Rechazado" } }
                                        else { span { style: "color: #ff9800", "Pendiente" } }
                                    }
                                    td { "{status}" }
                                    td {
                                        button { class: "btn btn-sm btn-success", onclick: { let i = id.clone(); move |_| do_update(i.clone(), "passed") }, "Aprobar" }
                                        button { class: "btn btn-sm btn-primary", style: "margin-left:4px", onclick: { let i = id.clone(); move |_| do_update(i.clone(), "hired") }, "Contratar" }
                                        button { class: "btn btn-sm btn-danger", style: "margin-left:4px", onclick: { let i = id.clone(); move |_| do_update(i.clone(), "rejected") }, "Rechazar" }
                                    }
                                }
                            }}
                        }
                    }
                }
                Some(Err(e)) => rsx! { div { class: "empty-state", "Error: {e}" } },
                None => rsx! { div { class: "empty-state", div { class: "loading-spinner", "Cargando..." } } },
            }
        }
    }
}
