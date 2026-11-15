//! Potagerie plan: data model (S05 §5) and calculations (S03).
//!
//! The crate has no clock, no storage and no UI. Plan dates are civil
//! (`chrono::NaiveDate`); "today" is always a parameter; technical timestamps
//! are UTC. Nothing derived is stored in a [`Plan`]: dates, conflicts, slots,
//! suggestions and tasks are recomputed from it.

mod conflicts;
mod creneaux;
mod days;
mod dispositions;
mod error;
mod ids;
mod model;
mod operations;
mod placement;
mod quantities;
mod serie;
mod stages;
mod suggestions;
mod tasks;
mod validation;
mod view;
mod window;
mod zone;

pub use conflicts::{ConflictDetail, conflict};
pub use creneaux::{Creneau, MIN_CRENEAU_DAYS};
pub use error::PlanError;
pub use ids::{AnnotationId, PlanId, PlancheId, SerieId};
pub use model::{
    Annotation, CaseCm, Disposition, Geometry, MAX_PLANCHES, Plan, Planche, Terrain,
    TypeAnnotation, TypePlanche,
};
pub use model::{MAX_NAME_CHARS, MIN_WIDTH_CM};
pub use placement::{ALLEE_CHOICES, MAX_TERRAIN_CM, MIN_TERRAIN_CM, check_name};
pub use quantities::{Quantity, quantity};
pub use serie::{Mode, Phases, Serie, SerieDates};
pub use stages::{Appearance, AppearanceChange, Stage};
pub use suggestions::{MAX_SUGGESTIONS, Suggestion, evaluate};
pub use tasks::{Task, TaskType};
pub use validation::Fraction;
pub use view::{
    AnnotationView, ConflictView, CreneauView, PlancheView, SerieView, SuggestionView, TaskView,
    TerrainView, YearView,
};
pub use window::{in_window, window_starting_in};
pub use zone::{Occupation, Zone};
