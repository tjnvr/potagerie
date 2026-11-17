//! A damaged or hostile garden file must be refused or read, never panic, and a
//! file that is read must not hang the views built from it.

use std::time::{Duration, Instant};

use catalogue::Catalogue;
use chrono::NaiveDate;
use export::{export_garden, read_garden};
use plan::{CellCm, Plan, PlanId, Plot, YearView};

fn sample(dir: &std::path::Path) -> Vec<u8> {
    let plan = Plan::new(
        PlanId::generate(),
        Plot {
            length_cm: 1100,
            width_cm: 650,
            cell_cm: CellCm::C50,
            aisle_cm: 50,
        },
        chrono::DateTime::from_timestamp(1_800_000_000, 0).unwrap(),
    );
    let path = dir.join("g.json");
    export_garden(&plan, &path).unwrap();
    std::fs::read(path).unwrap()
}

#[test]
fn mutated_files_never_panic_nor_hang() {
    let dir = tempfile::tempdir().unwrap();
    let original = sample(dir.path());
    let today = NaiveDate::from_ymd_opt(2027, 6, 1).unwrap();
    let mut seed: u64 = 7;
    let mut next = || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) as usize
    };
    let started = Instant::now();
    for case in 0..600 {
        let mut bytes = original.clone();
        match case % 4 {
            0 => bytes.truncate(next() % bytes.len()),
            1 => {
                for _ in 0..1 + next() % 4 {
                    let i = next() % bytes.len();
                    bytes[i] = (next() % 256) as u8;
                }
            }
            2 => {
                let i = next() % bytes.len();
                bytes.insert(i, b"0123456789-\"{}[],:"[next() % 18]);
            }
            _ => {
                let i = next() % bytes.len();
                bytes.remove(i);
            }
        }
        let path = dir.path().join(format!("m{case}.json"));
        std::fs::write(&path, &bytes).unwrap();
        if let Ok(plan) = read_garden(&path) {
            let _ = plan.incoherences();
            let _ = YearView::build(Some(&plan), 2027, today, Catalogue::v1());
        }
    }
    assert!(
        started.elapsed() < Duration::from_secs(60),
        "too slow: {:?}",
        started.elapsed()
    );
}
