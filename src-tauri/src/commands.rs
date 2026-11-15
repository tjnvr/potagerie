//! Tauri commands (S05 §7). Each command that changes the plan loads it,
//! applies one operation of the `plan` crate, saves it (`version + 1`) and
//! returns the year view. A refused command changes nothing.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use catalogue::Catalogue;
use chrono::{DateTime, Local, NaiveDate, Utc};
use export::TimelineFormat;
use plan::{
    Annotation, AnnotationId, BedId, CellCm, Geometry, Incoherence, Layout, Mode, Plan, PlanId,
    Planting, PlantingId, Plot, TypeAnnotation, TypeBed, YearView, Zone,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use store::{SqliteStore, Store};
use tauri::State;

use crate::error::{AppError, ErrorCode};

/// Shared state: the store of the device.
pub struct AppState {
    store: Mutex<SqliteStore>,
}

impl AppState {
    pub fn open(db_path: &Path) -> Result<Self, AppError> {
        Ok(Self {
            store: Mutex::new(SqliteStore::open(db_path)?),
        })
    }

    fn store(&self) -> MutexGuard<'_, SqliteStore> {
        // A panic while holding the lock cannot leave a half-written plan:
        // every write is one SQL statement.
        self.store.lock().unwrap_or_else(|e| e.into_inner())
    }
}

fn today() -> NaiveDate {
    Local::now().date_naive()
}

fn view(plan: Option<&Plan>, year: i32) -> YearView {
    YearView::build(plan, year, today(), Catalogue::v1())
}

/// Loads the plan, applies `change`, saves it and returns the view of `year`.
fn change(
    state: &AppState,
    year: i32,
    change: impl FnOnce(&mut Plan, NaiveDate) -> Result<(), AppError>,
) -> Result<YearView, AppError> {
    let mut store = state.store();
    let mut plan = store
        .load()?
        .ok_or_else(|| AppError::new(ErrorCode::PlotAbsent, "create the plot first"))?;
    let today = today();
    change(&mut plan, today)?;
    store.save(&mut plan, Utc::now())?;
    Ok(YearView::build(Some(&plan), year, today, Catalogue::v1()))
}

// ---------- Inputs and outputs ----------

/// A crop of the catalogue (S02). The name is not sent: the interface
/// translates `crop_<id>`.
#[derive(Debug, Clone, Serialize, Type)]
pub struct CropView {
    pub id: String,
    pub family: String,
    pub indoor_days: u32,
    pub harvest_after_days: u32,
    pub harvest_duration_days: u32,
    /// `MM-JJ`.
    pub planting_out_start: String,
    /// `MM-JJ`.
    pub planting_out_end: String,
    pub row_cm: u32,
    pub spacing_cm: u32,
}

#[derive(Debug, Clone, Deserialize, Type)]
pub struct PlotInput {
    pub length_cm: u32,
    pub width_cm: u32,
    /// 25, 50 or 100; fixed once the plot is created.
    pub cell_cm: u32,
    pub aisle_cm: u32,
}

#[derive(Debug, Clone, Deserialize, Type)]
pub struct PlotEdit {
    pub length_cm: u32,
    pub width_cm: u32,
    pub aisle_cm: u32,
}

#[derive(Debug, Clone, Deserialize, Type)]
pub struct GeometryInput {
    pub bed_type: TypeBed,
    pub x: u32,
    pub y: u32,
    pub length_cm: u32,
    pub width_cm: u32,
}

