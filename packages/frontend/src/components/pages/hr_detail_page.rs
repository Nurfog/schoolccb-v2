use dioxus::prelude::*;
use serde_json::json;
use serde_json::Value;

use crate::api::client;

const AFP_LIST: &[&str] = &["Capital", "Cuprum", "Habitat", "Modelo", "Planvital", "Provida", "Uno"];

fn commission_rate(afp: &str) -> &str {
    match afp {
        "Capital" => "1.44%", "Cuprum" => "1.44%", "Habitat" => "1.27%",
        "Modelo" => "0.58%", "Planvital" => "1.45%", "Provida" => "1.45%",
        "Uno" => "0.69%", _ => "-",
    }
}
const CONTRACT_TYPES: &[&str] = &["Indefinido", "PlazoFijo", "Honorarios", "PrestacionServicios"];

#[component]
pub fn HrDetailPage(employee_id: String) -> Element {
    let eid = employee_id.clone();
    let employee = use_resource(move || {
        let id = eid.clone();
        async move { client::fetch_json(&format!("/api/hr/employees/{}", id)).await }
    });

    let mut tab = use_signal(|| "contratos".to_string());

    rsx! {
        div { class: "page-header",
            h1 { "Ficha del Empleado" }
            p { "Contratos, previsión, asistencia y permisos" }
        }
        { match employee() {
            Some(Ok(data)) => {
                let emp = &data["employee"];
                let name = format!("{} {}",
                    emp["first_name"].as_str().unwrap_or(""),
                    emp["last_name"].as_str().unwrap_or("")
                );
                let category = emp["category"].as_str().unwrap_or("-");
                let position = emp["position"].as_str().unwrap_or("-");
                let vac_days = emp["vacation_days_available"].as_f64().unwrap_or(0.0);
                let rut = emp["rut"].as_str().unwrap_or("");
                let email = emp["email"].as_str().unwrap_or("-");
                let phone = emp["phone"].as_str().unwrap_or("-");
                let hire_date = emp["hire_date"].as_str().unwrap_or("-");
                rsx! {
                    div { class: "employee-summary-card",
                        div { class: "emp-avatar", "{name.chars().next().unwrap_or('?')}" }
                        div { class: "emp-info",
                            h2 { "{name}" }
                            p { "RUT: {rut} | {category} | {position}" }
                            p { "Email: {email} | Tel: {phone} | Ingreso: {hire_date} | Vacaciones: {vac_days} días" }
                        }
                    }
                    div { class: "tabs-container",
                        div { class: "tabs-header",
                            button { class: if tab() == "contratos" { "tab-active" } else { "tab" }, onclick: move |_| tab.set("contratos".to_string()), "Contratos" }
                            button { class: if tab() == "pension" { "tab-active" } else { "tab" }, onclick: move |_| tab.set("pension".to_string()), "Previsión y Salud" }
                            button { class: if tab() == "attendance" { "tab-active" } else { "tab" }, onclick: move |_| tab.set("attendance".to_string()), "Asistencia" }
                            button { class: if tab() == "leave" { "tab-active" } else { "tab" }, onclick: move |_| tab.set("leave".to_string()), "Vacaciones y Permisos" }
                        }
                        div { class: "tab-content",
                            match tab() {
                            s if s == "contratos" => rsx! { ContractsSection { employee_id: employee_id.clone() } },
                            s if s == "pension" => rsx! { PensionSection { employee_id: employee_id.clone() } },
                            s if s == "attendance" => rsx! { AttendanceSection { employee_id: employee_id.clone() } },
                            s if s == "leave" => rsx! { LeaveSection { employee_id: employee_id.clone() } },
                            _ => rsx! {}
                            }
                        }
                    }
                }
            }
            Some(Err(e)) => rsx! { div { class: "empty-state", "Error: {e}" } },
            None => rsx! { div { class: "loading-spinner", "Cargando..." } },
        }}
    }
}

