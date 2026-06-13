use axum::{
    Json, Router,
    extract::{
        Path, Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{NotifError, NotifResult};

pub use schoolccb_common::auth::Claims;
use schoolccb_common::auth::require_any_role;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ws", get(ws_handler))
        .route(
            "/api/communications/messages",
            get(list_messages).post(send_message),
        )
        .route(
            "/api/communications/messages/unread-count",
            get(unread_count),
        )
        .route("/api/communications/messages/{id}", get(get_message))
        .route("/api/communications/messages/{id}/read", post(mark_read))
        .route(
            "/api/communications/interviews",
            get(list_interviews).post(create_interview),
        )
        .route(
            "/api/communications/interviews/{id}",
            get(get_interview)
                .put(update_interview)
                .delete(delete_interview),
        )
        .route(
            "/api/communications/interviews/student/{student_id}",
            get(interviews_by_student),
        )
        .route("/api/notifications", post(send_notification))
        .route("/api/notifications", get(list_notifications))
        .route("/api/notifications/unread-count", get(unread_notification_count))
        .route("/api/notifications/{id}/read", post(mark_notification_read))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
) -> Response {
    let token = match params.get("token") {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Token requerido"})),
            )
                .into_response();
        }
    };

    let claims = match jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    ) {
        Ok(d) => d.claims,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Token inválido"})),
            )
                .into_response();
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Usuario inválido"})),
            )
                .into_response();
        }
    };

    ws.on_upgrade(move |socket| handle_socket(socket, user_id, state.ws_hub))
        .into_response()
}

async fn handle_socket(
    socket: WebSocket,
    user_id: Uuid,
    hub: std::sync::Arc<crate::ws::hub::WsHub>,
) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = hub.subscribe(user_id).await;

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(_msg)) = receiver.next().await {
            // Client messages are handled via HTTP endpoints
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

async fn list_messages(claims: Claims, State(state): State<AppState>) -> NotifResult<Json<Value>> {
    require_any_role(
        &claims,
        &[
            "Administrador",
            "Sostenedor",
            "Director",
            "UTP",
            "Profesor",
            "Apoderado",
        ],
    )?;

    let user_id: Uuid = claims.sub.parse().map_err(|_| NotifError::Unauthorized)?;

    let messages = sqlx::query_as::<_, schoolccb_common::communication::Message>(
        r#"
        SELECT id, sender_id, receiver_id, subject, body, read, created_at
        FROM messages WHERE receiver_id = $1 OR sender_id = $1
        ORDER BY created_at DESC LIMIT 50
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(
        json!({ "messages": messages, "total": messages.len() }),
    ))
}

async fn send_message(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::communication::CreateMessagePayload>,
) -> NotifResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;

    let sender_id: Uuid = claims.sub.parse().map_err(|_| NotifError::Unauthorized)?;

    if payload.subject.trim().is_empty() || payload.body.trim().is_empty() {
        return Err(NotifError::Validation(
            "Asunto y cuerpo son obligatorios".into(),
        ));
    }

    let recipients: Vec<Uuid> = resolve_recipients(&state.pool, &payload.audience).await?;

    if recipients.is_empty() {
        return Err(NotifError::Validation(
            "No hay destinatarios para la audiencia seleccionada".into(),
        ));
    }

    let mut sent: Vec<schoolccb_common::communication::Message> = vec![];
    for recv_id in &recipients {
        let id = Uuid::new_v4();
        let msg = sqlx::query_as::<_, schoolccb_common::communication::Message>(
            r#"
            INSERT INTO messages (id, sender_id, receiver_id, subject, body)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, sender_id, receiver_id, subject, body, read, created_at
            "#,
        )
        .bind(id)
        .bind(sender_id)
        .bind(recv_id)
        .bind(&payload.subject)
        .bind(&payload.body)
        .fetch_one(&state.pool)
        .await?;

        state.ws_hub.broadcast_to(
            recv_id,
            &json!({
                "type": "new_message",
                "receiver_id": recv_id,
                "message": &msg
            })
            .to_string(),
        ).await;

        sent.push(msg);
    }

    Ok(Json(json!({ "messages": sent, "total": sent.len() })))
}

