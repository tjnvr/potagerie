//! Potagerie storage (S05 §5).
//!
//! The plan is one document. In lots 1 and 2 it is stored as JSON in a single
//! SQLite row ([`SqliteStore`]); in lot 3 a CRDT store will implement the same
//! [`Store`] trait.

mod sqlite;

pub use sqlite::{SCHEMA_VERSION, SqliteStore};

use chrono::{DateTime, Utc};
use plan::Plan;
use thiserror::Error;

/// Loads and saves the plan of the device.
pub trait Store {
    /// The stored plan, or `None` before the plot is created.
    fn load(&self) -> Result<Option<Plan>, StoreError>;

    /// Saves a modified plan: bumps `version` and sets `updated_at` to
    /// `now`, then writes it (S05 §7).
    ///
    /// The first save inserts the plan as it is. Later saves are refused with
    /// [`StoreError::StaleVersion`] if `plan.version` is not the stored one, so
    /// a write based on an outdated copy never overwrites a newer one.
    fn save(&mut self, plan: &mut Plan, now: DateTime<Utc>) -> Result<(), StoreError>;

    /// Replaces the stored plan by `plan`, as it is (import, F-CPT-3).
    fn replace(&mut self, plan: &Plan) -> Result<(), StoreError>;

    /// Deletes every data of the garden (F-CPT-4).
    fn clear(&mut self) -> Result<(), StoreError>;
}

/// A storage failure.
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("stored document is unreadable: {0}")]
    Document(#[from] serde_json::Error),
    #[error("database schema {found} is newer than this app ({supported})")]
    SchemaTooNew { found: i64, supported: i64 },
    #[error("stale plan: version {given} was saved over version {stored}")]
    StaleVersion { given: u64, stored: u64 },
}

impl StoreError {
    /// The database file itself is damaged (`SQLITE_CORRUPT`, `SQLITE_NOTADB`)
    /// or holds an unreadable document: retrying cannot help.
    pub fn is_damaged(&self) -> bool {
        use rusqlite::ErrorCode::{DatabaseCorrupt, NotADatabase};
        match self {
            Self::Document(_) => true,
            Self::Database(rusqlite::Error::SqliteFailure(e, _)) => {
                matches!(e.code, DatabaseCorrupt | NotADatabase)
            }
            _ => false,
        }
    }
}
