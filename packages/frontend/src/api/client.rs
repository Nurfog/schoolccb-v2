use serde_json::{Value, json};
use std::sync::OnceLock;

fn base_url() -> String {
    web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_else(|| "http://localhost:8080".to_string())
}

fn abs_url(endpoint: &str) -> String {
    let base = base_url();
    let ep = endpoint.trim_start_matches('/');
    format!("{}/{}", base, ep)
}

fn client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .build()
            .expect("Failed to create HTTP client")
    })
}

fn get_token() -> Option<String> {
    None
}

pub fn remove_token() {
    // HttpOnly cookie — can't clear from JS. Call logout API instead.
}

fn auth_header() -> Option<String> {
    None
}

async fn request_inner(method: &str, endpoint: &str, body: Option<&Value>) -> Result<Value, String> {
    let url = abs_url(endpoint);
    let mut req = match method {
        "GET" => client().get(&url),
        "POST" => client().post(&url),
        "PUT" => client().put(&url),
        "DELETE" => client().delete(&url),
        _ => return Err(format!("Invalid method: {method}")),
    };
    if let Some(b) = body {
        req = req.json(b);
    }
    if let Some(auth) = auth_header() {
        req = req.header("Authorization", auth);
    }
    let resp = req.send().await.map_err(|e| format!("Error: {e}"))?;
    let status = resp.status();
    let body: Value = resp.json().await.map_err(|e| format!("Parse: {e}"))?;
    if status == 502 {
        return Err("502".to_string());
    }
    Ok(body)
}

async fn request(method: &str, endpoint: &str, body: Option<&Value>) -> Result<Value, String> {
    let mut last_error = String::new();
    for attempt in 0..3 {
        let result = request_inner(method, endpoint, body).await;
        match result {
            Err(e) if e == "502" && attempt < 2 => {
                last_error = "Servicio temporalmente no disponible".to_string();
            }
            Err(e) => return Err(e),
            Ok(v) => return Ok(v),
        }
    }
    Err(last_error)
}

pub async fn fetch_json(endpoint: &str) -> Result<Value, String> {
    request("GET", endpoint, None).await
}

pub async fn post_json(endpoint: &str, body: &Value) -> Result<Value, String> {
    request("POST", endpoint, Some(body)).await
}

pub async fn put_json(endpoint: &str, body: &Value) -> Result<Value, String> {
    request("PUT", endpoint, Some(body)).await
}

pub async fn delete_json(endpoint: &str) -> Result<Value, String> {
    request("DELETE", endpoint, None).await
}

pub async fn login(email: &str, password: &str) -> Result<Value, String> {
    let body = serde_json::json!({ "email": email, "password": password });
    let resp = client()
        .post(&abs_url("/api/auth/login"))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Error: {e}"))?;

    let result: Value = resp.json().await.map_err(|e| format!("Parse: {e}"))?;
    Ok(result)
}

pub async fn logout() -> Result<Value, String> {
    post_json("/api/auth/logout", &json!({})).await
}

pub async fn exchange_code(code: &str) -> Result<Value, String> {
    post_json("/api/auth/exchange", &json!({"code": code})).await
}

// ─── Dashboard ───
pub async fn fetch_dashboard_summary() -> Result<Value, String> {
    fetch_json("/api/dashboard/summary").await
}
pub async fn fetch_attendance_today() -> Result<Value, String> {
    fetch_json("/api/dashboard/attendance-today").await
}
pub async fn fetch_student_alerts() -> Result<Value, String> {
    fetch_json("/api/dashboard/student-alerts").await
}
pub async fn fetch_agenda() -> Result<Value, String> {
    fetch_json("/api/dashboard/agenda").await
}

// ─── Students ───
pub async fn search_students(query: &str) -> Result<Value, String> {
    let q = urlencoding(query);
    fetch_json(&format!("/api/students?search={q}")).await
}
pub async fn fetch_students(
    grade_level: Option<&str>,
    section: Option<&str>,
    search: Option<&str>,
) -> Result<Value, String> {
    let mut params = vec![];
    if let Some(gl) = grade_level {
        params.push(format!("grade_level={}", urlencoding(gl)));
    }
    if let Some(sec) = section {
        params.push(format!("section={}", urlencoding(sec)));
    }
    if let Some(q) = search {
        params.push(format!("search={}", urlencoding(q)));
    }
    let qs = if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
    };
    fetch_json(&format!("/api/students{}", qs)).await
}
pub async fn fetch_student_full(student_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/students/{}", student_id)).await
}

