use dioxus::html::FileEngine;
use std::{collections::HashSet, path::PathBuf, sync::Arc};

#[cfg(not(target_arch = "wasm32"))]
pub use native::VirtualPaths;
#[cfg(target_arch = "wasm32")]
pub use web::VirtualPaths;
// Conditionally export the appropriate implementation
// Web-specific implementation module
#[cfg(target_arch = "wasm32")]
mod web {
    use dioxus::html::FileEngine;
    use std::{collections::HashSet, path::PathBuf, sync::Arc};

    /// Web implementation of virtual paths for WebAssembly targets
    pub struct VirtualPaths {
        file_engine: Arc<dyn FileEngine>,
    }

    impl VirtualPaths {
        pub(crate) fn web(file_engine: Arc<dyn FileEngine>) -> Self {
            Self { file_engine }
        }

        /// Returns the number of paths.
        pub fn len(&self) -> usize {
            self.file_engine.files().len()
        }

        /// Returns the paths as `String`s. On web the path will be the file name.
        pub fn paths(&self) -> HashSet<String> {
            self.file_engine.files().iter().cloned().collect()
        }

        /// Reads all the files from the paths.
        pub async fn read_files(&self) -> Vec<(String, Vec<u8>)> {
            let mut files_and_data = Vec::new();

            for file_name in self.file_engine.files() {
                if let Some(data) = self.file_engine.read_file(&file_name).await {
                    files_and_data.push((file_name.clone(), data));
                } else {
                    dioxus::logger::tracing::warn!("Failed to read file: {:?}", file_name);
                }
            }

            files_and_data
        }
    }
}

// Native-specific implementation module
#[cfg(not(target_arch = "wasm32"))]
mod native {
    use dioxus::html::FileEngine;
    use std::{collections::HashSet, path::PathBuf, sync::Arc};
    use tokio::fs;

    /// Native implementation of virtual paths for non-WebAssembly targets
    pub struct VirtualPaths {
        paths: HashSet<PathBuf>,
    }

    impl VirtualPaths {
        pub(crate) fn native(paths: HashSet<PathBuf>) -> Self {
            Self { paths }
        }

        /// Returns the number of paths.
        pub fn len(&self) -> usize {
            self.paths.len()
        }

        /// Returns the paths as `String`s.
        pub fn paths(&self) -> HashSet<String> {
            self.paths
                .iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect()
        }

        /// Reads all the files from the paths.
        pub async fn read_files(&self) -> Vec<(String, Vec<u8>)> {
            let mut files_and_data = Vec::new();

            for path in &self.paths {
                if let Ok(data) = fs::read(&path).await {
                    files_and_data.push((path.to_string_lossy().into_owned(), data));
                } else {
                    dioxus::logger::tracing::warn!("Failed to read file: {:?}", path);
                }
            }

            files_and_data
        }
    }
}
