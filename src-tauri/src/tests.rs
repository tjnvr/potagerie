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
    app.manage(AppState::open(db));
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
fn add_a_bed_and_find_it_after_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join(DB_FILE);
    {
        let (_app, window) = app(&db);
        let view = invoke(&window, "view_year", json!({ "year": year() })).unwrap();
        assert_eq!(view["plot"], Value::Null);

        let view = invoke(
            &window,
            "create_plot",
            json!({
                "year": year(),
                "plot": { "length_cm": 1100, "width_cm": 650, "cell_cm": 50, "aisle_cm": 50 }
            }),
        )
        .unwrap();
        assert_eq!(view["plot"]["cell_cm"], 50);
        assert_eq!(view["version"], 1);

        let view = invoke(
            &window,
            "create_bed",
            json!({ "year": year(), "bedType": "bed", "name": "Bed A", "x": 50, "y": 50 }),
        )
        .unwrap();
        assert_eq!(view["beds"][0]["name"], "Bed A");
        assert_eq!(view["beds"][0]["segments"], 6);
        assert_eq!(view["version"], 2);
    }
    let (_app, window) = app(&db);
    let view = invoke(&window, "view_year", json!({ "year": year() })).unwrap();
    assert_eq!(view["beds"].as_array().unwrap().len(), 1);
    assert_eq!(view["beds"][0]["name"], "Bed A");
}

#[test]
fn refusals_carry_a_code_and_change_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let (_app, window) = app(&dir.path().join(DB_FILE));

    let err = invoke(
        &window,
        "create_bed",
        json!({ "year": year(), "bedType": "square", "name": "Square A", "x": 0, "y": 0 }),
    )
    .unwrap_err();
    assert_eq!(err["code"], "plot_absent");

    invoke(
        &window,
        "create_plot",
        json!({
            "year": year(),
            "plot": { "length_cm": 1100, "width_cm": 650, "cell_cm": 50, "aisle_cm": 50 }
        }),
    )
    .unwrap();
    let err = invoke(
        &window,
        "create_bed",
        json!({ "year": year() - 1, "bedType": "square", "name": "Square A", "x": 0, "y": 0 }),
    )
    .unwrap_err();
    assert_eq!(err["code"], "year_past");
    let view = invoke(&window, "view_year", json!({ "year": year() })).unwrap();
    assert_eq!(view["version"], 1);
}

#[test]
fn export_import_and_clear() {
    let dir = tempfile::tempdir().unwrap();
    let (_app, window) = app(&dir.path().join(DB_FILE));
    invoke(
        &window,
        "create_plot",
        json!({
            "year": year(),
            "plot": { "length_cm": 1100, "width_cm": 650, "cell_cm": 50, "aisle_cm": 50 }
        }),
    )
    .unwrap();
    invoke(
        &window,
        "create_bed",
        json!({ "year": year(), "bedType": "planter", "name": "Planter A", "x": 50, "y": 50 }),
    )
    .unwrap();

    let file = dir.path().join("garden.json");
    invoke(&window, "export_garden", json!({ "path": file })).unwrap();
    invoke(&window, "erase_data", json!({})).unwrap();
    let view = invoke(&window, "view_year", json!({ "year": year() })).unwrap();
    assert_eq!(view["plot"], Value::Null);

    let summary = invoke(&window, "read_import", json!({ "path": file })).unwrap();
    assert_eq!(summary["bed_count"], 1);
    assert_eq!(summary["replaces"], false);
    let view = invoke(
        &window,
        "import_garden",
        json!({ "year": year(), "path": file }),
    )
    .unwrap();
    assert_eq!(view["beds"][0]["name"], "Planter A");
}

