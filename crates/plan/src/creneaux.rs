//! Free slots (S03 §8).

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::days::plus_days;
use crate::ids::PlancheId;
use crate::model::Plan;
use crate::zone::Zone;

/// A free slot must last strictly more than this many days.
pub const MIN_CRENEAU_DAYS: i64 = 35;

/// A free rectangle of the planche: a zone, free every day of `[start, end]`
/// (both days included).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Creneau {
    pub planche_id: PlancheId,
    /// The year of the layout the slot was computed in.
    pub year: i32,
    pub zone: Zone,
    pub start: NaiveDate,
    pub end: NaiveDate,
}

impl Creneau {
    /// `N = F − D + 1`.
    pub fn length_days(&self) -> i64 {
        (self.end - self.start).num_days() + 1
    }
}

impl Plan {
    /// The free slots of every planche that exists in `year`.
    pub fn creneaux(&self, year: i32) -> Vec<Creneau> {
        self.planches
            .keys()
            .flat_map(|id| self.planche_creneaux(*id, year))
            .collect()
    }

    /// The free slots of one planche in `year` (empty if it does not exist then).
    ///
    /// The search window is `[Y-03-01, Y-11-15]`. A slot is a zone free on every
    /// day of a period that is longer than 35 days, cannot be extended in time
    /// (or beyond the window) and cannot be widened over the whole period. Slots
    /// contained in another are dropped.
    pub fn planche_creneaux(&self, planche_id: PlancheId, year: i32) -> Vec<Creneau> {
        let Some(n) = self.troncon_count(planche_id, year) else {
            return Vec::new();
        };
        let (Some(window_start), Some(window_end)) = (
            NaiveDate::from_ymd_opt(year, 3, 1),
            NaiveDate::from_ymd_opt(year, 11, 15),
        ) else {
            return Vec::new();
        };
        let n = n as usize;
        let day_count = usize::try_from((window_end - window_start).num_days() + 1).unwrap_or(0);

        // occupied[t][j]: troncon t+1 is occupied on day j of the window.
        let mut occupied = vec![vec![false; day_count]; n];
        for serie in self.series.values().filter(|s| s.planche_id == planche_id) {
            let occupation = serie.occupation();
            let first = occupation.start.max(window_start);
            let after_last = occupation.end.min(plus_days(window_end, 1));
            if first >= after_last {
                continue;
            }
            let days = day_index(window_start, first)..day_index(window_start, after_last);
            let t1 = serie.troncon_debut.max(1) as usize - 1;
            let t2 = (serie.troncon_fin as usize).min(n);
            for troncon_days in occupied.iter_mut().take(t2).skip(t1) {
                troncon_days[days.clone()].fill(true);
            }
        }

        let mut creneaux = Vec::new();
        for a in 0..n {
            let mut free = vec![true; day_count];
            for b in a..n {
                for (j, l) in free.iter_mut().enumerate() {
                    *l = *l && !occupied[b][j];
                }
                for (d0, d1) in true_runs(&free) {
                    let duration = i64::try_from(d1 - d0 + 1).unwrap_or(0);
                    if duration <= MIN_CRENEAU_DAYS {
                        continue;
                    }
                    let free_over_period = |t: usize| occupied[t][d0..=d1].iter().all(|o| !o);
                    let widenable = (a > 0 && free_over_period(a - 1))
                        || (b + 1 < n && free_over_period(b + 1));
                    if widenable {
                        continue;
                    }
                    creneaux.push(Creneau {
                        planche_id,
                        year,
                        zone: Zone {
                            t1: u32::try_from(a + 1).unwrap_or(u32::MAX),
                            t2: u32::try_from(b + 1).unwrap_or(u32::MAX),
                        },
                        start: plus_days(window_start, i64::try_from(d0).unwrap_or(0)),
                        end: plus_days(window_start, i64::try_from(d1).unwrap_or(0)),
                    });
                }
            }
        }
        keep_uncontained(creneaux)
    }
}

fn day_index(start: NaiveDate, day: NaiveDate) -> usize {
    usize::try_from((day - start).num_days()).unwrap_or(0)
}

/// Maximal runs of `true`, as inclusive `(first, last)` indices.
fn true_runs(valeurs: &[bool]) -> Vec<(usize, usize)> {
    let mut runs = Vec::new();
    let mut start = None;
    for (i, &v) in valeurs.iter().enumerate() {
        match (v, start) {
            (true, None) => start = Some(i),
            (false, Some(d)) => {
                runs.push((d, i - 1));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(d) = start {
        runs.push((d, valeurs.len() - 1));
    }
    runs
}

/// Drops the slots contained, in zone and in period, in another slot (S03 §8.4).
fn keep_uncontained(creneaux: Vec<Creneau>) -> Vec<Creneau> {
    let mut kept: Vec<Creneau> = creneaux
        .iter()
        .filter(|c| {
            !creneaux.iter().any(|other| {
                other != *c
                    && other.zone.contains(&c.zone)
                    && other.start <= c.start
                    && c.end <= other.end
            })
        })
        .copied()
        .collect();
    kept.sort();
    kept
}
