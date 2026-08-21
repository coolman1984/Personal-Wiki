//! Transactional apply and reversal.
//!
//! Contract (docs/UI_SPEC.md §9):
//!   approve → backup → BEGIN → apply → rebuild affected FTS rows → audit → COMMIT
//!   on any error: ROLLBACK, keep the backup, record the failure
//!   undo: reverse the batch from its stored `before_json`, reindex, audit
//!
//! Reversal is logical, not a file restore, so personal notes, favorites and pins
//! created since the apply are kept. A full snapshot restore is a separate,
//! explicit action (`backup::restore`).

use crate::clock::now_iso;
use crate::error::{CoreError, Result};
use crate::ids::ulid;
use crate::models::ApplyOutcome;
use crate::{audit, backup, search, staging, Store};
use rusqlite::{params, Transaction};
use serde_json::{json, Value};
use std::collections::HashSet;

fn s(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())
}
fn i(value: &Value, key: &str, default: i64) -> i64 {
    value.get(key).and_then(|v| v.as_i64()).unwrap_or(default)
}
fn arr<'a>(value: &'a Value, key: &str) -> Vec<&'a Value> {
    value
        .get(key)
        .and_then(|v| v.as_array())
        .map(|a| a.iter().collect())
        .unwrap_or_default()
}
fn strings(value: &Value, key: &str) -> Vec<String> {
    arr(value, key)
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect()
}

/// Apply every approved change in a batch. Returns the outcome; on failure the
/// database is untouched and the pre-apply backup is retained.
pub fn apply_batch(store: &mut Store, batch_id: &str, actor: &str) -> Result<ApplyOutcome> {
    let batch = staging::get_batch(store, batch_id)?;
    if batch.status == "applied" {
        return Err(CoreError::Refused(format!(
            "batch {batch_id} is already applied"
        )));
    }
    if batch.approved_count == 0 {
        return Err(CoreError::Refused(format!(
            "batch {batch_id} has no approved changes; approve them first (staging is the safe default)"
        )));
    }

    let snapshot = backup::create(store, "pre_apply", Some(batch_id))?;
    let changes = staging::list_changes(store, Some(batch_id), Some("approved"))?;

    let result = (|| -> Result<(i64, Vec<String>)> {
        let tx = store.conn_mut().transaction()?;
        let mut touched: HashSet<String> = HashSet::new();
        let mut applied = 0i64;

        // Dependency order: taxonomy before the items that reference it,
        // relationships after every item exists.
        let order = ["tool", "task", "category", "tag", "source", "item"];
        for entity_type in order {
            let mut group: Vec<_> = changes
                .iter()
                .filter(|c| c.entity_type == entity_type)
                .collect();
            if entity_type == "category" {
                // parents first, so a child never references a missing parent
                group.sort_by_key(|c| {
                    c.after_json
                        .as_ref()
                        .and_then(|a| a.get("parentSlug"))
                        .and_then(|v| v.as_str())
                        .map(|_| 1)
                        .unwrap_or(0)
                });
            }
            for change in group {
                apply_one(&tx, change, &mut touched)?;
                tx.execute(
                    "UPDATE staged_changes SET status = 'applied', applied_at = ?2 WHERE id = ?1",
                    params![change.id, now_iso()],
                )?;
                applied += 1;
            }
        }

        // Second pass: relationships, now that every referenced item exists.
        for change in changes.iter().filter(|c| c.entity_type == "item") {
            if let Some(after) = &change.after_json {
                link_relationships(&tx, &change.entity_key, after)?;
            }
        }

        for item_id in &touched {
            search::reindex_item(&tx, item_id)?;
        }

        tx.execute(
            "UPDATE update_batches SET status = ?2, applied_at = ?3, backup_id = ?4 WHERE id = ?1",
            params![
                batch_id,
                if batch.pending_count > 0 || batch.rejected_count > 0 {
                    "partially_applied"
                } else {
                    "applied"
                },
                now_iso(),
                snapshot.id
            ],
        )?;
        audit::record(
            &tx,
            actor,
            "batch.apply",
            Some("batch"),
            Some(batch_id),
            Some(batch_id),
            Some(json!({ "applied": applied, "backup": snapshot.id })),
        )?;

        tx.commit()?;
        Ok((applied, touched.into_iter().collect()))
    })();

    match result {
        Ok((applied, _)) => {
            let status = staging::get_batch(store, batch_id)?.status;
            Ok(ApplyOutcome {
                batch_id: batch_id.to_string(),
                applied,
                failed: 0,
                skipped: batch.pending_count + batch.rejected_count,
                backup_id: Some(snapshot.id),
                status,
            })
        }
        Err(err) => {
            // The transaction rolled back on drop; record the failure outside it.
            let message = err.to_string();
            store.conn().execute(
                "UPDATE update_batches SET status = 'failed' WHERE id = ?1",
                params![batch_id],
            )?;
            store.conn().execute(
                "UPDATE staged_changes SET status = 'failed', error_text = ?2
                 WHERE batch_id = ?1 AND status = 'approved'",
                params![batch_id, message],
            )?;
            audit::record(
                store.conn(),
                actor,
                "batch.apply_failed",
                Some("batch"),
                Some(batch_id),
                Some(batch_id),
                Some(json!({ "error": message, "backup": snapshot.id })),
            )?;
            Err(err)
        }
    }
}

