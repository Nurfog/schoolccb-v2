use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::error::SisResult;
use schoolccb_common::auth::Claims;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/corporation/dashboard/summary", get(summary))
        .route("/api/corporation/dashboard/schools", get(school_kpis))
        .route("/api/corporation/dashboard/comparisons", get(comparisons))
        .route("/api/corporation/dashboard/trends", get(trends))
        .route("/api/corporation/dashboard/alerts", get(alerts))
        .route("/api/corporation/dashboard/license", get(license_summary))
}

fn require_sostenedor(claims: &Claims) -> Result<(), crate::error::SisError> {
    if claims.role == "Sostenedor" || claims.role == "GerenteGeneral" || claims.role == "AdminGlobal" {
        return Ok(());
    }
    Err(crate::error::SisError::Forbidden("Se requiere rol Sostenedor".into()))
}

async fn summary(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    require_sostenedor(&claims)?;
    let corp_id = claims.corporation_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    let total_schools: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM schools WHERE corporation_id = $1 AND active = true",
    )
    .bind(corp_id)
    .fetch_one(&state.pool)
    .await.unwrap_or(0);

    let total_students: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM students s
         JOIN enrollments e ON e.student_id = s.id
         JOIN schools sch ON sch.id = e.school_id
         WHERE sch.corporation_id = $1 AND e.active = true",
    )
    .bind(corp_id)
    .fetch_one(&state.pool)
    .await.unwrap_or(0);

    let total_teachers: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM employees e
         JOIN schools sch ON sch.id = e.school_id
         WHERE sch.corporation_id = $1 AND e.role = 'Profesor' AND e.active = true",
    )
    .bind(corp_id)
    .fetch_one(&state.pool)
    .await.unwrap_or(0);

    let total_employees: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM employees e
         JOIN schools sch ON sch.id = e.school_id
         WHERE sch.corporation_id = $1 AND e.active = true",
    )
    .bind(corp_id)
    .fetch_one(&state.pool)
    .await.unwrap_or(0);

    // Attendance average across all schools
    let avg_attendance: f64 = sqlx::query_scalar(
        "SELECT COALESCE(AVG(a.percentage), 0) FROM (
            SELECT
                COUNT(CASE WHEN status = 'present' THEN 1 END)::float / NULLIF(COUNT(*), 0) * 100 as percentage
            FROM attendance_records ar
            JOIN students s ON s.id = ar.student_id
            JOIN enrollments e ON e.student_id = s.id
            JOIN schools sch ON sch.id = e.school_id
            WHERE sch.corporation_id = $1 AND ar.date >= NOW() - INTERVAL '30 days'
            GROUP BY ar.student_id
        ) a",
    )
    .bind(corp_id)
    .fetch_one(&state.pool)
    .await.unwrap_or(0.0);

    // Average grades
    let avg_grades: f64 = sqlx::query_scalar(
        "SELECT COALESCE(AVG(g.value), 0) FROM grades g
         JOIN enrollments e ON e.id = g.enrollment_id
         JOIN schools sch ON sch.id = e.school_id
         WHERE sch.corporation_id = $1 AND g.value IS NOT NULL",
    )
    .bind(corp_id)
    .fetch_one(&state.pool)
    .await.unwrap_or(0.0);

    // Active licenses
    let active_licenses: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM corporation_licenses
         WHERE corporation_id = $1 AND status = 'active'",
    )
    .bind(corp_id)
    .fetch_one(&state.pool)
    .await.unwrap_or(0);

    // Expiring licenses (< 30 days)
    let expiring_licenses: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM corporation_licenses
         WHERE corporation_id = $1 AND status = 'active'
           AND end_date IS NOT NULL
           AND end_date <= NOW()::date + INTERVAL '30 days'",
    )
    .bind(corp_id)
    .fetch_one(&state.pool)
    .await.unwrap_or(0);

    // Monthly revenue
    let monthly_revenue: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount), 0) FROM license_payments lp
         JOIN corporation_licenses cl ON cl.id = lp.corporation_license_id
         WHERE cl.corporation_id = $1
           AND lp.status = 'completed'
           AND lp.paid_at >= NOW() - INTERVAL '30 days'",
    )
    .bind(corp_id)
    .fetch_one(&state.pool)
    .await.unwrap_or(0.0);

    Ok(Json(json!({
        "total_schools": total_schools,
        "total_students": total_students,
        "total_teachers": total_teachers,
        "total_employees": total_employees,
        "avg_attendance": format!("{:.1}", avg_attendance),
        "avg_grades": format!("{:.1}", avg_grades),
        "active_licenses": active_licenses,
        "expiring_licenses": expiring_licenses,
        "monthly_revenue": monthly_revenue,
    })))
}

#[derive(serde::Deserialize)]
struct SchoolQuery {
    school_id: Option<Uuid>,
}

