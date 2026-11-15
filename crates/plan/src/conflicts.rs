//! Conflicts between two plantings (S03 §5).

use crate::planting::Planting;
use crate::zone::{Occupation, Zone};

/// What two conflicting plantings have in common.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConflictDetail {
    /// Segments both plantings occupy.
    pub segments: Zone,
    /// `[max(a.m, b.m), min(a.f, b.f)[`.
    pub period: Occupation,
}

/// Two plantings of the same bed conflict if their zones overlap and their
/// half-open occupations overlap. A planting may start the day the previous one ends.
pub fn conflict(a: &Planting, b: &Planting) -> Option<ConflictDetail> {
    if a.bed_id != b.bed_id {
        return None;
    }
    Some(ConflictDetail {
        segments: a.zone().intersection(&b.zone())?,
        period: a.occupation().intersection(&b.occupation())?,
    })
}
