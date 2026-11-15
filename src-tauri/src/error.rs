//! Typed command errors (S05 §3, §7). The interface translates [`ErrorCode`];
//! `detail` is a technical message for logs, never shown as is.

use export::ExportError;
use plan::PlanError;
use serde::Serialize;
use specta::Type;
use store::StoreError;

/// Stable error codes. The codes of [`PlanError`] are kept exactly as the
/// `plan` crate names them (`PlanError::code`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    // From the plan (S03).
    AnneePassee,
    TooManyPlanches,
    PlancheInconnue,
    SerieInconnue,
    PlancheExistante,
    SerieExistante,
    CultureInconnue,
    ZoneInvalide,
    PlancheAbsente,
    ZoneHorsPlanche,
    Conflict,
    PotDaysRequired,
    FinManuelleInvalide,
    OccupationVide,
    LongueurInvalide,
    TerrainInvalide,
    AlleeInvalide,
    LargeurInvalide,
    NomInvalide,
    PlancheHorsTerrain,
    PlancheChevauche,
    AlleeTropEtroite,
    // From the app.
    /// A command needs a terrain, and none was created yet (E2).
    TerrainAbsent,
    TerrainExistant,
    /// The case size must be 25, 50 or 100 cm.
    CaseInvalide,
    AnnotationInconnue,
    /// A note or a period has 1 to 80 characters.
    TexteInvalide,
    /// A period ends before it starts, or has no end.
    AnnotationDatesInvalides,
    // From storage.
    /// The database could not be read or written.
    Enregistrement,
    /// The plan changed since it was read; reload and retry.
    VersionPerimee,
    /// The database was written by a newer version of the app.
    BaseTropRecente,
    // From exports and imports.
    Fichier,
    ImportInvalide,
    ImportTropRecent,
    SvgInvalide,
    RenduImpossible,
}

/// The error of every command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type, thiserror::Error)]
#[error("{code:?}: {detail}")]
pub struct AppError {
    pub code: ErrorCode,
    pub detail: String,
}

impl AppError {
    pub fn new(code: ErrorCode, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

impl From<PlanError> for AppError {
    fn from(error: PlanError) -> Self {
        // Exhaustive on purpose: a new PlanError must get a code here.
        let code = match &error {
            PlanError::PastYear { .. } => ErrorCode::AnneePassee,
            PlanError::TooManyPlanches { .. } => ErrorCode::TooManyPlanches,
            PlanError::UnknownPlanche(_) => ErrorCode::PlancheInconnue,
            PlanError::UnknownSerie(_) => ErrorCode::SerieInconnue,
            PlanError::UnknownAnnotation(_) => ErrorCode::AnnotationInconnue,
            PlanError::InvalidAnnotationText => ErrorCode::TexteInvalide,
            PlanError::InvalidAnnotationDates => ErrorCode::AnnotationDatesInvalides,
            PlanError::PlancheExists(_) => ErrorCode::PlancheExistante,
            PlanError::SerieExists(_) => ErrorCode::SerieExistante,
            PlanError::UnknownCulture(_) => ErrorCode::CultureInconnue,
            PlanError::InvalidZone { .. } => ErrorCode::ZoneInvalide,
            PlanError::PlancheMissing { .. } => ErrorCode::PlancheAbsente,
            PlanError::ZoneOutOfPlanche { .. } => ErrorCode::ZoneHorsPlanche,
            PlanError::Conflict { .. } => ErrorCode::Conflict,
            PlanError::PotDaysRequired => ErrorCode::PotDaysRequired,
            PlanError::InvalidManualEnd { .. } => ErrorCode::FinManuelleInvalide,
            PlanError::EmptyOccupation => ErrorCode::OccupationVide,
            PlanError::InvalidLength { .. } => ErrorCode::LongueurInvalide,
            PlanError::InvalidTerrain { .. } => ErrorCode::TerrainInvalide,
            PlanError::InvalidAllee(_) => ErrorCode::AlleeInvalide,
            PlanError::InvalidWidth(_) => ErrorCode::LargeurInvalide,
            PlanError::InvalidName => ErrorCode::NomInvalide,
            PlanError::OutsideTerrain { .. } => ErrorCode::PlancheHorsTerrain,
            PlanError::PlanchesOverlap { .. } => ErrorCode::PlancheChevauche,
            PlanError::AlleeTooNarrow { .. } => ErrorCode::AlleeTropEtroite,
        };
        Self::new(code, error.to_string())
    }
}

impl From<StoreError> for AppError {
    fn from(error: StoreError) -> Self {
        let code = match &error {
            StoreError::Database(_) | StoreError::Document(_) => ErrorCode::Enregistrement,
            StoreError::StaleVersion { .. } => ErrorCode::VersionPerimee,
            StoreError::SchemaTooNew { .. } => ErrorCode::BaseTropRecente,
        };
        Self::new(code, error.to_string())
    }
}

impl From<ExportError> for AppError {
    fn from(error: ExportError) -> Self {
        let code = match &error {
            ExportError::Io(_) => ErrorCode::Fichier,
            ExportError::InvalidImport(_) => ErrorCode::ImportInvalide,
            ExportError::ImportTooNew { .. } => ErrorCode::ImportTropRecent,
            ExportError::InvalidSvg(_) => ErrorCode::SvgInvalide,
            ExportError::Render(_) => ErrorCode::RenduImpossible,
        };
        Self::new(code, error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The serialised code of a PlanError is the one the plan crate defines.
    #[test]
    fn plan_codes_are_kept() {
        let errors = [
            PlanError::PotDaysRequired,
            PlanError::EmptyOccupation,
            PlanError::InvalidLength {
                longueur_cm: 30,
                case_cm: 50,
            },
            PlanError::UnknownCulture("x".into()),
        ];
        for error in errors {
            let expected = error.code();
            let json = serde_json::to_value(AppError::from(error)).unwrap();
            assert_eq!(json["code"], expected);
        }
    }
}