async fn school_kpis(
    claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<SchoolQuery>,
) -> SisResult<Json<Value>> {
    require_sostenedor(&claims)?;
    let corp_id = claims.corporation_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    let rows = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT id, name FROM schools WHERE corporation_id = $1 AND active = true ORDER BY name",
    )
    .bind(corp_id)
    .fetch_all(&state.pool)
    .await?;

    let mut schools = Vec::new();
    for (id, name) in rows {
        if let Some(sid) = q.school_id {
            if sid != id { continue; }
        }
        let students: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM enrollments WHERE school_id = $1 AND active = true",
        ).bind(id).fetch_one(&state.pool).await.unwrap_or(0);

        let teachers: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM employees WHERE school_id = $1 AND role = 'Profesor' AND active = true",
        ).bind(id).fetch_one(&state.pool).await.unwrap_or(0);

        let attendance: f64 = sqlx::query_scalar(
            "SELECT COALESCE(AVG(CASE WHEN status = 'present' THEN 100.0 ELSE 0.0 END), 0)
             FROM attendance_records ar
             JOIN students s ON s.id = ar.student_id
             JOIN enrollments e ON e.student_id = s.id AND e.school_id = $1
             WHERE ar.date >= NOW() - INTERVAL '30 days'",
        ).bind(id).fetch_one(&state.pool).await.unwrap_or(0.0);

        let avg_grade: f64 = sqlx::query_scalar(
            "SELECT COALESCE(AVG(g.value), 0) FROM grades g
             JOIN enrollments e ON e.id = g.enrollment_id AND e.school_id = $1
             WHERE g.value IS NOT NULL",
        ).bind(id).fetch_one(&state.pool).await.unwrap_or(0.0);

        schools.push(json!({
            "id": id,
            "name": name,
            "students": students,
            "teachers": teachers,
            "attendance": format!("{:.1}", attendance),
            "avg_grade": format!("{:.1}", avg_grade),
        }));
    }

    Ok(Json(json!({"schools": schools})))
}

async fn comparisons(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    require_sostenedor(&claims)?;
    let corp_id = claims.corporation_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    let rows = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT id, name FROM schools WHERE corporation_id = $1 AND active = true ORDER BY name",
    )
    .bind(corp_id)
    .fetch_all(&state.pool)
    .await?;

    let mut school_data = Vec::new();
    for (id, name) in rows {
        let students: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM enrollments WHERE school_id = $1 AND active = true",
        ).bind(id).fetch_one(&state.pool).await.unwrap_or(0);

        let attendance: f64 = sqlx::query_scalar(
            "SELECT COALESCE(AVG(CASE WHEN status = 'present' THEN 100.0 ELSE 0.0 END), 0)
             FROM attendance_records ar
             JOIN students s ON s.id = ar.student_id
             JOIN enrollments e ON e.student_id = s.id AND e.school_id = $1
             WHERE ar.date >= NOW() - INTERVAL '30 days'",
        ).bind(id).fetch_one(&state.pool).await.unwrap_or(0.0);

        let avg_grade: f64 = sqlx::query_scalar(
            "SELECT COALESCE(AVG(g.value), 0) FROM grades g
             JOIN enrollments e ON e.id = g.enrollment_id AND e.school_id = $1
             WHERE g.value IS NOT NULL",
        ).bind(id).fetch_one(&state.pool).await.unwrap_or(0.0);

        school_data.push(json!({
            "school_id": id,
            "school_name": name,
            "total_students": students,
            "attendance_pct": format!("{:.1}", attendance),
            "avg_grade": format!("{:.1}", avg_grade),
        }));
    }

    Ok(Json(json!({"comparisons": school_data})))
}

async fn trends(
    claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<SchoolQuery>,
) -> SisResult<Json<Value>> {
    require_sostenedor(&claims)?;
    let corp_id = claims.corporation_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    let school_filter = match (corp_id, q.school_id) {
        (Some(cid), Some(sid)) => format!("AND sch.id = '{sid}' AND sch.corporation_id = '{cid}'"),
        (Some(cid), None) => format!("AND sch.corporation_id = '{cid}'"),
        _ => "AND 1=0".into(),
    };

    // Monthly enrollment growth (last 12 months)
    let enrollment_trend: Vec<Value> = sqlx::query_as::<_, (String, i64)>(&format!(
        "SELECT to_char(date_trunc('month', e.created_at), 'YYYY-MM') as month, COUNT(*) as count
         FROM enrollments e
         JOIN schools sch ON sch.id = e.school_id
         WHERE e.created_at >= NOW() - INTERVAL '12 months' {school_filter}
         GROUP BY date_trunc('month', e.created_at)
         ORDER BY month"
    ))
    .fetch_all(&state.pool)
    .await.unwrap_or_default()
    .into_iter()
    .map(|(m, c)| json!({"month": m, "enrollments": c}))
    .collect();

    // Monthly attendance trend
    let attendance_trend: Vec<Value> = sqlx::query_as::<_, (String, f64)>(&format!(
        "SELECT to_char(date_trunc('month', ar.date), 'YYYY-MM') as month,
                AVG(CASE WHEN ar.status = 'present' THEN 100.0 ELSE 0.0 END) as pct
         FROM attendance_records ar
         JOIN students s ON s.id = ar.student_id
         JOIN enrollments e ON e.student_id = s.id
         JOIN schools sch ON sch.id = e.school_id
         WHERE ar.date >= NOW() - INTERVAL '12 months' {school_filter}
         GROUP BY date_trunc('month', ar.date)
         ORDER BY month"
    ))
    .fetch_all(&state.pool)
    .await.unwrap_or_default()
    .into_iter()
    .map(|(m, p)| json!({"month": m, "attendance": format!("{:.1}", p)}))
    .collect();

    Ok(Json(json!({
        "enrollment_trend": enrollment_trend,
        "attendance_trend": attendance_trend,
    })))
}