impl From<GeometryInput> for Geometry {
    fn from(g: GeometryInput) -> Self {
        Self {
            bed_type: g.bed_type,
            x: g.x,
            y: g.y,
            length_cm: g.length_cm,
            width_cm: g.width_cm,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Type)]
pub struct PlantingInput {
    pub crop_id: String,
    pub bed_id: BedId,
    pub segment_start: u32,
    pub segment_end: u32,
    pub mode: Mode,
    /// Sowing (indoor, open ground) or planting (plants) date.
    pub date: NaiveDate,
    /// Days in pots, only for the mode indoor of a crop without pots (S03 §2).
    pub indoor_days: Option<u32>,
    pub manual_end: Option<NaiveDate>,
    /// Mode indoor only: `date` is the transplanting date, not the sowing date.
    /// The planting stores the sowing date `s = x - A` (S03 §2).
    #[serde(default)]
    pub date_est_transplanting: bool,
}

/// A zone of segments.
#[derive(Debug, Clone, Copy, Serialize, Type)]
pub struct ZoneView {
    pub segment_start: u32,
    pub segment_end: u32,
}

/// The zones of the shortcuts of F-PLA-1; `None` greys the shortcut out
/// (S03 §4).
#[derive(Debug, Clone, Serialize, Type)]
pub struct ShortcutsView {
    pub all: Option<ZoneView>,
    pub half: Option<ZoneView>,
    pub third: Option<ZoneView>,
    pub quarter: Option<ZoneView>,
}

/// Why a planting is not valid (S03 §5): a code to translate, and what the
/// interface needs to say where and when.
#[derive(Debug, Clone, Serialize, Type)]
pub struct ProblemView {
    pub code: ErrorCode,
    /// The other planting of a conflict.
    pub with: Option<PlantingId>,
    /// Segments in common (conflict) or the zone that does not fit.
    pub zone: Option<ZoneView>,
    /// Period in common (conflict), `[start, end[`.
    pub start: Option<NaiveDate>,
    pub end: Option<NaiveDate>,
    /// Year of a missing bed or of a zone that does not fit.
    pub year: Option<i32>,
    /// Segments of the bed that year (zone that does not fit).
    pub segments: Option<u32>,
}

/// The dates and quantities of a planting being entered (S03 §2, §6).
#[derive(Debug, Clone, Serialize, Type)]
pub struct DatesPreview {
    /// The date Rust would store (the sowing date, even if a transplanting
    /// date was entered).
    pub date: NaiveDate,
    pub sowing: Option<NaiveDate>,
    pub planting_out: NaiveDate,
    pub harvest_start: NaiveDate,
    pub end: NaiveDate,
    /// `end` without the manual end: « Recompute ».
    pub default_end: NaiveDate,
    pub manual_end: bool,
    pub pot_days: u32,
    pub rows: u32,
    pub plants_by_row: u32,
}

/// What the form of E07 shows while the user types: computed by Rust, never by
/// the interface.
#[derive(Debug, Clone, Serialize, Type)]
pub struct PlantingPreview {
    /// `None` when the planting cannot be built (a required field is missing).
    pub dates: Option<DatesPreview>,
    /// Empty when the planting is valid.
    pub problems: Vec<ProblemView>,
    pub shortcuts: ShortcutsView,
    /// The zones of other plantings of the bed that overlap the occupation in
    /// time: the segments the strip of the form shows as taken.
    pub occupied: Vec<ZoneView>,
}

fn zone_view(zone: Zone) -> ZoneView {
    ZoneView {
        segment_start: zone.t1,
        segment_end: zone.t2,
    }
}

fn problem(error: plan::PlanError) -> ProblemView {
    let mut p = ProblemView {
        code: AppError::from(error.clone()).code,
        with: None,
        zone: None,
        start: None,
        end: None,
        year: None,
        segments: None,
    };
    match error {
        plan::PlanError::Conflict {
            with,
            segments,
            period,
        } => {
            p.with = Some(with);
            p.zone = Some(zone_view(segments));
            p.start = Some(period.start);
            p.end = Some(period.end);
        }
        plan::PlanError::ZoneOutOfBed {
            year,
            segments,
            zone,
        } => {
            p.year = Some(year);
            p.segments = Some(segments);
            p.zone = Some(zone_view(zone));
        }
        plan::PlanError::BedMissing { year, .. } => p.year = Some(year),
        _ => {}
    }
    p
}

#[derive(Debug, Clone, Deserialize, Type)]
pub struct AnnotationInput {
    pub annotation_type: TypeAnnotation,
    pub date_start: NaiveDate,
    pub date_end: Option<NaiveDate>,
    pub bed_id: Option<BedId>,
    pub text: String,
    pub hidden: bool,
}

impl From<AnnotationInput> for Annotation {
    fn from(a: AnnotationInput) -> Self {
        Self {
            type_annotation: a.annotation_type,
            date_start: a.date_start,
            date_end: a.date_end,
            bed_id: a.bed_id,
            text: a.text,
            hidden: a.hidden,
        }
    }
}

/// What an import file holds, before it replaces anything (F-CPT-3).
#[derive(Debug, Clone, Serialize, Type)]
pub struct ImportSummaryView {
    pub updated_at: DateTime<Utc>,
    pub version: u32,
    pub bed_count: u32,
    pub planting_count: u32,
    /// The file is older than the plan of the device: warn before replacing.
    pub older: bool,
    /// The device has a plan that the import would replace.
    pub replaces: bool,
    /// What breaks the rules of the plan: kept, to resolve (S03 §13).
    pub incoherences: Vec<IncoherenceView>,
}

/// One incoherence of an imported plan. `code` is `too_many_beds`,
/// `planting_orphan`, `overlap`, `zone_outside_bed`, `crop_unknown`,
/// `note_bed_unknown`, `note_invalid`, or a placement code
/// (`bed_outside_plot`, `bed_overlaps`, `aisle_too_narrow`,
/// `width_invalid`). Names come from the file, not from the interface.
#[derive(Debug, Clone, Serialize, Type)]
pub struct IncoherenceView {
    pub code: String,
    pub year: Option<i32>,
    pub bed_name: Option<String>,
    pub other_bed_name: Option<String>,
    pub crop_id: Option<String>,
    pub other_crop_id: Option<String>,
    /// Beds in a year, or segments of the bed.
    pub count: Option<u32>,
}

fn incoherence_view(plan: &Plan, incoherence: &Incoherence) -> IncoherenceView {
    let name = |id: &BedId| plan.beds.get(id).map(|p| p.name.clone());
    let crop = |id: &PlantingId| plan.plantings.get(id).map(|s| s.crop_id.clone());
    let mut v = IncoherenceView {
        code: incoherence.code().to_owned(),
        year: None,
        bed_name: None,
        other_bed_name: None,
        crop_id: None,
        other_crop_id: None,
        count: None,
    };
    match incoherence {
        Incoherence::TooManyBeds { year, count } => {
            v.year = Some(*year);
            v.count = u32::try_from(*count).ok();
        }
        Incoherence::OrphanPlanting {
            planting_id,
            bed_id,
            year,
        } => {
            v.year = *year;
            v.bed_name = name(bed_id);
            v.crop_id = crop(planting_id);
        }
        Incoherence::ZoneOutOfBed {
            planting_id,
            bed_id,
            year,
            segments,
            ..
        } => {
            v.year = Some(*year);
            v.count = Some(*segments);
            v.bed_name = name(bed_id);
            v.crop_id = crop(planting_id);
        }
        Incoherence::Conflict { a, b } => {
            v.crop_id = crop(a);
            v.other_crop_id = crop(b);
            v.bed_name = plan.plantings.get(a).and_then(|s| name(&s.bed_id));
        }
        Incoherence::Placement(error) => match error {
            plan::PlanError::OutsidePlot { bed_id, year } => {
                v.year = Some(*year);
                v.bed_name = name(bed_id);
            }
            plan::PlanError::BedsOverlap { a, b, year }
            | plan::PlanError::AisleTooNarrow { a, b, year, .. } => {
                v.year = Some(*year);
                v.bed_name = name(a);
                v.other_bed_name = name(b);
            }
            _ => {}
        },
        Incoherence::UnknownCrop {
            planting_id,
            crop_id,
        } => {
            v.crop_id = Some(crop_id.clone());
            v.bed_name = plan
                .plantings
                .get(planting_id)
                .and_then(|s| name(&s.bed_id));
        }
        Incoherence::AnnotationUnknownBed { .. } | Incoherence::AnnotationInvalid { .. } => {}
    }
    v
}

#[derive(Debug, Clone, Copy, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum TimelineExportFormat {
    Png,
    Pdf,
}

