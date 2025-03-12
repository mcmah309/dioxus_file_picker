use std::{collections::HashSet, path::PathBuf, sync::Arc};

use dioxus::{html::FileEngine, logger::tracing::Level, prelude::*};
use dioxus_file_picker::FilePickerLauncher;

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::logger::init(Level::ERROR).expect("Failed to initialize logger");
    tracing_log::LogTracer::builder().init().expect("Failed to initialize log tracer");
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
                let path = paths.iter().next().unwrap();
                dioxus::logger::tracing::error!("Selected file: {:?}", path);
            },
            "click me"
        }
    }
}
