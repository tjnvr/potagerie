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
    YearPast,
    TooManyBeds,
    BedUnknown,
    PlantingUnknown,
    BedExisting,
    PlantingExisting,
    CropUnknown,
    ZoneInvalid,
    BedAbsent,
    ZoneOutsideBed,
    Conflict,
    PotDaysRequired,
    ManualEndInvalid,
    OccupationEmpty,
    LengthInvalid,
    PlotInvalid,
    AisleInvalid,
    WidthInvalid,
    NameInvalid,
    BedOutsidePlot,
    BedOverlaps,
    AisleTooNarrow,
    // From the app.
    /// A command needs a plot, and none was created yet (E2).
    PlotAbsent,
    PlotExisting,
    /// The case size must be 25, 50 or 100 cm.
    CaseInvalid,
    AnnotationUnknown,
    /// A note or a period has 1 to 80 characters.
    TextInvalid,
    /// A period ends before it starts, or has no end.
    AnnotationDatesInvalid,
    // From storage.
    /// The database could not be read or written.
    Saving,
    /// The plan changed since it was read; reload and retry.
    VersionStale,
    /// The database was written by a newer version of the app.
    BaseTooRecent,
    // From exports and imports.
    File,
    ImportInvalid,
    ImportTooRecent,
    SvgInvalid,
    RenderingImpossible,
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
            PlanError::PastYear { .. } => ErrorCode::YearPast,
            PlanError::TooManyBeds { .. } => ErrorCode::TooManyBeds,
            PlanError::UnknownBed(_) => ErrorCode::BedUnknown,
            PlanError::UnknownPlanting(_) => ErrorCode::PlantingUnknown,
            PlanError::UnknownAnnotation(_) => ErrorCode::AnnotationUnknown,
            PlanError::InvalidAnnotationText => ErrorCode::TextInvalid,
            PlanError::InvalidAnnotationDates => ErrorCode::AnnotationDatesInvalid,
            PlanError::BedExists(_) => ErrorCode::BedExisting,
            PlanError::PlantingExists(_) => ErrorCode::PlantingExisting,
            PlanError::UnknownCrop(_) => ErrorCode::CropUnknown,
            PlanError::InvalidZone { .. } => ErrorCode::ZoneInvalid,
            PlanError::BedMissing { .. } => ErrorCode::BedAbsent,
            PlanError::ZoneOutOfBed { .. } => ErrorCode::ZoneOutsideBed,
            PlanError::Conflict { .. } => ErrorCode::Conflict,
            PlanError::PotDaysRequired => ErrorCode::PotDaysRequired,
            PlanError::InvalidManualEnd { .. } => ErrorCode::ManualEndInvalid,
            PlanError::EmptyOccupation => ErrorCode::OccupationEmpty,
            PlanError::InvalidLength { .. } => ErrorCode::LengthInvalid,
            PlanError::InvalidPlot { .. } => ErrorCode::PlotInvalid,
            PlanError::InvalidAisle(_) => ErrorCode::AisleInvalid,
            PlanError::InvalidWidth(_) => ErrorCode::WidthInvalid,
            PlanError::InvalidName => ErrorCode::NameInvalid,
            PlanError::OutsidePlot { .. } => ErrorCode::BedOutsidePlot,
            PlanError::BedsOverlap { .. } => ErrorCode::BedOverlaps,
            PlanError::AisleTooNarrow { .. } => ErrorCode::AisleTooNarrow,
        };
        Self::new(code, error.to_string())
    }
}

impl From<StoreError> for AppError {
    fn from(error: StoreError) -> Self {
        let code = match &error {
            StoreError::Database(_) | StoreError::Document(_) => ErrorCode::Saving,
            StoreError::StaleVersion { .. } => ErrorCode::VersionStale,
            StoreError::SchemaTooNew { .. } => ErrorCode::BaseTooRecent,
        };
        Self::new(code, error.to_string())
    }
}

impl From<ExportError> for AppError {
    fn from(error: ExportError) -> Self {
        let code = match &error {
            ExportError::Io(_) => ErrorCode::File,
            ExportError::InvalidImport(_) => ErrorCode::ImportInvalid,
            ExportError::ImportTooNew { .. } => ErrorCode::ImportTooRecent,
            ExportError::InvalidSvg(_) => ErrorCode::SvgInvalid,
            ExportError::Render(_) => ErrorCode::RenderingImpossible,
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
                length_cm: 30,
                cell_cm: 50,
            },
            PlanError::UnknownCrop("x".into()),
        ];
        for error in errors {
            let expected = error.code();
            let json = serde_json::to_value(AppError::from(error)).unwrap();
            assert_eq!(json["code"], expected);
        }
    }
}
