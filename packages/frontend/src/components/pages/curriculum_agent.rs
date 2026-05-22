use dioxus::prelude::*;
use serde_json::json;

use crate::api::client;

#[derive(Clone)]
struct SearchResultItem {
    text: String,
    fuente: String,
    nivel: String,
    asignatura: String,
    #[allow(dead_code)]
    score: usize,
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

const NIVELES: &[&str] = &[
    "Sala Cuna",
    "Nivel Medio",
    "Nivel Transición",
    "1° Básico",
    "2° Básico",
    "3° Básico",
    "4° Básico",
    "5° Básico",
    "6° Básico",
    "7° Básico",
    "8° Básico",
    "1° Medio",
    "2° Medio",
    "3° Medio",
    "4° Medio",
];

const ASIGNATURAS: &[&str] = &[
    "Lenguaje y Comunicación",
    "Matemática",
    "Ciencias Naturales",
    "Historia, Geografía y Cs. Sociales",
    "Inglés",
    "Artes Visuales",
    "Música",
    "Educación Física y Salud",
    "Tecnología",
    "Orientación",
    "Religión",
];

#[component]
pub fn CurriculumAgent() -> Element {
    let mut active_tab = use_signal(|| "documentos".to_string());

    rsx! {
        div { class: "page-header",
            h1 { "Currículum Nacional" }
            p { "Bases Curriculares, normativa y programas de estudio del Ministerio de Educación" }
        }
        div { class: "tab-bar",
            button { class: "tab", class: if active_tab() == "documentos" { "active" }, onclick: move |_| active_tab.set("documentos".to_string()), "Documentos" }
            button { class: "tab", class: if active_tab() == "generar" { "active" }, onclick: move |_| active_tab.set("generar".to_string()), "Generar con IA" }
        }
        {if active_tab() == "documentos" {
            rsx! { SearchSection {} }
        } else {
            rsx! { AiGeneratorSection {} }
        }}
    }
}

#[component]
fn SearchSection() -> Element {
    let mut query = use_signal(String::new);
    let mut results = use_signal(Vec::<SearchResultItem>::new);
    let mut loading = use_signal(|| false);
    let mut searched = use_signal(|| false);

    let do_search = move |_| {
        let q = query().trim().to_string();
        if q.is_empty() || loading() {
            return;
        }
        query.set(String::new());
        loading.set(true);
        searched.set(true);

        spawn(async move {
            let resp = client::post_json(
                "/api/curriculum/search",
                &json!({ "q": q, "limit": 10 }),
            )
            .await;

            loading.set(false);

            match resp {
                Ok(data) => {
                    let items: Vec<SearchResultItem> = data["results"]
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .map(|v| SearchResultItem {
                                    text: v["text"].as_str().unwrap_or("").to_string(),
                                    fuente: v["fuente"].as_str().unwrap_or("").to_string(),
                                    nivel: v["nivel"].as_str().unwrap_or("").to_string(),
                                    asignatura: v["asignatura"].as_str().unwrap_or("").to_string(),
                                    score: v["score"].as_u64().unwrap_or(0) as usize,
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    results.set(items);
                }
                Err(e) => {
                    results.set(vec![SearchResultItem {
                        text: format!("Error al buscar: {e}"),
                        fuente: String::new(),
                        nivel: String::new(),
                        asignatura: String::new(),
                        score: 0,
                    }]);
                }
            }
        });
    };

    let on_key_down = move |e: Event<KeyboardData>| {
        if e.key() == Key::Enter {
            let q = query().trim().to_string();
            if q.is_empty() || loading() {
                return;
            }
            query.set(String::new());
            loading.set(true);
            searched.set(true);

            spawn(async move {
                let resp = client::post_json(
                    "/api/curriculum/search",
                    &json!({ "q": q, "limit": 10 }),
                )
                .await;

                loading.set(false);

                match resp {
                    Ok(data) => {
                        let items: Vec<SearchResultItem> = data["results"]
                            .as_array()
                            .map(|arr| {
                                arr.iter()
                                    .map(|v| SearchResultItem {
                                        text: v["text"].as_str().unwrap_or("").to_string(),
                                        fuente: v["fuente"].as_str().unwrap_or("").to_string(),
                                        nivel: v["nivel"].as_str().unwrap_or("").to_string(),
                                        asignatura: v["asignatura"].as_str().unwrap_or("").to_string(),
                                        score: v["score"].as_u64().unwrap_or(0) as usize,
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        results.set(items);
                    }
                    Err(e) => {
                        results.set(vec![SearchResultItem {
                            text: format!("Error al buscar: {e}"),
                            fuente: String::new(),
                            nivel: String::new(),
                            asignatura: String::new(),
                            score: 0,
                        }]);
                    }
                }
            });
        }
    };

    let content = if loading() {
        rsx! {
            div { class: "chat-message bot-message",
                div { class: "message-bubble bot-bubble",
                    "Buscando en la base de conocimientos del currículum nacional..."
                }
            }
        }
    } else if !results().is_empty() {
        let items = results();
        rsx! {
            p { class: "result-count", "Se encontraron {items.len()} resultados" }
            {items.iter().map(|r| {
                let has_meta = !r.nivel.is_empty() || !r.asignatura.is_empty();
                rsx! {
                    div { class: "search-result-card",
                        p { class: "result-text", "{truncate(&r.text, 300)}" }
                        if has_meta {
                            div { class: "result-meta",
                                if !r.nivel.is_empty() {
                                    span { class: "result-tag nivel", "{r.nivel}" }
                                }
                                if !r.asignatura.is_empty() {
                                    span { class: "result-tag asignatura", "{r.asignatura}" }
                                }
                            }
                        }
                        if !r.fuente.is_empty() {
                            div { class: "result-fuente", "Fuente: {r.fuente}" }
                        }
                    }
                }
            })}
        }
    } else if searched() {
        rsx! {
            div { class: "chat-message bot-message",
                div { class: "message-bubble bot-bubble",
                    "No se encontraron resultados para tu búsqueda."
                }
            }
        }
    } else {
        rsx! {
            div { class: "chat-message bot-message",
                div { class: "message-bubble bot-bubble",
                    "Ingresa un término de búsqueda para consultar el Currículum Nacional chileno."
                }
            }
        }
    };

    rsx! {
        div { class: "curriculum-chat-container",
            div { class: "chat-messages",
                {content}
            }
            div { class: "chat-input-area",
                input {
                    class: "chat-input",
                    value: "{query}",
                    oninput: move |e| query.set(e.value()),
                    onkeydown: on_key_down,
                    placeholder: "Ej: Decreto 67, OA Matemática 1° Básico..."
                }
                button {
                    class: "btn btn-primary",
                    disabled: loading() || query().trim().is_empty(),
                    onclick: do_search,
                    { if loading() { "Buscando..." } else { "Buscar" } }
                }
            }
        }
    }
}

async fn search_kb(query: &str, limit: usize) -> String {
    let resp = client::post_json(
        "/api/curriculum/search",
        &json!({ "q": query, "limit": limit }),
    )
    .await;

    match resp {
        Ok(data) => {
            let results = data["results"].as_array().cloned().unwrap_or_default();
            if results.is_empty() {
                return String::new();
            }
            let mut ctx = String::new();
            for (i, r) in results.iter().enumerate() {
                let text = r["text"].as_str().unwrap_or("");
                let fuente = r["fuente"].as_str().unwrap_or("");
                let nivel = r["nivel"].as_str().unwrap_or("");
                let asignatura = r["asignatura"].as_str().unwrap_or("");
                ctx.push_str(&format!(
                    "[{}] ({} - {} - {})\n{}\n\n",
                    i + 1,
                    nivel,
                    asignatura,
                    fuente,
                    text
                ));
            }
            ctx
        }
        Err(_) => String::new(),
    }
}

async fn stream_from_ollama(
    prompt: &str,
    mut on_token: impl FnMut(&str),
) -> Result<(String, String), String> {
    let origin = web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_else(|| "http://localhost:8080".to_string());
    let url = format!("{}/api/ai/api/chat", origin);

    let body = json!({
        "model": "llama3.1:latest",
        "messages": [
            {"role": "user", "content": prompt}
        ],
        "stream": true,
        "options": {
            "num_predict": 8192,
            "temperature": 0.7
        }
    });

    let client = reqwest::Client::new();
    let mut resp = client.post(&url).json(&body).send().await.map_err(|e| format!("Error de conexión: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body_text = resp.text().await.unwrap_or_default();
        return Err(format!("Error HTTP {status}: {body_text}"));
    }

    let mut full = String::new();
    let mut raw = String::new();
    let full_text = resp.text().await.map_err(|e| format!("Error de lectura: {e}"))?;

    for line in full_text.lines() {
        if line.is_empty() {
            continue;
        }
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(msg) = data["message"].as_object() {
                if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                    raw.push_str(content);
                    full.push_str(content);
                    on_token(content);
                }
            }
            if data.get("done").and_then(|d| d.as_bool()).unwrap_or(false) {
                break;
            }
        }
    }

    let mut thinking = String::new();
    let mut final_text = raw.clone();
    if let Some(start) = raw.find("<thinking>") {
        if let Some(end) = raw.find("</thinking>") {
            thinking = raw[start + 10..end].to_string();
            final_text = format!("{}{}", &raw[..start], &raw[end + 12..]);
            final_text = final_text.trim().to_string();
        }
    }

    Ok((thinking, final_text))
}

#[component]
fn AiGeneratorSection() -> Element {
    let mut modo = use_signal(|| "consultar".to_string());

    let mut query = use_signal(String::new);
    let mut nivel = use_signal(|| String::new());
    let mut asignatura = use_signal(|| String::new());
    let mut tipo = use_signal(|| "prueba".to_string());
    let mut oa_requirements = use_signal(String::new);

    let mut is_generating = use_signal(|| false);
    let mut response_text = use_signal(String::new);
    let mut thinking_text = use_signal(String::new);
    let mut show_thinking = use_signal(|| false);
    let mut error_msg = use_signal(String::new);
    let mut context_used = use_signal(String::new);

    let consultar_ok = move || !query().trim().is_empty();
    let generar_ok = move || !nivel().is_empty() && !asignatura().is_empty() && !oa_requirements().trim().is_empty();

    let mut start_ai = move || {
        if is_generating() {
            return;
        }
        let m = modo();
        if m == "consultar" && !consultar_ok() {
            return;
        }
        if m == "generar" && !generar_ok() {
            return;
        }

        let q = if m == "consultar" {
            query().trim().to_string()
        } else {
            oa_requirements().trim().to_string()
        };

        is_generating.set(true);
        error_msg.set(String::new());
        response_text.set(String::new());
        thinking_text.set(String::new());
        show_thinking.set(false);
        context_used.set(String::new());

        spawn(async move {
            let mut response_text = response_text;
            let mut thinking_text = thinking_text;
            let mut error_msg = error_msg;
            let mut context_used = context_used;
            let mut is_generating = is_generating;

            let context = search_kb(&q, 8).await;
            context_used.set(if context.is_empty() {
                "(sin contexto del KB)".to_string()
            } else {
                format!("(basado en {} fragmentos del currículum)", context.matches('[').count())
            });

            let prompt = if m == "consultar" {
                if context.is_empty() {
                    format!(
                        r#"Eres un asistente experto en el Currículum Nacional de Chile.

Responde la siguiente consulta del usuario. Si no tienes información suficiente, indícalo claramente.

Antes de responder, razona paso a paso dentro de etiquetas <thinking>.</thinking>.

=== CONSULTA ===
{consulta}"#,
                        consulta = q
                    )
                } else {
                    format!(
                        r#"Eres un asistente experto en el Currículum Nacional de Chile.

A continuación tienes fragmentos extraídos de la documentación oficial del currículum. Úsalos como contexto para responder la consulta del usuario.

Si la información en los fragmentos no es suficiente para responder completamente, indícalo claramente.

Antes de responder, razona paso a paso dentro de etiquetas <thinking>.</thinking>.

=== CONTEXTO ===
{context}

=== CONSULTA ===
{consulta}"#,
                        context = context,
                        consulta = q
                    )
                }
            } else {
                let n = nivel();
                let a = asignatura();
                let t = tipo();

                if context.is_empty() {
                    format!(
                        r#"Eres un profesor experto en el Currículum Nacional de Chile y en la creación de material educativo.

Cuando recibas una solicitud para crear material educativo:
1. Primero, analiza los OA solicitados y planifica el contenido DENTRO de etiquetas <thinking> y </thinking>.
2. Luego, fuera de las etiquetas, genera el material completo y listo para usar.

=== SOLICITUD ===
Nivel: {nivel}
Asignatura: {asignatura}
Tipo de material: {tipo}
OA / Requisitos: {oa}

Genera el material en español, con instrucciones claras para el estudiante."#,
                        nivel = n,
                        asignatura = a,
                        tipo = t,
                        oa = q
                    )
                } else {
                    format!(
                        r#"Eres un profesor experto en el Currículum Nacional de Chile y en la creación de material educativo.

A continuación tienes fragmentos del currículum oficial relevantes para la solicitud. Úsalos como referencia para alinear el material con los OA.

Antes de generar el material, razona paso a paso dentro de etiquetas <thinking>.</thinking>.

=== CONTEXTO DEL CURRÍCULUM ===
{context}

=== SOLICITUD ===
Nivel: {nivel}
Asignatura: {asignatura}
Tipo de material: {tipo}
OA / Requisitos: {oa}

Genera el material en español, con instrucciones claras para el estudiante."#,
                        context = context,
                        nivel = n,
                        asignatura = a,
                        tipo = t,
                        oa = q
                    )
                }
            };

            match stream_from_ollama(&prompt, move |token| {
                let cur = response_text();
                let mut next = cur;
                next.push_str(token);
                response_text.set(next);
            })
            .await
            {
                Ok((thinking, final_text)) => {
                    thinking_text.set(thinking);
                    response_text.set(final_text);
                }
                Err(e) => {
                    error_msg.set(e);
                }
            }

            is_generating.set(false);
        });
    };

    let on_enter = move |e: Event<KeyboardData>| {
        if e.key() == Key::Enter && modo() == "consultar" {
            start_ai();
        }
    };

    rsx! {
        div { class: "ai-generator-container",
            div { class: "ai-form-card",
                div { class: "modo-selector",
                    button { class: "modo-btn", class: if modo() == "consultar" { "active" }, onclick: move |_| modo.set("consultar".to_string()), "Consultar" }
                    button { class: "modo-btn", class: if modo() == "generar" { "active" }, onclick: move |_| modo.set("generar".to_string()), "Generar Material" }
                }

                {if modo() == "consultar" {
                    rsx! {
                        h3 { "Consultar Currículum Nacional" }
                        p { class: "form-description", "Haz una pregunta sobre el Currículum Nacional. La IA buscará en los documentos oficiales para responder." }
                        div { class: "form-group",
                            textarea {
                                class: "ai-textarea",
                                placeholder: "Ej: ¿Qué dice el Decreto 67 sobre evaluación? ¿Cuáles son los OA de Matemática en 1° Básico?",
                                value: "{query}",
                                oninput: move |e| query.set(e.value()),
                                onkeydown: on_enter,
                                rows: "3",
                            }
                        }
                        button {
                            class: "btn btn-primary generate-btn",
                            disabled: is_generating() || !consultar_ok(),
                            onclick: move |_| start_ai(),
                            { if is_generating() { "Consultando..." } else { "Consultar" } }
                        }
                    }
                } else {
                    rsx! {
                        h3 { "Generar Material Educativo" }
                        p { class: "form-description", "Completa los campos para que la IA genere una prueba, ejercicios o guía de estudio alineada al Currículum Nacional." }
                        div { class: "form-row",
                            div { class: "filter-group",
                                label { "Nivel" }
                                select {
                                    value: "{nivel}",
                                    oninput: move |e| nivel.set(e.value()),
                                    option { value: "", "Seleccionar..." }
                                    {NIVELES.iter().map(|n| {
                                        let selected = nivel() == *n;
                                        rsx! {
                                            option { selected: selected, value: "{n}", "{n}" }
                                        }
                                    })}
                                }
                            }
                            div { class: "filter-group",
                                label { "Asignatura" }
                                select {
                                    value: "{asignatura}",
                                    oninput: move |e| asignatura.set(e.value()),
                                    option { value: "", "Seleccionar..." }
                                    {ASIGNATURAS.iter().map(|a| {
                                        let selected = asignatura() == *a;
                                        rsx! {
                                            option { selected: selected, value: "{a}", "{a}" }
                                        }
                                    })}
                                }
                            }
                            div { class: "filter-group",
                                label { "Tipo" }
                                select {
                                    value: "{tipo}",
                                    oninput: move |e| tipo.set(e.value()),
                                    option { value: "prueba", "Prueba" }
                                    option { value: "ejercicios", "Ejercicios" }
                                    option { value: "guia", "Guía de Estudio" }
                                    option { value: "taller", "Taller" }
                                    option { value: "evaluacion", "Evaluación Formativa" }
                                }
                            }
                        }
                        div { class: "form-group",
                            label { "OA / Requisitos específicos" }
                            textarea {
                                class: "ai-textarea",
                                placeholder: "Ej: OA 1: Leer números del 0 al 100. OA 3: Comparar y ordenar números.",
                                value: "{oa_requirements}",
                                oninput: move |e| oa_requirements.set(e.value()),
                                rows: "4",
                            }
                        }
                        button {
                            class: "btn btn-primary generate-btn",
                            disabled: is_generating() || !generar_ok(),
                            onclick: move |_| start_ai(),
                            { if is_generating() { "Generando..." } else { "Generar" } }
                        }
                    }
                }}

                if !error_msg().is_empty() {
                    div { class: "ai-error", role: "alert", "{error_msg}" }
                }
            }

            if is_generating() || !response_text().is_empty() {
                div { class: "ai-output-card",
                    if is_generating() {
                        div { class: "generating-indicator",
                            span { class: "loading-dots", "Procesando" }
                        }
                    }

                    if !context_used().is_empty() {
                        div { class: "context-info", "{context_used()}" }
                    }

                    if !thinking_text().is_empty() {
                        div { class: "thinking-section",
                            button {
                                class: "thinking-toggle",
                                onclick: move |_| show_thinking.set(!show_thinking()),
                                if show_thinking() { "▼ Razonamiento" } else { "▶ Razonamiento" }
                            }
                            if show_thinking() {
                                div { class: "thinking-content",
                                    pre { "{thinking_text()}" }
                                }
                            }
                        }
                    }

                    {if !response_text().is_empty() {
                        rsx! {
                            div { class: "ai-response", "{response_text()}" }
                        }
                    } else {
                        rsx! {}
                    }}
                }
            }
        }
    }
}
