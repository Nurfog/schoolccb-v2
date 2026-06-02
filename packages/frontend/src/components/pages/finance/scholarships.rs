use dioxus::prelude::*;

use crate::api::client;
use crate::components::widgets::student_search::StudentSearchSelect;

#[component]
pub fn ScholarshipsTab() -> Element {
    let mut scholarships = use_resource(|| client::fetch_all_scholarships());
    let mut show_form = use_signal(|| false);
    let mut student_id = use_signal(|| String::new());
    let mut form_key = use_signal(|| 0u32);
    let mut name = use_signal(|| String::new());
    let mut discount = use_signal(|| String::new());
    let mut valid_from = use_signal(|| String::new());
    let mut valid_until = use_signal(|| String::new());
    let mut saving = use_signal(|| false);

    let mut reset_form = move || {
        form_key += 1;
        student_id.set(String::new());
        name.set(String::new());
        discount.set(String::new());
        valid_from.set(String::new());
        valid_until.set(String::new());
        show_form.set(false);
    };

    let do_save = move |_| {
        if student_id().is_empty() || name().is_empty() || discount().is_empty() {
            return;
        }
        saving.set(true);
        let payload = serde_json::json!({
            "student_id": student_id(),
            "name": name(),
            "discount_percentage": discount().parse::<f64>().unwrap_or(0.0),
            "valid_from": valid_from(),
            "valid_until": valid_until(),
        });
        spawn(async move {
            let _ = client::create_scholarship(&payload).await;
            saving.set(false);
            reset_form();
            scholarships.restart();
        });
    };

    let do_approve = move |sid: String| {
        spawn(async move {
            let _ = client::approve_scholarship(&sid).await;
            scholarships.restart();
        });
    };

    let do_delete = move |sid: String| {
        if !web_sys::window().unwrap().confirm_with_message("¿Estás seguro?").unwrap_or(false) {
            return;
        }
        spawn(async move {
            let _ = client::delete_scholarship(&sid).await;
            scholarships.restart();
        });
    };

    rsx! {
        div { class: "toolbar-row",
            button { class: "btn btn-primary", onclick: move |_| show_form.set(!show_form()), if show_form() { "Cancelar" } else { "Nueva Beca" } }
        }
        {
            if show_form() {
                rsx! {
                    div { class: "form-card",
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Estudiante:" }
                                StudentSearchSelect {
                                    on_select: move |id| student_id.set(id),
                                    reset_key: Some(form_key().to_string()),
                                }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Nombre Beca:" }
                                input { class: "form-input", value: "{name}", oninput: move |evt| name.set(evt.value()), placeholder: "Ej: Beca Excelencia", "aria-required": "true", autocomplete: "off" }
                            }
                            div { class: "form-group",
                                label { "% Descuento:" }
                                input { class: "form-input", value: "{discount}", oninput: move |evt| discount.set(evt.value()), type: "number", min: "1", max: "100", placeholder: "0" }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Vigencia desde:" }
                                input { class: "form-input", value: "{valid_from}", oninput: move |evt| valid_from.set(evt.value()), type: "date" }
                            }
                            div { class: "form-group",
                                label { "Vigencia hasta:" }
                                input { class: "form-input", value: "{valid_until}", oninput: move |evt| valid_until.set(evt.value()), type: "date" }
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
                match scholarships() {
                    Some(Ok(j)) => {
                        let list: Vec<(String, String, String, String, String, String, bool)> = j["student_scholarships"].as_array().map(|arr| {
                            arr.iter().map(|s| {
                                let disc = s["discount_percentage"].as_f64().unwrap_or(0.0);
                                let disc_display = format!("{:.0}%", disc);
                                (
                                    s["id"].as_str().unwrap_or("").to_string(),
                                    s["student_id"].as_str().unwrap_or("-").to_string(),
                                    s["name"].as_str().unwrap_or("").to_string(),
                                    disc_display,
                                    s["valid_from"].as_str().unwrap_or("").to_string(),
                                    s["valid_until"].as_str().unwrap_or("").to_string(),
                                    s["approved"].as_bool().unwrap_or(false),
                                )
                            }).collect()
                        }).unwrap_or_default();
                        rsx! {
                            table { class: "data-table",
                                thead { tr {
                                    th { "Estudiante" }
                                    th { "Nombre" }
                                    th { "% Descuento" }
                                    th { "Vigencia" }
                                    th { "Estado" }
                                    th { "Acciones" }
                                }}
                                tbody { for (sid, stu_id, sname, disc_display, vfrom, vuntil, approved) in &list {
                                    tr {
                                        td { "{stu_id}" }
                                        td { "{sname}" }
                                        td { "{disc_display}" }
                                        td { "{vfrom} - {vuntil}" }
                                        td { if *approved { span { class: "grade-good", "Aprobada" } } else { span { class: "grade-bad", "Pendiente" } } }
                                        td {
                                            if !approved {
                                                button { class: "btn btn-sm btn-success", onclick: { let id = sid.clone(); move |_| do_approve(id.clone()) }, "Aprobar" }
                                            }
                                            button { class: "btn btn-sm btn-danger", style: "margin-left: 4px;", onclick: { let id = sid.clone(); move |_| do_delete(id.clone()) }, "Eliminar" }
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
