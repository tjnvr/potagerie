//! Conflicts between two series (S03 §5).

use crate::serie::Serie;
use crate::zone::{Occupation, Zone};

/// What two conflicting series have in common.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConflictDetail {
    /// Troncons both series occupy.
    pub troncons: Zone,
    /// `[max(a.m, b.m), min(a.f, b.f)[`.
    pub period: Occupation,
}

/// Two series of the same planche conflict if their zones overlap and their
/// half-open occupations overlap. A serie may start the day the previous one ends.
pub fn conflict(a: &Serie, b: &Serie) -> Option<ConflictDetail> {
    if a.planche_id != b.planche_id {
        return None;
    }
    Some(ConflictDetail {
        troncons: a.zone().intersection(&b.zone())?,
        period: a.occupation().intersection(&b.occupation())?,
    })
}
