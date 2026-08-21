//! FTS5 search: normalization, index maintenance, query, ranking.
//!
//! Ranking contract (docs/UI_SPEC.md §8):
//!   bm25 base
//!   + 40 exact title      + 30 exact alias
//!   + 18 syntax prefix    + 12 task match    + 8 tool match
//!   +  3 favourite        +  2 recently opened
//!   - 25 deprecated       - 60 removed

use crate::error::Result;
use crate::models::{SearchFilters, SearchHit, SearchResponse};
use crate::repo;
use crate::Store;
use rusqlite::{params, Connection};
use std::time::Instant;

/// Fold text so Arabic orthographic variants, diacritics, and digit forms all
/// collapse to one comparable form. Applied to indexed text *and* to queries.
pub fn normalize(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut last_space = true;
    for ch in input.chars() {
        let mapped = match ch {
            // diacritics / tatweel / superscript alef — dropped
            '\u{064B}'..='\u{0652}' | '\u{0653}'..='\u{0655}' | '\u{0670}' | '\u{0640}' => continue,
            // zero-width marks
            '\u{200B}'..='\u{200F}' | '\u{FEFF}' => continue,
            // alef family
            'أ' | 'إ' | 'آ' | 'ٱ' | 'ٲ' | 'ٳ' => 'ا',
            // ya / alef maqsura
            'ى' | 'ئ' => 'ي',
            // ta marbuta
            'ة' => 'ه',
            // waw with hamza
            'ؤ' => 'و',
            'ء' => continue,
            // arabic-indic digits
            '\u{0660}'..='\u{0669}' => (b'0' + (ch as u32 - 0x0660) as u8) as char,
            '\u{06F0}'..='\u{06F9}' => (b'0' + (ch as u32 - 0x06F0) as u8) as char,
            other => other,
        };
        if mapped.is_whitespace() {
            if !last_space {
                out.push(' ');
                last_space = true;
            }
        } else {
            for l in mapped.to_lowercase() {
                out.push(l);
            }
            last_space = false;
        }
    }
    out.trim().to_string()
}

/// Text flattened into one FTS row for an item.
struct IndexRow {
    item_id: String,
    title: String,
    aliases: String,
    short_description: String,
    full_description: String,
    task_phrases: String,
    tags: String,
    syntax: String,
    examples: String,
    tool: String,
    category: String,
}

