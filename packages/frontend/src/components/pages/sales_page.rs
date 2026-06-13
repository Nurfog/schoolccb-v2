use dioxus::prelude::*;
use serde_json::{Value, json};
use crate::api::client;
use crate::components::widgets::kpi_card::KpiCard;

#[component]
pub fn SalesPage() -> Element {
    let mut active_tab = use_signal(|| "pipeline".to_string());

    let tab_pipeline = if active_tab() == "pipeline" { "tab active" } else { "tab" };
    let tab_dashboard = if active_tab() == "dashboard" { "tab active" } else { "tab" };
    let tab_team = if active_tab() == "team" { "tab active" } else { "tab" };
    let tab_proposals = if active_tab() == "proposals" { "tab active" } else { "tab" };
    let tab_contracts = if active_tab() == "contracts" { "tab active" } else { "tab" };
    let tab_documents = if active_tab() == "documents" { "tab active" } else { "tab" };

    rsx! {
        div { class: "page-header",
            h1 { "CRM de Ventas" }
            p { "Pipeline comercial — gesti\u{00f3}n de prospectos, propuestas, contratos, documentos y equipo" }
        }
        div { class: "tab-bar",
            button { class: "{tab_pipeline}", onclick: move |_| active_tab.set("pipeline".to_string()), "Pipeline" }
            button { class: "{tab_proposals}", onclick: move |_| active_tab.set("proposals".to_string()), "Cotizaciones" }
            button { class: "{tab_contracts}", onclick: move |_| active_tab.set("contracts".to_string()), "Contratos" }
            button { class: "{tab_documents}", onclick: move |_| active_tab.set("documents".to_string()), "Documentos" }
            button { class: "{tab_dashboard}", onclick: move |_| active_tab.set("dashboard".to_string()), "Dashboard" }
            button { class: "{tab_team}", onclick: move |_| active_tab.set("team".to_string()), "Equipo" }
        }
        div { class: "tab-content",
            if active_tab() == "pipeline" {
                SalesPipeline {}
            } else if active_tab() == "proposals" {
                SalesProposals {}
            } else if active_tab() == "contracts" {
                SalesContracts {}
            } else if active_tab() == "documents" {
                SalesDocuments {}
            } else if active_tab() == "dashboard" {
                SalesDashboard {}
            } else {
                SalesTeam {}
            }
        }
    }
}

// ─── Pipeline Tab ───

#[component]
fn SalesPipeline() -> Element {
    let stages = use_resource(|| client::fetch_json("/b2b/sales/stages"));
    let mut prospects = use_resource(|| client::fetch_json("/b2b/sales/prospects"));
    let mut selected_id = use_signal(|| None::<String>);
    let mut show_new = use_signal(|| false);
    let mut view_mode = use_signal(|| "kanban".to_string());
    let mut search_term = use_signal(String::new);

    let prospect_detail = use_resource(move || {
        let sid = selected_id();
        async move {
            match sid {
                Some(id) => client::fetch_json(&format!("/b2b/sales/prospects/{}", id)).await,
                None => Err("none".to_string()),
            }
        }
    });

    let mut first_name = use_signal(String::new);
    let mut last_name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut company = use_signal(String::new);
    let mut rut = use_signal(String::new);
    let mut position = use_signal(String::new);
    let mut source = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let mut saving = use_signal(|| false);

    let kanban_btn_class = if view_mode() == "kanban" { "btn btn-primary" } else { "btn btn-secondary" };
    let table_btn_class = if view_mode() == "table" { "btn btn-primary" } else { "btn btn-secondary" };

    let do_create = move |_| {
        saving.set(true);
        let payload = json!({
            "first_name": first_name(), "last_name": last_name(), "email": email(),
            "phone": phone(), "company": company(), "rut": rut(), "position": position(),
            "source": source(), "notes": notes(),
        });
        spawn(async move {
            let _ = client::post_json("/b2b/sales/prospects", &payload).await;
            saving.set(false);
            show_new.set(false);
            first_name.set(String::new()); last_name.set(String::new()); email.set(String::new());
            phone.set(String::new()); company.set(String::new()); rut.set(String::new()); position.set(String::new());
            source.set(String::new()); notes.set(String::new());
            prospects.restart();
        });
    };

    rsx! {
        div { class: "page-toolbar",
            button { class: "btn btn-primary", onclick: move |_| show_new.set(!show_new()),
                if show_new() { "Cancelar" } else { "Nuevo Prospecto" }
            }
            button { class: "{kanban_btn_class}", onclick: move |_| view_mode.set("kanban".to_string()), "Kanban" }
            button { class: "{table_btn_class}", onclick: move |_| view_mode.set("table".to_string()), "Tabla" }
            input { class: "search-input", placeholder: "Buscar prospecto...", value: "{search_term}", oninput: move |e| search_term.set(e.value()) }
        }
        if show_new() {
            div { class: "form-card",
                div { class: "form-row",
                    div { class: "form-group", label { "Nombre *" } input { class: "form-input", value: "{first_name}", oninput: move |e| first_name.set(e.value()) } }
                    div { class: "form-group", label { "Apellido *" } input { class: "form-input", value: "{last_name}", oninput: move |e| last_name.set(e.value()) } }
                }
                div { class: "form-row",
                    div { class: "form-group", label { "Email" } input { class: "form-input", value: "{email}", oninput: move |e| email.set(e.value()) } }
                    div { class: "form-group", label { "Tel\u{00e9}fono" } input { class: "form-input", value: "{phone}", oninput: move |e| phone.set(e.value()) } }
                }
                div { class: "form-row",
                    div { class: "form-group", label { "Colegio" } input { class: "form-input", value: "{company}", oninput: move |e| company.set(e.value()) } }
                    div { class: "form-group", label { "RUT (Empresa/Persona)" } input { class: "form-input", value: "{rut}", oninput: move |e| rut.set(e.value()), placeholder: "12.345.678-9" } }
                }
                div { class: "form-row",
                    div { class: "form-group", label { "Cargo" } input { class: "form-input", value: "{position}", oninput: move |e| position.set(e.value()) } }
                    div { class: "form-group",
                        label { "Fuente" }
                        select { class: "form-input", value: "{source}", oninput: move |e| source.set(e.value()),
                            option { value: "", "Seleccionar..." }
                            option { value: "web", "Web" } option { value: "referido", "Referido" }
                            option { value: "llamada", "Llamada" } option { value: "whatsapp", "WhatsApp" }
                            option { value: "email", "Email" } option { value: "feria", "Feria" } option { value: "otro", "Otro" }
                        }
                    }
                }
                div { class: "form-actions",
                    button { class: "btn btn-primary", disabled: saving() || first_name().trim().is_empty() || last_name().trim().is_empty(), onclick: do_create,
                        if saving() { "Creando..." } else { "Crear Prospecto" }
                    }
                }
            }
        }
        if view_mode() == "kanban" {
            match stages() {
                Some(Ok(data)) => {
                    let sl: Vec<Value> = data["stages"].as_array().cloned().unwrap_or_default();
                    rsx! { SalesKanbanBoard { stages: sl, prospects: prospects, selected_id: selected_id } }
                }
                _ => rsx! { div { class: "loading-spinner", "Cargando pipeline..." } },
            }
        } else {
            match prospects() {
                Some(Ok(pdata)) => {
                    let list: Vec<Value> = pdata["prospects"].as_array().cloned().unwrap_or_default();
                    let stages_map: std::collections::HashMap<String, String> = match stages() {
                        Some(Ok(d)) => d["stages"].as_array().cloned().unwrap_or_default()
                            .into_iter().filter_map(|s| {
                                Some((s["id"].as_str()?.to_string(), s["name"].as_str()?.to_string()))
                            }).collect(),
                        _ => std::collections::HashMap::new(),
                    };
                    rsx! { SalesTableView { prospects: list, stages_map: stages_map } }
                }
                _ => rsx! { div { class: "loading-spinner", "Cargando..." } },
            }
        }
        if selected_id().is_some() {
            ProspectDetailModal {
                detail: prospect_detail,
                on_close: move |_| selected_id.set(None),
            }
        }
    }
}

