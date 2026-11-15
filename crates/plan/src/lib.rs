//! Potagerie plan: data model (S05 §5) and calculations (S03).
//!
//! The crate has no clock, no storage and no UI. Plan dates are civil
//! (`chrono::NaiveDate`); "today" is always a parameter; technical timestamps
//! are UTC. Nothing derived is stored in a [`Plan`]: dates, conflicts, slots,
//! suggestions and tasks are recomputed from it.

mod annotations;
mod clearing;
mod coherence;
mod conflicts;
mod days;
mod error;
mod ids;
mod layouts;
mod legacy;
mod model;
mod operations;
mod placement;
mod planting;
mod quantities;
mod slots;
mod stages;
mod suggestions;
mod tasks;
mod validation;
mod view;
mod window;
mod zone;

pub use annotations::MAX_ANNOTATION_CHARS;
pub use clearing::Cleared;
pub use coherence::Incoherence;
pub use conflicts::{ConflictDetail, conflict};
pub use error::PlanError;
pub use ids::{AnnotationId, BedId, PlanId, PlantingId};
pub use legacy::upgrade_legacy_json;
pub use model::{
    Annotation, Bed, CellCm, Geometry, Layout, MAX_BEDS, Plan, Plot, TypeAnnotation, TypeBed,
};
pub use model::{MAX_NAME_CHARS, MIN_WIDTH_CM};
pub use placement::{AISLE_CHOICES, MAX_PLOT_CM, MIN_PLOT_CM, check_name};
pub use planting::{Mode, Phases, Planting, PlantingDates};
pub use quantities::{Quantity, quantity};
pub use slots::{MIN_SLOT_DAYS, Slot};
pub use stages::{Appearance, AppearanceChange, Stage};
pub use suggestions::{MAX_SUGGESTIONS, Suggestion, evaluate};
pub use tasks::{Task, TaskType};
pub use validation::Fraction;
pub use view::{
    AnnotationView, BedView, ConflictView, PlantingView, PlotView, SlotView, SuggestionView,
    TaskView, YearView,
};
pub use window::{in_window, window_starting_in};
pub use zone::{Occupation, Zone};
