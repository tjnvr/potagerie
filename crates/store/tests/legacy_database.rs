//! A database written by the real app version 0.2.0 (French field names,
//! schema 1, column `mis_a_jour_le`) opens, reads and is saved again.
//!
//! `fixtures/potagerie-v0.2.0.db` was produced by the code of the `v0.2.0` tag
//! with its own commands: plot 11 × 6.5 m, "Planche A" (a bed) and "Butte B"
//! (a mound), three plantings (carrot in the open ground, tomato sown under
//! cover with an end edited by hand, green beans as plants) and one note.

use std::path::Path;

use chrono::{TimeZone, Utc};
use plan::Mode;
use store::{SqliteStore, Store};

fn copy_fixture(dir: &Path) -> std::path::PathBuf {
    let db = dir.join("potagerie.db");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/potagerie-v0.2.0.db"),
        &db,
    )
    .unwrap();
    db
}

#[test]
fn the_real_0_2_0_database_is_upgraded_and_saved_again() {
    let dir = tempfile::tempdir().unwrap();
    let db = copy_fixture(dir.path());

    let mut store = SqliteStore::open(&db).unwrap();
    assert_eq!(store.schema_version().unwrap(), 2);
    let mut plan = store.load().unwrap().expect("the plan is there");

    assert_eq!(plan.version, 7);
    assert_eq!(
        (plan.plot.length_cm, plan.plot.width_cm, plan.plot.aisle_cm),
        (1100, 650, 50)
    );
    let mut names: Vec<&str> = plan.beds.values().map(|b| b.name.as_str()).collect();
    names.sort_unstable();
    assert_eq!(names, ["Butte B", "Planche A"]);
    assert_eq!(plan.annotations.len(), 1);
    assert_eq!(
        plan.annotations.values().next().unwrap().text,
        "Dernières gelées ?"
    );

    let mut plantings: Vec<_> = plan.plantings.values().collect();
    plantings.sort_by(|a, b| a.crop_id.cmp(&b.crop_id));
    let summary: Vec<_> = plantings
        .iter()
        .map(|p| {
            (
                p.crop_id.as_str(),
                p.mode,
                p.indoor_days,
                p.manual_end.is_some(),
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            ("carrot", Mode::OpenGround, 0, false),
            ("green-bean", Mode::Plants, 0, false),
            ("tomato", Mode::Indoor, 56, true),
        ]
    );
    plan.check_structure().expect("the upgraded plan is sound");

    // Saved again, it is written in the current format and still reads.
    store
        .save(
            &mut plan,
            Utc.with_ymd_and_hms(2027, 1, 1, 12, 0, 0).unwrap(),
        )
        .unwrap();
    drop(store);
    let store = SqliteStore::open(&db).unwrap();
    let again = store.load().unwrap().unwrap();
    assert_eq!(again.version, 8);
    assert_eq!(again.beds.len(), 2);
    assert_eq!(again.plantings.len(), 3);
}
