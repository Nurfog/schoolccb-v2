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
    let mut contracts = use_resource(|| client::list_enrollment_contracts());
    let mut scholarships = use_resource(|| client::list_scholarships());
    let mut show_new_scholarship = use_signal(|| false);
    let mut new_scholarship_name = use_signal(String::new);
    let mut new_scholarship_discount = use_signal(|| "10".to_string());
    let mut new_scholarship_max = use_signal(|| "10".to_string());
    let mut saving_scholarship = use_signal(|| false);
    let mut active_tab = use_signal(|| "pipeline".to_string());
    let mut enrolling_id = use_signal(|| None::<String>);
    let mut enroll_msg = use_signal(|| None::<String>);
    let mut enroll_success = use_signal(|| false);
    let mut show_apply_scholarship = use_signal(|| None::<String>);
    let mut apply_student_id = use_signal(String::new);
    let mut apply_msg = use_signal(|| None::<String>);
    let mut show_new_contract = use_signal(|| false);
    let mut new_contract_student = use_signal(String::new);
    let mut new_contract_grade = use_signal(String::new);
    let mut new_contract_amount = use_signal(|| "0".to_string());
    let mut saving_contract = use_signal(|| false);
    let mut view_contract_id = use_signal(|| None::<String>);
    let contract_detail = use_resource(move || {
        let vid = view_contract_id();
        async move {
            match vid {
                Some(id) => client::get_enrollment_contract(&id).await,
                None => Err("none".to_string()),
            }
        }
    });

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

    let do_create_scholarship = move |_| {
        saving_scholarship.set(true);
        let payload = serde_json::json!({
            "name": new_scholarship_name(),
            "discount": new_scholarship_discount().parse::<f64>().unwrap_or(0.0),
            "max": new_scholarship_max().parse::<i64>().unwrap_or(10),
        });
        spawn(async move {
            let _ = client::create_admission_scholarship(&payload).await;
            saving_scholarship.set(false);
            show_new_scholarship.set(false);
            new_scholarship_name.set(String::new());
            new_scholarship_discount.set("10".to_string());
            new_scholarship_max.set("10".to_string());
            scholarships.restart();
        });
    };

    let do_toggle_scholarship = move |id: String| {
        spawn(async move {
            let _ = client::toggle_scholarship(&id).await;
            scholarships.restart();
        });
    };

    let do_apply_scholarship = move |_| {
        let sid = apply_student_id();
        if sid.is_empty() { return; }
        let s_id = show_apply_scholarship().unwrap_or_default();
        spawn(async move {
            match client::apply_scholarship(&s_id, &sid).await {
                Ok(resp) => apply_msg.set(Some(resp["message"].as_str().unwrap_or("Aplicada").to_string())),
                Err(e) => apply_msg.set(Some(format!("Error: {e}"))),
            }
        });
    };

    let do_create_contract = move |_| {
        saving_contract.set(true);
        let payload = serde_json::json!({
            "student": new_contract_student(),
            "grade": new_contract_grade(),
            "amount": new_contract_amount().parse::<f64>().unwrap_or(0.0),
        });
        spawn(async move {
            let _ = client::create_enrollment_contract(&payload).await;
            saving_contract.set(false);
            show_new_contract.set(false);
            new_contract_student.set(String::new());
            new_contract_grade.set(String::new());
            new_contract_amount.set("0".to_string());
            contracts.restart();
        });
    };

    rsx! {
        div { class: "page-header",
            h1 { "Admisiones" }
            p { "Pipeline de postulantes - gestione el ciclo de admisión" }
        }
        div { class: "page-toolbar",
            button { class: "btn btn-outline", onclick: move |_| active_tab.set("pipeline".to_string()),
                if active_tab() == "pipeline" { "📋 Pipeline" } else { "Pipeline" }
            }
            button { class: "btn btn-outline", onclick: move |_| active_tab.set("contracts".to_string()),
                if active_tab() == "contracts" { "📝 Contratos" } else { "Contratos" }
            }
            button { class: "btn btn-outline", onclick: move |_| active_tab.set("scholarships".to_string()),
                if active_tab() == "scholarships" { "🎓 Becas" } else { "Becas" }
            }
        }
        {
            let tab = active_tab();
            if tab == "pipeline" {
                rsx! {
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
            } else if tab == "contracts" {
                rsx! {
                    div { class: "page-toolbar",
                        h3 { "Contratos de Matrícula" }
                        button { class: "btn btn-primary", onclick: move |_| show_new_contract.set(!show_new_contract()), if show_new_contract() { "Cancelar" } else { "Nuevo Contrato" } }
                    }
                    {
                        if show_new_contract() {
                            rsx! {
                                div { class: "form-card",
                                    div { class: "form-row",
                                        div { class: "form-group",
                                            label { "Estudiante:" }
                                            input { class: "form-input", value: "{new_contract_student}", oninput: move |e| new_contract_student.set(e.value()), placeholder: "Nombre del estudiante" }
                                        }
                                        div { class: "form-group",
                                            label { "Nivel:" }
                                            select { class: "form-input", value: "{new_contract_grade}", oninput: move |e| new_contract_grade.set(e.value()),
                                                option { value: "", "Seleccionar..." }
                                                option { value: "Pre-Kínder", "Pre-Kínder" }
                                                option { value: "Kínder", "Kínder" }
                                                option { value: "1° Básico", "1° Básico" }
                                                option { value: "2° Básico", "2° Básico" }
                                                option { value: "3° Básico", "3° Básico" }
                                                option { value: "4° Básico", "4° Básico" }
                                                option { value: "5° Básico", "5° Básico" }
                                                option { value: "6° Básico", "6° Básico" }
                                                option { value: "7° Básico", "7° Básico" }
                                                option { value: "8° Básico", "8° Básico" }
                                                option { value: "1° Medio", "1° Medio" }
                                                option { value: "2° Medio", "2° Medio" }
                                                option { value: "3° Medio", "3° Medio" }
                                                option { value: "4° Medio", "4° Medio" }
                                            }
                                        }
                                    }
                                    div { class: "form-group",
                                        label { "Monto:" }
                                        input { class: "form-input", value: "{new_contract_amount}", oninput: move |e| new_contract_amount.set(e.value()), type: "number", min: "0" }
                                    }
                                    div { class: "form-actions",
                                        button { class: "btn btn-primary", disabled: saving_contract(), onclick: do_create_contract, if saving_contract() { "Guardando..." } else { "Crear Contrato" } }
                                        button { class: "btn", onclick: move |_| show_new_contract.set(false), "Cancelar" }
                                    }
                                }
                            }
                        } else { rsx! {} }
                    }
                    if enroll_success() {
                        div { class: "success-card",
                            h2 { "✅ Matrícula Confirmada" }
                            p { "El alumno ha sido matriculado exitosamente." }
                            p { "Se ha generado el registro oficial en el sistema." }
                            button { class: "btn btn-primary", onclick: move |_| { enroll_success.set(false); enroll_msg.set(None); }, "Volver" }
                        }
                    } else {
                        match contracts() {
                            Some(Ok(data)) => {
                                let list = data["contracts"].as_array().cloned().unwrap_or_default();
                                let contract_rows: Vec<Element> = list.iter().map(|c| {
                                    let cid = c["id"].as_str().unwrap_or("").to_string();
                                    let cid2 = cid.clone();
                                    let is_draft = c["status"].as_str() == Some("draft");
                                    let cstudent = c["student"].as_str().unwrap_or("-").to_string();
                                    let cgrade = c["grade"].as_str().unwrap_or("-").to_string();
                                    let camt = c["amount"].as_f64().unwrap_or(0.0);
                                    let camt_str = format!("${:.0}", camt);
                                    let cstatus = c["status"].as_str().unwrap_or("-").to_string();
                                    let cdate = c["date"].as_str().unwrap_or("-").to_string();
                                    rsx! {
                                        tr {
                                            td { "{cstudent}" }
                                            td { "{cgrade}" }
                                            td { "{camt_str}" }
                                            td { "{cstatus}" }
                                            td { "{cdate}" }
                                            td {
                                                if is_draft {
                                                    button {
                                                        class: "btn btn-primary btn-small",
                                                        style: "margin-right: 4px;",
                                                        onclick: { let cid_enroll = cid.clone(); move |_| enrolling_id.set(Some(cid_enroll.clone())) },
                                                        "Matricular"
                                                    }
                                                    button {
                                                        class: "btn btn-outline btn-small",
                                                        style: "margin-right: 4px;",
                                                        onclick: move |_| {
                                                            let cid3 = cid2.clone();
                                                            spawn(async move {
                                                                let _ = client::pay_contract(&cid3, camt, "Efectivo").await;
                                                                contracts.restart();
                                                            });
                                                        },
                                                        "Pagar"
                                                    }
                                                } else {
                                                    span { class: "badge badge-success", "Completado" }
                                                }
                                                button {
                                                    class: "btn btn-sm",
                                                    onclick: { let cid4 = cid.clone(); move |_| view_contract_id.set(Some(cid4.clone())) },
                                                    "Ver"
                                                }
                                            }
                                        }
                                    }
                                }).collect();
                                let list_empty = list.is_empty();
                                rsx! {
                                    div { class: "data-table-container",
                                        table { class: "data-table",
                                            thead { tr {
                                                th { "Estudiante" }
                                                th { "Nivel" }
                                                th { "Monto Final" }
                                                th { "Estado" }
                                                th { "Creado" }
                                                th { "Acción" }
                                            }}
                                            tbody { {contract_rows.into_iter()} }
                                        }
                                        if list_empty {
                                            div { class: "empty-state", "Sin contratos de matrícula" }
                                        }
                                    }
                                    {
                                        let show_enroll = enrolling_id();
                                        let enroll_modal = show_enroll.as_ref().map(|eid| {
                                            let cid = eid.clone();
                                            let msg = enroll_msg();
                                            rsx! {
                                                div { class: "modal-overlay", role: "dialog", onclick: move |_| enrolling_id.set(None),
                                                    div { class: "modal-content", onclick: move |e| e.stop_propagation(),
                                                        div { class: "modal-header",
                                                            h2 { "Confirmar Matrícula" }
                                                            button { class: "btn-icon", onclick: move |_| enrolling_id.set(None), "✕" }
                                                        }
                                                        div { class: "modal-body",
                                                            p { "¿Estás seguro de matricular a este alumno?" }
                                                            p { "Se activará su condición de alumno regular en el sistema." }
                                                            if let Some(ref m) = msg {
                                                                div { class: "alert alert-info", "{m}" }
                                                            }
                                                        }
                                                        div { class: "modal-footer",
                                                            button {
                                                                class: "btn btn-primary",
                                                                onclick: move |_| {
                                                                    let cid2 = cid.clone();
                                                                    spawn(async move {
                                                                        match client::enroll_student(&cid2).await {
                                                                            Ok(resp) => {
                                                                                enroll_msg.set(Some(resp["message"].as_str().unwrap_or("OK").to_string()));
                                                                                enrolling_id.set(None);
                                                                                enroll_success.set(true);
                                                                                contracts.restart();
                                                                            }
                                                                            Err(e) => enroll_msg.set(Some(format!("Error: {e}"))),
                                                                        }
                                                                    });
                                                                },
                                                                "Confirmar Matrícula"
                                                            }
                                                            button { class: "btn", onclick: move |_| enrolling_id.set(None), "Cancelar" }
                                                        }
                                                    }
                                                }
                                            }
                                        });
                                        enroll_modal.unwrap_or(rsx! {})
                                    }
                                    {
                                        let show_view = view_contract_id();
                                        let view_modal = show_view.as_ref().map(|vid| {
                                            let cid = vid.clone();
                                            rsx! {
                                                div { class: "modal-overlay", role: "dialog", onclick: move |_| view_contract_id.set(None),
                                                    div { class: "modal-content", onclick: move |e| e.stop_propagation(),
                                                        div { class: "modal-header",
                                                            h2 { "Detalle del Contrato" }
                                                            button { class: "btn-icon", onclick: move |_| view_contract_id.set(None), "✕" }
                                                        }
                                                        div { class: "modal-body",
                                                            match contract_detail() {
                                                                Some(Ok(j)) => {
                                                                    let student = j["student"].as_str().unwrap_or("").to_string();
                                                                    let grade = j["grade"].as_str().unwrap_or("").to_string();
                                                                    let amount = j["amount"].as_str().unwrap_or("").to_string();
                                                                    let status = j["status"].as_str().unwrap_or("").to_string();
                                                                    let date = j["date"].as_str().unwrap_or("").to_string();
                                                                    rsx! {
                                                                        table { class: "detail-table",
                                                                            tbody {
                                                                                tr { td { "ID:" } td { "{cid}" } }
                                                                                tr { td { "Estudiante:" } td { "{student}" } }
                                                                                tr { td { "Nivel:" } td { "{grade}" } }
                                                                                tr { td { "Monto:" } td { "{amount}" } }
                                                                                tr { td { "Estado:" } td { "{status}" } }
                                                                                tr { td { "Creado:" } td { "{date}" } }
                                                                            }
                                                                        }
                                                                    }
                                                                },
                                                                Some(Err(e)) => rsx! { p { "Error: {e}" } },
                                                                None => rsx! { p { "Cargando..." } },
                                                            }
                                                        }
                                                        div { class: "modal-footer",
                                                            button { class: "btn", onclick: move |_| view_contract_id.set(None), "Cerrar" }
                                                        }
                                                    }
                                                }
                                            }
                                        });
                                        view_modal.unwrap_or(rsx! {})
                                    }
                                }
                            }
                            _ => rsx! { div { class: "loading-spinner", "Cargando contratos..." } },
                        }
                    }
                }
            } else {
                rsx! {
                    div { class: "page-toolbar",
                        h3 { "Becas y Descuentos" }
                        button { class: "btn btn-primary", onclick: move |_| show_new_scholarship.set(!show_new_scholarship()), if show_new_scholarship() { "Cancelar" } else { "Nueva Beca" } }
                    }
                    {
                        if show_new_scholarship() {
                            rsx! {
                                div { class: "form-card",
                                    div { class: "form-row",
                                        div { class: "form-group",
                                            label { "Nombre:" }
                                            input { class: "form-input", value: "{new_scholarship_name}", oninput: move |e| new_scholarship_name.set(e.value()), placeholder: "Beca Excelencia" }
                                        }
                                        div { class: "form-group",
                                            label { "% Descuento:" }
                                            input { class: "form-input", value: "{new_scholarship_discount}", oninput: move |e| new_scholarship_discount.set(e.value()), type: "number", min: "1", max: "100" }
                                        }
                                        div { class: "form-group",
                                            label { "Cupo Máx:" }
                                            input { class: "form-input", value: "{new_scholarship_max}", oninput: move |e| new_scholarship_max.set(e.value()), type: "number", min: "1" }
                                        }
                                    }
                                    div { class: "form-actions",
                                        button { class: "btn btn-primary", disabled: saving_scholarship(), onclick: do_create_scholarship, if saving_scholarship() { "Guardando..." } else { "Crear Beca" } }
                                        button { class: "btn", onclick: move |_| show_new_scholarship.set(false), "Cancelar" }
                                    }
                                }
                            }
                        } else { rsx! {} }
                    }
                    match scholarships() {
                        Some(Ok(data)) => {
                            let list = data["scholarships"].as_array().cloned().unwrap_or_default();
                            let s_rows: Vec<Element> = list.iter().map(|s| {
                                let sid = s["id"].as_str().unwrap_or("").to_string();
                                let sname = s["name"].as_str().unwrap_or("-").to_string();
                                let sdiscount = format!("{:.0}%", s["discount"].as_f64().unwrap_or(0.0));
                                let sbenef = format!("{}/{}", s["current"].as_i64().unwrap_or(0), s["max"].as_i64().unwrap_or(0));
                                let sactive = s["active"].as_bool().unwrap_or(false);
                                rsx! {
                                    tr {
                                        td { "{sname}" }
                                        td { "{sdiscount}" }
                                        td { "{sbenef}" }
                                        td { if sactive { span { class: "grade-good", "✓ Activa" } } else { span { class: "grade-bad", "✗ Inactiva" } } }
                                        td {
                                            button { class: "btn btn-sm", onclick: { let id = sid.clone(); move |_| do_toggle_scholarship(id.clone()) }, if sactive { "Desactivar" } else { "Activar" } }
                                            button { class: "btn btn-sm btn-primary", style: "margin-left: 4px;", onclick: { let id = sid.clone(); move |_| { show_apply_scholarship.set(Some(id.clone())); apply_student_id.set(String::new()); apply_msg.set(None); } }, "Aplicar" }
                                        }
                                    }
                                }
                            }).collect();
                            let s_empty = list.is_empty();
                            rsx! {
                                div { class: "data-table-container",
                                    table { class: "data-table",
                                        thead { tr {
                                            th { "Nombre" }
                                            th { "Descuento" }
                                            th { "Beneficiarios" }
                                            th { "Estado" }
                                            th { "Acciones" }
                                        }}
                                        tbody { {s_rows.into_iter()} }
                                    }
                                    if s_empty {
                                        div { class: "empty-state", "Sin becas configuradas" }
                                    }
                                }
                                {
                                    let show_apply = show_apply_scholarship();
                                    let apply_modal = show_apply.as_ref().map(|_| {
                                        rsx! {
                                            div { class: "modal-overlay", role: "dialog", onclick: move |_| show_apply_scholarship.set(None),
                                                div { class: "modal-content", onclick: move |e| e.stop_propagation(),
                                                    div { class: "modal-header",
                                                        h2 { "Aplicar Beca" }
                                                        button { class: "btn-icon", onclick: move |_| show_apply_scholarship.set(None), "✕" }
                                                    }
                                                    div { class: "modal-body",
                                                        div { class: "form-group",
                                                            label { "ID del Estudiante:" }
                                                            input { class: "form-input", value: "{apply_student_id}", oninput: move |e| apply_student_id.set(e.value()), placeholder: "Ingrese el ID del estudiante" }
                                                        }
                                                        if let Some(ref msg) = apply_msg() {
                                                            div { class: "alert alert-info", "{msg}" }
                                                        }
                                                    }
                                                    div { class: "modal-footer",
                                                        button { class: "btn btn-primary", onclick: do_apply_scholarship, "Aplicar" }
                                                        button { class: "btn", onclick: move |_| show_apply_scholarship.set(None), "Cancelar" }
                                                    }
                                                }
                                            }
                                        }
                                    });
                                    apply_modal.unwrap_or(rsx! {})
                                }
                            }
                        }
                        _ => rsx! { div { class: "loading-spinner", "Cargando becas..." } },
                    }
                }
            }
        }
    }
}