async fn resolve_recipients(
    pool: &sqlx::PgPool,
    audience: &schoolccb_common::communication::AudienceTarget,
) -> Result<Vec<Uuid>, NotifError> {
    match audience {
        schoolccb_common::communication::AudienceTarget::User(uid) => Ok(vec![*uid]),
        schoolccb_common::communication::AudienceTarget::Course(course_id) => {
            let rows: Vec<(Uuid,)> = sqlx::query_as(
                "SELECT DISTINCT u.id FROM users u \
                 JOIN students s ON s.rut = u.rut \
                 JOIN enrollments e ON e.student_id = s.id \
                 WHERE e.course_id = $1 AND e.active = true AND u.role = 'Alumno'",
            )
            .bind(course_id)
            .fetch_all(pool)
            .await?;
            Ok(rows.into_iter().map(|r| r.0).collect())
        }
        schoolccb_common::communication::AudienceTarget::AllStudents => {
            let rows: Vec<(Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE role = 'Alumno'")
                .fetch_all(pool)
                .await?;
            Ok(rows.into_iter().map(|r| r.0).collect())
        }
        schoolccb_common::communication::AudienceTarget::AllTeachers => {
            let rows: Vec<(Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE role = 'Profesor'")
                .fetch_all(pool)
                .await?;
            Ok(rows.into_iter().map(|r| r.0).collect())
        }
        schoolccb_common::communication::AudienceTarget::AllStaff => {
            let rows: Vec<(Uuid,)> = sqlx::query_as(
                "SELECT id FROM users WHERE role IN ('Administrador', 'Sostenedor', 'Director', 'UTP')"
            )
            .fetch_all(pool)
            .await?;
            Ok(rows.into_iter().map(|r| r.0).collect())
        }
        _ => Err(NotifError::Validation("unsupported audience target".into())),
    }
}

async fn get_message(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> NotifResult<Json<Value>> {
    require_any_role(
        &claims,
        &[
            "Administrador",
            "Sostenedor",
            "Director",
            "UTP",
            "Profesor",
            "Apoderado",
        ],
    )?;

    let msg = sqlx::query_as::<_, schoolccb_common::communication::Message>(
        "SELECT id, sender_id, receiver_id, subject, body, read, created_at FROM messages WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(NotifError::NotFound("Mensaje no encontrado".into()))?;

    Ok(Json(json!({ "message": msg })))
}

async fn mark_read(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> NotifResult<Json<Value>> {
    require_any_role(
        &claims,
        &[
            "Administrador",
            "Sostenedor",
            "Director",
            "UTP",
            "Profesor",
            "Apoderado",
        ],
    )?;

    let result = sqlx::query("UPDATE messages SET read = true WHERE id = $1 AND read = false")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({ "updated": result.rows_affected() > 0 })))
}

async fn unread_count(claims: Claims, State(state): State<AppState>) -> NotifResult<Json<Value>> {
    let user_id: Uuid = claims.sub.parse().map_err(|_| NotifError::Unauthorized)?;

    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM messages WHERE receiver_id = $1 AND read = false")
            .bind(user_id)
            .fetch_one(&state.pool)
            .await?;

    Ok(Json(json!({ "unread": count.0 })))
}

async fn list_interviews(
    claims: Claims,
    State(state): State<AppState>,
) -> NotifResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;

    let interviews = sqlx::query_as::<_, schoolccb_common::communication::InterviewLog>(
        r#"
        SELECT il.id, il.student_id, il.teacher_id, il.date, il.reason, il.notes, il.follow_up, il.created_at
        FROM interview_logs il
        ORDER BY il.date DESC LIMIT 50
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(
        json!({ "interviews": interviews, "total": interviews.len() }),
    ))
}

async fn create_interview(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::communication::CreateInterviewPayload>,
) -> NotifResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor"])?;

    let teacher_id: Uuid = claims.sub.parse().map_err(|_| NotifError::Unauthorized)?;

    if payload.reason.trim().is_empty() || payload.notes.trim().is_empty() {
        return Err(NotifError::Validation(
            "Motivo y notas son obligatorios".into(),
        ));
    }

    let id = Uuid::new_v4();
    let date = payload
        .date
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let result = sqlx::query_as::<_, schoolccb_common::communication::InterviewLog>(
        r#"
        INSERT INTO interview_logs (id, student_id, teacher_id, date, reason, notes, follow_up)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, student_id, teacher_id, date, reason, notes, follow_up, created_at
        "#,
    )
    .bind(id)
    .bind(payload.student_id)
    .bind(teacher_id)
    .bind(date)
    .bind(&payload.reason)
    .bind(&payload.notes)
    .bind(&payload.follow_up)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "interview": result })))
}