#[component]
fn ContractsSection(employee_id: String) -> Element {
    let mut show_form = use_signal(|| false);
    let mut contract_type = use_signal(|| "Indefinido".to_string());
    let mut salary_base = use_signal(|| "0".to_string());
    let mut weekly_hours = use_signal(|| "40".to_string());
    let mut start_date = use_signal(String::new);
    let mut end_date = use_signal(String::new);
    let mut ley_karin = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut generated = use_signal(|| None::<String>);
    let reload = use_signal(|| 0u32);

    let eid_data = employee_id.clone();
    let data = use_resource(move || {
        let id = eid_data.clone();
        let _r = reload();
        async move { client::fetch_json(&format!("/api/hr/employees/{}", id)).await }
    });

    let eid1 = employee_id.clone();
    let mut reload1 = reload.clone();
    let do_create = move |_| {
        if salary_base().trim().is_empty() || start_date().trim().is_empty() { return; }
        saving.set(true);
        let payload = json!({
            "contract_type": contract_type(),
            "salary_base": salary_base().parse::<f64>().unwrap_or(0.0),
            "weekly_hours": weekly_hours().parse::<i32>().unwrap_or(40),
            "ley_karin_signed": ley_karin(),
            "start_date": start_date(),
            "end_date": if end_date().trim().is_empty() { Value::Null } else { Value::String(end_date()) },
        });
        let eid = eid1.clone();
        spawn(async move {
            let _ = client::post_json(&format!("/api/hr/employees/{}/contracts", eid), &payload).await;
            saving.set(false);
            show_form.set(false);
            contract_type.set("Indefinido".to_string());
            salary_base.set("0".to_string());
            weekly_hours.set("40".to_string());
            ley_karin.set(false);
            start_date.set(String::new());
            end_date.set(String::new());
            reload1.set(reload1() + 1);
        });
    };

    let eid2 = employee_id.clone();
    let do_generate = move |ct: String, sb: f64, wh: i32, sd: String, lk: bool| {
        let eid = eid2.clone();
        spawn(async move {
            let payload = json!({
                "employee_id": eid,
                "contract_type": ct,
                "salary_base": sb,
                "weekly_hours": wh,
                "start_date": sd,
                "ley_karin_signed": lk,
            });
            let result = client::post_json("/api/hr/contract/generate", &payload).await;
            match result {
                Ok(resp) => { generated.set(resp["contract_text"].as_str().map(|s| s.to_string())); }
                Err(e) => { generated.set(Some(format!("Error: {}", e))); }
            }
        });
    };

    rsx! {
        div { class: "page-toolbar",
            h3 { style: "margin: 0; font-size: 14px; font-weight: 600;", "Contratos del Empleado" }
            button { class: "btn btn-primary", onclick: move |_| show_form.set(!show_form()),
                if show_form() { "Cancelar" } else { "Nuevo Contrato" }
            }
        }
        { if show_form() {
            rsx! {
                div { class: "form-card",
                    div { class: "form-section",
                        div { class: "form-section-title", "Datos del Contrato" }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Tipo de Contrato" }
                                select { class: "form-input", value: "{contract_type}", onchange: move |e| contract_type.set(e.value()),
                                    {CONTRACT_TYPES.iter().map(|ct| rsx! { option { value: "{ct}", "{ct}" } })}
                                }
                            }
                            div { class: "form-group",
                                label { "Sueldo Base $" }
                                input { class: "form-input", value: "{salary_base}", oninput: move |e| salary_base.set(e.value()), r#type: "number", min: "0" }
                            }
                            div { class: "form-group",
                                label { "Horas Semanales" }
                                input { class: "form-input", value: "{weekly_hours}", oninput: move |e| weekly_hours.set(e.value()), r#type: "number", min: "1", max: "45" }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Fecha Inicio" }
                                input { class: "form-input", value: "{start_date}", oninput: move |e| start_date.set(e.value()), r#type: "date" }
                            }
                            div { class: "form-group",
                                label { "Fecha Término (solo plazo fijo)" }
                                input { class: "form-input", value: "{end_date}", oninput: move |e| end_date.set(e.value()), r#type: "date" }
                            }
                        }
                        div { class: "form-row",
                            label { class: "checkbox-label",
                                input { r#type: "checkbox", checked: ley_karin, oninput: move |_| ley_karin.set(!ley_karin()) }
                                " Firmar declaración Ley Karin (Ley 21.643)"
                            }
                        }
                    }
                    div { class: "form-actions",
                        button { class: "btn btn-primary", disabled: saving() || salary_base().trim().is_empty() || start_date().trim().is_empty(), onclick: do_create,
                            if saving() { "Guardando..." } else { "Crear Contrato" }
                        }
                    }
                }
            }
        } else { rsx! {} }}
        { if let Some(ref text) = generated() {
            rsx! {
                div { class: "form-card", style: "margin-top: 12px;",
                    div { class: "form-card-header",
                        h3 { "Vista Previa del Contrato" }
                        button { class: "btn btn-sm btn-secondary", onclick: move |_| generated.set(None), "Cerrar" }
                    }
                    pre { style: "white-space: pre-wrap; font-family: var(--font); font-size: 13px; line-height: 1.6; color: var(--text-primary); padding: 16px; background: var(--bg); border-radius: var(--radius-sm); max-height: 400px; overflow-y: auto;",
                        "{text}"
                    }
                }
            }
        } else { rsx! {} }}
        div { class: "data-table-container", style: "margin-top: 12px;",
            match data() {
                Some(Ok(json)) => {
                    let contracts = json["contracts"].as_array().cloned().unwrap_or_default();
                    if contracts.is_empty() {
                        rsx! { div { class: "empty-state", "Sin contratos registrados" } }
                    } else {
                        let dg = do_generate;
                        let rows: Vec<Element> = contracts.iter().map(|c| {
                            let ct = c["contract_type"].as_str().unwrap_or("").to_string();
                            let sal = c["salary_base"].as_f64().unwrap_or(0.0);
                            let hrs = c["weekly_hours"].as_i64().unwrap_or(0);
                            let start = c["start_date"].as_str().unwrap_or("").to_string();
                            let end = c["end_date"].as_str().unwrap_or("-").to_string();
                            let karin = c["ley_karin_signed"].as_bool().unwrap_or(false);
                            let active = c["active"].as_bool().unwrap_or(false);
                            let dg = dg.clone();
                            rsx! {
                                tr {
                                    td { "{ct}" }
                                    td { "${sal:.0}" }
                                    td { "{hrs}" }
                                    td { "{start}" }
                                    td { "{end}" }
                                    td {
                                        if karin { span { class: "status-active", "Firmado" } }
                                        else { span { class: "status-pending", "Pendiente" } }
                                    }
                                    td {
                                        if active { span { class: "status-active", "Activo" } }
                                        else { span { class: "status-inactive", "Inactivo" } }
                                    }
                                    td {
                                        button { class: "btn btn-sm btn-secondary", onclick: move |_| dg(ct.clone(), sal, hrs as i32, start.clone(), karin), "Ver Contrato" }
                                    }
                                }
                            }
                        }).collect();
                        rsx! {
                            table { class: "data-table",
                                thead { tr {
                                    th { "Tipo" }
                                    th { "Salario Base" }
                                    th { "Horas" }
                                    th { "Inicio" }
                                    th { "Término" }
                                    th { "Ley Karin" }
                                    th { "Estado" }
                                    th { "Acción" }
                                }}
                                tbody { { rows.into_iter() } }
                            }
                        }
                    }
                }
                _ => rsx! { div { class: "empty-state", "Cargando contratos..." } },
            }
        }
    }
}

#[component]
fn PensionSection(employee_id: String) -> Element {
    let eid = employee_id.clone();
    let pf_data = use_resource(move || {
        let id = eid.clone();
        async move { client::fetch_json(&format!("/api/hr/employees/{}/pension-fund", id)).await }
    });

    let mut pension_fund = use_signal(|| "Provida".to_string());
    let mut health_system = use_signal(|| "Fonasa".to_string());
    let mut health_plan = use_signal(String::new);
    let mut health_amount = use_signal(|| "0".to_string());
    let mut saving = use_signal(|| false);
    let mut saved = use_signal(|| false);

    if let Some(Ok(ref data)) = pf_data() {
        let pf = &data["pension_fund"];
        if pension_fund() == "Provida" && pf["pension_fund"].as_str().is_some() {
            pension_fund.set(pf["pension_fund"].as_str().unwrap_or("Provida").to_string());
            health_system.set(pf["health_system"].as_str().unwrap_or("Fonasa").to_string());
            health_plan.set(pf["health_plan_name"].as_str().unwrap_or("").to_string());
            health_amount.set(pf["health_fixed_amount"].as_f64().unwrap_or(0.0).to_string());
        }
    }

    let do_save = move |_| {
        saving.set(true);
        let payload = json!({
            "pension_fund": pension_fund(),
            "health_system": health_system(),
            "health_plan_name": if health_system() == "Isapre" { health_plan() } else { String::new() },
            "health_fixed_amount": if health_system() == "Isapre" { health_amount().parse::<f64>().unwrap_or(0.0) } else { 0.0 },
        });
        let eid = employee_id.clone();
        spawn(async move {
            let _ = client::post_json(&format!("/api/hr/employees/{}/pension-fund", eid), &payload).await;
            saving.set(false);
            saved.set(true);
        });
    };

    rsx! {
        div { class: "form-card",
            div { class: "form-section",
                div { class: "form-section-title", "AFP (Administradora de Fondos de Pensiones)" }
                div { class: "form-row",
                    div { class: "form-group",
                        label { "AFP" }
                        select { class: "form-input", value: "{pension_fund}", onchange: move |e| pension_fund.set(e.value()),
                            {AFP_LIST.iter().map(|afp| rsx! { option { value: "{afp}", "{afp}" } })}
                        }
                    }
                    div { class: "form-group",
                        label { style: "font-size: 11px; color: var(--text-muted);", "Comisión actual" }
                        input { class: "form-input", value: "{commission_rate(&pension_fund())}", disabled: true }
                    }
                }
            }
            div { class: "form-section",
                div { class: "form-section-title", "Sistema de Salud" }
                div { class: "form-row",
                    div { class: "form-group",
                        label { "Sistema" }
                        select { class: "form-input", value: "{health_system}", onchange: move |e| {
                            let v = e.value();
                            health_system.set(v.clone());
                            if v == "Fonasa" { health_amount.set("0".to_string()); }
                        },
                            option { value: "Fonasa", "Fonasa (7%)" }
                            option { value: "Isapre", "Isapre" }
                        }
                    }
                    { if health_system() == "Isapre" {
                        rsx! {
                            div { class: "form-group",
                                label { "Plan" }
                                input { class: "form-input", value: "{health_plan}", oninput: move |e| health_plan.set(e.value()), placeholder: "Nombre del plan" }
                            }
                            div { class: "form-group",
                                label { "Monto Fijo (UF / $)" }
                                input { class: "form-input", value: "{health_amount}", oninput: move |e| health_amount.set(e.value()), r#type: "number", min: "0" }
                            }
                        }
                    } else { rsx! {
                        div { class: "form-group",
                            label { "Descuento" }
                            input { class: "form-input", value: "7% del sueldo imponible", disabled: true }
                        }
                    }}}
                }
            }
            div { class: "form-actions",
                button { class: "btn btn-primary", disabled: saving(), onclick: do_save,
                    if saving() { "Guardando..." } else if saved() { "¡Guardado!" } else { "Guardar Configuración" }
                }
            }
        }
    }
}

#[component]
fn AttendanceSection(employee_id: String) -> Element {
    let eid = employee_id.clone();
    let mut show_sync = use_signal(|| false);
    let mut sync_timestamp = use_signal(String::new);
    let mut sync_entry_type = use_signal(|| "Entrada".to_string());
    let mut syncing = use_signal(|| false);
    let mut attendance = use_resource(move || {
        let id = eid.clone();
        async move { client::fetch_json(&format!("/api/hr/employees/{}/attendance", id)).await }
    });

    let do_sync = move |_| {
        if sync_timestamp().trim().is_empty() { return; }
        syncing.set(true);
        let payload = serde_json::json!({
            "employee_id": employee_id.clone(),
            "timestamp": sync_timestamp(),
            "entry_type": sync_entry_type(),
        });
        spawn(async move {
            let _ = client::post_json("/api/hr/attendance/sync", &payload).await;
            syncing.set(false);
            show_sync.set(false);
            sync_timestamp.set(String::new());
            attendance.restart();
        });
    };

    rsx! {
        div { class: "page-toolbar",
            h3 { style: "margin: 0; font-size: 14px; font-weight: 600;", "Registro de Asistencia" }
            button { class: "btn btn-primary", onclick: move |_| show_sync.set(!show_sync()),
                if show_sync() { "Cancelar" } else { "Registrar Marcación" }
            }
        }
        { if show_sync() {
            rsx! {
                div { class: "form-card",
                    div { class: "form-row",
                        div { class: "form-group",
                            label { "Fecha y Hora" }
                            input { class: "form-input", value: "{sync_timestamp}", oninput: move |e| sync_timestamp.set(e.value()), placeholder: "2026-05-11T09:00:00", r#type: "datetime-local" }
                        }
                        div { class: "form-group",
                            label { "Tipo" }
                            select { class: "form-input", value: "{sync_entry_type}", onchange: move |e| sync_entry_type.set(e.value()),
                                option { value: "Entrada", "Entrada" }
                                option { value: "Salida Colacion", "Salida Colación" }
                                option { value: "Retorno Colacion", "Retorno Colación" }
                                option { value: "Salida", "Salida" }
                            }
                        }
                    }
                    div { class: "form-actions",
                        button { class: "btn btn-primary", disabled: syncing(), onclick: do_sync,
                            if syncing() { "Registrando..." } else { "Registrar" }
                        }
                    }
                }
            }
        } else { rsx! {} }}
        div { class: "data-table-container",
            match attendance() {
                Some(Ok(json)) => {
                    let logs = json["attendance_logs"].as_array().cloned().unwrap_or_default();
                    if logs.is_empty() {
                        rsx! { div { class: "empty-state", "Sin registros de asistencia" } }
                    } else {
                        let rows: Vec<Element> = logs.iter().map(|l| {
                            let ts = l["timestamp"].as_str().unwrap_or("").to_string();
                            let etype = l["entry_type"].as_str().unwrap_or("").to_string();
                            let dev = l["device_id"].as_str().unwrap_or("-").to_string();
                            let src = l["source"].as_str().unwrap_or("-").to_string();
                            rsx! {
                                tr {
                                    td { "{ts}" }
                                    td { span { class: "role-badge", "{etype}" } }
                                    td { "{dev}" }
                                    td { "{src}" }
                                }
                            }
                        }).collect();
                        rsx! {
                            table { class: "data-table",
                                thead { tr {
                                    th { "Fecha/Hora" }
                                    th { "Tipo" }
                                    th { "Dispositivo" }
                                    th { "Origen" }
                                }}
                                tbody { { rows.into_iter() } }
                            }
                        }
                    }
                }
                _ => rsx! { div { class: "empty-state", "Cargando asistencia..." } },
            }
        }
    }
}

#[component]
fn LeaveSection(employee_id: String) -> Element {
    let mut show_request = use_signal(|| false);
    let mut leave_type = use_signal(|| "Vacaciones".to_string());
    let mut start_date = use_signal(String::new);
    let mut end_date = use_signal(String::new);
    let mut reason = use_signal(String::new);
    let mut saving = use_signal(|| false);
    let eid = employee_id.clone();
    let mut leave_requests = use_resource(move || {
        let id = eid.clone();
        async move { client::fetch_json(&format!("/api/hr/employees/{}/leave-requests", id)).await }
    });

    let do_request = move |_| {
        if start_date().trim().is_empty() || end_date().trim().is_empty() { return; }
        saving.set(true);
        let payload = serde_json::json!({
            "employee_id": employee_id.clone(),
            "leave_type": leave_type(),
            "start_date": start_date(),
            "end_date": end_date(),
            "reason": reason(),
        });
        let eid2 = employee_id.clone();
        spawn(async move {
            let _ = client::post_json(&format!("/api/hr/employees/{}/leave-requests", eid2), &payload).await;
            saving.set(false);
            show_request.set(false);
            start_date.set(String::new());
            end_date.set(String::new());
            reason.set(String::new());
            leave_requests.restart();
        });
    };

    rsx! {
        div { class: "page-toolbar",
            h3 { style: "margin: 0; font-size: 14px; font-weight: 600;", "Vacaciones y Permisos" }
            button { class: "btn btn-primary", onclick: move |_| show_request.set(!show_request()),
                if show_request() { "Cancelar" } else { "Nueva Solicitud" }
            }
        }
        { if show_request() {
            rsx! {
                div { class: "form-card",
                    div { class: "form-row",
                        div { class: "form-group",
                            label { "Tipo" }
                            select { class: "form-input", value: "{leave_type}", onchange: move |e| leave_type.set(e.value()),
                                option { value: "Vacaciones", "Vacaciones" }
                                option { value: "Licencia Medica", "Licencia Médica" }
                                option { value: "Permiso Personal", "Permiso Personal" }
                                option { value: "Capacitacion", "Capacitación" }
                                option { value: "Otro", "Otro" }
                            }
                        }
                        div { class: "form-group",
                            label { "Fecha Inicio" }
                            input { class: "form-input", value: "{start_date}", oninput: move |e| start_date.set(e.value()), r#type: "date" }
                        }
                        div { class: "form-group",
                            label { "Fecha Término" }
                            input { class: "form-input", value: "{end_date}", oninput: move |e| end_date.set(e.value()), r#type: "date" }
                        }
                    }
                    div { class: "form-row",
                        div { class: "form-group",
                            label { "Motivo" }
                            input { class: "form-input", value: "{reason}", oninput: move |e| reason.set(e.value()), placeholder: "Opcional" }
                        }
                    }
                    div { class: "form-actions",
                        button { class: "btn btn-primary", disabled: saving(), onclick: do_request,
                            if saving() { "Enviando..." } else { "Solicitar" }
                        }
                    }
                }
            }
        } else { rsx! {} }}
        div { class: "data-table-container",
            match leave_requests() {
                Some(Ok(json)) => {
                    let requests = json["leave_requests"].as_array().cloned().unwrap_or_default();
                    if requests.is_empty() {
                        rsx! { div { class: "empty-state", "Sin solicitudes" } }
                    } else {
                        let rows: Vec<Element> = requests.iter().map(|r| {
                            let lt = r["leave_type"].as_str().unwrap_or("").to_string();
                            let sd = r["start_date"].as_str().unwrap_or("").to_string();
                            let ed = r["end_date"].as_str().unwrap_or("").to_string();
                            let status = r["status"].as_str().unwrap_or("Pendiente").to_string();
                            let approved = r["approved_by"].as_str().map(|s| if s.is_empty() { "-" } else { s }).unwrap_or("-").to_string();
                            rsx! {
                                tr {
                                    td { "{lt}" }
                                    td { "{sd}" }
                                    td { "{ed}" }
                                    td { "-" }
                                    td {
                                        if status == "Aprobado" { span { class: "status-active", "Aprobado" } }
                                        else if status == "Rechazado" { span { class: "status-inactive", "Rechazado" } }
                                        else { span { class: "status-pending", "Pendiente" } }
                                    }
                                    td { "{approved}" }
                                }
                            }
                        }).collect();
                        rsx! {
                            table { class: "data-table",
                                thead { tr {
                                    th { "Tipo" }
                                    th { "Inicio" }
                                    th { "Término" }
                                    th { "Días" }
                                    th { "Estado" }
                                    th { "Aprobado por" }
                                }}
                                tbody { { rows.into_iter() } }
                            }
                        }
                    }
                }
                _ => rsx! { div { class: "empty-state", "Cargando solicitudes..." } },
            }
        }
    }
}
