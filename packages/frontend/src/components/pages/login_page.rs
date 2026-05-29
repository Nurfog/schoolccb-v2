use dioxus::prelude::*;

use crate::api::client;

#[component]
pub fn LoginPage() -> Element {
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);
    let error_id = "login-error";

    let mut do_login = move |_| {
        if email().is_empty() || password().is_empty() {
            error.set(Some("Email y contraseña son obligatorios".to_string()));
            return;
        }
        loading.set(true);
        error.set(None);
        let e = email();
        let p = password();
        let nav = navigator();
        spawn(async move {
            match client::login(&e, &p).await {
                Ok(resp) => {
                    if resp.get("token").is_some() {
                        nav.replace("/");
                    } else {
                        let msg = resp
                            .get("error")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Credenciales inválidas");
                        loading.set(false);
                        error.set(Some(msg.to_string()));
                    }
                }
                Err(e) => {
                    loading.set(false);
                    error.set(Some(e));
                }
            }
        });
    };

    rsx! {
        div { class: "login-container",
            div { class: "login-card",
                div { class: "login-header",
                    div { class: "login-logo", "SC" }
                    h1 { "SchoolCBB" }
                    p { "Plataforma Escolar" }
                }
                form { class: "login-form", onsubmit: move |e| { e.prevent_default(); do_login(e); },
                    if let Some(ref msg) = error() {
                        div { id: "{error_id}", class: "login-error", role: "alert", aria_live: "assertive", "{msg}" }
                    }
                    div { class: "field",
                        label { r#for: "login-email", "Email" }
                        input {
                            id: "login-email",
                            class: "login-input",
                            "type": "email",
                            placeholder: "email@colegio.cl",
                            value: "{email}",
                            oninput: move |evt| email.set(evt.value()),
                            "aria-required": "true",
                            autocomplete: "email",
                            "aria-describedby": error().is_some().then_some(error_id),
                        }
                    }
                    div { class: "field",
                        label { r#for: "login-password", "Contraseña" }
                        input {
                            id: "login-password",
                            class: "login-input",
                            "type": "password",
                            placeholder: "contraseña",
                            value: "{password}",
                            oninput: move |evt| password.set(evt.value()),
                            "aria-required": "true",
                            autocomplete: "current-password",
                            "aria-describedby": error().is_some().then_some(error_id),
                        }
                    }
                    button {
                        class: "login-btn",
                        r#type: "submit",
                        disabled: loading(),
                        "aria-busy": "{loading}",
                        if loading() { "Ingresando..." } else { "Iniciar Sesión" }
                    }
                }
            }
        }
    }
}
