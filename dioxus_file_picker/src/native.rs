use std::{
    collections::HashSet,
    env, fs, mem,
    path::{Path, PathBuf},
};

use dioxus::{
    logger::tracing::error,
    prelude::*,
};

#[component]
pub(crate) fn NativeFilePicker(multiple: bool, on_submit: Callback<HashSet<PathBuf>, ()>) -> Element {
    let mut explorer = use_signal(FilesExplorerState::new);
    let reader = explorer.read();
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/tailwind.css") }
        document::Link { rel: "stylesheet", href: asset!("/assets/fallback_file_picker.css") }
        document::Link {
            href: "https://fonts.googleapis.com/icon?family=Material+Icons",
            rel: "stylesheet",
        }
        // File Explorer
        div { class: "flex flex-col h-full",
            // File Explorer header
            div { class: "flex items-center flex-row ",
                if reader.can_go_back() {
                    i {
                        class: "material-icons",
                        onclick: move |_| explorer.write().go_back(),
                        "arrow_back"
                    }
                } else {
                    i { class: "material-icons deactivated", "arrow_back" }
                }
                if reader.can_go_forward() {
                    i {
                        class: "material-icons",
                        onclick: move |_| explorer.write().go_forward(),
                        "arrow_forward"
                    }
                } else {
                    i { class: "material-icons deactivated", "arrow_forward" }
                }
                if reader.is_root {
                    i { class: "material-icons deactivated", "arrow_upward" }
                } else {
                    i {
                        class: "material-icons",
                        onclick: move |_| explorer.write().go_up(),
                        "arrow_upward"
                    }
                }
                div { class: "border border-white-500 p-4 rounded-md flex-grow overflow-x-auto",
                    {reader.current().display().to_string()}
                }
                i {
                    class: "material-icons",
                    onclick: move |_| explorer.write().reload(),
                    "refresh"
                }
                div {
                    class: "border border-white-500 p-4 rounded-md",
                    class: if !reader.is_selecting { "bg-blue-500" },
                    onclick: move |_| explorer.write().toggle_selecting(),
                    if reader.is_selecting {
                        "cancel"
                    } else {
                        "select"
                    }
                }
            }
            // File Explorer Content
            div { class: "flex-1 overflow-y-auto",
                if let Some(err) = reader.error.as_ref() {
                    div {
                        code { "{err}" }
                        button { onclick: move |_| explorer.write().error = None, "x" }
                    }
                } else {
                    div {
                        for entity in reader.current_entities.clone() {
                            {
                                let name = entity.name;
                                let selection_class = if reader.selection.contains(&entity.path) {
                                    "bg-blue-500"
                                } else {
                                    ""
                                };
                                match entity.r#type {
                                    FileSystemType::File => rsx! {
                                        div { class: "fse {selection_class}",
                                            i {
                                                class: "material-icons",
                                                onclick: move |_| {
                                                    let mut writer = explorer.write();
                                                    let path = entity.path.clone();
                                                    if writer.is_selecting {
                                                        if writer.selection.contains(&path) {
                                                            writer.selection.retain(|p| p != &path);
                                                        } else {
                                                            if !multiple {
                                                                writer.selection.clear();
                                                            }
                                                            writer.selection.insert(path);
                                                        }
                                                    }
                                                },
                                                "description"
                                            }
                                            h1 { "{name}" }
                                        }
                                    },
                                    FileSystemType::Directory => rsx! {
                                        div { class: "fse {selection_class}",
                                            i {
                                                class: "material-icons ",
                                                onclick: move |_| {
                                                    let mut writer = explorer.write();
                                                    let path = entity.path.clone();
                                                    if writer.is_selecting {
                                                        if writer.selection.contains(&path) {
                                                            writer.selection.retain(|p| p != &path);
                                                        } else {
                                                            if !multiple {
                                                                writer.selection.clear();
                                                            }
                                                            writer.selection.insert(path);
                                                        }
                                                    } else {
                                                        writer.enter_dir(path)
                                                    }
                                                },
                                                "folder"
                                            }
                                            h1 { "{name}" }
                                        }
                                    },
                                }
                            }
                        }
                    
                    }
                }
            }
            // Footer (Always in layout, button only shows when selecting)
            div { class: "bg-amber-50 p-4 border-t border-gray-300",
                if reader.is_selecting {
                    button {
                        class: "bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600",
                        onclick: move |_| {
                            let mut writer = explorer.write();
                            let selection = mem::take(&mut writer.selection);
                            writer.is_selecting = false;
                            on_submit.call(selection);
                        },
                        "Submit"
                    }
                }
            }
        }
    }
}

