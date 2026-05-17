use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailProvider {
    pub id: Uuid,
    pub corporation_id: Option<Uuid>,
    pub school_id: Option<Uuid>,
    pub provider_type: String,
    pub smtp_host: String,
    pub smtp_port: i32,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub from_email: String,
    pub from_name: Option<String>,
    pub reply_to: Option<String>,
    pub max_daily_sends: i32,
    pub sent_today: i32,
    pub last_sent_date: Option<NaiveDate>,
    pub is_verified: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEmailProviderPayload {
    pub corporation_id: Option<Uuid>,
    pub school_id: Option<Uuid>,
    pub provider_type: Option<String>,
    pub smtp_host: String,
    pub smtp_port: Option<i32>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub from_email: String,
    pub from_name: Option<String>,
    pub reply_to: Option<String>,
    pub max_daily_sends: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEmailProviderPayload {
    pub smtp_host: Option<String>,
    pub smtp_port: Option<i32>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub from_email: Option<String>,
    pub from_name: Option<String>,
    pub reply_to: Option<String>,
    pub max_daily_sends: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailQueueItem {
    pub id: Uuid,
    pub provider_id: Option<Uuid>,
    pub corporation_id: Option<Uuid>,
    pub school_id: Option<Uuid>,
    pub sender_email: String,
    pub sender_name: Option<String>,
    pub recipient_type: String,
    pub subject: String,
    pub body: String,
    pub body_type: String,
    pub status: String,
    pub priority: i32,
    pub total_recipients: i32,
    pub sent_count: i32,
    pub failed_count: i32,
    pub last_error: Option<String>,
    pub batch_id: Option<Uuid>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMassEmailPayload {
    pub corporation_id: Option<Uuid>,
    pub school_id: Option<Uuid>,
    pub subject: String,
    pub body: String,
    pub body_type: Option<String>,
    pub recipient_ids: Vec<Uuid>,
    pub recipient_type: String,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub priority: Option<i32>,
}

impl EmailProvider {
    pub fn smtp_connection_string(&self) -> String {
        format!("{}:{}", self.smtp_host, self.smtp_port)
    }

    pub fn display_name(&self) -> String {
        self.from_name
            .clone()
            .map(|n| format!("{} <{}>", n, self.from_email))
            .unwrap_or_else(|| self.from_email.clone())
    }
}
