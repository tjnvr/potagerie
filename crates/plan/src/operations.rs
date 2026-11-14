//! Operations that change the plan. Each one checks everything first: an
//! operation that returns an error leaves the plan untouched.

use chrono::{Datelike, NaiveDate};

use crate::error::PlanError;
use crate::ids::{PlancheId, SerieId};
use crate::jours::premier_janvier;
use crate::model::{Disposition, Geometrie, MAX_PLANCHES, Plan, Planche};
use crate::serie::Serie;

impl Plan {
    /// Adds a planche, with its first layout recorded in `annee`.
    pub fn ajouter_planche(
        &mut self,
        id: PlancheId,
        nom: String,
        annee: i32,
        geometrie: Geometrie,
        aujourd_hui: NaiveDate,
    ) -> Result<(), PlanError> {
        if self.planches.contains_key(&id) {
            return Err(PlanError::PlancheExistante(id));
        }
        self.planches.insert(
            id,
            Planche {
                nom,
                dispositions: Default::default(),
            },
        );
        let resultat =
            self.enregistrer_disposition(id, annee, Disposition::Presente(geometrie), aujourd_hui);
        if resultat.is_err() {
            self.planches.remove(&id);
        }
        resultat
    }

    /// Records the layout of a planche in `annee`, replacing the one recorded
    /// for that year. It holds for the following years until the next layout
    /// recorded for this same planche (S03 §7).
    ///
    /// Series that no longer fit are not refused: ask
    /// [`Plan::series_hors_planche_si_raccourcie`] before and warn the user.
    pub fn enregistrer_disposition(
        &mut self,
        planche_id: PlancheId,
        annee: i32,
        disposition: Disposition,
        aujourd_hui: NaiveDate,
    ) -> Result<(), PlanError> {
        if annee < aujourd_hui.year() {
            return Err(PlanError::AnneePassee { annee, aujourd_hui });
        }
        if !self.planches.contains_key(&planche_id) {
            return Err(PlanError::PlancheInconnue(planche_id));
        }
        if let Disposition::Presente(geometrie) = &disposition {
            let case_cm = self.terrain.case_cm.cm();
            if geometrie.longueur_cm == 0 || geometrie.longueur_cm % case_cm != 0 {
                return Err(PlanError::LongueurInvalide {
                    longueur_cm: geometrie.longueur_cm,
                    case_cm,
                });
            }
        }

        let mut apres = self.clone();
        if let Some(planche) = apres.planches.get_mut(&planche_id) {
            planche.dispositions.insert(annee, disposition);
        }
        // Years from `annee` to the last recorded layout may gain a planche.
        let derniere = self
            .planches
            .values()
            .flat_map(|p| p.dispositions.keys().copied())
            .max()
            .unwrap_or(annee)
            .max(annee);
        for y in annee..=derniere {
            let existait = self.nb_planches_en(y, Some(planche_id)) != self.nb_planches_en(y, None);
            let existe = apres.nb_planches_en(y, Some(planche_id)) != apres.nb_planches_en(y, None);
            if existe && !existait && apres.nb_planches_en(y, None) > MAX_PLANCHES {
                return Err(PlanError::TropDePlanches {
                    annee: y,
                    max: MAX_PLANCHES,
                });
            }
        }
        *self = apres;
        Ok(())
    }

    /// Number of planches in `annee`, leaving one out if asked.
    fn nb_planches_en(&self, annee: i32, sauf: Option<PlancheId>) -> usize {
        self.planches
            .iter()
            .filter(|(id, _)| Some(**id) != sauf)
            .filter(|(_, p)| p.disposition_en(annee).is_some())
            .count()
    }

    /// The series deleted with the planche from `annee` on: those whose end `f`
    /// is after January 1st of `annee`, including those that began earlier
    /// (S03 §5). Series ending on or before that day are kept.
    pub fn series_supprimees_par(&self, planche_id: PlancheId, annee: i32) -> Vec<SerieId> {
        let limite = premier_janvier(annee);
        self.series
            .iter()
            .filter(|(_, s)| s.planche_id == planche_id && s.dates().fin > limite)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Deletes a planche from `annee` on, with its series that touch `annee` or
    /// later. Returns the deleted series. Layouts recorded after `annee` are dropped.
    pub fn supprimer_planche(
        &mut self,
        planche_id: PlancheId,
        annee: i32,
        aujourd_hui: NaiveDate,
    ) -> Result<Vec<SerieId>, PlanError> {
        if annee < aujourd_hui.year() {
            return Err(PlanError::AnneePassee { annee, aujourd_hui });
        }
        let planche = self
            .planches
            .get_mut(&planche_id)
            .ok_or(PlanError::PlancheInconnue(planche_id))?;
        planche.dispositions.retain(|y, _| *y < annee);
        planche.dispositions.insert(annee, Disposition::Absente);
        let supprimees = self.series_supprimees_par(planche_id, annee);
        for id in &supprimees {
            self.series.remove(id);
        }
        Ok(supprimees)
    }

    /// Adds a valid serie (S03 §5).
    pub fn ajouter_serie(&mut self, id: SerieId, serie: Serie) -> Result<(), PlanError> {
        if self.series.contains_key(&id) {
            return Err(PlanError::SerieExistante(id));
        }
        self.valider_serie(None, &serie)?;
        self.series.insert(id, serie);
        Ok(())
    }

    /// Replaces a serie by a modified one, if the result is valid.
    pub fn remplacer_serie(&mut self, id: SerieId, serie: Serie) -> Result<(), PlanError> {
        if !self.series.contains_key(&id) {
            return Err(PlanError::SerieInconnue(id));
        }
        self.valider_serie(Some(id), &serie)?;
        self.series.insert(id, serie);
        Ok(())
    }

    /// Drags a bar by `jours` days: the entered date and the manual end shift.
    /// Refused, with the plan untouched, if the moved serie is no longer valid
    /// (S03 §2).
    pub fn decaler_serie(&mut self, id: SerieId, jours: i64) -> Result<(), PlanError> {
        let decalee = self
            .series
            .get(&id)
            .ok_or(PlanError::SerieInconnue(id))?
            .decalee(jours);
        self.remplacer_serie(id, decalee)
    }

    /// Deletes a serie.
    pub fn supprimer_serie(&mut self, id: SerieId) -> Result<Serie, PlanError> {
        self.series.remove(&id).ok_or(PlanError::SerieInconnue(id))
    }
}
