//! A document that arrives from a file (import, or a database edited by hand)
//! can hold any value serde accepts. None of them may crash the app, which
//! builds the year view of every plan it loads.

use std::panic::{AssertUnwindSafe, catch_unwind};

use catalogue::Catalogue;
use chrono::NaiveDate;
use plan::{Plan, YearView};
use serde_json::{Value, json};

fn doc(planting: Value, layout_year: i64) -> Value {
    json!({
        "id": "0192d5e0-0000-7000-8000-000000000001", "version": 1,
        "updated_at": "2027-06-01T10:00:00Z",
        "plot": {"length_cm": 2000, "width_cm": 1000, "cell_cm": 50, "aisle_cm": 50},
        "beds": {"0192d5e0-0000-7000-8000-0000000000a1": {"name": "A", "layouts": {
            layout_year.to_string(): {"state": "present", "type": "bed", "x": 0, "y": 0, "length_cm": 300, "width_cm": 120}}}},
        "plantings": {"0192d5e0-0000-7000-8000-0000000000b1": planting},
        "annotations": {}
    })
}

fn planting(date: &str, indoor: u64, after: u64, duration: u64, mode: &str) -> Value {
    json!({"crop_id": "tomato", "bed_id": "0192d5e0-0000-7000-8000-0000000000a1",
           "segment_start": 1, "segment_end": 3, "mode": mode, "date": date,
           "indoor_days": indoor, "after_days": after, "duration_days": duration})
}

/// Loads the document the way the app does and builds the view of `year`.
fn load_and_view(document: Value, year: i32) {
    let Ok(plan) = serde_json::from_value::<Plan>(plan::upgrade_legacy_json(document)) else {
        return; // refused by serde: fine
    };
    let today = NaiveDate::from_ymd_opt(2027, 6, 20).unwrap();
    // The import refuses a plan that fails this check: nothing else runs on it.
    if plan.check_structure().is_err() {
        return;
    }
    let _ = YearView::build(Some(&plan), year, today, Catalogue::v1());
    let _ = plan.incoherences();
}

/// Runs `f` on its own thread: it must finish (no panic) within a few seconds
/// (no endless loop either).
fn no_panic(name: &str, f: impl FnOnce() + Send + 'static) {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let ok = catch_unwind(AssertUnwindSafe(f)).is_ok();
        let _ = tx.send(ok);
    });
    match rx.recv_timeout(std::time::Duration::from_secs(5)) {
        Ok(true) => {}
        Ok(false) => panic!("panicked: {name}"),
        Err(_) => panic!("did not finish in 5 s: {name}"),
    }
}

#[test]
fn extreme_durations_do_not_panic() {
    for (name, p) in [
        (
            "huge indoor",
            planting("2027-03-10", u32::MAX as u64, 75, 70, "indoor"),
        ),
        (
            "huge after",
            planting("2027-03-10", 56, u32::MAX as u64, 70, "indoor"),
        ),
        (
            "huge duration",
            planting("2027-03-10", 56, 75, u32::MAX as u64, "indoor"),
        ),
        (
            "all huge, open ground",
            planting(
                "2027-03-10",
                u32::MAX as u64,
                u32::MAX as u64,
                u32::MAX as u64,
                "open_ground",
            ),
        ),
        ("zero everything", planting("2027-03-10", 0, 0, 0, "plants")),
        (
            "zero harvest",
            planting("2027-03-10", 0, 0, 1, "open_ground"),
        ),
    ] {
        for year in [2026, 2027, 2028] {
            let p = p.clone();
            no_panic(name, move || load_and_view(doc(p, 2027), year));
        }
    }
}

#[test]
fn extreme_dates_do_not_panic() {
    for date in [
        "0001-01-01",
        "0001-01-02",
        "9999-12-31",
        "9999-12-30",
        "+262142-12-31",
        "2027-02-29",
    ] {
        for mode in ["indoor", "plants", "open_ground"] {
            let p = planting(date, 56, 75, 70, mode);
            no_panic(date, move || load_and_view(doc(p, 2027), 2027));
        }
    }
}

#[test]
fn extreme_years_do_not_panic() {
    for year in [i32::MIN, -1, 0, 1, 9999, 10_000, 262_142, 262_143, i32::MAX] {
        no_panic("year", move || {
            load_and_view(
                doc(planting("2027-03-10", 56, 75, 70, "indoor"), 2027),
                year,
            )
        });
    }
    for layout_year in [-5_i64, 0, 262_143, i32::MAX as i64] {
        no_panic("layout year", move || {
            load_and_view(
                doc(planting("2027-03-10", 56, 75, 70, "indoor"), layout_year),
                2027,
            )
        });
    }
}

#[test]
fn layouts_far_apart_in_time_do_not_hang() {
    let mut document = doc(planting("2027-03-10", 56, 75, 70, "indoor"), 2027);
    let layouts = document["beds"]["0192d5e0-0000-7000-8000-0000000000a1"]["layouts"]
        .as_object_mut()
        .unwrap();
    let present = layouts["2027"].clone();
    layouts.insert("-2000000000".into(), present.clone());
    layouts.insert("2000000000".into(), present);
    no_panic("far layouts", move || load_and_view(document, 2027));
}

#[test]
fn the_legacy_upgrade_is_idempotent_and_does_not_panic_on_odd_json() {
    for odd in [
        json!(null),
        json!(1),
        json!("x"),
        json!([]),
        json!([{"planches": 1}]),
        json!({"series": null}),
        json!({"mode": 5, "type": {"a": 1}}),
    ] {
        let once = plan::upgrade_legacy_json(odd.clone());
        assert_eq!(plan::upgrade_legacy_json(once.clone()), once);
    }
}