async fn get_interview(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> NotifResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;

    let interview = sqlx::query_as::<_, schoolccb_common::communication::InterviewLog>(
        "SELECT id, student_id, teacher_id, date, reason, notes, follow_up, created_at FROM interview_logs WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(NotifError::NotFound("Entrevista no encontrada".into()))?;

    Ok(Json(json!({ "interview": interview })))
}

async fn update_interview(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::communication::UpdateInterviewPayload>,
) -> NotifResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor"])?;

    let existing = sqlx::query_as::<_, schoolccb_common::communication::InterviewLog>(
        "SELECT id, student_id, teacher_id, date, reason, notes, follow_up, created_at FROM interview_logs WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(NotifError::NotFound("Entrevista no encontrada".into()))?;

    let reason = payload.reason.unwrap_or(existing.reason);
    let notes = payload.notes.unwrap_or(existing.notes);
    let follow_up = payload.follow_up.or(existing.follow_up);

    let result = sqlx::query_as::<_, schoolccb_common::communication::InterviewLog>(
        r#"
        UPDATE interview_logs SET reason = $1, notes = $2, follow_up = $3
        WHERE id = $4
        RETURNING id, student_id, teacher_id, date, reason, notes, follow_up, created_at
        "#,
    )
    .bind(&reason)
    .bind(&notes)
    .bind(&follow_up)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "interview": result })))
}

async fn delete_interview(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> NotifResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP"])?;

    let result = sqlx::query("DELETE FROM interview_logs WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(NotifError::NotFound("Entrevista no encontrada".into()));
    }

    Ok(Json(
        json!({ "message": "Entrevista eliminada correctamente" }),
    ))
}

async fn interviews_by_student(
    claims: Claims,
    State(state): State<AppState>,
    Path(student_id): Path<Uuid>,
) -> NotifResult<Json<Value>> {
    require_any_role(
        &claims,
        &[
            "Administrador",
            "Sostenedor",
            "Director",
            "UTP",
            "Profesor",
            "Apoderado",
        ],
    )?;

    let interviews = sqlx::query_as::<_, schoolccb_common::communication::InterviewLog>(
        "SELECT id, student_id, teacher_id, date, reason, notes, follow_up, created_at FROM interview_logs WHERE student_id = $1 ORDER BY date DESC",
    )
    .bind(student_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "interviews": interviews })))
}

#[derive(Deserialize)]
struct SendNotificationPayload {
    user_id: Uuid,
    title: String,
    body: Option<String>,
    notification_type: Option<String>,
}

async fn send_notification(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<SendNotificationPayload>,
) -> NotifResult<Json<Value>> {
    require_any_role(&claims, &["GerenteGeneral", "Administrador", "Sostenedor"])?;

    let id = Uuid::new_v4();
    let ntype = payload.notification_type.unwrap_or_else(|| "info".into());

    sqlx::query(
        "INSERT INTO notifications (id, user_id, title, body, notification_type)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(payload.user_id)
    .bind(&payload.title)
    .bind(&payload.body)
    .bind(&ntype)
    .execute(&state.pool)
    .await?;

    let notif = json!({
        "id": id,
        "title": payload.title,
        "body": payload.body,
        "type": ntype,
        "read": false,
        "created_at": chrono::Utc::now(),
    });

    state.ws_hub.broadcast_to(&payload.user_id, &notif.to_string()).await;

    Ok(Json(json!({"id": id, "message": "Notificación enviada"})))
}

async fn list_notifications(
    claims: Claims,
    State(state): State<AppState>,
) -> NotifResult<Json<Value>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| NotifError::Unauthorized)?;

    let notifs: Vec<Value> = sqlx::query_as::<_, (Uuid, String, Option<String>, String, bool, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, title, body, notification_type, read, created_at
         FROM notifications WHERE user_id = $1
         ORDER BY created_at DESC LIMIT 50",
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await.unwrap_or_default()
    .into_iter()
    .map(|(id, title, body, ntype, read, created)| {
        json!({"id": id, "title": title, "body": body, "type": ntype, "read": read, "created_at": created})
    })
    .collect();

    Ok(Json(json!({"notifications": notifs})))
}

async fn unread_notification_count(
    claims: Claims,
    State(state): State<AppState>,
) -> NotifResult<Json<Value>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| NotifError::Unauthorized)?;

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND read = false",
    )
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .unwrap_or((0,));

    Ok(Json(json!({"unread": count.0})))
}

async fn mark_notification_read(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> NotifResult<Json<Value>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| NotifError::Unauthorized)?;

    sqlx::query(
        "UPDATE notifications SET read = true WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"message": "Notificación marcada como leída"})))
}
