use dioxus::prelude::*;

#[component]
pub fn KpiCard(
    label: String,
    value: String,
    color: Option<String>,
    icon: Option<String>,
    large: Option<bool>,
) -> Element {
    let color_class = match color.as_deref() {
        Some("#66bb6a" | "#16a34a" | "#22c55e") => "kpi-value success",
        Some("#ff7043" | "#ffa726" | "#f59e0b") => "kpi-value warning",
        Some("#ab47bc" | "#26c6da" | "#06b6d4" | "#3b82f6") => "kpi-value info",
        _ => "kpi-value primary",
    };
    let size_class = if large.unwrap_or(false) { " kpi-lg" } else { "" };
    rsx! {
        div { class: "kpi-card{size_class}",
            if let Some(ic) = icon {
                div { class: "kpi-icon", "{ic}" }
            }
            div { class: "{color_class}", "{value}" }
            div { class: "kpi-label", "{label}" }
        }
    }
}
