//! Reminders, tasks and previous families (S03 §11).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use catalogue::Catalogue;
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::ids::{PlancheId, SerieId};
use crate::model::Plan;
use crate::serie::{Mode, Serie};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TypeTache {
    #[serde(rename = "semer-abri")]
    SemerAbri,
    #[serde(rename = "repiquer")]
    Repiquer,
    #[serde(rename = "planter")]
    Planter,
    #[serde(rename = "semer-terre")]
    SemerTerre,
    #[serde(rename = "recolter")]
    Recolter,
}

impl TypeTache {
    /// The identifier part of the task id, which never changes.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SemerAbri => "semer-abri",
            Self::Repiquer => "repiquer",
            Self::Planter => "planter",
            Self::SemerTerre => "semer-terre",
            Self::Recolter => "recolter",
        }
    }
}

impl fmt::Display for TypeTache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A task of a serie.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tache {
    /// `<serie id>/<type>`: stable when the date moves (usable as an iCal `UID`).
    pub id: String,
    pub serie_id: SerieId,
    pub type_tache: TypeTache,
    pub date: NaiveDate,
}

impl Serie {
    /// One to three tasks (S03 §11).
    pub fn taches(&self, id: SerieId) -> Vec<Tache> {
        let dates = self.dates();
        let tache = |type_tache: TypeTache, date: NaiveDate| Tache {
            id: format!("{id}/{type_tache}"),
            serie_id: id,
            type_tache,
            date,
        };
        let mut taches = match self.mode {
            Mode::Abri => vec![
                tache(TypeTache::SemerAbri, self.date),
                tache(TypeTache::Repiquer, dates.mise_en_place),
            ],
            Mode::Plants => vec![tache(TypeTache::Planter, dates.mise_en_place)],
            Mode::PleineTerre => vec![tache(TypeTache::SemerTerre, dates.mise_en_place)],
        };
        taches.push(tache(TypeTache::Recolter, dates.recolte));
        taches
    }
}

impl Plan {
    /// All the tasks of the plan, by date then id.
    pub fn taches(&self) -> Vec<Tache> {
        let mut taches: Vec<Tache> = self
            .series
            .iter()
            .flat_map(|(id, serie)| serie.taches(*id))
            .collect();
        taches.sort_by(|a, b| (a.date, &a.id).cmp(&(b.date, &b.id)));
        taches
    }

    /// The tasks grouped by `(year, month)`, for the calendar export.
    pub fn calendrier_par_mois(&self) -> BTreeMap<(i32, u32), Vec<Tache>> {
        let mut mois: BTreeMap<(i32, u32), Vec<Tache>> = BTreeMap::new();
        for tache in self.taches() {
            mois.entry((tache.date.year(), tache.date.month()))
                .or_default()
                .push(tache);
        }
        mois
    }

    /// Families of the series of the planche whose occupation touches `annee − 1`
    /// or `annee − 2` (planche panel). Sorted, without duplicates; series of an
    /// unknown crop are ignored.
    pub fn familles_precedentes(
        &self,
        planche_id: PlancheId,
        annee: i32,
        catalogue: &Catalogue,
    ) -> Vec<String> {
        self.series
            .values()
            .filter(|s| s.planche_id == planche_id)
            .filter(|s| {
                let occupation = s.occupation();
                occupation.touche_annee(annee - 1) || occupation.touche_annee(annee - 2)
            })
            .filter_map(|s| catalogue.culture(&s.culture_id))
            .map(|c| c.famille.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}
