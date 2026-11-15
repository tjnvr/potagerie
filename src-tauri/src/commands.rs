//! Tauri commands (S05 §7). Each command that changes the plan loads it,
//! applies one operation of the `plan` crate, saves it (`version + 1`) and
//! returns the year view. A refused command changes nothing.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use catalogue::Catalogue;
use chrono::{DateTime, Local, NaiveDate, Utc};
use export::TimelineFormat;
use plan::{
    Annotation, AnnotationId, CaseCm, Disposition, Geometry, Incoherence, Mode, Plan, PlanId,
    PlancheId, Serie, SerieId, Terrain, TypeAnnotation, TypePlanche, YearView, Zone,
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

fn view(plan: Option<&Plan>, annee: i32) -> YearView {
    YearView::build(plan, annee, today(), Catalogue::v1())
}

/// Loads the plan, applies `change`, saves it and returns the view of `annee`.
fn change(
    state: &AppState,
    annee: i32,
    change: impl FnOnce(&mut Plan, NaiveDate) -> Result<(), AppError>,
) -> Result<YearView, AppError> {
    let mut store = state.store();
    let mut plan = store
        .load()?
        .ok_or_else(|| AppError::new(ErrorCode::TerrainAbsent, "create the terrain first"))?;
    let today = today();
    change(&mut plan, today)?;
    store.save(&mut plan, Utc::now())?;
    Ok(YearView::build(Some(&plan), annee, today, Catalogue::v1()))
}

// ---------- Inputs and outputs ----------

/// A crop of the catalogue (S02). The name is not sent: the interface
/// translates `culture_<id>`.
#[derive(Debug, Clone, Serialize, Type)]
pub struct CultureView {
    pub id: String,
    pub famille: String,
    pub abri_jours: u32,
    pub recolte_apres_jours: u32,
    pub recolte_duree_jours: u32,
    /// `MM-JJ`.
    pub mise_en_place_debut: String,
    /// `MM-JJ`.
    pub mise_en_place_fin: String,
    pub rang_cm: u32,
    pub pied_cm: u32,
}

#[derive(Debug, Clone, Deserialize, Type)]
pub struct TerrainInput {
    pub longueur_cm: u32,
    pub largeur_cm: u32,
    /// 25, 50 or 100; fixed once the terrain is created.
    pub case_cm: u32,
    pub allee_cm: u32,
}

#[derive(Debug, Clone, Deserialize, Type)]
pub struct TerrainEdit {
    pub longueur_cm: u32,
    pub largeur_cm: u32,
    pub allee_cm: u32,
}

#[derive(Debug, Clone, Deserialize, Type)]
pub struct GeometryInput {
    pub planche_type: TypePlanche,
    pub x: u32,
    pub y: u32,
    pub longueur_cm: u32,
    pub largeur_cm: u32,
}

impl From<GeometryInput> for Geometry {
    fn from(g: GeometryInput) -> Self {
        Self {
            planche_type: g.planche_type,
            x: g.x,
            y: g.y,
            longueur_cm: g.longueur_cm,
            largeur_cm: g.largeur_cm,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Type)]
pub struct SerieInput {
    pub culture_id: String,
    pub planche_id: PlancheId,
    pub troncon_debut: u32,
    pub troncon_fin: u32,
    pub mode: Mode,
    /// Sowing (abri, pleine terre) or planting (plants) date.
    pub date: NaiveDate,
    /// Days in pots, only for the mode abri of a crop without pots (S03 §2).
    pub abri_jours: Option<u32>,
    pub fin_manuelle: Option<NaiveDate>,
    /// Mode abri only: `date` is the transplanting date, not the sowing date.
    /// The serie stores the sowing date `s = x - A` (S03 §2).
    #[serde(default)]
    pub date_est_repiquage: bool,
}

/// A zone of troncons.
#[derive(Debug, Clone, Copy, Serialize, Type)]
pub struct ZoneView {
    pub troncon_debut: u32,
    pub troncon_fin: u32,
}

/// The zones of the shortcuts of F-PLA-1; `None` greys the shortcut out
/// (S03 §4).
#[derive(Debug, Clone, Serialize, Type)]
pub struct RaccourcisView {
    pub toute: Option<ZoneView>,
    pub moitie: Option<ZoneView>,
    pub tiers: Option<ZoneView>,
    pub quart: Option<ZoneView>,
}

/// Why a serie is not valid (S03 §5): a code to translate, and what the
/// interface needs to say where and when.
#[derive(Debug, Clone, Serialize, Type)]
pub struct ProblemeView {
    pub code: ErrorCode,
    /// The other serie of a conflict.
    pub avec: Option<SerieId>,
    /// Troncons in common (conflict) or the zone that does not fit.
    pub zone: Option<ZoneView>,
    /// Period in common (conflict), `[start, end[`.
    pub start: Option<NaiveDate>,
    pub end: Option<NaiveDate>,
    /// Year of a missing planche or of a zone that does not fit.
    pub annee: Option<i32>,
    /// Troncons of the planche that year (zone that does not fit).
    pub troncons: Option<u32>,
}

/// The dates and quantities of a serie being entered (S03 §2, §6).
#[derive(Debug, Clone, Serialize, Type)]
pub struct DatesApercu {
    /// The date Rust would store (the sowing date, even if a transplanting
    /// date was entered).
    pub date: NaiveDate,
    pub sowing: Option<NaiveDate>,
    pub planting_out: NaiveDate,
    pub harvest_start: NaiveDate,
    pub end: NaiveDate,
    /// `end` without the manual end: « Recalculer ».
    pub default_end: NaiveDate,
    pub manual_end: bool,
    pub pot_days: u32,
    pub rangs: u32,
    pub pieds_par_rang: u32,
}

/// What the form of E07 shows while the user types: computed by Rust, never by
/// the interface.
#[derive(Debug, Clone, Serialize, Type)]
pub struct SerieApercu {
    /// `None` when the serie cannot be built (a required field is missing).
    pub dates: Option<DatesApercu>,
    /// Empty when the serie is valid.
    pub problemes: Vec<ProblemeView>,
    pub raccourcis: RaccourcisView,
    /// The zones of other series of the planche that overlap the occupation in
    /// time: the troncons the strip of the form shows as taken.
    pub occupes: Vec<ZoneView>,
}

fn zone_view(zone: Zone) -> ZoneView {
    ZoneView {
        troncon_debut: zone.t1,
        troncon_fin: zone.t2,
    }
}

fn probleme(error: plan::PlanError) -> ProblemeView {
    let mut p = ProblemeView {
        code: AppError::from(error.clone()).code,
        avec: None,
        zone: None,
        start: None,
        end: None,
        annee: None,
        troncons: None,
    };
    match error {
        plan::PlanError::Conflict {
            with,
            troncons,
            period,
        } => {
            p.avec = Some(with);
            p.zone = Some(zone_view(troncons));
            p.start = Some(period.start);
            p.end = Some(period.end);
        }
        plan::PlanError::ZoneOutOfPlanche {
            year,
            troncons,
            zone,
        } => {
            p.annee = Some(year);
            p.troncons = Some(troncons);
            p.zone = Some(zone_view(zone));
        }
        plan::PlanError::PlancheMissing { year, .. } => p.annee = Some(year),
        _ => {}
    }
    p
}

#[derive(Debug, Clone, Deserialize, Type)]
pub struct AnnotationInput {
    pub annotation_type: TypeAnnotation,
    pub date_debut: NaiveDate,
    pub date_fin: Option<NaiveDate>,
    pub planche_id: Option<PlancheId>,
    pub texte: String,
    pub masquee: bool,
}

impl From<AnnotationInput> for Annotation {
    fn from(a: AnnotationInput) -> Self {
        Self {
            type_annotation: a.annotation_type,
            date_debut: a.date_debut,
            date_fin: a.date_fin,
            planche_id: a.planche_id,
            texte: a.texte,
            masquee: a.masquee,
        }
    }
}

/// What an import file holds, before it replaces anything (F-CPT-3).
#[derive(Debug, Clone, Serialize, Type)]
pub struct ImportSummaryView {
    pub updated_at: DateTime<Utc>,
    pub version: u32,
    pub planche_count: u32,
    pub serie_count: u32,
    /// The file is older than the plan of the device: warn before replacing.
    pub older: bool,
    /// The device has a plan that the import would replace.
    pub replaces: bool,
    /// What breaks the rules of the plan: kept, to resolve (S03 §13).
    pub incoherences: Vec<IncoherenceView>,
}

/// One incoherence of an imported plan. `code` is `trop_de_planches`,
/// `serie_orpheline`, `chevauchement`, `zone_hors_planche`, `culture_inconnue`,
/// `note_planche_inconnue`, `note_invalide`, or a placement code
/// (`planche_hors_terrain`, `planche_chevauche`, `allee_trop_etroite`,
/// `largeur_invalide`). Names come from the file, not from the interface.
#[derive(Debug, Clone, Serialize, Type)]
pub struct IncoherenceView {
    pub code: String,
    pub annee: Option<i32>,
    pub planche_nom: Option<String>,
    pub autre_planche_nom: Option<String>,
    pub culture_id: Option<String>,
    pub autre_culture_id: Option<String>,
    /// Planches in a year, or troncons of the planche.
    pub nombre: Option<u32>,
}

fn incoherence_view(plan: &Plan, incoherence: &Incoherence) -> IncoherenceView {
    let nom = |id: &PlancheId| plan.planches.get(id).map(|p| p.nom.clone());
    let culture = |id: &SerieId| plan.series.get(id).map(|s| s.culture_id.clone());
    let mut v = IncoherenceView {
        code: incoherence.code().to_owned(),
        annee: None,
        planche_nom: None,
        autre_planche_nom: None,
        culture_id: None,
        autre_culture_id: None,
        nombre: None,
    };
    match incoherence {
        Incoherence::TooManyPlanches { year, count } => {
            v.annee = Some(*year);
            v.nombre = u32::try_from(*count).ok();
        }
        Incoherence::OrphanSerie {
            serie_id,
            planche_id,
            year,
        } => {
            v.annee = *year;
            v.planche_nom = nom(planche_id);
            v.culture_id = culture(serie_id);
        }
        Incoherence::ZoneOutOfPlanche {
            serie_id,
            planche_id,
            year,
            troncons,
            ..
        } => {
            v.annee = Some(*year);
            v.nombre = Some(*troncons);
            v.planche_nom = nom(planche_id);
            v.culture_id = culture(serie_id);
        }
        Incoherence::Conflict { a, b } => {
            v.culture_id = culture(a);
            v.autre_culture_id = culture(b);
            v.planche_nom = plan.series.get(a).and_then(|s| nom(&s.planche_id));
        }
        Incoherence::Placement(error) => match error {
            plan::PlanError::OutsideTerrain { planche_id, year } => {
                v.annee = Some(*year);
                v.planche_nom = nom(planche_id);
            }
            plan::PlanError::PlanchesOverlap { a, b, year }
            | plan::PlanError::AlleeTooNarrow { a, b, year, .. } => {
                v.annee = Some(*year);
                v.planche_nom = nom(a);
                v.autre_planche_nom = nom(b);
            }
            _ => {}
        },
        Incoherence::UnknownCulture {
            serie_id,
            culture_id,
        } => {
            v.culture_id = Some(culture_id.clone());
            v.planche_nom = plan.series.get(serie_id).and_then(|s| nom(&s.planche_id));
        }
        Incoherence::AnnotationUnknownPlanche { .. } | Incoherence::AnnotationInvalid { .. } => {}
    }
    v
}

#[derive(Debug, Clone, Copy, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum FriseFormat {
    Png,
    Pdf,
}

/// Size of a new planche dropped from the tray (S01 F-PLN-1): longueur × largeur.
fn default_size(planche_type: TypePlanche) -> (u32, u32) {
    match planche_type {
        TypePlanche::Planche => (300, 120),
        TypePlanche::Carre => (100, 100),
        TypePlanche::Butte => (300, 120),
        TypePlanche::Bac => (200, 120),
    }
}

// ---------- Commands ----------

/// The crops of S02.
#[tauri::command]
#[specta::specta]
pub fn catalogue() -> Vec<CultureView> {
    Catalogue::v1()
        .iter()
        .map(|c| CultureView {
            id: c.id.clone(),
            famille: c.famille.clone(),
            abri_jours: c.abri_jours,
            recolte_apres_jours: c.recolte_apres_jours,
            recolte_duree_jours: c.recolte_duree_jours,
            mise_en_place_debut: c.mise_en_place.debut.to_string(),
            mise_en_place_fin: c.mise_en_place.fin.to_string(),
            rang_cm: c.rang_cm,
            pied_cm: c.pied_cm,
        })
        .collect()
}

/// Opens a year (tabs Potager and Plan).
#[tauri::command]
#[specta::specta]
pub fn vue_annee(state: State<'_, AppState>, annee: i32) -> Result<YearView, AppError> {
    let plan = state.store().load()?;
    Ok(view(plan.as_ref(), annee))
}

/// Creates the terrain, and with it the plan (E2, F-TER-1).
#[tauri::command]
#[specta::specta]
pub fn creer_terrain(
    state: State<'_, AppState>,
    annee: i32,
    terrain: TerrainInput,
) -> Result<YearView, AppError> {
    let mut store = state.store();
    if store.load()?.is_some() {
        return Err(AppError::new(
            ErrorCode::TerrainExistant,
            "the terrain already exists",
        ));
    }
    let case_cm =
        CaseCm::try_from(terrain.case_cm).map_err(|e| AppError::new(ErrorCode::CaseInvalide, e))?;
    let terrain = Terrain {
        longueur_cm: terrain.longueur_cm,
        largeur_cm: terrain.largeur_cm,
        case_cm,
        allee_cm: terrain.allee_cm,
    };
    terrain.validate()?;
    let mut plan = Plan::new(PlanId::generate(), terrain, Utc::now());
    store.save(&mut plan, Utc::now())?;
    Ok(view(Some(&plan), annee))
}

/// Changes the dimensions of the terrain and the allée width. The case size
/// is fixed at creation.
#[tauri::command]
#[specta::specta]
pub fn modifier_terrain(
    state: State<'_, AppState>,
    annee: i32,
    terrain: TerrainEdit,
) -> Result<YearView, AppError> {
    change(&state, annee, |plan, today| {
        plan.set_terrain(
            terrain.longueur_cm,
            terrain.largeur_cm,
            terrain.allee_cm,
            today,
        )?;
        Ok(())
    })
}

/// Drops a new planche from the tray at `(x, y)`, at its default size (F-PLN-6).
/// The default name ("Planche A") is chosen and translated by the interface.
#[tauri::command]
#[specta::specta]
pub fn creer_planche(
    state: State<'_, AppState>,
    annee: i32,
    planche_type: TypePlanche,
    nom: String,
    x: u32,
    y: u32,
) -> Result<YearView, AppError> {
    change(&state, annee, |plan, today| {
        let (longueur_cm, largeur_cm) = default_size(planche_type);
        plan::check_name(&nom)?;
        plan.add_planche(
            PlancheId::generate(),
            nom,
            annee,
            Geometry {
                planche_type,
                x,
                y,
                longueur_cm,
                largeur_cm,
            },
            today,
        )?;
        plan.check_placements(annee)?;
        Ok(())
    })
}

/// Renames a planche and/or changes its layout from `annee` on (S03 §7).
#[tauri::command]
#[specta::specta]
pub fn modifier_planche(
    state: State<'_, AppState>,
    annee: i32,
    id: PlancheId,
    nom: Option<String>,
    geometrie: Option<GeometryInput>,
) -> Result<YearView, AppError> {
    change(&state, annee, |plan, today| {
        if let Some(geometry) = geometrie {
            plan.set_disposition(id, annee, Disposition::Present(geometry.into()), today)?;
            plan.check_placements(annee)?;
        }
        if let Some(nom) = nom {
            plan::check_name(&nom)?;
            plan.planches
                .get_mut(&id)
                .ok_or(plan::PlanError::UnknownPlanche(id))?
                .nom = nom;
        }
        Ok(())
    })
}

/// Moves a planche from `annee` on, keeping its size.
#[tauri::command]
#[specta::specta]
pub fn deplacer_planche(
    state: State<'_, AppState>,
    annee: i32,
    id: PlancheId,
    x: u32,
    y: u32,
) -> Result<YearView, AppError> {
    change(&state, annee, |plan, today| {
        let mut geometry = plan
            .planches
            .get(&id)
            .ok_or(plan::PlanError::UnknownPlanche(id))?
            .disposition_in(annee)
            .ok_or(plan::PlanError::PlancheMissing {
                planche_id: id,
                year: annee,
            })?
            .clone();
        geometry.x = x;
        geometry.y = y;
        plan.set_disposition(id, annee, Disposition::Present(geometry), today)?;
        plan.check_placements(annee)?;
        Ok(())
    })
}

/// The series that would no longer fit if the planche were shortened to
/// `longueur_cm` from `annee` on (S03 §4). Read-only: the interface warns
/// before it calls `modifier_planche`.
#[tauri::command]
#[specta::specta]
pub fn series_depassant(
    state: State<'_, AppState>,
    annee: i32,
    id: PlancheId,
    longueur_cm: u32,
) -> Result<Vec<SerieId>, AppError> {
    let plan = state
        .store()
        .load()?
        .ok_or_else(|| AppError::new(ErrorCode::TerrainAbsent, "create the terrain first"))?;
    let case_cm = plan.terrain.case_cm.cm();
    Ok(plan.series_overflowing_if_shortened(id, annee, longueur_cm / case_cm))
}

/// Deletes a planche from `annee` on, with its series that touch `annee` or
/// later (F-PLN-4). The interface confirms first, with `series_deleted_by`.
#[tauri::command]
#[specta::specta]
pub fn supprimer_planche(
    state: State<'_, AppState>,
    annee: i32,
    id: PlancheId,
) -> Result<YearView, AppError> {
    change(&state, annee, |plan, today| {
        plan.delete_planche(id, annee, today)?;
        Ok(())
    })
}

/// The serie described by the form, ready to be validated (S03 §2).
///
/// Without `existing`, the durations are copied from the catalogue; with it,
/// they are copied again only if the crop or the mode changes.
fn build_serie(existing: Option<&Serie>, input: &SerieInput) -> Result<Serie, plan::PlanError> {
    let culture = Catalogue::v1()
        .culture(&input.culture_id)
        .ok_or_else(|| plan::PlanError::UnknownCulture(input.culture_id.clone()))?;
    let zone = Zone::new(input.troncon_debut, input.troncon_fin)?;
    let date = if input.date_est_repiquage && input.mode == Mode::Abri {
        let pot_days = if culture.abri_jours > 0 {
            culture.abri_jours
        } else {
            input.abri_jours.unwrap_or(0)
        };
        if pot_days == 0 {
            return Err(plan::PlanError::PotDaysRequired);
        }
        Serie::sowing_from_transplanting(input.date, pot_days)
    } else {
        input.date
    };
    let Some(old) = existing else {
        return Serie::new(
            culture,
            input.planche_id,
            zone,
            input.mode,
            date,
            input.abri_jours,
            input.fin_manuelle,
        );
    };
    // The entered fields first: the checks of a change of crop or mode must
    // see the new date and end, not the old manual end.
    let mut edited = old.clone();
    edited.planche_id = input.planche_id;
    edited.troncon_debut = zone.t1;
    edited.troncon_fin = zone.t2;
    edited.date = date;
    edited.fin_manuelle = input.fin_manuelle;
    if edited.culture_id != input.culture_id {
        edited.change_culture(culture, input.abri_jours)?;
    }
    if edited.mode != input.mode {
        edited.change_mode(input.mode, culture, input.abri_jours)?;
    } else if let (Mode::Abri, 0, Some(days)) = (input.mode, culture.abri_jours, input.abri_jours) {
        edited.abri_jours = days;
    }
    edited.check()?;
    Ok(edited)
}

/// Adds a serie; the durations are copied from the catalogue (S03 §2, §5).
/// Refused, with the plan untouched, on a conflict, a zone outside the
/// planche or a planche missing in a year the serie occupies.
#[tauri::command]
#[specta::specta]
pub fn creer_serie(
    state: State<'_, AppState>,
    annee: i32,
    serie: SerieInput,
) -> Result<YearView, AppError> {
    change(&state, annee, |plan, _| {
        plan.add_serie(SerieId::generate(), build_serie(None, &serie)?)?;
        Ok(())
    })
}

/// Changes a serie. Durations are copied again only if the crop or the mode
/// changes (S03 §2).
#[tauri::command]
#[specta::specta]
pub fn modifier_serie(
    state: State<'_, AppState>,
    annee: i32,
    id: SerieId,
    serie: SerieInput,
) -> Result<YearView, AppError> {
    change(&state, annee, |plan, _| {
        let old = plan
            .series
            .get(&id)
            .ok_or(plan::PlanError::UnknownSerie(id))?;
        let edited = build_serie(Some(old), &serie)?;
        plan.replace_serie(id, edited)?;
        Ok(())
    })
}

/// Drags a bar by `jours` days (S03 §2, T12): the entered date and the manual
/// end shift. Refused with `conflict` (or another code) if the shifted serie
/// is not valid; nothing changes and the interface brings the bar back.
#[tauri::command]
#[specta::specta]
pub fn decaler_serie(
    state: State<'_, AppState>,
    annee: i32,
    id: SerieId,
    jours: i32,
) -> Result<YearView, AppError> {
    change(&state, annee, |plan, _| {
        plan.shift_serie(id, i64::from(jours))?;
        Ok(())
    })
}

#[tauri::command]
#[specta::specta]
pub fn supprimer_serie(
    state: State<'_, AppState>,
    annee: i32,
    id: SerieId,
) -> Result<YearView, AppError> {
    change(&state, annee, |plan, _| {
        plan.delete_serie(id)?;
        Ok(())
    })
}

/// Checks a serie as the form of E07 fills in, without saving: its dates and
/// quantities, why it is not valid, and the zones of the shortcuts. `id` is
/// the serie being edited, which is not compared with its previous self.
#[tauri::command]
#[specta::specta]
pub fn apercu_serie(
    state: State<'_, AppState>,
    id: Option<SerieId>,
    serie: SerieInput,
) -> Result<SerieApercu, AppError> {
    let plan = state
        .store()
        .load()?
        .ok_or_else(|| AppError::new(ErrorCode::TerrainAbsent, "create the terrain first"))?;
    Ok(preview_serie(&plan, id, &serie))
}

fn preview_serie(plan: &Plan, id: Option<SerieId>, input: &SerieInput) -> SerieApercu {
    let catalogue = Catalogue::v1();
    let existing = id.and_then(|id| plan.series.get(&id));
    let (built, problemes) = match build_serie(existing, input) {
        Ok(serie) => {
            let problems = plan
                .serie_problems(id, &serie)
                .into_iter()
                .map(probleme)
                .collect::<Vec<_>>();
            (Some(serie), problems)
        }
        Err(error) => (None, vec![probleme(error)]),
    };

    // The shortcuts need the occupation, not the zone: probe with one troncon
    // when the entered zone is the problem.
    let probe = built.clone().or_else(|| {
        let mut probe = input.clone();
        (probe.troncon_debut, probe.troncon_fin) = (1, 1);
        build_serie(existing, &probe).ok()
    });
    let shortcut = |fraction| {
        let serie = probe.as_ref()?;
        plan.shortcut(serie.planche_id, fraction, serie.occupation(), id)
            .map(zone_view)
    };
    let raccourcis = RaccourcisView {
        toute: shortcut(plan::Fraction::Whole),
        moitie: shortcut(plan::Fraction::Half),
        tiers: shortcut(plan::Fraction::Third),
        quart: shortcut(plan::Fraction::Quarter),
    };

    let occupes = probe
        .as_ref()
        .map(|serie| {
            let occupation = serie.occupation();
            plan.series
                .iter()
                .filter(|(other, s)| {
                    Some(**other) != id
                        && s.planche_id == serie.planche_id
                        && s.occupation().overlaps(&occupation)
                })
                .map(|(_, s)| zone_view(s.zone()))
                .collect()
        })
        .unwrap_or_default();

    let dates = built.map(|serie| {
        let d = serie.dates();
        let start = d.sowing.unwrap_or(d.planting_out).min(d.planting_out);
        let default_end = Serie {
            fin_manuelle: None,
            ..serie.clone()
        }
        .dates()
        .end;
        let (rangs, pieds_par_rang) = match (
            plan.planches
                .get(&serie.planche_id)
                .and_then(|p| p.disposition_in(chrono::Datelike::year(&start))),
            catalogue.culture(&serie.culture_id),
        ) {
            (Some(g), Some(culture)) => {
                let q = plan::quantity(
                    g.largeur_cm,
                    serie.troncon_fin - serie.troncon_debut + 1,
                    plan.terrain.case_cm,
                    culture,
                );
                (q.rows, q.plants_per_row)
            }
            _ => (0, 0),
        };
        DatesApercu {
            date: serie.date,
            sowing: d.sowing,
            planting_out: d.planting_out,
            harvest_start: d.harvest_start,
            end: d.end,
            default_end,
            manual_end: serie.fin_manuelle.is_some(),
            pot_days: serie.abri_jours,
            rangs,
            pieds_par_rang,
        }
    });
    SerieApercu {
        dates,
        problemes,
        raccourcis,
        occupes,
    }
}

/// The crops whose planting-out window contains `date` (S03 §10, F-CAT-3
/// « de saison maintenant »).
#[tauri::command]
#[specta::specta]
pub fn cultures_de_saison(date: NaiveDate) -> Vec<String> {
    Catalogue::v1()
        .iter()
        .filter(|c| plan::in_window(c, date))
        .map(|c| c.id.clone())
        .collect()
}

/// Adds a note or a period (S01 F-VIS-5); the plan crate checks the rules.
#[tauri::command]
#[specta::specta]
pub fn creer_annotation(
    state: State<'_, AppState>,
    annee: i32,
    annotation: AnnotationInput,
) -> Result<YearView, AppError> {
    change(&state, annee, |plan, today| {
        plan.add_annotation(AnnotationId::generate(), annee, annotation.into(), today)?;
        Ok(())
    })
}

/// Edits a note or a period; hiding it from the timeline is `masquee: true`.
#[tauri::command]
#[specta::specta]
pub fn modifier_annotation(
    state: State<'_, AppState>,
    annee: i32,
    id: AnnotationId,
    annotation: AnnotationInput,
) -> Result<YearView, AppError> {
    change(&state, annee, |plan, today| {
        plan.replace_annotation(id, annee, annotation.into(), today)?;
        Ok(())
    })
}

#[tauri::command]
#[specta::specta]
pub fn supprimer_annotation(
    state: State<'_, AppState>,
    annee: i32,
    id: AnnotationId,
) -> Result<YearView, AppError> {
    change(&state, annee, |plan, today| {
        plan.delete_annotation(id, annee, today)?;
        Ok(())
    })
}

/// Converts the timeline SVG serialised by the interface and writes it at
/// the path chosen in the "Enregistrer sous" dialog.
#[tauri::command]
#[specta::specta]
pub async fn exporter_frise(
    svg: String,
    format: FriseFormat,
    chemin: PathBuf,
) -> Result<(), AppError> {
    let format = match format {
        FriseFormat::Png => TimelineFormat::Png,
        FriseFormat::Pdf => TimelineFormat::Pdf,
    };
    export::export_timeline(&svg, format, &chemin)?;
    Ok(())
}

/// Exports the whole potager in JSON (F-CPT-3).
#[tauri::command]
#[specta::specta]
pub fn exporter_potager(state: State<'_, AppState>, chemin: PathBuf) -> Result<(), AppError> {
    let plan = state
        .store()
        .load()?
        .ok_or_else(|| AppError::new(ErrorCode::TerrainAbsent, "nothing to export"))?;
    export::export_garden(&plan, &chemin)?;
    Ok(())
}

/// Reads an import file and compares it with the plan of the device, without
/// replacing anything. A file that breaks the rules of the plan is accepted,
/// with the list of what to resolve (S03 §13); a file that cannot be read or
/// whose structure is broken is refused (`import_invalide`).
#[tauri::command]
#[specta::specta]
pub fn lire_import(
    state: State<'_, AppState>,
    chemin: PathBuf,
) -> Result<ImportSummaryView, AppError> {
    let local = state.store().load()?;
    let plan = export::read_garden(&chemin)?;
    let summary = export::summarise(&plan, local.as_ref());
    Ok(ImportSummaryView {
        updated_at: summary.updated_at,
        version: u32::try_from(summary.version).unwrap_or(u32::MAX),
        planche_count: u32::try_from(summary.planche_count).unwrap_or(u32::MAX),
        serie_count: u32::try_from(summary.serie_count).unwrap_or(u32::MAX),
        older: summary.older,
        replaces: summary.replaces,
        incoherences: summary
            .incoherences
            .iter()
            .map(|i| incoherence_view(&plan, i))
            .collect(),
    })
}

/// Replaces the potager by the file, after confirmation (F-CPT-3).
#[tauri::command]
#[specta::specta]
pub fn importer_potager(
    state: State<'_, AppState>,
    annee: i32,
    chemin: PathBuf,
) -> Result<YearView, AppError> {
    let plan = export::read_garden(&chemin)?;
    // `read_garden` refused a broken structure; the other incoherences are
    // kept as they are (S03 §13) and were listed by `lire_import`.
    state.store().replace(&plan)?;
    Ok(view(Some(&plan), annee))
}

/// Empties the plan from `annee` on (F-PLN-4): every planche of that year,
/// with its series and notes from then on. The terrain, past years and the
/// notes of the whole potager stay.
#[tauri::command]
#[specta::specta]
pub fn vider_plan(state: State<'_, AppState>, annee: i32) -> Result<YearView, AppError> {
    change(&state, annee, |plan, today| {
        plan.clear_from(annee, today)?;
        Ok(())
    })
}

/// Deletes every data of the app, after double confirmation (F-CPT-4).
#[tauri::command]
#[specta::specta]
pub fn effacer_donnees(state: State<'_, AppState>) -> Result<(), AppError> {
    state.store().clear()?;
    Ok(())
}
