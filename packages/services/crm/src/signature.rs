use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Configuración del servicio de firma electrónica.
#[derive(Clone, Debug)]
pub struct SignatureConfig {
    pub provider: String,
    pub api_url: String,
    pub api_token: String,
}

impl SignatureConfig {
    pub fn from_env() -> Self {
        Self {
            provider: std::env::var("SIGNATURE_PROVIDER").unwrap_or_else(|_| "mock".into()),
            api_url: std::env::var("SIGNATURE_API_URL").unwrap_or_else(|_| String::new()),
            api_token: std::env::var("SIGNATURE_API_TOKEN").unwrap_or_else(|_| String::new()),
        }
    }
}

/// Solicitud de firma enviada al proveedor.
#[derive(Debug, Serialize)]
pub struct SignatureRequest {
    pub document_id: String,
    pub document_name: String,
    pub document_url: String,
    pub signers: Vec<SignerInfo>,
    pub expires_in_days: u32,
}

#[derive(Debug, Serialize)]
pub struct SignerInfo {
    pub rut: String,
    pub name: String,
    pub email: String,
    pub role: String,
}

/// Respuesta del proveedor de firma.
#[derive(Debug, Deserialize)]
pub struct SignatureResponse {
    pub request_id: String,
    pub status: String,
    pub signing_url: Option<String>,
}

/// Inicia un proceso de firma electrónica.
/// Si el proveedor es "mock", simula la firma sin llamar a una API externa.
pub async fn request_signature(
    config: &SignatureConfig,
    request: &SignatureRequest,
) -> Result<SignatureResponse, String> {
    match config.provider.as_str() {
        "toku" => request_toku_signature(config, request).await,
        _ => Ok(mock_signature(request)),
    }
}

fn mock_signature(request: &SignatureRequest) -> SignatureResponse {
    SignatureResponse {
        request_id: format!("mock-{}", uuid::Uuid::new_v4()),
        status: "pending".to_string(),
        signing_url: Some(format!("/mock/sign/{}", request.document_id)),
    }
}

/// Integración con Toku (firma electrónica chilena).
/// API reference: https://api.toku.com/v1/signatures
async fn request_toku_signature(
    config: &SignatureConfig,
    request: &SignatureRequest,
) -> Result<SignatureResponse, String> {
    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "document": {
            "id": request.document_id,
            "name": request.document_name,
            "url": request.document_url,
        },
        "signers": request.signers.iter().map(|s| serde_json::json!({
            "rut": s.rut,
            "name": s.name,
            "email": s.email,
            "role": s.role,
        })).collect::<Vec<Value>>(),
        "expires_in_days": request.expires_in_days,
    });

    let resp = client
        .post(format!("{}/v1/signatures", config.api_url))
        .header("Authorization", format!("Bearer {}", config.api_token))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Error al conectar con Toku: {e}"))?;

    let status = resp.status();
    let body: Value = resp
        .json()
        .await
        .map_err(|e| format!("Error al parsear respuesta Toku: {e}"))?;

    if status.is_success() {
        Ok(SignatureResponse {
            request_id: body["id"].as_str().unwrap_or("").to_string(),
            status: body["status"].as_str().unwrap_or("pending").to_string(),
            signing_url: body["signing_url"].as_str().map(|s| s.to_string()),
        })
    } else {
        Err(format!(
            "Toku error ({}): {}",
            status,
            body["message"].as_str().unwrap_or("unknown")
        ))
    }
}

/// Verifica el estado de una solicitud de firma en Toku.
#[allow(dead_code)]
pub async fn check_signature_status(
    config: &SignatureConfig,
    request_id: &str,
) -> Result<String, String> {
    match config.provider.as_str() {
        "toku" => {
            let client = reqwest::Client::new();
            let resp = client
                .get(format!("{}/v1/signatures/{}", config.api_url, request_id))
                .header("Authorization", format!("Bearer {}", config.api_token))
                .send()
                .await
                .map_err(|e| format!("Error al consultar Toku: {e}"))?;

            let body: Value = resp
                .json()
                .await
                .map_err(|e| format!("Error al parsear respuesta Toku: {e}"))?;

            Ok(body["status"].as_str().unwrap_or("unknown").to_string())
        }
        _ => Ok("completed".to_string()),
    }
}
