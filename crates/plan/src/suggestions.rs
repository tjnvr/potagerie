//! Suggestions for a free slot (S03 §9).

use std::cmp::Reverse;

use catalogue::{Catalogue, Culture};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::creneaux::Creneau;
use crate::error::PlanError;
use crate::fenetre::dans_fenetre;
use crate::jours::plus;
use crate::model::Plan;
use crate::serie::{Mode, Serie};
use crate::zone::Zone;

/// At most this many crops are suggested for a slot.
pub const MAX_SUGGESTIONS: usize = 3;

/// A crop proposed for a slot, with the values that prefill the new serie.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suggestion {
    pub culture_id: String,
    pub mode: Mode,
    /// Sowing date (abri, pleine terre) or planting date (plants).
    pub date: NaiveDate,
    pub zone: Zone,
    /// `m`.
    pub mise_en_place: NaiveDate,
    /// `r`.
    pub recolte: NaiveDate,
    /// Proposed end `f = min(F + 1, r + duree)`.
    pub fin: NaiveDate,
    /// Set when the end was shortened to fit the slot: it then counts as an end
    /// modified by hand and is not recomputed.
    pub fin_manuelle: Option<NaiveDate>,
    /// `f − D`: days of the slot the serie fills.
    pub jours_remplis: i64,
    /// `N = F − D + 1`: length of the slot.
    pub jours_creneau: i64,
}

impl Suggestion {
    /// Filling score `(f − D) ÷ N`, between 0 and 1.
    pub fn score(&self) -> f64 {
        self.jours_remplis as f64 / self.jours_creneau as f64
    }

    /// The score rounded to a tenth, in tenths (0 to 10), with integer maths.
    pub fn score_dixiemes(&self) -> i64 {
        (20 * self.jours_remplis + self.jours_creneau) / (2 * self.jours_creneau)
    }

    /// The serie to create when the suggestion is touched.
    pub fn en_serie(&self, culture: &Culture, creneau: &Creneau) -> Result<Serie, PlanError> {
        Serie::nouvelle(
            culture,
            creneau.planche_id,
            self.zone,
            self.mode,
            self.date,
            None,
            self.fin_manuelle,
        )
    }
}

/// Evaluates one crop for a slot, without ranking (S03 §9, steps 1 to 5).
///
/// Returns `None` when the crop is discarded: no planting date in the window
/// during the first 28 days of the slot, or not enough room for half of the
/// harvest.
pub fn evaluer(creneau: &Creneau, culture: &Culture, aujourd_hui: NaiveDate) -> Option<Suggestion> {
    let debut = creneau.debut;
    let fin_creneau = creneau.fin;

    // 1. The first day of [D, D + 27] in the planting window.
    let m = (0..28)
        .map(|i| plus(debut, i))
        .find(|jour| dans_fenetre(culture, *jour))?;

    // 2. Mode.
    let abri = i64::from(culture.abri_jours);
    let (mode, date) = if abri > 0 {
        let semis = plus(m, -abri);
        if semis >= aujourd_hui {
            (Mode::Abri, semis)
        } else {
            (Mode::Plants, m)
        }
    } else {
        (Mode::PleineTerre, m)
    };
    let serie = Serie {
        culture_id: culture.id.clone(),
        planche_id: creneau.planche_id,
        troncon_debut: creneau.zone.t1,
        troncon_fin: creneau.zone.t2,
        mode,
        date,
        abri_jours: culture.abri_jours,
        apres_jours: culture.recolte_apres_jours,
        duree_jours: culture.recolte_duree_jours,
        fin_manuelle: None,
    };
    let recolte = serie.dates().recolte;

    // 3. Minimal harvest: half the harvest duration, rounded up.
    let duree = i64::from(culture.recolte_duree_jours);
    let apres_creneau = plus(fin_creneau, 1);
    if plus(recolte, (duree + 1) / 2) > apres_creneau {
        return None;
    }

    // 4. Proposed end.
    let fin_naturelle = plus(recolte, duree);
    let fin = apres_creneau.min(fin_naturelle);

    Some(Suggestion {
        culture_id: culture.id.clone(),
        mode,
        date,
        zone: creneau.zone,
        mise_en_place: m,
        recolte,
        fin,
        fin_manuelle: (fin < fin_naturelle).then_some(fin),
        // 5. Filling score.
        jours_remplis: (fin - debut).num_days(),
        jours_creneau: creneau.longueur_jours(),
    })
}

impl Plan {
    /// Up to 3 crops for a slot, ranked (S03 §9):
    ///
    /// 1. score rounded to a tenth, highest first;
    /// 2. a family different from the last serie that ended before the slot on a
    ///    troncon of its zone;
    /// 3. alphabetical order of the name.
    pub fn suggestions(
        &self,
        creneau: &Creneau,
        catalogue: &Catalogue,
        aujourd_hui: NaiveDate,
    ) -> Vec<Suggestion> {
        let familles_precedentes = self.familles_derniere_serie_terminee(creneau, catalogue);
        let mut candidats: Vec<(Suggestion, &Culture)> = catalogue
            .iter()
            .filter_map(|c| Some((evaluer(creneau, c, aujourd_hui)?, c)))
            .collect();
        candidats.sort_by_key(|(suggestion, culture)| {
            (
                Reverse(suggestion.score_dixiemes()),
                familles_precedentes.contains(&culture.famille),
                culture.nom.to_lowercase(),
            )
        });
        candidats
            .into_iter()
            .take(MAX_SUGGESTIONS)
            .map(|(suggestion, _)| suggestion)
            .collect()
    }

    /// Families of the series that ended last (`f ≤ D`) on a troncon of the
    /// zone of the slot. Several series ending the same day all count.
    fn familles_derniere_serie_terminee(
        &self,
        creneau: &Creneau,
        catalogue: &Catalogue,
    ) -> Vec<String> {
        let terminees: Vec<(&Serie, NaiveDate)> = self
            .series
            .values()
            .filter(|s| s.planche_id == creneau.planche_id && s.zone().chevauche(&creneau.zone))
            .map(|s| (s, s.dates().fin))
            .filter(|(_, fin)| *fin <= creneau.debut)
            .collect();
        let Some(derniere) = terminees.iter().map(|(_, fin)| *fin).max() else {
            return Vec::new();
        };
        terminees
            .into_iter()
            .filter(|(_, fin)| *fin == derniere)
            .filter_map(|(s, _)| catalogue.culture(&s.culture_id))
            .map(|c| c.famille.clone())
            .collect()
    }
}