/// What the export screen sends (E11): the timeline drawn with the chosen
/// options (here a period and a note, annotations on) becomes a valid PNG and PDF.
#[test]
fn exported_timeline_is_a_valid_png_and_pdf() {
    let dir = tempfile::tempdir().unwrap();
    let (_app, window) = app(&dir.path().join(DB_FILE));
    let svg = r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="400" height="120" viewBox="0 0 400 120">
<rect width="400" height="120" fill="#fff"/>
<rect x="150" y="20" width="120" height="80" fill="#f1c40f" fill-opacity="0.3"/>
<text x="154" y="34" font-family="sans-serif" font-size="11" fill="#333">Mulching</text>
<line x1="60" y1="20" x2="60" y2="100" stroke="#c0392b"/>
<text x="64" y="60" font-family="sans-serif" font-size="11" fill="#333">Sow the radishes</text>
</svg>"##;
    for (format, name, magic) in [
        ("png", "timeline.png", &b"\x89PNG\r\n\x1a\n"[..]),
        ("pdf", "timeline.pdf", &b"%PDF-"[..]),
    ] {
        let file = dir.path().join(name);
        invoke(
            &window,
            "export_timeline",
            json!({ "svg": svg, "format": format, "path": file }),
        )
        .unwrap();
        let bytes = std::fs::read(&file).unwrap();
        assert!(bytes.starts_with(magic), "{name} has the wrong signature");
        assert!(bytes.len() > 500, "{name} looks empty");
    }
    // A path that cannot be written is an error for the file states (ET01).
    let err = invoke(
        &window,
        "export_timeline",
        json!({ "svg": svg, "format": "png", "path": dir.path().join("absent/timeline.png") }),
    )
    .unwrap_err();
    assert_eq!(err["code"], "file");
}

fn plot_args() -> Value {
    json!({
        "year": year(),
        "plot": { "length_cm": 1100, "width_cm": 650, "cell_cm": 50, "aisle_cm": 50 }
    })
}

fn place(
    window: &WebviewWindow<MockRuntime>,
    kind: &str,
    name: &str,
    x: u32,
    y: u32,
) -> Result<Value, Value> {
    invoke(
        window,
        "create_bed",
        json!({ "year": year(), "bedType": kind, "name": name, "x": x, "y": y }),
    )
}

