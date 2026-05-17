use dioxus::prelude::*;

use crate::api::client;

#[component]
pub fn FeesTab() -> Element {
    let mut fees = use_resource(|| client::fetch_all_fees());
    let mut show_form = use_signal(|| false);
    let mut student_search = use_signal(String::new);
    let mut selected_student = use_signal(|| None::<serde_json::Value>);
    let mut description = use_signal(|| String::new());
    let mut amount = use_signal(|| String::new());
    let mut due_date = use_signal(|| String::new());
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
        student_search.set(String::new());
        selected_student.set(None);
        description.set(String::new());
        amount.set(String::new());
        due_date.set(String::new());
        show_form.set(false);
    };

    let do_save = move |_| {
        if selected_student().is_none() || description().is_empty() || amount().is_empty() {
            return;
        }
        saving.set(true);
        let payload = serde_json::json!({
            "student_id": selected_student().unwrap()["id"].as_str().unwrap_or(""),
            "description": description(),
            "amount": amount().parse::<f64>().unwrap_or(0.0),
            "due_date": due_date(),
        });
        spawn(async move {
            let _ = client::create_fee(&payload).await;
            saving.set(false);
            reset_form();
            fees.restart();
        });
    };

    let do_mark_paid = move |fee_id: String| {
        spawn(async move {
            let _ = client::mark_fee_paid(&fee_id).await;
            fees.restart();
        });
    };

    let do_delete = move |fee_id: String| {
        if !web_sys::window().unwrap().confirm_with_message("¿Estás seguro?").unwrap_or(false) {
            return;
        }
        spawn(async move {
            let _ = client::delete_fee(&fee_id).await;
            fees.restart();
        });
    };

    let do_online_payment = move |fee_id: String| {
        spawn(async move {
            match client::init_online_payment(&fee_id).await {
                Ok(data) => {
                    if let Some(url) = data["url"].as_str() {
                        let _ = web_sys::window().and_then(|w| w.location().assign(url).ok());
                    }
                }
                Err(e) => {
                    web_sys::window()
                        .and_then(|w| w.alert_with_message(&format!("Error: {e}")).ok());
                }
            }
        });
    };

    rsx! {
        div { class: "toolbar-row",
            button { class: "btn btn-primary", onclick: move |_| show_form.set(!show_form()), if show_form() { "Cancelar" } else { "Nueva Cuota" } }
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
                                label { "Descripción:" }
                                input { class: "form-input", value: "{description}", oninput: move |evt| description.set(evt.value()), placeholder: "Ej: Matrícula 2025" }
                            }
                            div { class: "form-group",
                                label { "Monto:" }
                                input { class: "form-input", value: "{amount}", oninput: move |evt| amount.set(evt.value()), type: "number", step: "1000", placeholder: "0" }
                            }
                            div { class: "form-group",
                                label { "Vencimiento:" }
                                input { class: "form-input", value: "{due_date}", oninput: move |evt| due_date.set(evt.value()), type: "date" }
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
                match fees() {
                    Some(Ok(j)) => {
                        let list: Vec<(String, String, String, String, String, bool, String)> = j["fees"].as_array().map(|arr| {
                            arr.iter().map(|fee| {
                                let sid = fee["student_id"].as_str().unwrap_or("-").to_string();
                                let desc = fee["description"].as_str().unwrap_or("").to_string();
                                let fid = fee["id"].as_str().unwrap_or("").to_string();
                                let monto_display = {
                                    let m = fee["amount"].as_f64().unwrap_or(0.0);
                                    format!("${:.0}", m)
                                };
                                let due = fee["due_date"].as_str().unwrap_or("").to_string();
                                let paid = fee["paid"].as_bool().unwrap_or(false);
                                let paid_display = if paid {
                                    let amt = fee["paid_amount"].as_f64().unwrap_or(0.0);
                                    format!("${:.0}", amt)
                                } else { "-".to_string() };
                                (fid, sid, desc, monto_display, due, paid, paid_display)
                            }).collect()
                        }).unwrap_or_default();
                        rsx! {
                            table { class: "data-table",
                                thead { tr {
                                    th { "Estudiante" }
                                    th { "Descripción" }
                                    th { "Monto" }
                                    th { "Vencimiento" }
                                    th { "Estado" }
                                    th { "Pagado" }
                                    th { "Acciones" }
                                }}
                                tbody { for (fid, sid, desc, monto_display, due, paid, paid_display) in &list {
                                    tr {
                                        td { "{sid}" }
                                        td { "{desc}" }
                                        td { "{monto_display}" }
                                        td { "{due}" }
                                        td { if *paid { span { class: "grade-good", "Pagado" } } else { span { class: "grade-bad", "Pendiente" } } }
                                        td { "{paid_display}" }
                                        td {
                                            if !paid {
                                                button { class: "btn btn-sm btn-success", onclick: { let id = fid.clone(); move |_| do_mark_paid(id.clone()) }, "Pagar" }
                                                button { class: "btn btn-sm btn-info", style: "margin-left: 4px;", onclick: { let id = fid.clone(); move |_| do_online_payment(id.clone()) }, "Online" }
                                            }
                                            button { class: "btn btn-sm btn-danger", style: "margin-left: 4px;", onclick: { let id = fid.clone(); move |_| do_delete(id.clone()) }, "Eliminar" }
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
}