// ─── Grades ───
pub async fn fetch_subjects() -> Result<Value, String> {
    fetch_json("/api/grades/subjects").await
}
pub async fn fetch_grades_student(
    student_id: &str,
    semester: i32,
    year: i32,
) -> Result<Value, String> {
    fetch_json(&format!(
        "/api/grades/student/{}/{}/{}",
        student_id, semester, year
    ))
    .await
}
pub async fn fetch_student_report(student_id: &str, year: i32) -> Result<Value, String> {
    fetch_json(&format!(
        "/api/grades/reports/student/{}/{}",
        student_id, year
    ))
    .await
}
pub async fn fetch_course_performance(course_id: &str, year: i32) -> Result<Value, String> {
    fetch_json(&format!(
        "/api/grades/reports/course/{}/{}",
        course_id, year
    ))
    .await
}
pub async fn fetch_grades_by_subject(subject_id: &str, year: i32) -> Result<Value, String> {
    fetch_json(&format!("/api/grades/by-subject/{}/{}", subject_id, year)).await
}

// ─── Attendance ───
pub async fn fetch_attendance_monthly(year: i32, month: u32) -> Result<Value, String> {
    fetch_json(&format!("/api/attendance/monthly/{}/{}", year, month)).await
}
pub async fn fetch_attendance_by_course_date(course_id: &str, date: &str) -> Result<Value, String> {
    fetch_json(&format!(
        "/api/attendance/course/{}/date/{}",
        course_id, date
    ))
    .await
}
// ─── Communications ───
pub async fn fetch_interviews_student(student_id: &str) -> Result<Value, String> {
    fetch_json(&format!(
        "/api/communications/interviews/student/{}",
        student_id
    ))
    .await
}

// ─── Finance ───
pub async fn fetch_fees_student(student_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/finance/fees/student/{}", student_id)).await
}
pub async fn fetch_all_fees() -> Result<Value, String> {
    fetch_json("/api/finance/fees").await
}
pub async fn create_fee(payload: &Value) -> Result<Value, String> {
    post_json("/api/finance/fees", payload).await
}
pub async fn mark_fee_paid(fee_id: &str) -> Result<Value, String> {
    put_json(
        &format!("/api/finance/fees/{}", fee_id),
        &serde_json::json!({"paid": true}),
    )
    .await
}
pub async fn delete_fee(fee_id: &str) -> Result<Value, String> {
    delete_json(&format!("/api/finance/fees/{}", fee_id)).await
}
pub async fn fetch_all_payments() -> Result<Value, String> {
    fetch_json("/api/finance/payments").await
}
pub async fn create_payment(payload: &Value) -> Result<Value, String> {
    post_json("/api/finance/payments", payload).await
}
pub async fn fetch_all_scholarships() -> Result<Value, String> {
    fetch_json("/api/finance/scholarships").await
}
pub async fn create_scholarship(payload: &Value) -> Result<Value, String> {
    post_json("/api/finance/scholarships", payload).await
}
pub async fn approve_scholarship(scholarship_id: &str) -> Result<Value, String> {
    put_json(
        &format!("/api/finance/scholarships/{}", scholarship_id),
        &serde_json::json!({}),
    )
    .await
}
pub async fn delete_scholarship(scholarship_id: &str) -> Result<Value, String> {
    delete_json(&format!("/api/finance/scholarships/{}", scholarship_id)).await
}

// ─── Legal Representatives ───
pub async fn fetch_legal_reps(corporation_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/legal-representatives?corporation_id={}", corporation_id)).await
}

// ─── Reports ───
pub async fn fetch_student_certificate(student_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/reports/certificate/student/{}", student_id)).await
}
pub async fn fetch_student_concentration(student_id: &str, year: i32) -> Result<Value, String> {
    fetch_json(&format!(
        "/api/reports/concentration/{}/{}",
        student_id, year
    ))
    .await
}
pub async fn fetch_final_record(course_id: &str, year: i32) -> Result<Value, String> {
    fetch_json(&format!("/api/reports/final-record/{}/{}", course_id, year)).await
}
pub async fn fetch_sige_students() -> Result<Value, String> {
    fetch_json("/api/reports/sige/students").await
}
pub async fn fetch_sige_attendance(year: i32, month: u32) -> Result<Value, String> {
    fetch_json(&format!("/api/reports/sige/attendance/{}/{}", year, month)).await
}

