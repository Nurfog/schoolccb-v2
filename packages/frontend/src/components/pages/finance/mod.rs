use dioxus::prelude::*;

mod fees;
mod payments;
mod scholarships;

#[derive(PartialEq, Clone)]
enum FinanceTab {
    Fees,
    Payments,
    Scholarships,
}

#[component]
pub fn FinancePage() -> Element {
    let mut active_tab = use_signal(|| FinanceTab::Fees);

    rsx! {
        div { class: "page-header",
            h1 { "Finanzas" }
            p { "Gestión de cuotas, pagos y becas" }
        }
        div { class: "tab-bar",
            button {
                class: if active_tab() == FinanceTab::Fees { "tab active" } else { "tab" },
                onclick: move |_| active_tab.set(FinanceTab::Fees),
                "Cuotas"
            }
            button {
                class: if active_tab() == FinanceTab::Payments { "tab active" } else { "tab" },
                onclick: move |_| active_tab.set(FinanceTab::Payments),
                "Pagos"
            }
            button {
                class: if active_tab() == FinanceTab::Scholarships { "tab active" } else { "tab" },
                onclick: move |_| active_tab.set(FinanceTab::Scholarships),
                "Becas"
            }
        }
        div { class: "tab-content",
            {
                match active_tab() {
                    FinanceTab::Fees => rsx! { fees::FeesTab {} },
                    FinanceTab::Payments => rsx! { payments::PaymentsTab {} },
                    FinanceTab::Scholarships => rsx! { scholarships::ScholarshipsTab {} },
                }
            }
        }
    }
}
