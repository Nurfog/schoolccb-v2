use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct IconProps {
    pub name: String,
    pub size: Option<u16>,
    pub class: Option<String>,
}

fn icon_path(name: &str) -> &str {
    match name {
        "dashboard" => "/icons/dashboard.svg",
        "users" | "students" => "/icons/users.svg",
        "attendance" => "/icons/attendance.svg",
        "grades" => "/icons/grades.svg",
        "agenda" | "calendar" => "/icons/agenda.svg",
        "notifications" | "bell" => "/icons/notifications.svg",
        "finance" | "money" | "payroll" => "/icons/finance.svg",
        "reports" | "file-text" | "document" | "sige" => "/icons/reports.svg",
        "book" => "/icons/book.svg",
        "clipboard" => "/icons/clipboard.svg",
        "settings" | "gear" | "config" => "/icons/config.svg",
        "home" => "/icons/home.svg",
        "graduation" => "/icons/graduation.svg",
        "chart" => "/icons/chart.svg",
        "search" => "/icons/search.svg",
        "star" => "/icons/star.svg",
        "edit" | "pencil" => "/icons/edit.svg",
        "trash" | "delete" => "/icons/trash.svg",
        "check" => "/icons/check.svg",
        "x" | "close" => "/icons/close.svg",
        "info" => "/icons/info.svg",
        "shield" => "/icons/shield.svg",
        "email" | "mail" => "/icons/email.svg",
        "logout" => "/icons/logout.svg",
        "menu" | "hamburger" => "/icons/menu.svg",
        "list" => "/icons/list.svg",
        "grid" => "/icons/grid.svg",
        "layers" => "/icons/layers.svg",
        "wrench" | "tools" => "/icons/wrench.svg",
        "chevron-down" | "chevron" | "arrow-down" => "/icons/chevron-down.svg",
        "megaphone" => "/icons/megaphone.svg",
        "courses" => "/icons/courses.svg",
        "enrollments" | "enrollment" => "/icons/enrollments.svg",
        "subjects" => "/icons/subjects.svg",
        "grade-levels" => "/icons/grade-levels.svg",
        "academic-years" => "/icons/academic-years.svg",
        "classrooms" => "/icons/classrooms.svg",
        "admission" => "/icons/admission.svg",
        "hr" => "/icons/hr.svg",
        "complaints" => "/icons/complaints.svg",
        "roles" => "/icons/roles.svg",
        "audit" => "/icons/audit.svg",
        "corporations" => "/icons/corporations.svg",
        _ => "/icons/dashboard.svg",
    }
}

#[component]
pub fn Icon(props: IconProps) -> Element {
    let size = props.size.unwrap_or(20);
    let class = props.class.as_deref().unwrap_or("");
    let src = icon_path(&props.name);
    rsx! {
        img {
            class: "icon-img {class}",
            src: "{src}",
            alt: "",
            width: "{size}",
            height: "{size}",
            role: "presentation",
        }
    }
}