// ─── Corporations & Schools ───
pub async fn fetch_corporations() -> Result<Value, String> {
    fetch_json("/api/corporations").await
}
pub async fn create_corporation(payload: &Value) -> Result<Value, String> {
    post_json("/api/corporations", payload).await
}
pub async fn fetch_schools(corporation_id: Option<&str>) -> Result<Value, String> {
    match corporation_id {
        Some(id) => fetch_json(&format!("/api/schools?corporation_id={}", id)).await,
        None => fetch_json("/api/schools").await,
    }
}
pub async fn create_school(payload: &Value) -> Result<Value, String> {
    post_json("/api/schools", payload).await
}
pub async fn update_school(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/api/schools/{}", id), payload).await
}
pub async fn toggle_school(id: &str) -> Result<Value, String> {
    put_json(&format!("/api/schools/{}/toggle", id), &json!({})).await
}

// ─── Academic Years ───
pub async fn fetch_academic_years() -> Result<Value, String> {
    fetch_json("/api/academic-years").await
}
pub async fn create_academic_year(payload: &Value) -> Result<Value, String> {
    post_json("/api/academic-years", payload).await
}
pub async fn update_academic_year(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/api/academic-years/{}", id), payload).await
}
pub async fn delete_academic_year(id: &str) -> Result<Value, String> {
    delete_json(&format!("/api/academic-years/{}", id)).await
}
pub async fn activate_academic_year(id: &str) -> Result<Value, String> {
    post_json(
        &format!("/api/academic-years/{}/activate", id),
        &serde_json::json!({}),
    )
    .await
}
pub async fn clone_academic_year(payload: &Value) -> Result<Value, String> {
    post_json("/api/academic-years/clone", payload).await
}

// ─── Grade Levels ───
pub async fn fetch_grade_levels() -> Result<Value, String> {
    fetch_json("/api/academic/grade-levels").await
}
pub async fn create_grade_level(payload: &Value) -> Result<Value, String> {
    post_json("/api/academic/grade-levels", payload).await
}
pub async fn update_grade_level(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/api/academic/grade-levels/{}", id), payload).await
}
pub async fn delete_grade_level(id: &str) -> Result<Value, String> {
    delete_json(&format!("/api/academic/grade-levels/{}", id)).await
}
pub async fn import_subjects(payload: &Value) -> Result<Value, String> {
    post_json("/api/grades/subjects/import", payload).await
}

