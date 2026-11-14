//! Validity of a serie, conflicts and zone shortcuts (S03 §4, §5).

use crate::conflits::{DetailConflit, conflit};
use crate::error::PlanError;
use crate::ids::{PlancheId, SerieId};
use crate::jours::premier_janvier;
use crate::model::Plan;
use crate::serie::Serie;
use crate::zone::{Occupation, Zone};

/// The ½, ⅓ and ¼ zone shortcuts, plus the whole planche (S01 F-PLA-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fraction {
    Entiere,
    Moitie,
    Tiers,
    Quart,
}

impl Fraction {
    fn numerateur_denominateur(self) -> (u32, u32) {
        match self {
            Self::Entiere => (1, 1),
            Self::Moitie => (1, 2),
            Self::Tiers => (1, 3),
            Self::Quart => (1, 4),
        }
    }

    /// `k = max(1, arrondi(n × fraction))`, rounding halves up (S03 §4).
    pub fn taille(self, n: u32) -> u32 {
        let (num, den) = self.numerateur_denominateur();
        let (n, num, den) = (u64::from(n), u64::from(num), u64::from(den));
        let k = (2 * n * num + den) / (2 * den);
        u32::try_from(k).unwrap_or(u32::MAX).max(1)
    }
}

impl Plan {
    /// Everything that makes `serie` invalid in this plan (S03 §5): a planche
    /// missing in a year it occupies, a zone that does not fit, conflicts.
    ///
    /// `ignorer` is the id of the serie being edited, so it is not compared
    /// with its previous self. The list is empty when the serie is valid.
    pub fn problemes_serie(&self, ignorer: Option<SerieId>, serie: &Serie) -> Vec<PlanError> {
        if let Err(erreur) = serie.verifier() {
            return vec![erreur];
        }
        let Some(planche) = self.planches.get(&serie.planche_id) else {
            return vec![PlanError::PlancheInconnue(serie.planche_id)];
        };
        let zone = serie.zone();
        let mut problemes = Vec::new();
        for annee in serie.occupation().annees() {
            match planche.disposition_en(annee) {
                None => problemes.push(PlanError::PlancheAbsente {
                    planche_id: serie.planche_id,
                    annee,
                }),
                Some(geometrie) => {
                    let troncons = geometrie.troncons(self.terrain.case_cm);
                    if zone.t2 > troncons {
                        problemes.push(PlanError::ZoneHorsPlanche {
                            annee,
                            troncons,
                            zone,
                        });
                    }
                }
            }
        }
        for (autre_id, autre) in &self.series {
            if Some(*autre_id) == ignorer {
                continue;
            }
            if let Some(DetailConflit { troncons, periode }) = conflit(serie, autre) {
                problemes.push(PlanError::Conflit {
                    avec: *autre_id,
                    troncons,
                    periode,
                });
            }
        }
        problemes
    }

    /// `Ok` if the serie is valid, otherwise the first problem.
    pub fn valider_serie(&self, ignorer: Option<SerieId>, serie: &Serie) -> Result<(), PlanError> {
        match self.problemes_serie(ignorer, serie).into_iter().next() {
            Some(erreur) => Err(erreur),
            None => Ok(()),
        }
    }

    /// Every pair of conflicting series of the plan, each pair once.
    pub fn conflits(&self) -> Vec<(SerieId, SerieId, DetailConflit)> {
        let mut paires = Vec::new();
        for (a_id, a) in &self.series {
            for (b_id, b) in self.series.range(*a_id..).skip(1) {
                if let Some(detail) = conflit(a, b) {
                    paires.push((*a_id, *b_id, detail));
                }
            }
        }
        paires
    }

    /// The series that would no longer fit if `planche_id` were shortened to
    /// `nouveau_nb_troncons` from `annee` on (S03 §4): `t2 > n′` and an occupation
    /// that touches `annee` or a later year.
    pub fn series_hors_planche_si_raccourcie(
        &self,
        planche_id: PlancheId,
        annee: i32,
        nouveau_nb_troncons: u32,
    ) -> Vec<SerieId> {
        let debut_annee = premier_janvier(annee);
        self.series
            .iter()
            .filter(|(_, s)| s.planche_id == planche_id)
            .filter(|(_, s)| s.troncon_fin > nouveau_nb_troncons)
            .filter(|(_, s)| s.dates().fin > debut_annee)
            .map(|(id, _)| *id)
            .collect()
    }

    /// The zone proposed by a shortcut: the first run of `k` troncons that is
    /// free over the whole `occupation` and fits in the planche every year it
    /// touches. `None` greys the shortcut out (S03 §4).
    ///
    /// `n` is the length of the planche in the year the occupation starts.
    /// `ignorer` is the serie being edited, if any.
    pub fn raccourci(
        &self,
        planche_id: PlancheId,
        fraction: Fraction,
        occupation: Occupation,
        ignorer: Option<SerieId>,
    ) -> Option<Zone> {
        let planche = self.planches.get(&planche_id)?;
        let annee_debut = chrono::Datelike::year(&occupation.debut);
        let n = planche
            .disposition_en(annee_debut)?
            .troncons(self.terrain.case_cm);
        let k = fraction.taille(n);
        let annees = occupation.annees();
        let tient = |zone: &Zone| {
            annees.clone().all(|annee| {
                planche
                    .disposition_en(annee)
                    .is_some_and(|g| g.troncons(self.terrain.case_cm) >= zone.t2)
            })
        };
        let libre = |zone: &Zone| {
            self.series
                .iter()
                .filter(|(id, s)| Some(**id) != ignorer && s.planche_id == planche_id)
                .all(|(_, s)| !(s.zone().chevauche(zone) && s.occupation().chevauche(&occupation)))
        };
        (1..=n.checked_sub(k)? + 1)
            .map(|t1| Zone { t1, t2: t1 + k - 1 })
            .find(|zone| tient(zone) && libre(zone))
    }
}
