//! Staging and diffing.
//!
//! A validated pack never touches trusted rows. It is turned into a batch of
//! `staged_changes`, one row per human-meaningful entity: a tool, a task, a
//! category, a tag, a source, or a whole item (its aliases, examples, parameters,
//! tags, task/category links and relationships travel inside the item payload,
//! because that is the unit a reviewer actually approves).
//!
//! Each staged row carries the complete canonical `after_json`, so applying a
//! batch later needs only the database — never the original file.

use crate::clock::now_iso;
use crate::error::Result;
use crate::ids::ulid;
use crate::models::{FieldDiff, StagedChange, UpdateBatch};
use crate::pack::{KnowledgePack, PackItem};
use crate::{audit, Store};
use rusqlite::{params, Connection};
use serde_json::{json, Value};

/// Canonical JSON for an entity, used both for storage and for diffing.
fn tool_json(t: &crate::pack::PackTool) -> Value {
    json!({
        "slug": t.slug, "name": t.name, "shortName": t.short_name,
        "description": t.description, "iconKey": t.icon_key.clone().unwrap_or_else(|| "book".into()),
        "accentKey": t.accent_key.clone().unwrap_or_else(|| "blue".into()),
        "officialHomeUrl": t.official_home_url, "sortOrder": t.sort_order.unwrap_or(100),
        "externalKey": t.external_key.clone().unwrap_or_else(|| format!("tool:{}", t.slug)),
    })
}

fn task_json(t: &crate::pack::PackTask) -> Value {
    json!({
        "slug": t.slug, "name": t.name, "description": t.description,
        "iconKey": t.icon_key.clone().unwrap_or_else(|| "target".into()),
        "sortOrder": t.sort_order.unwrap_or(100),
        "externalKey": t.external_key.clone().unwrap_or_else(|| format!("task:{}", t.slug)),
    })
}

fn category_json(c: &crate::pack::PackCategory) -> Value {
    json!({
        "slug": c.slug, "name": c.name, "description": c.description,
        "iconKey": c.icon_key.clone().unwrap_or_else(|| "folder".into()),
        "accentKey": c.accent_key.clone().unwrap_or_else(|| "blue".into()),
        "parentSlug": c.parent_slug, "sortOrder": c.sort_order.unwrap_or(100),
        "externalKey": c.external_key.clone().unwrap_or_else(|| format!("category:{}", c.slug)),
    })
}

fn tag_json(t: &crate::pack::PackTag) -> Value {
    json!({ "slug": t.slug, "name": t.name, "accentKey": t.accent_key.clone().unwrap_or_else(|| "blue".into()) })
}

fn source_json(s: &crate::pack::PackSource) -> Value {
    json!({
        "key": s.key, "name": s.name, "sourceType": s.source_type, "url": s.url,
        "isOfficial": s.is_official.unwrap_or(false), "toolSlug": s.tool_slug,
        "syncStrategy": s.sync_strategy.clone().unwrap_or_else(|| "agent".into()),
        "notes": s.notes,
    })
}

fn item_json(i: &PackItem) -> Value {
    json!({
        "externalKey": i.external_key,
        "slug": i.slug,
        "toolSlug": i.tool_slug,
        "itemType": i.item_type,
        "title": i.title,
        "subtitle": i.subtitle,
        "shortDescription": i.short_description,
        "fullDescription": i.full_description,
        "syntax": i.syntax,
        "whenToUse": i.when_to_use,
        "expectedResult": i.expected_result,
        "warningText": i.warning_text,
        "resultDiagram": i.result_diagram,
        "verificationStatus": i.verification_status.clone().unwrap_or_else(|| "unverified".into()),
        "lifecycleStatus": i.lifecycle_status.clone().unwrap_or_else(|| "active".into()),
        "difficulty": i.difficulty,
        "iconKey": i.icon_key,
        "accentKey": i.accent_key,
        "sortOrder": i.sort_order.unwrap_or(100),
        "lastVerifiedAt": i.last_verified_at,
        "aliases": i.aliases,
        "tasks": i.tasks,
        "categories": i.categories,
        "tags": i.tags,
        "examples": i.examples.iter().map(|e| json!({
            "title": e.title, "description": e.description, "code": e.code,
            "expectedOutput": e.expected_output, "sortOrder": e.sort_order.unwrap_or(100)
        })).collect::<Vec<_>>(),
        "parameters": i.parameters.iter().map(|p| json!({
            "name": p.name, "description": p.description,
            "isRequired": p.is_required.unwrap_or(false),
            "defaultValue": p.default_value, "valueHint": p.value_hint,
            "sortOrder": p.sort_order.unwrap_or(100)
        })).collect::<Vec<_>>(),
        "sources": i.sources.iter().map(|s| json!({
            "sourceKey": s.source_key, "url": s.url, "anchor": s.anchor,
            "version": s.version, "verifiedAt": s.verified_at
        })).collect::<Vec<_>>(),
        "relationships": i.relationships.iter().map(|r| json!({
            "toExternalKey": r.to_external_key, "relationshipType": r.relationship_type
        })).collect::<Vec<_>>(),
    })
}

