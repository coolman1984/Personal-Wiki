//! Read model. Every list, card, tab, badge, and counter the UI renders comes
//! from a function in here — nothing is hard-coded in the frontend.

use crate::error::{CoreError, Result};
use crate::models::*;
use crate::Store;
use rusqlite::{params, params_from_iter, Connection, Row};
use std::collections::{BTreeMap, HashMap, HashSet};

const ITEM_COLS: &str = "i.id, i.slug, i.title, i.subtitle, i.short_description, i.item_type,
     i.syntax, i.verification_status, i.lifecycle_status, i.icon_key, i.accent_key,
     i.tool_id, t.slug, t.name, i.updated_at,
     EXISTS(SELECT 1 FROM favorites f WHERE f.item_id = i.id),
     EXISTS(SELECT 1 FROM pins p WHERE p.item_id = i.id),
     EXISTS(SELECT 1 FROM examples e WHERE e.item_id = i.id),
     EXISTS(SELECT 1 FROM item_sources s WHERE s.item_id = i.id)";

fn item_summary_from_row(row: &Row) -> rusqlite::Result<ItemSummary> {
    Ok(ItemSummary {
        id: row.get(0)?,
        slug: row.get(1)?,
        title: row.get(2)?,
        subtitle: row.get(3)?,
        short_description: row.get(4)?,
        item_type: row.get(5)?,
        syntax: row.get(6)?,
        verification_status: row.get(7)?,
        lifecycle_status: row.get(8)?,
        icon_key: row.get(9)?,
        accent_key: row.get(10)?,
        tool_id: row.get(11)?,
        tool_slug: row.get(12)?,
        tool_name: row.get(13)?,
        updated_at: row.get(14)?,
        is_favorite: row.get::<_, i64>(15)? == 1,
        is_pinned: row.get::<_, i64>(16)? == 1,
        has_examples: row.get::<_, i64>(17)? == 1,
        has_sources: row.get::<_, i64>(18)? == 1,
        tags: vec![],
    })
}

