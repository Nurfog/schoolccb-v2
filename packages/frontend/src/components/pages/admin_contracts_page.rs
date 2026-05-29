use dioxus::prelude::*;
use serde_json::json;

use crate::api::client;

#[component]
pub fn AdminContractsPage() -> Element {
    let mut licenses = use_resource(|| client::admin_list_licenses());
    let mut corps = use_resource(|| client::fetch_corporations());
    let mut plans = use_resource(|| client::admin_list_plans());

    let mut show_assign = use_signal(|| false);
    let mut sel_corp = use_signal(String::new);
    let mut sel_plan = use_signal(String::new);
    let sel_status = use_signal(|| "active".to_string());
    let mut saving = use_signal(|| false);

    let mut extend_id = use_signal(|| None::<String>);
    let mut extend_days = use_signal(|| "30".to_string());
    let mut extend_reason = use_signal(String::new);

    let mut change_plan_id = use_signal(|| None::<String>);
    let mut change_plan_to = use_signal(String::new);

    let do_assign = move |_| {
        let cid = sel_corp();
        let pid = sel_plan();
        let st = sel_status();
        if cid.is_empty() || pid.is_empty() { return; }
        saving.set(true);
        spawn(async move {
            let _ = client::admin_assign_license(&json!({
                "corporation_id": cid, "plan_id": pid, "status": st,
            })).await;
            saving.set(false);
            show_assign.set(false);
            sel_corp.set(String::new());
            sel_plan.set(String::new());
            licenses.restart();
            corps.restart();
            plans.restart();
        });
    };

    let do_extend = move |_| {
        let id = extend_id();
        let days = extend_days();
        let reason = extend_reason();
        if id.is_none() || days.is_empty() { return; }
        saving.set(true);
        spawn(async move {
            let _ = client::admin_extend_license(&id.unwrap(), &json!({"days": days.parse::<i32>().unwrap_or(30), "reason": reason})).await;
            saving.set(false);
            extend_id.set(None);
            extend_days.set("30".to_string());
            extend_reason.set(String::new());
            licenses.restart();
            corps.restart();
            plans.restart();
        });
    };

    let do_change_plan = move |_| {
        let id = change_plan_id();
        let pid = change_plan_to();
        if id.is_none() || pid.is_empty() { return; }
        saving.set(true);
        spawn(async move {
            let _ = client::admin_change_plan(&id.unwrap(), &json!({"plan_id": pid})).await;
            saving.set(false);
            change_plan_id.set(None);
            change_plan_to.set(String::new());
            licenses.restart();
            corps.restart();
            plans.restart();
        });
    };

    rsx! {
        div { class: "page-header",
            h1 { "Contratos" }
            p { "Asignaci\u{00f3}n y gesti\u{00f3}n de licencias por corporaci\u{00f3}n" }
        }
        div { class: "page-toolbar",
            button { class: "btn btn-primary", onclick: move |_| show_assign.set(!show_assign()),
                if show_assign() { "Cancelar" } else { "Asignar Licencia" }
            }
        }
        {if show_assign() {
            rsx! {
                div { class: "form-card",
                    div { class: "form-row",
                        div { class: "form-group",
                            label { "Corporaci\u{00f3}n:" }
                            select { class: "form-input", value: "{sel_corp}", oninput: move |e| sel_corp.set(e.value()),
                                option { value: "", "Seleccionar..." }
                                {match corps() {
                                    Some(Ok(data)) => {
                                        let list = data["corporations"].as_array().cloned().unwrap_or_default();
                                        rsx! { {list.into_iter().map(|c| {
                                            let id = c["id"].as_str().unwrap_or("").to_string();
                                            let nm = c["name"].as_str().unwrap_or("").to_string();
                                            rsx! { option { key: "{id}", value: "{id}", "{nm}" } }
                                        })} }
                                    }
                                    _ => rsx! {}
                                }}
                            }
                        }
                        div { class: "form-group",
                            label { "Plan:" }
                            select { class: "form-input", value: "{sel_plan}", oninput: move |e| sel_plan.set(e.value()),
                                option { value: "", "Seleccionar..." }
                                {match plans() {
                                    Some(Ok(data)) => {
                                        let list = data["plans"].as_array().cloned().unwrap_or_default();
                                        rsx! { {list.into_iter().filter(|p| p["active"].as_bool().unwrap_or(false)).map(|p| {
                                            let id = p["id"].as_str().unwrap_or("").to_string();
                                            let nm = p["name"].as_str().unwrap_or("").to_string();
                                            rsx! { option { key: "{id}", value: "{id}", "{nm}" } }
                                        })} }
                                    }
                                    _ => rsx! {}
                                }}
                            }
                        }
                    }
                    div { class: "form-actions",
                        button { class: "btn btn-primary", disabled: saving(), onclick: do_assign,
                            if saving() { "Asignando..." } else { "Asignar Licencia" }
                        }
                    }
                }
            }
        } else { rsx! {} }}
        div { class: "data-table-container",
            match licenses() {
                Some(Ok(data)) => {
                    let list = data["licenses"].as_array().cloned().unwrap_or_default();
                    if list.is_empty() {
                        rsx! { p { class: "empty-state", "No hay licencias asignadas" } }
                    } else {
                        rsx! {
                            table { class: "data-table",
                                thead {
                                    tr {
                                        th { "Corporaci\u{00f3}n" } th { "Plan" } th { "Inicio" } th { "Vencimiento" }
                                        th { "Estado" } th { "D\u{00ed}as rest." } th { "Acciones" }
                                    }
                                }
                                tbody {
                                    {list.into_iter().map(|l| {
                                        let lid = l["id"].as_str().unwrap_or("").to_string();
                                        let corp = l["corporation_name"].as_str().unwrap_or("—").to_string();
                                        let plan = l["plan_name"].as_str().unwrap_or("—").to_string();
                                        let start = l["start_date"].as_str().unwrap_or("—").to_string();
                                        let end = l["end_date"].as_str().unwrap_or("—").to_string();
                                        let st = l["status"].as_str().unwrap_or("").to_string();
                                        let days_left = l["days_remaining"].as_i64().unwrap_or(0);
                                        let badge_cls = match st.as_str() {
                                            "active" => "badge-success",
                                            "expired" | "cancelled" => "badge-danger",
                                            _ => "badge-warning",
                                        };
                                        rsx! {
                                            tr { key: "{lid}",
                                                td { "{corp}" } td { "{plan}" } td { "{start}" } td { "{end}" }
                                                td { span { class: "badge {badge_cls}", "{st}" } }
                                                td { "{days_left}" }
                                                td {
                                                    button { class: "btn btn-sm", onclick: { let lid = lid.clone(); move |_| { extend_id.set(Some(lid.clone())); } }, "Prorrogar" }
                                                    button { class: "btn btn-sm", onclick: { let lid = lid.clone(); move |_| { change_plan_id.set(Some(lid.clone())); } }, "Cambiar Plan" }
                                                }
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
        // Extend modal
        {extend_id().as_ref().map(|_id| {
            rsx! {
                div { class: "modal-overlay", onclick: move |_| extend_id.set(None),
                    div { class: "modal", role: "dialog", "aria-modal": "true", onclick: move |e| e.stop_propagation(),
                        h3 { "Prorrogar Licencia" }
                        div { class: "form-group",
                            label { "D\u{00ed}as a agregar:" }
                            input { class: "form-input", r#type: "number", value: "{extend_days}", oninput: move |e| extend_days.set(e.value()) }
                        }
                        div { class: "form-group",
                            label { "Motivo:" }
                            input { class: "form-input", value: "{extend_reason}", oninput: move |e| extend_reason.set(e.value()), placeholder: "Ej: Renovaci\u{00f3}n anual" }
                        }
                        div { class: "form-actions",
                            button { class: "btn btn-primary", disabled: saving(), onclick: do_extend, "Guardar" }
                            button { class: "btn", onclick: move |_| extend_id.set(None), "Cancelar" }
                        }
                    }
                }
            }
        })}
        // Change plan modal
        {change_plan_id().as_ref().map(|_id| {
            rsx! {
                div { class: "modal-overlay", onclick: move |_| change_plan_id.set(None),
                    div { class: "modal", role: "dialog", "aria-modal": "true", onclick: move |e| e.stop_propagation(),
                        h3 { "Cambiar Plan" }
                        div { class: "form-group",
                            label { "Nuevo plan:" }
                            select { class: "form-input", value: "{change_plan_to}", oninput: move |e| change_plan_to.set(e.value()),
                                option { value: "", "Seleccionar..." }
                                {match plans() {
                                    Some(Ok(data)) => {
                                        let list = data["plans"].as_array().cloned().unwrap_or_default();
                                        rsx! { {list.into_iter().map(|p| {
                                            let pid = p["id"].as_str().unwrap_or("").to_string();
                                            let nm = p["name"].as_str().unwrap_or("").to_string();
                                            rsx! { option { key: "{pid}", value: "{pid}", "{nm}" } }
                                        })} }
                                    }
                                    _ => rsx! {}
                                }}
                            }
                        }
                        div { class: "form-actions",
                            button { class: "btn btn-primary", disabled: saving(), onclick: do_change_plan, "Cambiar" }
                            button { class: "btn", onclick: move |_| change_plan_id.set(None), "Cancelar" }
                        }
                    }
                }
            }
        })}
    }
}