/// Size of a new bed dropped from the tray (S01 F-PLN-1): length × width.
fn default_size(bed_type: TypeBed) -> (u32, u32) {
    match bed_type {
        TypeBed::Bed => (300, 120),
        TypeBed::Square => (100, 100),
        TypeBed::Mound => (300, 120),
        TypeBed::Planter => (200, 120),
    }
}

// ---------- Commands ----------

/// The crops of S02.
#[tauri::command]
#[specta::specta]
pub fn catalogue() -> Vec<CropView> {
    Catalogue::v1()
        .iter()
        .map(|c| CropView {
            id: c.id.clone(),
            family: c.family.clone(),
            indoor_days: c.indoor_days,
            harvest_after_days: c.harvest_after_days,
            harvest_duration_days: c.harvest_duration_days,
            planting_out_start: c.planting_out.start.to_string(),
            planting_out_end: c.planting_out.end.to_string(),
            row_cm: c.row_cm,
            spacing_cm: c.spacing_cm,
        })
        .collect()
}

/// Opens a year (tabs Garden and Plan).
#[tauri::command]
#[specta::specta]
pub fn view_year(state: State<'_, AppState>, year: i32) -> Result<YearView, AppError> {
    let plan = state.store().load()?;
    Ok(view(plan.as_ref(), year))
}

