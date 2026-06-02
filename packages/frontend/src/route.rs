use dioxus::prelude::*;
use dioxus::router::components::Outlet;

use crate::api::client;
use crate::seo::use_page_title;
use crate::components::layout::{sidebar::Sidebar, topbar::Topbar};
use crate::components::pages::academic_years_page::AcademicYearsPage;
use crate::components::pages::admission::AdmissionPage;

use crate::components::pages::agenda_page::AgendaPage;
use crate::components::pages::attendance_page::AttendancePage;
use crate::components::pages::audit_page::AuditPage;
use crate::components::pages::classrooms_page::ClassroomsPage;
use crate::components::pages::client_portal_page::ClientPortalPage;
use crate::components::pages::complementary_subjects_page::ComplementarySubjectsPage;
use crate::components::pages::complaints_page::ComplaintsPage;
use crate::components::pages::config_page::ConfigPage;
use crate::components::pages::corporations_page::CorporationsPage;
use crate::components::pages::courses_page::CoursesPage;
use crate::components::pages::csv_import_page::CsvImportPage;
use crate::components::pages::employee_portal_page::EmployeePortalPage;
use crate::components::pages::enrollments_page::EnrollmentsPage;
use crate::components::pages::finance::FinancePage;
use crate::components::pages::grade_levels_page::GradeLevelsPage;
use crate::components::pages::grades_page::GradesPage;
use crate::components::pages::hr_detail_page::HrDetailPage;
use crate::components::pages::hr_page::HrPage;
use crate::components::pages::login_page::LoginPage;
use crate::components::pages::module_manager::ModuleManager;
use crate::components::pages::notifications_page::NotificationsPage;
use crate::components::pages::payroll_page::PayrollPage;
use crate::components::pages::reports::ReportsPage;
use crate::components::pages::roles_page::RolesPage;
use crate::components::pages::sales_page::SalesPage;
use crate::components::pages::teacher_schedules_page::TeacherSchedulesPage;
use crate::components::pages::sige_page::SigePage;
use crate::components::pages::students_page::StudentsPage;
use crate::components::pages::subjects_page::SubjectsPage;
use crate::components::pages::curriculum_agent::CurriculumAgent;
use crate::components::pages::dashboard_mosaicos_page::DashboardMosaicosPage;
use crate::components::pages::interview_process_page::InterviewProcessPage;

use crate::components::pages::sostenedor_page::SostenedorPage;
use crate::components::pages::parent_meetings_page::ParentMeetingsPage;
use crate::components::pages::parent_portal_page::ParentPortalPage;
use crate::components::pages::student_portal_page::StudentPortalPage;
use crate::components::pages::academic_calendar_page::AcademicCalendarPage;
use crate::components::pages::users_page::UsersPage;

pub fn has_token() -> bool {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return false,
    };
    let doc = match window.document() {
        Some(d) => d,
        None => return false,
    };
    let cookie = js_sys::Reflect::get(&doc, &wasm_bindgen::JsValue::from_str("cookie"))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default();
    cookie.split(';').any(|c| {
        let c = c.trim();
        c == "session_active=1"
    })
}

fn require_auth() {
    if !has_token() {
        let nav = navigator();
        nav.push("/login");
    }
}