// ─── Admission ───
pub async fn fetch_pipeline_stages() -> Result<Value, String> {
    fetch_json("/api/admission/stages").await
}
pub async fn fetch_prospects() -> Result<Value, String> {
    fetch_json("/api/admission/prospects").await
}
pub async fn fetch_prospect(id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/admission/prospects/{}", id)).await
}
pub async fn create_prospect(payload: &Value) -> Result<Value, String> {
    post_json("/api/admission/prospects", payload).await
}
pub async fn update_prospect(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/api/admission/prospects/{}", id), payload).await
}
pub async fn delete_prospect(id: &str) -> Result<Value, String> {
    delete_json(&format!("/api/admission/prospects/{}", id)).await
}
pub async fn change_prospect_stage(id: &str, stage_id: &str) -> Result<Value, String> {
    put_json(
        &format!("/api/admission/prospects/{}/stage", id),
        &serde_json::json!({ "stage_id": stage_id }),
    )
    .await
}
pub async fn fetch_classrooms() -> Result<Value, String> {
    fetch_json("/api/admission/classrooms").await
}
pub async fn create_classroom(payload: &Value) -> Result<Value, String> {
    post_json("/api/admission/classrooms", payload).await
}
pub async fn update_classroom(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/api/admission/classrooms/{}", id), payload).await
}
pub async fn delete_classroom(id: &str) -> Result<Value, String> {
    delete_json(&format!("/api/admission/classrooms/{}", id)).await
}
pub async fn fetch_audit_logs() -> Result<Value, String> {
    fetch_json("/api/academic/audit-log").await
}
pub async fn fetch_custom_field_definitions(entity_type: &str) -> Result<Value, String> {
    fetch_json(&format!(
        "/api/admission/custom-fields/definitions?entity_type={}",
        entity_type
    ))
    .await
}
pub async fn fetch_custom_field_values(entity_id: &str) -> Result<Value, String> {
    fetch_json(&format!(
        "/api/admission/custom-fields/values/{}",
        entity_id
    ))
    .await
}
pub async fn fetch_my_permissions() -> Result<Value, String> {
    fetch_json("/api/auth/my-permissions").await
}
pub async fn fetch_roles() -> Result<Value, String> {
    fetch_json("/api/roles").await
}
pub async fn create_role(payload: &Value) -> Result<Value, String> {
    post_json("/api/roles", payload).await
}
pub async fn delete_role(id: &str) -> Result<Value, String> {
    delete_json(&format!("/api/roles/{}", id)).await
}
pub async fn update_role_permissions(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/api/roles/{}/permissions", id), payload).await
}
pub async fn fetch_permission_definitions() -> Result<Value, String> {
    fetch_json("/api/permissions/definitions").await
}
pub async fn fetch_user_roles(user_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/users/{}/roles", user_id)).await
}
pub async fn assign_role(user_id: &str, role_id: &str) -> Result<Value, String> {
    post_json(
        &format!("/api/users/{}/roles", user_id),
        &json!({"role_id": role_id}),
    )
    .await
}
pub async fn remove_role(user_id: &str, role_id: &str) -> Result<Value, String> {
    delete_json(&format!("/api/users/{}/roles/{}", user_id, role_id)).await
}
pub async fn save_custom_field_values(entity_id: &str, payload: &Value) -> Result<Value, String> {
    put_json(
        &format!("/api/admission/custom-fields/values/{}", entity_id),
        payload,
    )
    .await
}
pub async fn init_online_payment(fee_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/finance/payment/init/{}", fee_id)).await
}
// ─── Admin / Root ───
pub async fn admin_list_plans() -> Result<Value, String> {
    fetch_json("/api/admin/license-plans").await
}
pub async fn admin_create_plan(payload: &Value) -> Result<Value, String> {
    post_json("/api/admin/license-plans", payload).await
}
pub async fn admin_update_plan(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/api/admin/license-plans/{}", id), payload).await
}
pub async fn admin_delete_plan(id: &str) -> Result<Value, String> {
    delete_json(&format!("/api/admin/license-plans/{}", id)).await
}
pub async fn admin_set_plan_modules(id: &str, payload: &Value) -> Result<Value, String> {
    post_json(&format!("/api/admin/license-plans/{}/modules", id), payload).await
}
pub async fn admin_list_licenses() -> Result<Value, String> {
    fetch_json("/api/admin/licenses").await
}
pub async fn admin_assign_license(payload: &Value) -> Result<Value, String> {
    post_json("/api/admin/licenses", payload).await
}
pub async fn admin_extend_license(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/api/admin/licenses/{}/extend", id), payload).await
}
pub async fn admin_change_plan(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/api/admin/licenses/{}/change-plan", id), payload).await
}
pub async fn admin_update_license_status(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/api/admin/licenses/{}/status", id), payload).await
}
pub async fn admin_list_payments() -> Result<Value, String> {
    fetch_json("/api/admin/payments").await
}
pub async fn admin_register_payment(payload: &Value) -> Result<Value, String> {
    post_json("/api/admin/payments", payload).await
}
pub async fn admin_activity_log() -> Result<Value, String> {
    fetch_json("/api/admin/activity-log").await
}
pub async fn admin_system_health() -> Result<Value, String> {
    fetch_json("/api/admin/system/health").await
}
pub async fn admin_list_corporations() -> Result<Value, String> {
    fetch_json("/api/admin/corporations").await
}
pub async fn admin_create_corporation(payload: &Value) -> Result<Value, String> {
    post_json("/api/admin/corporations", payload).await
}
pub async fn admin_update_corporation(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/api/admin/corporations/{}", id), payload).await
}
pub async fn admin_toggle_corporation(id: &str) -> Result<Value, String> {
    put_json(&format!("/api/admin/corporations/{}/toggle", id), &json!({})).await
}
pub async fn admin_fetch_legal_reps(corporation_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/admin/legal-representatives?corporation_id={}", corporation_id)).await
}
pub async fn admin_create_legal_rep(payload: &Value) -> Result<Value, String> {
    post_json("/api/admin/legal-representatives", payload).await
}
pub async fn admin_update_legal_rep(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/api/admin/legal-representatives/{}", id), payload).await
}
pub async fn admin_delete_legal_rep(id: &str) -> Result<Value, String> {
    delete_json(&format!("/api/admin/legal-representatives/{}", id)).await
}

pub async fn check_vacancies() -> Result<Value, String> {
    fetch_json("/api/admission/vacancy-check").await
}
pub async fn fetch_admission_metrics() -> Result<Value, String> {
    fetch_json("/api/admission/metrics").await
}

fn urlencoding(s: &str) -> String {
    js_sys::encode_uri_component(s).as_string().unwrap_or_else(|| s.to_string())
}
