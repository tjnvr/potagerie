//! Quantities (S03 §6).

use catalogue::Culture;

use crate::model::CaseCm;

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

/// For a planche of width `largeur_cm`, a zone of `troncon_count` troncons and a
/// case size `case`:
///
/// ```text
/// rows          = max(1, ⌊L ÷ rang_cm⌋)
/// pieds par rang = max(1, ⌊(k × c) ÷ pied_cm⌋)
/// ```
///
/// A spacing of 0 in the catalogue counts as "no room for more than one".
pub fn quantity(largeur_cm: u32, troncon_count: u32, case: CaseCm, culture: &Culture) -> Quantity {
    let zone_length_cm = troncon_count.saturating_mul(case.cm());
    Quantity {
        rows: largeur_cm.checked_div(culture.rang_cm).unwrap_or(0).max(1),
        plants_per_row: zone_length_cm
            .checked_div(culture.pied_cm)
            .unwrap_or(0)
            .max(1),
    }
}

#[cfg(test)]
mod tests {
    use catalogue::Catalogue;

    use super::*;

    fn q(id: &str, width: u32, k: u32) -> Quantity {
        quantity(width, k, CaseCm::C50, Catalogue::v1().culture(id).unwrap())
    }

    #[test]
    fn at_least_one_row_and_one_plant() {
        // Courgette: 100 × 100 cm; a 60 cm wide planche and a single 50 cm case.
        let quantity = q("courgette", 60, 1);
        assert_eq!((quantity.rows, quantity.plants_per_row), (1, 1));
        assert_eq!(quantity.total(), 1);
    }

    #[test]
    fn rounds_down() {
        // Tomato on 120 cm: ⌊120 ÷ 80⌋ = 1 row; 3 cases = 150 cm: ⌊150 ÷ 60⌋ = 2.
        assert_eq!(q("tomate", 120, 3).total(), 2);
        // Lettuce 30 × 30: 4 rows, ⌊(2 × 50) ÷ 30⌋ = 3 plants per row.
        let laitue = q("laitue", 120, 2);
        assert_eq!((laitue.rows, laitue.plants_per_row), (4, 3));
    }

    #[test]
    fn zero_spacing_does_not_panic() {
        let mut culture = Catalogue::v1().culture("tomate").unwrap().clone();
        culture.rang_cm = 0;
        culture.pied_cm = 0;
        assert_eq!(quantity(120, 3, CaseCm::C50, &culture).total(), 1);
    }
}
