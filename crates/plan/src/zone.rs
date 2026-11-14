//! Zones (intervals of troncons) and half-open occupations (S03 §1, §4).

use std::fmt;
use std::ops::RangeInclusive;

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::error::PlanError;
use crate::jours::{plus, premier_janvier};

/// An interval `[t1, t2]` of troncons, numbered from 1 (S03 §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Zone {
    pub t1: u32,
    pub t2: u32,
}

impl Zone {
    /// A zone with `1 ≤ t1 ≤ t2`. The upper bound depends on the planche
    /// and is checked by the plan.
    pub fn new(t1: u32, t2: u32) -> Result<Self, PlanError> {
        if t1 >= 1 && t1 <= t2 {
            Ok(Self { t1, t2 })
        } else {
            Err(PlanError::ZoneInvalide { t1, t2 })
        }
    }

    /// Number of troncons in the zone.
    pub fn nb_troncons(&self) -> u32 {
        self.t2.saturating_sub(self.t1) + 1
    }

    /// True when the zones share at least one troncon (S03 §5).
    pub fn chevauche(&self, autre: &Zone) -> bool {
        self.t1 <= autre.t2 && autre.t1 <= self.t2
    }

    /// The troncons the two zones have in common.
    pub fn intersection(&self, autre: &Zone) -> Option<Zone> {
        let t1 = self.t1.max(autre.t1);
        let t2 = self.t2.min(autre.t2);
        (t1 <= t2).then_some(Zone { t1, t2 })
    }

    /// True when `autre` lies entirely inside `self`.
    pub fn contient(&self, autre: &Zone) -> bool {
        self.t1 <= autre.t1 && autre.t2 <= self.t2
    }
}

impl fmt::Display for Zone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.t1, self.t2)
    }
}

/// A half-open period `[debut, fin[` (S03 §1): `debut` is occupied, `fin` is not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Occupation {
    pub debut: NaiveDate,
    pub fin: NaiveDate,
}

impl Occupation {
    pub fn est_vide(&self) -> bool {
        self.fin <= self.debut
    }

    /// `a.debut < b.fin and b.debut < a.fin` (S03 §5).
    pub fn chevauche(&self, autre: &Occupation) -> bool {
        self.debut < autre.fin && autre.debut < self.fin
    }

    /// `[max(debuts), min(fins)[`, if it is not empty.
    pub fn intersection(&self, autre: &Occupation) -> Option<Occupation> {
        let commune = Occupation {
            debut: self.debut.max(autre.debut),
            fin: self.fin.min(autre.fin),
        };
        (!commune.est_vide()).then_some(commune)
    }

    /// True if the day is occupied.
    pub fn contient(&self, jour: NaiveDate) -> bool {
        self.debut <= jour && jour < self.fin
    }

    /// The calendar years the occupation touches. A series ending on
    /// January 1st does not touch the new year.
    pub fn annees(&self) -> RangeInclusive<i32> {
        let derniere = if self.est_vide() {
            self.debut
        } else {
            plus(self.fin, -1)
        };
        self.debut.year()..=derniere.year()
    }

    /// True if at least one occupied day falls in `annee`.
    pub fn touche_annee(&self, annee: i32) -> bool {
        !self.est_vide()
            && self.debut < premier_janvier(annee.saturating_add(1))
            && self.fin > premier_janvier(annee)
    }
}

impl fmt::Display for Occupation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}[", self.debut, self.fin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, j: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, j).unwrap()
    }

    #[test]
    fn zone_refuse_les_bornes_invalides() {
        assert!(Zone::new(0, 3).is_err());
        assert!(Zone::new(4, 3).is_err());
        assert_eq!(Zone::new(2, 2).unwrap().nb_troncons(), 1);
    }

    #[test]
    fn zones_adjacentes_ne_se_chevauchent_pas() {
        let a = Zone::new(1, 2).unwrap();
        let b = Zone::new(3, 4).unwrap();
        assert!(!a.chevauche(&b));
        assert!(a.chevauche(&Zone::new(2, 4).unwrap()));
        assert_eq!(
            a.intersection(&Zone::new(2, 4).unwrap()),
            Zone::new(2, 2).ok()
        );
    }

    #[test]
    fn occupation_semi_ouverte() {
        let a = Occupation {
            debut: d(2027, 5, 5),
            fin: d(2027, 9, 27),
        };
        let b = Occupation {
            debut: d(2027, 9, 27),
            fin: d(2027, 12, 6),
        };
        assert!(!a.chevauche(&b));
        assert!(a.contient(d(2027, 9, 26)));
        assert!(!a.contient(d(2027, 9, 27)));
    }

    #[test]
    fn occupation_finie_le_1er_janvier_ne_touche_pas_la_nouvelle_annee() {
        let o = Occupation {
            debut: d(2027, 11, 1),
            fin: d(2028, 1, 1),
        };
        assert_eq!(o.annees(), 2027..=2027);
        assert!(o.touche_annee(2027));
        assert!(!o.touche_annee(2028));
        let o = Occupation {
            debut: d(2027, 11, 1),
            fin: d(2028, 1, 2),
        };
        assert_eq!(o.annees(), 2027..=2028);
        assert!(o.touche_annee(2028));
    }
}
