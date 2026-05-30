use axum::{Json, Router, extract::{Query, State}, routing::get};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{SisError, SisResult};
use crate::routes::students::Claims;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/dashboard/summary", get(summary))
        .route("/api/dashboard/attendance-today", get(attendance_today))
        .route("/api/dashboard/student-alerts", get(student_alerts))
        .route("/api/dashboard/agenda", get(agenda))
        .route("/api/school/dashboard/attendance-trends", get(attendance_trends))
        .route("/api/school/dashboard/grades-distribution", get(grades_distribution))
        .route("/api/school/dashboard/finance-summary", get(finance_summary))
        .route("/api/school/dashboard/teacher-performance", get(teacher_performance))
        .route("/api/school/dashboard/top-alerts", get(school_top_alerts))
}

fn school_and_corp_id(claims: &Claims) -> (Option<Uuid>, Option<Uuid>) {
    let school_id = claims.school_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let corporation_id = claims.corporation_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    (school_id, corporation_id)
}

async fn summary(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "dashboard",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;
    let (school_id, corporation_id) = school_and_corp_id(&claims);
    let data = crate::routes::models::get_dashboard_summary(&state.pool, school_id, corporation_id).await?;
    Ok(Json(serde_json::to_value(data).map_err(|e| SisError::Internal(e.to_string()))?))
}

async fn attendance_today(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "dashboard",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;
    let (school_id, corporation_id) = school_and_corp_id(&claims);
    let today = chrono::Utc::now().date_naive().to_string();
    let records = crate::routes::models::get_attendance_today(&state.pool, &today, school_id, corporation_id).await?;

    let total = records.len() as i64;
    let present = records
        .iter()
        .filter(|r| r.status == schoolccb_common::attendance::AttendanceStatus::Presente)
        .count() as i64;
    let absent = records
        .iter()
        .filter(|r| r.status == schoolccb_common::attendance::AttendanceStatus::Ausente)
        .count() as i64;
    let late = records
        .iter()
        .filter(|r| r.status == schoolccb_common::attendance::AttendanceStatus::Atraso)
        .count() as i64;
    let justified = records.iter().filter(|r| r.status.es_justificado()).count() as i64;

    Ok(Json(serde_json::json!({
        "date": today,
        "total_students": total,
        "present": present,
        "absent": absent,
        "late": late,
        "justified": justified,
        "attendance_percentage": if total > 0 {
            ((present + justified) as f64 / total as f64) * 100.0
        } else {
            100.0
        }
    })))
}

async fn student_alerts(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "dashboard",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;
    let (school_id, corporation_id) = school_and_corp_id(&claims);
    let alerts = crate::routes::models::get_attendance_alerts(&state.pool, school_id, corporation_id).await?;
    Ok(Json(serde_json::json!({ "alerts": alerts })))
}

async fn agenda(_claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    let today = chrono::Utc::now().date_naive().to_string();
    let events = crate::routes::models::get_agenda_events(&state.pool, &today).await?;
    Ok(Json(serde_json::json!({ "events": events })))
}

#[derive(serde::Deserialize)]
struct TrendQuery {
    months: Option<i32>,
}

async fn attendance_trends(
    claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<TrendQuery>,
) -> SisResult<Json<Value>> {
    let school_id = claims.school_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let months = q.months.unwrap_or(12);

    let trends: Vec<Value> = sqlx::query_as::<_, (String, f64)>(
        "SELECT to_char(date_trunc('month', ar.date), 'YYYY-MM') as month,
                AVG(CASE WHEN ar.status = 'present' THEN 100.0 ELSE 0.0 END) as pct
         FROM attendance_records ar
         JOIN students s ON s.id = ar.student_id
         JOIN enrollments e ON e.student_id = s.id AND ($1::uuid IS NULL OR e.school_id = $1)
         WHERE ar.date >= NOW() - ($2 || ' months')::interval
         GROUP BY date_trunc('month', ar.date)
         ORDER BY month",
    )
    .bind(school_id)
    .bind(months.to_string())
    .fetch_all(&state.pool)
    .await.unwrap_or_default()
    .into_iter()
    .map(|(m, p)| json!({"month": m, "attendance": format!("{:.1}", p)}))
    .collect();

    let avg: f64 = if !trends.is_empty() {
        trends.iter().filter_map(|t| t["attendance"].as_str().and_then(|s| s.parse::<f64>().ok())).sum::<f64>() / trends.len() as f64
    } else { 0.0 };

    Ok(Json(json!({"trends": trends, "average": format!("{:.1}", avg)})))
}