/// The first-use journey: plot, beds, move, refusals, deletion.
#[test]
fn journey_plot_place_move_refuse_delete() {
    let dir = tempfile::tempdir().unwrap();
    let (_app, window) = app(&dir.path().join(DB_FILE));

    // Plot: dimensions and aisle are validated.
    for bad in [
        json!({ "length_cm": 50, "width_cm": 650, "cell_cm": 50, "aisle_cm": 50 }),
        json!({ "length_cm": 1100, "width_cm": 650, "cell_cm": 50, "aisle_cm": 40 }),
    ] {
        let err = invoke(
            &window,
            "create_plot",
            json!({ "year": year(), "plot": bad }),
        )
        .unwrap_err();
        assert!(["plot_invalid", "aisle_invalid"].contains(&err["code"].as_str().unwrap()));
    }
    let view = invoke(&window, "view_year", json!({ "year": year() })).unwrap();
    assert_eq!(view["plot"], Value::Null);
    invoke(&window, "create_plot", plot_args()).unwrap();

    // Two beds, 50 cm apart (the aisle width).
    let view = place(&window, "bed", "Bed A", 50, 50).unwrap();
    let a = view["beds"][0]["id"].as_str().unwrap().to_string();
    let view = place(&window, "square", "Square B", 400, 50).unwrap();
    assert_eq!(view["beds"].as_array().unwrap().len(), 2);
    let version = view["version"].clone();

    // Refusals: overlap, aisle too narrow, outside the plot. Nothing changes.
    let err = place(&window, "planter", "Planter C", 300, 100).unwrap_err();
    assert_eq!(err["code"], "bed_overlaps");
    let err = place(&window, "planter", "Planter C", 420, 100).unwrap_err();
    assert_eq!(err["code"], "bed_overlaps");
    let err = place(&window, "planter", "Planter C", 360, 190).unwrap_err();
    assert_eq!(err["code"], "aisle_too_narrow");
    let err = place(&window, "bed", "Bed D", 900, 50).unwrap_err();
    assert_eq!(err["code"], "bed_outside_plot");
    let err = place(&window, "bed", "", 50, 400).unwrap_err();
    assert_eq!(err["code"], "name_invalid");
    let view = invoke(&window, "view_year", json!({ "year": year() })).unwrap();
    assert_eq!(view["version"], version);
    assert_eq!(view["beds"].as_array().unwrap().len(), 2);

    // Move A: onto B is refused and it stays, then somewhere free works.
    let err = invoke(
        &window,
        "move_bed",
        json!({ "year": year(), "id": a, "x": 350, "y": 50 }),
    )
    .unwrap_err();
    assert_eq!(err["code"], "bed_overlaps");
    let view = invoke(
        &window,
        "move_bed",
        json!({ "year": year(), "id": a, "x": 50, "y": 300 }),
    )
    .unwrap();
    let moved = view["beds"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == a.as_str())
        .unwrap();
    assert_eq!(
        (moved["x"].as_u64(), moved["y"].as_u64()),
        (Some(50), Some(300))
    );

    // Resize: length in whole cases, width free, past the plot refused.
    let err = invoke(
        &window,
        "modify_bed",
        json!({ "year": year(), "id": a, "geometry": {
            "bed_type": "bed", "x": 50, "y": 300, "length_cm": 330, "width_cm": 120 } }),
    )
    .unwrap_err();
    assert_eq!(err["code"], "length_invalid");
    let view = invoke(
        &window,
        "modify_bed",
        json!({ "year": year(), "id": a, "name": "The tomatoes", "geometry": {
            "bed_type": "bed", "x": 50, "y": 300, "length_cm": 400, "width_cm": 135 } }),
    )
    .unwrap();
    assert_eq!(view["beds"][0]["name"], "The tomatoes");
    assert_eq!(view["beds"][0]["segments"], 8);

    // The plot cannot shrink under a bed.
    let err = invoke(
        &window,
        "modify_plot",
        json!({ "year": year(), "plot": { "length_cm": 1100, "width_cm": 400, "aisle_cm": 50 } }),
    )
    .unwrap_err();
    assert_eq!(err["code"], "bed_outside_plot");

    // A planting makes the deletion warn, and a shortening overflow.
    let planting = json!({
        "crop_id": "carrot", "bed_id": a, "segment_start": 1, "segment_end": 6,
        "mode": "open_ground", "date": format!("{}-03-15", year()),
        "indoor_days": null, "manual_end": null
    });
    let view = invoke(
        &window,
        "create_planting",
        json!({ "year": year(), "planting": planting }),
    )
    .unwrap();
    let bed = view["beds"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == a.as_str())
        .unwrap();
    assert_eq!(bed["plantings_deleted"].as_array().unwrap().len(), 1);
    let over = invoke(
        &window,
        "plantings_exceeding",
        json!({ "year": year(), "id": a, "lengthCm": 200 }),
    )
    .unwrap();
    assert_eq!(over.as_array().unwrap().len(), 1);

    // Delete A: its planting goes with it.
    let view = invoke(&window, "delete_bed", json!({ "year": year(), "id": a })).unwrap();
    assert_eq!(view["beds"].as_array().unwrap().len(), 1);
    assert_eq!(view["plantings"].as_array().unwrap().len(), 0);
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

// ---------- Plantings: the planning journey (S03 §2, §5, T1, T6, T9, T12, T13) ----------

/// A bed of 6 segments (3 m, cases of 50 cm) and its id.
fn bed_of_six(window: &WebviewWindow<MockRuntime>) -> String {
    invoke(window, "create_plot", plot_args()).unwrap();
    let view = place(window, "bed", "Bed A", 50, 50).unwrap();
    view["beds"][0]["id"].as_str().unwrap().to_owned()
}

fn planting_input(crop: &str, bed: &str, t1: u32, t2: u32, mode: &str, date: String) -> Value {
    json!({
        "crop_id": crop, "bed_id": bed, "segment_start": t1, "segment_end": t2,
        "mode": mode, "date": date, "indoor_days": null, "manual_end": null
    })
}

fn find_planting<'a>(view: &'a Value, crop: &str) -> &'a Value {
    view["plantings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["crop_id"] == crop)
        .unwrap()
}

