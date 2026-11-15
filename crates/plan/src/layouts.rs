//! Layouts by year (S03 §7).

use crate::ids::BedId;
use crate::model::{Bed, Geometry, Layout, Plan};

impl Bed {
    /// The layout in force in `year`: the last one recorded for a year ≤ `year`.
    /// `None` if the bed does not exist yet, or was deleted.
    pub fn layout_in(&self, year: i32) -> Option<&Geometry> {
        match self.layouts.range(..=year).next_back() {
            Some((_, Layout::Present(geometry))) => Some(geometry),
            _ => None,
        }
    }
}

impl Plan {
    /// The beds that exist in `year`, with their layout.
    pub fn beds_in(&self, year: i32) -> Vec<(BedId, &Bed, &Geometry)> {
        self.beds
            .iter()
            .filter_map(|(id, bed)| Some((*id, bed, bed.layout_in(year)?)))
            .collect()
    }

    /// Number of segments of a bed in `year`, if it exists then.
    pub fn segment_count(&self, bed_id: BedId, year: i32) -> Option<u32> {
        let geometry = self.beds.get(&bed_id)?.layout_in(year)?;
        Some(geometry.segments(self.plot.cell_cm))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::model::TypeBed;

    fn geo(length_cm: u32) -> Geometry {
        Geometry {
            bed_type: TypeBed::Bed,
            x: 0,
            y: 0,
            length_cm,
            width_cm: 120,
        }
    }

    #[test]
    fn last_recorded_layout_holds_until_the_next() {
        let bed = Bed {
            name: "A".into(),
            layouts: BTreeMap::from([
                (2026, Layout::Present(geo(300))),
                (2028, Layout::Present(geo(200))),
                (2030, Layout::Absent),
            ]),
        };
        assert_eq!(bed.layout_in(2025), None);
        assert_eq!(bed.layout_in(2026).unwrap().length_cm, 300);
        assert_eq!(bed.layout_in(2027).unwrap().length_cm, 300);
        assert_eq!(bed.layout_in(2028).unwrap().length_cm, 200);
        assert_eq!(bed.layout_in(2029).unwrap().length_cm, 200);
        assert_eq!(bed.layout_in(2030), None);
        assert_eq!(bed.layout_in(2040), None);
    }
}
