use crate::error::Result;
use crate::models::Diagnostics;
use crate::{backup, db, repo, search, staging, Store};

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Everything the Diagnostics screen and `personal-wiki doctor` report.
pub fn collect(store: &Store) -> Result<Diagnostics> {
    let schema_version = store.schema_version()?;
    let expected = db::latest_schema_version();
    let integrity = store.integrity_check().unwrap_or_else(|e| format!("error: {e}"));
    let fk_violations = store.foreign_key_check().unwrap_or(0) as i64;
    let (fts_status, fts_rows) = search::health(store.conn());
    let counts = repo::table_counts(store)?;
    let db_size = std::fs::metadata(store.path()).map(|m| m.len() as i64).unwrap_or(0);

    let mut problems = Vec::new();
    if integrity != "ok" {
        problems.push(format!("database integrity check returned: {integrity}"));
    }
    if fk_violations > 0 {
        problems.push(format!("{fk_violations} foreign key violation(s)"));
    }
    if schema_version != expected {
        problems.push(format!(
            "schema version {schema_version} does not match the expected {expected}; run the app once to migrate"
        ));
    }
    if fts_status != "ok" {
        problems.push(format!(
            "search index is {fts_status} ({fts_rows} indexed vs {} items); rebuild it from Settings or run `personal-wiki doctor --repair`",
            counts.get("items").copied().unwrap_or(0)
        ));
    }
    if counts.get("tools").copied().unwrap_or(0) == 0 {
        problems.push("no tools yet: import a Knowledge Pack to get started".into());
    }

    Ok(Diagnostics {
        app_version: APP_VERSION.to_string(),
        schema_version,
        expected_schema_version: expected,
        db_path: store.path().to_string_lossy().to_string(),
        db_size_bytes: db_size,
        integrity,
        foreign_key_violations: fk_violations,
        fts_status,
        fts_row_count: fts_rows,
        counts,
        last_batch: staging::list_batches(store, 1)?.into_iter().next(),
        last_backup: backup::list(store)?.into_iter().next(),
        problems,
    })
}

/// Fix what can be fixed safely: currently the search index.
pub fn repair(store: &Store) -> Result<Vec<String>> {
    let mut done = Vec::new();
    let (status, _) = search::health(store.conn());
    if status != "ok" {
        let n = search::rebuild(store.conn())?;
        done.push(format!("rebuilt the search index ({n} items)"));
    }
    Ok(done)
}
