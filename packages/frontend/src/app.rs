use dioxus::prelude::*;

use crate::route::Route;

#[component]
pub fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
