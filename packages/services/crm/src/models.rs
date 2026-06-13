use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

pub use schoolccb_common::auth::Claims;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesStage {
    pub id: Uuid,
    pub name: String,
    pub sort_order: i32,
    pub is_final: bool,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesProspect {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub rut: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub position: Option<String>,
    pub source: Option<String>,
    pub requirements: Option<serde_json::Value>,
    pub current_stage_id: Option<Uuid>,
    pub assigned_to: Option<Uuid>,
    pub estimated_value: Option<f64>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProspectPayload {
    pub first_name: String,
    pub last_name: String,
    pub rut: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub position: Option<String>,
    pub source: Option<String>,
    pub requirements: Option<serde_json::Value>,
    pub current_stage_id: Option<Uuid>,
    pub estimated_value: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProspectPayload {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub rut: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub position: Option<String>,
    pub source: Option<String>,
    pub requirements: Option<serde_json::Value>,
    pub current_stage_id: Option<Uuid>,
    pub assigned_to: Option<Uuid>,
    pub estimated_value: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesActivity {
    pub id: Uuid,
    pub prospect_id: Uuid,
    pub activity_type: String,
    pub subject: String,
    pub description: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub is_completed: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateActivityPayload {
    pub prospect_id: Uuid,
    pub activity_type: String,
    pub subject: String,
    pub description: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesContract {
    pub id: Uuid,
    pub prospect_id: Uuid,
    pub tax_id: Option<String>,
    pub plan_id: Option<Uuid>,
    pub modules: Option<serde_json::Value>,
    pub total_value: f64,
    pub discount: f64,
    pub tax_rate: Option<f64>,
    pub tax_amount: Option<f64>,
    pub subtotal: Option<f64>,
    pub status: String,
    pub signed_at: Option<DateTime<Utc>>,
    pub verified_at: Option<DateTime<Utc>>,
    pub activated_at: Option<DateTime<Utc>>,
    pub invoices: Option<serde_json::Value>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContractPayload {
    pub prospect_id: Uuid,
    pub plan_id: Uuid,
    pub modules: Option<serde_json::Value>,
    pub total_value: f64,
    pub discount: Option<f64>,
    pub tax_id: Option<String>,
    pub tax_rate: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesDocument {
    pub id: Uuid,
    pub contract_id: Uuid,
    pub file_name: String,
    pub file_url: Option<String>,
    pub doc_type: String,
    pub is_verified: bool,
    pub uploaded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentPayload {
    pub contract_id: Uuid,
    pub file_name: String,
    pub doc_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SalesProposal {
    pub id: Uuid,
    pub prospect_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub modules: Option<serde_json::Value>,
    pub total_value: f64,
    pub discount: f64,
    pub version: i32,
    pub status: String,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

// Sales Agent
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SalesAgent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub quota_monthly: f64,
    pub quota_quarterly: f64,
    pub commission_rate: f64,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentPayload {
    pub user_id: Uuid,
    pub quota_monthly: Option<f64>,
    pub quota_quarterly: Option<f64>,
    pub commission_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SalesGoal {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub goal_type: String,
    pub target_amount: f64,
    pub target_count: i32,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub achieved_amount: f64,
    pub achieved_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGoalPayload {
    pub agent_id: Uuid,
    pub goal_type: String,
    pub target_amount: f64,
    pub target_count: i32,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
}

// CSV Import
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SalesImport {
    pub id: Uuid,
    pub file_name: String,
    pub total_rows: i32,
    pub imported_rows: i32,
    pub failed_rows: i32,
    pub errors: Option<serde_json::Value>,
    pub status: String,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvImportPayload {
    pub rows: Vec<CsvProspectRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvProspectRow {
    pub first_name: String,
    pub last_name: String,
    pub rut: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub position: Option<String>,
    pub source: Option<String>,
    pub notes: Option<String>,
}

// Round-robin config
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoundRobinConfig {
    pub id: Uuid,
    pub active: bool,
    pub last_assigned_index: i32,
    pub updated_at: DateTime<Utc>,
}

// Invoice request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvoicePayload {
    pub invoice_type: Option<String>,
    pub notes: Option<String>,
}
