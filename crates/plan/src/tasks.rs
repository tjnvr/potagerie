//! Reminders, tasks and previous families (S03 §11).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use catalogue::Catalogue;
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::ids::{BedId, PlantingId};
use crate::model::Plan;
use crate::planting::{Mode, Planting};

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TaskType {
    #[serde(rename = "sow-indoor")]
    SowUnderCover,
    #[serde(rename = "transplant")]
    Transplant,
    #[serde(rename = "plant")]
    Plant,
    #[serde(rename = "sow-ground")]
    SowInGround,
    #[serde(rename = "harvest")]
    Harvest,
}

impl TaskType {
    /// The identifier part of the task id, which never changes.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SowUnderCover => "sow-indoor",
            Self::Transplant => "transplant",
            Self::Plant => "plant",
            Self::SowInGround => "sow-ground",
            Self::Harvest => "harvest",
        }
    }
}

impl fmt::Display for TaskType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A task of a planting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    /// `<planting id>/<type>`: stable when the date moves (usable as an iCal `UID`).
    pub id: String,
    pub planting_id: PlantingId,
    pub task_type: TaskType,
    pub date: NaiveDate,
}

impl Planting {
    /// One to three tasks (S03 §11).
    pub fn tasks(&self, id: PlantingId) -> Vec<Task> {
        let dates = self.dates();
        let task = |task_type: TaskType, date: NaiveDate| Task {
            id: format!("{id}/{task_type}"),
            planting_id: id,
            task_type,
            date,
        };
        let mut tasks = match self.mode {
            Mode::Indoor => vec![
                task(TaskType::SowUnderCover, self.date),
                task(TaskType::Transplant, dates.planting_out),
            ],
            Mode::Plants => vec![task(TaskType::Plant, dates.planting_out)],
            Mode::OpenGround => vec![task(TaskType::SowInGround, dates.planting_out)],
        };
        if self.perennial.is_none() {
            tasks.push(task(TaskType::Harvest, dates.harvest_start));
            return tasks;
        }
        // One harvest a year: `harvest`, then `harvest-2`, `harvest-3`… (S03 §11).
        for (i, harvest) in self.harvests().iter().enumerate() {
            let mut t = task(TaskType::Harvest, harvest.start);
            if i > 0 {
                t.id = format!("{}-{}", t.id, i + 1);
            }
            tasks.push(t);
        }
        tasks
    }
}

impl Plan {
    /// All the tasks of the plan, by date then id.
    pub fn tasks(&self) -> Vec<Task> {
        let mut tasks: Vec<Task> = self
            .plantings
            .iter()
            .flat_map(|(id, planting)| planting.tasks(*id))
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

    /// Families of the plantings of the bed whose occupation touches `year − 1`
    /// or `year − 2` (bed panel). Sorted, without duplicates; plantings of an
    /// unknown crop are ignored.
    pub fn previous_families(
        &self,
        bed_id: BedId,
        year: i32,
        catalogue: &Catalogue,
    ) -> Vec<String> {
        self.plantings
            .values()
            .filter(|s| s.bed_id == bed_id)
            .filter(|s| {
                let occupation = s.occupation();
                occupation.touches_year(year - 1) || occupation.touches_year(year - 2)
            })
            .filter_map(|s| catalogue.crop(&s.crop_id))
            .map(|c| c.family.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}
