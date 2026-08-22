//! Snapshot / restore. A backup is a real SQLite copy made with the online backup
//! API, so it is consistent even while the app is running.

use crate::clock::now_iso;
use crate::error::{CoreError, Result};
use crate::ids::ulid;
use crate::models::BackupRecord;
use crate::{audit, paths, Store};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

fn backup_dir_for(store: &Store) -> PathBuf {
    let db = store.path();
    if db == Path::new(":memory:") {
        return paths::backups_dir();
    }
    db.parent()
        .map(|p| p.join("backups"))
        .unwrap_or_else(paths::backups_dir)
}

/// Create a snapshot and register it. `reason` explains why it exists
/// ("pre_apply", "manual", "pre_restore") and shows up in the UI.
pub fn create(store: &Store, reason: &str, batch_id: Option<&str>) -> Result<BackupRecord> {
    let dir = backup_dir_for(store);
    std::fs::create_dir_all(&dir)?;
    let id = ulid();
    let file = dir.join(format!("personal-wiki-{id}.db"));

    {
        let mut dest = Connection::open(&file)?;
        let backup = rusqlite::backup::Backup::new(store.conn(), &mut dest)?;
        backup.run_to_completion(64, std::time::Duration::from_millis(10), None)?;
    }

    let size = std::fs::metadata(&file)
        .map(|m| m.len() as i64)
        .unwrap_or(0);
    let schema_version = store.schema_version()?;
    let item_count = store.count("items")?;
    let record = BackupRecord {
        id: id.clone(),
        path: file.to_string_lossy().to_string(),
        reason: reason.to_string(),
        batch_id: batch_id.map(|s| s.to_string()),
        size_bytes: size,
        schema_version,
        item_count,
        created_at: now_iso(),
    };
    store.conn().execute(
        "INSERT INTO backups (id, path, reason, batch_id, size_bytes, schema_version, item_count, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            record.id, record.path, record.reason, record.batch_id,
            record.size_bytes, record.schema_version, record.item_count, record.created_at
        ],
    )?;
    audit::record(
        store.conn(),
        "system",
        "backup.create",
        Some("backup"),
        Some(&record.id),
        batch_id,
        Some(serde_json::json!({ "reason": reason, "path": record.path })),
    )?;
    Ok(record)
}

pub fn list(store: &Store) -> Result<Vec<BackupRecord>> {
    let mut stmt = store.conn().prepare(
        "SELECT id, path, reason, batch_id, size_bytes, schema_version, item_count, created_at
         FROM backups ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(BackupRecord {
            id: r.get(0)?,
            path: r.get(1)?,
            reason: r.get(2)?,
            batch_id: r.get(3)?,
            size_bytes: r.get(4)?,
            schema_version: r.get(5)?,
            item_count: r.get(6)?,
            created_at: r.get(7)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn get(store: &Store, id: &str) -> Result<BackupRecord> {
    list(store)?
        .into_iter()
        .find(|b| b.id == id)
        .ok_or_else(|| CoreError::NotFound(format!("backup {id}")))
}

/// Restore a snapshot over the live database.
///
/// A safety snapshot of the *current* state is taken first, so restoring is
/// itself undoable. The restore replaces every page of the live database in
/// place; open connections keep working.
pub fn restore(store: &mut Store, id: &str) -> Result<BackupRecord> {
    let record = get(store, id)?;
    let src_path = PathBuf::from(&record.path);
    if !src_path.exists() {
        return Err(CoreError::NotFound(format!(
            "backup file missing: {}",
            record.path
        )));
    }
    let safety = create(store, "pre_restore", None)?;

    let src = Connection::open(&src_path)?;
    {
        let backup = rusqlite::backup::Backup::new(&src, store.conn_mut())?;
        backup.run_to_completion(64, std::time::Duration::from_millis(10), None)?;
    }
    store.migrate()?;
    audit::record(
        store.conn(),
        "user",
        "backup.restore",
        Some("backup"),
        Some(&record.id),
        record.batch_id.as_deref(),
        Some(serde_json::json!({ "safety_backup": safety.id })),
    )?;
    Ok(record)
}
