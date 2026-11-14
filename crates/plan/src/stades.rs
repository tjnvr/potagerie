//! Visual stages in the garden view (S03 §3).

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::serie::{Mode, Serie};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stade {
    Semis,
    JeunePousse,
    Adulte,
    Recolte,
}

/// What the garden view draws on the zone at a given day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "affichage", content = "stade", rename_all = "snake_case")]
pub enum Affichage {
    Rien,
    /// A label planted on the zone: sown under cover, not yet transplanted.
    Etiquette,
    Plante(Stade),
}

impl Serie {
    /// The display at day `d` (S03 §3).
    ///
    /// `p = (d − m) / (r − m)`. Thresholds are compared with integers; the
    /// bands are `[0, 0.15[`, `[0.15, 0.50[`, `[0.50, 1[` and `[1, ∞[`.
    pub fn affichage_a(&self, d: NaiveDate) -> Affichage {
        let dates = self.dates();
        if d < dates.mise_en_place {
            return match (self.mode, dates.semis) {
                (Mode::Abri, Some(semis)) if d >= semis => Affichage::Etiquette,
                _ => Affichage::Rien,
            };
        }
        if d >= dates.fin {
            return Affichage::Rien;
        }
        let ecoule = (d - dates.mise_en_place).num_days();
        let duree = (dates.recolte - dates.mise_en_place).num_days();
        let stade = if duree <= 0 || ecoule >= duree {
            Stade::Recolte
        } else if ecoule * 100 < duree * 15 {
            Stade::Semis
        } else if ecoule * 100 < duree * 50 {
            Stade::JeunePousse
        } else {
            Stade::Adulte
        };
        // Bought plants are never smaller than a young plant.
        let stade = if self.mode == Mode::Plants {
            stade.max(Stade::JeunePousse)
        } else {
            stade
        };
        Affichage::Plante(stade)
    }
}

#[cfg(test)]
mod tests {
    use catalogue::Catalogue;

    use super::*;
    use crate::ids::PlancheId;
    use crate::zone::Zone;

    fn d(y: i32, m: u32, j: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, j).unwrap()
    }

    /// Tomato sown under cover on 2027-03-10: m 05-05, r 07-19 (75 days), f 09-27.
    fn tomate(mode: Mode, date: NaiveDate) -> Serie {
        let culture = Catalogue::v1().culture("tomate").unwrap();
        Serie::nouvelle(
            culture,
            PlancheId::nouveau(),
            Zone::new(1, 1).unwrap(),
            mode,
            date,
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn etiquette_du_semis_a_la_mise_en_place() {
        let s = tomate(Mode::Abri, d(2027, 3, 10));
        assert_eq!(s.affichage_a(d(2027, 3, 9)), Affichage::Rien);
        assert_eq!(s.affichage_a(d(2027, 3, 10)), Affichage::Etiquette);
        assert_eq!(s.affichage_a(d(2027, 5, 4)), Affichage::Etiquette);
        assert_eq!(
            s.affichage_a(d(2027, 5, 5)),
            Affichage::Plante(Stade::Semis)
        );
    }

    #[test]
    fn rien_a_partir_de_la_fin() {
        let s = tomate(Mode::Abri, d(2027, 3, 10));
        assert_eq!(
            s.affichage_a(d(2027, 9, 26)),
            Affichage::Plante(Stade::Recolte)
        );
        assert_eq!(s.affichage_a(d(2027, 9, 27)), Affichage::Rien);
    }

    #[test]
    fn seuils_des_stades() {
        // 100 days from m to r: p = 0.15 at day 15, 0.50 at day 50, 1 at day 100.
        let mut s = tomate(Mode::PleineTerre, d(2027, 5, 1));
        s.abri_jours = 0;
        s.apres_jours = 100;
        s.duree_jours = 30;
        let m = d(2027, 5, 1);
        let jour = |n: i64| m + chrono::Duration::days(n);
        assert_eq!(s.affichage_a(jour(14)), Affichage::Plante(Stade::Semis));
        assert_eq!(
            s.affichage_a(jour(15)),
            Affichage::Plante(Stade::JeunePousse)
        );
        assert_eq!(
            s.affichage_a(jour(49)),
            Affichage::Plante(Stade::JeunePousse)
        );
        assert_eq!(s.affichage_a(jour(50)), Affichage::Plante(Stade::Adulte));
        assert_eq!(s.affichage_a(jour(99)), Affichage::Plante(Stade::Adulte));
        assert_eq!(s.affichage_a(jour(100)), Affichage::Plante(Stade::Recolte));
    }

    #[test]
    fn plants_achetes_jamais_sous_jeune_pousse() {
        let s = tomate(Mode::Plants, d(2027, 5, 5));
        assert_eq!(
            s.affichage_a(d(2027, 5, 5)),
            Affichage::Plante(Stade::JeunePousse)
        );
        assert_eq!(s.affichage_a(d(2027, 5, 4)), Affichage::Rien);
    }

    #[test]
    fn pleine_terre_n_a_pas_d_etiquette() {
        let s = tomate(Mode::PleineTerre, d(2027, 5, 5));
        assert_eq!(s.affichage_a(d(2027, 5, 4)), Affichage::Rien);
    }

    #[test]
    fn recolte_immediate_si_aucun_delai() {
        let mut s = tomate(Mode::Plants, d(2027, 5, 5));
        s.apres_jours = 0;
        assert_eq!(
            s.affichage_a(d(2027, 5, 5)),
            Affichage::Plante(Stade::Recolte)
        );
    }
}
