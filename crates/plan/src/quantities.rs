//! Quantities (S03 §6).

use catalogue::Crop;

use crate::model::CellCm;

/// Rows × plants per row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quantity {
    pub rows: u32,
    pub plants_per_row: u32,
}

impl Quantity {
    /// Total number of plants.
    pub fn total(&self) -> u32 {
        self.rows * self.plants_per_row
    }
}

/// For a bed of width `width_cm`, a zone of `segment_count` segments and a
/// case size `case`:
///
/// ```text
/// rows          = max(1, ⌊L ÷ row_cm⌋)
/// plants by row = max(1, ⌊(k × c) ÷ spacing_cm⌋)
/// ```
///
/// A spacing of 0 in the catalogue counts as "no room for more than one".
pub fn quantity(width_cm: u32, segment_count: u32, case: CellCm, crop: &Crop) -> Quantity {
    let zone_length_cm = segment_count.saturating_mul(case.cm());
    Quantity {
        rows: width_cm.checked_div(crop.row_cm).unwrap_or(0).max(1),
        plants_per_row: zone_length_cm
            .checked_div(crop.spacing_cm)
            .unwrap_or(0)
            .max(1),
    }
}

#[cfg(test)]
mod tests {
    use catalogue::Catalogue;

    use super::*;

    fn q(id: &str, width: u32, k: u32) -> Quantity {
        quantity(
            width,
            k,
            CellCm::C50,
            Catalogue::reference().crop(id).unwrap(),
        )
    }

    #[test]
    fn at_least_one_row_and_one_plant() {
        // Zucchini: 100 × 100 cm; a 60 cm wide bed and a single 50 cm case.
        let quantity = q("zucchini", 60, 1);
        assert_eq!((quantity.rows, quantity.plants_per_row), (1, 1));
        assert_eq!(quantity.total(), 1);
    }

    #[test]
    fn rounds_down() {
        // Tomato on 120 cm: ⌊120 ÷ 80⌋ = 1 row; 3 cases = 150 cm: ⌊150 ÷ 60⌋ = 2.
        assert_eq!(q("tomato", 120, 3).total(), 2);
        // Lettuce 30 × 30: 4 rows, ⌊(2 × 50) ÷ 30⌋ = 3 plants per row.
        let lettuce = q("lettuce", 120, 2);
        assert_eq!((lettuce.rows, lettuce.plants_per_row), (4, 3));
    }

    #[test]
    fn zero_spacing_does_not_panic() {
        let mut crop = Catalogue::reference().crop("tomato").unwrap().clone();
        crop.row_cm = 0;
        crop.spacing_cm = 0;
        assert_eq!(quantity(120, 3, CellCm::C50, &crop).total(), 1);
    }
}
