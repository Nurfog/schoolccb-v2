use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct Mailer {
    pool: PgPool,
    smtp_host: String,
    smtp_port: u16,
    smtp_user: String,
    smtp_pass: String,
    from_address: String,
    from_name: String,
}

impl Mailer {
    pub fn new(
        pool: PgPool,
        smtp_host: String,
        smtp_port: u16,
        smtp_user: String,
        smtp_pass: String,
        from_address: String,
        from_name: String,
    ) -> Self {
        Mailer { pool, smtp_host, smtp_port, smtp_user, smtp_pass, from_address, from_name }
    }

    pub async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        let from: lettre::message::Mailbox = format!("{} <{}>", self.from_name, self.from_address)
            .parse()
            .map_err(|e: lettre::address::AddressError| format!("Dirección from inválida: {e}"))?;
        let to: lettre::message::Mailbox = to
            .parse()
            .map_err(|e: lettre::address::AddressError| format!("Dirección to inválida: {e}"))?;
        let email = Message::builder()
            .from(from)
            .to(to)
            .subject(subject.to_string())
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())
            .map_err(|e| format!("Error creando email: {e}"))?;

        let creds = Credentials::new(self.smtp_user.clone(), self.smtp_pass.clone());

        match AsyncSmtpTransport::<Tokio1Executor>::relay(&self.smtp_host) {
            Ok(relay) => {
                let mailer = relay
                    .credentials(creds)
                    .port(self.smtp_port)
                    .build();
                mailer.send(email).await.map_err(|e| format!("Error enviando email: {e}"))?;
                Ok(())
            }
            Err(e) => Err(format!("Error configurando SMTP: {e}")),
        }
    }

    pub async fn send_enrollment_confirmation(
        &self,
        to: &str,
        student_name: &str,
        school_name: &str,
        grade: &str,
    ) {
        let subject = format!("Confirmación de Matrícula — {}", student_name);
        let body = format!(
            "Estimado(a) apoderado(a),\n\n\
             Le informamos que el proceso de matrícula de {student_name} en {school_name} \
             ({grade}) se ha completado exitosamente.\n\n\
             El alumno se encuentra oficialmente matriculado y podrá acceder \
             a su portal de alumno para revisar horarios y actividades.\n\n\
             Portal Alumno: https://schoolccb.cl/student-portal\n\
             Portal Apoderado: https://schoolccb.cl/parent-portal\n\n\
             Saludos cordiales,\n\
             {school_name}\n\
             vía SchoolCBB"
        );
        if let Err(e) = self.send_email(to, &subject, &body).await {
            tracing::warn!("Error enviando confirmación de matrícula a {to}: {e}");
        }
    }

    pub async fn send_payment_confirmation(
        &self,
        to: &str,
        student_name: &str,
        amount: f64,
        method: &str,
    ) {
        let subject = "Confirmación de Pago — Colegio".to_string();
        let body = format!(
            "Estimado(a) apoderado(a),\n\n\
             Hemos recibido el pago de matrícula de {student_name}.\n\n\
             Monto: ${:.0}\n\
             Método: {method}\n\n\
             Puede revisar el detalle en su portal apoderado.\n\n\
             Saludos cordiales,\n\
             SchoolCBB",
            amount
        );
        if let Err(e) = self.send_email(to, &subject, &body).await {
            tracing::warn!("Error enviando confirmación de pago a {to}: {e}");
        }
    }

    pub async fn send_meeting_reminder(
        &self,
        to: &str,
        parent_name: &str,
        teacher_name: &str,
        date: &str,
        reason: &str,
    ) {
        let subject = "Recordatorio: Reunión de Apoderados".to_string();
        let body = format!(
            "Estimado(a) {parent_name},\n\n\
             Le recordamos que tiene una reunión agendada con el profesor {teacher_name}.\n\n\
             Fecha: {date}\n\
             Motivo: {reason}\n\n\
             Por favor, confirme su asistencia desde su portal apoderado.\n\n\
             Saludos cordiales,\n\
             SchoolCBB"
        );
        if let Err(e) = self.send_email(to, &subject, &body).await {
            tracing::warn!("Error enviando recordatorio a {to}: {e}");
        }
    }

    pub async fn send_attendance_alert(
        &self,
        to: &str,
        student_name: &str,
        attendance_pct: f64,
    ) {
        let subject = format!("Alerta de Asistencia — {}", student_name);
        let body = format!(
            "Estimado(a) apoderado(a),\n\n\
             Le informamos que la asistencia de {student_name} es del {:.1}%, \
             la cual se encuentra por debajo del umbral recomendado.\n\n\
             Le invitamos a contactar al profesor jefe para conocer más detalles \
             y coordinar las acciones necesarias.\n\n\
             Saludos cordiales,\n\
             SchoolCBB",
            attendance_pct
        );
        if let Err(e) = self.send_email(to, &subject, &body).await {
            tracing::warn!("Error enviando alerta de asistencia a {to}: {e}");
        }
    }

    pub async fn process_email_queue(&self) {
        let emails: Vec<(Uuid, String, String, String)> = sqlx::query_as(
            "SELECT id, recipient, subject, body FROM email_queue WHERE status = 'pending' LIMIT 10",
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        for (id, recipient, subject, body) in emails {
            match self.send_email(&recipient, &subject, &body).await {
                Ok(_) => {
                    sqlx::query("UPDATE email_queue SET status = 'sent', sent_at = NOW() WHERE id = $1")
                        .bind(id)
                        .execute(&self.pool)
                        .await
                        .unwrap_or_default();
                }
                Err(e) => {
                    tracing::error!("Error sending queued email {}: {e}", id);
                    sqlx::query("UPDATE email_queue SET status = 'failed', attempts = attempts + 1 WHERE id = $1")
                        .bind(id)
                        .execute(&self.pool)
                        .await
                        .unwrap_or_default();
                }
            }
        }
    }
}
