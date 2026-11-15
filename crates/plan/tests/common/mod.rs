//! Fixtures shared by the integration tests. Data come from S02, and the
//! bed is 1.2 m wide with 50 cm cases (S03 §12).
#![allow(dead_code)]

use catalogue::{Catalogue, Crop};
use chrono::{DateTime, NaiveDate, Utc};
use plan::{
    BedId, CellCm, Geometry, Mode, Plan, PlanId, Planting, PlantingId, Plot, TypeBed, Zone,
};
use uuid::Uuid;

/// Civil date.
pub fn d(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

/// "Aujourd'hui" of S03 §12 unless a case says otherwise.
pub fn today() -> NaiveDate {
    d(2026, 11, 14)
}

pub fn crop(id: &str) -> &'static Crop {
    Catalogue::reference().crop(id).unwrap()
}

pub fn zone(t1: u32, t2: u32) -> Zone {
    Zone::new(t1, t2).unwrap()
}

pub fn bed_id(n: u128) -> BedId {
    BedId::from_uuid(Uuid::from_u128(n))
}

pub fn planting_id(n: u128) -> PlantingId {
    PlantingId::from_uuid(Uuid::from_u128(n))
}

pub fn now() -> DateTime<Utc> {
    DateTime::from_timestamp(1_800_000_000, 0).unwrap()
}

pub fn plot() -> Plot {
    Plot {
        length_cm: 2000,
        width_cm: 1000,
        cell_cm: CellCm::C50,
        aisle_cm: 50,
    }
}

/// A bed of `segment_count` cases (50 cm), 1.2 m wide.
pub fn geometry(segment_count: u32) -> Geometry {
    Geometry {
        bed_type: TypeBed::Bed,
        x: 0,
        y: 0,
        length_cm: segment_count * 50,
        width_cm: 120,
    }
}

/// A plan with one bed of 6 segments (3 m × 1.2 m), present from 2026.
pub fn plan_with_bed() -> (Plan, BedId) {
    plan_with_bed_of(6)
}

pub fn plan_with_bed_of(segment_count: u32) -> (Plan, BedId) {
    let mut plan = Plan::new(PlanId::from_uuid(Uuid::from_u128(1)), plot(), now());
    let id = bed_id(100);
    plan.add_bed(id, "Bed A".into(), 2026, geometry(segment_count), today())
        .unwrap();
    (plan, id)
}

/// A planting of catalogue crop `crop_id`, without pots days or manual end.
pub fn planting(
    crop_id: &str,
    bed: BedId,
    t1: u32,
    t2: u32,
    mode: Mode,
    date: NaiveDate,
) -> Planting {
    Planting::new(crop(crop_id), bed, zone(t1, t2), mode, date, None, None).unwrap()
}

/// A planting of `crop_id` that occupies exactly `[start, end[`, whatever the
/// catalogue says: bought plants, harvest after 1 day, then the rest of the time.
pub fn occupying(
    crop_id: &str,
    bed: BedId,
    t1: u32,
    t2: u32,
    start: NaiveDate,
    end: NaiveDate,
) -> Planting {
    let mut s = planting(crop_id, bed, t1, t2, Mode::Plants, start);
    s.after_days = 1;
    s.duration_days = u32::try_from((end - start).num_days() - 1).unwrap();
    assert_eq!(s.occupation().end, end);
    s
}