#[test]
fn journey_plantings_create_move_edit_delete() {
    let dir = tempfile::tempdir().unwrap();
    let (_app, window) = app(&dir.path().join(DB_FILE));
    let bed = bed_of_six(&window);
    let y = year() + 1;
    let args = |planting: Value| json!({ "year": y, "planting": planting });

    // T1: tomato sown in pots on March 10th.
    let view = invoke(
        &window,
        "create_planting",
        args(planting_input(
            "tomato",
            &bed,
            1,
            3,
            "indoor",
            format!("{y}-03-10"),
        )),
    )
    .unwrap();
    let tomato = find_planting(&view, "tomato");
    assert_eq!(tomato["planting_out"], format!("{y}-05-05"));
    assert_eq!(tomato["harvest_start"], format!("{y}-07-19"));
    assert_eq!(tomato["end"], format!("{y}-09-27"));
    assert_eq!(tomato["pot_days"], 56);
    // S03 §6: 1 row, 3 segments of 50 cm on a 1.2 m bed: 2 plants.
    assert_eq!(tomato["rows"], 1);
    assert_eq!(tomato["plants_by_row"], 2);

    // T6: a lettuce starting the day the tomato ends does not conflict.
    let view = invoke(
        &window,
        "create_planting",
        args(planting_input(
            "lettuce",
            &bed,
            1,
            2,
            "plants",
            format!("{y}-09-27"),
        )),
    )
    .unwrap();
    let lettuce_id = find_planting(&view, "lettuce")["id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(view["conflicts"].as_array().unwrap().len(), 0);

    // T12: dragging it one day earlier is refused, and nothing changes.
    let version = view["version"].clone();
    let err = invoke(
        &window,
        "shift_planting",
        json!({ "year": y, "id": lettuce_id, "days": -1 }),
    )
    .unwrap_err();
    assert_eq!(err["code"], "conflict");
    let view = invoke(&window, "view_year", json!({ "year": y })).unwrap();
    assert_eq!(view["version"], version);
    assert_eq!(
        find_planting(&view, "lettuce")["date"],
        format!("{y}-09-27")
    );

    // The preview says where and when it conflicts, without saving anything.
    let moved = planting_input("lettuce", &bed, 1, 2, "plants", format!("{y}-09-26"));
    let preview = invoke(
        &window,
        "preview_planting",
        json!({ "id": lettuce_id, "planting": moved }),
    )
    .unwrap();
    let problem = &preview["problems"][0];
    assert_eq!(problem["code"], "conflict");
    assert_eq!(problem["zone"]["segment_start"], 1);
    assert_eq!(problem["zone"]["segment_end"], 2);
    assert_eq!(problem["start"], format!("{y}-09-26"));
    assert_eq!(problem["end"], format!("{y}-09-27"));
    assert!(preview["dates"]["end"].is_string());

    // A drag two days later is accepted and the dates follow.
    let view = invoke(
        &window,
        "shift_planting",
        json!({ "year": y, "id": lettuce_id, "days": 2 }),
    )
    .unwrap();
    assert_eq!(
        find_planting(&view, "lettuce")["date"],
        format!("{y}-09-29")
    );

    // A zone that does not fit in the bed, and a conflict on creation.
    let err = invoke(
        &window,
        "create_planting",
        args(planting_input(
            "carrot",
            &bed,
            5,
            9,
            "open_ground",
            format!("{y}-03-15"),
        )),
    )
    .unwrap_err();
    assert_eq!(err["code"], "zone_outside_bed");
    let err = invoke(
        &window,
        "create_planting",
        args(planting_input(
            "carrot",
            &bed,
            3,
            4,
            "open_ground",
            format!("{y}-04-01"),
        )),
    )
    .unwrap_err();
    assert_eq!(err["code"], "conflict");
    let view = invoke(&window, "view_year", json!({ "year": y })).unwrap();
    assert_eq!(view["plantings"].as_array().unwrap().len(), 2);

    // T13: a shortened end is kept as an end modified by hand; it must stay
    // after the start of harvest.
    let mut edit = planting_input("lettuce", &bed, 1, 2, "plants", format!("{y}-09-29"));
    edit["manual_end"] = json!(format!("{y}-11-25"));
    let view = invoke(
        &window,
        "modify_planting",
        json!({ "year": y, "id": lettuce_id, "planting": edit }),
    )
    .unwrap();
    let lettuce = find_planting(&view, "lettuce");
    assert_eq!(lettuce["end"], format!("{y}-11-25"));
    assert_eq!(lettuce["manual_end"], true);
    let mut bad = edit.clone();
    bad["manual_end"] = json!(format!("{y}-11-10"));
    let err = invoke(
        &window,
        "modify_planting",
        json!({ "year": y, "id": lettuce_id, "planting": bad }),
    )
    .unwrap_err();
    assert_eq!(err["code"], "manual_end_invalid");

    // Changing the crop copies the durations again (S03 §2): zucchini, 50 days.
    let other = planting_input("zucchini", &bed, 1, 2, "plants", format!("{y}-09-29"));
    let view = invoke(
        &window,
        "modify_planting",
        json!({ "year": y, "id": lettuce_id, "planting": other }),
    )
    .unwrap();
    let zucchini = find_planting(&view, "zucchini");
    assert_eq!(zucchini["harvest_start"], format!("{y}-11-18"));
    assert_eq!(zucchini["manual_end"], false);

    // Delete.
    let view = invoke(
        &window,
        "delete_planting",
        json!({ "year": y, "id": lettuce_id }),
    )
    .unwrap();
    assert_eq!(view["plantings"].as_array().unwrap().len(), 1);
    let err = invoke(
        &window,
        "delete_planting",
        json!({ "year": y, "id": lettuce_id }),
    )
    .unwrap_err();
    assert_eq!(err["code"], "planting_unknown");
}

#[test]
fn preview_enters_a_transplanting_date_and_offers_shortcuts() {
    let dir = tempfile::tempdir().unwrap();
    let (_app, window) = app(&dir.path().join(DB_FILE));
    let bed = bed_of_six(&window);
    let y = year() + 1;

    // T9: a carrot in pots needs the days; the transplanting date of
    // March 22nd is stored as a sowing date of March 1st.
    let mut carrot = planting_input("carrot", &bed, 1, 6, "indoor", format!("{y}-03-22"));
    let preview = invoke(
        &window,
        "preview_planting",
        json!({ "id": null, "planting": carrot }),
    )
    .unwrap();
    assert_eq!(preview["dates"], Value::Null);
    assert_eq!(preview["problems"][0]["code"], "pot_days_required");
    carrot["indoor_days"] = json!(21);
    carrot["date_is_transplanting"] = json!(true);
    let preview = invoke(
        &window,
        "preview_planting",
        json!({ "id": null, "planting": carrot }),
    )
    .unwrap();
    let dates = &preview["dates"];
    assert_eq!(dates["date"], format!("{y}-03-01"));
    assert_eq!(dates["planting_out"], format!("{y}-03-22"));
    assert_eq!(dates["harvest_start"], format!("{y}-06-30"));
    assert_eq!(dates["end"], format!("{y}-08-09"));
    assert_eq!(preview["problems"].as_array().unwrap().len(), 0);
    // 1.2 m wide, 25 cm between rows: 4 rows; 3 m of 4 cm: 75 plants per row.
    assert_eq!(dates["rows"], 4);
    assert_eq!(dates["plants_by_row"], 75);
    // The whole bed is free, and so are its fractions.
    assert_eq!(preview["shortcuts"]["all"]["segment_end"], 6);
    assert_eq!(preview["shortcuts"]["half"]["segment_end"], 3);

    // Saving stores the sowing date.
    let view = invoke(
        &window,
        "create_planting",
        json!({ "year": y, "planting": carrot }),
    )
    .unwrap();
    assert_eq!(find_planting(&view, "carrot")["date"], format!("{y}-03-01"));

    // The carrot now occupies the whole bed until August: the lettuce
    // conflicts, the shortcuts are greyed out, and Rust says on which segments.
    let preview = invoke(
        &window,
        "preview_planting",
        json!({ "id": null, "planting": planting_input("lettuce", &bed, 1, 6, "plants", format!("{y}-04-01")) }),
    )
    .unwrap();
    assert_eq!(preview["problems"][0]["code"], "conflict");
    assert_eq!(preview["problems"][0]["zone"]["segment_start"], 1);
    assert_eq!(preview["shortcuts"]["all"], Value::Null);
    assert_eq!(preview["shortcuts"]["quarter"], Value::Null);
    // The strip of the form shows the whole bed as taken during that time.
    assert_eq!(preview["occupied"][0]["segment_start"], 1);
    assert_eq!(preview["occupied"][0]["segment_end"], 6);
}

#[test]
fn crops_in_season_follow_the_planting_window() {
    let dir = tempfile::tempdir().unwrap();
    let (_app, window) = app(&dir.path().join(DB_FILE));
    let in_april = invoke(&window, "crops_of_season", json!({ "date": "2027-04-20" })).unwrap();
    let ids: Vec<&str> = in_april
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    // Of the 6 reference crops, only these two are in their window in April.
    assert!(ids.contains(&"carrot") && ids.contains(&"lettuce"));
    assert!(!ids.contains(&"tomato") && !ids.contains(&"leek"));
    let in_january = invoke(&window, "crops_of_season", json!({ "date": "2027-01-10" })).unwrap();
    assert_eq!(in_january.as_array().unwrap().len(), 0);
}

fn start_with_a_bed(window: &WebviewWindow<MockRuntime>) -> String {
    invoke(window, "create_plot", plot_args()).unwrap();
    let view = place(window, "bed", "Bed A", 50, 50).unwrap();
    view["beds"][0]["id"].as_str().unwrap().to_string()
}

fn annotation(kind: &str, start: &str, end: Value, bed: Value, text: &str) -> Value {
    json!({
        "annotation_type": kind, "date_start": start, "date_end": end,
        "bed_id": bed, "text": text, "hidden": false
    })
}

#[test]
fn annotations_are_created_edited_hidden_and_deleted() {
    let dir = tempfile::tempdir().unwrap();
    let (_app, window) = app(&dir.path().join(DB_FILE));
    let bed = start_with_a_bed(&window);
    let day = format!("{}-12-01", year());

    let view = invoke(
        &window,
        "create_annotation",
        json!({ "year": year(), "annotation":
            annotation("note", &day, Value::Null, json!(bed), "Mulch") }),
    )
    .unwrap();
    let note = &view["annotations"][0];
    assert_eq!(note["text"], "Mulch");
    let id = note["id"].as_str().unwrap().to_string();

    let mut hidden = annotation("note", &day, Value::Null, json!(bed), "Mulch");
    hidden["hidden"] = json!(true);
    let view = invoke(
        &window,
        "modify_annotation",
        json!({ "year": year(), "id": id, "annotation": hidden }),
    )
    .unwrap();
    assert_eq!(view["annotations"][0]["hidden"], true);

    let view = invoke(
        &window,
        "delete_annotation",
        json!({ "year": year(), "id": id }),
    )
    .unwrap();
    assert!(view["annotations"].as_array().unwrap().is_empty());
    let err = invoke(
        &window,
        "delete_annotation",
        json!({ "year": year(), "id": id }),
    )
    .unwrap_err();
    assert_eq!(err["code"], "annotation_unknown");
}

#[test]
fn annotation_refusals_carry_a_code_and_change_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let (_app, window) = app(&dir.path().join(DB_FILE));
    let bed = start_with_a_bed(&window);
    let before =
        invoke(&window, "view_year", json!({ "year": year() })).unwrap()["version"].clone();
    let y = year();
    let ghost = "0190c5a0-0000-7000-8000-000000000009";
    let june = format!("{y}-06-01");
    let june2 = format!("{y}-06-02");
    let last_year = format!("{}-06-01", y - 1);
    let long = "a".repeat(81);
    let cases = [
        (
            y,
            annotation("note", &last_year, Value::Null, Value::Null, "x"),
            "year_past",
        ),
        (
            y - 1,
            annotation("note", &june, Value::Null, Value::Null, "x"),
            "year_past",
        ),
        (
            y,
            annotation("note", &june, Value::Null, json!(ghost), "x"),
            "bed_unknown",
        ),
        (
            y,
            annotation("period", &june2, json!(june), json!(bed), "x"),
            "annotation_dates_invalid",
        ),
        (
            y,
            annotation("period", &june2, Value::Null, Value::Null, "x"),
            "annotation_dates_invalid",
        ),
        (
            y,
            annotation("note", &june, Value::Null, Value::Null, "  "),
            "text_invalid",
        ),
        (
            y,
            annotation("note", &june, Value::Null, Value::Null, &long),
            "text_invalid",
        ),
    ];
    for (year, a, code) in cases {
        let err = invoke(
            &window,
            "create_annotation",
            json!({ "year": year, "annotation": a }),
        )
        .unwrap_err();
        assert_eq!(err["code"], code);
    }
    let view = invoke(&window, "view_year", json!({ "year": y })).unwrap();
    assert_eq!(view["version"], before);
    assert!(view["annotations"].as_array().unwrap().is_empty());
}

