//! Typed, validated editing for locally authored records.
//!
//! These are the forms the user fills in. Records created here are
//! `managed_by = 'local'`, which is what keeps them out of the way of pack
//! refreshes: an import only adopts a local record when it has no external key
//! and the slug matches exactly.

use crate::clock::now_iso;
use crate::error::{CoreError, Result};
use crate::ids::{slugify, ulid};
use crate::models::{Category, ItemDetail, Task, Tool};
use crate::{audit, pack, repo, search, url, Store};
use rusqlite::params;
use serde::Deserialize;

fn require(value: &str, field: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CoreError::Invalid(format!("{field} is required")));
    }
    Ok(trimmed.to_string())
}

fn check_enum(value: &str, allowed: &[&str], field: &str) -> Result<()> {
    if !allowed.contains(&value) {
        return Err(CoreError::Invalid(format!(
            "{field}: {value:?} must be one of {}",
            allowed.join(", ")
        )));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInput {
    #[serde(default)] pub id: Option<String>,
    pub name: String,
    #[serde(default)] pub slug: Option<String>,
    #[serde(default)] pub short_name: Option<String>,
    #[serde(default)] pub description: Option<String>,
    #[serde(default)] pub icon_key: Option<String>,
    #[serde(default)] pub accent_key: Option<String>,
    #[serde(default)] pub official_home_url: Option<String>,
    #[serde(default)] pub sort_order: Option<i64>,
}

pub fn save_tool(store: &Store, input: ToolInput) -> Result<Tool> {
    let name = require(&input.name, "name")?;
    let slug = match &input.slug {
        Some(value) if !value.trim().is_empty() => slugify(value),
        _ => slugify(&name),
    };
    let accent = input.accent_key.clone().unwrap_or_else(|| "blue".into());
    check_enum(&accent, pack::ACCENT_KEYS, "accentKey")?;
    if let Some(url_value) = &input.official_home_url {
        if !url_value.trim().is_empty() {
            url::validate_web_url(url_value)?;
        }
    }
    let now = now_iso();
    let id = match input.id {
        Some(existing) => {
            store.conn().execute(
                "UPDATE tools SET slug=?2, name=?3, short_name=?4, description=?5, icon_key=?6,
                        accent_key=?7, official_home_url=?8, sort_order=?9, updated_at=?10 WHERE id=?1",
                params![existing, slug, name, input.short_name, input.description,
                        input.icon_key.unwrap_or_else(|| "book".into()), accent,
                        input.official_home_url, input.sort_order.unwrap_or(100), now],
            )?;
            existing
        }
        None => {
            let id = ulid();
            store.conn().execute(
                "INSERT INTO tools (id, slug, name, short_name, description, icon_key, accent_key,
                        official_home_url, sort_order, is_active, managed_by, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,1,'local',?10,?10)",
                params![id, slug, name, input.short_name, input.description,
                        input.icon_key.unwrap_or_else(|| "book".into()), accent,
                        input.official_home_url, input.sort_order.unwrap_or(100), now],
            )?;
            id
        }
    };
    audit::record(store.conn(), "user", "tool.save", Some("tool"), Some(&id), None, None)?;
    repo::list_tools(store, true)?
        .into_iter()
        .find(|t| t.id == id)
        .ok_or_else(|| CoreError::NotFound(format!("tool {id}")))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInput {
    #[serde(default)] pub id: Option<String>,
    pub name: String,
    #[serde(default)] pub slug: Option<String>,
    #[serde(default)] pub description: Option<String>,
    #[serde(default)] pub icon_key: Option<String>,
    #[serde(default)] pub sort_order: Option<i64>,
}

pub fn save_task(store: &Store, input: TaskInput) -> Result<Task> {
    let name = require(&input.name, "name")?;
    let slug = match &input.slug {
        Some(value) if !value.trim().is_empty() => slugify(value),
        _ => slugify(&name),
    };
    let now = now_iso();
    let id = match input.id {
        Some(existing) => {
            store.conn().execute(
                "UPDATE tasks SET slug=?2, name=?3, description=?4, icon_key=?5, sort_order=?6, updated_at=?7 WHERE id=?1",
                params![existing, slug, name, input.description,
                        input.icon_key.unwrap_or_else(|| "target".into()), input.sort_order.unwrap_or(100), now],
            )?;
            existing
        }
        None => {
            let id = ulid();
            store.conn().execute(
                "INSERT INTO tasks (id, slug, name, description, icon_key, sort_order, is_active, managed_by, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,1,'local',?7,?7)",
                params![id, slug, name, input.description,
                        input.icon_key.unwrap_or_else(|| "target".into()), input.sort_order.unwrap_or(100), now],
            )?;
            id
        }
    };
    repo::list_tasks(store, true)?
        .into_iter()
        .find(|t| t.id == id)
        .ok_or_else(|| CoreError::NotFound(format!("task {id}")))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryInput {
    #[serde(default)] pub id: Option<String>,
    pub name: String,
    #[serde(default)] pub slug: Option<String>,
    #[serde(default)] pub description: Option<String>,
    #[serde(default)] pub icon_key: Option<String>,
    #[serde(default)] pub accent_key: Option<String>,
    #[serde(default)] pub parent_id: Option<String>,
    #[serde(default)] pub sort_order: Option<i64>,
}

pub fn save_category(store: &Store, input: CategoryInput) -> Result<Category> {
    let name = require(&input.name, "name")?;
    let slug = match &input.slug {
        Some(value) if !value.trim().is_empty() => slugify(value),
        _ => slugify(&name),
    };
    let accent = input.accent_key.clone().unwrap_or_else(|| "blue".into());
    check_enum(&accent, pack::ACCENT_KEYS, "accentKey")?;
    let now = now_iso();

    // A category may not become its own ancestor.
    if let (Some(id), Some(parent)) = (&input.id, &input.parent_id) {
        if id == parent {
            return Err(CoreError::Invalid("a category cannot be its own parent".into()));
        }
        let mut cursor = Some(parent.clone());
        let mut hops = 0;
        while let Some(current) = cursor {
            if &current == id {
                return Err(CoreError::Invalid("that parent would create a cycle".into()));
            }
            hops += 1;
            if hops > 16 {
                break;
            }
            cursor = store
                .conn()
                .query_row("SELECT parent_id FROM categories WHERE id = ?1", params![current], |r| {
                    r.get::<_, Option<String>>(0)
                })
                .ok()
                .flatten();
        }
    }

    let id = match input.id {
        Some(existing) => {
            store.conn().execute(
                "UPDATE categories SET slug=?2, name=?3, description=?4, icon_key=?5, accent_key=?6,
                        parent_id=?7, sort_order=?8, updated_at=?9 WHERE id=?1",
                params![existing, slug, name, input.description,
                        input.icon_key.unwrap_or_else(|| "folder".into()), accent,
                        input.parent_id, input.sort_order.unwrap_or(100), now],
            )?;
            existing
        }
        None => {
            let id = ulid();
            store.conn().execute(
                "INSERT INTO categories (id, slug, name, description, icon_key, accent_key, parent_id,
                        sort_order, is_active, managed_by, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,1,'local',?9,?9)",
                params![id, slug, name, input.description,
                        input.icon_key.unwrap_or_else(|| "folder".into()), accent,
                        input.parent_id, input.sort_order.unwrap_or(100), now],
            )?;
            id
        }
    };
    repo::category_tree(store, None)?;
    store
        .conn()
        .query_row(
            "SELECT id, slug, name, description, icon_key, accent_key, parent_id, sort_order, is_active, external_key
             FROM categories WHERE id = ?1",
            params![id],
            |r| {
                Ok(Category {
                    id: r.get(0)?,
                    slug: r.get(1)?,
                    name: r.get(2)?,
                    description: r.get(3)?,
                    icon_key: r.get(4)?,
                    accent_key: r.get(5)?,
                    parent_id: r.get(6)?,
                    sort_order: r.get(7)?,
                    is_active: r.get::<_, i64>(8)? == 1,
                    external_key: r.get(9)?,
                })
            },
        )
        .map_err(Into::into)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemInput {
    #[serde(default)] pub id: Option<String>,
    pub title: String,
    pub item_type: String,
    #[serde(default)] pub slug: Option<String>,
    #[serde(default)] pub tool_id: Option<String>,
    #[serde(default)] pub subtitle: Option<String>,
    #[serde(default)] pub short_description: Option<String>,
    #[serde(default)] pub full_description: Option<String>,
    #[serde(default)] pub syntax: Option<String>,
    #[serde(default)] pub when_to_use: Option<String>,
    #[serde(default)] pub expected_result: Option<String>,
    #[serde(default)] pub warning_text: Option<String>,
    #[serde(default)] pub verification_status: Option<String>,
    #[serde(default)] pub lifecycle_status: Option<String>,
    #[serde(default)] pub icon_key: Option<String>,
    #[serde(default)] pub accent_key: Option<String>,
    #[serde(default)] pub aliases: Vec<String>,
    #[serde(default)] pub task_ids: Vec<String>,
    #[serde(default)] pub category_ids: Vec<String>,
    #[serde(default)] pub tag_slugs: Vec<String>,
    #[serde(default)] pub examples: Vec<ExampleInput>,
    #[serde(default)] pub parameters: Vec<ParameterInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExampleInput {
    pub code: String,
    #[serde(default)] pub title: Option<String>,
    #[serde(default)] pub description: Option<String>,
    #[serde(default)] pub expected_output: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParameterInput {
    pub name: String,
    #[serde(default)] pub description: Option<String>,
    #[serde(default)] pub is_required: Option<bool>,
    #[serde(default)] pub default_value: Option<String>,
}

pub fn save_item(store: &Store, input: ItemInput) -> Result<ItemDetail> {
    let title = require(&input.title, "title")?;
    check_enum(&input.item_type, pack::ITEM_TYPES, "itemType")?;
    let verification = input.verification_status.clone().unwrap_or_else(|| "unverified".into());
    check_enum(&verification, pack::VERIFICATION_STATUSES, "verificationStatus")?;
    // A locally authored item may not claim official status: provenance comes
    // from a source record, and this form does not collect one.
    if verification == "official" {
        return Err(CoreError::Refused(
            "a manually created item cannot be marked official; import it through a Knowledge Pack with a source".into(),
        ));
    }
    let lifecycle = input.lifecycle_status.clone().unwrap_or_else(|| "active".into());
    check_enum(&lifecycle, pack::LIFECYCLE_STATUSES, "lifecycleStatus")?;

    let slug = match &input.slug {
        Some(value) if !value.trim().is_empty() => slugify(value),
        _ => slugify(&title),
    };
    let now = now_iso();

    let id = match input.id.clone() {
        Some(existing) => existing,
        None => {
            let id = ulid();
            store.conn().execute(
                "INSERT INTO items (id, slug, item_type, title, managed_by, created_at, updated_at)
                 VALUES (?1,?2,?3,?4,'local',?5,?5)",
                params![id, slug, input.item_type, title, now],
            )?;
            id
        }
    };

    store.conn().execute(
        "UPDATE items SET tool_id=?2, item_type=?3, slug=?4, title=?5, subtitle=?6, short_description=?7,
                full_description=?8, syntax=?9, when_to_use=?10, expected_result=?11, warning_text=?12,
                verification_status=?13, lifecycle_status=?14, icon_key=?15, accent_key=?16, updated_at=?17
         WHERE id=?1",
        params![id, input.tool_id, input.item_type, slug, title, input.subtitle, input.short_description,
                input.full_description, input.syntax, input.when_to_use, input.expected_result,
                input.warning_text, verification, lifecycle, input.icon_key, input.accent_key, now],
    )?;

    let conn = store.conn();
    conn.execute("DELETE FROM aliases WHERE item_id = ?1", params![id])?;
    for alias in input.aliases.iter().filter(|a| !a.trim().is_empty()) {
        conn.execute(
            "INSERT OR IGNORE INTO aliases (id, item_id, alias) VALUES (?1,?2,?3)",
            params![ulid(), id, alias.trim()],
        )?;
    }
    conn.execute("DELETE FROM item_tasks WHERE item_id = ?1", params![id])?;
    for task_id in &input.task_ids {
        conn.execute("INSERT OR IGNORE INTO item_tasks (item_id, task_id) VALUES (?1,?2)", params![id, task_id])?;
    }
    conn.execute("DELETE FROM item_categories WHERE item_id = ?1", params![id])?;
    for category_id in &input.category_ids {
        conn.execute("INSERT OR IGNORE INTO item_categories (item_id, category_id) VALUES (?1,?2)", params![id, category_id])?;
    }
    conn.execute("DELETE FROM item_tags WHERE item_id = ?1", params![id])?;
    for tag_slug in &input.tag_slugs {
        let tag_id: String = match conn.query_row("SELECT id FROM tags WHERE slug = ?1", params![tag_slug], |r| r.get(0)) {
            Ok(existing) => existing,
            Err(_) => {
                let new_id = ulid();
                conn.execute(
                    "INSERT INTO tags (id, slug, name, accent_key) VALUES (?1,?2,?3,'blue')",
                    params![new_id, slugify(tag_slug), tag_slug],
                )?;
                new_id
            }
        };
        conn.execute("INSERT OR IGNORE INTO item_tags (item_id, tag_id) VALUES (?1,?2)", params![id, tag_id])?;
    }
    conn.execute("DELETE FROM examples WHERE item_id = ?1", params![id])?;
    for (index, example) in input.examples.iter().enumerate() {
        conn.execute(
            "INSERT INTO examples (id, item_id, title, description, code, expected_output, sort_order)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![ulid(), id, example.title, example.description, example.code, example.expected_output, index as i64],
        )?;
    }
    conn.execute("DELETE FROM parameters WHERE item_id = ?1", params![id])?;
    for (index, param) in input.parameters.iter().enumerate() {
        conn.execute(
            "INSERT OR REPLACE INTO parameters (id, item_id, name, description, is_required, default_value, sort_order)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![ulid(), id, param.name, param.description, param.is_required.unwrap_or(false) as i64,
                    param.default_value, index as i64],
        )?;
    }

    search::reindex_item(conn, &id)?;
    audit::record(conn, "user", "item.save", Some("item"), Some(&id), None, None)?;
    repo::item_detail(store, &slug)
}

/// Archive / restore. Deactivating hides a record everywhere without deleting it.
pub fn set_active(store: &Store, entity: &str, id: &str, active: bool) -> Result<()> {
    let table = match entity {
        "tool" => "tools",
        "task" => "tasks",
        "category" => "categories",
        "item" => "items",
        other => return Err(CoreError::Invalid(format!("unknown entity {other}"))),
    };
    let changed = store.conn().execute(
        &format!("UPDATE {table} SET is_active = ?2, updated_at = ?3 WHERE id = ?1"),
        params![id, active as i64, now_iso()],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound(format!("{entity} {id}")));
    }
    if entity == "item" {
        search::reindex_item(store.conn(), id)?;
    }
    audit::record(
        store.conn(),
        "user",
        if active { "entity.activate" } else { "entity.deactivate" },
        Some(entity),
        Some(id),
        None,
        None,
    )?;
    Ok(())
}