fn apply_one(
    tx: &Transaction,
    change: &crate::models::StagedChange,
    touched: &mut HashSet<String>,
) -> Result<()> {
    match (change.entity_type.as_str(), change.change_type.as_str()) {
        ("item", "remove_candidate") => {
            // Never a hard delete: the record is retired and kept for history.
            if let Some(id) = item_id_for_key(tx, &change.entity_key)? {
                tx.execute(
                    "UPDATE items SET lifecycle_status = 'removed', is_active = 0, updated_at = ?2 WHERE id = ?1",
                    params![id, now_iso()],
                )?;
                touched.insert(id);
            }
            Ok(())
        }
        (_, "remove_candidate") => {
            let table = match change.entity_type.as_str() {
                "tool" => "tools",
                "task" => "tasks",
                "category" => "categories",
                _ => return Ok(()),
            };
            tx.execute(
                &format!(
                    "UPDATE {table} SET is_active = 0, updated_at = ?2 WHERE external_key = ?1"
                ),
                params![change.entity_key, now_iso()],
            )?;
            Ok(())
        }
        ("item", _) => {
            let after = change.after_json.as_ref().ok_or_else(|| {
                CoreError::Invalid(format!("staged item {} has no payload", change.entity_key))
            })?;
            let id = upsert_item(tx, after)?;
            touched.insert(id);
            Ok(())
        }
        ("tool", _) => upsert_tool(tx, payload(change)?),
        ("task", _) => upsert_task(tx, payload(change)?),
        ("category", _) => upsert_category(tx, payload(change)?),
        ("tag", _) => upsert_tag(tx, payload(change)?),
        ("source", _) => upsert_source(tx, payload(change)?),
        (other, _) => Err(CoreError::Invalid(format!(
            "cannot apply entity type {other}"
        ))),
    }
}

fn payload(change: &crate::models::StagedChange) -> Result<&Value> {
    change.after_json.as_ref().ok_or_else(|| {
        CoreError::Invalid(format!(
            "staged change {} has no payload",
            change.entity_key
        ))
    })
}

fn item_id_for_key(tx: &Transaction, external_key: &str) -> Result<Option<String>> {
    Ok(tx
        .query_row(
            "SELECT id FROM items WHERE external_key = ?1",
            params![external_key],
            |r| r.get::<_, String>(0),
        )
        .ok())
}

