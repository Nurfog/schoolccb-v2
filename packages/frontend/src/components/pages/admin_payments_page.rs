use dioxus::prelude::*;
use serde_json::json;

use crate::api::client;

#[component]
pub fn AdminPaymentsPage() -> Element {
    let mut payments = use_resource(|| client::admin_list_payments());
    let mut show_form = use_signal(|| false);
    let mut amount = use_signal(|| "".to_string());
    let mut method = use_signal(|| "transfer".to_string());
    let mut notes = use_signal(String::new);
    let mut saving = use_signal(|| false);

    let do_register = move |_| {
        let a = amount();
        let m = method();
        let n = notes();
        if a.is_empty() { return; }
        saving.set(true);
        spawn(async move {
            let _ = client::admin_register_payment(&json!({
                "amount": a.parse::<f64>().unwrap_or(0.0),
                "payment_method": m,
                "notes": n,
            })).await;
            saving.set(false);
            show_form.set(false);
            amount.set("".to_string());
            method.set("transfer".to_string());
            notes.set(String::new());
            payments.restart();
        });
    };

    rsx! {
        div { class: "page-header",
            h1 { "Pagos" }
            p { "Historial y registro de pagos de licencias" }
        }
        div { class: "page-toolbar",
            button { class: "btn btn-primary", onclick: move |_| show_form.set(!show_form()),
                if show_form() { "Cancelar" } else { "Registrar Pago" }
            }
        }
        {if show_form() {
            rsx! {
                div { class: "form-card",
                    div { class: "form-row",
                        div { class: "form-group",
                            label { "Monto:" }
                            input { class: "form-input", r#type: "number", step: "any", value: "{amount}", oninput: move |e| amount.set(e.value()), placeholder: "Ej: 150000" }
                        }
                        div { class: "form-group",
                            label { "M\u{00e9}todo:" }
                            select { class: "form-input", value: "{method}", oninput: move |e| method.set(e.value()),
                                option { value: "transfer", "Transferencia" }
                                option { value: "webpay", "Webpay" }
                                option { value: "cash", "Efectivo" }
                                option { value: "invoice", "Factura" }
                            }
                        }
                    }
                    div { class: "form-group",
                        label { "Notas:" }
                        input { class: "form-input", value: "{notes}", oninput: move |e| notes.set(e.value()), placeholder: "Opcional" }
                    }
                    div { class: "form-actions",
                        button { class: "btn btn-primary", disabled: saving(), onclick: do_register,
                            if saving() { "Registrando..." } else { "Registrar Pago" }
                        }
                    }
                }
            }
        } else { rsx! {} }}
        div { class: "data-table-container",
            match payments() {
                Some(Ok(data)) => {
                    let list = data["payments"].as_array().cloned().unwrap_or_default();
                    if list.is_empty() {
                        rsx! { p { class: "empty-state", "No hay pagos registrados" } }
                    } else {
                        rsx! {
                            table { class: "data-table",
                                thead {
                                    tr {
                                        th { "Corporaci\u{00f3}n" } th { "Monto" } th { "M\u{00e9}todo" }
                                        th { "Estado" } th { "Fecha" } th { "Notas" }
                                    }
                                }
                                tbody {
                                    {list.into_iter().map(|p| {
                                        let corp = p["corporation_name"].as_str().unwrap_or("—").to_string();
                                        let amt = p["amount"].as_f64().unwrap_or(0.0);
                                        let mtd = p["payment_method"].as_str().unwrap_or("—").to_string();
                                        let st = p["status"].as_str().unwrap_or("—").to_string();
                                        let date = p["paid_at"].as_str().unwrap_or(p["created_at"].as_str().unwrap_or("—")).to_string();
                                        let nts = p["notes"].as_str().unwrap_or("").to_string();
                                        rsx! {
                                            tr {
                                                td { "{corp}" } td { "${amt:.0}" } td { "{mtd}" }
                                                td { span { class: "badge badge-success", "{st}" } }
                                                td { "{date}" } td { "{nts}" }
                                            }
                                        }
                                    })}
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! { p { class: "state-error", "Error: {e}" } },
                None => rsx! { div { class: "loading-spinner", "Cargando..." } },
            }
        }
    }
}