#[component]
fn SalesKanbanBoard(stages: Vec<Value>, prospects: Resource<Result<Value, String>>, selected_id: Signal<Option<String>>) -> Element {
    let prospect_list: Vec<Value> = match prospects() {
        Some(Ok(d)) => d["prospects"].as_array().cloned().unwrap_or_default(),
        _ => vec![],
    };

    let stage_infos: Vec<StageInfo> = stages.iter().map(|s| {
        let stage_id = s["id"].as_str().unwrap_or("").to_string();
        let items: Vec<ProspectInfo> = prospect_list.iter()
            .filter(|p| p["current_stage_id"].as_str().unwrap_or("") == stage_id)
            .map(|p| {
                let pid = p["id"].as_str().unwrap_or("").to_string();
                let name = format!("{} {}", p["first_name"].as_str().unwrap_or(""), p["last_name"].as_str().unwrap_or(""));
                let company = p["company"].as_str().unwrap_or("").to_string();
                ProspectInfo { id: pid, name, company }
            })
            .collect();
        StageInfo {
            name: s["name"].as_str().unwrap_or("").to_string(),
            color: s["color"].as_str().unwrap_or("#6B7280").to_string(),
            items,
        }
    }).collect();

    rsx! {
        div { class: "kanban-board",
            for si in &stage_infos {
                div { class: "kanban-column",
                    div { class: "kanban-column-header", style: "border-top-color: {si.color}",
                        div { class: "kanban-column-title", "{si.name}" }
                        div { class: "kanban-column-count", "{si.items.len()}" }
                    }
                    div { class: "kanban-column-body",
                        for item in &si.items {
                            SalesKanbanCard {
                                name: item.name.clone(),
                                company: item.company.clone(),
                                onclick: { let pid = item.id.clone(); move |_| selected_id.set(Some(pid.clone())) },
                            }
                        }
                    }
                }
            }
        }
    }
}

struct StageInfo {
    name: String,
    color: String,
    items: Vec<ProspectInfo>,
}

struct ProspectInfo {
    id: String,
    name: String,
    company: String,
}

#[component]
fn SalesKanbanCard(name: String, company: String, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        div { class: "kanban-card", onclick: move |e| onclick.call(e),
            div { class: "kanban-card-name", "{name}" }
            if !company.is_empty() {
                div { class: "kanban-card-company", "{company}" }
            }
        }
    }
}

#[component]
fn SalesTableView(prospects: Vec<Value>, stages_map: std::collections::HashMap<String, String>) -> Element {
    rsx! {
        div { class: "data-table-container",
            table { class: "data-table",
                thead { tr { th { "Nombre" } th { "RUT" } th { "Email" } th { "Colegio" } th { "Etapa" } th { "Valor" } } }
                tbody {
                    for p in &prospects {
                        SalesTableRow { prospect: p.clone(), stages_map: stages_map.clone() }
                    }
                }
            }
        }
    }
}

#[component]
fn SalesTableRow(prospect: Value, stages_map: std::collections::HashMap<String, String>) -> Element {
    let first = prospect["first_name"].as_str().unwrap_or("").to_string();
    let last = prospect["last_name"].as_str().unwrap_or("").to_string();
    let email = prospect["email"].as_str().unwrap_or("-").to_string();
    let company = prospect["company"].as_str().unwrap_or("-").to_string();
    let stage_id = prospect["current_stage_id"].as_str().unwrap_or("").to_string();
    let stage_name = stages_map.get(&stage_id).cloned().unwrap_or_else(|| "-".to_string());
    let val = prospect["estimated_value"].as_f64().map(|v| format!("${:.0}", v)).unwrap_or_else(|| "-".to_string());

    rsx! {
        tr {
            td { "{first} {last}" }
            td { "{prospect[\"rut\"].as_str().unwrap_or(\"-\")}" }
            td { "{email}" }
            td { "{company}" }
            td { span { class: "stage-badge", "{stage_name}" } }
            td { "{val}" }
        }
    }
}