#[component]
fn AppLayout() -> Element {
    let fav_ver = use_signal(|| 0u32);
    use_context_provider(|| fav_ver);

    rsx! {
        a { class: "skip-link", href: "#main-content", "Saltar al contenido" }
        div { class: "app-layout",
            Sidebar {}
            div { class: "main-area",
                Topbar {}
                div { id: "main-content", class: "dashboard-content", role: "main",
                    div { role: "alert", aria_live: "polite",
                        ErrorBoundary {
                            Outlet::<Route> {}
                        }
                    }
                }
            }
        }
    }
}

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/login")]
    Login {},
    #[route("/session-login")]
    SessionLogin {},
    #[layout(AppLayout)]
        #[route("/dashboard")]
        Dashboard {},
        #[route("/sostenedor")]
        SostenedorPortal {},
        #[route("/")]
        ModuleManagerRoot {},
        #[route("/students")]
        Students {},
        #[route("/students/{student_id}")]
        StudentDetailPage { student_id: String },
        #[route("/attendance")]
        Attendance {},
        #[route("/grades")]
        Grades {},
        #[route("/notifications")]
        Notifications {},
        #[route("/reports")]
        Reports {},
        #[route("/finance")]
        Finance {},
        #[route("/users")]
        Users {},
        #[route("/courses")]
        Courses {},
        #[route("/enrollments")]
        Enrollments {},
        #[route("/subjects")]
        Subjects {},
        #[route("/config")]
        Config {},
        #[route("/admission")]
        Admission {},
        #[route("/hr")]
        Hr {},
        #[route("/hr/{employee_id}")]
        HrDetail { employee_id: String },
        #[route("/import")]
        Import {},
        #[route("/corporations")]
        Corporations {},
        #[route("/agenda")]
        Agenda {},
        #[route("/academic-years")]
        AcademicYears {},
        #[route("/academic-calendar")]
        AcademicCalendar {},
        #[route("/audit")]
        Audit {},
        #[route("/grade-levels")]
        GradeLevels {},
        #[route("/roles")]
        Roles {},
        #[route("/classrooms")]
        Classrooms {},
        #[route("/payroll")]
        Payroll {},
        #[route("/license-portal")]
        ClientPortal {},
        #[route("/my-portal")]
        EmployeePortal {},
        #[route("/parent-portal")]
        ParentPortal {},
        #[route("/parent-meetings")]
        ParentMeetings {},
        #[route("/student-portal")]
        StudentPortal {},
        #[route("/teacher-schedules")]
        TeacherSchedules {},
        #[route("/hr/interviews")]
        Interviews {},
        #[route("/sige")]
        Sige {},
        #[route("/complaints")]
        Complaints {},
        #[route("/complementary-subjects")]
        ComplementarySubjects {},
        #[route("/curriculum")]
        Curriculum {},
        #[route("/sales")]
        Sales {},
}

#[component]
pub fn Login() -> Element { use_page_title("Login"); rsx! { LoginPage {} } }

#[component]
pub fn SessionLogin() -> Element {
    use_page_title("Iniciando Sesión...");
    let mut error = use_signal(|| None::<String>);
    let mut started = use_signal(|| false);

    if !started() {
        started.set(true);
        let nav = navigator();
        spawn(async move {
            let search = web_sys::window()
                .and_then(|w| w.location().search().ok())
                .unwrap_or_default();
            let code = search
                .strip_prefix('?')
                .unwrap_or(&search)
                .split('&')
                .filter_map(|pair| {
                    let mut parts = pair.splitn(2, '=');
                    let key = parts.next()?;
                    let val = parts.next()?;
                    if key == "code" { Some(val.to_string()) } else { None }
                })
                .next();
            match code {
                Some(c) => {
                    match client::exchange_code(&c).await {
                        Ok(_) => { let _ = nav.replace("/"); }
                        Err(e) => error.set(Some(e)),
                    }
                }
                None => {
                    error.set(Some("Código de sesión no encontrado en la URL".to_string()));
                }
            }
        });
    }

    rsx! {
        div { class: "login-container",
            div { class: "login-card",
                div { class: "login-header",
                    div { class: "login-logo", "SC" }
                    h1 { "SchoolCBB" }
                    p { "Plataforma Escolar" }
                }
                div { style: "text-align: center; padding: 1rem 0;",
                    if let Some(ref msg) = error() {
                        div { class: "login-error", role: "alert", "{msg}" }
                        br {}
                        a { class: "login-btn", href: "/login",
                            style: "display: inline-block; text-decoration: none;",
                            "Volver a iniciar sesión"
                        }
                    } else {
                        div { class: "loading-spinner", "Validando credenciales..." }
                    }
                }
            }
        }
    }
}

#[component]
pub fn StudentDetailPage(student_id: String) -> Element { require_auth(); use_page_title("Detalle Estudiante"); rsx! { crate::components::pages::student_detail_page::StudentDetailPage { student_id } } }

#[component]
pub fn Dashboard() -> Element { require_auth(); use_page_title("Dashboard"); rsx! { DashboardMosaicosPage {} } }

#[component]
pub fn SostenedorPortal() -> Element { require_auth(); use_page_title("Portal Sostenedor"); rsx! { SostenedorPage {} } }

#[component]
pub fn ModuleManagerRoot() -> Element { require_auth(); use_page_title("SchoolCBB v2"); rsx! { ModuleManager {} } }

#[component]
pub fn Students() -> Element { require_auth(); use_page_title("Estudiantes"); rsx! { StudentsPage {} } }

#[component]
pub fn Attendance() -> Element { require_auth(); use_page_title("Asistencia"); rsx! { AttendancePage {} } }

#[component]
pub fn Grades() -> Element { require_auth(); use_page_title("Calificaciones"); rsx! { GradesPage {} } }

#[component]
pub fn Notifications() -> Element { require_auth(); use_page_title("Notificaciones"); rsx! { NotificationsPage {} } }

