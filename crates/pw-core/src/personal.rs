//! The personal overlay: favorites, pins, recents, notes, saved links, settings,
//! saved searches, and search history.
//!
//! Nothing here is ever written by a Knowledge Pack apply. That is the guarantee
//! behind "your notes survive an official refresh".

use crate::clock::now_iso;
use crate::error::{CoreError, Result};
use crate::ids::ulid;
use crate::models::*;
use crate::{url, Store};
use rusqlite::params;

// ------------------------------------------------------------ favorites & pins

pub fn set_favorite(store: &Store, item_id: &str, favorite: bool) -> Result<bool> {
    ensure_item_exists(store, item_id)?;
    if favorite {
        store.conn().execute(
            "INSERT OR IGNORE INTO favorites (item_id, created_at) VALUES (?1, ?2)",
            params![item_id, now_iso()],
        )?;
    } else {
        store.conn().execute("DELETE FROM favorites WHERE item_id = ?1", params![item_id])?;
    }
    Ok(favorite)
}

pub fn set_pin(store: &Store, item_id: &str, pinned: bool) -> Result<bool> {
    ensure_item_exists(store, item_id)?;
    if pinned {
        store.conn().execute(
            "INSERT OR IGNORE INTO pins (item_id, sort_order, created_at) VALUES (?1, 100, ?2)",
            params![item_id, now_iso()],
        )?;
    } else {
        store.conn().execute("DELETE FROM pins WHERE item_id = ?1", params![item_id])?;
    }
    Ok(pinned)
}

fn ensure_item_exists(store: &Store, item_id: &str) -> Result<()> {
    let n: i64 = store
        .conn()
        .query_row("SELECT COUNT(*) FROM items WHERE id = ?1", params![item_id], |r| r.get(0))?;
    if n == 0 {
        return Err(CoreError::NotFound(format!("item {item_id}")));
    }
    Ok(())
}

pub fn record_open(store: &Store, item_id: &str) -> Result<()> {
    ensure_item_exists(store, item_id)?;
    store.conn().execute(
        "INSERT INTO recent_items (item_id, opened_at, open_count) VALUES (?1, ?2, 1)
         ON CONFLICT(item_id) DO UPDATE SET opened_at = ?2, open_count = open_count + 1",
        params![item_id, now_iso()],
    )?;
    Ok(())
}

// ------------------------------------------------------------ notes

fn note_from_row(r: &rusqlite::Row) -> rusqlite::Result<Note> {
    Ok(Note {
        id: r.get(0)?,
        item_id: r.get(1)?,
        item_title: r.get(2)?,
        title: r.get(3)?,
        body: r.get(4)?,
        created_at: r.get(5)?,
        updated_at: r.get(6)?,
    })
}

const NOTE_SQL: &str = "SELECT n.id, n.item_id, i.title, n.title, n.body, n.created_at, n.updated_at
     FROM notes n LEFT JOIN items i ON i.id = n.item_id";

