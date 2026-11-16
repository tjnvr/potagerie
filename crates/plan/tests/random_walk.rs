//! Random sequences of operations: after every operation that succeeds, the
//! plan must stay sound and its views must build. A failure prints the seed and
//! the operation so that it can be replayed.

mod common;

use catalogue::Catalogue;
use common::*;
use plan::{Geometry, Layout, Mode, Plan, PlanId, Planting, PlantingId, TypeBed, YearView};
use uuid::Uuid;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
}

fn random_geometry(r: &mut Rng) -> Geometry {
    Geometry {
        bed_type: [
            TypeBed::Bed,
            TypeBed::Square,
            TypeBed::Mound,
            TypeBed::Planter,
        ][r.below(4) as usize],
        x: (r.below(40) * 50) as u32,
        y: (r.below(18) * 50) as u32,
        length_cm: (r.below(8) as u32 + 1) * 50,
        width_cm: 50 + r.below(150) as u32,
    }
}

fn check(plan: &Plan, today: chrono::NaiveDate, what: &str, seed: u64, step: usize) {
    let ctx = format!("seed {seed} step {step}: {what}");
    plan.check_structure()
        .unwrap_or_else(|e| panic!("{ctx}: structure {e:?}"));
    for year in today.year_()..=today.year_() + 2 {
        let _ = YearView::build(Some(plan), year, today, Catalogue::v1());
    }
    // Shortening a bed under a planting is allowed after a warning (S03 §17): the
    // planting then overflows its bed and is listed, so it is not a failure here.
    let problems: Vec<_> = plan
        .incoherences()
        .into_iter()
        .filter(|i| !matches!(i, plan::Incoherence::ZoneOutOfBed { .. }))
        .collect();
    assert!(problems.is_empty(), "{ctx}: incoherences {problems:?}");
}

trait Year {
    fn year_(&self) -> i32;
}
impl Year for chrono::NaiveDate {
    fn year_(&self) -> i32 {
        chrono::Datelike::year(self)
    }
}

#[test]
fn random_operations_keep_the_plan_sound() {
    let today = d(2027, 3, 10);
    let crops: Vec<_> = Catalogue::v1().iter().collect();
    for seed in 1..=25u64 {
        let mut r = Rng(seed);
        let mut plan = Plan::new(PlanId::from_uuid(Uuid::from_u128(1)), plot(), now());
        let mut n: u128 = 10;
        for step in 0..150 {
            n += 1;
            let beds: Vec<_> = plan.beds.keys().copied().collect();
            let year = 2027 + r.below(3) as i32;
            let what;
            match r.below(7) {
                0 => {
                    what = "add_bed".to_string();
                    // As the command does: the placements of the following years too.
                    let before = plan.clone();
                    let ok = plan
                        .add_bed(
                            bed_id(n),
                            format!("B{n}"),
                            year,
                            random_geometry(&mut r),
                            today,
                        )
                        .and_then(|_| plan.check_placements(year));
                    if ok.is_err() {
                        plan = before;
                    }
                }
                1 if !beds.is_empty() => {
                    let id = beds[r.below(beds.len() as u64) as usize];
                    what = format!("move bed {id:?} in {year}");
                    let before = plan.clone();
                    let ok = plan
                        .set_layout(id, year, Layout::Present(random_geometry(&mut r)), today)
                        .and_then(|_| plan.check_placements(year));
                    if ok.is_err() {
                        plan = before;
                    }
                }
                2 if !beds.is_empty() => {
                    let id = beds[r.below(beds.len() as u64) as usize];
                    what = format!("delete bed {id:?} in {year}");
                    let _ = plan.delete_bed(id, year, today);
                }
                3 | 4 if !beds.is_empty() => {
                    let id = beds[r.below(beds.len() as u64) as usize];
                    let crop = crops[r.below(crops.len() as u64) as usize];
                    let t1 = r.below(4) as u32 + 1;
                    let t2 = t1 + r.below(3) as u32;
                    let mode = [Mode::OpenGround, Mode::Plants, Mode::Indoor][r.below(3) as usize];
                    let date = d(year, 1 + r.below(12) as u32, 1 + r.below(28) as u32);
                    what = format!("plant {} on {id:?} {t1}-{t2} {mode:?} {date}", crop.id);
                    if let Ok(zone) = plan::Zone::new(t1, t2)
                        && let Ok(p) = Planting::new(crop, id, zone, mode, date, None, None)
                    {
                        let _ = plan.add_planting(PlantingId::from_uuid(Uuid::from_u128(n)), p);
                    }
                }
                5 if !plan.plantings.is_empty() => {
                    let ids: Vec<_> = plan.plantings.keys().copied().collect();
                    let id = ids[r.below(ids.len() as u64) as usize];
                    what = "delete planting".into();
                    let _ = plan.delete_planting(id);
                }
                6 if r.below(10) == 0 => {
                    what = format!("clear from {year}");
                    let _ = plan.clear_from(year, today);
                }
                _ => {
                    what = "noop".into();
                }
            }
            check(&plan, today, &what, seed, step);
        }
    }
}