/// Rebuild the canonical JSON of what is *currently* stored, so the reviewer sees
/// a real before/after and unchanged entities produce no staged row at all.
fn current_item_json(conn: &Connection, external_key: &str) -> Result<Option<Value>> {
    let row: Option<(
        String,
        String,
        Option<String>,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        i64,
        Option<String>,
    )> = conn
        .query_row(
            "SELECT i.id, i.slug, t.slug, i.item_type, i.title, i.subtitle, i.short_description,
                    i.full_description, i.syntax, i.when_to_use, i.expected_result, i.warning_text,
                    i.result_diagram, i.verification_status, i.lifecycle_status, i.difficulty,
                    i.icon_key, i.accent_key, i.sort_order, i.last_verified_at
             FROM items i LEFT JOIN tools t ON t.id = i.tool_id
             WHERE i.external_key = ?1",
            params![external_key],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                    r.get(7)?,
                    r.get(8)?,
                    r.get(9)?,
                    r.get(10)?,
                    r.get(11)?,
                    r.get(12)?,
                    r.get(13)?,
                    r.get(14)?,
                    r.get(15)?,
                    r.get(16)?,
                    r.get(17)?,
                    r.get(18)?,
                    r.get(19)?,
                ))
            },
        )
        .ok();
    let Some(row) = row else { return Ok(None) };
    let id = row.0;

    let strings = |sql: &str| -> Result<Vec<String>> {
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params![id], |r| r.get::<_, String>(0))?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    };
    let aliases = strings("SELECT alias FROM aliases WHERE item_id = ?1 ORDER BY alias")?;
    let tasks = strings("SELECT t.slug FROM tasks t JOIN item_tasks it ON it.task_id = t.id WHERE it.item_id = ?1 ORDER BY t.slug")?;
    let categories = strings("SELECT c.slug FROM categories c JOIN item_categories ic ON ic.category_id = c.id WHERE ic.item_id = ?1 ORDER BY c.slug")?;
    let tags = strings("SELECT tg.slug FROM tags tg JOIN item_tags it ON it.tag_id = tg.id WHERE it.item_id = ?1 ORDER BY tg.slug")?;

    let mut stmt = conn.prepare(
        "SELECT title, description, code, expected_output, sort_order FROM examples WHERE item_id = ?1 ORDER BY sort_order, id",
    )?;
    let examples: Vec<Value> = stmt
        .query_map(params![id], |r| {
            Ok(json!({
                "title": r.get::<_, Option<String>>(0)?, "description": r.get::<_, Option<String>>(1)?,
                "code": r.get::<_, String>(2)?, "expectedOutput": r.get::<_, Option<String>>(3)?,
                "sortOrder": r.get::<_, i64>(4)?
            }))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT name, description, is_required, default_value, value_hint, sort_order
         FROM parameters WHERE item_id = ?1 ORDER BY sort_order, name",
    )?;
    let parameters: Vec<Value> = stmt
        .query_map(params![id], |r| {
            Ok(json!({
                "name": r.get::<_, String>(0)?, "description": r.get::<_, Option<String>>(1)?,
                "isRequired": r.get::<_, i64>(2)? == 1, "defaultValue": r.get::<_, Option<String>>(3)?,
                "valueHint": r.get::<_, Option<String>>(4)?, "sortOrder": r.get::<_, i64>(5)?
            }))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let diagram: Option<Value> = row.12.as_ref().and_then(|d| serde_json::from_str(d).ok());

    Ok(Some(json!({
        "externalKey": external_key,
        "slug": row.1, "toolSlug": row.2, "itemType": row.3, "title": row.4, "subtitle": row.5,
        "shortDescription": row.6, "fullDescription": row.7, "syntax": row.8, "whenToUse": row.9,
        "expectedResult": row.10, "warningText": row.11, "resultDiagram": diagram,
        "verificationStatus": row.13, "lifecycleStatus": row.14, "difficulty": row.15,
        "iconKey": row.16, "accentKey": row.17, "sortOrder": row.18, "lastVerifiedAt": row.19,
        "aliases": aliases, "tasks": tasks, "categories": categories, "tags": tags,
        "examples": examples, "parameters": parameters,
        // Provenance and relationships are compared through the item payload the
        // pack supplies; they are re-linked on apply.
        "sources": Value::Null, "relationships": Value::Null,
    })))
}

fn current_simple_json(
    conn: &Connection,
    table: &str,
    external_key: &str,
) -> Result<Option<Value>> {
    let sql = match table {
        "tools" => "SELECT slug, name, short_name, description, icon_key, accent_key, official_home_url, sort_order FROM tools WHERE external_key = ?1",
        "tasks" => "SELECT slug, name, description, icon_key, sort_order FROM tasks WHERE external_key = ?1",
        "categories" => "SELECT c.slug, c.name, c.description, c.icon_key, c.accent_key, p.slug, c.sort_order
                         FROM categories c LEFT JOIN categories p ON p.id = c.parent_id WHERE c.external_key = ?1",
        _ => return Ok(None),
    };
    let value = conn
        .query_row(sql, params![external_key], |r| {
            Ok(match table {
                "tools" => json!({
                    "slug": r.get::<_, String>(0)?, "name": r.get::<_, String>(1)?,
                    "shortName": r.get::<_, Option<String>>(2)?, "description": r.get::<_, Option<String>>(3)?,
                    "iconKey": r.get::<_, String>(4)?, "accentKey": r.get::<_, String>(5)?,
                    "officialHomeUrl": r.get::<_, Option<String>>(6)?, "sortOrder": r.get::<_, i64>(7)?,
                    "externalKey": external_key
                }),
                "tasks" => json!({
                    "slug": r.get::<_, String>(0)?, "name": r.get::<_, String>(1)?,
                    "description": r.get::<_, Option<String>>(2)?, "iconKey": r.get::<_, String>(3)?,
                    "sortOrder": r.get::<_, i64>(4)?, "externalKey": external_key
                }),
                _ => json!({
                    "slug": r.get::<_, String>(0)?, "name": r.get::<_, String>(1)?,
                    "description": r.get::<_, Option<String>>(2)?, "iconKey": r.get::<_, String>(3)?,
                    "accentKey": r.get::<_, String>(4)?, "parentSlug": r.get::<_, Option<String>>(5)?,
                    "sortOrder": r.get::<_, i64>(6)?, "externalKey": external_key
                }),
            })
        })
        .ok();
    Ok(value)
}

struct Row {
    entity_type: &'static str,
    entity_key: String,
    change_type: &'static str,
    title: String,
    detail: Option<String>,
    before: Option<Value>,
    after: Option<Value>,
}

/// Import a validated pack into a new staged batch. Nothing trusted is modified.
pub fn stage_pack(store: &Store, pack: &KnowledgePack, created_by: &str) -> Result<String> {
    let conn = store.conn();
    let batch_id = ulid();
    let now = now_iso();

    let mut rows: Vec<Row> = Vec::new();

    for tool in &pack.tools {
        let key = tool
            .external_key
            .clone()
            .unwrap_or_else(|| format!("tool:{}", tool.slug));
        let after = tool_json(tool);
        let before = current_simple_json(conn, "tools", &key)?;
        push_change(
            &mut rows,
            "tool",
            key,
            tool.name.clone(),
            None,
            before,
            after,
        );
    }
    for task in &pack.tasks {
        let key = task
            .external_key
            .clone()
            .unwrap_or_else(|| format!("task:{}", task.slug));
        let after = task_json(task);
        let before = current_simple_json(conn, "tasks", &key)?;
        push_change(
            &mut rows,
            "task",
            key,
            task.name.clone(),
            None,
            before,
            after,
        );
    }
    for cat in &pack.categories {
        let key = cat
            .external_key
            .clone()
            .unwrap_or_else(|| format!("category:{}", cat.slug));
        let after = category_json(cat);
        let before = current_simple_json(conn, "categories", &key)?;
        let detail = cat.parent_slug.as_ref().map(|p| format!("under {p}"));
        push_change(
            &mut rows,
            "category",
            key,
            cat.name.clone(),
            detail,
            before,
            after,
        );
    }
    for tag in &pack.tags {
        let key = format!("tag:{}", tag.slug);
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM tags WHERE slug = ?1",
            params![tag.slug],
            |r| r.get(0),
        )?;
        if exists == 0 {
            rows.push(Row {
                entity_type: "tag",
                entity_key: key,
                change_type: "add",
                title: tag.name.clone(),
                detail: None,
                before: None,
                after: Some(tag_json(tag)),
            });
        }
    }
    for src in &pack.sources {
        let key = format!("source:{}", src.key);
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sources WHERE external_key = ?1",
            params![key],
            |r| r.get(0),
        )?;
        rows.push(Row {
            entity_type: "source",
            entity_key: key,
            change_type: if exists == 0 { "add" } else { "update" },
            title: src.name.clone(),
            detail: src.url.clone(),
            before: None,
            after: Some(source_json(src)),
        });
    }

    for item in &pack.items {
        let after = item_json(item);
        let before = current_item_json(conn, &item.external_key)?;
        let deprecating = item.lifecycle_status.as_deref() == Some("deprecated")
            && before
                .as_ref()
                .and_then(|b| b.get("lifecycleStatus"))
                .and_then(|v| v.as_str())
                .map(|s| s != "deprecated")
                .unwrap_or(false);
        let change_type = match (&before, deprecating) {
            (None, _) => "add",
            (Some(_), true) => "deprecate",
            (Some(_), false) => "update",
        };
        if change_type == "update" && !item_changed(before.as_ref().unwrap(), &after) {
            continue; // identical: not a change, so not a review item
        }
        rows.push(Row {
            entity_type: "item",
            entity_key: item.external_key.clone(),
            change_type,
            title: item.title.clone(),
            detail: item.short_description.clone(),
            before,
            after: Some(after),
        });
    }

    for removal in &pack.removals {
        let entity_type: &'static str = match removal.entity_type.as_str() {
            "tool" => "tool",
            "task" => "task",
            "category" => "category",
            _ => "item",
        };
        let before = if entity_type == "item" {
            current_item_json(conn, &removal.external_key)?
        } else {
            None
        };
        // A source dropping an item is a candidate for removal, never a deletion.
        rows.push(Row {
            entity_type,
            entity_key: removal.external_key.clone(),
            change_type: "remove_candidate",
            title: before
                .as_ref()
                .and_then(|b| b.get("title"))
                .and_then(|v| v.as_str())
                .unwrap_or(&removal.external_key)
                .to_string(),
            detail: removal.reason.clone(),
            before,
            after: None,
        });
    }

    let summary = pack
        .batch
        .summary
        .clone()
        .unwrap_or_else(|| format!("{} change(s) from {}", rows.len(), pack.batch.producer));

    conn.execute(
        "INSERT INTO update_batches (id, created_by, producer, summary, notes, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6)",
        params![
            batch_id,
            created_by,
            pack.batch.producer,
            summary,
            pack.batch.notes,
            now
        ],
    )?;

    for row in &rows {
        conn.execute(
            "INSERT INTO staged_changes
               (id, batch_id, entity_type, entity_key, change_type, title, detail,
                before_json, after_json, status, detected_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'pending', ?10)",
            params![
                ulid(),
                batch_id,
                row.entity_type,
                row.entity_key,
                row.change_type,
                row.title,
                row.detail,
                row.before.as_ref().map(|v| v.to_string()),
                row.after.as_ref().map(|v| v.to_string()),
                now
            ],
        )?;
    }

    audit::record(
        conn,
        created_by,
        "pack.stage",
        Some("batch"),
        Some(&batch_id),
        Some(&batch_id),
        Some(json!({ "changes": rows.len(), "producer": pack.batch.producer })),
    )?;

    Ok(batch_id)
}

