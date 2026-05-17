use dioxus::prelude::*;

use crate::components::widgets::icon::Icon;
use serde_json::Value;

#[derive(Clone)]
struct StepInfo {
    stage_id: String,
    name: String,
    is_completed: bool,
    is_current: bool,
    is_final: bool,
    is_last: bool,
}

#[component]
pub fn BusinessProcessFlow(stages: Vec<Value>, current_stage_id: String) -> Element {
    let current_idx = stages
        .iter()
        .position(|s| s["id"].as_str().unwrap_or("") == current_stage_id);

    let total = stages.len();

    let steps: Vec<StepInfo> = stages
        .iter()
        .enumerate()
        .map(|(i, s)| StepInfo {
            stage_id: s["id"].as_str().unwrap_or("").to_string(),
            name: s["name"].as_str().unwrap_or("").to_string(),
            is_completed: current_idx.map(|idx| i < idx).unwrap_or(false),
            is_current: current_idx.map(|idx| i == idx).unwrap_or(false),
            is_final: s["is_final"].as_bool().unwrap_or(false),
            is_last: i == total - 1,
        })
        .collect();

    let steps_with_keys: Vec<(String, StepInfo)> =
        steps.into_iter().map(|s| (s.stage_id.clone(), s)).collect();

    rsx! {
        div { class: "bpf-container",
            div { class: "bpf-stages",
                for (key, step) in steps_with_keys {
                    BpfStep {
                        key: "{key}",
                        stage_id: step.stage_id,
                        name: step.name,
                        is_completed: step.is_completed,
                        is_current: step.is_current,
                        is_final: step.is_final,
                        is_last: step.is_last,
                    }
                }
            }
        }
    }
}

#[component]
fn BpfStep(
    stage_id: String,
    name: String,
    is_completed: bool,
    is_current: bool,
    is_final: bool,
    is_last: bool,
) -> Element {
    let circle_class = if is_current {
        "bpf-circle current"
    } else if is_completed {
        "bpf-circle completed"
    } else {
        "bpf-circle pending"
    };

    let label_class = if is_current {
        "bpf-label current"
    } else if is_completed {
        "bpf-label completed"
    } else {
        "bpf-label pending"
    };

    let icon = if is_completed {
        rsx! { Icon { name: "check", size: 12 } }
    } else if is_current {
        rsx! { span { "●" } }
    } else {
        rsx! { span { style: "opacity:0.3", "●" } }
    };

    rsx! {
        div { class: "bpf-step",
            div { class: "{circle_class}", {icon} }
            {
                if !is_last {
                    let line_class = if is_final { "bpf-step-line final-step-line" } else if is_completed { "bpf-step-line completed" } else { "bpf-step-line pending" };
                    rsx! { div { class: "{line_class}" } }
                } else {
                    rsx! {}
                }
            }
            div { class: "{label_class}", "{name}" }
        }
    }
}
