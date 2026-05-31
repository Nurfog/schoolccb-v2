use crate::api::client;
use crate::components::widgets::business_process_flow::BusinessProcessFlow;
use crate::components::widgets::custom_fields_section::CustomFieldsSection;
use dioxus::prelude::*;
use js_sys::eval as js_eval;

#[component]
pub fn ProspectDetailModal(
    prospect_detail: Resource<Result<serde_json::Value, String>>,
    stages: Resource<Result<serde_json::Value, String>>,
    prospects: Resource<Result<serde_json::Value, String>>,
    selected_id: Signal<Option<String>>,
    editing_prospect: Signal<bool>,
    edit_first_name: Signal<String>,
    edit_last_name: Signal<String>,
    edit_rut: Signal<String>,
    edit_email: Signal<String>,
    edit_phone: Signal<String>,
    edit_source: Signal<String>,
    edit_notes: Signal<String>,
    saving: Signal<bool>,
) -> Element {
    let close = move |_: Event<MouseData>| selected_id.set(None);

    rsx! {
        {
            match prospect_detail() {
                Some(Ok(j)) => {
                    let p = &j["prospect"];
                    let activities = j["activities"].as_array().cloned().unwrap_or_default();
                    let documents = j["documents"].as_array().cloned().unwrap_or_default();
                    let pid = p["id"].as_str().unwrap_or("").to_string();
                    let pname = format!("{} {}", p["first_name"].as_str().unwrap_or(""), p["last_name"].as_str().unwrap_or(""));
                    let prut = p["rut"].as_str().unwrap_or("-").to_string();
                    let pemail = p["email"].as_str().unwrap_or("-").to_string();
                    let pphone = p["phone"].as_str().unwrap_or("-").to_string();
                    let psource = p["source"].as_str().unwrap_or("-").to_string();
                    let pstage = p["current_stage_id"].as_str().unwrap_or("").to_string();

                    let stage_options: Vec<Element> = stages().and_then(|r| r.ok()).map(|sj| {
                        sj["stages"].as_array().cloned().unwrap_or_default().into_iter().filter_map(|s| {
                            let sid = s["id"].as_str()?.to_string();
                            let sname = s["name"].as_str()?.to_string();
                            Some(rsx! { option { value: "{sid}", selected: sid == pstage, "{sname}" } })
                        }).collect::<Vec<_>>()
                    }).unwrap_or_default();

                    let do_stage_change = { let pid = pid.clone(); move |e: Event<FormData>| {
                        let new_stage = e.value();
                        spawn({
                            let pid = pid.clone();
                            async move {
                                let _ = client::change_prospect_stage(&pid, &new_stage).await;
                                prospects.restart();
                            }
                        });
                    }};

                    let do_delete_prospect = { let pid = pid.clone(); move |_| {
                        if !web_sys::window().unwrap().confirm_with_message("¿Estás seguro?").unwrap_or(false) { return; }
                        spawn({
                            let pid = pid.clone();
                            async move {
                                let _ = client::delete_prospect(&pid).await;
                                selected_id.set(None);
                                prospects.restart();
                            }
                        });
                    }};

                    let do_edit_save = { let pid = pid.clone(); move |_| {
                        saving.set(true);
                        let payload = serde_json::json!({
                            "first_name": edit_first_name(),
                            "last_name": edit_last_name(),
                            "rut": edit_rut(),
                            "email": edit_email(),
                            "phone": edit_phone(),
                            "source": edit_source(),
                            "notes": edit_notes(),
                        });
                        spawn({
                            let pid = pid.clone();
                            async move {
                                let _ = client::update_prospect(&pid, &payload).await;
                                saving.set(false);
                                editing_prospect.set(false);
                                prospects.restart();
                            }
                        });
                    }};

    let reminders = j["reminders"].as_array().cloned().unwrap_or_default();

    let activity_items: Vec<Element> = activities.iter().map(|a| {
        let atype = a["activity_type"].as_str().unwrap_or("").to_string();
        let asubj = a["subject"].as_str().unwrap_or("").to_string();
        rsx! { div { class: "activity-item",
            span { class: "activity-type", "{atype}" }
            span { "{asubj}" }
        }}
    }).collect();

    let doc_items: Vec<Element> = documents.iter().map(|d| {
        let fname = d["file_name"].as_str().unwrap_or("").to_string();
        let verified = d["is_verified"].as_bool().unwrap_or(false);
        rsx! { div { class: "doc-item",
            span { "{fname}" }
            span { class: "doc-status",
                if verified { "✓ Verificado" } else { "⏳ Pendiente" }
            }
        }}
    }).collect();

    let reminder_items: Vec<Element> = reminders.iter().map(|r| {
        let rtitle = r["title"].as_str().unwrap_or("").to_string();
        let rtype = r["reminder_type"].as_str().unwrap_or("").to_string();
        let sent = r["is_sent"].as_bool().unwrap_or(false);
        let rid = r["id"].as_str().unwrap_or("").to_string();
        rsx! {
            div { class: "reminder-item",
                span { class: "reminder-type", "{rtype}" }
                span { "{rtitle}" }
                span { class: if sent { "reminder-sent" } else { "reminder-pending" },
                    if sent { "✓ Enviado" } else { "⏳ Pendiente" }
                }
                button {
                    class: "btn-icon btn-small",
                    onclick: move |_| {
                        spawn({
                            let rid = rid.clone();
                            async move {
                                let _ = client::delete_reminder(&rid).await;
                                prospect_detail.restart();
                            }
                        });
                    },
                    "✕"
                }
            }
        }
    }).collect();

                    rsx! {
                        div { class: "modal-overlay", role: "dialog", "aria-modal": "true", "aria-label": "Detalle del postulante", tabindex: "-1", onclick: close, onkeydown: move |e| if e.key() == Key::Escape { selected_id.set(None); },
                            div { class: "modal-content", onclick: move |e| e.stop_propagation(),
                                div { class: "modal-header",
                                    h2 { "{pname}" }
                                    button { class: "btn-icon", "aria-label": "Cerrar", onclick: close, "✕" }
                                }
                                {
                                    let stages_data = stages();
                                    let p_stage = pstage.clone();
                                    match stages_data {
                                        Some(Ok(sj)) => {
                                            let list = sj["stages"].as_array().cloned().unwrap_or_default();
                                            rsx! {
                                                BusinessProcessFlow { stages: list, current_stage_id: p_stage }
                                            }
                                        }
                                        _ => rsx! {},
                                    }
                                }
                                div { class: "modal-body",
                                    if editing_prospect() {
                                        div { class: "form-card",
                                            h4 { "Editar Postulante" }
                                            div { class: "form-row",
                                                div { class: "form-group",
                                                    label { "Nombres:" }
                                                    input { class: "form-input", value: "{edit_first_name}", oninput: move |e| edit_first_name.set(e.value()) }
                                                }
                                                div { class: "form-group",
                                                    label { "Apellidos:" }
                                                    input { class: "form-input", value: "{edit_last_name}", oninput: move |e| edit_last_name.set(e.value()) }
                                                }
                                            }
                                            div { class: "form-row",
                                                div { class: "form-group",
                                                    label { "RUT:" }
                                                    input { class: "form-input", value: "{edit_rut}", oninput: move |e| edit_rut.set(e.value()) }
                                                }
                                                div { class: "form-group",
                                                    label { "Email:" }
                                                    input { class: "form-input", value: "{edit_email}", oninput: move |e| edit_email.set(e.value()) }
                                                }
                                            }
                                            div { class: "form-row",
                                                div { class: "form-group",
                                                    label { "Teléfono:" }
                                                    input { class: "form-input", value: "{edit_phone}", oninput: move |e| edit_phone.set(e.value()) }
                                                }
                                                div { class: "form-group",
                                                    label { "Origen:" }
                                                    select { class: "form-input", value: "{edit_source}", oninput: move |e| edit_source.set(e.value()),
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
                                                button { class: "btn btn-primary", disabled: saving(), onclick: do_edit_save, if saving() { "Guardando..." } else { "Guardar" } }
                                                button { class: "btn", onclick: move |_| editing_prospect.set(false), "Cancelar" }
                                            }
                                        }
                                    } else {
                                        div { class: "detail-grid",
                                            div { class: "detail-section",
                                                h4 { "Datos Personales" }
                                                p { "RUT: {prut}" }
                                                p { "Email: {pemail}" }
                                                p { "Teléfono: {pphone}" }
                                                p { "Origen: {psource}" }
                                            }
                                            div { class: "detail-section",
                                                h4 { "Cambiar Etapa" }
                                                select { class: "form-input", oninput: do_stage_change, {stage_options.into_iter()} }
                                            }
                                            div { class: "detail-section",
                                                h4 { "Actividades ({activities.len()})" }
                                                {
                                                    if activity_items.is_empty() {
                                                        rsx! { p { "Sin actividades registradas" } }
                                                    } else {
                                                        rsx! { { activity_items.into_iter() } }
                                                    }
                                                }
                                            }
                                            div { class: "detail-section",
                                                h4 { "Documentos ({documents.len()})" }
                                                {
                                                    if doc_items.is_empty() {
                                                        rsx! { p { "Sin documentos" } }
                                                    } else {
                                                        rsx! { { doc_items.into_iter() } }
                                                    }
                                                }
                                            }
                                            div { class: "detail-section",
                                                h4 { "Recordatorios ({reminders.len()})" }
                                                {
                                                    if reminder_items.is_empty() {
                                                        rsx! { p { "Sin recordatorios" } }
                                                    } else {
                                                        rsx! { { reminder_items.into_iter() } }
                                                    }
                                                }
                                                div { class: "form-row", style: "margin-top: 8px; gap: 4px;",
                                                    input {
                                                        class: "form-input",
                                                        placeholder: "Título",
                                                        id: "reminder-title",
                                                        style: "flex: 1;",
                                                    }
                                                    input {
                                                        class: "form-input",
                                                        r#type: "datetime-local",
                                                        id: "reminder-date",
                                                        style: "width: auto;",
                                                    }
                                                    button {
                                                        class: "btn btn-primary btn-small",
                                                        onclick: move |_| {
                                                    let title = js_eval("document.getElementById('reminder-title').value");
                                                    let remind_at = js_eval("document.getElementById('reminder-date').value");
                                                    spawn({
                                                        let pid = pid.clone();
                                                        async move {
                                                            let title_str = title.ok().and_then(|t| t.as_string());
                                                            let date_str = remind_at.ok().and_then(|d| d.as_string());
                                                            if let (Some(t), Some(d)) = (title_str, date_str) {
                                                                if !t.is_empty() && !d.is_empty() {
                                                                    let payload = serde_json::json!({
                                                                        "prospect_id": pid,
                                                                        "reminder_type": "follow_up",
                                                                        "title": t,
                                                                        "remind_at": format!("{}:00Z", d),
                                                                    });
                                                                    let _ = client::create_reminder(&payload).await;
                                                                    prospect_detail.restart();
                                                                }
                                                            }
                                                        }
                                                    });
                                                        },
                                                        "Agregar"
                                                    }
                                                }
                                            }
                                            CustomFieldsSection { entity_id: pid.clone(), entity_type: "prospect".to_string() }
                                        }
                                    }
                                }
                                div { class: "modal-footer",
                                    button { class: "btn", onclick: { let fn_ = p["first_name"].as_str().unwrap_or("").to_string(); let ln = p["last_name"].as_str().unwrap_or("").to_string(); let r = p["rut"].as_str().unwrap_or("").to_string(); let e = p["email"].as_str().unwrap_or("").to_string(); let ph = p["phone"].as_str().unwrap_or("").to_string(); let s = p["source"].as_str().unwrap_or("").to_string(); let n = p["notes"].as_str().unwrap_or("").to_string(); move |_| { edit_first_name.set(fn_.clone()); edit_last_name.set(ln.clone()); edit_rut.set(r.clone()); edit_email.set(e.clone()); edit_phone.set(ph.clone()); edit_source.set(s.clone()); edit_notes.set(n.clone()); editing_prospect.set(true); } }, "Editar" }
                                    button { class: "btn btn-danger", onclick: do_delete_prospect, "Eliminar" }
                                }
                            }
                        }
                    }
                }
                Some(Err(_)) => rsx! {},
                None => rsx! {},
            }
        }
    }
}