fn push_change(
    rows: &mut Vec<Row>,
    entity_type: &'static str,
    key: String,
    title: String,
    detail: Option<String>,
    before: Option<Value>,
    after: Value,
) {
    let change_type = match &before {
        None => "add",
        Some(b) => {
            if b == &after {
                return;
            }
            "update"
        }
    };
    rows.push(Row {
        entity_type,
        entity_key: key,
        change_type,
        title,
        detail,
        before,
        after: Some(after),
    });
}

/// Compare an item's stored form to the incoming one, ignoring the fields the
/// stored projection cannot reconstruct (sources and relationships are re-linked
/// on every apply).
fn item_changed(before: &Value, after: &Value) -> bool {
    let ignore = ["sources", "relationships"];
    let (Some(b), Some(a)) = (before.as_object(), after.as_object()) else {
        return true;
    };
    for (key, after_value) in a {
        if ignore.contains(&key.as_str()) {
            continue;
        }
        if b.get(key).unwrap_or(&Value::Null) != after_value {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------- read side

fn field_diff(before: Option<&Value>, after: Option<&Value>) -> Vec<FieldDiff> {
    let mut out = Vec::new();
    let empty = serde_json::Map::new();
    let b = before.and_then(|v| v.as_object()).unwrap_or(&empty);
    let a = after.and_then(|v| v.as_object()).unwrap_or(&empty);
    let mut keys: Vec<&String> = b.keys().chain(a.keys()).collect();
    keys.sort();
    keys.dedup();
    for key in keys {
        let bv = b.get(key).unwrap_or(&Value::Null);
        let av = a.get(key).unwrap_or(&Value::Null);
        if bv == av {
            continue;
        }
        let render = |v: &Value| -> Option<String> {
            match v {
                Value::Null => None,
                Value::String(s) => Some(s.clone()),
                other => Some(other.to_string()),
            }
        };
        out.push(FieldDiff {
            field: key.clone(),
            before: render(bv),
            after: render(av),
        });
    }
    out
}

pub fn list_batches(store: &Store, limit: i64) -> Result<Vec<UpdateBatch>> {
    let mut stmt = store.conn().prepare(
        "SELECT b.id, b.source_id, b.created_by, b.producer, b.summary, b.notes, b.status,
                b.backup_id, b.created_at, b.applied_at, b.reverted_at,
                (SELECT COUNT(*) FROM staged_changes s WHERE s.batch_id = b.id AND s.status='pending'),
                (SELECT COUNT(*) FROM staged_changes s WHERE s.batch_id = b.id AND s.status='approved'),
                (SELECT COUNT(*) FROM staged_changes s WHERE s.batch_id = b.id AND s.status='rejected'),
                (SELECT COUNT(*) FROM staged_changes s WHERE s.batch_id = b.id AND s.status='applied'),
                (SELECT COUNT(*) FROM staged_changes s WHERE s.batch_id = b.id AND s.change_type='add'),
                (SELECT COUNT(*) FROM staged_changes s WHERE s.batch_id = b.id AND s.change_type IN ('update','relink')),
                (SELECT COUNT(*) FROM staged_changes s WHERE s.batch_id = b.id AND s.change_type IN ('remove_candidate','deprecate'))
         FROM update_batches b ORDER BY b.created_at DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit], |r| {
        Ok(UpdateBatch {
            id: r.get(0)?,
            source_id: r.get(1)?,
            created_by: r.get(2)?,
            producer: r.get(3)?,
            summary: r.get(4)?,
            notes: r.get(5)?,
            status: r.get(6)?,
            backup_id: r.get(7)?,
            created_at: r.get(8)?,
            applied_at: r.get(9)?,
            reverted_at: r.get(10)?,
            pending_count: r.get(11)?,
            approved_count: r.get(12)?,
            rejected_count: r.get(13)?,
            applied_count: r.get(14)?,
            add_count: r.get(15)?,
            update_count: r.get(16)?,
            remove_count: r.get(17)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn get_batch(store: &Store, batch_id: &str) -> Result<UpdateBatch> {
    list_batches(store, 1000)?
        .into_iter()
        .find(|b| b.id == batch_id)
        .ok_or_else(|| crate::CoreError::NotFound(format!("batch {batch_id}")))
}

pub fn list_changes(
    store: &Store,
    batch_id: Option<&str>,
    status: Option<&str>,
) -> Result<Vec<StagedChange>> {
    let mut sql = String::from(
        "SELECT id, batch_id, entity_type, entity_key, change_type, title, detail,
                before_json, after_json, status, confidence, error_text, detected_at
         FROM staged_changes WHERE 1=1",
    );
    let mut args: Vec<String> = Vec::new();
    if let Some(b) = batch_id {
        sql.push_str(" AND batch_id = ?");
        args.push(b.to_string());
    }
    if let Some(s) = status {
        sql.push_str(" AND status = ?");
        args.push(s.to_string());
    }
    sql.push_str(" ORDER BY entity_type, title");

    let mut stmt = store.conn().prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(args.iter()), |r| {
        let before: Option<String> = r.get(7)?;
        let after: Option<String> = r.get(8)?;
        let before_json = before.and_then(|s| serde_json::from_str::<Value>(&s).ok());
        let after_json = after.and_then(|s| serde_json::from_str::<Value>(&s).ok());
        Ok(StagedChange {
            id: r.get(0)?,
            batch_id: r.get(1)?,
            entity_type: r.get(2)?,
            entity_key: r.get(3)?,
            change_type: r.get(4)?,
            title: r.get(5)?,
            detail: r.get(6)?,
            field_diff: field_diff(before_json.as_ref(), after_json.as_ref()),
            before_json,
            after_json,
            status: r.get(9)?,
            confidence: r.get(10)?,
            error_text: r.get(11)?,
            detected_at: r.get(12)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// Approve or reject: either the whole batch, or individual staged rows.
pub fn set_review_status(
    store: &Store,
    batch_id: &str,
    change_ids: Option<&[String]>,
    status: &str,
) -> Result<usize> {
    if !matches!(status, "approved" | "rejected" | "pending") {
        return Err(crate::CoreError::Invalid(format!(
            "unknown review status {status}"
        )));
    }
    let now = now_iso();
    let changed = match change_ids {
        Some(ids) if !ids.is_empty() => {
            let marks = vec!["?"; ids.len()].join(",");
            let sql = format!(
                "UPDATE staged_changes SET status = ?1, reviewed_at = ?2
                 WHERE batch_id = ?3 AND status IN ('pending','approved','rejected') AND id IN ({marks})"
            );
            let mut args: Vec<&dyn rusqlite::ToSql> = vec![&status, &now, &batch_id];
            for id in ids {
                args.push(id);
            }
            store.conn().execute(&sql, args.as_slice())?
        }
        _ => store.conn().execute(
            "UPDATE staged_changes SET status = ?1, reviewed_at = ?2
             WHERE batch_id = ?3 AND status IN ('pending','approved','rejected')",
            params![status, now, batch_id],
        )?,
    };

    let pending: i64 = store.conn().query_row(
        "SELECT COUNT(*) FROM staged_changes WHERE batch_id = ?1 AND status IN ('pending','approved')",
        params![batch_id],
        |r| r.get(0),
    )?;
    if pending == 0 && status == "rejected" {
        store.conn().execute(
            "UPDATE update_batches SET status = 'rejected' WHERE id = ?1 AND status = 'pending'",
            params![batch_id],
        )?;
    }
    audit::record(
        store.conn(),
        "user",
        &format!("changes.{status}"),
        Some("batch"),
        Some(batch_id),
        Some(batch_id),
        Some(json!({ "count": changed })),
    )?;
    Ok(changed)
}