fn upsert_tool(tx: &Transaction, v: &Value) -> Result<()> {
    let now = now_iso();
    let key = s(v, "externalKey");
    let slug = s(v, "slug").ok_or_else(|| CoreError::Invalid("tool payload has no slug".into()))?;
    let existing: Option<String> = tx
        .query_row(
            "SELECT id FROM tools WHERE external_key = ?1 OR slug = ?2",
            params![key, slug],
            |r| r.get(0),
        )
        .ok();
    match existing {
        Some(id) => {
            tx.execute(
                "UPDATE tools SET slug=?2, name=?3, short_name=?4, description=?5, icon_key=?6,
                        accent_key=?7, official_home_url=?8, sort_order=?9, external_key=?10,
                        managed_by='pack', is_active=1, updated_at=?11 WHERE id=?1",
                params![
                    id,
                    slug,
                    s(v, "name"),
                    s(v, "shortName"),
                    s(v, "description"),
                    s(v, "iconKey").unwrap_or_else(|| "book".into()),
                    s(v, "accentKey").unwrap_or_else(|| "blue".into()),
                    s(v, "officialHomeUrl"),
                    i(v, "sortOrder", 100),
                    key,
                    now
                ],
            )?;
        }
        None => {
            tx.execute(
                "INSERT INTO tools (id, slug, name, short_name, description, icon_key, accent_key,
                        official_home_url, sort_order, is_active, external_key, managed_by, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,1,?10,'pack',?11,?11)",
                params![ulid(), slug, s(v,"name"), s(v,"shortName"), s(v,"description"),
                        s(v,"iconKey").unwrap_or_else(|| "book".into()),
                        s(v,"accentKey").unwrap_or_else(|| "blue".into()),
                        s(v,"officialHomeUrl"), i(v,"sortOrder",100), key, now],
            )?;
        }
    }
    Ok(())
}

fn upsert_task(tx: &Transaction, v: &Value) -> Result<()> {
    let now = now_iso();
    let key = s(v, "externalKey");
    let slug = s(v, "slug").ok_or_else(|| CoreError::Invalid("task payload has no slug".into()))?;
    let existing: Option<String> = tx
        .query_row(
            "SELECT id FROM tasks WHERE external_key = ?1 OR slug = ?2",
            params![key, slug],
            |r| r.get(0),
        )
        .ok();
    match existing {
        Some(id) => {
            tx.execute(
                "UPDATE tasks SET slug=?2, name=?3, description=?4, icon_key=?5, sort_order=?6,
                        external_key=?7, managed_by='pack', is_active=1, updated_at=?8 WHERE id=?1",
                params![
                    id,
                    slug,
                    s(v, "name"),
                    s(v, "description"),
                    s(v, "iconKey").unwrap_or_else(|| "target".into()),
                    i(v, "sortOrder", 100),
                    key,
                    now
                ],
            )?;
        }
        None => {
            tx.execute(
                "INSERT INTO tasks (id, slug, name, description, icon_key, sort_order, is_active,
                        external_key, managed_by, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,1,?7,'pack',?8,?8)",
                params![
                    ulid(),
                    slug,
                    s(v, "name"),
                    s(v, "description"),
                    s(v, "iconKey").unwrap_or_else(|| "target".into()),
                    i(v, "sortOrder", 100),
                    key,
                    now
                ],
            )?;
        }
    }
    Ok(())
}

