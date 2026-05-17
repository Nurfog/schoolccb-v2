use std::sync::Arc;

use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;

#[derive(Clone)]
pub struct Mailer {
    pool: PgPool,
    config: Arc<Config>,
}

impl Mailer {
    pub fn new(pool: PgPool, config: Arc<Config>) -> Self {
        Mailer { pool, config }
    }

    pub async fn send_welcome(
        &self,
        to: &str,
        temp_password: &str,
        corporation_name: &str,
        school_name: &str,
        corporation_id: Option<Uuid>,
    ) {
        let subject = "Bienvenido a SchoolCBB — Credenciales de Acceso".to_string();
        let body = format!(
            "Hola,\n\n\
             Bienvenido a SchoolCBB. Tu cuenta ha sido creada exitosamente.\n\n\
             Corporación: {corporation_name}\n\
             Colegio: {school_name}\n\n\
             Credenciales de acceso:\n\
             Email: {to}\n\
             Contraseña temporal: {temp_password}\n\n\
             Por favor, cambia tu contraseña al iniciar sesión.\n\n\
             Accede a: https://schoolccb.cl\n\n\
             Saludos,\n\
             Equipo SchoolCBB"
        );

        let result = self.send_with_best_provider(corporation_id, None, to, subject, body).await;
        if let Err(e) = result {
            tracing::warn!("Error enviando email de bienvenida a {to}: {e}");
        }
    }

    pub async fn send_proposal(
        &self,
        to: &str,
        client_name: &str,
        _pdf_filename: &str,
        corporation_id: Option<Uuid>,
    ) {
        let subject = format!("Propuesta Comercial SchoolCBB — {client_name}");
        let body = format!(
            "Hola {client_name},\n\n\
             Adjuntamos la propuesta comercial solicitada.\n\n\
             Puedes descargar el PDF desde tu portal.\n\n\
             Saludos,\n\
             Equipo SchoolCBB"
        );

        let result = self.send_with_best_provider(corporation_id, None, to, subject, body).await;
        if let Err(e) = result {
            tracing::warn!("Error enviando propuesta a {to}: {e}");
        }
    }

    async fn send_with_best_provider(
        &self,
        corporation_id: Option<Uuid>,
        school_id: Option<Uuid>,
        to: &str,
        subject: String,
        body: String,
    ) -> Result<(), String> {
        let provider = self.find_provider(corporation_id, school_id).await;

        if let Some(p) = provider {
            self.send_via_provider(&p, to, &subject, &body).await
        } else {
            self.send_via_global_config(to, &subject, &body).await
        }
    }

    async fn find_provider(
        &self,
        corporation_id: Option<Uuid>,
        school_id: Option<Uuid>,
    ) -> Option<schoolccb_common::email::EmailProvider> {
        // Try school-specific first, then corporation-wide
        if let Some(sid) = school_id {
            if let Some(p) = self
                .fetch_provider(corporation_id, Some(sid))
                .await
            {
                return Some(p);
            }
        }
        if let Some(cid) = corporation_id {
            if let Some(p) = self.fetch_provider(Some(cid), None).await {
                return Some(p);
            }
        }
        None
    }

    async fn fetch_provider(
        &self,
        corporation_id: Option<Uuid>,
        school_id: Option<Uuid>,
    ) -> Option<schoolccb_common::email::EmailProvider> {
        sqlx::query_as::<_, schoolccb_common::email::EmailProvider>(
            "SELECT id, corporation_id, school_id, provider_type, smtp_host, smtp_port,
                    smtp_username, smtp_password, from_email, from_name, reply_to,
                    max_daily_sends, sent_today, last_sent_date, is_verified, is_active,
                    created_at, updated_at
             FROM email_providers
             WHERE ($1::uuid IS NULL OR corporation_id = $1)
               AND ($2::uuid IS NULL OR school_id = $2)
               AND is_active = true
             LIMIT 1",
        )
        .bind(corporation_id)
        .bind(school_id)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten()
    }

    async fn send_via_provider(
        &self,
        provider: &schoolccb_common::email::EmailProvider,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), String> {
        let creds = Credentials::new(
            provider.smtp_username.clone().unwrap_or_default(),
            provider.smtp_password.clone().unwrap_or_default(),
        );

        let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&provider.smtp_host)
            .map_err(|e| format!("Error conectando SMTP {0}: {e}", provider.smtp_host))?
            .credentials(creds)
            .port(provider.smtp_port as u16)
            .build();

