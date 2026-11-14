//! Free slots (S03 §8).

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::ids::PlancheId;
use crate::jours::plus;
use crate::model::Plan;
use crate::zone::Zone;

/// A free slot must last strictly more than this many days.
pub const DUREE_MIN_CRENEAU: i64 = 35;

/// A free rectangle of the planche: a zone, free every day of `[debut, fin]`
/// (both days included).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Creneau {
    pub planche_id: PlancheId,
    /// The year of the layout the slot was computed in.
    pub annee: i32,
    pub zone: Zone,
    pub debut: NaiveDate,
    pub fin: NaiveDate,
}

impl Creneau {
    /// `N = F − D + 1`.
    pub fn longueur_jours(&self) -> i64 {
        (self.fin - self.debut).num_days() + 1
    }
}

impl Plan {
    /// The free slots of every planche that exists in `annee`.
    pub fn creneaux(&self, annee: i32) -> Vec<Creneau> {
        self.planches
            .keys()
            .flat_map(|id| self.creneaux_planche(*id, annee))
            .collect()
    }

    /// The free slots of one planche in `annee` (empty if it does not exist then).
    ///
    /// The search window is `[Y-03-01, Y-11-15]`. A slot is a zone free on every
    /// day of a period that is longer than 35 days, cannot be extended in time
    /// (or beyond the window) and cannot be widened over the whole period. Slots
    /// contained in another are dropped.
    pub fn creneaux_planche(&self, planche_id: PlancheId, annee: i32) -> Vec<Creneau> {
        let Some(n) = self.nb_troncons(planche_id, annee) else {
            return Vec::new();
        };
        let (Some(debut_fenetre), Some(fin_fenetre)) = (
            NaiveDate::from_ymd_opt(annee, 3, 1),
            NaiveDate::from_ymd_opt(annee, 11, 15),
        ) else {
            return Vec::new();
        };
        let n = n as usize;
        let nb_jours = usize::try_from((fin_fenetre - debut_fenetre).num_days() + 1).unwrap_or(0);

        // occupe[t][j]: troncon t+1 is occupied on day j of the window.
        let mut occupe = vec![vec![false; nb_jours]; n];
        for serie in self.series.values().filter(|s| s.planche_id == planche_id) {
            let occupation = serie.occupation();
            let premier = occupation.debut.max(debut_fenetre);
            let apres_dernier = occupation.fin.min(plus(fin_fenetre, 1));
            if premier >= apres_dernier {
                continue;
            }
            let jours =
                jour_index(debut_fenetre, premier)..jour_index(debut_fenetre, apres_dernier);
            let t1 = serie.troncon_debut.max(1) as usize - 1;
            let t2 = (serie.troncon_fin as usize).min(n);
            for ligne in occupe.iter_mut().take(t2).skip(t1) {
                ligne[jours.clone()].fill(true);
            }
        }

        let mut creneaux = Vec::new();
        for a in 0..n {
            let mut libre = vec![true; nb_jours];
            for b in a..n {
                for (j, l) in libre.iter_mut().enumerate() {
                    *l = *l && !occupe[b][j];
                }
                for (d0, d1) in plages_vraies(&libre) {
                    let duree = i64::try_from(d1 - d0 + 1).unwrap_or(0);
                    if duree <= DUREE_MIN_CRENEAU {
                        continue;
                    }
                    let libre_sur_la_periode = |t: usize| occupe[t][d0..=d1].iter().all(|o| !o);
                    let elargissable = (a > 0 && libre_sur_la_periode(a - 1))
                        || (b + 1 < n && libre_sur_la_periode(b + 1));
                    if elargissable {
                        continue;
                    }
                    creneaux.push(Creneau {
                        planche_id,
                        annee,
                        zone: Zone {
                            t1: u32::try_from(a + 1).unwrap_or(u32::MAX),
                            t2: u32::try_from(b + 1).unwrap_or(u32::MAX),
                        },
                        debut: plus(debut_fenetre, i64::try_from(d0).unwrap_or(0)),
                        fin: plus(debut_fenetre, i64::try_from(d1).unwrap_or(0)),
                    });
                }
            }
        }
        garder_les_non_contenus(creneaux)
    }
}

fn jour_index(debut: NaiveDate, jour: NaiveDate) -> usize {
    usize::try_from((jour - debut).num_days()).unwrap_or(0)
}

/// Maximal runs of `true`, as inclusive `(first, last)` indices.
fn plages_vraies(valeurs: &[bool]) -> Vec<(usize, usize)> {
    let mut plages = Vec::new();
    let mut debut = None;
    for (i, &v) in valeurs.iter().enumerate() {
        match (v, debut) {
            (true, None) => debut = Some(i),
            (false, Some(d)) => {
                plages.push((d, i - 1));
                debut = None;
            }
            _ => {}
        }
    }
    if let Some(d) = debut {
        plages.push((d, valeurs.len() - 1));
    }
    plages
}

/// Drops the slots contained, in zone and in period, in another slot (S03 §8.4).
fn garder_les_non_contenus(creneaux: Vec<Creneau>) -> Vec<Creneau> {
    let mut gardes: Vec<Creneau> = creneaux
        .iter()
        .filter(|c| {
            !creneaux.iter().any(|autre| {
                autre != *c
                    && autre.zone.contient(&c.zone)
                    && autre.debut <= c.debut
                    && c.fin <= autre.fin
            })
        })
        .copied()
        .collect();
    gardes.sort();
    gardes
}
