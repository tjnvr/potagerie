//! Potagerie plan: data model (S05 §5) and calculations (S03).
//!
//! The crate has no clock, no storage and no UI. Plan dates are civil
//! (`chrono::NaiveDate`); "today" is always a parameter; technical timestamps
//! are UTC. Nothing derived is stored in a [`Plan`]: dates, conflicts, slots,
//! suggestions and tasks are recomputed from it.

mod conflits;
mod creneaux;
mod dispositions;
mod error;
mod fenetre;
mod ids;
mod jours;
mod model;
mod operations;
mod quantites;
mod serie;
mod stades;
mod suggestions;
mod taches;
mod validation;
mod zone;

pub use conflits::{DetailConflit, conflit};
pub use creneaux::{Creneau, DUREE_MIN_CRENEAU};
pub use error::PlanError;
pub use fenetre::{dans_fenetre, fenetre_commencant_en};
pub use ids::{AnnotationId, PlanId, PlancheId, SerieId};
pub use model::{
    Annotation, CaseCm, Disposition, Geometrie, MAX_PLANCHES, Plan, Planche, Terrain,
    TypeAnnotation, TypePlanche,
};
pub use quantites::{Quantite, quantite};
pub use serie::{DatesSerie, Mode, Phases, Serie};
pub use stades::{Affichage, Stade};
pub use suggestions::{MAX_SUGGESTIONS, Suggestion, evaluer};
pub use taches::{Tache, TypeTache};
pub use validation::Fraction;
pub use zone::{Occupation, Zone};
