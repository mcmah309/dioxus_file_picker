use std::{collections::HashSet, path::PathBuf};

use dioxus::{logger::tracing::Level, prelude::*};
use dioxus_file_picker::FilePicker;

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::logger::init(Level::ERROR).expect("Failed to initialize logger");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        FilePicker {
            multiple: false,
            on_submit: move |paths: HashSet<PathBuf>| {
                debug_assert!(paths.len() == 1);
                let path = paths.iter().next().unwrap();
                dioxus::logger::tracing::error!("Selected file: {:?}", path);
            },
        }
    }
}