/// Creates the plot, and with it the plan (E2, F-TER-1).
#[tauri::command]
#[specta::specta]
pub fn create_plot(
    state: State<'_, AppState>,
    year: i32,
    plot: PlotInput,
) -> Result<YearView, AppError> {
    let mut store = state.store();
    if store.load()?.is_some() {
        return Err(AppError::new(
            ErrorCode::PlotExisting,
            "the plot already exists",
        ));
    }
    let cell_cm =
        CellCm::try_from(plot.cell_cm).map_err(|e| AppError::new(ErrorCode::CaseInvalid, e))?;
    let plot = Plot {
        length_cm: plot.length_cm,
        width_cm: plot.width_cm,
        cell_cm,
        aisle_cm: plot.aisle_cm,
    };
    plot.validate()?;
    let mut plan = Plan::new(PlanId::generate(), plot, Utc::now());
    store.save(&mut plan, Utc::now())?;
    Ok(view(Some(&plan), year))
}

/// Changes the dimensions of the plot and the allée width. The case size
/// is fixed at creation.
#[tauri::command]
#[specta::specta]
pub fn modify_plot(
    state: State<'_, AppState>,
    year: i32,
    plot: PlotEdit,
) -> Result<YearView, AppError> {
    change(&state, year, |plan, today| {
        plan.set_plot(plot.length_cm, plot.width_cm, plot.aisle_cm, today)?;
        Ok(())
    })
}

/// Drops a new bed from the tray at `(x, y)`, at its default size (F-PLN-6).
/// The default name ("Bed A") is chosen and translated by the interface.
#[tauri::command]
#[specta::specta]
pub fn create_bed(
    state: State<'_, AppState>,
    year: i32,
    bed_type: TypeBed,
    name: String,
    x: u32,
    y: u32,
) -> Result<YearView, AppError> {
    change(&state, year, |plan, today| {
        let (length_cm, width_cm) = default_size(bed_type);
        plan::check_name(&name)?;
        plan.add_bed(
            BedId::generate(),
            name,
            year,
            Geometry {
                bed_type,
                x,
                y,
                length_cm,
                width_cm,
            },
            today,
        )?;
        plan.check_placements(year)?;
        Ok(())
    })
}

