//! Layouts by year (S03 §7).

use crate::ids::PlancheId;
use crate::model::{Disposition, Geometry, Plan, Planche};

impl Planche {
    /// The layout in force in `year`: the last one recorded for a year ≤ `year`.
    /// `None` if the planche does not exist yet, or was deleted.
    pub fn disposition_in(&self, year: i32) -> Option<&Geometry> {
        match self.dispositions.range(..=year).next_back() {
            Some((_, Disposition::Present(geometry))) => Some(geometry),
            _ => None,
        }
    }
}

impl Plan {
    /// The planches that exist in `year`, with their layout.
    pub fn planches_in(&self, year: i32) -> Vec<(PlancheId, &Planche, &Geometry)> {
        self.planches
            .iter()
            .filter_map(|(id, planche)| Some((*id, planche, planche.disposition_in(year)?)))
            .collect()
    }

    /// Number of troncons of a planche in `year`, if it exists then.
    pub fn troncon_count(&self, planche_id: PlancheId, year: i32) -> Option<u32> {
        let geometry = self.planches.get(&planche_id)?.disposition_in(year)?;
        Some(geometry.troncons(self.terrain.case_cm))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::model::TypePlanche;

    fn geo(longueur_cm: u32) -> Geometry {
        Geometry {
            planche_type: TypePlanche::Planche,
            x: 0,
            y: 0,
            longueur_cm,
            largeur_cm: 120,
        }
    }

    #[test]
    fn last_recorded_disposition_holds_until_the_next() {
        let planche = Planche {
            nom: "A".into(),
            dispositions: BTreeMap::from([
                (2026, Disposition::Present(geo(300))),
                (2028, Disposition::Present(geo(200))),
                (2030, Disposition::Absent),
            ]),
        };
        assert_eq!(planche.disposition_in(2025), None);
        assert_eq!(planche.disposition_in(2026).unwrap().longueur_cm, 300);
        assert_eq!(planche.disposition_in(2027).unwrap().longueur_cm, 300);
        assert_eq!(planche.disposition_in(2028).unwrap().longueur_cm, 200);
        assert_eq!(planche.disposition_in(2029).unwrap().longueur_cm, 200);
        assert_eq!(planche.disposition_in(2030), None);
        assert_eq!(planche.disposition_in(2040), None);
    }
}