/// What the app offers must be accepted: every suggestion of every free slot can
/// be created as it is, in any state of the plan.
#[test]
fn every_suggestion_of_every_slot_can_be_created() {
    let today = d(2027, 3, 10);
    let crops: Vec<_> = Catalogue::v1().iter().collect();
    let mut offered = 0;
    for seed in 100..112u64 {
        let mut r = Rng(seed);
        let mut plan = Plan::new(PlanId::from_uuid(Uuid::from_u128(1)), plot(), now());
        let mut n: u128 = 10;
        for step in 0..60 {
            n += 1;
            let beds: Vec<_> = plan.beds.keys().copied().collect();
            if r.below(3) == 0 || beds.is_empty() {
                let before = plan.clone();
                let ok = plan
                    .add_bed(
                        bed_id(n),
                        format!("B{n}"),
                        2027,
                        random_geometry(&mut r),
                        today,
                    )
                    .and_then(|_| plan.check_placements(2027));
                if ok.is_err() {
                    plan = before;
                }
            } else {
                let id = beds[r.below(beds.len() as u64) as usize];
                let crop = crops[r.below(crops.len() as u64) as usize];
                let t1 = r.below(4) as u32 + 1;
                let zone = plan::Zone::new(t1, t1 + r.below(2) as u32);
                let date = d(2027, 1 + r.below(12) as u32, 1 + r.below(28) as u32);
                if let Ok(z) = zone
                    && let Ok(p) = Planting::new(crop, id, z, Mode::OpenGround, date, None, None)
                {
                    let _ = plan.add_planting(PlantingId::from_uuid(Uuid::from_u128(n)), p);
                }
            }
            for year in [2027, 2028] {
                for slot in plan.slots(year) {
                    for s in plan.suggestions_with_ties(&slot, Catalogue::v1(), today) {
                        offered += 1;
                        let crop = Catalogue::v1().crop(&s.crop_id).unwrap();
                        let planting = s.into_planting(crop, &slot).unwrap_or_else(|e| {
                            panic!(
                                "seed {seed} step {step}: {} cannot be built: {e:?}",
                                s.crop_id
                            )
                        });
                        let mut probe = plan.clone();
                        probe
                            .add_planting(PlantingId::from_uuid(Uuid::from_u128(999_999)), planting)
                            .unwrap_or_else(|e| {
                                panic!(
                                    "seed {seed} step {step} year {year}: offered {} in {:?}..{:?} on {:?} but refused: {e:?}",
                                    s.crop_id, slot.start, slot.end, slot.zone
                                )
                            });
                    }
                }
            }
        }
    }
    assert!(
        offered > 100,
        "the test must really offer something: {offered}"
    );
}