fn upsert_category(tx: &Transaction, v: &Value) -> Result<()> {
    let now = now_iso();
    let key = s(v, "externalKey");
    let slug =
        s(v, "slug").ok_or_else(|| CoreError::Invalid("category payload has no slug".into()))?;
    let parent_id: Option<String> = match s(v, "parentSlug") {
        Some(parent_slug) => Some(
            tx.query_row(
                "SELECT id FROM categories WHERE slug = ?1",
                params![parent_slug],
                |r| r.get(0),
            )
            .map_err(|_| {
                CoreError::Invalid(format!("parent category {parent_slug:?} does not exist"))
            })?,
        ),
        None => None,
    };
    let existing: Option<String> = tx
        .query_row(
            "SELECT id FROM categories WHERE external_key = ?1 OR slug = ?2",
            params![key, slug],
            |r| r.get(0),
        )
        .ok();
    match existing {
        Some(id) => {
            if parent_id.as_deref() == Some(id.as_str()) {
                return Err(CoreError::Invalid(
                    "a category cannot be its own parent".into(),
                ));
            }
            tx.execute(
                "UPDATE categories SET slug=?2, name=?3, description=?4, icon_key=?5, accent_key=?6,
                        parent_id=?7, sort_order=?8, external_key=?9, managed_by='pack', is_active=1,
                        updated_at=?10 WHERE id=?1",
                params![id, slug, s(v,"name"), s(v,"description"),
                        s(v,"iconKey").unwrap_or_else(|| "folder".into()),
                        s(v,"accentKey").unwrap_or_else(|| "blue".into()),
                        parent_id, i(v,"sortOrder",100), key, now],
            )?;
        }
        None => {
            tx.execute(
                "INSERT INTO categories (id, slug, name, description, icon_key, accent_key, parent_id,
                        sort_order, is_active, external_key, managed_by, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,1,?9,'pack',?10,?10)",
                params![ulid(), slug, s(v,"name"), s(v,"description"),
                        s(v,"iconKey").unwrap_or_else(|| "folder".into()),
                        s(v,"accentKey").unwrap_or_else(|| "blue".into()),
                        parent_id, i(v,"sortOrder",100), key, now],
            )?;
        }
    }
    Ok(())
}

fn upsert_tag(tx: &Transaction, v: &Value) -> Result<()> {
    let slug = s(v, "slug").ok_or_else(|| CoreError::Invalid("tag payload has no slug".into()))?;
    tx.execute(
        "INSERT INTO tags (id, slug, name, accent_key) VALUES (?1,?2,?3,?4)
         ON CONFLICT(slug) DO UPDATE SET name=?3, accent_key=?4",
        params![
            ulid(),
            slug,
            s(v, "name"),
            s(v, "accentKey").unwrap_or_else(|| "blue".into())
        ],
    )?;
    Ok(())
}

fn upsert_source(tx: &Transaction, v: &Value) -> Result<()> {
    let now = now_iso();
    let key = format!("source:{}", s(v, "key").unwrap_or_default());
    let tool_id: Option<String> = match s(v, "toolSlug") {
        Some(slug) => tx
            .query_row("SELECT id FROM tools WHERE slug = ?1", params![slug], |r| {
                r.get(0)
            })
            .ok(),
        None => None,
    };
    let existing: Option<String> = tx
        .query_row(
            "SELECT id FROM sources WHERE external_key = ?1",
            params![key],
            |r| r.get(0),
        )
        .ok();
    let is_official = v
        .get("isOfficial")
        .and_then(|x| x.as_bool())
        .unwrap_or(false) as i64;
    match existing {
        Some(id) => {
            tx.execute(
                "UPDATE sources SET tool_id=?2, name=?3, source_type=?4, url=?5, is_official=?6,
                        sync_strategy=?7, notes=?8, is_active=1, updated_at=?9 WHERE id=?1",
                params![
                    id,
                    tool_id,
                    s(v, "name"),
                    s(v, "sourceType"),
                    s(v, "url"),
                    is_official,
                    s(v, "syncStrategy").unwrap_or_else(|| "agent".into()),
                    s(v, "notes"),
                    now
                ],
            )?;
        }
        None => {
            tx.execute(
                "INSERT INTO sources (id, tool_id, name, source_type, url, is_official, sync_strategy,
                        is_active, notes, external_key, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,1,?8,?9,?10,?10)",
                params![ulid(), tool_id, s(v,"name"), s(v,"sourceType"), s(v,"url"), is_official,
                        s(v,"syncStrategy").unwrap_or_else(|| "agent".into()), s(v,"notes"), key, now],
            )?;
        }
    }
    Ok(())
}

