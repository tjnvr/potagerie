//! Round trips through the real IPC layer (mock runtime): the capability of
//! the main window, the commands, SQLite and the year view.

use std::path::Path;

use chrono::{Datelike, Local};
use serde_json::{Value, json};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder};
use tauri::webview::InvokeRequest;
use tauri::{Manager, WebviewWindow, WebviewWindowBuilder};

use super::*;

fn app(db: &Path) -> (tauri::App<MockRuntime>, WebviewWindow<MockRuntime>) {
    let builder = specta_builder::<MockRuntime>();
    let app = mock_builder()
        .invoke_handler(builder.invoke_handler())
        .build(context())
        .expect("failed to build the app");
    app.manage(AppState::open(db).unwrap());
    let window = WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    (app, window)
}

fn invoke(window: &WebviewWindow<MockRuntime>, cmd: &str, args: Value) -> Result<Value, Value> {
    get_ipc_response(
        window,
        InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: if cfg!(windows) {
                "http://tauri.localhost"
            } else {
                "tauri://localhost"
            }
            .parse()
            .unwrap(),
            body: InvokeBody::Json(args),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.into(),
        },
    )
    .map(|body| body.deserialize::<Value>().unwrap())
}

fn year() -> i32 {
    Local::now().year()
}

#[test]
fn add_a_planche_and_find_it_after_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join(DB_FILE);
    {
        let (_app, window) = app(&db);
        let view = invoke(&window, "vue_annee", json!({ "annee": year() })).unwrap();
        assert_eq!(view["terrain"], Value::Null);

        let view = invoke(
            &window,
            "creer_terrain",
            json!({
                "annee": year(),
                "terrain": { "longueur_cm": 1100, "largeur_cm": 650, "case_cm": 50, "allee_cm": 50 }
            }),
        )
        .unwrap();
        assert_eq!(view["terrain"]["case_cm"], 50);
        assert_eq!(view["version"], 1);

        let view = invoke(
            &window,
            "creer_planche",
            json!({ "annee": year(), "plancheType": "planche", "nom": "Planche A", "x": 50, "y": 50 }),
        )
        .unwrap();
        assert_eq!(view["planches"][0]["nom"], "Planche A");
        assert_eq!(view["planches"][0]["troncons"], 6);
        assert_eq!(view["version"], 2);
    }
    let (_app, window) = app(&db);
    let view = invoke(&window, "vue_annee", json!({ "annee": year() })).unwrap();
    assert_eq!(view["planches"].as_array().unwrap().len(), 1);
    assert_eq!(view["planches"][0]["nom"], "Planche A");
}

#[test]
fn refusals_carry_a_code_and_change_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let (_app, window) = app(&dir.path().join(DB_FILE));

    let err = invoke(
        &window,
        "creer_planche",
        json!({ "annee": year(), "plancheType": "carre", "nom": "Carré A", "x": 0, "y": 0 }),
    )
    .unwrap_err();
    assert_eq!(err["code"], "terrain_absent");

    invoke(
        &window,
        "creer_terrain",
        json!({
            "annee": year(),
            "terrain": { "longueur_cm": 1100, "largeur_cm": 650, "case_cm": 50, "allee_cm": 50 }
        }),
    )
    .unwrap();
    let err = invoke(
        &window,
        "creer_planche",
        json!({ "annee": year() - 1, "plancheType": "carre", "nom": "Carré A", "x": 0, "y": 0 }),
    )
    .unwrap_err();
    assert_eq!(err["code"], "annee_passee");
    let view = invoke(&window, "vue_annee", json!({ "annee": year() })).unwrap();
    assert_eq!(view["version"], 1);
}

#[test]
fn export_import_and_clear() {
    let dir = tempfile::tempdir().unwrap();
    let (_app, window) = app(&dir.path().join(DB_FILE));
    invoke(
        &window,
        "creer_terrain",
        json!({
            "annee": year(),
            "terrain": { "longueur_cm": 1100, "largeur_cm": 650, "case_cm": 50, "allee_cm": 50 }
        }),
    )
    .unwrap();
    invoke(
        &window,
        "creer_planche",
        json!({ "annee": year(), "plancheType": "bac", "nom": "Bac A", "x": 50, "y": 50 }),
    )
    .unwrap();

    let file = dir.path().join("potager.json");
    invoke(&window, "exporter_potager", json!({ "chemin": file })).unwrap();
    invoke(&window, "effacer_donnees", json!({})).unwrap();
    let view = invoke(&window, "vue_annee", json!({ "annee": year() })).unwrap();
    assert_eq!(view["terrain"], Value::Null);

    let summary = invoke(&window, "lire_import", json!({ "chemin": file })).unwrap();
    assert_eq!(summary["planche_count"], 1);
    assert_eq!(summary["replaces"], false);
    let view = invoke(
        &window,
        "importer_potager",
        json!({ "annee": year(), "chemin": file }),
    )
    .unwrap();
    assert_eq!(view["planches"][0]["nom"], "Bac A");
}

/// Every command of `command_names.rs` is registered and allowed for the main
/// window: none answers "not found" or "not allowed".
#[test]
fn every_command_is_registered_and_allowed() {
    let dir = tempfile::tempdir().unwrap();
    let (_app, window) = app(&dir.path().join(DB_FILE));
    for cmd in COMMANDS {
        if let Err(err) = invoke(&window, cmd, json!({})) {
            let message = err.to_string();
            assert!(
                !message.contains("not found") && !message.contains("not allowed"),
                "{cmd}: {message}"
            );
        }
    }
}

/// `front/src/lib/bindings.ts` is the one generated from the current Rust.
/// `UPDATE_BINDINGS=1 cargo test -p potagerie bindings` (`make bindings`)
/// rewrites it.
#[test]
fn bindings_are_up_to_date() {
    if std::env::var_os("UPDATE_BINDINGS").is_some() {
        export_bindings(&specta_builder::<tauri::Wry>(), &bindings_path()).unwrap();
    }
    let dir = tempfile::tempdir().unwrap();
    let fresh = dir.path().join("bindings.ts");
    export_bindings(&specta_builder::<tauri::Wry>(), &fresh).unwrap();
    let committed = std::fs::read_to_string(bindings_path()).unwrap_or_default();
    let fresh = std::fs::read_to_string(fresh).unwrap();
    assert!(
        committed.replace("\r\n", "\n") == fresh.replace("\r\n", "\n"),
        "front/src/lib/bindings.ts is stale: run `make bindings` and commit it"
    );
}
