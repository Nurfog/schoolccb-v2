use dioxus::prelude::*;

use crate::api::client;
use crate::components::widgets::student_search::StudentSearchSelect;

#[component]
pub fn PaymentsTab() -> Element {
    let mut payments = use_resource(|| client::fetch_all_payments());
    let mut show_form = use_signal(|| false);
    let mut fee_id = use_signal(|| String::new());
    let mut student_id = use_signal(|| String::new());
    let mut form_key = use_signal(|| 0u32);
    let mut amount = use_signal(|| String::new());
    let mut payment_method = use_signal(|| "Efectivo".to_string());
    let mut reference = use_signal(|| String::new());
    let mut saving = use_signal(|| false);

    let mut reset_form = move || {
        form_key += 1;
        fee_id.set(String::new());
        student_id.set(String::new());
        amount.set(String::new());
        payment_method.set("Efectivo".to_string());
        reference.set(String::new());
        show_form.set(false);
    };

    let do_save = move |_| {
        if student_id().is_empty() || amount().is_empty() {
            return;
        }
        saving.set(true);
        let payload = serde_json::json!({
            "fee_id": fee_id(),
            "student_id": student_id(),
            "amount": amount().parse::<f64>().unwrap_or(0.0),
            "payment_method": payment_method(),
            "reference": reference(),
        });
        spawn(async move {
            let _ = client::create_payment(&payload).await;
            saving.set(false);
            reset_form();
            payments.restart();
        });
    };

    rsx! {
        div { class: "toolbar-row",
            button { class: "btn btn-primary", onclick: move |_| show_form.set(!show_form()), if show_form() { "Cancelar" } else { "Nuevo Pago" } }
        }
        {
            if show_form() {
                rsx! {
                    div { class: "form-card",
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Estudiante:" }
                                StudentSearchSelect {
                                    on_select: move |id| student_id.set(id),
                                    reset_key: Some(form_key().to_string()),
                                }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "ID Cuota (opcional):" }
                                input { class: "form-input", value: "{fee_id}", oninput: move |evt| fee_id.set(evt.value()), placeholder: "UUID de la cuota" }
                            }
                            div { class: "form-group",
                                label { "Monto:" }
                                input { class: "form-input", value: "{amount}", oninput: move |evt| amount.set(evt.value()), type: "number", step: "1000", placeholder: "0", "aria-required": "true", autocomplete: "off" }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Método:" }
                                select { class: "form-input", value: "{payment_method}", onchange: move |evt| payment_method.set(evt.value()),
                                    option { value: "Efectivo", "Efectivo" }
                                    option { value: "Transferencia", "Transferencia" }
                                    option { value: "Tarjeta", "Tarjeta" }
                                    option { value: "Cheque", "Cheque" }
                                }
                            }
                            div { class: "form-group",
                                label { "Referencia:" }
                                input { class: "form-input", value: "{reference}", oninput: move |evt| reference.set(evt.value()), placeholder: "Nº transferencia, cheque..." }
                            }
                        }
                        div { class: "form-actions",
                            button { class: "btn btn-primary", disabled: saving(), onclick: do_save, if saving() { "Guardando..." } else { "Guardar" } }
                            button { class: "btn", onclick: move |_| reset_form(), "Cancelar" }
                        }
                    }
                }
            } else { rsx! {} }
        }
        div { class: "data-table-container",
            {
                match payments() {
                    Some(Ok(j)) => {
                        let list: Vec<(String, String, String, String, String)> = j["payments"].as_array().map(|arr| {
                            arr.iter().map(|p| {
                                let amt = p["amount"].as_f64().unwrap_or(0.0);
                                let amt_display = format!("${:.0}", amt);
                                (
                                    p["student_id"].as_str().unwrap_or("-").to_string(),
                                    amt_display,
                                    p["payment_date"].as_str().unwrap_or("").to_string(),
                                    p["payment_method"].as_str().unwrap_or("").to_string(),
                                    p["reference"].as_str().unwrap_or("-").to_string(),
                                )
                            }).collect()
                        }).unwrap_or_default();
                        rsx! {
                            table { class: "data-table",
                                thead { tr {
                                    th { "Estudiante" }
                                    th { "Monto" }
                                    th { "Fecha" }
                                    th { "Método" }
                                    th { "Referencia" }
                                }}
                                tbody { for (sid, amt_display, date, method, ref_text) in &list {
                                    tr {
                                        td { "{sid}" }
                                        td { "{amt_display}" }
                                        td { "{date}" }
                                        td { "{method}" }
                                        td { "{ref_text}" }
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
}
