//! Planting-out window of a crop (S03 §10).

use catalogue::Culture;
use chrono::{Datelike, NaiveDate};

/// The window `[début, fin]` that starts in year `annee`.
///
/// A window within one year is `[Y-debut, Y-fin]`; a window across two years
/// (`debut > fin`, garlic for example) is `[Y-debut, (Y+1)-fin]`.
pub fn fenetre_commencant_en(culture: &Culture, annee: i32) -> (NaiveDate, NaiveDate) {
    let periode = &culture.mise_en_place;
    let annee_fin = if periode.a_cheval() { annee + 1 } else { annee };
    (
        periode.debut.dans_annee(annee),
        periode.fin.dans_annee(annee_fin),
    )
}

/// True if `date` falls in the window of the crop, whichever year it started in.
pub fn dans_fenetre(culture: &Culture, date: NaiveDate) -> bool {
    [date.year() - 1, date.year()].into_iter().any(|annee| {
        let (debut, fin) = fenetre_commencant_en(culture, annee);
        debut <= date && date <= fin
    })
}

#[cfg(test)]
mod tests {
    use catalogue::{Catalogue, MoisJour, Periode};

    use super::*;

    fn d(y: i32, m: u32, j: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, j).unwrap()
    }

    #[test]
    fn fenetre_sur_une_annee_bornes_incluses() {
        let tomate = Catalogue::v1().culture("tomate").unwrap();
        assert!(!dans_fenetre(tomate, d(2027, 4, 30)));
        assert!(dans_fenetre(tomate, d(2027, 5, 1)));
        assert!(dans_fenetre(tomate, d(2027, 6, 15)));
        assert!(!dans_fenetre(tomate, d(2027, 6, 16)));
    }

    #[test]
    fn fenetre_a_cheval_sur_deux_annees() {
        let mut ail = Catalogue::v1().culture("tomate").unwrap().clone();
        ail.mise_en_place = Periode {
            debut: MoisJour::new(10, 15).unwrap(),
            fin: MoisJour::new(2, 15).unwrap(),
        };
        assert_eq!(
            fenetre_commencant_en(&ail, 2027),
            (d(2027, 10, 15), d(2028, 2, 15))
        );
        assert!(dans_fenetre(&ail, d(2027, 10, 15)));
        assert!(dans_fenetre(&ail, d(2027, 12, 31)));
        assert!(dans_fenetre(&ail, d(2028, 1, 1)));
        assert!(dans_fenetre(&ail, d(2028, 2, 15)));
        assert!(!dans_fenetre(&ail, d(2028, 2, 16)));
        assert!(!dans_fenetre(&ail, d(2027, 10, 14)));
    }

    #[test]
    fn fenetre_finissant_un_29_fevrier_en_annee_non_bissextile() {
        let mut culture = Catalogue::v1().culture("tomate").unwrap().clone();
        culture.mise_en_place = Periode {
            debut: MoisJour::new(2, 1).unwrap(),
            fin: MoisJour::new(2, 29).unwrap(),
        };
        assert!(dans_fenetre(&culture, d(2027, 2, 28)));
        assert!(!dans_fenetre(&culture, d(2027, 3, 1)));
        assert!(dans_fenetre(&culture, d(2028, 2, 29)));
    }
}
