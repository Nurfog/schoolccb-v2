use dioxus::prelude::*;
use serde_json::{Value, json};

use crate::api::client;

#[component]
fn RoleAssignRow(
    name: String,
    has_role: bool,
    on_assign: EventHandler<()>,
    on_remove: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "role-assign-row",
            span { "{name}" }
            if has_role {
                button { class: "btn btn-sm btn-danger", onclick: move |_| on_remove.call(()), "Quitar" }
            } else {
                button { class: "btn btn-sm btn-primary", onclick: move |_| on_assign.call(()), "Asignar" }
            }
        }
    }
}

fn get_users(data: &Value) -> Vec<Value> {
    data["users"].as_array().cloned().unwrap_or_default()
}

fn current_user_is_root() -> bool {
    let window = match web_sys::window() { Some(w) => w, None => return false };
    let store = match window.local_storage() { Ok(Some(s)) => s, _ => return false };
    let token = match store.get_item("jwt_token") { Ok(Some(t)) => t, _ => return false };
    let parts: Vec<&str> = token.split('.').collect();
    let payload_b64 = match parts.get(1) { Some(p) => p, None => return false };
    let decoded = match window.atob(payload_b64).ok() { Some(d) => d, None => return false };
    let claims: serde_json::Value = match serde_json::from_str(&decoded) { Ok(c) => c, Err(_) => return false };
    claims["role"].as_str() == Some("GerenteGeneral")
}

