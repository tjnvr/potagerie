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

// ---------- Series: the planning journey (S03 §2, §5, T1, T6, T9, T12, T13) ----------

/// A planche of 6 troncons (3 m, cases of 50 cm) and its id.
fn planche_of_six(window: &WebviewWindow<MockRuntime>) -> String {
    invoke(window, "creer_terrain", terrain_args()).unwrap();
    let view = place(window, "planche", "Planche A", 50, 50).unwrap();
    view["planches"][0]["id"].as_str().unwrap().to_owned()
}

fn serie_input(culture: &str, planche: &str, t1: u32, t2: u32, mode: &str, date: String) -> Value {
    json!({
        "culture_id": culture, "planche_id": planche, "troncon_debut": t1, "troncon_fin": t2,
        "mode": mode, "date": date, "abri_jours": null, "fin_manuelle": null
    })
}

fn find_serie<'a>(view: &'a Value, culture: &str) -> &'a Value {
    view["series"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["culture_id"] == culture)
        .unwrap()
}

#[test]
fn journey_series_create_move_edit_delete() {
    let dir = tempfile::tempdir().unwrap();
    let (_app, window) = app(&dir.path().join(DB_FILE));
    let planche = planche_of_six(&window);
    let y = year() + 1;
    let args = |serie: Value| json!({ "annee": y, "serie": serie });

    // T1: tomato sown in pots on March 10th.
    let view = invoke(
        &window,
        "creer_serie",
        args(serie_input(
            "tomate",
            &planche,
            1,
            3,
            "abri",
            format!("{y}-03-10"),
        )),
    )
    .unwrap();
    let tomate = find_serie(&view, "tomate");
    assert_eq!(tomate["planting_out"], format!("{y}-05-05"));
    assert_eq!(tomate["harvest_start"], format!("{y}-07-19"));
    assert_eq!(tomate["end"], format!("{y}-09-27"));
    assert_eq!(tomate["pot_days"], 56);
    // S03 §6: 1 row, 3 troncons of 50 cm on a 1.2 m planche: 2 plants.
    assert_eq!(tomate["rangs"], 1);
    assert_eq!(tomate["pieds_par_rang"], 2);

    // T6: a lettuce starting the day the tomato ends does not conflict.
    let view = invoke(
        &window,
        "creer_serie",
        args(serie_input(
            "laitue",
            &planche,
            1,
            2,
            "plants",
            format!("{y}-09-27"),
        )),
    )
    .unwrap();
    let laitue_id = find_serie(&view, "laitue")["id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(view["conflicts"].as_array().unwrap().len(), 0);

    // T12: dragging it one day earlier is refused, and nothing changes.
    let version = view["version"].clone();
    let err = invoke(
        &window,
        "decaler_serie",
        json!({ "annee": y, "id": laitue_id, "jours": -1 }),
    )
    .unwrap_err();
    assert_eq!(err["code"], "conflict");
    let view = invoke(&window, "vue_annee", json!({ "annee": y })).unwrap();
    assert_eq!(view["version"], version);
    assert_eq!(find_serie(&view, "laitue")["date"], format!("{y}-09-27"));

    // The preview says where and when it conflicts, without saving anything.
    let moved = serie_input("laitue", &planche, 1, 2, "plants", format!("{y}-09-26"));
    let apercu = invoke(
        &window,
        "apercu_serie",
        json!({ "id": laitue_id, "serie": moved }),
    )
    .unwrap();
    let probleme = &apercu["problemes"][0];
    assert_eq!(probleme["code"], "conflict");
    assert_eq!(probleme["zone"]["troncon_debut"], 1);
    assert_eq!(probleme["zone"]["troncon_fin"], 2);
    assert_eq!(probleme["start"], format!("{y}-09-26"));
    assert_eq!(probleme["end"], format!("{y}-09-27"));
    assert!(apercu["dates"]["end"].is_string());

    // A drag two days later is accepted and the dates follow.
    let view = invoke(
        &window,
        "decaler_serie",
        json!({ "annee": y, "id": laitue_id, "jours": 2 }),
    )
    .unwrap();
    assert_eq!(find_serie(&view, "laitue")["date"], format!("{y}-09-29"));

    // A zone that does not fit in the planche, and a conflict on creation.
    let err = invoke(
        &window,
        "creer_serie",
        args(serie_input(
            "carotte",
            &planche,
            5,
            9,
            "pleine_terre",
            format!("{y}-03-15"),
        )),
    )
    .unwrap_err();
    assert_eq!(err["code"], "zone_hors_planche");
    let err = invoke(
        &window,
        "creer_serie",
        args(serie_input(
            "carotte",
            &planche,
            3,
            4,
            "pleine_terre",
            format!("{y}-04-01"),
        )),
    )
    .unwrap_err();
    assert_eq!(err["code"], "conflict");
    let view = invoke(&window, "vue_annee", json!({ "annee": y })).unwrap();
    assert_eq!(view["series"].as_array().unwrap().len(), 2);

    // T13: a shortened end is kept as an end modified by hand; it must stay
    // after the start of harvest.
    let mut edit = serie_input("laitue", &planche, 1, 2, "plants", format!("{y}-09-29"));
    edit["fin_manuelle"] = json!(format!("{y}-11-25"));
    let view = invoke(
        &window,
        "modifier_serie",
        json!({ "annee": y, "id": laitue_id, "serie": edit }),
    )
    .unwrap();
    let laitue = find_serie(&view, "laitue");
    assert_eq!(laitue["end"], format!("{y}-11-25"));
    assert_eq!(laitue["manual_end"], true);
    let mut bad = edit.clone();
    bad["fin_manuelle"] = json!(format!("{y}-11-10"));
    let err = invoke(
        &window,
        "modifier_serie",
        json!({ "annee": y, "id": laitue_id, "serie": bad }),
    )
    .unwrap_err();
    assert_eq!(err["code"], "fin_manuelle_invalide");

    // Changing the crop copies the durations again (S03 §2): courgette, 50 days.
    let other = serie_input("courgette", &planche, 1, 2, "plants", format!("{y}-09-29"));
    let view = invoke(
        &window,
        "modifier_serie",
        json!({ "annee": y, "id": laitue_id, "serie": other }),
    )
    .unwrap();
    let courgette = find_serie(&view, "courgette");
    assert_eq!(courgette["harvest_start"], format!("{y}-11-18"));
    assert_eq!(courgette["manual_end"], false);

    // Delete.
    let view = invoke(
        &window,
        "supprimer_serie",
        json!({ "annee": y, "id": laitue_id }),
    )
    .unwrap();
    assert_eq!(view["series"].as_array().unwrap().len(), 1);
    let err = invoke(
        &window,
        "supprimer_serie",
        json!({ "annee": y, "id": laitue_id }),
    )
    .unwrap_err();
    assert_eq!(err["code"], "serie_inconnue");
}

#[test]
fn preview_enters_a_transplanting_date_and_offers_shortcuts() {
    let dir = tempfile::tempdir().unwrap();
    let (_app, window) = app(&dir.path().join(DB_FILE));
    let planche = planche_of_six(&window);
    let y = year() + 1;

    // T9: a carrot in pots needs the days; the transplanting date of
    // March 22nd is stored as a sowing date of March 1st.
    let mut carotte = serie_input("carotte", &planche, 1, 6, "abri", format!("{y}-03-22"));
    let apercu = invoke(
        &window,
        "apercu_serie",
        json!({ "id": null, "serie": carotte }),
    )
    .unwrap();
    assert_eq!(apercu["dates"], Value::Null);
    assert_eq!(apercu["problemes"][0]["code"], "pot_days_required");
    carotte["abri_jours"] = json!(21);
    carotte["date_est_repiquage"] = json!(true);
    let apercu = invoke(
        &window,
        "apercu_serie",
        json!({ "id": null, "serie": carotte }),
    )
    .unwrap();
    let dates = &apercu["dates"];
    assert_eq!(dates["date"], format!("{y}-03-01"));
    assert_eq!(dates["planting_out"], format!("{y}-03-22"));
    assert_eq!(dates["harvest_start"], format!("{y}-06-30"));
    assert_eq!(dates["end"], format!("{y}-08-09"));
    assert_eq!(apercu["problemes"].as_array().unwrap().len(), 0);
    // 1.2 m wide, 25 cm between rows: 4 rows; 3 m of 4 cm: 75 plants per row.
    assert_eq!(dates["rangs"], 4);
    assert_eq!(dates["pieds_par_rang"], 75);
    // The whole planche is free, and so are its fractions.
    assert_eq!(apercu["raccourcis"]["toute"]["troncon_fin"], 6);
    assert_eq!(apercu["raccourcis"]["moitie"]["troncon_fin"], 3);

    // Saving stores the sowing date.
    let view = invoke(
        &window,
        "creer_serie",
        json!({ "annee": y, "serie": carotte }),
    )
    .unwrap();
    assert_eq!(find_serie(&view, "carotte")["date"], format!("{y}-03-01"));

    // The carrot now occupies the whole planche until August: the lettuce
    // conflicts, the shortcuts are greyed out, and Rust says on which troncons.
    let apercu = invoke(
        &window,
        "apercu_serie",
        json!({ "id": null, "serie": serie_input("laitue", &planche, 1, 6, "plants", format!("{y}-04-01")) }),
    )
    .unwrap();
    assert_eq!(apercu["problemes"][0]["code"], "conflict");
    assert_eq!(apercu["problemes"][0]["zone"]["troncon_debut"], 1);
    assert_eq!(apercu["raccourcis"]["toute"], Value::Null);
    assert_eq!(apercu["raccourcis"]["quart"], Value::Null);
}

#[test]
fn crops_in_season_follow_the_planting_window() {
    let dir = tempfile::tempdir().unwrap();
    let (_app, window) = app(&dir.path().join(DB_FILE));
    let in_april = invoke(
        &window,
        "cultures_de_saison",
        json!({ "date": "2027-04-20" }),
    )
    .unwrap();
    let mut ids: Vec<&str> = in_april
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    ids.sort_unstable();
    assert_eq!(ids, ["carotte", "laitue"]);
    let in_january = invoke(
        &window,
        "cultures_de_saison",
        json!({ "date": "2027-01-10" }),
    )
    .unwrap();
    assert_eq!(in_january.as_array().unwrap().len(), 0);
}
