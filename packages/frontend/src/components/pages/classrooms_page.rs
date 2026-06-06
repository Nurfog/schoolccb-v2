use crate::api::client;
use dioxus::prelude::*;

#[component]
pub fn ClassroomsPage() -> Element {
    let mut rooms = use_resource(|| client::fetch_classrooms());
    let mut availability = use_resource(|| async {
        let rooms = client::fetch_classrooms().await.ok();
        let ids: Vec<String> = rooms
            .and_then(|j| j["classrooms"].as_array().cloned())
            .unwrap_or_default()
            .iter()
            .filter_map(|r| r["id"].as_str().map(String::from))
            .collect();
        let mut map = std::collections::HashMap::new();
        for id in ids {
            if let Ok(data) = client::classroom_availability(&id).await {
                if let Some(c) = data["classroom"].as_object() {
                    map.insert(id.clone(), c.clone());
                }
            }
        }
        map
    });

    let mut name = use_signal(String::new);
    let mut capacity = use_signal(|| 30i32);
    let mut location = use_signal(String::new);
    let mut editing_id = use_signal(|| None::<String>);
    let mut show_form = use_signal(|| false);
    let mut saving = use_signal(|| false);

    let mut reset_form = move || {
        name.set(String::new());
        capacity.set(30);
        location.set(String::new());
        editing_id.set(None);
        show_form.set(false);
    };

    let do_save = move |_| {
        saving.set(true);
        let payload = serde_json::json!({ "name": name(), "capacity": capacity(), "location": if location().is_empty() { serde_json::Value::Null } else { serde_json::json!(location()) } });
        let is_edit = editing_id().is_some();
        spawn(async move {
            if is_edit {
                let _ = client::update_classroom(&editing_id().unwrap_or_default(), &payload).await;
            } else {
                let _ = client::create_classroom(&payload).await;
            }
            saving.set(false);
            reset_form();
            rooms.restart();
            availability.restart();
        });
    };

    let mut do_edit = move |id: String, n: String, cap: i32, loc: String| {
        name.set(n);
        capacity.set(cap);
        location.set(loc);
        editing_id.set(Some(id));
        show_form.set(true);
    };

    let do_delete = move |id: String| {
        if !web_sys::window().unwrap().confirm_with_message("¿Estás seguro?").unwrap_or(false) {
            return;
        }
        spawn(async move {
            let _ = client::delete_classroom(&id).await;
            rooms.restart();
            availability.restart();
        });
    };

    rsx! {
        div { class: "page-header", h1 { "Salas" } p { "Gestión de salas, capacidad y disponibilidad" } }
        div { class: "page-toolbar", button { class: "btn btn-primary", onclick: move |_| { reset_form(); show_form.set(true); }, "Nueva Sala" } }
        {
            if show_form() {
                rsx! {
                    div { class: "form-card",
                        div { class: "form-row",
                            div { class: "form-group", label { "Nombre:" } input { class: "form-input", value: "{name}", oninput: move |e| name.set(e.value()), placeholder: "Sala 101" } }
                            div { class: "form-group", label { "Capacidad:" } input { class: "form-input", value: "{capacity}", oninput: move |e| { if let Ok(v) = e.value().parse() { capacity.set(v); } }, type: "number", min: "1" } }
                        }
                        div { class: "form-row",
                            div { class: "form-group", label { "Ubicación:" } input { class: "form-input", value: "{location}", oninput: move |e| location.set(e.value()), placeholder: "Piso 1, Edificio A" } }
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
            style { "
                .cap-bar {{ display: flex; align-items: center; gap: 8px; }}
                .cap-track {{ flex: 1; height: 8px; background: #eceff1; border-radius: 4px; overflow: hidden; }}
                .cap-fill {{ height: 100%; border-radius: 4px; transition: width 0.3s; }}
                .cap-fill.low {{ background: #4caf50; }}
                .cap-fill.med {{ background: #ff9800; }}
                .cap-fill.high {{ background: #f44336; }}
                .cap-label {{ font-size: 0.85rem; color: #546e7a; white-space: nowrap; }}
            " }
            match rooms() {
                Some(Ok(j)) => {
                    let rows: Vec<(String, String, i32, String)> = j["classrooms"].as_array().map(|arr| arr.iter().map(|r| {
                        (r["id"].as_str().unwrap_or("").to_string(), r["name"].as_str().unwrap_or("").to_string(), r["capacity"].as_i64().unwrap_or(0) as i32, r["location"].as_str().unwrap_or("").to_string())
                    }).collect()).unwrap_or_default();

                    let av_map = availability();

                    let rows_enhanced: Vec<(String, String, i32, String, String, String, String)> = rows.iter().map(|(id, n, cap, loc)| {
                        let enrolled = av_map.as_ref().and_then(|m| m.get(id)).and_then(|a| a.get("enrolled").and_then(|v| v.as_i64())).unwrap_or(0);
                        let pct = if *cap > 0 { (enrolled as f64 / *cap as f64) * 100.0 } else { 0.0 };
                        let fill_class = if pct >= 90.0 { "high".to_string() } else if pct >= 70.0 { "med".to_string() } else { "low".to_string() };
                        let style_width = format!("width: {:.0}%", pct);
                        let label = format!("{}/{} ({:.0}%)", enrolled, cap, pct);
                        (id.clone(), n.clone(), *cap, loc.clone(), style_width, fill_class, label)
                    }).collect();

                    rsx! {
                        table { class: "data-table",
                            thead { tr { th { "Nombre" } th { "Capacidad" } th { "Ocupación" } th { "Ubicación" } th { "Acciones" } } }
                            tbody { for (id, n, cap, loc, style_width, fill_class, label) in &rows_enhanced {
                                tr {
                                    td { "{n}" }
                                    td { "{cap}" }
                                    td {
                                        div { class: "cap-bar",
                                            div { class: "cap-track",
                                                div { class: "cap-fill {fill_class}", style: "{style_width}" }
                                            }
                                            span { class: "cap-label", "{label}" }
                                        }
                                    }
                                    td { if loc.is_empty() { "-" } else { "{loc}" } }
                                    td {
                                        button { class: "btn btn-sm", onclick: { let i = id.clone(); let nn = n.clone(); let cc = *cap; let ll = loc.clone(); move |_| do_edit(i.clone(), nn.clone(), cc, ll.clone()) }, "Editar" }
                                        button { class: "btn btn-sm btn-danger", style: "margin-left: 4px;", onclick: { let i = id.clone(); move |_| do_delete(i.clone()) }, "Eliminar" }
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
