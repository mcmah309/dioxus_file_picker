use std::{collections::HashSet, path::PathBuf, sync::Arc};

use dioxus::{html::FileEngine, logger::tracing::error, prelude::*};
use web_sys::{Blob, File};

// #[cfg(not(target_arch = "wasm32"))]
mod native;

#[component]
pub fn FilePicker(
    multiple: bool,
    directory: bool,
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
        rsx! {
            native::NativeFilePicker { multiple, on_submit }
        }
    }
}
