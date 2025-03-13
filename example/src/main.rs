use std::{collections::HashSet, path::PathBuf};

use dioxus::{logger::tracing::Level, prelude::*};
use dioxus_file_picker::{FilePickerLauncher, VirtualPaths};

fn main() {
    dioxus::logger::init(Level::INFO).expect("Failed to initialize logger");
    tracing_log::LogTracer::builder()
        .init()
        .expect("Failed to initialize log tracer");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        FilePickerLauncher {
            desktop_native: true,
            desktop_windowed: false,
            multiple: false,
            on_submit: move |paths: VirtualPaths| {
                debug_assert!(paths.len() == 1);
                dioxus::logger::tracing::info!("Selected file: {:?}", paths.paths());
            },
            "click me"
        }
    }
}