async fn grades_distribution(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    let school_id = claims.school_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());

    let distribution: Vec<(String, i64)> = sqlx::query_as(
        "SELECT
            CASE
                WHEN g.value >= 6.0 THEN '6.0-7.0'
                WHEN g.value >= 5.0 THEN '5.0-5.9'
                WHEN g.value >= 4.0 THEN '4.0-4.9'
                ELSE '1.0-3.9'
            END as range,
            COUNT(*) as count
         FROM grades g
         JOIN enrollments e ON e.id = g.enrollment_id AND ($1::uuid IS NULL OR e.school_id = $1)
         WHERE g.value IS NOT NULL
         GROUP BY range
         ORDER BY range DESC",
    )
    .bind(school_id)
    .fetch_all(&state.pool)
    .await.unwrap_or_default();

    let total: i64 = distribution.iter().map(|(_, c)| c).sum();
    let avg: f64 = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(AVG(g.value), 0) FROM grades g
         JOIN enrollments e ON e.id = g.enrollment_id AND ($1::uuid IS NULL OR e.school_id = $1)
         WHERE g.value IS NOT NULL",
    )
    .bind(school_id)
    .fetch_one(&state.pool)
    .await.unwrap_or(0.0);

    Ok(Json(json!({
        "distribution": distribution.into_iter().map(|(r, c)| json!({"range": r, "count": c})).collect::<Vec<_>>(),
        "total_grades": total,
        "average": format!("{:.1}", avg),
    })))
}

async fn finance_summary(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    let school_id = claims.school_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());

    let monthly_revenue: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount), 0) FROM fees
         WHERE ($1::uuid IS NULL OR school_id = $1)
           AND paid = true
           AND paid_at >= NOW() - INTERVAL '30 days'",
    )
    .bind(school_id)
    .fetch_one(&state.pool)
    .await.unwrap_or(0.0);

    let total_pending: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount), 0) FROM fees
         WHERE ($1::uuid IS NULL OR school_id = $1) AND paid = false",
    )
    .bind(school_id)
    .fetch_one(&state.pool)
    .await.unwrap_or(0.0);

    let total_collected: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount), 0) FROM fees
         WHERE ($1::uuid IS NULL OR school_id = $1) AND paid = true",
    )
    .bind(school_id)
    .fetch_one(&state.pool)
    .await.unwrap_or(0.0);

    let pending_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM fees
         WHERE ($1::uuid IS NULL OR school_id = $1) AND paid = false",
    )
    .bind(school_id)
    .fetch_one(&state.pool)
    .await.unwrap_or(0);

    Ok(Json(json!({
        "monthly_revenue": monthly_revenue,
        "total_pending": total_pending,
        "total_collected": total_collected,
        "pending_count": pending_count,
    })))
}

async fn teacher_performance(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    let school_id = claims.school_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());

    let teachers: Vec<Value> = sqlx::query_as::<_, (Uuid, String, Option<f64>, Option<f64>)>(
        "SELECT e.id, e.full_name,
                AVG(g.value)::float8 as avg_grade,
                COUNT(*)::float8 as total_grades
         FROM employees e
         JOIN course_subjects cs ON cs.teacher_id = e.id
         JOIN grades g ON g.course_subject_id = cs.id
         WHERE ($1::uuid IS NULL OR e.school_id = $1)
           AND e.role = 'Profesor'
           AND e.active = true
         GROUP BY e.id, e.full_name
         ORDER BY avg_grade DESC NULLS LAST
         LIMIT 20",
    )
    .bind(school_id)
    .fetch_all(&state.pool)
    .await.unwrap_or_default()
    .into_iter()
    .map(|(id, name, avg, count)| json!({
        "id": id,
        "name": name,
        "avg_grade": avg.map(|a| format!("{:.1}", a)).unwrap_or_else(|| "-".into()),
        "total_grades": count.unwrap_or(0.0) as i64,
    }))
    .collect();

    Ok(Json(json!({"teachers": teachers})))
}

async fn school_top_alerts(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    let school_id = claims.school_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());

    let low_attendance: Vec<Value> = sqlx::query_as::<_, (Uuid, String, String, f64)>(
        "SELECT s.id, s.full_name, c.name as course,
                AVG(CASE WHEN ar.status = 'present' THEN 100.0 ELSE 0.0 END) as pct
         FROM attendance_records ar
         JOIN students s ON s.id = ar.student_id
         JOIN enrollments e ON e.student_id = s.id AND ($1::uuid IS NULL OR e.school_id = $1)
         JOIN courses c ON c.id = e.course_id
         WHERE ar.date >= NOW() - INTERVAL '15 days'
         GROUP BY s.id, s.full_name, c.name
         HAVING AVG(CASE WHEN ar.status = 'present' THEN 100.0 ELSE 0.0 END) < 80.0
         ORDER BY pct ASC
         LIMIT 10",
    )
    .bind(school_id)
    .fetch_all(&state.pool)
    .await.unwrap_or_default()
    .into_iter()
    .map(|(id, name, course, pct)| json!({
        "type": "low_attendance",
        "student_id": id,
        "student_name": name,
        "course": course,
        "attendance": format!("{:.1}", pct),
        "severity": if pct < 60.0 { "critical" } else if pct < 70.0 { "high" } else { "medium" },
        "message": format!("{} tiene {}% de asistencia (últimos 15 días)", name, format!("{:.1}", pct)),
    }))
    .collect();

    Ok(Json(json!({"alerts": low_attendance})))
}
