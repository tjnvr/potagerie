//! SQLite store: one row holds the plan serialised in JSON (S05 §5).

use std::path::Path;

use chrono::{DateTime, Utc};
use plan::Plan;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

use crate::{Store, StoreError};

/// Migrations, in order. Migration `i` takes the schema from version `i` to
/// `i + 1`; the schema version is SQLite's `user_version`. Never edit a
/// released migration: add a new one.
const MIGRATIONS: &[&str] = &[
    // 1: the plan document, a single row. `IF NOT EXISTS`: a database written
    // before the schema was versioned (`user_version` 0) already has it.
    "CREATE TABLE IF NOT EXISTS plan (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        version INTEGER NOT NULL,
        mis_a_jour_le TEXT NOT NULL,
        document TEXT NOT NULL
    ) STRICT;",
    // 2: the column is named in English. The documents of version 1 keep the
    // French field names until they are next saved; `load` upgrades them.
    "ALTER TABLE plan RENAME COLUMN mis_a_jour_le TO updated_at;",
];

/// How long a write waits for another connection (a second instance of the
/// app on the same file) to release its lock before failing with `SQLITE_BUSY`.
const BUSY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

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
        // First: switching to WAL and migrating already take locks.
        conn.busy_timeout(BUSY_TIMEOUT)?;
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
}

fn stored_version(conn: &Connection) -> Result<Option<u64>, StoreError> {
    Ok(conn
        .query_row("SELECT version FROM plan WHERE id = 1", [], |row| {
            row.get::<_, i64>(0)
        })
        .optional()?
        .map(|v| u64::try_from(v).unwrap_or(0)))
}

fn write(conn: &Connection, plan: &Plan) -> Result<(), StoreError> {
    let document = serde_json::to_string(plan)?;
    conn.execute(
        "INSERT INTO plan (id, version, updated_at, document) VALUES (1, ?1, ?2, ?3)
         ON CONFLICT (id) DO UPDATE SET
            version = excluded.version,
            updated_at = excluded.updated_at,
            document = excluded.document",
        params![
            i64::try_from(stored_form(plan.version)).unwrap_or(i64::MAX),
            plan.updated_at.to_rfc3339(),
            document
        ],
    )?;
    Ok(())
}

/// The version as the `version` column keeps it (an SQLite integer is signed,
/// so versions beyond `i64::MAX` are clamped): `save` compares in this form.
fn stored_form(version: u64) -> u64 {
    version.min(i64::MAX as u64)
}