fn upsert_item(tx: &Transaction, v: &Value) -> Result<String> {
    let now = now_iso();
    let external_key = s(v, "externalKey")
        .ok_or_else(|| CoreError::Invalid("item payload has no externalKey".into()))?;
    let slug = s(v, "slug").ok_or_else(|| CoreError::Invalid("item payload has no slug".into()))?;

    let tool_id: Option<String> = match s(v, "toolSlug") {
        Some(tool_slug) => Some(
            tx.query_row(
                "SELECT id FROM tools WHERE slug = ?1",
                params![tool_slug],
                |r| r.get(0),
            )
            .map_err(|_| CoreError::Invalid(format!("tool {tool_slug:?} does not exist")))?,
        ),
        None => None,
    };

    let by_key: Option<String> = tx
        .query_row(
            "SELECT id FROM items WHERE external_key = ?1",
            params![external_key],
            |r| r.get(0),
        )
        .ok();
    let by_slug: Option<(String, Option<String>)> = tx
        .query_row(
            "SELECT id, external_key FROM items WHERE slug = ?1",
            params![slug],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok();

    // Identity rules: match on external_key first. A slug already owned by a
    // *different* external key is a real conflict and fails the change rather than
    // silently overwriting someone else's record.
    let id = match (by_key, by_slug) {
        (Some(id), _) => id,
        (None, Some((id, None))) => {
            tx.execute(
                "UPDATE items SET external_key = ?2, managed_by = 'pack' WHERE id = ?1",
                params![id, external_key],
            )?;
            id
        }
        (None, Some((_, Some(other)))) => {
            return Err(CoreError::Invalid(format!(
                "slug {slug:?} already belongs to external key {other:?}"
            )))
        }
        (None, None) => {
            let id = ulid();
            tx.execute(
                "INSERT INTO items (id, slug, item_type, title, external_key, managed_by, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'pack', ?6, ?6)",
                params![id, slug, s(v,"itemType").unwrap_or_else(|| "concept".into()), s(v,"title"), external_key, now],
            )?;
            id
        }
    };

    tx.execute(
        "UPDATE items SET tool_id=?2, item_type=?3, slug=?4, title=?5, subtitle=?6,
                short_description=?7, full_description=?8, syntax=?9, when_to_use=?10,
                expected_result=?11, warning_text=?12, result_diagram=?13,
                verification_status=?14, lifecycle_status=?15, difficulty=?16, icon_key=?17,
                accent_key=?18, sort_order=?19, last_verified_at=?20, is_active=?21,
                managed_by='pack', updated_at=?22
         WHERE id=?1",
        params![
            id,
            tool_id,
            s(v, "itemType").unwrap_or_else(|| "concept".into()),
            slug,
            s(v, "title"),
            s(v, "subtitle"),
            s(v, "shortDescription"),
            s(v, "fullDescription"),
            s(v, "syntax"),
            s(v, "whenToUse"),
            s(v, "expectedResult"),
            s(v, "warningText"),
            v.get("resultDiagram")
                .filter(|d| !d.is_null())
                .map(|d| d.to_string()),
            s(v, "verificationStatus").unwrap_or_else(|| "unverified".into()),
            s(v, "lifecycleStatus").unwrap_or_else(|| "active".into()),
            s(v, "difficulty"),
            s(v, "iconKey"),
            s(v, "accentKey"),
            i(v, "sortOrder", 100),
            s(v, "lastVerifiedAt"),
            if s(v, "lifecycleStatus").as_deref() == Some("removed") {
                0
            } else {
                1
            },
            now
        ],
    )?;

    // Child collections are replaced wholesale — they belong to the source record.
    // Personal data (favorites, pins, notes) lives in other tables and is untouched.
    tx.execute("DELETE FROM aliases WHERE item_id = ?1", params![id])?;
    for alias in strings(v, "aliases") {
        tx.execute(
            "INSERT OR IGNORE INTO aliases (id, item_id, alias) VALUES (?1, ?2, ?3)",
            params![ulid(), id, alias],
        )?;
    }

    tx.execute("DELETE FROM examples WHERE item_id = ?1", params![id])?;
    for example in arr(v, "examples") {
        tx.execute(
            "INSERT INTO examples (id, item_id, title, description, code, expected_output, sort_order)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![ulid(), id, s(example,"title"), s(example,"description"),
                    s(example,"code").unwrap_or_default(), s(example,"expectedOutput"),
                    i(example,"sortOrder",100)],
        )?;
    }

    tx.execute("DELETE FROM parameters WHERE item_id = ?1", params![id])?;
    for param in arr(v, "parameters") {
        tx.execute(
            "INSERT OR REPLACE INTO parameters (id, item_id, name, description, is_required,
                     default_value, value_hint, sort_order)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                ulid(),
                id,
                s(param, "name").unwrap_or_default(),
                s(param, "description"),
                param
                    .get("isRequired")
                    .and_then(|b| b.as_bool())
                    .unwrap_or(false) as i64,
                s(param, "defaultValue"),
                s(param, "valueHint"),
                i(param, "sortOrder", 100)
            ],
        )?;
    }

    tx.execute("DELETE FROM item_tasks WHERE item_id = ?1", params![id])?;
    for task_slug in strings(v, "tasks") {
        if let Ok(task_id) = tx.query_row(
            "SELECT id FROM tasks WHERE slug = ?1",
            params![task_slug],
            |r| r.get::<_, String>(0),
        ) {
            tx.execute(
                "INSERT OR IGNORE INTO item_tasks (item_id, task_id) VALUES (?1,?2)",
                params![id, task_id],
            )?;
        } else {
            return Err(CoreError::Invalid(format!(
                "task {task_slug:?} does not exist"
            )));
        }
    }

    tx.execute(
        "DELETE FROM item_categories WHERE item_id = ?1",
        params![id],
    )?;
    for cat_slug in strings(v, "categories") {
        if let Ok(cat_id) = tx.query_row(
            "SELECT id FROM categories WHERE slug = ?1",
            params![cat_slug],
            |r| r.get::<_, String>(0),
        ) {
            tx.execute(
                "INSERT OR IGNORE INTO item_categories (item_id, category_id) VALUES (?1,?2)",
                params![id, cat_id],
            )?;
        } else {
            return Err(CoreError::Invalid(format!(
                "category {cat_slug:?} does not exist"
            )));
        }
    }

    tx.execute("DELETE FROM item_tags WHERE item_id = ?1", params![id])?;
    for tag_slug in strings(v, "tags") {
        let tag_id: String = match tx.query_row(
            "SELECT id FROM tags WHERE slug = ?1",
            params![tag_slug],
            |r| r.get(0),
        ) {
            Ok(existing) => existing,
            Err(_) => {
                let new_id = ulid();
                tx.execute(
                    "INSERT INTO tags (id, slug, name, accent_key) VALUES (?1,?2,?3,'blue')",
                    params![new_id, tag_slug, tag_slug],
                )?;
                new_id
            }
        };
        tx.execute(
            "INSERT OR IGNORE INTO item_tags (item_id, tag_id) VALUES (?1,?2)",
            params![id, tag_id],
        )?;
    }

    tx.execute("DELETE FROM item_sources WHERE item_id = ?1", params![id])?;
    for src in arr(v, "sources") {
        let key = format!("source:{}", s(src, "sourceKey").unwrap_or_default());
        if let Ok(source_id) = tx.query_row(
            "SELECT id FROM sources WHERE external_key = ?1",
            params![key],
            |r| r.get::<_, String>(0),
        ) {
            tx.execute(
                "INSERT OR REPLACE INTO item_sources (item_id, source_id, source_url, source_anchor,
                        source_version, verified_at) VALUES (?1,?2,?3,?4,?5,?6)",
                params![id, source_id, s(src,"url"), s(src,"anchor"), s(src,"version"), s(src,"verifiedAt")],
            )?;
        }
    }

    Ok(id)
}