/// Renames a bed and/or changes its layout from `year` on (S03 §7).
#[tauri::command]
#[specta::specta]
pub fn modify_bed(
    state: State<'_, AppState>,
    year: i32,
    id: BedId,
    name: Option<String>,
    geometry: Option<GeometryInput>,
) -> Result<YearView, AppError> {
    change(&state, year, |plan, today| {
        if let Some(geometry) = geometry {
            plan.set_layout(id, year, Layout::Present(geometry.into()), today)?;
            plan.check_placements(year)?;
        }
        if let Some(name) = name {
            plan::check_name(&name)?;
            plan.beds
                .get_mut(&id)
                .ok_or(plan::PlanError::UnknownBed(id))?
                .name = name;
        }
        Ok(())
    })
}

/// Moves a bed from `year` on, keeping its size.
#[tauri::command]
#[specta::specta]
pub fn move_bed(
    state: State<'_, AppState>,
    year: i32,
    id: BedId,
    x: u32,
    y: u32,
) -> Result<YearView, AppError> {
    change(&state, year, |plan, today| {
        let mut geometry = plan
            .beds
            .get(&id)
            .ok_or(plan::PlanError::UnknownBed(id))?
            .layout_in(year)
            .ok_or(plan::PlanError::BedMissing { bed_id: id, year })?
            .clone();
        geometry.x = x;
        geometry.y = y;
        plan.set_layout(id, year, Layout::Present(geometry), today)?;
        plan.check_placements(year)?;
        Ok(())
    })
}

/// The plantings that would no longer fit if the bed were shortened to
/// `length_cm` from `year` on (S03 §4). Read-only: the interface warns
/// before it calls `modify_bed`.
#[tauri::command]
#[specta::specta]
pub fn plantings_exceeding(
    state: State<'_, AppState>,
    year: i32,
    id: BedId,
    length_cm: u32,
) -> Result<Vec<PlantingId>, AppError> {
    let plan = state
        .store()
        .load()?
        .ok_or_else(|| AppError::new(ErrorCode::PlotAbsent, "create the plot first"))?;
    let cell_cm = plan.plot.cell_cm.cm();
    Ok(plan.plantings_overflowing_if_shortened(id, year, length_cm / cell_cm))
}

/// Deletes a bed from `year` on, with its plantings that touch `year` or
/// later (F-PLN-4). The interface confirms first, with `plantings_deleted_by`.
#[tauri::command]
#[specta::specta]
pub fn delete_bed(state: State<'_, AppState>, year: i32, id: BedId) -> Result<YearView, AppError> {
    change(&state, year, |plan, today| {
        plan.delete_bed(id, year, today)?;
        Ok(())
    })
}

/// The planting described by the form, ready to be validated (S03 §2).
///
/// Without `existing`, the durations are copied from the catalogue; with it,
/// they are copied again only if the crop or the mode changes.
fn build_planting(
    existing: Option<&Planting>,
    input: &PlantingInput,
) -> Result<Planting, plan::PlanError> {
    let crop = Catalogue::v1()
        .crop(&input.crop_id)
        .ok_or_else(|| plan::PlanError::UnknownCrop(input.crop_id.clone()))?;
    let zone = Zone::new(input.segment_start, input.segment_end)?;
    let date = if input.date_est_transplanting && input.mode == Mode::Indoor {
        let pot_days = if crop.indoor_days > 0 {
            crop.indoor_days
        } else {
            input.indoor_days.unwrap_or(0)
        };
        if pot_days == 0 {
            return Err(plan::PlanError::PotDaysRequired);
        }
        Planting::sowing_from_transplanting(input.date, pot_days)
    } else {
        input.date
    };
    let Some(old) = existing else {
        return Planting::new(
            crop,
            input.bed_id,
            zone,
            input.mode,
            date,
            input.indoor_days,
            input.manual_end,
        );
    };
    // The entered fields first: the checks of a change of crop or mode must
    // see the new date and end, not the old manual end.
    let mut edited = old.clone();
    edited.bed_id = input.bed_id;
    edited.segment_start = zone.t1;
    edited.segment_end = zone.t2;
    edited.date = date;
    edited.manual_end = input.manual_end;
    if edited.crop_id != input.crop_id {
        edited.change_crop(crop, input.indoor_days)?;
    }
    if edited.mode != input.mode {
        edited.change_mode(input.mode, crop, input.indoor_days)?;
    } else if let (Mode::Indoor, 0, Some(days)) = (input.mode, crop.indoor_days, input.indoor_days)
    {
        edited.indoor_days = days;
    }
    edited.check()?;
    Ok(edited)
}

