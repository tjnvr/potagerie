//! What the smoke tests of the real app need (`e2e-app/`, S05 §3): a data
//! folder of their own, and file windows that answer without a human.
//! Both are read from the environment at launch; without them the app
//! behaves as usual.

use std::ffi::OsString;
use std::path::PathBuf;

use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Runtime, State};

/// Folder of `potagerie.db` instead of `app_data_dir`.
pub const DATA_DIR_VAR: &str = "POTAGERIE_DATA_DIR";

/// File that "Open" and "Save as" return, without showing the window.
pub const DIALOG_FILE_VAR: &str = "POTAGERIE_DIALOG_FILE";

/// The data folder: `var` when it is set and not empty, otherwise `default`.
pub fn data_dir(
    var: Option<OsString>,
    default: impl FnOnce() -> tauri::Result<PathBuf>,
) -> tauri::Result<PathBuf> {
    match var {
        Some(dir) if !dir.is_empty() => Ok(PathBuf::from(dir)),
        _ => default(),
    }
}

struct DialogFile(PathBuf);

#[tauri::command]
fn open(file: State<'_, DialogFile>) -> PathBuf {
    file.0.clone()
}

#[tauri::command]
fn save(file: State<'_, DialogFile>) -> PathBuf {
    file.0.clone()
}

/// Stands in for the dialog plugin: same name and commands, so the interface
/// and the `main` capability do not change, but each window answers `file`.
pub fn dialog_stand_in<R: Runtime>(file: PathBuf) -> TauriPlugin<R> {
    Builder::new("dialog")
        .invoke_handler(tauri::generate_handler![open, save])
        .setup(move |app, _| {
            use tauri::Manager;
            app.manage(DialogFile(file));
            Ok(())
        })
        .build()
}
