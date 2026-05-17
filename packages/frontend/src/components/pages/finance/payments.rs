use dioxus::prelude::*;

use crate::api::client;

#[component]
pub fn PaymentsTab() -> Element {
    let mut payments = use_resource(|| client::fetch_all_payments());
    let mut show_form = use_signal(|| false);
    let mut fee_id = use_signal(|| String::new());
    let mut student_search = use_signal(String::new);
    let mut selected_student = use_signal(|| None::<serde_json::Value>);
    let mut amount = use_signal(|| String::new());
    let mut payment_method = use_signal(|| "Efectivo".to_string());
    let mut reference = use_signal(|| String::new());
    let mut saving = use_signal(|| false);
    let search_results = use_resource(move || {
        let q = student_search();
        async move {
            if q.len() < 2 {
                Ok(serde_json::json!({"students": []}))
            } else {
                client::search_students(&q).await
            }
        }
    });

    let mut reset_form = move || {
        fee_id.set(String::new());
        student_search.set(String::new());
        selected_student.set(None);
        amount.set(String::new());
        payment_method.set("Efectivo".to_string());
        reference.set(String::new());
        show_form.set(false);
    };

    let do_save = move |_| {
        if selected_student().is_none() || amount().is_empty() {
            return;
        }
        saving.set(true);
        let payload = serde_json::json!({
            "fee_id": fee_id(),
            "student_id": selected_student().unwrap()["id"].as_str().unwrap_or(""),
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
                            div { class: "student-selector", style: "width: 100%;",
                                label { "Estudiante:" }
                                {
                                    match selected_student() {
                                        Some(ref s) => {
                                            let sname = format!("{} {}",
                                                s["first_name"].as_str().unwrap_or(""),
                                                s["last_name"].as_str().unwrap_or("")
                                            );
                                            rsx! {
                                                div { class: "selected-student",
                                                    span { "{sname}" }
                                                    button { class: "btn-icon", "aria-label": "Cerrar", onclick: move |_| selected_student.set(None), "✕" }
                                                }
                                            }
                                        }
                                        None => rsx! {
                                            input { class: "search-input", value: "{student_search}", oninput: move |evt| student_search.set(evt.value()), placeholder: "Buscar estudiante..." }
                                        }
                                    }
                                }
                                {
                                    match search_results() {
                                        Some(Ok(j)) => {
                                            let list = j["students"].as_array().cloned().unwrap_or_default();
                                            if !list.is_empty() && student_search().len() >= 2 && selected_student().is_none() {
                                                rsx! { div { class: "search-results",
                                                    for s in &list {
                                                        let sid = s["id"].as_str().unwrap_or("").to_string();
                                                        let sname = format!("{} {}",
                                                            s["first_name"].as_str().unwrap_or(""),
                                                            s["last_name"].as_str().unwrap_or("")
                                                        );
                                                        rsx! {
                                                            div {
                                                                class: "search-result-item",
                                                                onclick: move |_| {
                                                                    selected_student.set(Some(serde_json::json!({"id": sid.clone(), "first_name": sname.clone()})));
                                                                    student_search.set(String::new());
                                                                },
                                                                span { "{sname}" }
                                                            }
                                                        }
                                                    }
                                                } }
                                            } else { rsx! {} }
                                        }
                                        _ => rsx! {},
                                    }
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
                                input { class: "form-input", value: "{amount}", oninput: move |evt| amount.set(evt.value()), type: "number", step: "1000", placeholder: "0" }
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
