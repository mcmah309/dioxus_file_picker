use std::{
    collections::HashSet,
    env,
    path::PathBuf,
    rc::Weak,
    sync::Arc,
    time::{self},
};

use dioxus::{
    html::FileEngine,
    logger::tracing::warn,
    prelude::*,
};
#[cfg(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
use dioxus::desktop::{DesktopContext, DesktopService};
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

use crate::{Overlay, file_picker};

#[component]
pub fn FilePickerLauncher(
    /// If true, on desktop will launch a native file picker.
    desktop_native: bool,
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
    /// The callback to call when a file(s) is selected and submitted. If `multiple` is false, the set may be empty or
    /// contain one.
    on_submit: Callback<HashSet<PathBuf>, ()>,
    /// The path to open the file picker at. If null, defaults to current directory. Has no effect on web.
    open_at: Option<PathBuf>,
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
                        on_submit.call(paths);
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
        let mut overlay_active = use_signal(|| false);
        let mut current_opened_window: Signal<Option<Weak<DesktopService>>> = use_signal(|| None);
        fn close_window(window_signal: &mut Signal<Option<Weak<DesktopService>>>) {
            if let Some(window) = window_signal.write().take().and_then(|e| e.upgrade()) {
                window.close();
            }
        }
        let on_submit = use_callback(move |paths: HashSet<PathBuf>| {
            on_submit.call(paths);
            overlay_active.set(false);
            close_window(&mut current_opened_window);
        });
        let on_click = move |_event| {
            fn create_dioxus_window(
                multiple: bool,
                on_submit: Callback<HashSet<PathBuf>, ()>,
                window_signal: &mut Signal<Option<Weak<DesktopService>>>,
            ) {
                let dom = VirtualDom::new_with_props(
                    file_picker::FilePicker,
                    file_picker::FilePickerProps {
                        multiple,
                        open_at: None,
                        on_submit,
                    },
                );
                let window = dioxus::desktop::window().new_window(dom, Default::default());
                window_signal.set(Some(window));
            }
            let path_clone = open_at.clone();
            let path = path_clone
                .unwrap_or_else(|| env::current_dir().expect("Failed to get current directory"));
            close_window(&mut current_opened_window);
            if desktop_native {
                let start_time = time::Instant::now();
                let files;
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
                    create_dioxus_window(multiple, on_submit, &mut current_opened_window);
                } else {
                    on_submit.call(files.into_iter().collect());
                }
            } else if desktop_windowed {
                create_dioxus_window(multiple, on_submit, &mut current_opened_window);
            } else {
                overlay_active.set(true);
            }
        };
        return rsx! {
            div { onclick: on_click, {children} }
            Overlay {
                active: overlay_active,
                file_picker::FilePicker { multiple, on_submit }
            }
        };
    }
    #[cfg(not(any(
        target_arch = "wasm32",
        //
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    {
        let mut overlay_active = use_signal(|| false);
        let on_submit = use_callback(move |paths: HashSet<PathBuf>| {
            on_submit.call(paths);
            overlay_active.set(false);
        });
        let on_click = move |event| {
            overlay_active.set(true);
        };
        return rsx! {
            div { onclick: on_click, {children} }
            Overlay {
                active: overlay_active,
                file_picker::FilePicker { multiple, on_submit }
            }
        };
    }
}
