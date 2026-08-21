use crate::clock::now_iso;
use crate::error::Result;
use crate::ids::ulid;
use crate::models::AuditEntry;
use crate::Store;
use rusqlite::{params, Connection};

/// Append an audit record. Takes a `Connection` rather than `Store` so it can be
/// called from inside an open transaction during apply.
pub fn record(
    conn: &Connection,
    actor: &str,
    action: &str,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
    batch_id: Option<&str>,
    detail: Option<serde_json::Value>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO audit_log (id, at, actor, action, entity_type, entity_id, batch_id, detail_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            ulid(),
            now_iso(),
            actor,
            action,
            entity_type,
            entity_id,
            batch_id,
            detail.map(|d| d.to_string())
        ],
    )?;
    Ok(())
}

pub fn list(store: &Store, limit: i64) -> Result<Vec<AuditEntry>> {
    let mut stmt = store.conn().prepare(
        "SELECT id, at, actor, action, entity_type, entity_id, batch_id, detail_json
         FROM audit_log ORDER BY at DESC, id DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit], |r| {
        let detail: Option<String> = r.get(7)?;
        Ok(AuditEntry {
            id: r.get(0)?,
            at: r.get(1)?,
            actor: r.get(2)?,
            action: r.get(3)?,
            entity_type: r.get(4)?,
            entity_id: r.get(5)?,
            batch_id: r.get(6)?,
            detail: detail.and_then(|d| serde_json::from_str(&d).ok()),
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}