/// Adds a planting; the durations are copied from the catalogue (S03 §2, §5).
/// Refused, with the plan untouched, on a conflict, a zone outside the
/// bed or a bed missing in a year the planting occupies.
#[tauri::command]
#[specta::specta]
pub fn create_planting(
    state: State<'_, AppState>,
    year: i32,
    planting: PlantingInput,
) -> Result<YearView, AppError> {
    change(&state, year, |plan, _| {
        plan.add_planting(PlantingId::generate(), build_planting(None, &planting)?)?;
        Ok(())
    })
}

/// Changes a planting. Durations are copied again only if the crop or the mode
/// changes (S03 §2).
#[tauri::command]
#[specta::specta]
pub fn modify_planting(
    state: State<'_, AppState>,
    year: i32,
    id: PlantingId,
    planting: PlantingInput,
) -> Result<YearView, AppError> {
    change(&state, year, |plan, _| {
        let old = plan
            .plantings
            .get(&id)
            .ok_or(plan::PlanError::UnknownPlanting(id))?;
        let edited = build_planting(Some(old), &planting)?;
        plan.replace_planting(id, edited)?;
        Ok(())
    })
}

/// Drags a bar by `days` days (S03 §2, T12): the entered date and the manual
/// end shift. Refused with `conflict` (or another code) if the shifted planting
/// is not valid; nothing changes and the interface brings the bar back.
#[tauri::command]
#[specta::specta]
pub fn shift_planting(
    state: State<'_, AppState>,
    year: i32,
    id: PlantingId,
    days: i32,
) -> Result<YearView, AppError> {
    change(&state, year, |plan, _| {
        plan.shift_planting(id, i64::from(days))?;
        Ok(())
    })
}

#[tauri::command]
#[specta::specta]
pub fn delete_planting(
    state: State<'_, AppState>,
    year: i32,
    id: PlantingId,
) -> Result<YearView, AppError> {
    change(&state, year, |plan, _| {
        plan.delete_planting(id)?;
        Ok(())
    })
}

/// Checks a planting as the form of E07 fills in, without saving: its dates and
/// quantities, why it is not valid, and the zones of the shortcuts. `id` is
/// the planting being edited, which is not compared with its previous self.
#[tauri::command]
#[specta::specta]
pub fn preview_planting(
    state: State<'_, AppState>,
    id: Option<PlantingId>,
    planting: PlantingInput,
) -> Result<PlantingPreview, AppError> {
    let plan = state
        .store()
        .load()?
        .ok_or_else(|| AppError::new(ErrorCode::PlotAbsent, "create the plot first"))?;
    Ok(build_planting_preview(&plan, id, &planting))
}

