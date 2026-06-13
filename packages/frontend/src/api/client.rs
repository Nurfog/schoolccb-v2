use serde_json::{Value, json};
use std::sync::{Mutex, OnceLock};
use wasm_bindgen::JsCast;

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

static TOKEN: Mutex<Option<String>> = Mutex::new(None);

fn get_token() -> Option<String> {
    TOKEN.lock().ok()?.clone()
}

fn set_token(token: &str) {
    if let Ok(mut t) = TOKEN.lock() {
        *t = Some(token.to_string());
    }
}

pub fn remove_token() {
    if let Ok(mut t) = TOKEN.lock() {
        *t = None;
    }
    // Also clear server-side session via logout API
}

fn auth_header() -> Option<String> {
    get_token().map(|t| format!("Bearer {}", t))
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

    if let Some(token) = result.get("token").and_then(|t| t.as_str()) {
        set_token(token);
    }

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
#[allow(dead_code)]
pub async fn fetch_student_alerts() -> Result<Value, String> {
    fetch_json("/api/dashboard/student-alerts").await
}
pub async fn fetch_agenda() -> Result<Value, String> {
    fetch_json("/api/dashboard/agenda").await
}

// ─── Corporation Dashboard (Sostenedor) ───
pub async fn fetch_corp_dashboard_summary() -> Result<Value, String> {
    fetch_json("/b2b/corporation/dashboard/summary").await
}
pub async fn fetch_corp_dashboard_schools() -> Result<Value, String> {
    fetch_json("/b2b/corporation/dashboard/schools").await
}
pub async fn fetch_corp_dashboard_comparisons() -> Result<Value, String> {
    fetch_json("/b2b/corporation/dashboard/comparisons").await
}
pub async fn fetch_corp_dashboard_trends() -> Result<Value, String> {
    fetch_json("/b2b/corporation/dashboard/trends").await
}
pub async fn fetch_corp_dashboard_alerts() -> Result<Value, String> {
    fetch_json("/b2b/corporation/dashboard/alerts").await
}
pub async fn fetch_corp_license() -> Result<Value, String> {
    fetch_json("/b2b/corporation/dashboard/license").await
}

// ─── School Dashboard (Colegio) ───
pub async fn fetch_school_attendance_trends() -> Result<Value, String> {
    fetch_json("/b2b/schools/dashboard/attendance-trends").await
}
pub async fn fetch_school_grades_distribution() -> Result<Value, String> {
    fetch_json("/b2b/schools/dashboard/grades-distribution").await
}
pub async fn fetch_school_finance_summary() -> Result<Value, String> {
    fetch_json("/b2b/schools/dashboard/finance-summary").await
}
pub async fn fetch_school_teacher_performance() -> Result<Value, String> {
    fetch_json("/b2b/schools/dashboard/teacher-performance").await
}
pub async fn fetch_school_top_alerts() -> Result<Value, String> {
    fetch_json("/b2b/schools/dashboard/top-alerts").await
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
    fetch_json("/api/finance/student_scholarships").await
}
pub async fn create_scholarship(payload: &Value) -> Result<Value, String> {
    post_json("/api/finance/student_scholarships", payload).await
}
pub async fn approve_scholarship(scholarship_id: &str) -> Result<Value, String> {
    put_json(
        &format!("/api/finance/student_scholarships/{}", scholarship_id),
        &serde_json::json!({}),
    )
    .await
}
pub async fn delete_scholarship(scholarship_id: &str) -> Result<Value, String> {
    delete_json(&format!("/api/finance/student_scholarships/{}", scholarship_id)).await
}

// ─── Legal Representatives ───
pub async fn fetch_legal_reps(corporation_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/b2b/legal-representatives?corporation_id={}", corporation_id)).await
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
    fetch_json("/b2b/corporations").await
}
pub async fn create_corporation(payload: &Value) -> Result<Value, String> {
    post_json("/b2b/corporations", payload).await
}
pub async fn fetch_schools(corporation_id: Option<&str>) -> Result<Value, String> {
    match corporation_id {
        Some(id) => fetch_json(&format!("/b2b/schools?corporation_id={}", id)).await,
        None => fetch_json("/b2b/schools").await,
    }
}
pub async fn create_school(payload: &Value) -> Result<Value, String> {
    post_json("/b2b/schools", payload).await
}
pub async fn update_school(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/b2b/schools/{}", id), payload).await
}
pub async fn toggle_school(id: &str) -> Result<Value, String> {
    put_json(&format!("/b2b/schools/{}/toggle", id), &json!({})).await
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
pub async fn quick_test(payload: &Value) -> Result<Value, String> {
    post_json("/api/grades/quick-test", payload).await
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

// ─── Academic Periods ───
pub async fn fetch_academic_periods() -> Result<Value, String> {
    fetch_json("/api/grades/periods").await
}
pub async fn create_academic_period(payload: &Value) -> Result<Value, String> {
    post_json("/api/grades/periods", payload).await
}
#[allow(dead_code)]
pub async fn get_academic_period(id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/grades/periods/{}", id)).await
}
pub async fn update_academic_period(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/api/grades/periods/{}", id), payload).await
}
#[allow(dead_code)]
pub async fn get_current_period() -> Result<Value, String> {
    fetch_json("/api/grades/periods/current").await
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
pub async fn classroom_availability(id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/admission/classrooms/{}/availability", id)).await
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
#[allow(dead_code)]
pub async fn download_certificate_pdf(student_id: &str) -> Result<(), String> {
    let url = abs_url(&format!("/api/reports/certificate/student/{}/pdf", student_id));
    let mut req = client().get(&url);
    if let Some(auth) = auth_header() {
        req = req.header("Authorization", auth);
    }
    let resp = req.send().await.map_err(|e| format!("Error: {e}"))?;
    let bytes = resp.bytes().await.map_err(|e| format!("Error obteniendo PDF: {e}"))?;
    let array = js_sys::Uint8Array::from(&bytes[..]);
    let blob = web_sys::Blob::new_with_u8_array_sequence(&array)
        .map_err(|_| "Error creando blob".to_string())?;
    let blob_url = web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|_| "Error creando URL".to_string())?;
    let window = web_sys::window().ok_or("No window")?;
    let doc = window.document().ok_or("No document")?;
    let link = doc.create_element("a").map_err(|_| "Error creando link")?;
    link.set_attribute("href", &blob_url).map_err(|_| "Error")?;
    link.set_attribute("download", "certificado.pdf").map_err(|_| "Error")?;
    if let Some(el) = link.dyn_ref::<web_sys::HtmlElement>() {
        el.click();
    }
    Ok(())
}

// ─── Parent Portal (Apoderado) ───
pub async fn fetch_parent_children() -> Result<Value, String> {
    fetch_json("/api/portal/parent/children").await
}
pub async fn fetch_child_grades(child_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/portal/parent/children/{}/grades", child_id)).await
}
pub async fn fetch_child_attendance(child_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/portal/parent/children/{}/attendance", child_id)).await
}
pub async fn fetch_child_schedule(child_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/portal/parent/children/{}/schedule", child_id)).await
}
pub async fn fetch_child_annotations(child_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/portal/parent/children/{}/annotations", child_id)).await
}
pub async fn fetch_parent_certificates() -> Result<Value, String> {
    fetch_json("/api/portal/parent/certificates").await
}
pub async fn request_certificate(payload: &Value) -> Result<Value, String> {
    post_json("/api/portal/parent/certificates/request", payload).await
}
pub async fn fetch_parent_appointments() -> Result<Value, String> {
    fetch_json("/api/portal/parent/appointments").await
}
pub async fn create_parent_appointment(payload: &Value) -> Result<Value, String> {
    post_json("/api/portal/parent/appointments", payload).await
}
pub async fn cancel_parent_appointment(id: &str) -> Result<Value, String> {
    put_json(&format!("/api/portal/parent/appointments/{}", id), &json!({})).await
}
pub async fn fetch_parent_messages() -> Result<Value, String> {
    fetch_json("/api/portal/parent/messages").await
}
pub async fn send_parent_message(payload: &Value) -> Result<Value, String> {
    post_json("/api/portal/parent/messages", payload).await
}
pub async fn fetch_available_slots() -> Result<Value, String> {
    fetch_json("/api/portal/parent/available-slots").await
}
pub async fn download_certificate(id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/portal/parent/certificates/{}/download", id)).await
}

// ─── Student Portal (Alumno) ───
pub async fn fetch_student_grades() -> Result<Value, String> {
    fetch_json("/api/portal/student/grades").await
}
pub async fn fetch_student_attendance() -> Result<Value, String> {
    fetch_json("/api/portal/student/attendance").await
}
pub async fn fetch_student_schedule() -> Result<Value, String> {
    fetch_json("/api/portal/student/schedule").await
}
pub async fn fetch_student_annotations() -> Result<Value, String> {
    fetch_json("/api/portal/student/annotations").await
}
pub async fn fetch_student_profile() -> Result<Value, String> {
    fetch_json("/api/portal/student/profile").await
}
pub async fn fetch_student_appointments() -> Result<Value, String> {
    fetch_json("/api/portal/student/appointments").await
}
pub async fn create_student_appointment(payload: &Value) -> Result<Value, String> {
    post_json("/api/portal/student/appointments", payload).await
}

// ─── CRM / Sales ───
pub async fn fetch_sales_proposals() -> Result<Value, String> {
    fetch_json("/b2b/sales/proposals").await
}
pub async fn create_sales_proposal(payload: &Value) -> Result<Value, String> {
    post_json("/b2b/sales/proposals", payload).await
}
pub async fn get_sales_proposal(id: &str) -> Result<Value, String> {
    fetch_json(&format!("/b2b/sales/proposals/{}", id)).await
}
pub async fn apply_proposal_discount(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/b2b/sales/proposals/{}/discount", id), payload).await
}
pub async fn generate_proposal_pdf(id: &str) -> Result<Value, String> {
    post_json(&format!("/b2b/sales/proposals/{}/generate-pdf", id), &json!({})).await
}
pub async fn create_sales_contract(payload: &Value) -> Result<Value, String> {
    post_json("/b2b/sales/contracts", payload).await
}
pub async fn get_sales_contract(id: &str) -> Result<Value, String> {
    fetch_json(&format!("/b2b/sales/contracts/{}", id)).await
}
pub async fn verify_contract_signatures(id: &str) -> Result<Value, String> {
    put_json(&format!("/b2b/sales/contracts/{}/verify-signatures", id), &json!({})).await
}

pub async fn generate_contract_invoice(id: &str) -> Result<Value, String> {
    post_json(&format!("/b2b/sales/contracts/{}/invoice", id), &json!({})).await
}
pub async fn fetch_sales_plans() -> Result<Value, String> {
    fetch_json("/b2b/sales/plans").await
}
pub async fn fetch_contract_documents(contract_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/b2b/sales/contracts/{}/documents", contract_id)).await
}
pub async fn upload_contract_document(contract_id: &str, payload: &Value) -> Result<Value, String> {
    post_json(&format!("/b2b/sales/contracts/{}/documents", contract_id), payload).await
}

// ─── Complementary Subjects ───
pub async fn fetch_complementary_subjects(course_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/courses/{}/complementary-subjects", course_id)).await
}
pub async fn create_complementary_subject(course_id: &str, payload: &Value) -> Result<Value, String> {
    post_json(&format!("/api/courses/{}/complementary-subjects", course_id), payload).await
}
pub async fn update_complementary_subject(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/api/complementary-subjects/{}", id), payload).await
}
pub async fn delete_complementary_subject(id: &str) -> Result<Value, String> {
    delete_json(&format!("/api/complementary-subjects/{}", id)).await
}

// ─── Teacher Schedules ───
pub async fn fetch_teacher_schedules(teacher_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/hr/teachers/{}/schedules", teacher_id)).await
}
pub async fn create_teacher_schedule(teacher_id: &str, payload: &Value) -> Result<Value, String> {
    post_json(&format!("/api/hr/teachers/{}/schedules", teacher_id), payload).await
}
pub async fn delete_teacher_schedule(id: &str) -> Result<Value, String> {
    delete_json(&format!("/api/hr/schedules/{}", id)).await
}
pub async fn fetch_teacher_hours(teacher_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/hr/teachers/{}/hours", teacher_id)).await
}
pub async fn set_teacher_hours(teacher_id: &str, payload: &Value) -> Result<Value, String> {
    post_json(&format!("/api/hr/teachers/{}/hours", teacher_id), payload).await
}
pub async fn fetch_extra_duties(teacher_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/hr/teachers/{}/extra-duties", teacher_id)).await
}
pub async fn create_extra_duty(teacher_id: &str, payload: &Value) -> Result<Value, String> {
    post_json(&format!("/api/hr/teachers/{}/extra-duties", teacher_id), payload).await
}
pub async fn update_extra_duty(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/api/hr/extra-duties/{}", id), payload).await
}

// ─── Academic Calendar ───
pub async fn fetch_calendar_events() -> Result<Value, String> {
    fetch_json("/api/academic/calendar").await
}
pub async fn create_calendar_event(payload: &Value) -> Result<Value, String> {
    post_json("/api/academic/calendar", payload).await
}
pub async fn update_calendar_event(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/api/academic/calendar/{}", id), payload).await
}
pub async fn delete_calendar_event(id: &str) -> Result<Value, String> {
    delete_json(&format!("/api/academic/calendar/{}", id)).await
}
pub async fn fetch_holidays() -> Result<Value, String> {
    fetch_json("/api/academic/holidays").await
}
pub async fn create_holiday(payload: &Value) -> Result<Value, String> {
    post_json("/api/academic/holidays", payload).await
}
pub async fn delete_holiday(id: &str) -> Result<Value, String> {
    delete_json(&format!("/api/academic/holidays/{}", id)).await
}
pub async fn fetch_exams() -> Result<Value, String> {
    fetch_json("/api/academic/exams").await
}
pub async fn create_exam(payload: &Value) -> Result<Value, String> {
    post_json("/api/academic/exams", payload).await
}
pub async fn update_exam(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/api/academic/exams/{}", id), payload).await
}
pub async fn delete_exam(id: &str) -> Result<Value, String> {
    delete_json(&format!("/api/academic/exams/{}", id)).await
}

// ─── Parent Meetings ───
pub async fn fetch_meetings() -> Result<Value, String> {
    fetch_json("/api/meetings").await
}
pub async fn create_meeting(payload: &Value) -> Result<Value, String> {
    post_json("/api/meetings", payload).await
}
pub async fn cancel_meeting(id: &str) -> Result<Value, String> {
    post_json(&format!("/api/meetings/{}/cancel", id), &json!({})).await
}
pub async fn fetch_general_meetings() -> Result<Value, String> {
    fetch_json("/api/meetings/general").await
}
pub async fn create_general_meeting(payload: &Value) -> Result<Value, String> {
    post_json("/api/meetings/general", payload).await
}
pub async fn fetch_meeting_minutes(meeting_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/meetings/general/{}/minutes", meeting_id)).await
}
pub async fn save_meeting_minutes(meeting_id: &str, payload: &Value) -> Result<Value, String> {
    post_json(&format!("/api/meetings/general/{}/minutes", meeting_id), payload).await
}

pub async fn check_vacancies() -> Result<Value, String> {
    fetch_json("/api/admission/vacancy-check").await
}
pub async fn fetch_admission_metrics() -> Result<Value, String> {
    fetch_json("/api/admission/metrics").await
}

#[allow(dead_code)]
pub async fn fetch_prospect_reminders(prospect_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/admission/reminders?prospect_id={}", prospect_id)).await
}

pub async fn create_reminder(payload: &Value) -> Result<Value, String> {
    post_json("/api/admission/reminders", payload).await
}

pub async fn delete_reminder(id: &str) -> Result<Value, String> {
    delete_json(&format!("/api/admission/reminders/{}", id)).await
}

pub async fn list_enrollment_contracts() -> Result<Value, String> {
    fetch_json("/api/admission/contracts").await
}

pub async fn get_enrollment_contract(id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/admission/contracts/{}", id)).await
}

pub async fn create_enrollment_contract(payload: &Value) -> Result<Value, String> {
    post_json("/api/admission/contracts", payload).await
}

pub async fn enroll_student(contract_id: &str) -> Result<Value, String> {
    post_json(&format!("/api/admission/contracts/{}/enroll", contract_id), &json!({})).await
}

pub async fn pay_contract(contract_id: &str, amount: f64, method: &str) -> Result<Value, String> {
    post_json(&format!("/api/admission/contracts/{}/pay", contract_id), &json!({"amount": amount, "method": method})).await
}

pub async fn list_scholarships() -> Result<Value, String> {
    fetch_json("/api/admission/scholarships").await
}

pub async fn create_admission_scholarship(payload: &Value) -> Result<Value, String> {
    post_json("/api/admission/scholarships", payload).await
}

pub async fn toggle_scholarship(id: &str) -> Result<Value, String> {
    put_json(&format!("/api/admission/scholarships/{}/toggle", id), &json!({})).await
}

pub async fn apply_scholarship(scholarship_id: &str, student_id: &str) -> Result<Value, String> {
    post_json(&format!("/api/admission/scholarships/{}/apply", scholarship_id), &json!({"student_id": student_id})).await
}

pub async fn fetch_interviews() -> Result<Value, String> {
    fetch_json("/api/hr/interviews").await
}
pub async fn create_interview(payload: &Value) -> Result<Value, String> {
    post_json("/api/hr/interviews", payload).await
}
pub async fn update_interview(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/api/hr/interviews/{}", id), payload).await
}
pub async fn delete_interview(id: &str) -> Result<Value, String> {
    delete_json(&format!("/api/hr/interviews/{}", id)).await
}

// ─── Admin License Plans ───
pub async fn admin_list_plans() -> Result<Value, String> {
    fetch_json("/b2b/admin/management/license-plans").await
}
pub async fn admin_get_plan(id: &str) -> Result<Value, String> {
    fetch_json(&format!("/b2b/admin/management/license-plans/{}", id)).await
}
pub async fn admin_create_plan(payload: &Value) -> Result<Value, String> {
    post_json("/b2b/admin/management/license-plans", payload).await
}
pub async fn admin_update_plan(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/b2b/admin/management/license-plans/{}", id), payload).await
}
#[allow(dead_code)]
pub async fn admin_delete_plan(id: &str) -> Result<Value, String> {
    delete_json(&format!("/b2b/admin/management/license-plans/{}", id)).await
}
pub async fn admin_set_plan_modules(id: &str, payload: &Value) -> Result<Value, String> {
    post_json(&format!("/b2b/admin/management/license-plans/{}/modules", id), payload).await
}
#[allow(dead_code)]
pub async fn admin_list_licenses(status: Option<&str>, corp_id: Option<&str>) -> Result<Value, String> {
    let mut q = String::new();
    if let Some(s) = status { q.push_str(&format!("status={}", s)); }
    if let Some(c) = corp_id { if !q.is_empty() { q.push('&'); } q.push_str(&format!("corporation_id={}", c)); }
    fetch_json(&format!("/b2b/admin/management/licenses?{}", q)).await
}
#[allow(dead_code)]
pub async fn admin_assign_license(payload: &Value) -> Result<Value, String> {
    post_json("/b2b/admin/management/licenses", payload).await
}
#[allow(dead_code)]
pub async fn admin_extend_license(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/b2b/admin/management/licenses/{}/extend", id), payload).await
}
#[allow(dead_code)]
pub async fn admin_change_plan(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/b2b/admin/management/licenses/{}/change-plan", id), payload).await
}
#[allow(dead_code)]
pub async fn admin_update_license_status(id: &str, payload: &Value) -> Result<Value, String> {
    put_json(&format!("/b2b/admin/management/licenses/{}/status", id), payload).await
}
#[allow(dead_code)]
pub async fn admin_list_payments() -> Result<Value, String> {
    fetch_json("/b2b/admin/management/payments").await
}
#[allow(dead_code)]
pub async fn admin_register_payment(payload: &Value) -> Result<Value, String> {
    post_json("/b2b/admin/management/payments", payload).await
}
#[allow(dead_code)]
pub async fn admin_list_corporations() -> Result<Value, String> {
    fetch_json("/b2b/admin/management/corporations").await
}

fn urlencoding(s: &str) -> String {
    js_sys::encode_uri_component(s).as_string().unwrap_or_else(|| s.to_string())
}
