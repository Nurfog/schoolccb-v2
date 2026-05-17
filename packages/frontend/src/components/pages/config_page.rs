use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client;

fn jwt_claims() -> Option<Value> {
    let window = web_sys::window()?;
    let store = window.local_storage().ok().flatten()?;
    let token = store.get_item("jwt_token").ok()?;
    let token = token?;
    let parts: Vec<&str> = token.split('.').collect();
    let payload_b64 = parts.get(1)?;
    let decoded = window.atob(payload_b64).ok()?;
    serde_json::from_str(&decoded).ok()
}

fn has_role(claims: &Option<Value>, roles: &[&str]) -> bool {
    claims
        .as_ref()
        .and_then(|c| c["role"].as_str())
        .map_or(false, |r| roles.contains(&r))
}

#[component]
pub fn ConfigPage() -> Element {
    let claims = use_signal(jwt_claims);
    let _role = claims()
        .as_ref()
        .and_then(|c| c["role"].as_str())
        .unwrap_or("")
        .to_string();
    let user_id = claims()
        .as_ref()
        .and_then(|c| c["sub"].as_str())
        .unwrap_or("")
        .to_string();

    let profile = use_resource(move || async move { client::fetch_json("/api/auth/me").await });

    let prefs =
        use_resource(move || async move { client::fetch_json("/api/user/preferences").await });

    let branding = use_resource(move || async move {
        if has_role(&claims(), &["Sostenedor"]) {
            client::fetch_json("/api/config/branding").await
        } else {
            Err("no_access".into())
        }
    });

    rsx! {
        div { class: "page-header",
            h1 { "Configuración" }
            p { "Administra tu perfil y preferencias" }
        }
        div { class: "config-grid",
            ProfileSection { profile: profile, user_id: user_id.clone() }
            PasswordSection {}
            PreferencesSection { prefs: prefs }
            if has_role(&claims(), &["Sostenedor"]) {
                BrandingSection { branding: branding }
                CreateAdminSection {}
            }
        }
    }
}