fn build_planting_preview(
    plan: &Plan,
    id: Option<PlantingId>,
    input: &PlantingInput,
) -> PlantingPreview {
    let catalogue = Catalogue::v1();
    let existing = id.and_then(|id| plan.plantings.get(&id));
    let (built, problems) = match build_planting(existing, input) {
        Ok(planting) => {
            let problems = plan
                .planting_problems(id, &planting)
                .into_iter()
                .map(problem)
                .collect::<Vec<_>>();
            (Some(planting), problems)
        }
        Err(error) => (None, vec![problem(error)]),
    };

    // The shortcuts need the occupation, not the zone: probe with one segment
    // when the entered zone is the problem.
    let probe = built.clone().or_else(|| {
        let mut probe = input.clone();
        (probe.segment_start, probe.segment_end) = (1, 1);
        build_planting(existing, &probe).ok()
    });
    let shortcut = |fraction| {
        let planting = probe.as_ref()?;
        plan.shortcut(planting.bed_id, fraction, planting.occupation(), id)
            .map(zone_view)
    };
    let shortcuts = ShortcutsView {
        all: shortcut(plan::Fraction::Whole),
        half: shortcut(plan::Fraction::Half),
        third: shortcut(plan::Fraction::Third),
        quarter: shortcut(plan::Fraction::Quarter),
    };

    let occupied = probe
        .as_ref()
        .map(|planting| {
            let occupation = planting.occupation();
            plan.plantings
                .iter()
                .filter(|(other, s)| {
                    Some(**other) != id
                        && s.bed_id == planting.bed_id
                        && s.occupation().overlaps(&occupation)
                })
                .map(|(_, s)| zone_view(s.zone()))
                .collect()
        })
        .unwrap_or_default();

    let dates = built.map(|planting| {
        let d = planting.dates();
        let start = d.sowing.unwrap_or(d.planting_out).min(d.planting_out);
        let default_end = Planting {
            manual_end: None,
            ..planting.clone()
        }
        .dates()
        .end;
        let (rows, plants_by_row) = match (
            plan.beds
                .get(&planting.bed_id)
                .and_then(|p| p.layout_in(chrono::Datelike::year(&start))),
            catalogue.crop(&planting.crop_id),
        ) {
            (Some(g), Some(crop)) => {
                let q = plan::quantity(
                    g.width_cm,
                    planting.segment_end - planting.segment_start + 1,
                    plan.plot.cell_cm,
                    crop,
                );
                (q.rows, q.plants_per_row)
            }
            _ => (0, 0),
        };
        DatesPreview {
            date: planting.date,
            sowing: d.sowing,
            planting_out: d.planting_out,
            harvest_start: d.harvest_start,
            end: d.end,
            default_end,
            manual_end: planting.manual_end.is_some(),
            pot_days: planting.indoor_days,
            rows,
            plants_by_row,
        }
    });
    PlantingPreview {
        dates,
        problems,
        shortcuts,
        occupied,
    }
}

/// The crops whose planting-out window contains `date` (S03 §10, F-CAT-3
/// « of season now »).
#[tauri::command]
#[specta::specta]
pub fn crops_of_season(date: NaiveDate) -> Vec<String> {
    Catalogue::v1()
        .iter()
        .filter(|c| plan::in_window(c, date))
        .map(|c| c.id.clone())
        .collect()
}

/// Adds a note or a period (S01 F-VIS-5); the plan crate checks the rules.
#[tauri::command]
#[specta::specta]
pub fn create_annotation(
    state: State<'_, AppState>,
    year: i32,
    annotation: AnnotationInput,
) -> Result<YearView, AppError> {
    change(&state, year, |plan, today| {
        plan.add_annotation(AnnotationId::generate(), year, annotation.into(), today)?;
        Ok(())
    })
}

/// Edits a note or a period; hiding it from the timeline is `hidden: true`.
#[tauri::command]
#[specta::specta]
pub fn modify_annotation(
    state: State<'_, AppState>,
    year: i32,
    id: AnnotationId,
    annotation: AnnotationInput,
) -> Result<YearView, AppError> {
    change(&state, year, |plan, today| {
        plan.replace_annotation(id, year, annotation.into(), today)?;
        Ok(())
    })
}

#[tauri::command]
#[specta::specta]
pub fn delete_annotation(
    state: State<'_, AppState>,
    year: i32,
    id: AnnotationId,
) -> Result<YearView, AppError> {
    change(&state, year, |plan, today| {
        plan.delete_annotation(id, year, today)?;
        Ok(())
    })
}