fn link_relationships(tx: &Transaction, external_key: &str, v: &Value) -> Result<()> {
    let Some(from_id) = item_id_for_key(tx, external_key)? else {
        return Ok(());
    };
    tx.execute(
        "DELETE FROM relationships WHERE from_item_id = ?1",
        params![from_id],
    )?;
    for rel in arr(v, "relationships") {
        let Some(target_key) = s(rel, "toExternalKey") else {
            continue;
        };
        let Some(to_id) = item_id_for_key(tx, &target_key)? else {
            continue;
        };
        if to_id == from_id {
            continue;
        }
        tx.execute(
            "INSERT OR IGNORE INTO relationships (id, from_item_id, to_item_id, relationship_type)
             VALUES (?1,?2,?3,?4)",
            params![
                ulid(),
                from_id,
                to_id,
                s(rel, "relationshipType").unwrap_or_else(|| "related".into())
            ],
        )?;
    }
    Ok(())
}

/// Reverse an applied batch using the stored before-images.
/// Additions are retired (never hard-deleted); updates are rolled back field by
/// field. The personal overlay is not touched.
pub fn revert_batch(store: &mut Store, batch_id: &str, actor: &str) -> Result<ApplyOutcome> {
    let batch = staging::get_batch(store, batch_id)?;
    if batch.status != "applied" && batch.status != "partially_applied" {
        return Err(CoreError::Refused(format!(
            "batch {batch_id} is {} and cannot be reverted",
            batch.status
        )));
    }
    let snapshot = backup::create(store, "pre_revert", Some(batch_id))?;
    let changes = staging::list_changes(store, Some(batch_id), Some("applied"))?;

    let tx = store.conn_mut().transaction()?;
    let mut touched: HashSet<String> = HashSet::new();
    let mut reverted = 0i64;

    for change in changes.iter().rev() {
        match (&change.before_json, change.entity_type.as_str()) {
            // it did not exist before this batch: retire it
            (None, "item") => {
                if let Some(id) = item_id_for_key(&tx, &change.entity_key)? {
                    tx.execute(
                        "UPDATE items SET is_active = 0, lifecycle_status = 'removed', updated_at = ?2 WHERE id = ?1",
                        params![id, now_iso()],
                    )?;
                    touched.insert(id);
                }
            }
            (None, "tool") | (None, "task") | (None, "category") => {
                let table = match change.entity_type.as_str() {
                    "tool" => "tools",
                    "task" => "tasks",
                    _ => "categories",
                };
                tx.execute(
                    &format!(
                        "UPDATE {table} SET is_active = 0, updated_at = ?2 WHERE external_key = ?1"
                    ),
                    params![change.entity_key, now_iso()],
                )?;
            }
            (Some(before), "item") => {
                let id = upsert_item(&tx, before)?;
                touched.insert(id);
            }
            (Some(before), "tool") => upsert_tool(&tx, before)?,
            (Some(before), "task") => upsert_task(&tx, before)?,
            (Some(before), "category") => upsert_category(&tx, before)?,
            _ => {}
        }
        reverted += 1;
    }

    for item_id in &touched {
        search::reindex_item(&tx, item_id)?;
    }
    tx.execute(
        "UPDATE update_batches SET status = 'reverted', reverted_at = ?2 WHERE id = ?1",
        params![batch_id, now_iso()],
    )?;
    audit::record(
        &tx,
        actor,
        "batch.revert",
        Some("batch"),
        Some(batch_id),
        Some(batch_id),
        Some(json!({ "reverted": reverted, "safety_backup": snapshot.id })),
    )?;
    tx.commit()?;

    Ok(ApplyOutcome {
        batch_id: batch_id.to_string(),
        applied: 0,
        failed: 0,
        skipped: 0,
        backup_id: Some(snapshot.id),
        status: "reverted".into(),
    })
}

/// The most recent batch that can still be undone.
pub fn last_applied_batch(store: &Store) -> Result<Option<String>> {
    Ok(store
        .conn()
        .query_row(
            "SELECT id FROM update_batches WHERE status IN ('applied','partially_applied')
             ORDER BY applied_at DESC LIMIT 1",
            [],
            |r| r.get::<_, String>(0),
        )
        .ok())
}
