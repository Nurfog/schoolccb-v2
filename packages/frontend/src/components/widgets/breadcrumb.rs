use dioxus::prelude::*;

#[derive(PartialEq, Clone)]
pub struct BreadcrumbItem {
    pub label: &'static str,
    pub route: Option<&'static str>,
}

#[component]
pub fn Breadcrumb(items: Vec<BreadcrumbItem>) -> Element {
    let nav = use_navigator();

    rsx! {
        nav { class: "breadcrumb", aria_label: "Navegación",
            {items.iter().enumerate().map(|(i, item)| {
                let is_last = i == items.len() - 1;
                let item_class = if is_last { "breadcrumb-item active" } else { "breadcrumb-item" };
                rsx! {
                    div { class: "{item_class}",
                        if i > 0 {
                            span { class: "breadcrumb-separator", "/" }
                        }
                        if let Some(route) = item.route {
                            if !is_last {
                                a {
                                    href: route,
                                    onclick: move |ev| {
                                        ev.prevent_default();
                                        nav.push(route);
                                    },
                                    "{item.label}"
                                }
                            } else {
                                span { "{item.label}" }
                            }
                        } else {
                            span { "{item.label}" }
                        }
                    }
                }
            })}
        }
    }
}
