use dioxus::prelude::*;
use serde_json::{Value, json};

use crate::api::client;
use crate::seo::use_page_title;

#[component]
pub fn EmailConfigPage() -> Element {
    use_page_title("Configuración Email");
    let providers = use_resource(|| client::fetch_json("/api/email/providers"));
    let queue = use_resource(|| client::fetch_json("/api/email/queue"));

    let mut show_form = use_signal(|| false);
    let mut smtp_host = use_signal(String::new);
    let mut smtp_port = use_signal(|| "587".to_string());
    let mut smtp_user = use_signal(String::new);
    let mut smtp_pass = use_signal(String::new);
    let mut from_email = use_signal(String::new);
    let mut from_name = use_signal(String::new);
    let mut reply_to = use_signal(String::new);
    let mut max_daily = use_signal(|| "500".to_string());
    let mut provider_type = use_signal(|| "smtp".to_string());
    let mut saving = use_signal(|| false);
    let mut test_result = use_signal(|| None::<String>);
    let mut test_to = use_signal(String::new);

    let provider_list: Vec<Value> = match providers() {
        Some(Ok(ref d)) => d["providers"].as_array().cloned().unwrap_or_default(),
        _ => vec![],
    };

    let queue_items: Vec<Value> = match queue() {
        Some(Ok(ref d)) => d["queue"].as_array().cloned().unwrap_or_default(),
        _ => vec![],
    };

    let do_save = move |_| {
        saving.set(true);
        let payload = json!({
            "provider_type": provider_type(),
            "smtp_host": smtp_host(),
            "smtp_port": smtp_port().parse::<i32>().unwrap_or(587),
            "smtp_username": smtp_user(),
            "smtp_password": smtp_pass(),
            "from_email": from_email(),
            "from_name": from_name(),
            "reply_to": reply_to(),
            "max_daily_sends": max_daily().parse::<i32>().unwrap_or(500),
        });
        spawn(async move {
            let _ = client::post_json("/api/email/providers", &payload).await;
            saving.set(false);
            show_form.set(false);
            providers.restart();
        });
    };

    let do_test = move |provider_id: String| {
        let to = test_to();
        let payload = json!({"to": to});
        spawn(async move {
            let resp = client::post_json(&format!("/api/email/providers/{}/test", provider_id), &payload).await;
            test_result.set(Some(match resp {
                Ok(_) => "Email de prueba enviado correctamente".to_string(),
                Err(e) => format!("Error: {e}"),
            }));
        });
    };

    let batch_status = |batch_id: &str| -> String {
        let _count = queue_items.iter().filter(|i| i["batch_id"].as_str() == Some(batch_id)).count();
        let sent = queue_items.iter().filter(|i| i["batch_id"].as_str() == Some(batch_id) && i["status"] == "sent").count();
        let failed = queue_items.iter().filter(|i| i["batch_id"].as_str() == Some(batch_id) && i["status"] == "failed").count();
        format!("{sent} enviados, {failed} fallidos")
    };

    rsx! {
        div { class: "page-header",
            h1 { "Configuración de Correo Electrónico" }
            p { "Gestiona proveedores SMTP para envío de emails a clientes, apoderados y alumnos" }
        }

        div { class: "tabs",
            button { class: "tab active", "Proveedores SMTP" }
        }

        div { class: "page-toolbar",
            button { class: "btn btn-primary", onclick: move |_| show_form.set(!show_form()),
                if show_form() { "Cancelar" } else { "Nuevo Proveedor" }
            }
        }

        if show_form() {
            div { class: "form-card",
                h3 { "Configurar Proveedor SMTP" }
                div { class: "form-group",
                    label { "Tipo de Proveedor" }
                    select { class: "form-input", value: "{provider_type}",
                        onchange: move |e| provider_type.set(e.value()),
                        option { value: "smtp", "SMTP (Gmail, Outlook, etc.)" }
                        option { value: "sendgrid", "SendGrid" }
                        option { value: "mailgun", "Mailgun" }
                        option { value: "ses", "AWS SES" }
                    }
                }
                div { class: "form-row",
                    div { class: "form-group",
                        label { "Servidor SMTP *" }
                        input { class: "form-input", placeholder: "smtp.gmail.com", value: "{smtp_host}", oninput: move |e| smtp_host.set(e.value()) }
                    }
                    div { class: "form-group",
                        label { "Puerto" }
                        input { class: "form-input", value: "{smtp_port}", oninput: move |e| smtp_port.set(e.value()) }
                    }
                }
                div { class: "form-row",
                    div { class: "form-group",
                        label { "Usuario" }
                        input { class: "form-input", placeholder: "correo@gmail.com", value: "{smtp_user}", oninput: move |e| smtp_user.set(e.value()) }
                    }
                    div { class: "form-group",
                        label { "Contraseña" }
                        input { class: "form-input", r#type: "password", placeholder: "••••••••", value: "{smtp_pass}", oninput: move |e| smtp_pass.set(e.value()) }
                    }
                }
                div { class: "form-row",
                    div { class: "form-group",
                        label { "Email Desde *" }
                        input { class: "form-input", placeholder: "ventas@colegio.cl", value: "{from_email}", oninput: move |e| from_email.set(e.value()) }
                    }
                    div { class: "form-group",
                        label { "Nombre Desde" }
                        input { class: "form-input", placeholder: "Colegio SchoolCBB", value: "{from_name}", oninput: move |e| from_name.set(e.value()) }
                    }
                }
                div { class: "form-row",
                    div { class: "form-group",
                        label { "Reply-To" }
                        input { class: "form-input", placeholder: "noreply@colegio.cl", value: "{reply_to}", oninput: move |e| reply_to.set(e.value()) }
                    }
                    div { class: "form-group",
                        label { "Límite diario" }
                        input { class: "form-input", value: "{max_daily}", oninput: move |e| max_daily.set(e.value()) }
                    }
                }
                div { class: "provider-info",
                    p { class: "text-sm text-gray-500",
                        "Puedes usar cualquier proveedor SMTP: ",
                        b { "Gmail" }, " (smtp.gmail.com:587), ",
                        b { "Outlook" }, " (smtp-mail.outlook.com:587), ",
                        b { "Office 365" }, " (smtp.office365.com:587), ",
                        b { "Zoho" }, " (smtp.zoho.com:587), ",
                        "o tu servidor corporativo."
                    }
                }
                div { class: "form-row",
                    button { class: "btn btn-primary", disabled: saving(), onclick: do_save,
                        if saving() { "Guardando..." } else { "Guardar Proveedor" }
                    }
                }
            }
        }

        div { class: "dashboard-section",
            h3 { "Proveedores Configurados" }
            if provider_list.is_empty() {
                div { class: "empty-state", "No hay proveedores configurados. Usa el botón \"Nuevo Proveedor\" para agregar uno." }
            } else {
                div { class: "data-table-container",
                    table { class: "data-table",
                        thead { tr { th { "Email" } th { "Proveedor" } th { "Host" } th { "Estado" } th { "Envíos Hoy" } th { "Acción" } } }
                        tbody {
                            {provider_list.iter().map(|p| {
                                let pid = p["id"].as_str().unwrap_or("").to_string();
                                let email = p["from_email"].as_str().unwrap_or("").to_string();
                                let ptype = p["provider_type"].as_str().unwrap_or("smtp").to_string();
                                let host = p["smtp_host"].as_str().unwrap_or("").to_string();
                                let verified = p["is_verified"].as_bool().unwrap_or(false);
                                let active = p["is_active"].as_bool().unwrap_or(false);
                                let sent = p["sent_today"].as_i64().unwrap_or(0);
                                let max_s = p["max_daily_sends"].as_i64().unwrap_or(500);
                                let pid_clone = pid.clone();
                                rsx! {
                                    tr { key: "{pid}",
                                        td { "{email}" }
                                        td { "{ptype}" }
                                        td { "{host}" }
                                        td {
                                            if !active { span { class: "badge badge-error", "Inactivo" } }
                                            else if verified { span { class: "badge badge-success", "Verificado" } }
                                            else { span { class: "badge badge-warning", "Sin verificar" } }
                                        }
                                        td { "{sent}/{max_s}" }
                                        td {
                                            button { class: "btn btn-sm", onclick: move |_| do_test(pid_clone.clone()), "Probar" }
                                        }
                                    }
                                }
                            })}
                        }
                    }
                }
            }
        }

        if let Some(ref result) = test_result() {
            div { class: "alert alert-info", "{result}" }
        }

        div { class: "dashboard-section",
            h3 { "Bandeja de Envíos" }
            if queue_items.is_empty() {
                div { class: "empty-state", "No hay envíos recientes" }
            } else {
                div { class: "data-table-container",
                    table { class: "data-table",
                        thead { tr { th { "Asunto" } th { "Destino" } th { "Estado" } th { "Fecha" } } }
                        tbody {
                            {queue_items.iter().map(|item| {
                                let subject = item["subject"].as_str().unwrap_or("").to_string();
                                let to = item["sender_email"].as_str().unwrap_or("").to_string();
                                let status = item["status"].as_str().unwrap_or("").to_string();
                                let date = item["created_at"].as_str().unwrap_or("").to_string();
                                let batch = item["batch_id"].as_str().map(|b| batch_status(b));
                                rsx! {
                                    tr {
                                        td { "{subject}" }
                                        td { "{to}" }
                                        td { span { class: "badge badge-{status}", "{status}" } }
                                        td { "{date}" }
                                    }
                                }
                            })}
                        }
                    }
                }
            }
        }
    }
}