#[test]
fn clear_plan_empties_the_year_and_the_following_ones() {
    let dir = tempfile::tempdir().unwrap();
    let (_app, window) = app(&dir.path().join(DB_FILE));
    let bed = start_with_a_bed(&window);
    place(&window, "square", "Square B", 600, 50).unwrap();
    let dec1 = format!("{}-12-01", year());
    let dec2 = format!("{}-12-02", year());
    for (day, target, text) in [
        (&dec1, json!(bed), "On la bed"),
        (&dec2, Value::Null, "Everywhere"),
    ] {
        invoke(
            &window,
            "create_annotation",
            json!({ "year": year(), "annotation": annotation("note", day, Value::Null, target, text) }),
        )
        .unwrap();
    }

    let err = invoke(&window, "clear_plan", json!({ "year": year() - 1 })).unwrap_err();
    assert_eq!(err["code"], "year_past");
    let view = invoke(&window, "clear_plan", json!({ "year": year() })).unwrap();
    assert!(view["beds"].as_array().unwrap().is_empty());
    assert_eq!(view["plot"]["length_cm"], 1100);
    assert_eq!(view["annotations"].as_array().unwrap().len(), 1);
    assert_eq!(view["annotations"][0]["text"], "Everywhere");
    // Next year is empty too, and an empty plan can be emptied again.
    let next = invoke(&window, "view_year", json!({ "year": year() + 1 })).unwrap();
    assert!(next["beds"].as_array().unwrap().is_empty());
    invoke(&window, "clear_plan", json!({ "year": year() })).unwrap();
}

