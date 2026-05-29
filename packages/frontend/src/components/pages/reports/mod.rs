use dioxus::prelude::*;

mod course;
mod individual;
mod sige;

pub(super) fn current_year() -> i32 {
    js_sys::Date::new_0().get_full_year() as i32
}

#[derive(PartialEq, Clone)]
enum ReportTab {
    Individual,
    Course,
    Sige,
}

#[component]
pub fn ReportsPage() -> Element {
    let mut active_tab = use_signal(|| ReportTab::Individual);

    rsx! {
        div { class: "page-header",
            h1 { "Reportes" }
            p { "Certificados, concentraciones de notas y actas oficiales" }
        }
        div { class: "tab-bar",
            button {
                class: if active_tab() == ReportTab::Individual { "tab active" } else { "tab" },
                onclick: move |_| active_tab.set(ReportTab::Individual),
                "Individuales"
            }
            button {
                class: if active_tab() == ReportTab::Course { "tab active" } else { "tab" },
                onclick: move |_| active_tab.set(ReportTab::Course),
                "Por Curso"
            }
            button {
                class: if active_tab() == ReportTab::Sige { "tab active" } else { "tab" },
                onclick: move |_| active_tab.set(ReportTab::Sige),
                "Exportaciones SIGE"
            }
        }
        div { class: "tab-content",
            {
                match active_tab() {
                    ReportTab::Individual => rsx! { individual::IndividualReports {} },
                    ReportTab::Course => rsx! { course::CourseReports {} },
                    ReportTab::Sige => rsx! { sige::SigeReports {} },
                }
            }
        }
    }
}
