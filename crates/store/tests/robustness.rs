//! Unusual but valid data and databases: the store must neither lose nor
//! refuse them (S05 §5).

use chrono::{NaiveDate, TimeZone, Utc};
use plan::{
    Annotation, AnnotationId, BedId, CellCm, Geometry, Plan, PlanId, Plot, TypeAnnotation, TypeBed,
};
use rusqlite::Connection;
use store::{SqliteStore, Store, StoreError};

fn now() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2027, 3, 1, 8, 0, 0).unwrap()
}

fn new_plan() -> Plan {
    Plan::new(
        PlanId::generate(),
        Plot {
            length_cm: 1100,
            width_cm: 650,
            cell_cm: CellCm::C50,
            aisle_cm: 50,
        },
        now(),
    )
}

fn day(y: i32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, 6, 1).unwrap()
}

#[test]
fn extreme_years_unicode_names_and_many_annotations_round_trip() {
    let mut plan = new_plan();
    for (i, (year, name)) in [(1900, "Bête à pois 🍅"), (2200, "菜園 – Ωmega"), (2027, "")]
        .into_iter()
        .enumerate()
    {
        plan.add_bed(
            BedId::generate(),
            name.to_owned(),
            year,
            Geometry {
                bed_type: TypeBed::Bed,
                x: 50,
                y: 50 + 200 * i as u32,
                length_cm: 300,
                width_cm: 120,
            },
            day(1900),
        )
        .unwrap();
    }
    for i in 0..500 {
        plan.add_annotation(
            AnnotationId::generate(),
            1900,
            Annotation {
                type_annotation: TypeAnnotation::Note,
                date_start: day(1900 + i % 300),
                date_end: None,
                bed_id: None,
                text: format!("note ✿ {i} e\u{301}"),
                hidden: false,
            },
            day(1900),
        )
        .unwrap();
    }
    let mut store = SqliteStore::open_in_memory().unwrap();
    store.save(&mut plan, now()).unwrap();
    store.save(&mut plan, now()).unwrap();
    assert_eq!(store.load().unwrap().unwrap(), plan);
}

/// A negative `user_version` is not a version this app wrote: it must be
/// refused, not treated as "migrated" (no table would exist).
#[test]
fn a_negative_schema_version_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("potagerie.db");
    Connection::open(&path)
        .unwrap()
        .pragma_update(None, "user_version", -1)
        .unwrap();
    assert!(SqliteStore::open(&path).is_err());
}

/// An imported plan may carry any `version` (`replace` keeps it): saving it
/// afterwards must work, and the version must stay exact.
#[test]
fn a_replaced_plan_with_a_huge_version_can_still_be_saved() {
    let mut store = SqliteStore::open_in_memory().unwrap();
    let mut imported = new_plan();
    imported.version = u64::MAX - 1;
    store.replace(&imported).unwrap();
    let mut loaded = store.load().unwrap().unwrap();
    store.save(&mut loaded, now()).unwrap();
    assert_eq!(loaded.version, u64::MAX);
    assert_eq!(store.load().unwrap().unwrap(), loaded);
}

/// A document that cannot be read is reported by `load`.
#[test]
fn a_corrupt_document_is_reported_by_load() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("potagerie.db");
    drop(SqliteStore::open(&path).unwrap());
    Connection::open(&path)
        .unwrap()
        .execute(
            "INSERT INTO plan (id, version, updated_at, document) VALUES (1, 3, 'x', '{oops')",
            [],
        )
        .unwrap();
    let store = SqliteStore::open(&path).unwrap();
    assert!(matches!(store.load(), Err(StoreError::Document(_))));
}

/// Reopening an already migrated database, several times, changes nothing;
/// an empty file is initialised.
#[test]
fn reopening_is_idempotent_and_an_empty_file_is_initialised() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("potagerie.db");
    std::fs::File::create(&path).unwrap();
    for _ in 0..3 {
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.schema_version().unwrap(), store::SCHEMA_VERSION);
        assert_eq!(store.load().unwrap(), None);
    }
}

#[test]
fn clear_then_save_starts_again_at_version_one() {
    let mut store = SqliteStore::open_in_memory().unwrap();
    let mut plan = new_plan();
    store.save(&mut plan, now()).unwrap();
    store.save(&mut plan, now()).unwrap();
    store.clear().unwrap();
    let mut fresh = new_plan();
    store.save(&mut fresh, now()).unwrap();
    assert_eq!(fresh.version, 1);
}
