use dioxus::prelude::*;
use serde_json::{Value, json};

use crate::api::client;
use crate::seo::use_page_title;

#[component]
pub fn ScholarshipsPage() -> Element {
    use_page_title("Becas");
    let data = use_resource(|| client::fetch_json("/api/admission/scholarships"));
    let contracts = use_resource(|| client::fetch_json("/api/admission/contracts"));

    let mut show_form = use_signal(|| false);
    let mut name = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut discount = use_signal(|| "10".to_string());
    let mut max_be = use_signal(|| "0".to_string());
    let mut saving = use_signal(|| false);
    let mut apply_sid = use_signal(|| None::<String>);
    let mut apply_to = use_signal(|| None::<String>);

    let list: Vec<Value> = match data() {
        Some(Ok(ref d)) => d["scholarships"].as_array().cloned().unwrap_or_default(),
        _ => vec![],
    };
    let contract_list: Vec<Value> = match contracts() {
        Some(Ok(ref d)) => d["contracts"].as_array().cloned().unwrap_or_default(),
        _ => vec![],
    };

    let do_create = move |_| {
        saving.set(true);
        let payload = json!({
            "name": name(), "description": description(),
            "discount": discount().parse::<f64>().unwrap_or(10.0),
            "max_beneficiaries": max_be().parse::<i64>().unwrap_or(0),
        });
        spawn(async move {
            let _ = client::post_json("/api/admission/scholarships", &payload).await;
            saving.set(false); show_form.set(false);
            name.set(String::new()); description.set(String::new());
            data.restart();
        });
    };

    let do_toggle = move |id: String| {
        spawn(async move {
            let _ = client::put_json(&format!("/api/admission/scholarships/{}/toggle", id), &json!({})).await;
            data.restart();
        });
    };

    let do_apply = move |sid: String| {
        let s_id = apply_sid();
        let payload = json!({"student_id": s_id});
        spawn(async move {
            let _ = client::post_json(&format!("/api/admission/scholarships/{}/apply", sid), &payload).await;
            data.restart();
        });
    };

    rsx! {
        div { class: "page-header",
            h1 { "Gestión de Becas" }
            p { "Administra becas y descuentos para matrícula" }
        }

        div { class: "page-toolbar",
            button { class: "btn btn-primary", onclick: move |_| show_form.set(!show_form()),
                if show_form() { "Cancelar" } else { "Nueva Beca" }
            }
        }

        if show_form() {
            div { class: "form-card",
                h3 { "Nueva Beca" }
                div { class: "form-group",
                    label { "Nombre *" }
                    input { class: "form-input", value: "{name}", oninput: move |e| name.set(e.value()) }
                }
                div { class: "form-group",
                    label { "Descripción" }
                    textarea { class: "form-input", rows: 2, value: "{description}", oninput: move |e| description.set(e.value()) }
                }
                div { class: "form-row",
                    div { class: "form-group",
                        label { "Descuento (%)" }
                        input { class: "form-input", r#type: "number", min: 0, max: 100, value: "{discount}",
                            oninput: move |e| discount.set(e.value()) }
                    }
                    div { class: "form-group",
                        label { "Máx. beneficiarios (0 = ilimitado)" }
                        input { class: "form-input", r#type: "number", min: 0, value: "{max_be}",
                            oninput: move |e| max_be.set(e.value()) }
                    }
                }
                button { class: "btn btn-primary", disabled: saving(), onclick: do_create,
                    if saving() { "Guardando..." } else { "Crear Beca" }
                }
            }
        }

        div { class: "dashboard-section",
            h3 { "Becas" }
            if list.is_empty() {
                div { class: "empty-state", "No hay becas configuradas" }
            } else {
                div { class: "data-table-container",
                    table { class: "data-table",
                        thead { tr { th { "Nombre" } th { "Dto." } th { "Usados" } th { "Estado" } th { "Acciones" } } }
                        tbody {
                            {list.iter().map(|s| {
                                let sid = s["id"].as_str().unwrap_or("").to_string();
                                let nm = s["name"].as_str().unwrap_or("").to_string();
                                let dsc = s["discount"].as_f64().unwrap_or(0.0);
                                let cur = s["current"].as_i64().unwrap_or(0);
                                let max = s["max"].as_i64().unwrap_or(0);
                                let active = s["active"].as_bool().unwrap_or(false);
                                let sid_c = sid.clone();
                                rsx! {
                                    tr {
                                        td { b { "{nm}" } }
                                        td { "{dsc:.0}%" }
                                        td { "{cur}/{if max > 0 { max.to_string() } else { \"∞\" }}" }
                                        td { if active { span { class: "badge badge-success", "Activa" } } else { span { class: "badge badge-error", "Inactiva" } } }
                                        td {
                                            button { class: "btn btn-sm", onclick: move |_| {
                                                apply_sid.set(Some(sid_c.clone()));
                                                apply_to.set(Some(sid.clone()));
                                            }, "Aplicar" }
                                            button { class: "btn btn-sm", onclick: move |_| do_toggle(sid.clone()), "Toggle" }
                                        }
                                    }
                                }
                            })}
                        }
                    }
                }
            }
        }

        div { class: "dashboard-section",
            h3 { "Contratos de Matrícula" }
            if contract_list.is_empty() {
                div { class: "empty-state", "Sin contratos registrados" }
            } else {
                div { class: "data-table-container",
                    table { class: "data-table",
                        thead { tr { th { "Estudiante" } th { "Nivel" } th { "Monto" } th { "Estado" } th { "Fecha" } } }
                        tbody {
                            {contract_list.iter().map(|c| {
                                let name = c["student"].as_str().unwrap_or("").to_string();
                                let grade = c["grade"].as_str().unwrap_or("").to_string();
                                let amount = c["amount"].as_f64().unwrap_or(0.0);
                                let status = c["status"].as_str().unwrap_or("").to_string();
                                let date = c["date"].as_str().unwrap_or("").to_string();
                                rsx! {
                                    tr {
                                        td { "{name}" }
                                        td { "{grade}" }
                                        td { "${amount:.0}" }
                                        td { span { class: "badge badge-{status}", "{status}" } }
                                        td { "{date}" }
                                    }
                                }
                            })}
                        }
                    }
                }
            }
        }
    }
}
