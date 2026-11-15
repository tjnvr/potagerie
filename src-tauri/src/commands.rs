//! Tauri commands (S05 §7). Each command that changes the plan loads it,
//! applies one operation of the `plan` crate, saves it (`version + 1`) and
//! returns the year view. A refused command changes nothing.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use catalogue::Catalogue;
use chrono::{DateTime, Local, NaiveDate, Utc};
use export::TimelineFormat;
use plan::{
    Annotation, AnnotationId, CaseCm, Disposition, Geometry, Mode, Plan, PlanId, PlancheId, Serie,
    SerieId, Terrain, TypeAnnotation, TypePlanche, YearView, Zone,
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

/// Adds a serie; the durations are copied from the catalogue (S03 §2, §5).
#[tauri::command]
#[specta::specta]
pub fn creer_serie(
    state: State<'_, AppState>,
    annee: i32,
    serie: SerieInput,
) -> Result<YearView, AppError> {
    change(&state, annee, |plan, _| {
        let culture = Catalogue::v1()
            .culture(&serie.culture_id)
            .ok_or_else(|| plan::PlanError::UnknownCulture(serie.culture_id.clone()))?;
        let new = Serie::new(
            culture,
            serie.planche_id,
            Zone::new(serie.troncon_debut, serie.troncon_fin)?,
            serie.mode,
            serie.date,
            serie.abri_jours,
            serie.fin_manuelle,
        )?;
        plan.add_serie(SerieId::generate(), new)?;
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
        let mut edited = plan
            .series
            .get(&id)
            .ok_or(plan::PlanError::UnknownSerie(id))?
            .clone();
        let culture = Catalogue::v1()
            .culture(&serie.culture_id)
            .ok_or_else(|| plan::PlanError::UnknownCulture(serie.culture_id.clone()))?;
        if edited.culture_id != serie.culture_id {
            edited.change_culture(culture, serie.abri_jours)?;
        }
        if edited.mode != serie.mode {
            edited.change_mode(serie.mode, culture, serie.abri_jours)?;
        } else if let (Mode::Abri, 0, Some(days)) =
            (serie.mode, culture.abri_jours, serie.abri_jours)
        {
            edited.abri_jours = days;
        }
        let zone = Zone::new(serie.troncon_debut, serie.troncon_fin)?;
        edited.planche_id = serie.planche_id;
        edited.troncon_debut = zone.t1;
        edited.troncon_fin = zone.t2;
        edited.date = serie.date;
        edited.fin_manuelle = serie.fin_manuelle;
        edited.check()?;
        plan.replace_serie(id, edited)?;
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

// TODO(plan): annotations have no rule in the plan crate yet (past years,
// unknown planche, end before start). They are stored without validation.

#[tauri::command]
#[specta::specta]
pub fn creer_annotation(
    state: State<'_, AppState>,
    annee: i32,
    annotation: AnnotationInput,
) -> Result<YearView, AppError> {
    change(&state, annee, |plan, _| {
        plan.annotations
            .insert(AnnotationId::generate(), annotation.into());
        Ok(())
    })
}

#[tauri::command]
#[specta::specta]
pub fn modifier_annotation(
    state: State<'_, AppState>,
    annee: i32,
    id: AnnotationId,
    annotation: AnnotationInput,
) -> Result<YearView, AppError> {
    change(&state, annee, |plan, _| {
        let slot = plan
            .annotations
            .get_mut(&id)
            .ok_or_else(|| AppError::new(ErrorCode::AnnotationInconnue, id.to_string()))?;
        *slot = annotation.into();
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
    change(&state, annee, |plan, _| {
        plan.annotations
            .remove(&id)
            .ok_or_else(|| AppError::new(ErrorCode::AnnotationInconnue, id.to_string()))?;
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
/// replacing anything.
#[tauri::command]
#[specta::specta]
pub fn lire_import(
    state: State<'_, AppState>,
    chemin: PathBuf,
) -> Result<ImportSummaryView, AppError> {
    let local = state.store().load()?;
    let summary = export::read_import(&chemin, local.as_ref())?;
    Ok(ImportSummaryView {
        updated_at: summary.updated_at,
        version: u32::try_from(summary.version).unwrap_or(u32::MAX),
        planche_count: u32::try_from(summary.planche_count).unwrap_or(u32::MAX),
        serie_count: u32::try_from(summary.serie_count).unwrap_or(u32::MAX),
        older: summary.older,
        replaces: summary.replaces,
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
    // TODO(plan): validate the imported plan (S03 §13) before replacing.
    state.store().replace(&plan)?;
    Ok(view(Some(&plan), annee))
}

/// Deletes every data of the app, after double confirmation (F-CPT-4).
#[tauri::command]
#[specta::specta]
pub fn effacer_donnees(state: State<'_, AppState>) -> Result<(), AppError> {
    state.store().clear()?;
    Ok(())
}
