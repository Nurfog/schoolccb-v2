mod activities;
mod classrooms;
mod custom_fields;
mod documents;
mod metrics;
mod prospects;
mod reminders;
mod scholarships;
mod stages;
mod upload;

pub use stages::seed_pipeline_stages;

pub fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .merge(stages::router())
        .merge(prospects::router())
        .merge(activities::router())
        .merge(documents::router())
        .merge(classrooms::router())
        .merge(upload::router())
        .merge(metrics::router())
        .merge(custom_fields::router())
        .merge(reminders::router())
        .merge(scholarships::router())
}
