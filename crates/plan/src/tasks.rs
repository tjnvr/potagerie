//! Reminders, tasks and previous families (S03 §11).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use catalogue::Catalogue;
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::ids::{PlancheId, SerieId};
use crate::model::Plan;
use crate::serie::{Mode, Serie};

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TaskType {
    #[serde(rename = "semer-abri")]
    SowUnderCover,
    #[serde(rename = "repiquer")]
    Transplant,
    #[serde(rename = "planter")]
    Plant,
    #[serde(rename = "semer-terre")]
    SowInGround,
    #[serde(rename = "recolter")]
    Harvest,
}

impl TaskType {
    /// The identifier part of the task id, which never changes.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SowUnderCover => "semer-abri",
            Self::Transplant => "repiquer",
            Self::Plant => "planter",
            Self::SowInGround => "semer-terre",
            Self::Harvest => "recolter",
        }
    }
}

impl fmt::Display for TaskType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A task of a serie.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    /// `<serie id>/<type>`: stable when the date moves (usable as an iCal `UID`).
    pub id: String,
    pub serie_id: SerieId,
    pub task_type: TaskType,
    pub date: NaiveDate,
}

impl Serie {
    /// One to three tasks (S03 §11).
    pub fn tasks(&self, id: SerieId) -> Vec<Task> {
        let dates = self.dates();
        let task = |task_type: TaskType, date: NaiveDate| Task {
            id: format!("{id}/{task_type}"),
            serie_id: id,
            task_type,
            date,
        };
        let mut tasks = match self.mode {
            Mode::Abri => vec![
                task(TaskType::SowUnderCover, self.date),
                task(TaskType::Transplant, dates.planting_out),
            ],
            Mode::Plants => vec![task(TaskType::Plant, dates.planting_out)],
            Mode::PleineTerre => vec![task(TaskType::SowInGround, dates.planting_out)],
        };
        tasks.push(task(TaskType::Harvest, dates.harvest_start));
        tasks
    }
}

impl Plan {
    /// All the tasks of the plan, by date then id.
    pub fn tasks(&self) -> Vec<Task> {
        let mut tasks: Vec<Task> = self
            .series
            .iter()
            .flat_map(|(id, serie)| serie.tasks(*id))
            .collect();
        tasks.sort_by(|a, b| (a.date, &a.id).cmp(&(b.date, &b.id)));
        tasks
    }

    /// The tasks grouped by `(year, month)`, for the calendar export.
    pub fn calendar_by_month(&self) -> BTreeMap<(i32, u32), Vec<Task>> {
        let mut month: BTreeMap<(i32, u32), Vec<Task>> = BTreeMap::new();
        for task in self.tasks() {
            month
                .entry((task.date.year(), task.date.month()))
                .or_default()
                .push(task);
        }
        month
    }

    /// Families of the series of the planche whose occupation touches `year − 1`
    /// or `year − 2` (planche panel). Sorted, without duplicates; series of an
    /// unknown crop are ignored.
    pub fn previous_families(
        &self,
        planche_id: PlancheId,
        year: i32,
        catalogue: &Catalogue,
    ) -> Vec<String> {
        self.series
            .values()
            .filter(|s| s.planche_id == planche_id)
            .filter(|s| {
                let occupation = s.occupation();
                occupation.touches_year(year - 1) || occupation.touches_year(year - 2)
            })
            .filter_map(|s| catalogue.culture(&s.culture_id))
            .map(|c| c.famille.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}
