//! Quantities (S03 §6).

use catalogue::Culture;

use crate::model::CaseCm;

/// Rows × plants per row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quantite {
    pub rangs: u32,
    pub pieds_par_rang: u32,
}

impl Quantite {
    /// Total number of plants.
    pub fn total(&self) -> u32 {
        self.rangs * self.pieds_par_rang
    }
}

/// For a planche of width `largeur_cm`, a zone of `nb_troncons` troncons and a
/// case size `case`:
///
/// ```text
/// rangs          = max(1, ⌊L ÷ rang_cm⌋)
/// pieds par rang = max(1, ⌊(k × c) ÷ pied_cm⌋)
/// ```
///
/// A spacing of 0 in the catalogue counts as "no room for more than one".
pub fn quantite(largeur_cm: u32, nb_troncons: u32, case: CaseCm, culture: &Culture) -> Quantite {
    let longueur_zone = nb_troncons.saturating_mul(case.cm());
    Quantite {
        rangs: largeur_cm.checked_div(culture.rang_cm).unwrap_or(0).max(1),
        pieds_par_rang: longueur_zone
            .checked_div(culture.pied_cm)
            .unwrap_or(0)
            .max(1),
    }
}

#[cfg(test)]
mod tests {
    use catalogue::Catalogue;

    use super::*;

    fn q(id: &str, largeur: u32, k: u32) -> Quantite {
        quantite(
            largeur,
            k,
            CaseCm::C50,
            Catalogue::v1().culture(id).unwrap(),
        )
    }

    #[test]
    fn au_moins_un_rang_et_un_pied() {
        // Courgette: 100 × 100 cm; a 60 cm wide planche and a single 50 cm case.
        let quantite = q("courgette", 60, 1);
        assert_eq!((quantite.rangs, quantite.pieds_par_rang), (1, 1));
        assert_eq!(quantite.total(), 1);
    }

    #[test]
    fn arrondi_a_l_entier_inferieur() {
        // Tomato on 120 cm: ⌊120 ÷ 80⌋ = 1 row; 3 cases = 150 cm: ⌊150 ÷ 60⌋ = 2.
        assert_eq!(q("tomate", 120, 3).total(), 2);
        // Lettuce 30 × 30: 4 rows, ⌊(2 × 50) ÷ 30⌋ = 3 plants per row.
        let laitue = q("laitue", 120, 2);
        assert_eq!((laitue.rangs, laitue.pieds_par_rang), (4, 3));
    }

    #[test]
    fn espacement_nul_ne_panique_pas() {
        let mut culture = Catalogue::v1().culture("tomate").unwrap().clone();
        culture.rang_cm = 0;
        culture.pied_cm = 0;
        assert_eq!(quantite(120, 3, CaseCm::C50, &culture).total(), 1);
    }
}