/// A simple little struct to hold the file explorer state
///
/// We don't use any fancy signals or memoization here - Dioxus is so fast that even a file explorer can be done with a
/// single signal.
struct FilesExplorerState {
    /// Always canonicalized
    current_entities: Vec<TypedPathBuf>,
    is_root: bool,
    history: Vec<PathBuf>,
    history_position: usize,
    selection: HashSet<PathBuf>,
    is_selecting: bool,
    /// If Some, an error occurred with the current operation
    error: Option<String>,
}

impl FilesExplorerState {
    fn new() -> Self {
        Self::init_at(
            env::current_dir()
                .expect("Failed to retrieve current working directory using env::current_dir()"),
        )
    }

    fn init_at(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let current = std::path::absolute(path).expect(&format!(
            "Could not get absolute path of {}",
            path.display()
        ));
        let mut explorer = Self {
            current_entities: vec![],
            is_root: false,
            history: vec![current],
            history_position: 0,
            selection: HashSet::new(),
            is_selecting: false,
            error: None,
        };

        explorer.reload();

        explorer
    }

    fn reload(&mut self) {
        let current_path = self.current();
        let current_file_system_entities = match std::fs::read_dir(current_path) {
            Ok(e) => e,
            Err(err) => {
                let error = format!("An error occurred: {err:?}");
                self.error = Some(error);
                return;
            }
        };

        self.is_root = current_path.parent().is_none();
        self.error = None;

        self.current_entities.clear();
        for entity in current_file_system_entities {
            // todo make async and use async
            let typed_path: Option<TypedPathBuf> = (|| {
                let entity = entity.ok()?;
                let mut path = entity.path();
                let name = path.file_name()?.to_string_lossy().to_string();
                let entity_type = entity.file_type().ok()?;
                let fs_type;
                if entity_type.is_dir() {
                    fs_type = FileSystemType::Directory;
                } else if entity_type.is_symlink() {
                    path = fs::canonicalize(path).ok()?;
                    let metadata = fs::metadata(&path).ok()?;
                    debug_assert!(
                        !metadata.is_symlink(),
                        "canonicalize should resolve symlinks"
                    );
                    if metadata.is_dir() {
                        fs_type = FileSystemType::Directory;
                    } else {
                        fs_type = FileSystemType::File;
                    }
                } else {
                    fs_type = FileSystemType::File;
                };
                // todo check permissions?
                Some(TypedPathBuf::new(fs_type, path.to_path_buf(), name))
            })();
            match typed_path {
                Some(typed_path) => self.current_entities.push(typed_path),
                None => {
                    error!("Could not get typed path for entity");
                    continue;
                }
            }
        }
    }

    fn toggle_selecting(&mut self) {
        self.selection.clear();
        self.is_selecting = !self.is_selecting;
    }

    fn current(&self) -> &PathBuf {
        &self.history[self.history.len() - self.history_position - 1]
    }

    // Nav
    //************************************************************************//

    fn enter_dir(&mut self, path: PathBuf) {
        self.history_add(path);
        self.reload();
    }

    fn go_up(&mut self) {
        debug_assert!(
            !self.is_root,
            "This should not be exposed if we are at root"
        );
        let new = self
            .current()
            .parent()
            .expect("This should not be exposed if we are at root")
            .to_path_buf();
        self.history_add(new);
        self.reload();
    }

    fn go_back(&mut self) {
        debug_assert!(self.can_go_back());
        self.history_back();
        self.reload();
    }

    fn go_forward(&mut self) {
        debug_assert!(self.can_go_forward());
        self.history_forward();
        self.reload();
    }

    fn can_go_back(&self) -> bool {
        self.history_position + 1 < self.history.len()
    }

    fn history_back(&mut self) {
        self.history_position += 1;
        self.reload();
    }

    fn can_go_forward(&self) -> bool {
        self.history_position > 0
    }

    fn history_forward(&mut self) {
        self.history_position -= 1;
        self.reload();
    }

    fn history_add(&mut self, path: PathBuf) {
        if self.history_position != 0 {
            self.history
                .truncate(self.history.len() - self.history_position);
            self.history_position = 0;
        }
        self.history.push(path);
    }
}

#[derive(Clone)]
struct TypedPathBuf {
    /// The file system entity type
    r#type: FileSystemType,
    /// The resolved path
    path: PathBuf,
    /// Usually `file_name` [path], except when [path] is a symlink
    name: String,
}

impl TypedPathBuf {
    fn new(r#type: FileSystemType, path: PathBuf, name: String) -> Self {
        Self { r#type, path, name }
    }
}

#[derive(Clone)]
enum FileSystemType {
    File,
    Directory,
}