#[component]
fn ProspectDetailModal(
    detail: Resource<Result<Value, String>>,
    on_close: EventHandler<()>,
) -> Element {
    let mut show_timeline = use_signal(|| true);
    let mut activation_result = use_signal(|| None::<Value>);
    let is_activating = use_signal(|| false);
    let mut show_activate_wizard = use_signal(|| false);
    let mut wizard_step = use_signal(|| 0u32);

    let detail_data = match detail() {
        Some(Ok(ref data)) => Some(data.clone()),
        _ => None,
    };

    let first_name;
    let last_name;
    let stage_color;
    let stage_name;
    let email_val;
    let phone_val;
    let company_val;
    let source_val;
    let value_val;
    let assigned_name;
    let prospect_id;
    let p_rut;
    let contract_cards: Vec<_>;

    if let Some(ref data) = detail_data {
        let p = &data["prospect"];
        let stage = &data["stage"];
        let assigned = &data["assigned_user"];
        let contracts = data["contracts"].as_array().cloned().unwrap_or_default();
        prospect_id = p["id"].as_str().unwrap_or("").to_string();
        first_name = p["first_name"].as_str().unwrap_or("").to_string();
        last_name = p["last_name"].as_str().unwrap_or("").to_string();
        stage_color = stage["color"].as_str().unwrap_or("#6B7280").to_string();
        stage_name = stage["name"].as_str().unwrap_or("Sin etapa").to_string();
        email_val = p["email"].as_str().unwrap_or("-").to_string();
        phone_val = p["phone"].as_str().unwrap_or("-").to_string();
        company_val = p["company"].as_str().unwrap_or("-").to_string();
        source_val = p["source"].as_str().unwrap_or("-").to_string();
        value_val = p["estimated_value"].as_f64().map(|v| format!("${:.0}", v)).unwrap_or_else(|| "-".to_string());
        assigned_name = assigned["name"].as_str().unwrap_or("-").to_string();
        p_rut = p["rut"].as_str().unwrap_or("-").to_string();
        contract_cards = contracts.iter().map(|c| {
            let cs = c["status"].as_str().unwrap_or("draft").to_string();
            let cv = c["total_value"].as_f64().unwrap_or(0.0);
            let cid = c["id"].as_str().unwrap_or("").to_string();
            let is_verified = cs == "verified";
            let act_label = if is_activating() { "Activando..." } else { "Iniciar Activación" };
            rsx! {
                div { key: "{cid}", class: "contract-card",
                    div { class: "contract-status-{cs}", "{cs}" }
                    div { "Valor: ${cv}" }
                    if is_verified {
                        button {
                            class: "btn btn-sm btn-success",
                            disabled: is_activating(),
                            onclick: move |_| {
                                show_activate_wizard.set(true);
                                wizard_step.set(0);
                            },
                            "{act_label}"
                        }
                    }
                }
            }
        }).collect();
    } else {
        first_name = String::new(); last_name = String::new(); stage_color = String::new(); stage_name = String::new();
        email_val = String::new(); phone_val = String::new(); company_val = String::new();
        source_val = String::new(); value_val = String::new(); assigned_name = String::new();
        prospect_id = String::new(); p_rut = String::new(); contract_cards = Vec::new();
    }

    let loading = detail_data.is_none();
    let activation_data = activation_result();
    let activation_email = activation_data.as_ref().and_then(|d| d["admin_email"].as_str()).unwrap_or("").to_string();
    let activation_password = activation_data.as_ref().and_then(|d| d["temp_password"].as_str()).unwrap_or("").to_string();
    let tab_activity_class = if show_timeline() { "tab active" } else { "tab" };
    let tab_info_class = if !show_timeline() { "tab active" } else { "tab" };

    rsx! {
        div { class: "modal-overlay", onclick: move |_| on_close.call(()),
            div { class: "modal-content modal-lg", onclick: move |e| e.stop_propagation(),
                if activation_data.is_some() {
                    div { class: "p-8 text-center",
                        h3 { class: "text-2xl font-bold text-success mb-4", "Licencia Activada" }
                        p { class: "mb-6", "La corporaci\u{00f3}n y el colegio han sido creados exitosamente." }
                        div { class: "bg-gray-50 p-6 rounded-lg mb-6 text-left border border-gray-200",
                            div { class: "mb-2", b { "Email: " } "{activation_email}" }
                            div { class: "mb-2", b { "Contrase\u{00f1}a Temporal: " } span { class: "font-mono bg-blue-50 text-blue-700 px-2 py-1 rounded", "{activation_password}" } }
                        }
                        p { class: "text-sm text-gray-500 mb-6", "Por favor, comparte estas credenciales con el sostenedor." }
                        button { class: "btn btn-primary w-full", onclick: move |_| activation_result.set(None), "Entendido" }
                    }
                } else if loading {
                    div { class: "modal-header",
                        h2 { "Cargando..." }
                        button { class: "btn-close", onclick: move |_| on_close.call(()) }
                    }
                    div { class: "modal-body", div { class: "loading-spinner", "Cargando..." } }
                } else {
                    div { class: "modal-header",
                        h2 { "{first_name} {last_name}" }
                        span { class: "stage-badge", style: "background: {stage_color}", "{stage_name}" }
                        button { class: "btn-close", onclick: move |_| on_close.call(()) }
                    }
                    div { class: "modal-body",
                        div { class: "detail-tabs",
                            button { class: "{tab_activity_class}", onclick: move |_| show_timeline.set(true), "Actividad" }
                            button { class: "{tab_info_class}", onclick: move |_| show_timeline.set(false), "Info" }
                        }
                        if show_timeline() {
                            ContactTimeline { prospect_id: prospect_id.clone() }
                        } else {
                            div { class: "detail-grid",
                                div { class: "detail-section",
                                    h3 { "Informaci\u{00f3}n" }
                                    div { class: "detail-row", label { "Email:" }, span { "{email_val}" } }
                                    div { class: "detail-row", label { "RUT:" }, span { "{p_rut}" } }
                                    div { class: "detail-row", label { "Tel\u{00e9}fono:" }, span { "{phone_val}" } }
                                    div { class: "detail-row", label { "Colegio:" }, span { "{company_val}" } }
                                    div { class: "detail-row", label { "Fuente:" }, span { "{source_val}" } }
                                    div { class: "detail-row", label { "Valor est.:" }, span { "{value_val}" } }
                                    div { class: "detail-row", label { "Asignado:" }, span { "{assigned_name}" } }
                                }
                            }
                        }
                        {build_activation_wizard(
                            show_activate_wizard,
                            wizard_step,
                            activation_result,
                            is_activating,
                            company_val.clone(),
                            email_val.clone(),
                            prospect_id.clone(),
                        )}
                        if !contract_cards.is_empty() {
                            div { class: "detail-section",
                                h3 { "Contratos" }
                                {contract_cards.into_iter()}
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ContactTimeline(prospect_id: String) -> Element {
    let activities = use_resource(move || {
        let pid = prospect_id.clone();
        async move { client::fetch_json(&format!("/b2b/sales/prospects/{}/activities", pid)).await }
    });

    let items = match activities() {
        Some(Ok(data)) => {
            let list = data["activities"].as_array().cloned().unwrap_or_default();
            let cards: Vec<_> = list.iter().map(|act| {
                let atype = act["activity_type"].as_str().unwrap_or("").to_string();
                let subject = act["subject"].as_str().unwrap_or("").to_string();
                let desc = act["description"].as_str().unwrap_or("").to_string();
                let created = act["created_at"].as_str().unwrap_or("").to_string();
                let icon = match atype.as_str() {
                    "call" => "📞", "email" => "✉️", "whatsapp" => "💬",
                    "meeting" => "🤝", "proposal" => "📄",
                    "contract" => "📃", "activation" => "✅",
                    "stage_change" => "🔄", "assign" => "👤",
                    _ => "📋",
                };
                rsx! {
                    div { class: "timeline-item",
                        div { class: "timeline-icon", "{icon}" }
                        div { class: "timeline-content",
                            div { class: "timeline-subject", "{subject}" }
                            if !desc.is_empty() { div { class: "timeline-desc", "{desc}" } }
                            div { class: "timeline-date", "{created}" }
                        }
                    }
                }
            }).collect();
            if cards.is_empty() {
                rsx! { div { class: "empty-state", "Sin actividad registrada" } }
            } else {
                rsx! { div { class: "timeline", {cards.into_iter()} } }
            }
        }
        Some(Err(_)) => rsx! { div { class: "empty-state", "Error al cargar actividades" } },
        None => rsx! { div { class: "loading-spinner", "Cargando..." } },
    };

    rsx! {
        div { class: "timeline-container",
            h3 { "Historial de Actividad" }
            {items}
        }
    }
}

// ─── Dashboard Tab ───

#[component]
fn SalesDashboard() -> Element {
    let dashboard = use_resource(|| client::fetch_json("/b2b/sales/dashboard/summary"));
    let agents = use_resource(|| client::fetch_json("/b2b/sales/agents"));

    let data = match dashboard() {
        Some(Ok(ref d)) => Some(d.clone()),
        _ => None,
    };

    let total_prospects = data.as_ref().and_then(|d| d["total_prospects"].as_i64()).unwrap_or(0);
    let my_prospects = data.as_ref().and_then(|d| d["my_prospects"].as_i64()).unwrap_or(0);
    let total_contracts = data.as_ref().and_then(|d| d["total_contracts"].as_i64()).unwrap_or(0);
    let total_value = data.as_ref().and_then(|d| d["total_value"].as_f64()).unwrap_or(0.0);
    let pipeline = data.as_ref().and_then(|d| d["pipeline"].as_array()).cloned().unwrap_or_default();

    let total_agents = agents().and_then(|r| r.ok())
        .and_then(|d| d["agents"].as_array().map(|a| a.len() as i64)).unwrap_or(0);

    let pipeline_total: f64 = pipeline.iter().filter_map(|s| s["count"].as_i64()).map(|c| c as f64).sum();
    let projected_revenue = if pipeline_total > 0.0 && total_prospects > 0 {
        pipeline_total * (total_value / total_prospects as f64) * 0.3
    } else {
        0.0
    };
    let has_pipeline = !pipeline.is_empty();

    rsx! {
        div { class: "sales-dashboard",
            div { class: "kpi-grid",
                KpiCard { label: "Total Prospectos".to_string(), value: total_prospects.to_string() }
                KpiCard { label: "Mis Prospectos".to_string(), value: my_prospects.to_string() }
                KpiCard { label: "Contratos Activos".to_string(), value: total_contracts.to_string() }
                KpiCard { label: "Valor Total".to_string(), value: format!("${:.0}", total_value), color: Some("#16a34a".to_string()) }
                KpiCard { label: "Agentes".to_string(), value: total_agents.to_string() }
            }
            div { class: "dashboard-section",
                h3 { "Pipeline (embudo de ventas)" }
                if has_pipeline {
                    SalesFunnelChart { data: pipeline.clone() }
                } else {
                    div { class: "empty-state", "Sin datos de pipeline" }
                }
            }
            if has_pipeline {
                div { class: "dashboard-section",
                    h3 { "Proyecci\u{00f3}n de Ingresos" }
                    KpiCard { label: "Ingresos Proyectados (basado en pipeline actual)".to_string(), value: format!("${:.0}", projected_revenue), large: Some(true) }
                }
            }
        }
    }
}
#[component]
fn SalesFunnelChart(data: Vec<Value>) -> Element {
    let max_count = data.iter()
        .filter_map(|s| s["count"].as_i64())
        .max()
        .unwrap_or(1)
        .max(1) as f64;

    let bars: Vec<_> = data.iter().enumerate().map(|(i, s)| {
        let name = s["name"].as_str().unwrap_or("").to_string();
        let count = s["count"].as_i64().unwrap_or(0);
        let pct = (count as f64 / max_count * 100.0).max(5.0);
        let colors = ["#6B7280", "#3B82F6", "#8B5CF6", "#F59E0B", "#F97316", "#10B981", "#059669", "#EF4444"];
        let color = colors[i % colors.len()];
        rsx! {
            div { class: "funnel-row", key: "f{i}",
                div { class: "funnel-label", "{name}" }
                div { class: "funnel-bar-container",
                    div { class: "funnel-bar", style: "width: {pct}%; background: {color};",
                        "{count}"
                    }
                }
            }
        }
    }).collect();

    rsx! {
        div { class: "funnel-chart",
            {bars.into_iter()}
        }
    }
}

// ─── Proposals Tab (Quote Builder) ───

#[component]
fn SalesProposals() -> Element {
    let mut proposals = use_resource(client::fetch_sales_proposals);
    let plans = use_resource(client::fetch_sales_plans);
    let mut show_form = use_signal(|| false);
    let mut sel_prospect_id = use_signal(String::new);
    let mut sel_plan_id = use_signal(String::new);
    let mut total_value = use_signal(|| 0.0);
    let mut saving = use_signal(|| false);

    rsx! {
        div { class: "page-toolbar",
            button { class: "btn btn-primary", onclick: move |_| show_form.set(!show_form()),
                if show_form() { "Cancelar" } else { "Nueva Cotizaci\u{00f3}n" }
            }
        }
        if show_form() {
            match plans() {
                Some(Ok(data)) => {
                    let list = data["plans"].as_array().cloned().unwrap_or_default();
                    let plan_opts: Vec<Element> = list.iter().map(|plan| {
                        let pid = plan["id"].as_str().unwrap_or("").to_string();
                        let pname = plan["name"].as_str().unwrap_or("").to_string();
                        let price = plan["price_monthly"].as_f64().unwrap_or(0.0);
                        let modules = plan["modules"].as_array().cloned().unwrap_or_default();
                        let mod_count = modules.iter().filter(|m| m["included"].as_bool().unwrap_or(false)).count();
                        rsx! {
                            option { value: "{pid}", "{pname} — ${price:.0}/mes ({mod_count} m\u{00f3}dulos)" }
                        }
                    }).collect();
                    let plan_id = sel_plan_id();
                    let sel_plan = list.iter().find(|p| p["id"].as_str() == Some(&plan_id)).cloned();
                    let plan_detail = match sel_plan {
                        Some(plan) => {
                            let plan_name = plan["name"].as_str().unwrap_or("").to_string();
                            let base_price = plan["price_monthly"].as_f64().unwrap_or(0.0);
                            let mods = plan["modules"].as_array().cloned().unwrap_or_default();
                            let included_count = mods.iter().filter(|m| m["included"].as_bool().unwrap_or(false)).count();
                            let mod_rows: Vec<Element> = mods.iter().map(|m| {
                                let name = m["module_name"].as_str().unwrap_or("").to_string();
                                let inc = m["included"].as_bool().unwrap_or(false);
                                let icon = if inc { "✅" } else { "❌" };
                                rsx! { div { class: "alert-item", span { "{icon} {name}" } } }
                            }).collect();
                            rsx! {
                                div { class: "widget-card", style: "margin-top: 12px;",
                                    div { class: "widget-card-header",
                                        h3 { "{plan_name}" }
                                        span { "${base_price:.0}/mes" }
                                    }
                                    div { class: "widget-card-body",
                                        p { "{included_count} m\u{00f3}dulos incluidos" }
                                        {mod_rows.into_iter()}
                                        div { class: "form-group", style: "margin-top: 12px;",
                                            label { "Valor Total Estimado:" }
                                            input {
                                                class: "form-input",
                                                r#type: "number",
                                                value: "{total_value}",
                                                oninput: move |e| {
                                                    if let Ok(v) = e.value().parse::<f64>() { total_value.set(v); }
                                                }
                                            }
                                        }
                                        div { class: "form-actions",
                                            button {
                                                class: "btn btn-primary",
                                                disabled: saving() || sel_prospect_id().trim().is_empty(),
                                                onclick: move |_| {
                                                    saving.set(true);
                                                    let payload = serde_json::json!({
                                                        "prospect_id": sel_prospect_id(),
                                                        "plan_id": sel_plan_id(),
                                                        "total_value": total_value(),
                                                        "modules": mods,
                                                        "notes": "Cotizaci\u{00f3}n generada desde CRM"
                                                    });
                                                    spawn(async move {
                                                        let _ = client::create_sales_proposal(&payload).await;
                                                        saving.set(false);
                                                        show_form.set(false);
                                                        proposals.restart();
                                                    });
                                                },
                                                if saving() { "Creando..." } else { "Crear Cotizaci\u{00f3}n" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        None => rsx! {}
                    };
                    rsx! {
                        div { class: "form-card",
                            h3 { "Crear Cotizaci\u{00f3}n" }
                            div { class: "form-row",
                                div { class: "form-group",
                                    label { "ID del Prospecto:" }
                                    input { class: "form-input", value: "{sel_prospect_id}", oninput: move |e| sel_prospect_id.set(e.value()), placeholder: "UUID del prospecto" }
                                }
                                div { class: "form-group",
                                    label { "Plan:" }
                                    select { class: "form-input", value: "{sel_plan_id}", oninput: move |e| sel_plan_id.set(e.value()),
                                        option { value: "", "Seleccionar plan..." }
                                        {plan_opts.into_iter()}
                                    }
                                }
                            }
                            {plan_detail}
                        }
                    }
                }
                _ => rsx! { div { class: "loading-spinner", "Cargando planes..." } }
            }
        }
        match proposals() {
            Some(Ok(data)) => {
                let list = data["proposals"].as_array().cloned().unwrap_or_default();
                rsx! {
                    div { class: "widget-card",
                        div { class: "widget-card-header",
                            h3 { "Cotizaciones" }
                            span { "{list.len()} cotizaciones" }
                        }
                        div { class: "widget-card-body",
                            if list.is_empty() {
                                div { class: "empty-state", "Sin cotizaciones" }
                            } else {
                                {list.into_iter().map(|p| rsx! { SalesProposalRow { p: p } })}
                            }
                        }
                    }
                }
            }
            _ => rsx! { div { class: "loading-spinner", "Cargando..." } }
        }
    }
}

// ─── Contracts Tab (Contract Builder) ───

#[component]
fn SalesContracts() -> Element {
    let proposals = use_resource(client::fetch_sales_proposals);
    let contracts = use_resource(|| client::fetch_json("/b2b/sales/contracts"));
    let mut sel_proposal_id = use_signal(String::new);
    let mut tax_rate = use_signal(|| 19.0);
    let mut notes = use_signal(String::new);
    let mut saving = use_signal(|| false);
    let mut msg = use_signal(|| None::<String>);

    rsx! {
        div { class: "page-toolbar",
            h3 { "Generar Contrato desde Cotizaci\u{00f3}n" }
        }
        div { class: "form-card",
            div { class: "form-row",
                div { class: "form-group",
                    label { "Seleccionar Cotizaci\u{00f3}n Aprobada:" }
                    match proposals() {
                        Some(Ok(data)) => {
                            let list = data["proposals"].as_array().cloned().unwrap_or_default();
                            let approved: Vec<Value> = list.into_iter().filter(|p| p["status"].as_str() == Some("accepted")).collect();
                            let opts: Vec<Element> = approved.iter().map(|p| {
                                let pid = p["id"].as_str().unwrap_or("").to_string();
                                let pname = format!("{} {} — ${:.0}", p["first_name"].as_str().unwrap_or(""), p["last_name"].as_str().unwrap_or(""), p["total_value"].as_f64().unwrap_or(0.0));
                                rsx! { option { value: "{pid}", "{pname}" } }
                            }).collect();
                            rsx! {
                                select { class: "form-input", value: "{sel_proposal_id}", oninput: move |e| sel_proposal_id.set(e.value()),
                                    option { value: "", "Seleccionar..." }
                                    {opts.into_iter()}
                                }
                            }
                        }
                        _ => rsx! { div { class: "loading-spinner", "Cargando..." } }
                    }
                }
            }
            div { class: "form-row",
                div { class: "form-group",
                    label { "Tasa de Impuesto (%):" }
                    input { class: "form-input", r#type: "number", value: "{tax_rate}", oninput: move |e| { if let Ok(v) = e.value().parse::<f64>() { tax_rate.set(v); } } }
                }
                div { class: "form-group",
                    label { "Notas:" }
                    input { class: "form-input", value: "{notes}", oninput: move |e| notes.set(e.value()) }
                }
            }
            div { class: "form-actions",
                button {
                    class: "btn btn-primary",
                    disabled: saving() || sel_proposal_id().trim().is_empty(),
                    onclick: move |_| {
                        saving.set(true);
                        let payload = serde_json::json!({
                            "prospect_id": sel_proposal_id(),
                            "plan_id": "",
                            "total_value": 0,
                            "modules": [],
                            "tax_rate": tax_rate() / 100.0,
                            "notes": notes(),
                        });
                        let _pid = sel_proposal_id();
                        spawn(async move {
                            let result = client::create_sales_contract(&payload).await;
                            saving.set(false);
                            match result {
                                Ok(resp) => {
                                    let cid = resp["id"].as_str().unwrap_or("").to_string();
                                    msg.set(Some(format!("Contrato creado exitosamente (ID: {cid})")));
                                }
                                Err(e) => msg.set(Some(format!("Error: {e}"))),
                            }
                        });
                    },
                    if saving() { "Creando..." } else { "Crear Contrato" }
                }
            }
            if let Some(ref m) = msg() {
                div { class: "alert alert-success", style: "margin-top: 12px;", "{m}" }
            }
        }
        match contracts() {
            Some(Ok(data)) => {
                let list = data["contracts"].as_array().cloned().unwrap_or_default();
                rsx! {
                    div { class: "widget-card", style: "margin-top: 16px;",
                        div { class: "widget-card-header",
                            h3 { "Contratos" }
                            span { "{list.len()} contratos" }
                        }
                        div { class: "widget-card-body",
                            if list.is_empty() {
                                div { class: "empty-state", "Sin contratos" }
                            } else {
                                {list.into_iter().map(|c| rsx! { SalesContractRow { c: c } })}
                            }
                        }
                    }
                }
            }
            _ => rsx! { div { class: "loading-spinner", "Cargando contratos..." } }
        }
    }
}

#[component]
fn SalesProposalRow(p: Value) -> Element {
    let mut show_detail = use_signal(|| false);
    let mut detail_result = use_signal(|| None::<Result<Value, String>>);
    let mut show_discount = use_signal(|| false);
    let mut discount_pct = use_signal(|| 0.0);
    let mut discount_msg = use_signal(|| None::<String>);
    let mut applying = use_signal(|| false);
    let mut generating = use_signal(|| false);
    let mut pdf_msg = use_signal(|| None::<String>);

    let pid = p["id"].as_str().unwrap_or("").to_string();
    let prospect_name = format!("{} {}", p["first_name"].as_str().unwrap_or(""), p["last_name"].as_str().unwrap_or(""));
    let plan_name = p["plan_name"].as_str().unwrap_or("-").to_string();
    let val = p["total_value"].as_f64().unwrap_or(0.0);
    let discount = p["discount"].as_f64().unwrap_or(0.0);
    let status = p["status"].as_str().unwrap_or("draft").to_string();
    let version = p["version"].as_i64().unwrap_or(1);
    let net = val - discount;

    rsx! {
        div { class: "contract-card",
            div { class: "contract-status-{status}", "{status}" }
            div { style: "flex: 1;",
                div { class: "alert-name", "{prospect_name} — {plan_name}" }
                div { class: "alert-detail", "Valor: ${val:.0} | Desc: ${discount:.0} | Neto: ${net:.0} | v{version}" }
            }
            div { class: "contract-actions",
                button {
                    class: "btn btn-sm",
                    onclick: {
                        let pid = pid.clone();
                        move |_| {
                            show_detail.set(!show_detail());
                            if !show_detail() {
                                detail_result.set(None);
                            } else {
                                let pid = pid.clone();
                                spawn(async move {
                                    detail_result.set(Some(client::get_sales_proposal(&pid).await));
                                });
                            }
                        }
                    },
                    if show_detail() { "Ocultar Detalle" } else { "Detalle" }
                }
                button { class: "btn btn-sm", onclick: move |_| show_discount.set(!show_discount()), "Aplicar Descuento" }
                button {
                    class: "btn btn-sm",
                    disabled: generating(),
                    onclick: {
                        let pid = pid.clone();
                        move |_| {
                            generating.set(true);
                            let pid = pid.clone();
                            spawn(async move {
                                match client::generate_proposal_pdf(&pid).await {
                                    Ok(resp) => pdf_msg.set(Some(resp["url"].as_str().unwrap_or("PDF generado").to_string())),
                                    Err(e) => pdf_msg.set(Some(format!("Error: {e}"))),
                                }
                                generating.set(false);
                            });
                        }
                    },
                    if generating() { "Generando..." } else { "Generar PDF" }
                }
            }
            if show_discount() {
                div { class: "discount-form", style: "margin-top: 8px; display: flex; gap: 8px; align-items: center;",
                    input {
                        class: "form-input",
                        r#type: "number",
                        style: "width: 100px;",
                        placeholder: "%",
                        value: "{discount_pct}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<f64>() { discount_pct.set(v); }
                        }
                    }
                    button {
                        class: "btn btn-sm btn-primary",
                        disabled: applying(),
                        onclick: {
                            let pid = pid.clone();
                            move |_| {
                                applying.set(true);
                                let pid = pid.clone();
                                let payload = json!({"discount_pct": discount_pct()});
                                spawn(async move {
                                    match client::apply_proposal_discount(&pid, &payload).await {
                                        Ok(_) => discount_msg.set(Some("Descuento aplicado".to_string())),
                                        Err(e) => discount_msg.set(Some(format!("Error: {e}"))),
                                    }
                                    applying.set(false);
                                });
                            }
                        },
                        if applying() { "Aplicando..." } else { "Aplicar" }
                    }
                    if let Some(ref msg) = discount_msg() {
                        span { class: "text-success", style: "font-size: 12px;", "{msg}" }
                    }
                }
            }
            if show_detail() {
                match detail_result() {
                    Some(Ok(ref data)) => {
                        let d_plan = data["plan_name"].as_str().unwrap_or("-").to_string();
                        let d_val = data["total_value"].as_f64().unwrap_or(0.0);
                        let d_disc = data["discount"].as_f64().unwrap_or(0.0);
                        let d_notes = data["notes"].as_str().unwrap_or("-").to_string();
                        let d_status = data["status"].as_str().unwrap_or("-").to_string();
                        rsx! {
                            div { class: "detail-section", style: "margin-top: 8px; padding: 8px; background: #f9fafb; border-radius: 6px;",
                                div { class: "detail-row", label { "Plan:" } span { "{d_plan}" } }
                                div { class: "detail-row", label { "Valor:" } span { "${d_val:.0}" } }
                                div { class: "detail-row", label { "Descuento:" } span { "${d_disc:.0}" } }
                                div { class: "detail-row", label { "Notas:" } span { "{d_notes}" } }
                                div { class: "detail-row", label { "Estado:" } span { "{d_status}" } }
                            }
                        }
                    }
                    Some(Err(e)) => rsx! { div { class: "text-error", style: "font-size: 12px; margin-top: 8px;", "Error: {e}" } },
                    None => rsx! { div { class: "loading-spinner", style: "margin-top: 8px;", "Cargando..." } },
                }
            }
            if let Some(ref msg) = pdf_msg() {
                div { class: "text-success", style: "font-size: 12px; margin-top: 4px;", "{msg}" }
            }
        }
    }
}

#[component]
fn SalesContractRow(c: Value) -> Element {
    let mut show_detail = use_signal(|| false);
    let mut detail_result = use_signal(|| None::<Result<Value, String>>);
    let mut verifying = use_signal(|| false);
    let mut verify_msg = use_signal(|| None::<String>);
    let mut invoicing = use_signal(|| false);
    let mut invoice_msg = use_signal(|| None::<String>);

    let cid = c["id"].as_str().unwrap_or("").to_string();
    let status = c["status"].as_str().unwrap_or("draft").to_string();
    let total = c["total_value"].as_f64().unwrap_or(0.0);
    let prospect_name = c["prospect_name"].as_str().unwrap_or("-").to_string();

    rsx! {
        div { class: "contract-card",
            div { class: "contract-status-{status}", "{status}" }
            div { style: "flex: 1;",
                div { class: "alert-name", "{prospect_name}" }
                div { class: "alert-detail", "Valor: ${total:.0} | ID: {cid}" }
            }
            div { class: "contract-actions",
                button {
                    class: "btn btn-sm",
                    onclick: {
                        let cid = cid.clone();
                        move |_| {
                            show_detail.set(!show_detail());
                            if !show_detail() {
                                detail_result.set(None);
                            } else {
                                let cid = cid.clone();
                                spawn(async move {
                                    detail_result.set(Some(client::get_sales_contract(&cid).await));
                                });
                            }
                        }
                    },
                    if show_detail() { "Ocultar Detalle" } else { "Detalle" }
                }
                button {
                    class: "btn btn-sm",
                    disabled: verifying(),
                    onclick: {
                        let cid = cid.clone();
                        move |_| {
                            verifying.set(true);
                            let cid = cid.clone();
                            spawn(async move {
                                match client::verify_contract_signatures(&cid).await {
                                    Ok(resp) => verify_msg.set(Some(resp["status"].as_str().unwrap_or("Verificado").to_string())),
                                    Err(e) => verify_msg.set(Some(format!("Error: {e}"))),
                                }
                                verifying.set(false);
                            });
                        }
                    },
                    if verifying() { "Verificando..." } else { "Verificar Firmas" }
                }
                button {
                    class: "btn btn-sm",
                    disabled: invoicing(),
                    onclick: {
                        let cid = cid.clone();
                        move |_| {
                            invoicing.set(true);
                            let cid = cid.clone();
                            spawn(async move {
                                match client::generate_contract_invoice(&cid).await {
                                    Ok(resp) => invoice_msg.set(Some(resp["url"].as_str().unwrap_or("Factura generada").to_string())),
                                    Err(e) => invoice_msg.set(Some(format!("Error: {e}"))),
                                }
                                invoicing.set(false);
                            });
                        }
                    },
                    if invoicing() { "Generando..." } else { "Generar Factura" }
                }
            }
            if show_detail() {
                match detail_result() {
                    Some(Ok(ref data)) => {
                        let d_status = data["status"].as_str().unwrap_or("-").to_string();
                        let d_val = data["total_value"].as_f64().unwrap_or(0.0);
                        let d_tax = data["tax_rate"].as_f64().unwrap_or(0.0) * 100.0;
                        let d_notes = data["notes"].as_str().unwrap_or("-").to_string();
                        rsx! {
                            div { class: "detail-section", style: "margin-top: 8px; padding: 8px; background: #f9fafb; border-radius: 6px;",
                                div { class: "detail-row", label { "Estado:" } span { "{d_status}" } }
                                div { class: "detail-row", label { "Valor Total:" } span { "${d_val:.0}" } }
                                div { class: "detail-row", label { "Impuesto:" } span { "{d_tax:.0}%" } }
                                div { class: "detail-row", label { "Notas:" } span { "{d_notes}" } }
                            }
                        }
                    }
                    Some(Err(e)) => rsx! { div { class: "text-error", style: "font-size: 12px; margin-top: 8px;", "Error: {e}" } },
                    None => rsx! { div { class: "loading-spinner", style: "margin-top: 8px;", "Cargando..." } },
                }
            }
            if let Some(ref msg) = verify_msg() {
                div { class: "text-success", style: "font-size: 12px; margin-top: 4px;", "{msg}" }
            }
            if let Some(ref msg) = invoice_msg() {
                div { class: "text-success", style: "font-size: 12px; margin-top: 4px;", "{msg}" }
            }
        }
    }
}

// ─── Documents Tab ───

#[component]
fn SalesDocuments() -> Element {
    let contracts = use_resource(client::fetch_sales_proposals);
    let mut sel_contract_id = use_signal(String::new);
    let mut doc_type = use_signal(|| "contract".to_string());
    let mut file_name = use_signal(String::new);
    let mut file_url = use_signal(String::new);
    let mut saving = use_signal(|| false);

    let documents = use_resource(move || {
        let cid = sel_contract_id();
        async move {
            if cid.is_empty() {
                return Ok(serde_json::json!({"documents": []}));
            }
            client::fetch_contract_documents(&cid).await
        }
    });

    rsx! {
        div { class: "page-toolbar",
            h3 { "Visor de Documentos" }
            p { class: "text-muted", style: "font-size: 13px;", "Gestión de documentos de contratos: subir, listar y verificar" }
        }
        div { class: "form-card",
            div { class: "form-row",
                div { class: "form-group",
                    label { "Seleccionar Contrato:" }
                    match contracts() {
                        Some(Ok(data)) => {
                            let list = data["proposals"].as_array().cloned().unwrap_or_default();
                            let opts: Vec<Element> = list.iter().map(|p| {
                                let pid = p["prospect_id"].as_str().unwrap_or("").to_string();
                                let pname = format!("{} {} — ${:.0}", p["first_name"].as_str().unwrap_or(""), p["last_name"].as_str().unwrap_or(""), p["total_value"].as_f64().unwrap_or(0.0));
                                rsx! { option { value: "{pid}", "{pname}" } }
                            }).collect();
                            rsx! {
                                select { class: "form-input", value: "{sel_contract_id}", oninput: move |e| sel_contract_id.set(e.value()),
                                    option { value: "", "Seleccionar..." }
                                    {opts.into_iter()}
                                }
                            }
                        }
                        _ => rsx! { div { class: "loading-spinner", "Cargando..." } }
                    }
                }
            }
        }
        div { class: "form-card",
            h3 { "Subir Documento" }
            div { class: "form-row",
                div { class: "form-group",
                    label { "Tipo:" }
                    select { class: "form-input", value: "{doc_type}", oninput: move |e| doc_type.set(e.value()),
                        option { value: "contract", "Contrato firmado" }
                        option { value: "identification", "Identificación" }
                        option { value: "tax", "Documento tributario" }
                        option { value: "annex", "Anexo" }
                        option { value: "other", "Otro" }
                    }
                }
                div { class: "form-group",
                    label { "Nombre del archivo:" }
                    input { class: "form-input", value: "{file_name}", placeholder: "ej. contrato_firmado.pdf", oninput: move |e| file_name.set(e.value()) }
                }
            }
            div { class: "form-group",
                label { "URL del archivo:" }
                input { class: "form-input", value: "{file_url}", placeholder: "https://storage.example.com/documento.pdf", oninput: move |e| file_url.set(e.value()) }
            }
            div { class: "form-actions",
                button {
                    class: "btn btn-primary",
                    disabled: saving() || sel_contract_id().trim().is_empty() || file_name().trim().is_empty(),
                    onclick: move |_| {
                        saving.set(true);
                        let cid = sel_contract_id();
                        let payload = serde_json::json!({
                            "file_name": file_name(),
                            "file_url": file_url(),
                            "doc_type": doc_type(),
                        });
                        spawn(async move {
                            let _ = client::upload_contract_document(&cid, &payload).await;
                            saving.set(false);
                            file_name.set(String::new());
                            file_url.set(String::new());
                        });
                    },
                    if saving() { "Subiendo..." } else { "Subir Documento" }
                }
            }
        }
        div { class: "widget-card",
            div { class: "widget-card-header",
                h3 { "Documentos del Contrato" }
                span { }
            }
            div { class: "widget-card-body",
                match documents() {
                    Some(Ok(data)) => {
                        let docs = data["documents"].as_array().cloned().unwrap_or_default();
                        if docs.is_empty() {
                            rsx! { div { class: "empty-state", "Seleccione un contrato para ver sus documentos" } }
                        } else {
                            rsx! {
                                div { class: "data-table-container",
                                    table { class: "data-table",
                                        thead {
                                            tr { th { "Nombre" } th { "Tipo" } th { "Verificado" } th { "Subido por" } th { "Fecha" } th { "Acción" } }
                                        }
                                        tbody {
                                            {docs.iter().map(|doc| {
                                                let doc_id = doc["id"].as_str().unwrap_or("");
                                                let name = doc["file_name"].as_str().unwrap_or("");
                                                let dtype = doc["doc_type"].as_str().unwrap_or("");
                                                let verified = doc["is_verified"].as_bool().unwrap_or(false);
                                                let uploader = doc["uploaded_by"].as_str().unwrap_or("-");
                                                let created = doc["created_at"].as_str().unwrap_or("");
                                                let url = doc["file_url"].as_str().unwrap_or("");
                                                rsx! {
                                                    tr {
                                                        key: "{doc_id}",
                                                        td { "{name}" }
                                                        td { span { class: "badge", "{dtype}" } }
                                                        td { if verified { span { class: "badge badge-success", "✅ Verificado" } } else { span { class: "badge badge-warning", "⏳ Pendiente" } } }
                                                        td { "{uploader}" }
                                                        td { "{created}" }
                                                        td {
                                                            if !url.is_empty() {
                                                                a { class: "btn btn-sm", href: "{url}", target: "_blank", "Ver" }
                                                            }
                                                        }
                                                    }
                                                }
                                            })}
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => rsx! { div { class: "loading-spinner", "Cargando..." } }
                }
            }
        }
    }
}

// ─── Team Tab ───

#[component]
fn SalesTeam() -> Element {
    let agents_data = use_resource(|| client::fetch_json("/b2b/sales/agents"));
    let rr_status = use_resource(|| client::fetch_json("/b2b/sales/round-robin/status"));

    let agents_list: Vec<Value> = match agents_data() {
        Some(Ok(d)) => d["agents"].as_array().cloned().unwrap_or_default(),
        _ => vec![],
    };

    let rr_active = match rr_status() {
        Some(Ok(d)) => d["active"].as_bool().unwrap_or(false),
        _ => false,
    };

    rsx! {
        div { class: "page-toolbar",
            h3 { "Equipo de Ventas" }
        }
        div { class: "dashboard-section",
            h3 { "Asignaci\u{00f3}n Autom\u{00e1}tica (Round-Robin)" }
            KpiCard { label: "Round-Robin".to_string(), value: (if rr_active { "Activado" } else { "Desactivado" }).to_string() }
        }
        div { class: "data-table-container",
            table { class: "data-table",
                thead {
                    tr { th { "Agente" } th { "Email" } th { "Meta Mensual" } th { "Meta Trimestral" } th { "Comisi\u{00f3}n" } th { "Activo" } }
                }
                tbody {
                    {agents_list.iter().map(|agent_entry| {
                        let agent = &agent_entry["agent"];
                        let user = &agent_entry["user"];
                        let name = user["name"].as_str().unwrap_or("-").to_string();
                        let email = user["email"].as_str().unwrap_or("-").to_string();
                        let monthly = agent["quota_monthly"].as_f64().unwrap_or(0.0);
                        let quarterly = agent["quota_quarterly"].as_f64().unwrap_or(0.0);
                        let commission = agent["commission_rate"].as_f64().unwrap_or(0.0);
                        let active = agent["active"].as_bool().unwrap_or(false);
                        rsx! {
                            tr {
                                key: "{name}",
                                td { "{name}" }
                                td { "{email}" }
                                td { "${monthly}" }
                                td { "${quarterly}" }
                                td { "{commission}%" }
                                td { if active { span { class: "badge badge-success", "Activo" } } else { span { class: "badge badge-error", "Inactivo" } } }
                            }
                        }
                    })}
                }
            }
        }
    }
}

fn build_activation_wizard(
    mut show_activate_wizard: Signal<bool>,
    mut wizard_step: Signal<u32>,
    mut activation_result: Signal<Option<Value>>,
    mut is_activating: Signal<bool>,
    company_val: String,
    email_val: String,
    prospect_id: String,
) -> Element {
    if !show_activate_wizard() {
        return rsx! {};
    }
    let ws = wizard_step();
    let ws_label = format!("Paso {} de 3", ws + 1);
    rsx! {
        div { class: "widget-card card-accent-green",
            div { class: "widget-card-header",
                h3 { "Wizard de Activación" }
                span { class: "badge", "{ws_label}" }
            }
            div { class: "widget-card-body",
                if ws == 0 {
                    div { class: "detail-section",
                        h3 { "Resumen de Activación" }
                        p { "Se crearán los siguientes recursos:" }
                        ul { style: "list-style: disc; padding-left: 20px; margin: 12px 0;",
                            li { "Corporación: {company_val}" }
                            li { "Colegio: Colegio {company_val}" }
                            li { "Usuario Administrador: {email_val}" }
                            li { "Licencia activa vinculada al plan" }
                        }
                        div { class: "form-actions",
                            button { class: "btn btn-primary", onclick: move |_| wizard_step.set(1), "Continuar" }
                            button { class: "btn btn-secondary", style: "margin-left: 8px;", onclick: move |_| show_activate_wizard.set(false), "Cancelar" }
                        }
                    }
                } else if ws == 1 {
                    div { class: "detail-section",
                        h3 { "Confirmar Activación" }
                        p { "¿Estás seguro de activar la licencia para {company_val}?" }
                        p { class: "text-muted", style: "font-size: 13px;", "Esta acción creará la corporación, el colegio y el usuario administrador." }
                        div { class: "form-actions",
                            button {
                                class: "btn btn-success",
                                disabled: is_activating(),
                                onclick: move |_| {
                                    let id = prospect_id.clone();
                                    spawn(async move {
                                        is_activating.set(true);
                                        if let Ok(resp) = client::post_json(&format!("/b2b/sales/contracts/{}/activate", id), &json!({})).await {
                                            activation_result.set(Some(resp));
                                            wizard_step.set(2);
                                        }
                                        is_activating.set(false);
                                    });
                                },
                                if is_activating() { "Activando..." } else { "Confirmar y Activar" }
                            }
                            button { class: "btn btn-secondary", style: "margin-left: 8px;", onclick: move |_| wizard_step.set(0), "Atrás" }
                        }
                    }
                } else if ws == 2 {
                    match activation_result() {
                        Some(ref data) => {
                            let admin_email = data["admin_email"].as_str().unwrap_or("").to_string();
                            let temp_password = data["temp_password"].as_str().unwrap_or("").to_string();
                            rsx! {
                                div { class: "detail-section",
                                    h3 { "Licencia Activada" }
                                    p { "La corporación, colegio y usuario administrador han sido creados." }
                                    div { class: "info-card", style: "margin: 16px 0;",
                                        div { class: "detail-row", label { "Email:" } span { "{admin_email}" } }
                                        div { class: "detail-row", label { "Contraseña Temporal:" } span { class: "font-mono", "{temp_password}" } }
                                    }
                                    p { class: "text-muted", style: "font-size: 12px;", "Comparte estas credenciales con el sostenedor." }
                                    div { class: "form-actions",
                                        button { class: "btn btn-primary", onclick: move |_| { show_activate_wizard.set(false); activation_result.set(None); }, "Finalizar" }
                                    }
                                }
                            }
                        }
                        None => rsx! { div { class: "loading-spinner", "Procesando..." } },
                    }
                }
            }
        }
    }
}
