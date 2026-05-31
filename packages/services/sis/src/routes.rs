pub mod academic_calendar;
mod complementary_subjects;
mod corporation_dashboard;
mod parent_meetings;
mod courses;
mod dashboard;
mod enrollments;
mod import;
mod models;
mod parent_portal;
mod student_portal;
mod teacher_schedules;
pub mod students;



pub fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .merge(corporation_dashboard::router())
        .merge(dashboard::router())
        .merge(students::router())
        .merge(courses::router())
        .merge(enrollments::router())
        .merge(import::router())
        .merge(academic_calendar::router())
        .merge(complementary_subjects::router())
        .merge(parent_meetings::router())
        .merge(parent_portal::router())
        .merge(student_portal::router())
        .merge(teacher_schedules::router())
}