pub fn list_notes(store: &Store) -> Result<Vec<Note>> {
    let mut stmt = store.conn().prepare(&format!("{NOTE_SQL} ORDER BY n.updated_at DESC"))?;
    let rows = stmt.query_map([], note_from_row)?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn notes_for_item(store: &Store, item_id: &str) -> Result<Vec<Note>> {
    let mut stmt = store
        .conn()
        .prepare(&format!("{NOTE_SQL} WHERE n.item_id = ?1 ORDER BY n.updated_at DESC"))?;
    let rows = stmt.query_map(params![item_id], note_from_row)?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn save_note(
    store: &Store,
    id: Option<&str>,
    item_id: Option<&str>,
    title: &str,
    body: &str,
) -> Result<Note> {
    if title.trim().is_empty() {
        return Err(CoreError::Invalid("note title is required".into()));
    }
    let now = now_iso();
    let id = match id {
        Some(existing) => {
            let changed = store.conn().execute(
                "UPDATE notes SET item_id = ?2, title = ?3, body = ?4, updated_at = ?5 WHERE id = ?1",
                params![existing, item_id, title.trim(), body, now],
            )?;
            if changed == 0 {
                return Err(CoreError::NotFound(format!("note {existing}")));
            }
            existing.to_string()
        }
        None => {
            let new_id = ulid();
            store.conn().execute(
                "INSERT INTO notes (id, item_id, title, body, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
                params![new_id, item_id, title.trim(), body, now],
            )?;
            new_id
        }
    };
    let mut stmt = store.conn().prepare(&format!("{NOTE_SQL} WHERE n.id = ?1"))?;
    Ok(stmt.query_row(params![id], note_from_row)?)
}

pub fn delete_note(store: &Store, id: &str) -> Result<()> {
    let n = store.conn().execute("DELETE FROM notes WHERE id = ?1", params![id])?;
    if n == 0 {
        return Err(CoreError::NotFound(format!("note {id}")));
    }
    Ok(())
}

// ------------------------------------------------------------ saved links

fn link_from_row(r: &rusqlite::Row) -> rusqlite::Result<SavedLink> {
    Ok(SavedLink {
        id: r.get(0)?,
        title: r.get(1)?,
        url: r.get(2)?,
        description: r.get(3)?,
        tool_id: r.get(4)?,
        tool_name: r.get(5)?,
        category_id: r.get(6)?,
        is_favorite: r.get::<_, i64>(7)? == 1,
        sort_order: r.get(8)?,
        updated_at: r.get(9)?,
    })
}

const LINK_SQL: &str = "SELECT l.id, l.title, l.url, l.description, l.tool_id, t.name,
     l.category_id, l.is_favorite, l.sort_order, l.updated_at
     FROM saved_links l LEFT JOIN tools t ON t.id = l.tool_id";

pub fn list_links(store: &Store) -> Result<Vec<SavedLink>> {
    let mut stmt = store
        .conn()
        .prepare(&format!("{LINK_SQL} ORDER BY l.is_favorite DESC, l.sort_order, l.title"))?;
    let rows = stmt.query_map([], link_from_row)?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn save_link(
    store: &Store,
    id: Option<&str>,
    title: &str,
    raw_url: &str,
    description: Option<&str>,
    tool_id: Option<&str>,
    category_id: Option<&str>,
    is_favorite: bool,
) -> Result<SavedLink> {
    if title.trim().is_empty() {
        return Err(CoreError::Invalid("link title is required".into()));
    }
    // Untrusted input: a saved link is opened in the browser later.
    let safe_url = url::validate_web_url(raw_url)?;
    let now = now_iso();
    let id = match id {
        Some(existing) => {
            let changed = store.conn().execute(
                "UPDATE saved_links SET title = ?2, url = ?3, description = ?4, tool_id = ?5,
                        category_id = ?6, is_favorite = ?7, updated_at = ?8 WHERE id = ?1",
                params![existing, title.trim(), safe_url, description, tool_id, category_id, is_favorite as i64, now],
            )?;
            if changed == 0 {
                return Err(CoreError::NotFound(format!("link {existing}")));
            }
            existing.to_string()
        }
        None => {
            let new_id = ulid();
            store.conn().execute(
                "INSERT INTO saved_links (id, title, url, description, tool_id, category_id,
                        is_favorite, sort_order, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 100, ?8, ?8)",
                params![new_id, title.trim(), safe_url, description, tool_id, category_id, is_favorite as i64, now],
            )?;
            new_id
        }
    };
    let mut stmt = store.conn().prepare(&format!("{LINK_SQL} WHERE l.id = ?1"))?;
    Ok(stmt.query_row(params![id], link_from_row)?)
}

pub fn delete_link(store: &Store, id: &str) -> Result<()> {
    let n = store.conn().execute("DELETE FROM saved_links WHERE id = ?1", params![id])?;
    if n == 0 {
        return Err(CoreError::NotFound(format!("link {id}")));
    }
    Ok(())
}

// ------------------------------------------------------------ settings

pub fn get_setting(store: &Store, key: &str) -> Result<Option<serde_json::Value>> {
    let raw: Option<String> = store
        .conn()
        .query_row("SELECT value_json FROM app_settings WHERE key = ?1", params![key], |r| r.get(0))
        .ok();
    Ok(match raw {
        Some(text) => Some(serde_json::from_str(&text)?),
        None => None,
    })
}

pub fn set_setting(store: &Store, key: &str, value: &serde_json::Value) -> Result<()> {
    store.conn().execute(
        "INSERT INTO app_settings (key, value_json, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value_json = ?2, updated_at = ?3",
        params![key, value.to_string(), now_iso()],
    )?;
    Ok(())
}

pub fn all_settings(store: &Store) -> Result<serde_json::Map<String, serde_json::Value>> {
    let mut stmt = store.conn().prepare("SELECT key, value_json FROM app_settings")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    let mut map = serde_json::Map::new();
    for row in rows {
        let (key, raw) = row?;
        map.insert(key, serde_json::from_str(&raw).unwrap_or(serde_json::Value::Null));
    }
    Ok(map)
}

// ------------------------------------------------------------ searches

pub fn record_search(store: &Store, query: &str, filters: &SearchFilters, result_count: i64) -> Result<()> {
    if query.trim().is_empty() {
        return Ok(());
    }
    store.conn().execute("DELETE FROM search_history WHERE query = ?1", params![query])?;
    store.conn().execute(
        "INSERT INTO search_history (id, query, filters_json, result_count, executed_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![ulid(), query, serde_json::to_string(filters)?, result_count, now_iso()],
    )?;
    // Keep history small; it is a convenience, not an archive.
    store.conn().execute(
        "DELETE FROM search_history WHERE id NOT IN
           (SELECT id FROM search_history ORDER BY executed_at DESC LIMIT 30)",
        [],
    )?;
    Ok(())
}

pub fn search_history(store: &Store, limit: i64) -> Result<Vec<SearchHistoryEntry>> {
    let mut stmt = store.conn().prepare(
        "SELECT id, query, result_count, executed_at FROM search_history
         ORDER BY executed_at DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit], |r| {
        Ok(SearchHistoryEntry {
            id: r.get(0)?,
            query: r.get(1)?,
            result_count: r.get(2)?,
            executed_at: r.get(3)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn clear_search_history(store: &Store) -> Result<()> {
    store.conn().execute("DELETE FROM search_history", [])?;
    Ok(())
}

pub fn list_saved_searches(store: &Store) -> Result<Vec<SavedSearch>> {
    let mut stmt = store.conn().prepare(
        "SELECT id, name, query, filters_json, created_at, last_run_at
         FROM saved_searches ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        let filters: String = r.get(3)?;
        Ok(SavedSearch {
            id: r.get(0)?,
            name: r.get(1)?,
            query: r.get(2)?,
            filters: serde_json::from_str(&filters).unwrap_or_default(),
            created_at: r.get(4)?,
            last_run_at: r.get(5)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn save_search(store: &Store, name: &str, query: &str, filters: &SearchFilters) -> Result<SavedSearch> {
    if name.trim().is_empty() {
        return Err(CoreError::Invalid("saved search needs a name".into()));
    }
    let id = ulid();
    let now = now_iso();
    store.conn().execute(
        "INSERT INTO saved_searches (id, name, query, filters_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(name) DO UPDATE SET query = ?3, filters_json = ?4",
        params![id, name.trim(), query, serde_json::to_string(filters)?, now],
    )?;
    list_saved_searches(store)?
        .into_iter()
        .find(|s| s.name == name.trim())
        .ok_or_else(|| CoreError::NotFound(format!("saved search {name}")))
}

pub fn delete_saved_search(store: &Store, id: &str) -> Result<()> {
    let n = store.conn().execute("DELETE FROM saved_searches WHERE id = ?1", params![id])?;
    if n == 0 {
        return Err(CoreError::NotFound(format!("saved search {id}")));
    }
    Ok(())
}
