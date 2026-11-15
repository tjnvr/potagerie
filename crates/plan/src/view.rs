//! Year view (S05 §5): what the commands return to the interface.
//!
//! It is computed from the document, the catalogue and today, and never
//! stored. It only assembles the calculations of the other modules (S03): it
//! adds no rule of its own.

use catalogue::Catalogue;
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::Serialize;

use crate::ids::{AnnotationId, BedId, PlantingId};
use crate::model::{Plan, TypeAnnotation, TypeBed};
use crate::planting::Mode;
use crate::stages::{Appearance, AppearanceChange};
use crate::tasks::TaskType;
use crate::zone::Occupation;

/// The year view.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct YearView {
    pub year: i32,
    pub today: NaiveDate,
    /// Past years are read-only (S03 §7).
    pub read_only: bool,
    /// `None` until the plot is created (E2): there is no plan yet.
    pub plot: Option<PlotView>,
    /// `version` of the plan, 0 when there is none. Exported as a JS number.
    pub version: u32,
    pub updated_at: Option<DateTime<Utc>>,
    /// The beds that exist in `year`, with their layout of that year.
    pub beds: Vec<BedView>,
    /// The plantings whose timeline (pots included) touches `year`.
    pub plantings: Vec<PlantingView>,
    /// Conflicting pairs among `plantings` (S03 §5).
    pub conflicts: Vec<ConflictView>,
    /// The plantings that ended on or before January 1st of `year`: the past
    /// seasons of the bed panel (S01 F-VIS-7), read-only.
    pub history: Vec<PlantingView>,
    /// Free slots of `year` and their suggestions (S03 §8, §9).
    pub slots: Vec<SlotView>,
    /// Annotations that touch `year`.
    pub annotations: Vec<AnnotationView>,
    /// Tasks dated in `year` (S03 §11).
    pub tasks: Vec<TaskView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PlotView {
    pub length_cm: u32,
    pub width_cm: u32,
    pub cell_cm: u32,
    pub aisle_cm: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct BedView {
    pub id: BedId,
    pub name: String,
    pub bed_type: TypeBed,
    pub x: u32,
    pub y: u32,
    pub length_cm: u32,
    pub width_cm: u32,
    /// `n = length ÷ case` (S03 §4).
    pub segments: u32,
    /// The plantings deleted with this bed if it is deleted from this year
    /// on (S03 §5): the interface warns before confirming.
    pub plantings_deleted: Vec<PlantingId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PlantingView {
    pub id: PlantingId,
    pub crop_id: String,
    pub bed_id: BedId,
    pub segment_start: u32,
    pub segment_end: u32,
    pub mode: Mode,
    /// Entered date: sowing (indoor, open ground) or planting (plants).
    pub date: NaiveDate,
    pub sowing: Option<NaiveDate>,
    /// `m`.
    pub planting_out: NaiveDate,
    /// `r`.
    pub harvest_start: NaiveDate,
    /// `f`.
    pub end: NaiveDate,
    /// True when `end` was modified by hand.
    pub manual_end: bool,
    /// Days in pots (mode indoor): from the catalogue, or entered for a crop
    /// without pots (S03 §2). The edit form shows it back.
    pub pot_days: u32,
    /// Rows × plants per row on the bed (S03 §6); 0 when the bed has
    /// no layout to measure.
    pub rows: u32,
    pub plants_by_row: u32,
    /// What the garden view draws today (S03 §3).
    pub appearance_today: Appearance,
    /// Every change of display over the life of the planting (S03 §3), so the
    /// garden view can show any date, and play the season, without
    /// calculating a stage itself.
    pub appearance_changes: Vec<AppearanceChange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ConflictView {
    pub a: PlantingId,
    pub b: PlantingId,
    pub segment_start: u32,
    pub segment_end: u32,
    pub start: NaiveDate,
    pub end: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SlotView {
    pub bed_id: BedId,
    pub segment_start: u32,
    pub segment_end: u32,
    /// First free day.
    pub start: NaiveDate,
    /// Last free day, included.
    pub end: NaiveDate,
    pub suggestions: Vec<SuggestionView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SuggestionView {
    pub crop_id: String,
    pub mode: Mode,
    pub date: NaiveDate,
    pub planting_out: NaiveDate,
    pub harvest_start: NaiveDate,
    pub end: NaiveDate,
    pub manual_end: Option<NaiveDate>,
    /// Filling score rounded to a tenth, in tenths (0 to 10).
    pub score_tenths: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct AnnotationView {
    pub id: AnnotationId,
    pub annotation_type: TypeAnnotation,
    pub date_start: NaiveDate,
    pub date_end: Option<NaiveDate>,
    pub bed_id: Option<BedId>,
    pub text: String,
    pub hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct TaskView {
    pub id: String,
    pub planting_id: PlantingId,
    pub task_type: TaskType,
    pub date: NaiveDate,
}

/// The view of one planting, with its quantities on the bed of the year it
/// starts in (S03 §6).
fn planting_view(
    plan: &Plan,
    id: PlantingId,
    s: &crate::planting::Planting,
    today: NaiveDate,
    catalogue: &Catalogue,
) -> PlantingView {
    let dates = s.dates();
    let start = dates
        .sowing
        .unwrap_or(dates.planting_out)
        .min(dates.planting_out);
    let width = plan
        .beds
        .get(&s.bed_id)
        .and_then(|p| p.layout_in(start.year()))
        .map(|g| g.width_cm);
    let (rows, plants_by_row) = match (width, catalogue.crop(&s.crop_id)) {
        (Some(width), Some(crop)) => {
            let q = crate::quantities::quantity(
                width,
                s.segment_end - s.segment_start + 1,
                plan.plot.cell_cm,
                crop,
            );
            (q.rows, q.plants_per_row)
        }
        _ => (0, 0),
    };
    PlantingView {
        id,
        crop_id: s.crop_id.clone(),
        bed_id: s.bed_id,
        segment_start: s.segment_start,
        segment_end: s.segment_end,
        mode: s.mode,
        date: s.date,
        sowing: dates.sowing,
        planting_out: dates.planting_out,
        harvest_start: dates.harvest_start,
        end: dates.end,
        manual_end: s.manual_end.is_some(),
        pot_days: s.indoor_days,
        rows,
        plants_by_row,
        appearance_today: s.appearance_on(today),
        appearance_changes: s.appearance_changes(),
    }
}

impl YearView {
    /// The view of `year`. `plan` is `None` before the plot is created.
    pub fn build(plan: Option<&Plan>, year: i32, today: NaiveDate, catalogue: &Catalogue) -> Self {
        let read_only = year < today.year();
        let Some(plan) = plan else {
            return Self {
                year,
                today,
                read_only,
                plot: None,
                version: 0,
                updated_at: None,
                beds: Vec::new(),
                plantings: Vec::new(),
                history: Vec::new(),
                conflicts: Vec::new(),
                slots: Vec::new(),
                annotations: Vec::new(),
                tasks: Vec::new(),
            };
        };

        let beds = plan
            .beds_in(year)
            .into_iter()
            .map(|(id, bed, geometry)| BedView {
                id,
                name: bed.name.clone(),
                bed_type: geometry.bed_type,
                x: geometry.x,
                y: geometry.y,
                length_cm: geometry.length_cm,
                width_cm: geometry.width_cm,
                segments: geometry.segments(plan.plot.cell_cm),
                plantings_deleted: plan.plantings_deleted_by(id, year),
            })
            .collect();

        let plantings: Vec<PlantingView> = plan
            .plantings
            .iter()
            .filter(|(_, s)| {
                let dates = s.dates();
                Occupation {
                    start: dates
                        .sowing
                        .unwrap_or(dates.planting_out)
                        .min(dates.planting_out),
                    end: dates.end,
                }
                .touches_year(year)
            })
            .map(|(id, s)| planting_view(plan, *id, s, today, catalogue))
            .collect();

        let january_first = crate::days::january_first(year);
        let history = plan
            .plantings
            .iter()
            .filter(|(_, s)| s.dates().end <= january_first)
            .map(|(id, s)| planting_view(plan, *id, s, today, catalogue))
            .collect();

        let shown = |id: &PlantingId| plantings.iter().any(|s| s.id == *id);
        let conflicts = plan
            .conflicts()
            .into_iter()
            .filter(|(a, b, _)| shown(a) || shown(b))
            .map(|(a, b, detail)| ConflictView {
                a,
                b,
                segment_start: detail.segments.t1,
                segment_end: detail.segments.t2,
                start: detail.period.start,
                end: detail.period.end,
            })
            .collect();

        let slots = plan
            .slots(year)
            .iter()
            .map(|slot| SlotView {
                bed_id: slot.bed_id,
                segment_start: slot.zone.t1,
                segment_end: slot.zone.t2,
                start: slot.start,
                end: slot.end,
                suggestions: plan
                    .suggestions(slot, catalogue, today)
                    .into_iter()
                    .map(|s| SuggestionView {
                        score_tenths: u32::try_from(s.score_tenths()).unwrap_or(0),
                        crop_id: s.crop_id,
                        mode: s.mode,
                        date: s.date,
                        planting_out: s.planting_out,
                        harvest_start: s.harvest_start,
                        end: s.end,
                        manual_end: s.manual_end,
                    })
                    .collect(),
            })
            .collect();

        let annotations = plan
            .annotations
            .iter()
            .filter(|(_, a)| {
                let end = a.date_end.unwrap_or(a.date_start);
                (a.date_start.year()..=end.year()).contains(&year)
            })
            .map(|(id, a)| AnnotationView {
                id: *id,
                annotation_type: a.type_annotation,
                date_start: a.date_start,
                date_end: a.date_end,
                bed_id: a.bed_id,
                text: a.text.clone(),
                hidden: a.hidden,
            })
            .collect();

        let tasks = plan
            .tasks()
            .into_iter()
            .filter(|t| t.date.year() == year)
            .map(|t| TaskView {
                id: t.id,
                planting_id: t.planting_id,
                task_type: t.task_type,
                date: t.date,
            })
            .collect();

        Self {
            year,
            today,
            read_only,
            plot: Some(PlotView {
                length_cm: plan.plot.length_cm,
                width_cm: plan.plot.width_cm,
                cell_cm: plan.plot.cell_cm.cm(),
                aisle_cm: plan.plot.aisle_cm,
            }),
            version: u32::try_from(plan.version).unwrap_or(u32::MAX),
            updated_at: Some(plan.updated_at),
            beds,
            plantings,
            history,
            conflicts,
            slots,
            annotations,
            tasks,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::ids::PlanId;
    use crate::model::{CellCm, Geometry, Plot};
    use crate::planting::Planting;
    use crate::zone::Zone;

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn plan_with_one_bed() -> (Plan, BedId) {
        let now = Utc.with_ymd_and_hms(2027, 1, 10, 8, 0, 0).unwrap();
        let mut plan = Plan::new(
            PlanId::generate(),
            Plot {
                length_cm: 1100,
                width_cm: 650,
                cell_cm: CellCm::C50,
                aisle_cm: 50,
            },
            now,
        );
        let id = BedId::generate();
        plan.add_bed(
            id,
            "Bed A".into(),
            2027,
            Geometry {
                bed_type: TypeBed::Bed,
                x: 50,
                y: 50,
                length_cm: 300,
                width_cm: 120,
            },
            day(2027, 1, 10),
        )
        .unwrap();
        (plan, id)
    }

    #[test]
    fn no_plan_gives_an_empty_view() {
        let view = YearView::build(None, 2027, day(2027, 6, 20), Catalogue::reference());
        assert_eq!(view.plot, None);
        assert!(view.beds.is_empty());
        assert!(!view.read_only);
        assert_eq!(view.version, 0);
    }

    #[test]
    fn past_years_are_read_only() {
        let view = YearView::build(None, 2026, day(2027, 6, 20), Catalogue::reference());
        assert!(view.read_only);
    }

    #[test]
    fn beds_follow_their_layouts() {
        let (plan, id) = plan_with_one_bed();
        let today = day(2027, 1, 10);
        let view = YearView::build(Some(&plan), 2027, today, Catalogue::reference());
        assert_eq!(view.beds.len(), 1);
        assert_eq!(view.beds[0].id, id);
        assert_eq!(view.beds[0].segments, 6);
        let before = YearView::build(Some(&plan), 2026, today, Catalogue::reference());
        assert!(before.beds.is_empty());
        let after = YearView::build(Some(&plan), 2028, today, Catalogue::reference());
        assert_eq!(after.beds.len(), 1);
    }

    #[test]
    fn plantings_carry_their_calculated_dates() {
        let (mut plan, bed_id) = plan_with_one_bed();
        let tomato = Catalogue::reference().crop("tomato").unwrap();
        let planting = Planting::new(
            tomato,
            bed_id,
            Zone::new(1, 3).unwrap(),
            Mode::Indoor,
            day(2027, 3, 10),
            None,
            None,
        )
        .unwrap();
        let expected = planting.dates();
        plan.add_planting(PlantingId::generate(), planting).unwrap();

        let view = YearView::build(Some(&plan), 2027, day(2027, 6, 20), Catalogue::reference());
        assert_eq!(view.plantings.len(), 1);
        assert_eq!(view.plantings[0].planting_out, expected.planting_out);
        assert_eq!(view.plantings[0].end, expected.end);
        assert_eq!(view.tasks.len(), 3);
        assert!(view.conflicts.is_empty());
        let other_year =
            YearView::build(Some(&plan), 2029, day(2027, 6, 20), Catalogue::reference());
        assert!(other_year.plantings.is_empty());
    }

    #[test]
    fn deletion_warning_quantities_and_history() {
        let (mut plan, bed_id) = plan_with_one_bed();
        let carrot = Catalogue::reference().crop("carrot").unwrap();
        let planting = |date| {
            Planting::new(
                carrot,
                bed_id,
                Zone::new(1, 2).unwrap(),
                Mode::OpenGround,
                date,
                None,
                None,
            )
            .unwrap()
        };
        let in_2027 = PlantingId::generate();
        plan.add_planting(in_2027, planting(day(2027, 3, 15)))
            .unwrap();
        let view = YearView::build(Some(&plan), 2027, day(2027, 1, 10), Catalogue::reference());
        assert_eq!(view.beds[0].plantings_deleted, vec![in_2027]);
        // 2 segments of 50 cm on 1.2 m: 4 rows (30 cm) of 100 / 4 cm plants (S03 T4).
        assert_eq!(
            (view.plantings[0].rows, view.plantings[0].plants_by_row),
            (4, 25)
        );
        assert!(view.history.is_empty());
        // The 2027 planting is history in 2028 and is no longer deleted by a 2028 deletion.
        let next = YearView::build(Some(&plan), 2028, day(2027, 1, 10), Catalogue::reference());
        assert_eq!(next.history.len(), 1);
        assert!(next.beds[0].plantings_deleted.is_empty());
    }
}
