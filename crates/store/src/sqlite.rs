//! SQLite store: one row holds the plan serialised in JSON (S05 §5).

use std::path::Path;

use chrono::{DateTime, Utc};
use plan::Plan;
use rusqlite::{Connection, OptionalExtension, params};

use crate::{Store, StoreError};

/// Migrations, in order. Migration `i` takes the schema from version `i` to
/// `i + 1`; the schema version is SQLite's `user_version`. Never edit a
/// released migration: add a new one.
const MIGRATIONS: &[&str] = &[
    // 1: the plan document, a single row.
    "CREATE TABLE plan (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        version INTEGER NOT NULL,
        mis_a_jour_le TEXT NOT NULL,
        document TEXT NOT NULL
    ) STRICT;",
];

/// Schema version this build of the app writes.
pub const SCHEMA_VERSION: i64 = MIGRATIONS.len() as i64;

/// The SQLite implementation of [`Store`].
#[derive(Debug)]
pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    /// Opens (or creates) the database at `path` and migrates it.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        Self::init(Connection::open(path)?)
    }

    /// An in-memory database, for tests.
    pub fn open_in_memory() -> Result<Self, StoreError> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(mut conn: Connection) -> Result<Self, StoreError> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        migrate(&mut conn)?;
        Ok(Self { conn })
    }

    /// Current schema version of the open database.
    pub fn schema_version(&self) -> Result<i64, StoreError> {
        Ok(self
            .conn
            .pragma_query_value(None, "user_version", |row| row.get(0))?)
    }

    fn stored_version(&self) -> Result<Option<u64>, StoreError> {
        Ok(self
            .conn
            .query_row("SELECT version FROM plan WHERE id = 1", [], |row| {
                row.get::<_, i64>(0)
            })
            .optional()?
            .map(|v| u64::try_from(v).unwrap_or(0)))
    }

    fn write(&self, plan: &Plan) -> Result<(), StoreError> {
        let document = serde_json::to_string(plan)?;
        self.conn.execute(
            "INSERT INTO plan (id, version, mis_a_jour_le, document) VALUES (1, ?1, ?2, ?3)
             ON CONFLICT (id) DO UPDATE SET
                version = excluded.version,
                mis_a_jour_le = excluded.mis_a_jour_le,
                document = excluded.document",
            params![
                i64::try_from(plan.version).unwrap_or(i64::MAX),
                plan.mis_a_jour_le.to_rfc3339(),
                document
            ],
        )?;
        Ok(())
    }
}

fn migrate(conn: &mut Connection) -> Result<(), StoreError> {
    let current: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if current > SCHEMA_VERSION {
        return Err(StoreError::SchemaTooNew {
            found: current,
            supported: SCHEMA_VERSION,
        });
    }
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(current as usize) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", index as i64 + 1)?;
        tx.commit()?;
    }
    Ok(())
}

impl Store for SqliteStore {
    fn load(&self) -> Result<Option<Plan>, StoreError> {
        let document: Option<String> = self
            .conn
            .query_row("SELECT document FROM plan WHERE id = 1", [], |row| {
                row.get(0)
            })
            .optional()?;
        Ok(document
            .map(|json| serde_json::from_str(&json))
            .transpose()?)
    }

    fn save(&mut self, plan: &mut Plan, now: DateTime<Utc>) -> Result<(), StoreError> {
        match self.stored_version()? {
            None => self.write(plan),
            Some(stored) if stored != plan.version => Err(StoreError::StaleVersion {
                given: plan.version,
                stored,
            }),
            Some(_) => {
                let mut next = plan.clone();
                next.touch(now);
                self.write(&next)?;
                *plan = next;
                Ok(())
            }
        }
    }

    fn replace(&mut self, plan: &Plan) -> Result<(), StoreError> {
        self.write(plan)
    }

    fn clear(&mut self) -> Result<(), StoreError> {
        self.conn.execute("DELETE FROM plan", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, TimeZone};
    use plan::{CaseCm, Geometry, PlanId, PlancheId, Terrain, TypePlanche};

    use super::*;

    fn at(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2027, 3, 1, hour, 0, 0).unwrap()
    }

    fn new_plan() -> Plan {
        Plan::new(
            PlanId::generate(),
            Terrain {
                longueur_cm: 1100,
                largeur_cm: 650,
                case_cm: CaseCm::C50,
                allee_cm: 50,
            },
            at(8),
        )
    }

    fn add_planche(plan: &mut Plan) {
        plan.add_planche(
            PlancheId::generate(),
            "Planche A".into(),
            2027,
            Geometry {
                planche_type: TypePlanche::Planche,
                x: 50,
                y: 50,
                longueur_cm: 300,
                largeur_cm: 120,
            },
            NaiveDate::from_ymd_opt(2027, 3, 1).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn empty_database_has_no_plan() {
        let store = SqliteStore::open_in_memory().unwrap();
        assert_eq!(store.load().unwrap(), None);
        assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
    }

    #[test]
    fn first_save_inserts_then_saves_bump_the_version() {
        let mut store = SqliteStore::open_in_memory().unwrap();
        let mut plan = new_plan();
        store.save(&mut plan, at(9)).unwrap();
        assert_eq!(plan.version, 1);
        assert_eq!(store.load().unwrap().unwrap(), plan);

        add_planche(&mut plan);
        store.save(&mut plan, at(10)).unwrap();
        assert_eq!(plan.version, 2);
        assert_eq!(plan.mis_a_jour_le, at(10));
        assert_eq!(store.load().unwrap().unwrap(), plan);
    }

    #[test]
    fn a_stale_copy_is_refused_and_changes_nothing() {
        let mut store = SqliteStore::open_in_memory().unwrap();
        let mut plan = new_plan();
        store.save(&mut plan, at(9)).unwrap();
        let mut stale = plan.clone();
        store.save(&mut plan, at(10)).unwrap();

        add_planche(&mut stale);
        let err = store.save(&mut stale, at(11)).unwrap_err();
        assert!(matches!(
            err,
            StoreError::StaleVersion {
                given: 1,
                stored: 2
            }
        ));
        assert_eq!(store.load().unwrap().unwrap(), plan);
    }

    #[test]
    fn data_survives_reopening_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("potagerie.db");
        let mut plan = new_plan();
        add_planche(&mut plan);
        {
            let mut store = SqliteStore::open(&path).unwrap();
            store.save(&mut plan, at(9)).unwrap();
        }
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.load().unwrap().unwrap(), plan);
        assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
    }

    #[test]
    fn replace_and_clear() {
        let mut store = SqliteStore::open_in_memory().unwrap();
        let mut plan = new_plan();
        store.save(&mut plan, at(9)).unwrap();

        let mut imported = new_plan();
        imported.version = 42;
        store.replace(&imported).unwrap();
        assert_eq!(store.load().unwrap().unwrap(), imported);

        store.clear().unwrap();
        assert_eq!(store.load().unwrap(), None);
    }

    #[test]
    fn a_newer_schema_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("potagerie.db");
        {
            let conn = Connection::open(&path).unwrap();
            conn.pragma_update(None, "user_version", SCHEMA_VERSION + 1)
                .unwrap();
        }
        let err = SqliteStore::open(&path).unwrap_err();
        assert!(matches!(err, StoreError::SchemaTooNew { .. }));
    }
}