#[component]
pub fn UsersPage() -> Element {
    let users = use_resource(|| client::fetch_json("/api/auth/users"));
    let roles = use_resource(|| client::fetch_roles());
    let schools = use_resource(|| client::fetch_schools(None));
    let corps = use_resource(|| client::fetch_corporations());
    let selected_user_id = use_signal(|| Option::<String>::None);
    let permissions = use_resource(|| client::fetch_my_permissions());
    let is_root = current_user_is_root();

    let mut users = users;
    let mut show_new = use_signal(|| false);
    let mut nrut = use_signal(String::new);
    let mut nname = use_signal(String::new);
    let mut nemail = use_signal(String::new);
    let mut npass = use_signal(String::new);
    let mut nrole = use_signal(|| "Sostenedor".to_string());
    let mut nadmin_type = use_signal(|| "".to_string());
    let mut ncorp = use_signal(String::new);
    let mut nschool = use_signal(String::new);
    let mut saving = use_signal(|| false);
    let mut school_filter = use_signal(|| String::new());

    let do_create = move |_| {
        let rut = nrut();
        let name = nname();
        let email = nemail();
        let pass = npass();
        let role = nrole();
        let admin_type = nadmin_type();
        let corp = ncorp();
        let school = nschool();
        if rut.is_empty() || name.is_empty() { return; }
        saving.set(true);
        spawn(async move {
            let mut payload = serde_json::json!({
                "rut": rut, "name": name, "email": email,
                "password": pass, "role": role,
            });
            if !admin_type.is_empty() { payload["admin_type"] = json!(admin_type); }
            if !corp.is_empty() { payload["corporation_id"] = json!(corp); }
            if !school.is_empty() { payload["school_id"] = json!(school); }
            let _ = client::post_json("/api/auth/register", &payload).await;
            saving.set(false);
            show_new.set(false);
            nrut.set(String::new());
            nname.set(String::new());
            nemail.set(String::new());
            npass.set(String::new());
            nrole.set("Sostenedor".to_string());
            ncorp.set(String::new());
            nschool.set(String::new());
            users.restart();
        });
    };

    let mut user_roles: Resource<Result<Value, String>> = use_resource(move || {
        let uid = selected_user_id();
        async move {
            match uid {
                Some(ref id) => client::fetch_user_roles(id).await,
                None => Err("none".into()),
            }
        }
    });

    let do_assign = move |uid: String, rid: String| {
        spawn({
            let uid = uid.clone();
            async move {
                let _ = client::assign_role(&uid, &rid).await;
                user_roles.restart();
            }
        });
    };

    let do_remove = move |uid: String, rid: String| {
        spawn({
            async move {
                let _ = client::remove_role(&uid, &rid).await;
                user_roles.restart();
            }
        });
    };

    let corp_options: Vec<Element> = match corps() {
        Some(Ok(data)) => {
            let list = data["corporations"].as_array().cloned().unwrap_or_default();
            list.iter().map(|c| {
                let cid = c["id"].as_str().unwrap_or("").to_string();
                let cn = c["name"].as_str().unwrap_or("").to_string();
                rsx! { option { key: "{cid}", value: "{cid}", "{cn}" } }
            }).collect()
        }
        _ => vec![],
    };

    let school_filter_options: Vec<Element> = match schools() {
        Some(Ok(s)) => {
            let list = s["schools"].as_array().cloned().unwrap_or_default();
            list.iter().map(|sc| {
                let sid = sc["id"].as_str().unwrap_or("").to_string();
                let sname = sc["name"].as_str().unwrap_or("").to_string();
                rsx! { option { value: "{sid}", "{sname}" } }
            }).collect()
        }
        _ => vec![],
    };

    let school_options: Vec<Element> = match schools() {
        Some(Ok(s)) => {
            let list = s["schools"].as_array().cloned().unwrap_or_default();
            list.iter().map(|sc| {
                let sid = sc["id"].as_str().unwrap_or("").to_string();
                let sname = sc["name"].as_str().unwrap_or("").to_string();
                rsx! { option { value: "{sid}", "{sname}" } }
            }).collect()
        }
        _ => vec![],
    };

    let assignment_panel: Element = match permissions() {
        Some(Ok(p)) if p["can_assign_roles"].as_bool().unwrap_or(false) => {
            match selected_user_id() {
                Some(ref uid) => {
                    rsx! {
                        div { class: "role-assignment-panel",
                            div { class: "detail-section",
                                h4 { "Colegio" }
                                select { class: "form-input",
                                    option { value: "", "Sin asignar" }
                                    { school_options.into_iter() }
                                }
                            }
                            h4 { "Roles asignados" }
                            match user_roles() {
                                Some(Ok(data)) => {
                                    let assigned: Vec<String> = data["role_ids"].as_array()
                                        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                        .unwrap_or_default();
                                    let all_roles: Vec<Value> = match roles() {
                                        Some(Ok(r)) => r["roles"].as_array().cloned().unwrap_or_default(),
                                        _ => vec![],
                                    };
                                    rsx! {
                                        {all_roles.into_iter().map(|r| {
                                            let rid = r["id"].as_str().unwrap_or("").to_string();
                                            let rname = r["name"].as_str().unwrap_or("").to_string();
                                            let has_role = assigned.contains(&rid);
                                            rsx! {
                                                RoleAssignRow {
                                                    key: "{rid}",
                                                    name: rname,
                                                    has_role: has_role,
                                                    on_assign: { let u = uid.clone(); let rd = rid.clone(); move |_| do_assign(u.clone(), rd.clone()) },
                                                    on_remove: { let u = uid.clone(); let rd = rid.clone(); move |_| do_remove(u.clone(), rd.clone()) },
                                                }
                                            }
                                        })}
                                    }
                                }
                                _ => rsx! { div { class: "loading-spinner", "Cargando..." } },
                            }
                        }
                    }
                }
                None => {
                    rsx! { div { class: "role-assignment-panel", div { class: "empty-state", "Seleccione un usuario" } } }
                }
            }
        }
        _ => {
            rsx! { div { class: "role-assignment-panel", div { class: "empty-state", "Sin permisos para asignar roles" } } }
        }
    };

    rsx! {
        div { class: "page-header",
            h1 { "Usuarios y Perfiles" }
            p { "Gestión de usuarios y asignación de roles" }
        }
        div { class: "page-toolbar",
            button { class: "btn btn-primary", onclick: move |_| show_new.set(!show_new()),
                if show_new() { "Cancelar" } else { "Nuevo Usuario" }
            }
        }
        if show_new() {
            div { class: "form-card",
                div { class: "form-row",
                    div { class: "form-group",
                        label { "RUT:" }
                        input { class: "form-input", value: "{nrut}", oninput: move |e| nrut.set(e.value()), placeholder: "11.111.111-1" }
                    }
                    div { class: "form-group",
                        label { "Nombre:" }
                        input { class: "form-input", value: "{nname}", oninput: move |e| nname.set(e.value()), placeholder: "Nombre completo" }
                    }
                }
                div { class: "form-row",
                    div { class: "form-group",
                        label { "Email:" }
                        input { class: "form-input", r#type: "email", value: "{nemail}", oninput: move |e| nemail.set(e.value()), placeholder: "email@colegio.cl" }
                    }
                    div { class: "form-group",
                        label { "Contraseña:" }
                        input { class: "form-input", r#type: "password", value: "{npass}", oninput: move |e| npass.set(e.value()), placeholder: "Mín. 6 caracteres" }
                    }
                }
                div { class: "form-row",
                    div { class: "form-group",
                        label { "Rol:" }
                        select { class: "form-input", value: "{nrole}", oninput: move |e| nrole.set(e.value()),
                            if is_root { option { value: "Sostenedor", "Sostenedor" } }
                            option { value: "Administrador", "Administrador" }
                            option { value: "Director", "Director" }
                            option { value: "UTP", "UTP" }
                            option { value: "Profesor", "Profesor" }
                            option { value: "Apoderado", "Apoderado" }
                            option { value: "Alumno", "Alumno" }
                        }
                    }
                    if is_root && nrole() == "Sostenedor" {
                        div { class: "form-group",
                            label { "Corporación:" }
                            select { class: "form-input", value: "{ncorp}", oninput: move |e| ncorp.set(e.value()),
                                option { value: "", "Seleccionar..." }
                                { corp_options.into_iter() }
                            }
                        }
                    }
                    if nrole() == "Administrador" {
                        div { class: "form-group",
                            label { "Tipo de Admin:" }
                            select { class: "form-input", value: "{nadmin_type}", oninput: move |e| nadmin_type.set(e.value()),
                                option { value: "", "Normal (solo su colegio)" }
                                option { value: "global", "Global (todos los colegios)" }
                            }
                        }
                    }
                }
                div { class: "form-actions",
                    button { class: "btn btn-primary", disabled: saving(), onclick: do_create,
                        if saving() { "Creando..." } else { "Crear Usuario" }
                    }
                }
            }
        }
        div { class: "users-layout",
            div { class: "users-table-wrap",
                div { class: "page-toolbar", style: "margin-bottom: 8px;",
                    div { class: "form-group", style: "display: flex; align-items: center; gap: 8px; margin: 0;",
                        label { "Filtrar por colegio:" }
                        select { class: "form-input", style: "width: auto;", value: "{school_filter}", oninput: move |e| school_filter.set(e.value()),
                            option { value: "", "Todos los colegios" }
                            { school_filter_options.into_iter() }
                        }
                    }
                }
                table { class: "data-table",
                    thead { tr {
                        th { "Nombre" } th { "Email" } th { "RUT" } th { "Rol" } th { "Tipo Admin" } th { "Estado" }
                    }}
                    tbody {
                        match users() {
                            Some(Ok(data)) => {
                                let list = get_users(&data);
                                let sf = school_filter();
                                let filtered: Vec<Value> = if sf.is_empty() {
                                    list
                                } else {
                                    list.into_iter().filter(|u| u["school_id"].as_str() == Some(&sf)).collect()
                                };
                                if filtered.is_empty() {
                                    rsx! { tr { td { colspan: "6", class: "empty-state", "No hay usuarios" } } }
                                } else {
                                    rsx! {
                                        {filtered.into_iter().map(|user| {
                                            let uid = user["id"].as_str().unwrap_or("").to_string();
                                            let is_sel = selected_user_id.read().as_deref() == Some(&uid);
                                            let mut sel = selected_user_id.clone();
                                            rsx! {
                                                UserRow {
                                                    key: "{uid}",
                                                    user: user.clone(),
                                                    is_selected: is_sel,
                                                    on_select: move |u: String| sel.set(Some(u)),
                                                }
                                            }
                                        })}
                                    }
                                }
                            }
                            Some(Err(e)) => rsx! { tr { td { colspan: "5", "Error: {e}" } } },
                            None => rsx! { tr { td { colspan: "5", div { class: "loading-spinner", "Cargando..." } } } },
                        }
                    }
                }
            }
            { assignment_panel }
        }
    }
}

#[component]
fn UserRow(user: Value, is_selected: bool, on_select: EventHandler<String>) -> Element {
    let name = user["name"].as_str().unwrap_or("").to_string();
    let email = user["email"].as_str().unwrap_or("").to_string();
    let rut = user["rut"].as_str().unwrap_or("").to_string();
    let role = user["role"].as_str().unwrap_or("").to_string();
    let atype = user["admin_type"].as_str().unwrap_or("").to_string();
    let active = user["active"].as_bool().unwrap_or(true);
    let uid = user["id"].as_str().unwrap_or("").to_string();
    let row_class = if is_selected { "selected" } else { "" };
    let admin_label: Option<&str> = if atype == "global" { Some("Global") } else if !atype.is_empty() { Some(atype.as_str()) } else { None };
    rsx! {
        tr { class: "{row_class}",
            onclick: move |_| on_select.call(uid.clone()),
            td { class: "cell-name", "{name}" }
            td { "{email}" }
            td { span { class: "rut-badge", "{rut}" } }
            td { span { class: "role-badge", "{role}" } }
            td {
                if let Some(label) = admin_label {
                    span { class: "badge badge-info", "{label}" }
                } else {
                    span { style: "color: var(--text-secondary); font-size: 12px;", "—" }
                }
            }
            td {
                if active {
                    span { class: "status-active", "Activo" }
                } else {
                    span { class: "status-inactive", "Inactivo" }
                }
            }
        }
    }
}