#[component]
pub fn Reports() -> Element { require_auth(); use_page_title("Reportes"); rsx! { ReportsPage {} } }

#[component]
pub fn Finance() -> Element { require_auth(); use_page_title("Finanzas"); rsx! { FinancePage {} } }

#[component]
pub fn Users() -> Element { require_auth(); use_page_title("Usuarios"); rsx! { UsersPage {} } }

#[component]
pub fn Courses() -> Element { require_auth(); use_page_title("Cursos"); rsx! { CoursesPage {} } }

#[component]
pub fn Enrollments() -> Element { require_auth(); use_page_title("Matrículas"); rsx! { EnrollmentsPage {} } }

#[component]
pub fn Subjects() -> Element { require_auth(); use_page_title("Asignaturas"); rsx! { SubjectsPage {} } }

#[component]
pub fn Config() -> Element { require_auth(); use_page_title("Configuración"); rsx! { ConfigPage {} } }

#[component]
pub fn Admission() -> Element { require_auth(); use_page_title("Admisión"); rsx! { AdmissionPage {} } }

#[component]
pub fn Hr() -> Element { require_auth(); use_page_title("RRHH"); rsx! { HrPage {} } }

#[component]
pub fn HrDetail(employee_id: String) -> Element { require_auth(); use_page_title("Detalle RRHH"); rsx! { HrDetailPage { employee_id } } }

#[component]
pub fn Import() -> Element { require_auth(); use_page_title("Importar Datos"); rsx! { CsvImportPage { entity_type: "students".to_string() } } }

#[component]
pub fn Corporations() -> Element { require_auth(); use_page_title("Corporaciones"); rsx! { CorporationsPage {} } }

#[component]
pub fn Agenda() -> Element { require_auth(); use_page_title("Agenda"); rsx! { AgendaPage {} } }

#[component]
pub fn AcademicYears() -> Element { require_auth(); use_page_title("Años Académicos"); rsx! { AcademicYearsPage {} } }

#[component]
pub fn AcademicCalendar() -> Element { require_auth(); use_page_title("Calendario Académico"); rsx! { AcademicCalendarPage {} } }

#[component]
pub fn Audit() -> Element { require_auth(); use_page_title("Auditoría"); rsx! { AuditPage {} } }

#[component]
pub fn GradeLevels() -> Element { require_auth(); use_page_title("Niveles"); rsx! { GradeLevelsPage {} } }

#[component]
pub fn Roles() -> Element { require_auth(); use_page_title("Roles"); rsx! { RolesPage {} } }

#[component]
pub fn Classrooms() -> Element { require_auth(); use_page_title("Salas"); rsx! { ClassroomsPage {} } }

#[component]
pub fn Payroll() -> Element { require_auth(); use_page_title("Remuneraciones"); rsx! { PayrollPage {} } }

#[component]
pub fn ClientPortal() -> Element { require_auth(); use_page_title("Portal Cliente"); rsx! { ClientPortalPage {} } }

#[component]
pub fn EmployeePortal() -> Element { require_auth(); use_page_title("Mi Portal"); rsx! { EmployeePortalPage {} } }

#[component]
pub fn ParentPortal() -> Element { require_auth(); use_page_title("Portal Apoderado"); rsx! { ParentPortalPage {} } }

#[component]
pub fn ParentMeetings() -> Element { require_auth(); use_page_title("Reuniones Apoderados"); rsx! { ParentMeetingsPage {} } }

#[component]
pub fn StudentPortal() -> Element { require_auth(); use_page_title("Portal Alumno"); rsx! { StudentPortalPage {} } }

#[component]
pub fn TeacherSchedules() -> Element { require_auth(); use_page_title("Horarios Docentes"); rsx! { TeacherSchedulesPage {} } }

#[component]
pub fn Interviews() -> Element { require_auth(); use_page_title("Entrevistas"); rsx! { InterviewProcessPage {} } }

#[component]
pub fn Sige() -> Element { require_auth(); use_page_title("Sincronización SIGE"); rsx! { SigePage {} } }

#[component]
pub fn Complaints() -> Element { require_auth(); use_page_title("Reclamos y Sugerencias"); rsx! { ComplaintsPage {} } }

#[component]
pub fn ComplementarySubjects() -> Element { require_auth(); use_page_title("Asignaturas Complementarias"); rsx! { ComplementarySubjectsPage {} } }

#[component]
pub fn Curriculum() -> Element { use_page_title("Currículum Nacional"); rsx! { CurriculumAgent {} } }

#[component]
pub fn Sales() -> Element { require_auth(); use_page_title("CRM Ventas"); rsx! { SalesPage {} } }