/// Converts the timeline SVG serialised by the interface and writes it at
/// the path chosen in the "Save under" dialog.
#[tauri::command]
#[specta::specta]
pub async fn export_timeline(
    svg: String,
    format: TimelineExportFormat,
    path: PathBuf,
) -> Result<(), AppError> {
    let format = match format {
        TimelineExportFormat::Png => TimelineFormat::Png,
        TimelineExportFormat::Pdf => TimelineFormat::Pdf,
    };
    export::export_timeline(&svg, format, &path)?;
    Ok(())
}

/// Exports the whole garden in JSON (F-CPT-3).
#[tauri::command]
#[specta::specta]
pub fn export_garden(state: State<'_, AppState>, path: PathBuf) -> Result<(), AppError> {
    let plan = state
        .store()
        .load()?
        .ok_or_else(|| AppError::new(ErrorCode::PlotAbsent, "nothing to export"))?;
    export::export_garden(&plan, &path)?;
    Ok(())
}

/// Reads an import file and compares it with the plan of the device, without
/// replacing anything. A file that breaks the rules of the plan is accepted,
/// with the list of what to resolve (S03 §13); a file that cannot be read or
/// whose structure is broken is refused (`import_invalid`).
#[tauri::command]
#[specta::specta]
pub fn read_import(
    state: State<'_, AppState>,
    path: PathBuf,
) -> Result<ImportSummaryView, AppError> {
    let local = state.store().load()?;
    let plan = export::read_garden(&path)?;
    let summary = export::summarise(&plan, local.as_ref());
    Ok(ImportSummaryView {
        updated_at: summary.updated_at,
        version: u32::try_from(summary.version).unwrap_or(u32::MAX),
        bed_count: u32::try_from(summary.bed_count).unwrap_or(u32::MAX),
        planting_count: u32::try_from(summary.planting_count).unwrap_or(u32::MAX),
        older: summary.older,
        replaces: summary.replaces,
        incoherences: summary
            .incoherences
            .iter()
            .map(|i| incoherence_view(&plan, i))
            .collect(),
    })
}

/// What breaks the rules of the plan in the data of the device, to resolve
/// (S03 §13). Nothing is deleted or corrected: the list lasts, after an
/// import, until each case is settled with the usual actions.
#[tauri::command]
#[specta::specta]
pub fn incoherences_plan(state: State<'_, AppState>) -> Result<Vec<IncoherenceView>, AppError> {
    let Some(plan) = state.store().load()? else {
        return Ok(Vec::new());
    };
    Ok(plan
        .incoherences()
        .iter()
        .map(|i| incoherence_view(&plan, i))
        .collect())
}

/// Replaces the garden by the file, after confirmation (F-CPT-3).
#[tauri::command]
#[specta::specta]
pub fn import_garden(
    state: State<'_, AppState>,
    year: i32,
    path: PathBuf,
) -> Result<YearView, AppError> {
    let plan = export::read_garden(&path)?;
    // `read_garden` refused a broken structure; the other incoherences are
    // kept as they are (S03 §13) and were listed by `read_import`.
    state.store().replace(&plan)?;
    Ok(view(Some(&plan), year))
}

/// Empties the plan from `year` on (F-PLN-4): every bed of that year,
/// with its plantings and notes from then on. The plot, past years and the
/// notes of the whole garden stay.
#[tauri::command]
#[specta::specta]
pub fn clear_plan(state: State<'_, AppState>, year: i32) -> Result<YearView, AppError> {
    change(&state, year, |plan, today| {
        plan.clear_from(year, today)?;
        Ok(())
    })
}

/// Deletes every data of the app, after double confirmation (F-CPT-4).
#[tauri::command]
#[specta::specta]
pub fn erase_data(state: State<'_, AppState>) -> Result<(), AppError> {
    state.store().clear()?;
    Ok(())
}
