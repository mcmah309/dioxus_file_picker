use std::{collections::HashSet, path::PathBuf};

use dioxus::{logger::tracing::Level, prelude::*};
use dioxus_file_picker::FilePickerLauncher;

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::logger::init(Level::ERROR).expect("Failed to initialize logger");
    tracing_log::LogTracer::builder()
        .init()
        .expect("Failed to initialize log tracer");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        FilePickerLauncher {
            desktop_native: false,
            desktop_windowed: false,
            multiple: false,
            on_submit: move |paths: HashSet<PathBuf>| {
                debug_assert!(paths.len() == 1);
                if let Some(path) = paths.into_iter().next() {
                    dioxus::logger::tracing::error!("Selected file: {:?}", path);
                } else {
                    dioxus::logger::tracing::error!("No file selected");
                }
            },
            "click me"
        }
    }
}