fn collect_row(conn: &Connection, item_id: &str) -> Result<Option<IndexRow>> {
    let base: Option<(String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> = conn
        .query_row(
            "SELECT i.title, i.short_description, i.full_description, i.syntax, i.when_to_use, t.name
             FROM items i LEFT JOIN tools t ON t.id = i.tool_id
             WHERE i.id = ?1",
            params![item_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
        )
        .ok();
    let Some((title, short_desc, full_desc, syntax, when_to_use, tool)) = base else {
        return Ok(None);
    };

    let join = |sql: &str| -> Result<String> {
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params![item_id], |r| r.get::<_, String>(0))?;
        let mut parts = Vec::new();
        for row in rows {
            parts.push(row?);
        }
        Ok(parts.join(" \u{2022} "))
    };

    let aliases = join("SELECT alias FROM aliases WHERE item_id = ?1")?;
    let tags = join("SELECT t.name || ' ' || t.slug FROM tags t JOIN item_tags it ON it.tag_id = t.id WHERE it.item_id = ?1")?;
    let task_phrases = join(
        "SELECT tk.name || ' ' || COALESCE(tk.description,'') || ' ' || tk.slug
         FROM tasks tk JOIN item_tasks it ON it.task_id = tk.id WHERE it.item_id = ?1",
    )?;
    let category = join(
        "SELECT c.name || ' ' || c.slug FROM categories c
         JOIN item_categories ic ON ic.category_id = c.id WHERE ic.item_id = ?1",
    )?;
    let examples = join(
        "SELECT COALESCE(title,'') || ' ' || COALESCE(description,'') || ' ' || code
         FROM examples WHERE item_id = ?1",
    )?;

    Ok(Some(IndexRow {
        item_id: item_id.to_string(),
        title: normalize(&title),
        aliases: normalize(&aliases),
        short_description: normalize(short_desc.as_deref().unwrap_or("")),
        full_description: normalize(&format!(
            "{} {}",
            full_desc.as_deref().unwrap_or(""),
            when_to_use.as_deref().unwrap_or("")
        )),
        task_phrases: normalize(&task_phrases),
        tags: normalize(&tags),
        syntax: normalize(syntax.as_deref().unwrap_or("")),
        examples: normalize(&examples),
        tool: normalize(tool.as_deref().unwrap_or("")),
        category: normalize(&category),
    }))
}

fn write_row(conn: &Connection, row: &IndexRow) -> Result<()> {
    conn.execute(
        "INSERT INTO items_fts
           (item_id, title, aliases, short_description, full_description,
            task_phrases, tags, syntax, examples, tool, category)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        params![
            row.item_id,
            row.title,
            row.aliases,
            row.short_description,
            row.full_description,
            row.task_phrases,
            row.tags,
            row.syntax,
            row.examples,
            row.tool,
            row.category
        ],
    )?;
    Ok(())
}

/// Re-project one item into the index. Safe to call repeatedly; usable inside an
/// open transaction (apply calls it for every touched item before COMMIT).
pub fn reindex_item(conn: &Connection, item_id: &str) -> Result<()> {
    conn.execute("DELETE FROM items_fts WHERE item_id = ?1", params![item_id])?;
    if let Some(row) = collect_row(conn, item_id)? {
        write_row(conn, &row)?;
    }
    Ok(())
}

pub fn remove_item(conn: &Connection, item_id: &str) -> Result<()> {
    conn.execute("DELETE FROM items_fts WHERE item_id = ?1", params![item_id])?;
    Ok(())
}

/// Full rebuild. Used by `doctor --repair`, after a restore, and by the UI's
/// "rebuild index" recovery action.
pub fn rebuild(conn: &Connection) -> Result<i64> {
    conn.execute("DELETE FROM items_fts", [])?;
    let ids: Vec<String> = {
        let mut stmt = conn.prepare("SELECT id FROM items")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()?
    };
    let mut n = 0;
    for id in ids {
        if let Some(row) = collect_row(conn, &id)? {
            write_row(conn, &row)?;
            n += 1;
        }
    }
    Ok(n)
}

/// Is the index usable and in step with `items`?
pub fn health(conn: &Connection) -> (String, i64) {
    let indexed: i64 = conn
        .query_row("SELECT COUNT(*) FROM items_fts", [], |r| r.get(0))
        .unwrap_or(-1);
    if indexed < 0 {
        return ("unavailable".into(), 0);
    }
    let items: i64 = conn
        .query_row("SELECT COUNT(*) FROM items", [], |r| r.get(0))
        .unwrap_or(0);
    let status = if indexed == items { "ok" } else { "stale" };
    (status.into(), indexed)
}

/// Build an FTS5 MATCH expression from a normalized user query.
/// Every token is quoted (so punctuation can never inject syntax) and given a
/// prefix wildcard so partial words match.
fn match_expression(normalized: &str, all_terms: bool) -> Option<String> {
    let tokens: Vec<String> = normalized
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|t| !t.is_empty())
        .map(|t| format!("\"{}\"*", t.replace('"', "")))
        .collect();
    if tokens.is_empty() {
        return None;
    }
    Some(tokens.join(if all_terms { " AND " } else { " OR " }))
}

const BM25_WEIGHTS: &str = "0.0, 10.0, 8.0, 4.0, 2.0, 5.0, 3.0, 6.0, 2.0, 2.0, 2.0";

