use std::{
    collections::HashSet,
    env,
    path::PathBuf,
    sync::Arc,
    time::{self},
};

use dioxus::{html::FileEngine, logger::tracing::warn, prelude::*};
#[cfg(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
use rfd::FileDialog;
#[cfg(target_arch = "wasm32")]
use web_sys::{Blob, File};

use crate::cross_platform;

#[component]
pub fn FilePickerLauncher(
    /// If true, on desktop will launch a native file picker.
    desktop_native: bool, // todo
    /// If true, on desktop will launch in a new window. This is treated as true if `desktop_native` is true.
    desktop_windowed: bool,
    /// If true, on mobile will launch a native file picker.
    // mobile_native: bool, // todo
    /// Can select multiple
    multiple: bool,
    /// Can select directories
    // directory: bool, // todo
    /// File extensions to accept
    // accept: Vec<String>, // todo
    on_submit: Callback<(Arc<dyn FileEngine>, HashSet<PathBuf>), ()>,
    /// The path to open the directory at. If null, defaults to current directory. Has no effect on web.
    open_directory_path: Option<PathBuf>,
    children: Element,
) -> Element {
    #[cfg(target_arch = "wasm32")]
    {
        let id = uuid::Uuid::now_v7().to_string();
        rsx! {
            input {
                id: id.clone(),
                r#type: "file",
                style: "display: none;",
                multiple,
                onchange: move |event: Event<FormData>| {
                    event.prevent_default();
                    if let Some(file_engine) = &event.files() {
                        let file_names = file_engine.files();
                        let mut paths = HashSet::new();
                        for file_name in file_names {
                            paths.insert(PathBuf::from(file_name));
                        }
                        on_submit.call((file_engine.clone(), paths));
                    }
                },
            }
            button {
                onclick: move |_| {
                    {
                        let id = id.clone();
                        async move {
                            let _ = document::eval(
                                    &*format!("document.getElementById('{}').click()", &id),
                                )
                                .await
                                .ok();
                        }
                    }
                },
                {children}
            }
        }
    }
    #[cfg(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    {
        let launch_cross_platform_file_picker = move |_| {
            if desktop_native || desktop_windowed {
                let dom = VirtualDom::new_with_props(
                    cross_platform::CrossPlatformFilePicker,
                    cross_platform::CrossPlatformFilePickerProps {
                        multiple,
                        on_submit,
                    },
                );
                dioxus::desktop::window().new_window(dom, Default::default());
            }
            let rsx = rsx! {
                cross_platform::CrossPlatformFilePicker { multiple, on_submit } // todo bootstrap the on_submit to go back
            };
            Vec::new()
        };
        if desktop_native {
            let on_click = move |event| {
                let path_clone = open_directory_path.clone();
                let path = path_clone.unwrap_or_else(|| {
                    env::current_dir().expect("Failed to get current directory")
                });
                let start_time = time::Instant::now();
                let mut files;
                if multiple {
                    files = rfd::FileDialog::new()
                        .set_directory(&path)
                        .pick_files()
                        .unwrap_or(Vec::new())
                } else {
                    files = rfd::FileDialog::new()
                        .set_directory(&path)
                        .pick_file()
                        .map(|e| vec![e])
                        .unwrap_or(Vec::new());
                }
                let elapsed = start_time.elapsed();
                let within_one_second = elapsed <= time::Duration::from_secs(1);
                if within_one_second {
                    debug_assert!(files.is_empty());
                    warn!(
                        "Native file dialog closed too quickly. This was likely an error. Launching a dioxus file dialog instead"
                    );
                    files = launch_cross_platform_file_picker(event);
                }
                // todo
            };
            return rsx! {
                input {
                    name: "textreader",
                    r#type: "file",
                    multiple,
                    accept: ".txt,.rs",
                }
                div { onclick: on_click, {children} }
            };
        } else {
            return rsx! {
                div {
                    onclick: move |event| {
                        let files = launch_cross_platform_file_picker(event);
                    },
                    {children}
                }
            };
        }
    }
    rsx! {
        div {
            onclick: move |_| {
                let nav = use_navigator();
            },
        }
    }
}