fn migrate(conn: &mut Connection) -> Result<(), StoreError> {
    let current: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if current < 0 {
        // Not a version this app wrote: unreadable, not "already migrated".
        return Err(StoreError::Database(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CORRUPT),
            Some(format!("negative schema version {current}")),
        )));
    }
    if current > SCHEMA_VERSION {
        return Err(StoreError::SchemaTooNew {
            found: current,
            supported: SCHEMA_VERSION,
        });
    }
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(current as usize) {
        // Two instances may start together: the first takes the write lock, the
        // second waits, then finds the migration done.
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let done: i64 = tx.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if done > index as i64 {
            continue;
        }
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
        document
            .map(|json| {
                let value: serde_json::Value = serde_json::from_str(&json)?;
                Ok(serde_json::from_value(plan::upgrade_legacy_json(value))?)
            })
            .transpose()
    }

    fn save(&mut self, plan: &mut Plan, now: DateTime<Utc>) -> Result<(), StoreError> {
        // IMMEDIATE: the write lock is taken before the version is read, so
        // the check and the write are one step even with another instance
        // on the same file; the loser waits (busy timeout) and is then refused.
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        match stored_version(&tx)? {
            None => write(&tx, plan)?,
            Some(stored) if stored != stored_form(plan.version) => {
                return Err(StoreError::StaleVersion {
                    given: plan.version,
                    stored,
                });
            }
            Some(_) => {
                let mut next = plan.clone();
                next.touch(now);
                write(&tx, &next)?;
                tx.commit()?;
                *plan = next;
                return Ok(());
            }
        }
        tx.commit()?;
        Ok(())
    }

    fn replace(&mut self, plan: &Plan) -> Result<(), StoreError> {
        write(&self.conn, plan)
    }

    fn clear(&mut self) -> Result<(), StoreError> {
        self.conn.execute("DELETE FROM plan", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, TimeZone};
    use plan::{BedId, CellCm, Geometry, PlanId, Plot, TypeBed};

    use super::*;

    fn at(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2027, 3, 1, hour, 0, 0).unwrap()
    }

    fn new_plan() -> Plan {
        Plan::new(
            PlanId::generate(),
            Plot {
                length_cm: 1100,
                width_cm: 650,
                cell_cm: CellCm::C50,
                aisle_cm: 50,
            },
            at(8),
        )
    }

    fn add_bed(plan: &mut Plan) {
        add_bed_named(plan, "Bed A", 50);
    }

    fn add_bed_named(plan: &mut Plan, name: &str, x: u32) {
        plan.add_bed(
            BedId::generate(),
            name.into(),
            2027,
            Geometry {
                bed_type: TypeBed::Bed,
                x,
                y: 50,
                length_cm: 300,
                width_cm: 120,
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

        add_bed(&mut plan);
        store.save(&mut plan, at(10)).unwrap();
        assert_eq!(plan.version, 2);
        assert_eq!(plan.updated_at, at(10));
        assert_eq!(store.load().unwrap().unwrap(), plan);
    }

    #[test]
    fn a_stale_copy_is_refused_and_changes_nothing() {
        let mut store = SqliteStore::open_in_memory().unwrap();
        let mut plan = new_plan();
        store.save(&mut plan, at(9)).unwrap();
        let mut stale = plan.clone();
        store.save(&mut plan, at(10)).unwrap();

        add_bed(&mut stale);
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
        add_bed(&mut plan);
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

    /// A database written by app version 0.2 (schema 1, French column and
    /// field names) opens, reads and is saved again in the current format.
    #[test]
    fn a_database_of_version_0_2_is_upgraded() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("potagerie.db");
        let former = r#"{
            "id": "0192d5e0-0000-7000-8000-000000000001", "version": 4,
            "mis_a_jour_le": "2027-06-01T10:00:00Z",
            "terrain": {"longueur_cm": 900, "largeur_cm": 650, "case_cm": 50, "allee_cm": 50},
            "planches": {"0192d5e0-0000-7000-8000-0000000000a1": {"nom": "A",
              "dispositions": {"2027": {"etat": "presente", "type": "planche", "x": 50, "y": 50, "longueur_cm": 300, "largeur_cm": 120}}}},
            "series": {}, "annotations": {}
        }"#;
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(MIGRATIONS[0]).unwrap();
            conn.pragma_update(None, "user_version", 1).unwrap();
            conn.execute(
                "INSERT INTO plan (id, version, mis_a_jour_le, document) VALUES (1, 4, '2027-06-01T10:00:00Z', ?1)",
                [former],
            )
            .unwrap();
        }
        let mut store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
        let mut plan = store.load().unwrap().unwrap();
        assert_eq!(plan.version, 4);
        assert_eq!(plan.beds.len(), 1);
        store.save(&mut plan, at(10)).unwrap();
        assert_eq!(store.load().unwrap().unwrap().version, 5);
    }

    /// Two connections on one file save from the same version at the same
    /// time: exactly one wins, the other is refused with `StaleVersion`, and
    /// nobody sees `SQLITE_BUSY`.
    #[test]
    fn concurrent_saves_from_two_connections_let_one_win() {
        use std::sync::{Arc, Barrier};

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("potagerie.db");
        let mut first = SqliteStore::open(&path).unwrap();
        let mut plan = new_plan();
        first.save(&mut plan, at(9)).unwrap();
        drop(first);

        for round in 0..20u64 {
            let barrier = Arc::new(Barrier::new(2));
            let handles: Vec<_> = (0..2u32)
                .map(|i| {
                    let (path, barrier) = (path.clone(), barrier.clone());
                    let mut store = SqliteStore::open(&path).unwrap();
                    let mut copy = store.load().unwrap().unwrap();
                    copy.plot.length_cm = 2000 + i; // marks which copy was saved
                    std::thread::spawn(move || {
                        barrier.wait();
                        store.save(&mut copy, at(10)).map(|()| copy.version)
                    })
                })
                .collect();
            let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
            let wins = results.iter().filter(|r| r.is_ok()).count();
            let stale = results
                .iter()
                .filter(|r| matches!(r, Err(StoreError::StaleVersion { .. })))
                .count();
            assert_eq!((wins, stale), (1, 1), "round {round}: {results:?}");
            let stored = SqliteStore::open(&path).unwrap().load().unwrap().unwrap();
            assert_eq!(stored.version, round + 2);
            // The stored plan is the winner's, whole.
            let winner = results.iter().position(|r| r.is_ok()).unwrap();
            assert_eq!(stored.plot.length_cm, 2000 + winner as u32);
        }
    }

    /// A database written before the schema was versioned (`user_version` 0,
    /// table already there) is adopted, not re-created, and keeps its data.
    #[test]
    fn a_database_from_before_versioning_is_adopted() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("potagerie.db");
        let mut plan = new_plan();
        plan.version = 3;
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                "CREATE TABLE plan (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    version INTEGER NOT NULL,
                    mis_a_jour_le TEXT NOT NULL,
                    document TEXT NOT NULL
                ) STRICT;",
            )
            .unwrap();
            conn.execute(
                "INSERT INTO plan (id, version, mis_a_jour_le, document) VALUES (1, 3, ?1, ?2)",
                params![
                    plan.updated_at.to_rfc3339(),
                    serde_json::to_string(&plan).unwrap()
                ],
            )
            .unwrap();
            assert_eq!(
                conn.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
                    .unwrap(),
                0
            );
        }
        let mut store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
        assert_eq!(store.load().unwrap().unwrap(), plan);
        store.save(&mut plan, at(10)).unwrap();
        assert_eq!(plan.version, 4);
    }
}