fn plan_json(beds: Value, plantings: Value) -> Value {
    json!({
        "id": "0190c5a0-0000-7000-8000-000000000001", "version": 3,
        "updated_at": "2027-01-01T00:00:00Z",
        "plot": { "length_cm": 2000, "width_cm": 1000, "cell_cm": 50, "aisle_cm": 50 },
        "beds": beds, "plantings": plantings, "annotations": {}
    })
}

fn envelope(plan: Value) -> String {
    json!({ "format": "potagerie", "format_version": 1, "plan": plan }).to_string()
}

#[test]
fn invalid_imports_are_refused_and_the_plan_is_kept() {
    let dir = tempfile::tempdir().unwrap();
    let (_app, window) = app(&dir.path().join(DB_FILE));
    start_with_a_bed(&window);
    let file = dir.path().join("bad.json");
    let y = year();
    let short_layout = json!({"0190c5a0-0000-7000-8000-0000000000a1": {"name":"A","layouts":
        {y.to_string(): {"state":"present","type":"bed","x":0,"y":0,"length_cm":30,"width_cm":100}}}});
    let cases: Vec<(String, &str)> = vec![
        ("step from json".into(), "import_invalid"),
        (r#"{"hello":1}"#.into(), "import_invalid"),
        (
            r#"{"format":"potagerie","format_version":9,"plan":{}}"#.into(),
            "import_too_recent",
        ),
        (envelope(json!({ "id": "x" })), "import_invalid"),
        // A layout whose length is not a whole number of cases.
        (
            envelope(plan_json(short_layout, json!({}))),
            "import_invalid",
        ),
    ];
    for (text, code) in cases {
        std::fs::write(&file, text).unwrap();
        for cmd in ["read_import", "import_garden"] {
            let err = invoke(&window, cmd, json!({ "year": y, "path": file })).unwrap_err();
            assert_eq!(err["code"], code, "{cmd}");
        }
    }
    let missing = dir.path().join("nope.json");
    let err = invoke(&window, "read_import", json!({ "path": missing })).unwrap_err();
    assert_eq!(err["code"], "file");
    let view = invoke(&window, "view_year", json!({ "year": y })).unwrap();
    assert_eq!(view["beds"][0]["name"], "Bed A");
}

#[test]
fn incoherent_imports_are_accepted_with_a_list_and_nothing_is_deleted() {
    let dir = tempfile::tempdir().unwrap();
    let (_app, window) = app(&dir.path().join(DB_FILE));
    let file = dir.path().join("incoherent.json");
    let y = year();
    let p = |n: u32| format!("0190c5a0-0000-7000-8000-0000000000{n:02}");
    let bed = |name: &str, x: u32| {
        json!({"name": name, "layouts": {y.to_string(): {"state":"present","type":"bed",
            "x": x, "y": 0, "length_cm": 300, "width_cm": 100}}})
    };
    let planting = |bed_id: &str, t1: u32, t2: u32| {
        json!({"crop_id":"tomato","bed_id":bed_id,"segment_start":t1,"segment_end":t2,
            "mode":"plants","date":format!("{y}-05-20"),"indoor_days":0,"after_days":60,"duration_days":40})
    };
    // 11 beds, the first two touch each other; two plantings overlap on
    // bed 1; one planting points to a bed that does not exist.
    let mut beds = serde_json::Map::new();
    beds.insert(p(1), bed("A", 0));
    beds.insert(p(2), bed("B", 300));
    for n in 3..=11u32 {
        beds.insert(p(n), bed(&format!("P{n}"), 600 + (n - 3) * 100));
    }
    let plantings = json!({ p(20): planting(&p(1), 1, 3), p(21): planting(&p(1), 2, 4), p(22): planting(&p(99), 1, 1) });
    std::fs::write(&file, envelope(plan_json(Value::Object(beds), plantings))).unwrap();

    let summary = invoke(&window, "read_import", json!({ "path": file })).unwrap();
    let codes: Vec<&str> = summary["incoherences"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["code"].as_str().unwrap())
        .collect();
    for expected in [
        "too_many_beds",
        "overlap",
        "planting_orphan",
        "aisle_too_narrow",
    ] {
        assert!(codes.contains(&expected), "{expected} in {codes:?}");
    }
    let view = invoke(&window, "import_garden", json!({ "year": y, "path": file })).unwrap();
    assert_eq!(view["beds"].as_array().unwrap().len(), 11);
    // The planting of the unknown bed is kept, not shown.
    let again = invoke(&window, "read_import", json!({ "path": file })).unwrap();
    assert_eq!(again["planting_count"], 3);
}

#[test]
fn a_damaged_database_does_not_stop_the_app_and_can_be_erased() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join(DB_FILE);
    std::fs::write(&db, b"this is not a SQLite database, just text".repeat(50)).unwrap();

    let (_app, window) = app(&db);
    let err = invoke(&window, "view_year", json!({ "year": year() })).unwrap_err();
    assert_eq!(err["code"], "base_unreadable");
    let err = invoke(
        &window,
        "create_plot",
        json!({
            "year": year(),
            "plot": { "length_cm": 1100, "width_cm": 650, "cell_cm": 50, "aisle_cm": 50 }
        }),
    )
    .unwrap_err();
    assert_eq!(err["code"], "base_unreadable");

    // Erasing removes the damaged file and starts again from an empty garden.
    invoke(&window, "erase_data", json!({})).unwrap();
    let view = invoke(&window, "view_year", json!({ "year": year() })).unwrap();
    assert_eq!(view["plot"], Value::Null);
    invoke(
        &window,
        "create_plot",
        json!({
            "year": year(),
            "plot": { "length_cm": 1100, "width_cm": 650, "cell_cm": 50, "aisle_cm": 50 }
        }),
    )
    .unwrap();
}

#[test]
fn a_database_with_an_unreadable_document_is_reported_as_unreadable() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join(DB_FILE);
    {
        let (_app, window) = app(&db);
        invoke(&window, "view_year", json!({ "year": year() })).unwrap();
    }
    let conn = rusqlite::Connection::open(&db).unwrap();
    conn.execute(
        "INSERT INTO plan (id, version, updated_at, document) VALUES (1, 1, '2027-01-01T00:00:00Z', '{not json')",
        [],
    )
    .unwrap();
    drop(conn);
    let (_app, window) = app(&db);
    let err = invoke(&window, "view_year", json!({ "year": year() })).unwrap_err();
    assert_eq!(err["code"], "base_unreadable");
    invoke(&window, "erase_data", json!({})).unwrap();
    invoke(&window, "view_year", json!({ "year": year() })).unwrap();
}
