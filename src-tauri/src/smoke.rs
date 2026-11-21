//! What the smoke tests of the real app need (`e2e-app/`, S05 §3): a data
//! folder of their own, file windows that answer without a human, and the
//! WebView2 arguments of the driver. All are read from the environment at
//! launch; without them the app behaves as usual.

use std::ffi::OsString;
use std::path::PathBuf;

use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Context, Runtime, State};

/// Folder of `potagerie.db` instead of `app_data_dir`.
pub const DATA_DIR_VAR: &str = "POTAGERIE_DATA_DIR";

/// File that "Open" and "Save as" return, without showing the window.
pub const DIALOG_FILE_VAR: &str = "POTAGERIE_DIALOG_FILE";

/// WebView2 arguments set by msedgedriver (its debugging port, on Windows).
pub const BROWSER_ARGS_VAR: &str = "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS";

/// The arguments wry gives WebView2 when the window sets none.
const WRY_BROWSER_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --autoplay-policy=no-user-gesture-required";

/// wry always passes its own arguments to WebView2, which then ignores
/// `var`: when it is set and not empty, every window passes both.
pub fn add_browser_args<R: Runtime>(context: &mut Context<R>, var: Option<OsString>) {
    let Some(args) = var.filter(|a| !a.is_empty()) else {
        return;
    };
    let args = format!("{WRY_BROWSER_ARGS} {}", args.to_string_lossy());
    for window in &mut context.config_mut().app.windows {
        window.additional_browser_args = Some(args.clone());
    }
}

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