#[component]
fn ProfileSection(profile: Resource<Result<Value, String>>, user_id: String) -> Element {
    let mut name = use_signal(|| "".to_string());
    let mut email = use_signal(|| "".to_string());
    let mut rut = use_signal(|| "".to_string());
    let mut saving = use_signal(|| false);
    let mut saved = use_signal(|| false);
    let mut error = use_signal(|| "".to_string());

    let profile_data = profile().and_then(|r| r.ok());
    if let Some(data) = profile_data {
        if name().is_empty() && email().is_empty() {
            name.set(data["user"]["name"].as_str().unwrap_or("").to_string());
            email.set(data["user"]["email"].as_str().unwrap_or("").to_string());
            rut.set(data["user"]["rut"].as_str().unwrap_or("").to_string());
        }
    }

    let do_save = move |_| {
        saving.set(true);
        saved.set(false);
        error.set("".to_string());
        let payload = serde_json::json!({
            "name": name(),
            "email": email(),
        });
        spawn(async move {
            match client::put_json("/api/user/profile", &payload).await {
                Ok(_) => {
                    saving.set(false);
                    saved.set(true);
                }
                Err(e) => {
                    saving.set(false);
                    error.set(e);
                }
            }
        });
    };

    rsx! {
        div { class: "config-card",
            h2 { "Mi Perfil" }
            div { class: "config-card-body",
                div { class: "field",
                    label { "RUT" }
                    input { class: "login-input", value: "{rut}", disabled: true }
                }
                div { class: "field",
                    label { "Nombre" }
                    input { class: "login-input", value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                }
                div { class: "field",
                    label { "Email" }
                    input { class: "login-input", value: "{email}",
                        oninput: move |e| email.set(e.value()),
                    }
                }
                if !error().is_empty() {
                    p { class: "login-error", "{error}" }
                }
                if saved() {
                    p { class: "config-success", "Perfil actualizado correctamente" }
                }
                button { class: "login-btn", onclick: do_save, disabled: saving(),
                    if saving() { "Guardando..." } else { "Guardar Cambios" }
                }
            }
        }
    }
}

#[component]
fn PasswordSection() -> Element {
    let mut current = use_signal(|| "".to_string());
    let mut new_pwd = use_signal(|| "".to_string());
    let mut confirm = use_signal(|| "".to_string());
    let mut saving = use_signal(|| false);
    let mut saved = use_signal(|| false);
    let mut error = use_signal(|| "".to_string());

    let do_change = move |_| {
        if new_pwd().len() < 6 {
            error.set("La contraseña debe tener al menos 6 caracteres".into());
            return;
        }
        if new_pwd() != confirm() {
            error.set("Las contraseñas no coinciden".into());
            return;
        }
        saving.set(true);
        saved.set(false);
        error.set("".to_string());
        let payload = serde_json::json!({
            "current_password": current(),
            "new_password": new_pwd(),
        });
        spawn(async move {
            match client::put_json("/api/user/password", &payload).await {
                Ok(_) => {
                    saving.set(false);
                    saved.set(true);
                    current.set("".to_string());
                    new_pwd.set("".to_string());
                    confirm.set("".to_string());
                }
                Err(e) => {
                    saving.set(false);
                    error.set(e);
                }
            }
        });
    };

    rsx! {
        div { class: "config-card",
            h2 { "Cambiar Contraseña" }
            div { class: "config-card-body",
                div { class: "field",
                    label { "Contraseña Actual" }
                    input { class: "login-input", r#type: "password", value: "{current}",
                        oninput: move |e| current.set(e.value()),
                    }
                }
                div { class: "field",
                    label { "Nueva Contraseña" }
                    input { class: "login-input", r#type: "password", value: "{new_pwd}",
                        oninput: move |e| new_pwd.set(e.value()),
                    }
                }
                div { class: "field",
                    label { "Confirmar Nueva Contraseña" }
                    input { class: "login-input", r#type: "password", value: "{confirm}",
                        oninput: move |e| confirm.set(e.value()),
                    }
                }
                if !error().is_empty() {
                    p { class: "login-error", "{error}" }
                }
                if saved() {
                    p { class: "config-success", "Contraseña actualizada correctamente" }
                }
                button { class: "login-btn", onclick: do_change, disabled: saving(),
                    if saving() { "Cambiando..." } else { "Cambiar Contraseña" }
                }
            }
        }
    }
}

#[component]
fn PreferencesSection(prefs: Resource<Result<Value, String>>) -> Element {
    let mut show_mm = use_signal(|| true);
    let mut saving = use_signal(|| false);
    let mut saved = use_signal(|| false);

    if let Some(Ok(data)) = prefs() {
        if let Some(val) = data.get("show_module_manager").and_then(|v| v.as_bool()) {
            if show_mm() == true && *show_mm.peek() != val {
                show_mm.set(val);
            }
        }
    }

    let do_save = move |_| {
        saving.set(true);
        saved.set(false);
        let payload = serde_json::json!({ "show_module_manager": show_mm() });
        spawn(async move {
            let _ = client::put_json("/api/user/preferences", &payload).await;
            saving.set(false);
            saved.set(true);
        });
    };

    rsx! {
        div { class: "config-card",
            h2 { "Personalización" }
            div { class: "config-card-body",
                div { class: "field config-toggle",
                    label { "Mostrar Module Manager al iniciar sesión" }
                    div { class: "toggle-switch",
                        input {
                            r#type: "checkbox",
                            checked: show_mm(),
                            oninput: move |e| show_mm.set(e.checked()),
                        }
                        span { class: "toggle-slider" }
                    }
                }
                if saved() {
                    p { class: "config-success", "Personalización guardada" }
                }
                button { class: "login-btn", onclick: do_save, disabled: saving(),
                    if saving() { "Guardando..." } else { "Guardar" }
                }
            }
        }
    }
}

#[component]
fn BrandingSection(branding: Resource<Result<Value, String>>) -> Element {
    let mut school_name = use_signal(|| "".to_string());
    let mut primary_color = use_signal(|| "#1A2B3C".to_string());
    let mut secondary_color = use_signal(|| "#243B4F".to_string());
    let mut saving = use_signal(|| false);
    let mut saved = use_signal(|| false);
    let mut error = use_signal(|| "".to_string());

    if let Some(Ok(data)) = branding() {
        if school_name().is_empty() {
            school_name.set(data["school_name"].as_str().unwrap_or("").to_string());
            primary_color.set(
                data["primary_color"]
                    .as_str()
                    .unwrap_or("#1A2B3C")
                    .to_string(),
            );
            secondary_color.set(
                data["secondary_color"]
                    .as_str()
                    .unwrap_or("#243B4F")
                    .to_string(),
            );
        }
    }

    let do_save = move |_| {
        saving.set(true);
        saved.set(false);
        error.set("".to_string());
        let payload = serde_json::json!({
            "school_name": school_name(),
            "school_logo_url": "",
            "primary_color": primary_color(),
            "secondary_color": secondary_color(),
        });
        spawn(async move {
            match client::put_json("/api/config/branding", &payload).await {
                Ok(_) => {
                    saving.set(false);
                    saved.set(true);
                }
                Err(e) => {
                    saving.set(false);
                    error.set(e);
                }
            }
        });
    };

    rsx! {
        div { class: "config-card",
            h2 { "Branding del Colegio" }
            div { class: "config-card-body",
                div { class: "field",
                    label { "Nombre del Colegio" }
                    input { class: "login-input", value: "{school_name}",
                        oninput: move |e| school_name.set(e.value()),
                    }
                }
                div { class: "field",
                    label { "Color Primario" }
                    div { class: "color-picker-row",
                        input { r#type: "color", value: "{primary_color}",
                            oninput: move |e| primary_color.set(e.value()),
                        }
                        input { class: "login-input color-hex", value: "{primary_color}",
                            oninput: move |e| primary_color.set(e.value()),
                        }
                    }
                }
                div { class: "field",
                    label { "Color Secundario" }
                    div { class: "color-picker-row",
                        input { r#type: "color", value: "{secondary_color}",
                            oninput: move |e| secondary_color.set(e.value()),
                        }
                        input { class: "login-input color-hex", value: "{secondary_color}",
                            oninput: move |e| secondary_color.set(e.value()),
                        }
                    }
                }
                if !error().is_empty() {
                    p { class: "login-error", "{error}" }
                }
                if saved() {
                    p { class: "config-success", "Branding actualizado correctamente" }
                }
                button { class: "login-btn", onclick: do_save, disabled: saving(),
                    if saving() { "Guardando..." } else { "Guardar Branding" }
                }
            }
        }
    }
}

#[component]
fn CreateAdminSection() -> Element {
    let mut name = use_signal(|| "".to_string());
    let mut email = use_signal(|| "".to_string());
    let mut rut_val = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());
    let mut saving = use_signal(|| false);
    let mut saved = use_signal(|| false);
    let mut error = use_signal(|| "".to_string());

    let do_create = move |_| {
        if name().is_empty() || email().is_empty() || rut_val().is_empty() || password().is_empty()
        {
            error.set("Todos los campos son obligatorios".into());
            return;
        }
        saving.set(true);
        saved.set(false);
        error.set("".to_string());
        let payload = serde_json::json!({
            "rut": rut_val(),
            "name": name(),
            "email": email(),
            "password": password(),
            "role": "Administrador",
        });
        spawn(async move {
            match client::post_json("/api/auth/register", &payload).await {
                Ok(_) => {
                    saving.set(false);
                    saved.set(true);
                    name.set("".to_string());
                    email.set("".to_string());
                    rut_val.set("".to_string());
                    password.set("".to_string());
                }
                Err(e) => {
                    saving.set(false);
                    error.set(e);
                }
            }
        });
    };

    rsx! {
        div { class: "config-card",
            h2 { "Crear Administrador" }
            p { class: "config-hint", "Crea un usuario con rol Administrador para la gestión del sistema" }
            div { class: "config-card-body",
                div { class: "field",
                    label { "RUT" }
                    input { class: "login-input", placeholder: "11.111.111-1", value: "{rut_val}",
                        oninput: move |e| rut_val.set(e.value()),
                    }
                }
                div { class: "field",
                    label { "Nombre" }
                    input { class: "login-input", placeholder: "Nombre completo", value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                }
                div { class: "field",
                    label { "Email" }
                    input { class: "login-input", placeholder: "admin@colegio.cl", value: "{email}",
                        oninput: move |e| email.set(e.value()),
                    }
                }
                div { class: "field",
                    label { "Contraseña" }
                    input { class: "login-input", r#type: "password", placeholder: "Mínimo 6 caracteres", value: "{password}",
                        oninput: move |e| password.set(e.value()),
                    }
                }
                if !error().is_empty() {
                    p { class: "login-error", "{error}" }
                }
                if saved() {
                    p { class: "config-success", "Administrador creado correctamente" }
                }
                button { class: "login-btn", onclick: do_create, disabled: saving(),
                    if saving() { "Creando..." } else { "Crear Administrador" }
                }
            }
        }
    }
}
