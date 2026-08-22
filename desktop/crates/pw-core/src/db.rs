use crate::clock::now_iso;
use crate::error::{CoreError, Result};
use rusqlite::Connection;
use std::path::{Path, PathBuf};

/// Numbered, deterministic migrations. Never edit a released entry: append.
pub const MIGRATIONS: &[(i64, &str, &str)] = &[
    (
        1,
        "001_initial",
        include_str!("../migrations/001_initial.sql"),
    ),
    (
        2,
        "002_update_engine",
        include_str!("../migrations/002_update_engine.sql"),
    ),
    (
        3,
        "003_personal_and_app_state",
        include_str!("../migrations/003_personal_and_app_state.sql"),
    ),
    (
        4,
        "004_search_fts5",
        include_str!("../migrations/004_search_fts5.sql"),
    ),
];

pub fn latest_schema_version() -> i64 {
    MIGRATIONS.last().map(|m| m.0).unwrap_or(0)
}

/// An open knowledge database. Owns the connection; every repository, the search
/// engine, the pack importer, and the CLI all go through this one type so the
/// desktop app and the CLI cannot drift apart.
pub struct Store {
    pub(crate) conn: Connection,
    path: PathBuf,
}

impl Store {
    /// Open (creating if needed) the database at `path` and migrate it forward.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let conn = Connection::open(&path)?;
        let mut store = Store { conn, path };
        store.configure()?;
        store.migrate()?;
        Ok(store)
    }

    /// In-memory store, used by tests and by `doctor --dry-run`.
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let mut store = Store {
            conn,
            path: PathBuf::from(":memory:"),
        };
        store.configure()?;
        store.migrate()?;
        Ok(store)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    fn configure(&mut self) -> Result<()> {
        self.conn.pragma_update(None, "foreign_keys", "ON")?;
        self.conn.pragma_update(None, "busy_timeout", 5000)?;
        // WAL is not available for :memory: databases; ignore the failure there.
        if self.path != Path::new(":memory:") {
            let _ = self.conn.pragma_update(None, "journal_mode", "WAL");
        }
        self.conn.pragma_update(None, "synchronous", "NORMAL")?;
        Ok(())
    }

    /// Apply every migration whose version is above the recorded one, each inside
    /// its own transaction, so a failure leaves the database at the last good version.
    pub fn migrate(&mut self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
               version    INTEGER PRIMARY KEY,
               name       TEXT NOT NULL,
               applied_at TEXT NOT NULL
             );",
        )?;
        let current: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |r| r.get(0),
        )?;

        for (version, name, sql) in MIGRATIONS {
            if *version <= current {
                continue;
            }
            let tx = self.conn.transaction()?;
            tx.execute_batch(sql)
                .map_err(|e| CoreError::Invalid(format!("migration {name} failed: {e}")))?;
            tx.execute(
                "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![version, name, now_iso()],
            )?;
            tx.commit()?;
        }
        Ok(())
    }

    pub fn schema_version(&self) -> Result<i64> {
        Ok(self.conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |r| r.get(0),
        )?)
    }

    pub fn integrity_check(&self) -> Result<String> {
        Ok(self
            .conn
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))?)
    }

    pub fn foreign_key_check(&self) -> Result<usize> {
        let mut stmt = self.conn.prepare("PRAGMA foreign_key_check")?;
        let rows = stmt.query_map([], |_| Ok(()))?;
        Ok(rows.count())
    }

    pub fn count(&self, table: &str) -> Result<i64> {
        // `table` is never user input: callers pass literals from this crate.
        Ok(self
            .conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))?)
    }
}
