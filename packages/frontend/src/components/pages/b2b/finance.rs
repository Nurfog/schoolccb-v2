use dioxus::prelude::*;
use crate::api::client;

#[component]
pub fn B2bFinancePage() -> Element {
    let dashboard = use_resource(|| client::fetch_json("/api/sales/dashboard/summary"));

    rsx! {
        div { class: "page-header",
            h1 { "Finanzas B2B" }
            p { "Resumen financiero del área corporativa y comercial" }
        }
        div { class: "dashboard-grid",
            match dashboard() {
                Some(Ok(data)) => {
                    let total_value = data["total_value"].as_f64().unwrap_or(0.0);
                    let total_contracts = data["total_contracts"].as_i64().unwrap_or(0);
                    let total_prospects = data["total_prospects"].as_i64().unwrap_or(0);
                    rsx! {
                        div { class: "kpi-card",
                            div { class: "kpi-value", "${total_value:.0}" }
                            div { class: "kpi-label", "Valor Total Contratos" }
                        }
                        div { class: "kpi-card",
                            div { class: "kpi-value", "{total_contracts}" }
                            div { class: "kpi-label", "Contratos Activos" }
                        }
                        div { class: "kpi-card",
                            div { class: "kpi-value", "{total_prospects}" }
                            div { class: "kpi-label", "Prospectos" }
                        }
                    }
                }
                Some(Err(e)) => rsx! { div { class: "empty-state", "Error: {e}" } },
                None => rsx! { div { class: "empty-state", div { class: "loading-spinner", "Cargando..." } } },
            }
        }
    }
}