fn candidate_scores(conn: &Connection, expr: &str) -> Result<Vec<(String, f64)>> {
    let sql = format!(
        "SELECT item_id, bm25(items_fts, {BM25_WEIGHTS}) AS rank
         FROM items_fts WHERE items_fts MATCH ?1 ORDER BY rank LIMIT 500"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![expr], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?))
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// Run a search. Deterministic, offline, no model involved.
pub fn search(
    store: &Store,
    query: &str,
    filters: &SearchFilters,
    limit: i64,
    offset: i64,
) -> Result<SearchResponse> {
    let started = Instant::now();
    let normalized = normalize(query);
    let conn = store.conn();

    if normalized.is_empty() {
        return Ok(SearchResponse {
            query: query.to_string(),
            normalized_query: normalized,
            total: 0,
            took_ms: started.elapsed().as_secs_f64() * 1000.0,
            hits: vec![],
        });
    }

    // Precision first: require every token. Fall back to any-token if that finds nothing.
    let mut candidates = match match_expression(&normalized, true) {
        Some(expr) => candidate_scores(conn, &expr)?,
        None => vec![],
    };
    if candidates.is_empty() {
        if let Some(expr) = match_expression(&normalized, false) {
            candidates = candidate_scores(conn, &expr)?;
        }
    }
    if candidates.is_empty() {
        return Ok(SearchResponse {
            query: query.to_string(),
            normalized_query: normalized,
            total: 0,
            took_ms: started.elapsed().as_secs_f64() * 1000.0,
            hits: vec![],
        });
    }

    let ids: Vec<String> = candidates.iter().map(|(id, _)| id.clone()).collect();
    let summaries = repo::items_by_ids(store, &ids, filters)?;

    let mut hits: Vec<SearchHit> = Vec::with_capacity(summaries.len());
    for (id, bm25) in &candidates {
        let Some(summary) = summaries.get(id) else {
            continue;
        };

        // bm25() returns a negative number; more negative means a better match.
        let mut score = -bm25;

        let title_n = normalize(&summary.title);
        if title_n == normalized {
            score += 40.0;
        } else if title_n.starts_with(&normalized) {
            score += 14.0;
        }

        let mut matched_alias = None;
        for alias in repo::aliases_for(store, id)? {
            let alias_n = normalize(&alias);
            if alias_n == normalized {
                score += 30.0;
                matched_alias = Some(alias.clone());
                break;
            } else if alias_n.contains(&normalized) && matched_alias.is_none() {
                score += 10.0;
                matched_alias = Some(alias.clone());
            }
        }

        if let Some(syntax) = &summary.syntax {
            let syn = normalize(syntax);
            if syn.starts_with(&normalized) {
                score += 18.0;
            } else if syn.contains(&normalized) {
                score += 6.0;
            }
        }

        if repo::item_matches_task_text(store, id, &normalized)? {
            score += 12.0;
        }
        if let Some(tool_name) = &summary.tool_name {
            if normalize(tool_name).contains(&normalized) {
                score += 8.0;
            }
        }
        if summary.is_favorite {
            score += 3.0;
        }
        if repo::is_recent(store, id)? {
            score += 2.0;
        }
        match summary.lifecycle_status.as_str() {
            "deprecated" => score -= 25.0,
            "removed" => score -= 60.0,
            _ => {}
        }

        hits.push(SearchHit {
            item: summary.clone(),
            score,
            relevance: 0,
            matched_alias,
        });
    }

    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let best = hits.first().map(|h| h.score).unwrap_or(1.0).max(1.0);
    for hit in &mut hits {
        let pct = (hit.score / best * 98.0).round() as i64;
        hit.relevance = pct.clamp(1, 99);
    }

    let total = hits.len() as i64;
    let page: Vec<SearchHit> = hits
        .into_iter()
        .skip(offset.max(0) as usize)
        .take(limit.max(1) as usize)
        .collect();

    Ok(SearchResponse {
        query: query.to_string(),
        normalized_query: normalized,
        total,
        took_ms: started.elapsed().as_secs_f64() * 1000.0,
        hits: page,
    })
}

/// Alias- and task-phrase completions for the search suggestions panel.
pub fn suggestions(store: &Store, query: &str, limit: i64) -> Result<Vec<String>> {
    let normalized = normalize(query);
    if normalized.is_empty() {
        return Ok(vec![]);
    }
    let mut out: Vec<String> = Vec::new();
    let mut stmt = store.conn().prepare(
        "SELECT DISTINCT value FROM (
            SELECT a.alias AS value FROM aliases a
              JOIN items i ON i.id = a.item_id WHERE i.is_active = 1
            UNION SELECT i2.title FROM items i2 WHERE i2.is_active = 1
            UNION SELECT t.name FROM tasks t WHERE t.is_active = 1
         )",
    )?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
    for row in rows {
        let value = row?;
        if normalize(&value).contains(&normalized) {
            out.push(value);
        }
        if out.len() as i64 >= limit {
            break;
        }
    }
    Ok(out)
}