async fn alerts(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    require_sostenedor(&claims)?;
    let corp_id = claims.corporation_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    let mut alerts_list: Vec<Value> = Vec::new();

    // Schools with low attendance (< 85%)
    let low_attendance: Vec<(Uuid, String, f64)> = sqlx::query_as(
        "SELECT sch.id, sch.name,
                AVG(CASE WHEN ar.status = 'present' THEN 100.0 ELSE 0.0 END) as pct
         FROM attendance_records ar
         JOIN students s ON s.id = ar.student_id
         JOIN enrollments e ON e.student_id = s.id
         JOIN schools sch ON sch.id = e.school_id
         WHERE sch.corporation_id = $1 AND ar.date >= NOW() - INTERVAL '30 days'
         GROUP BY sch.id, sch.name
         HAVING AVG(CASE WHEN ar.status = 'present' THEN 100.0 ELSE 0.0 END) < 85.0
         ORDER BY pct ASC",
    )
    .bind(corp_id)
    .fetch_all(&state.pool)
    .await.unwrap_or_default();

    for (id, name, pct) in &low_attendance {
        alerts_list.push(json!({
            "type": "low_attendance",
            "severity": if *pct < 75.0 { "critical" } else { "high" },
            "school_id": id,
            "school_name": name,
            "message": format!("Asistencia por debajo del 85%: {pct:.1}%"),
            "value": format!("{pct:.1}%"),
        }));
    }

    // Expiring licenses
    let expiring: Vec<(Uuid, String, String)> = sqlx::query_as(
        "SELECT cl.id, lp.name, cl.end_date::text
         FROM corporation_licenses cl
         JOIN license_plans lp ON lp.id = cl.plan_id
         WHERE cl.corporation_id = $1 AND cl.status = 'active'
           AND cl.end_date IS NOT NULL
           AND cl.end_date <= NOW()::date + INTERVAL '30 days'
         ORDER BY cl.end_date ASC",
    )
    .bind(corp_id)
    .fetch_all(&state.pool)
    .await.unwrap_or_default();

    for (_id, plan, end_date) in &expiring {
        alerts_list.push(json!({
            "type": "expiring_license",
            "severity": "high",
            "message": format!("Licencia {plan} vence el {end_date}"),
            "value": end_date,
        }));
    }

    Ok(Json(json!({"alerts": alerts_list})))
}

async fn license_summary(
    claims: Claims,
    State(state): State<AppState>,
) -> SisResult<Json<Value>> {
    require_sostenedor(&claims)?;
    let corp_id = claims.corporation_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    let license = sqlx::query_as::<_, (Uuid, String, String, Option<String>, Option<String>, String, bool)>(
        "SELECT cl.id, lp.name, cl.status, cl.start_date::text, cl.end_date::text,
                COALESCE(lp.price_monthly, 0)::text, cl.auto_renew
         FROM corporation_licenses cl
         JOIN license_plans lp ON lp.id = cl.plan_id
         WHERE cl.corporation_id = $1 AND cl.status = 'active'
         LIMIT 1",
    )
    .bind(corp_id)
    .fetch_optional(&state.pool)
    .await?;

    match license {
        Some((id, plan_name, status, start, end, price, renew)) => {
            let days_left = end.as_deref().and_then(|d| {
                chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()
                    .map(|d| (d - chrono::Utc::now().date_naive()).num_days())
            }).unwrap_or(0);

            let modules: Vec<Value> = sqlx::query_as::<_, (String, String, bool)>(
                "SELECT pm.module_key, pm.module_name, pm.included
                 FROM plan_modules pm
                 JOIN corporation_licenses cl ON cl.plan_id = pm.plan_id
                 WHERE cl.id = $1
                 ORDER BY pm.module_key",
            )
            .bind(id)
            .fetch_all(&state.pool)
            .await.unwrap_or_default()
            .into_iter()
            .map(|(k, n, inc)| json!({"key": k, "name": n, "included": inc}))
            .collect();

            Ok(Json(json!({
                "plan_name": plan_name,
                "status": status,
                "start_date": start,
                "end_date": end,
                "days_remaining": days_left,
                "price": price,
                "auto_renew": renew,
                "modules": modules,
            })))
        }
        None => Ok(Json(json!({
            "plan_name": null,
            "status": "no_license",
            "modules": []
        }))),
    }
}
