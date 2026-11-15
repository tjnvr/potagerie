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

fn terrain_args() -> Value {
    json!({
        "annee": year(),
        "terrain": { "longueur_cm": 1100, "largeur_cm": 650, "case_cm": 50, "allee_cm": 50 }
    })
}

fn place(
    window: &WebviewWindow<MockRuntime>,
    kind: &str,
    nom: &str,
    x: u32,
    y: u32,
) -> Result<Value, Value> {
    invoke(
        window,
        "creer_planche",
        json!({ "annee": year(), "plancheType": kind, "nom": nom, "x": x, "y": y }),
    )
}

/// The first-use journey: terrain, planches, move, refusals, deletion.
#[test]
fn journey_terrain_place_move_refuse_delete() {
    let dir = tempfile::tempdir().unwrap();
    let (_app, window) = app(&dir.path().join(DB_FILE));

    // Terrain: dimensions and allée are validated.
    for bad in [
        json!({ "longueur_cm": 50, "largeur_cm": 650, "case_cm": 50, "allee_cm": 50 }),
        json!({ "longueur_cm": 1100, "largeur_cm": 650, "case_cm": 50, "allee_cm": 40 }),
    ] {
        let err = invoke(
            &window,
            "creer_terrain",
            json!({ "annee": year(), "terrain": bad }),
        )
        .unwrap_err();
        assert!(["terrain_invalide", "allee_invalide"].contains(&err["code"].as_str().unwrap()));
    }
    let view = invoke(&window, "vue_annee", json!({ "annee": year() })).unwrap();
    assert_eq!(view["terrain"], Value::Null);
    invoke(&window, "creer_terrain", terrain_args()).unwrap();

    // Two planches, 50 cm apart (the allée width).
    let view = place(&window, "planche", "Planche A", 50, 50).unwrap();
    let a = view["planches"][0]["id"].as_str().unwrap().to_string();
    let view = place(&window, "carre", "Carré B", 400, 50).unwrap();
    assert_eq!(view["planches"].as_array().unwrap().len(), 2);
    let version = view["version"].clone();

    // Refusals: overlap, allée too narrow, outside the terrain. Nothing changes.
    let err = place(&window, "bac", "Bac C", 300, 100).unwrap_err();
    assert_eq!(err["code"], "planche_chevauche");
    let err = place(&window, "bac", "Bac C", 420, 100).unwrap_err();
    assert_eq!(err["code"], "planche_chevauche");
    let err = place(&window, "bac", "Bac C", 360, 190).unwrap_err();
    assert_eq!(err["code"], "allee_trop_etroite");
    let err = place(&window, "planche", "Planche D", 900, 50).unwrap_err();
    assert_eq!(err["code"], "planche_hors_terrain");
    let err = place(&window, "planche", "", 50, 400).unwrap_err();
    assert_eq!(err["code"], "nom_invalide");
    let view = invoke(&window, "vue_annee", json!({ "annee": year() })).unwrap();
    assert_eq!(view["version"], version);
    assert_eq!(view["planches"].as_array().unwrap().len(), 2);

    // Move A: onto B is refused and it stays, then somewhere free works.
    let err = invoke(
        &window,
        "deplacer_planche",
        json!({ "annee": year(), "id": a, "x": 350, "y": 50 }),
    )
    .unwrap_err();
    assert_eq!(err["code"], "planche_chevauche");
    let view = invoke(
        &window,
        "deplacer_planche",
        json!({ "annee": year(), "id": a, "x": 50, "y": 300 }),
    )
    .unwrap();
    let moved = view["planches"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == a.as_str())
        .unwrap();
    assert_eq!(
        (moved["x"].as_u64(), moved["y"].as_u64()),
        (Some(50), Some(300))
    );

    // Resize: length in whole cases, width free, past the terrain refused.
    let err = invoke(
        &window,
        "modifier_planche",
        json!({ "annee": year(), "id": a, "geometrie": {
            "planche_type": "planche", "x": 50, "y": 300, "longueur_cm": 330, "largeur_cm": 120 } }),
    )
    .unwrap_err();
    assert_eq!(err["code"], "longueur_invalide");
    let view = invoke(
        &window,
        "modifier_planche",
        json!({ "annee": year(), "id": a, "nom": "Les tomates", "geometrie": {
            "planche_type": "planche", "x": 50, "y": 300, "longueur_cm": 400, "largeur_cm": 135 } }),
    )
    .unwrap();
    assert_eq!(view["planches"][0]["nom"], "Les tomates");
    assert_eq!(view["planches"][0]["troncons"], 8);

    // The terrain cannot shrink under a planche.
    let err = invoke(
        &window,
        "modifier_terrain",
        json!({ "annee": year(), "terrain": { "longueur_cm": 1100, "largeur_cm": 400, "allee_cm": 50 } }),
    )
    .unwrap_err();
    assert_eq!(err["code"], "planche_hors_terrain");

    // A serie makes the deletion warn, and a shortening overflow.
    let serie = json!({
        "culture_id": "carotte", "planche_id": a, "troncon_debut": 1, "troncon_fin": 6,
        "mode": "pleine_terre", "date": format!("{}-03-15", year()),
        "abri_jours": null, "fin_manuelle": null
    });
    let view = invoke(
        &window,
        "creer_serie",
        json!({ "annee": year(), "serie": serie }),
    )
    .unwrap();
    let planche = view["planches"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == a.as_str())
        .unwrap();
    assert_eq!(planche["series_supprimees"].as_array().unwrap().len(), 1);
    let over = invoke(
        &window,
        "series_depassant",
        json!({ "annee": year(), "id": a, "longueurCm": 200 }),
    )
    .unwrap();
    assert_eq!(over.as_array().unwrap().len(), 1);

    // Delete A: its serie goes with it.
    let view = invoke(
        &window,
        "supprimer_planche",
        json!({ "annee": year(), "id": a }),
    )
    .unwrap();
    assert_eq!(view["planches"].as_array().unwrap().len(), 1);
    assert_eq!(view["series"].as_array().unwrap().len(), 0);
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
