use std::{collections::HashSet, path::PathBuf, sync::Arc};

use dioxus::{html::FileEngine, logger::tracing::error, prelude::*};
use rfd::FileDialog;
#[cfg(target_arch = "wasm32")]
use web_sys::{Blob, File};
mod cross_platform;

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
    #[cfg(not(target_arch = "wasm32"))]
    {
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
            if desktop_native {
                // todo launch native window

                return rsx! {
                    div {
                        onclick: |_| {
                            let _ = FileDialog::new().set_directory("/").pick_file();
                        },
                        {children}
                    }
                };
            } else {
                return rsx! {
                    div {
                        onclick: move |_| {
                            let rsx = rsx! {
                                cross_platform::CrossPlatformFilePicker { multiple, on_submit } // todo bootstrap the on_submit to go back
                            };
                            // if desktop_windowed {
                            //     let dom = VirtualDom::new(rsx);
                            //     dioxus::desktop::window().new_window(dom, Default::default());
                            // } else {
                            //     let nav = use_navigator();
                            //     nav.push(rsx);
                            // }
                        },
                        {children}
                    }
                };
            }
        }
        rsx! {
            div {
                onclick: move |_| {
                    let nav = use_navigator(); // todo use with back navigation
                    // nav.push(cross_platform::CrossPlatformFilePicker { multiple, on_submit });
                }
            }
        }
    }
}