/// Attach tags to a batch of summaries in one query (no N+1).
fn hydrate_tags(conn: &Connection, items: &mut [ItemSummary]) -> Result<()> {
    if items.is_empty() {
        return Ok(());
    }
    let ids: Vec<String> = items.iter().map(|i| i.id.clone()).collect();
    let placeholders = vec!["?"; ids.len()].join(",");
    let sql = format!(
        "SELECT it.item_id, tg.id, tg.slug, tg.name, tg.accent_key
         FROM item_tags it JOIN tags tg ON tg.id = it.tag_id
         WHERE it.item_id IN ({placeholders}) ORDER BY tg.name"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(ids.iter()), |r| {
        Ok((
            r.get::<_, String>(0)?,
            Tag { id: r.get(1)?, slug: r.get(2)?, name: r.get(3)?, accent_key: r.get(4)? },
        ))
    })?;
    let mut map: HashMap<String, Vec<Tag>> = HashMap::new();
    for row in rows {
        let (item_id, tag) = row?;
        map.entry(item_id).or_default().push(tag);
    }
    for item in items.iter_mut() {
        if let Some(tags) = map.remove(&item.id) {
            item.tags = tags;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------- tools

pub fn list_tools(store: &Store, include_inactive: bool) -> Result<Vec<Tool>> {
    let sql = format!(
        "SELECT id, slug, name, short_name, description, icon_key, accent_key,
                official_home_url, sort_order, is_active, external_key, managed_by, updated_at
         FROM tools {} ORDER BY sort_order, name",
        if include_inactive { "" } else { "WHERE is_active = 1" }
    );
    let mut stmt = store.conn().prepare(&sql)?;
    let rows = stmt.query_map([], |r| {
        Ok(Tool {
            id: r.get(0)?,
            slug: r.get(1)?,
            name: r.get(2)?,
            short_name: r.get(3)?,
            description: r.get(4)?,
            icon_key: r.get(5)?,
            accent_key: r.get(6)?,
            official_home_url: r.get(7)?,
            sort_order: r.get(8)?,
            is_active: r.get::<_, i64>(9)? == 1,
            external_key: r.get(10)?,
            managed_by: r.get(11)?,
            updated_at: r.get(12)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

fn tool_stats(conn: &Connection, tool_id: &str) -> Result<ToolStats> {
    let row = conn.query_row(
        "SELECT
           (SELECT COUNT(*) FROM items i WHERE i.tool_id = ?1 AND i.is_active = 1),
           (SELECT COUNT(*) FROM examples e JOIN items i ON i.id = e.item_id WHERE i.tool_id = ?1),
           (SELECT COUNT(*) FROM notes n JOIN items i ON i.id = n.item_id WHERE i.tool_id = ?1),
           (SELECT COUNT(*) FROM sources s WHERE s.tool_id = ?1),
           (SELECT MAX(i.updated_at) FROM items i WHERE i.tool_id = ?1)",
        params![tool_id],
        |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get::<_, i64>(2)?, r.get::<_, i64>(3)?, r.get::<_, Option<String>>(4)?)),
    )?;
    Ok(ToolStats {
        item_count: row.0,
        example_count: row.1,
        note_count: row.2,
        source_count: row.3,
        last_updated_at: row.4,
    })
}

pub fn tool_cards(store: &Store) -> Result<Vec<ToolCard>> {
    let mut out = Vec::new();
    for tool in list_tools(store, false)? {
        let stats = tool_stats(store.conn(), &tool.id)?;
        out.push(ToolCard {
            item_count: stats.item_count,
            example_count: stats.example_count,
            note_count: stats.note_count,
            source_count: stats.source_count,
            last_updated_at: stats.last_updated_at,
            tool,
        });
    }
    Ok(out)
}

pub fn tool_by_slug(store: &Store, slug: &str) -> Result<Tool> {
    list_tools(store, true)?
        .into_iter()
        .find(|t| t.slug == slug)
        .ok_or_else(|| CoreError::NotFound(format!("tool {slug}")))
}

/// Everything the tool-family screen needs: identity, stat strip, section tabs
/// (the categories that actually contain this tool's items), and its tag set.
pub fn tool_page(store: &Store, slug: &str) -> Result<ToolPage> {
    let tool = tool_by_slug(store, slug)?;
    let stats = tool_stats(store.conn(), &tool.id)?;
    let sections = category_tree(store, Some(&tool.id))?;

    let mut stmt = store.conn().prepare(
        "SELECT DISTINCT tg.id, tg.slug, tg.name, tg.accent_key
         FROM tags tg JOIN item_tags it ON it.tag_id = tg.id
         JOIN items i ON i.id = it.item_id
         WHERE i.tool_id = ?1 AND i.is_active = 1 ORDER BY tg.name",
    )?;
    let rows = stmt.query_map(params![tool.id], |r| {
        Ok(Tag { id: r.get(0)?, slug: r.get(1)?, name: r.get(2)?, accent_key: r.get(3)? })
    })?;
    let tags = rows.collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(ToolPage { tool, stats, sections, tags })
}

// ---------------------------------------------------------------- tasks

pub fn list_tasks(store: &Store, include_inactive: bool) -> Result<Vec<Task>> {
    let sql = format!(
        "SELECT id, slug, name, description, icon_key, sort_order, is_active, external_key
         FROM tasks {} ORDER BY sort_order, name",
        if include_inactive { "" } else { "WHERE is_active = 1" }
    );
    let mut stmt = store.conn().prepare(&sql)?;
    let rows = stmt.query_map([], |r| {
        Ok(Task {
            id: r.get(0)?,
            slug: r.get(1)?,
            name: r.get(2)?,
            description: r.get(3)?,
            icon_key: r.get(4)?,
            sort_order: r.get(5)?,
            is_active: r.get::<_, i64>(6)? == 1,
            external_key: r.get(7)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn task_cards(store: &Store) -> Result<Vec<TaskCard>> {
    let mut out = Vec::new();
    for task in list_tasks(store, false)? {
        let (item_count, tool_count): (i64, i64) = store.conn().query_row(
            "SELECT COUNT(*), COUNT(DISTINCT i.tool_id)
             FROM item_tasks it JOIN items i ON i.id = it.item_id
             WHERE it.task_id = ?1 AND i.is_active = 1",
            params![task.id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let mut stmt = store.conn().prepare(
            "SELECT DISTINCT t.icon_key FROM item_tasks it
             JOIN items i ON i.id = it.item_id JOIN tools t ON t.id = i.tool_id
             WHERE it.task_id = ?1 AND i.is_active = 1 LIMIT 5",
        )?;
        let icons = stmt
            .query_map(params![task.id], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        out.push(TaskCard { task, item_count, tool_count, tool_icons: icons });
    }
    Ok(out)
}

pub fn task_by_slug(store: &Store, slug: &str) -> Result<Task> {
    list_tasks(store, true)?
        .into_iter()
        .find(|t| t.slug == slug)
        .ok_or_else(|| CoreError::NotFound(format!("task {slug}")))
}

// ---------------------------------------------------------------- categories

fn all_categories(conn: &Connection) -> Result<Vec<Category>> {
    let mut stmt = conn.prepare(
        "SELECT id, slug, name, description, icon_key, accent_key, parent_id, sort_order, is_active, external_key
         FROM categories WHERE is_active = 1 ORDER BY sort_order, name",
    )?;
    let rows = stmt.query_map([], |r| {
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
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// Recursive category tree at arbitrary depth. When `tool_id` is given, only
/// branches that actually contain that tool's items are kept — that is what
/// produces the section tabs on a tool page.
pub fn category_tree(store: &Store, tool_id: Option<&str>) -> Result<Vec<CategoryNode>> {
    let conn = store.conn();
    let categories = all_categories(conn)?;

    let mut counts: HashMap<String, i64> = HashMap::new();
    {
        let sql = match tool_id {
            Some(_) => "SELECT ic.category_id, COUNT(*) FROM item_categories ic
                        JOIN items i ON i.id = ic.item_id
                        WHERE i.is_active = 1 AND i.tool_id = ?1 GROUP BY ic.category_id",
            None => "SELECT ic.category_id, COUNT(*) FROM item_categories ic
                     JOIN items i ON i.id = ic.item_id WHERE i.is_active = 1 GROUP BY ic.category_id",
        };
        let mut stmt = conn.prepare(sql)?;
        let args: Vec<&dyn rusqlite::ToSql> = match tool_id.as_ref() {
            Some(id) => vec![id],
            None => vec![],
        };
        let rows = stmt.query_map(args.as_slice(), |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
        for row in rows {
            let (id, n) = row?;
            counts.insert(id, n);
        }
    }

    let mut children: HashMap<Option<String>, Vec<Category>> = HashMap::new();
    for cat in categories {
        children.entry(cat.parent_id.clone()).or_default().push(cat);
    }

    fn build(
        parent: Option<String>,
        children: &HashMap<Option<String>, Vec<Category>>,
        counts: &HashMap<String, i64>,
        depth: usize,
    ) -> Vec<CategoryNode> {
        if depth > 12 {
            return vec![]; // cycle guard; import validation rejects cycles anyway
        }
        children
            .get(&parent)
            .map(|list| {
                list.iter()
                    .map(|cat| {
                        let kids = build(Some(cat.id.clone()), children, counts, depth + 1);
                        let own = counts.get(&cat.id).copied().unwrap_or(0);
                        let total = own + kids.iter().map(|k| k.item_count).sum::<i64>();
                        CategoryNode { category: cat.clone(), item_count: total, children: kids }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    let mut tree = build(None, &children, &counts, 0);
    if tool_id.is_some() {
        fn prune(nodes: Vec<CategoryNode>) -> Vec<CategoryNode> {
            nodes
                .into_iter()
                .filter_map(|mut n| {
                    n.children = prune(std::mem::take(&mut n.children));
                    if n.item_count > 0 {
                        Some(n)
                    } else {
                        None
                    }
                })
                .collect()
        }
        tree = prune(tree);
    }
    Ok(tree)
}

pub fn category_by_slug(store: &Store, slug: &str) -> Result<Category> {
    all_categories(store.conn())?
        .into_iter()
        .find(|c| c.slug == slug)
        .ok_or_else(|| CoreError::NotFound(format!("category {slug}")))
}

/// Breadcrumb path from the root down to `slug`.
pub fn category_path(store: &Store, slug: &str) -> Result<Vec<Category>> {
    let all = all_categories(store.conn())?;
    let by_id: HashMap<String, Category> = all.iter().map(|c| (c.id.clone(), c.clone())).collect();
    let mut current = all
        .iter()
        .find(|c| c.slug == slug)
        .cloned()
        .ok_or_else(|| CoreError::NotFound(format!("category {slug}")))?;
    let mut path = vec![current.clone()];
    let mut guard = 0;
    while let Some(parent_id) = current.parent_id.clone() {
        guard += 1;
        if guard > 16 {
            break;
        }
        match by_id.get(&parent_id) {
            Some(parent) => {
                path.push(parent.clone());
                current = parent.clone();
            }
            None => break,
        }
    }
    path.reverse();
    Ok(path)
}

fn category_subtree_ids(conn: &Connection, slug: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "WITH RECURSIVE subtree(id) AS (
           SELECT id FROM categories WHERE slug = ?1
           UNION ALL
           SELECT c.id FROM categories c JOIN subtree s ON c.parent_id = s.id
         ) SELECT id FROM subtree",
    )?;
    let rows = stmt.query_map(params![slug], |r| r.get::<_, String>(0))?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

// ---------------------------------------------------------------- items

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemSort {
    Recent,
    Alphabetical,
    Popular,
}

impl ItemSort {
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("alpha") | Some("alphabetical") => ItemSort::Alphabetical,
            Some("popular") => ItemSort::Popular,
            _ => ItemSort::Recent,
        }
    }
    fn order_clause(self) -> &'static str {
        match self {
            ItemSort::Recent => "ORDER BY i.updated_at DESC, i.title",
            ItemSort::Alphabetical => "ORDER BY i.title",
            ItemSort::Popular => {
                "ORDER BY (SELECT COALESCE(open_count,0) FROM recent_items r WHERE r.item_id = i.id) DESC, i.title"
            }
        }
    }
}

fn run_item_query(store: &Store, sql: &str, args: &[&dyn rusqlite::ToSql]) -> Result<Vec<ItemSummary>> {
    let mut stmt = store.conn().prepare(sql)?;
    let rows = stmt.query_map(args, item_summary_from_row)?;
    let mut items = rows.collect::<std::result::Result<Vec<_>, _>>()?;
    hydrate_tags(store.conn(), &mut items)?;
    Ok(items)
}

pub fn items_for_tool(
    store: &Store,
    tool_slug: &str,
    category_slug: Option<&str>,
    sort: ItemSort,
    limit: i64,
) -> Result<Vec<ItemSummary>> {
    let tool = tool_by_slug(store, tool_slug)?;
    match category_slug {
        Some(cat) => {
            let ids = category_subtree_ids(store.conn(), cat)?;
            if ids.is_empty() {
                return Ok(vec![]);
            }
            let placeholders = vec!["?"; ids.len()].join(",");
            let sql = format!(
                "SELECT {ITEM_COLS} FROM items i LEFT JOIN tools t ON t.id = i.tool_id
                 WHERE i.is_active = 1 AND i.tool_id = ?1
                   AND EXISTS (SELECT 1 FROM item_categories ic
                               WHERE ic.item_id = i.id AND ic.category_id IN ({placeholders}))
                 {} LIMIT ?{}",
                sort.order_clause(),
                ids.len() + 2
            );
            let mut args: Vec<&dyn rusqlite::ToSql> = vec![&tool.id];
            for id in &ids {
                args.push(id);
            }
            args.push(&limit);
            run_item_query(store, &sql, &args)
        }
        None => {
            let sql = format!(
                "SELECT {ITEM_COLS} FROM items i LEFT JOIN tools t ON t.id = i.tool_id
                 WHERE i.is_active = 1 AND i.tool_id = ?1 {} LIMIT ?2",
                sort.order_clause()
            );
            run_item_query(store, &sql, &[&tool.id, &limit])
        }
    }
}

pub fn items_for_category(store: &Store, slug: &str, sort: ItemSort, limit: i64) -> Result<Vec<ItemSummary>> {
    let ids = category_subtree_ids(store.conn(), slug)?;
    if ids.is_empty() {
        return Ok(vec![]);
    }
    let placeholders = vec!["?"; ids.len()].join(",");
    let sql = format!(
        "SELECT {ITEM_COLS} FROM items i LEFT JOIN tools t ON t.id = i.tool_id
         WHERE i.is_active = 1
           AND EXISTS (SELECT 1 FROM item_categories ic
                       WHERE ic.item_id = i.id AND ic.category_id IN ({placeholders}))
         {} LIMIT ?{}",
        sort.order_clause(),
        ids.len() + 1
    );
    let mut args: Vec<&dyn rusqlite::ToSql> = Vec::new();
    for id in &ids {
        args.push(id);
    }
    args.push(&limit);
    run_item_query(store, &sql, &args)
}

pub fn items_for_task(store: &Store, slug: &str, sort: ItemSort, limit: i64) -> Result<Vec<ItemSummary>> {
    let task = task_by_slug(store, slug)?;
    let sql = format!(
        "SELECT {ITEM_COLS} FROM items i LEFT JOIN tools t ON t.id = i.tool_id
         JOIN item_tasks it ON it.item_id = i.id
         WHERE i.is_active = 1 AND it.task_id = ?1 {} LIMIT ?2",
        sort.order_clause()
    );
    run_item_query(store, &sql, &[&task.id, &limit])
}

pub fn items_by_ids(
    store: &Store,
    ids: &[String],
    filters: &SearchFilters,
) -> Result<HashMap<String, ItemSummary>> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let placeholders = vec!["?"; ids.len()].join(",");
    let mut clauses = vec![format!("i.id IN ({placeholders})"), "i.is_active = 1".to_string()];

    let mut extra: Vec<String> = Vec::new();
    let push_in = |column: &str, values: &[String], clauses: &mut Vec<String>, extra: &mut Vec<String>| {
        if values.is_empty() {
            return;
        }
        let marks = vec!["?"; values.len()].join(",");
        clauses.push(format!("{column} IN ({marks})"));
        extra.extend(values.iter().cloned());
    };
    push_in("t.slug", &filters.tool_slugs, &mut clauses, &mut extra);
    push_in("i.item_type", &filters.item_types, &mut clauses, &mut extra);
    push_in("i.verification_status", &filters.verification_statuses, &mut clauses, &mut extra);
    push_in("i.lifecycle_status", &filters.lifecycle_statuses, &mut clauses, &mut extra);

    if !filters.task_slugs.is_empty() {
        let marks = vec!["?"; filters.task_slugs.len()].join(",");
        clauses.push(format!(
            "EXISTS (SELECT 1 FROM item_tasks it JOIN tasks tk ON tk.id = it.task_id
                     WHERE it.item_id = i.id AND tk.slug IN ({marks}))"
        ));
        extra.extend(filters.task_slugs.iter().cloned());
    }
    if !filters.category_slugs.is_empty() {
        let marks = vec!["?"; filters.category_slugs.len()].join(",");
        clauses.push(format!(
            "EXISTS (SELECT 1 FROM item_categories ic JOIN categories c ON c.id = ic.category_id
                     WHERE ic.item_id = i.id AND c.slug IN ({marks}))"
        ));
        extra.extend(filters.category_slugs.iter().cloned());
    }
    if !filters.tag_slugs.is_empty() {
        let marks = vec!["?"; filters.tag_slugs.len()].join(",");
        clauses.push(format!(
            "EXISTS (SELECT 1 FROM item_tags itg JOIN tags tg ON tg.id = itg.tag_id
                     WHERE itg.item_id = i.id AND tg.slug IN ({marks}))"
        ));
        extra.extend(filters.tag_slugs.iter().cloned());
    }
    if filters.favorites_only {
        clauses.push("EXISTS (SELECT 1 FROM favorites f WHERE f.item_id = i.id)".to_string());
    }

    let sql = format!(
        "SELECT {ITEM_COLS} FROM items i LEFT JOIN tools t ON t.id = i.tool_id WHERE {}",
        clauses.join(" AND ")
    );
    let all_args: Vec<&dyn rusqlite::ToSql> = ids
        .iter()
        .chain(extra.iter())
        .map(|s| s as &dyn rusqlite::ToSql)
        .collect();
    let items = run_item_query(store, &sql, &all_args)?;
    Ok(items.into_iter().map(|i| (i.id.clone(), i)).collect())
}

pub fn aliases_for(store: &Store, item_id: &str) -> Result<Vec<String>> {
    let mut stmt = store.conn().prepare("SELECT alias FROM aliases WHERE item_id = ?1")?;
    let rows = stmt.query_map(params![item_id], |r| r.get::<_, String>(0))?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn item_matches_task_text(store: &Store, item_id: &str, normalized: &str) -> Result<bool> {
    let mut stmt = store.conn().prepare(
        "SELECT tk.name FROM tasks tk JOIN item_tasks it ON it.task_id = tk.id WHERE it.item_id = ?1",
    )?;
    let rows = stmt.query_map(params![item_id], |r| r.get::<_, String>(0))?;
    for row in rows {
        if crate::search::normalize(&row?).contains(normalized) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn is_recent(store: &Store, item_id: &str) -> Result<bool> {
    let n: i64 = store.conn().query_row(
        "SELECT COUNT(*) FROM recent_items WHERE item_id = ?1",
        params![item_id],
        |r| r.get(0),
    )?;
    Ok(n > 0)
}

pub fn item_detail(store: &Store, slug: &str) -> Result<ItemDetail> {
    let conn = store.conn();
    let sql = format!(
        "SELECT {ITEM_COLS}, i.full_description, i.when_to_use, i.expected_result, i.warning_text,
                i.result_diagram, i.difficulty, i.last_verified_at, i.managed_by
         FROM items i LEFT JOIN tools t ON t.id = i.tool_id WHERE i.slug = ?1"
    );
    let (mut summary, full_description, when_to_use, expected_result, warning_text, diagram_json, difficulty, last_verified_at, managed_by) =
        conn.query_row(&sql, params![slug], |r| {
            Ok((
                item_summary_from_row(r)?,
                r.get::<_, Option<String>>(19)?,
                r.get::<_, Option<String>>(20)?,
                r.get::<_, Option<String>>(21)?,
                r.get::<_, Option<String>>(22)?,
                r.get::<_, Option<String>>(23)?,
                r.get::<_, Option<String>>(24)?,
                r.get::<_, Option<String>>(25)?,
                r.get::<_, String>(26)?,
            ))
        })
        .map_err(|_| CoreError::NotFound(format!("item {slug}")))?;

    let mut one = vec![std::mem::replace(&mut summary, ItemSummary {
        id: String::new(), slug: String::new(), title: String::new(), subtitle: None,
        short_description: None, item_type: String::new(), syntax: None,
        verification_status: String::new(), lifecycle_status: String::new(), icon_key: None,
        accent_key: None, tool_id: None, tool_slug: None, tool_name: None, tags: vec![],
        is_favorite: false, is_pinned: false, has_examples: false, has_sources: false,
        updated_at: String::new(),
    })];
    hydrate_tags(conn, &mut one)?;
    let summary = one.remove(0);
    let item_id = summary.id.clone();

    let mut stmt = conn.prepare(
        "SELECT id, title, description, code, expected_output, sort_order
         FROM examples WHERE item_id = ?1 ORDER BY sort_order, id",
    )?;
    let examples = stmt
        .query_map(params![item_id], |r| {
            Ok(Example {
                id: r.get(0)?,
                title: r.get(1)?,
                description: r.get(2)?,
                code: r.get(3)?,
                expected_output: r.get(4)?,
                sort_order: r.get(5)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT id, name, description, is_required, default_value, value_hint, sort_order
         FROM parameters WHERE item_id = ?1 ORDER BY sort_order, name",
    )?;
    let parameters = stmt
        .query_map(params![item_id], |r| {
            Ok(Parameter {
                id: r.get(0)?,
                name: r.get(1)?,
                description: r.get(2)?,
                is_required: r.get::<_, i64>(3)? == 1,
                default_value: r.get(4)?,
                value_hint: r.get(5)?,
                sort_order: r.get(6)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT s.id, s.name, s.source_type, s.is_official, s.url,
                isr.source_url, isr.source_anchor, isr.source_version, isr.verified_at
         FROM item_sources isr JOIN sources s ON s.id = isr.source_id
         WHERE isr.item_id = ?1",
    )?;
    let sources = stmt
        .query_map(params![item_id], |r| {
            Ok(ItemSource {
                source_id: r.get(0)?,
                name: r.get(1)?,
                source_type: r.get(2)?,
                is_official: r.get::<_, i64>(3)? == 1,
                url: r.get(4)?,
                source_url: r.get(5)?,
                source_anchor: r.get(6)?,
                source_version: r.get(7)?,
                verified_at: r.get(8)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT i2.id, i2.slug, i2.title, r.relationship_type, t.slug, i2.icon_key, i2.accent_key
         FROM relationships r JOIN items i2 ON i2.id = r.to_item_id
         LEFT JOIN tools t ON t.id = i2.tool_id
         WHERE r.from_item_id = ?1 AND i2.is_active = 1",
    )?;
    let related = stmt
        .query_map(params![item_id], |r| {
            Ok(RelatedItem {
                id: r.get(0)?,
                slug: r.get(1)?,
                title: r.get(2)?,
                relationship_type: r.get(3)?,
                tool_slug: r.get(4)?,
                icon_key: r.get(5)?,
                accent_key: r.get(6)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT tk.id, tk.slug, tk.name, tk.description, tk.icon_key, tk.sort_order, tk.is_active, tk.external_key
         FROM tasks tk JOIN item_tasks it ON it.task_id = tk.id WHERE it.item_id = ?1",
    )?;
    let tasks = stmt
        .query_map(params![item_id], |r| {
            Ok(Task {
                id: r.get(0)?,
                slug: r.get(1)?,
                name: r.get(2)?,
                description: r.get(3)?,
                icon_key: r.get(4)?,
                sort_order: r.get(5)?,
                is_active: r.get::<_, i64>(6)? == 1,
                external_key: r.get(7)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT c.id, c.slug, c.name, c.description, c.icon_key, c.accent_key, c.parent_id,
                c.sort_order, c.is_active, c.external_key
         FROM categories c JOIN item_categories ic ON ic.category_id = c.id WHERE ic.item_id = ?1",
    )?;
    let categories = stmt
        .query_map(params![item_id], |r| {
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
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(ItemDetail {
        aliases: aliases_for(store, &item_id)?,
        notes: crate::personal::notes_for_item(store, &item_id)?,
        result_diagram: diagram_json.and_then(|j| serde_json::from_str(&j).ok()),
        summary,
        full_description,
        when_to_use,
        expected_result,
        warning_text,
        difficulty,
        last_verified_at,
        managed_by,
        examples,
        parameters,
        sources,
        related,
        tasks,
        categories,
    })
}

// ---------------------------------------------------------------- reference data

pub fn list_tags(store: &Store) -> Result<Vec<Tag>> {
    let mut stmt = store
        .conn()
        .prepare("SELECT id, slug, name, accent_key FROM tags ORDER BY name")?;
    let rows = stmt.query_map([], |r| {
        Ok(Tag { id: r.get(0)?, slug: r.get(1)?, name: r.get(2)?, accent_key: r.get(3)? })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// The item types actually present in the database — the type filter is data-driven
/// like everything else, not a literal list in the frontend.
pub fn list_item_types(store: &Store) -> Result<Vec<String>> {
    let mut stmt = store
        .conn()
        .prepare("SELECT DISTINCT item_type FROM items WHERE is_active = 1 ORDER BY item_type")?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

fn derive_health(is_active: bool, last_success: &Option<String>, last_failure: &Option<String>) -> String {
    if !is_active {
        return "disabled".into();
    }
    match (last_success, last_failure) {
        (None, None) => "unchecked".into(),
        (Some(_), None) => "ok".into(),
        (None, Some(_)) => "failing".into(),
        (Some(s), Some(f)) => {
            if f > s {
                "failing".into()
            } else {
                "ok".into()
            }
        }
    }
}

pub fn list_sources(store: &Store) -> Result<Vec<Source>> {
    let mut stmt = store.conn().prepare(
        "SELECT s.id, s.tool_id, t.name, s.name, s.source_type, s.url, s.is_official,
                s.sync_strategy, s.is_active, s.check_interval_minutes, s.last_checked_at,
                s.last_success_at, s.last_failure_at, s.last_error, s.next_check_at, s.notes
         FROM sources s LEFT JOIN tools t ON t.id = s.tool_id
         ORDER BY s.is_active DESC, s.name",
    )?;
    let rows = stmt.query_map([], |r| {
        let is_active = r.get::<_, i64>(8)? == 1;
        let last_success: Option<String> = r.get(11)?;
        let last_failure: Option<String> = r.get(12)?;
        Ok(Source {
            id: r.get(0)?,
            tool_id: r.get(1)?,
            tool_name: r.get(2)?,
            name: r.get(3)?,
            source_type: r.get(4)?,
            url: r.get(5)?,
            is_official: r.get::<_, i64>(6)? == 1,
            sync_strategy: r.get(7)?,
            is_active,
            check_interval_minutes: r.get(9)?,
            last_checked_at: r.get(10)?,
            health: derive_health(is_active, &last_success, &last_failure),
            last_success_at: last_success,
            last_failure_at: last_failure,
            last_error: r.get(13)?,
            next_check_at: r.get(14)?,
            notes: r.get(15)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

// ---------------------------------------------------------------- home

pub fn home_summary(store: &Store) -> Result<HomeSummary> {
    let conn = store.conn();
    let tools = tool_cards(store)?;
    let tasks = task_cards(store)?;

    let sql = format!(
        "SELECT {ITEM_COLS} FROM items i LEFT JOIN tools t ON t.id = i.tool_id
         JOIN recent_items r ON r.item_id = i.id
         WHERE i.is_active = 1 ORDER BY r.opened_at DESC LIMIT 6"
    );
    let recent_items = run_item_query(store, &sql, &[])?;

    let mut stmt = conn.prepare(
        "SELECT sc.id, sc.change_type, COALESCE(sc.title, sc.entity_key), sc.detail,
                sc.applied_at, sc.detected_at, i.slug, t.slug, t.icon_key, t.accent_key
         FROM staged_changes sc
         LEFT JOIN items i ON i.id = sc.entity_id
         LEFT JOIN tools t ON t.id = i.tool_id
         WHERE sc.status = 'applied'
         ORDER BY COALESCE(sc.applied_at, sc.detected_at) DESC LIMIT 6",
    )?;
    let recent_activity = stmt
        .query_map([], |r| {
            let applied: Option<String> = r.get(4)?;
            let detected: String = r.get(5)?;
            Ok(ActivityEntry {
                id: r.get(0)?,
                kind: r.get(1)?,
                title: r.get(2)?,
                detail: r.get(3)?,
                item_slug: r.get(6)?,
                tool_slug: r.get(7)?,
                icon_key: r.get(8)?,
                accent_key: r.get(9)?,
                at: applied.unwrap_or(detected),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let pending_changes: i64 =
        conn.query_row("SELECT COUNT(*) FROM staged_changes WHERE status = 'pending'", [], |r| r.get(0))?;
    let total_items: i64 =
        conn.query_row("SELECT COUNT(*) FROM items WHERE is_active = 1", [], |r| r.get(0))?;
    let total_examples: i64 = conn.query_row("SELECT COUNT(*) FROM examples", [], |r| r.get(0))?;
    let last_sync: Option<(String, String)> = conn
        .query_row(
            "SELECT COALESCE(finished_at, started_at), status FROM sync_runs ORDER BY started_at DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok();

    Ok(HomeSummary {
        tools,
        tasks,
        recent_items,
        recent_activity,
        pending_changes,
        total_items,
        total_examples,
        last_sync_at: last_sync.as_ref().map(|s| s.0.clone()),
        last_sync_status: last_sync.map(|s| s.1),
    })
}

/// Counts used by Diagnostics and by `personal-wiki status`.
pub fn table_counts(store: &Store) -> Result<BTreeMap<String, i64>> {
    let tables = [
        "tools", "tasks", "categories", "items", "examples", "parameters", "aliases", "tags",
        "sources", "item_sources", "relationships", "favorites", "pins", "notes", "saved_links",
        "update_batches", "staged_changes", "backups", "audit_log",
    ];
    let mut map = BTreeMap::new();
    for table in tables {
        map.insert(table.to_string(), store.count(table)?);
    }
    Ok(map)
}

/// Slugs currently used, so importers can detect collisions cheaply.
pub fn existing_slugs(conn: &Connection, table: &str) -> Result<HashSet<String>> {
    let mut stmt = conn.prepare(&format!("SELECT slug FROM {table}"))?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
    Ok(rows.collect::<std::result::Result<HashSet<_>, _>>()?)
}

/// The personal collections, rendered with the same `ItemSummary` shape as
/// every other list.
pub fn favorites(store: &Store) -> Result<Vec<ItemSummary>> {
    let sql = format!(
        "SELECT {ITEM_COLS} FROM items i LEFT JOIN tools t ON t.id = i.tool_id
         JOIN favorites f ON f.item_id = i.id
         WHERE i.is_active = 1 ORDER BY f.created_at DESC"
    );
    run_item_query(store, &sql, &[])
}

pub fn pinned(store: &Store) -> Result<Vec<ItemSummary>> {
    let sql = format!(
        "SELECT {ITEM_COLS} FROM items i LEFT JOIN tools t ON t.id = i.tool_id
         JOIN pins p ON p.item_id = i.id
         WHERE i.is_active = 1 ORDER BY p.sort_order, p.created_at DESC"
    );
    run_item_query(store, &sql, &[])
}