        let from = provider.display_name();
        let email = Message::builder()
            .from(from.parse().map_err(|e| format!("From inválido: {e}"))?)
            .to(to.parse().map_err(|e| format!("To inválido: {e}"))?)
            .subject(subject.to_string())
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())
            .map_err(|e| format!("Error construyendo email: {e}"))?;

        transport
            .send(email)
            .await
            .map_err(|e| format!("Error SMTP: {e}"))?;

        Self::update_sent_count(&self.pool, provider.id).await;
        tracing::info!("📧 Email enviado vía {} a {to}", provider.from_email);
        Ok(())
    }

    async fn send_via_global_config(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), String> {
        match (
            &self.config.smtp_host,
            &self.config.smtp_username,
            &self.config.smtp_password,
            &self.config.smtp_from,
        ) {
            (Some(host), Some(user), Some(pass), Some(from)) => {
                let creds = Credentials::new(user.clone(), pass.clone());
                let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)
                    .map_err(|e| format!("Error conectando SMTP: {e}"))?
                    .credentials(creds)
                    .port(self.config.smtp_port)
                    .build();

                let email = Message::builder()
                    .from(from.parse().map_err(|e| format!("From inválido: {e}"))?)
                    .to(to.parse().map_err(|e| format!("To inválido: {e}"))?)
                    .subject(subject.to_string())
                    .header(ContentType::TEXT_PLAIN)
                    .body(body.to_string())
                    .map_err(|e| format!("Error construyendo email: {e}"))?;

                transport.send(email).await.map_err(|e| format!("Error SMTP: {e}"))?;
                tracing::info!("📧 Email enviado vía config global a {to}");
                Ok(())
            }
            _ => {
                tracing::info!("📧 SIMULACIÓN: Email a {to} | Asunto: {subject}");
                Ok(())
            }
        }
    }

    async fn update_sent_count(pool: &PgPool, provider_id: Uuid) {
        let today = chrono::Utc::now().date_naive();
        let _ = sqlx::query(
            "UPDATE email_providers SET
             sent_today = CASE WHEN last_sent_date = $1 THEN sent_today + 1 ELSE 1 END,
             last_sent_date = $1,
             updated_at = NOW()
             WHERE id = $2",
        )
        .bind(today)
        .bind(provider_id)
        .execute(pool)
        .await;
    }

    pub async fn send_via_provider_for_test(
        &self,
        provider: &schoolccb_common::email::EmailProvider,
        to: &str,
    ) -> Result<(), String> {
        self.send_via_provider(provider, to, "SchoolCBB — Prueba de Configuración SMTP",
            "Este es un email de prueba para verificar la configuración SMTP.\n\nSi recibes este mensaje, la configuración es correcta.\n\nSaludos,\nEquipo SchoolCBB"
        ).await
    }

    // ─── Mass Email ───

    pub async fn send_mass_email(
        &self,
        corporation_id: Option<Uuid>,
        school_id: Option<Uuid>,
        subject: String,
        body: String,
        body_type: String,
        recipient_emails: Vec<(Uuid, String)>,
        created_by: Option<Uuid>,
    ) -> Result<Uuid, String> {
        let batch_id = Uuid::new_v4();
        let total = recipient_emails.len() as i32;

        let provider = self.find_provider(corporation_id, school_id).await;

        for (_user_id, email) in &recipient_emails {
            let queue_id = Uuid::new_v4();
            let _ = sqlx::query(
                "INSERT INTO email_queue (id, provider_id, corporation_id, school_id,
                 sender_email, sender_name, subject, body, body_type, status, priority,
                 total_recipients, batch_id, created_by)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
            )
            .bind(queue_id)
            .bind(provider.as_ref().map(|p| p.id))
            .bind(corporation_id)
            .bind(school_id)
            .bind(email)
            .bind(provider.as_ref().and_then(|p| p.from_name.clone()))
            .bind(&subject)
            .bind(&body)
            .bind(&body_type)
            .bind("pending")
            .bind(0i32)
            .bind(total)
            .bind(batch_id)
            .bind(created_by)
            .execute(&self.pool)
            .await;

            // Try to send immediately if no schedule
            if let Some(ref p) = provider {
                let result = self.send_via_provider(p, email, &subject, &body).await;
                let (is_ok, err_str) = match result {
                    Ok(_) => (true, None),
                    Err(e) => (false, Some(e)),
                };
                let status = if is_ok { "sent" } else { "failed" };
                let _ = sqlx::query(
                    "UPDATE email_queue SET status = $1, sent_count = CASE WHEN $2 THEN 1 ELSE 0 END,
                     failed_count = CASE WHEN $3 THEN 1 ELSE 0 END,
                     last_error = $4, sent_at = NOW(), updated_at = NOW()
                     WHERE id = $5",
                )
                .bind(status)
                .bind(is_ok)
                .bind(!is_ok)
                .bind(err_str)
                .bind(queue_id)
                .execute(&self.pool)
                .await;
            }
        }

        tracing::info!("📧 Lote {batch_id}: {total} emails encolados");
        Ok(batch_id)
    }
}
