//! Development-only fixture recorder (never part of the shipped binary).
//!
//! Runs the real `pw-core` read model against a freshly seeded database and
//! records the responses the UI asks for, keyed by command + arguments. The
//! browser test harness replays those recordings so Playwright can drive the
//! real Svelte UI without a Tauri runtime — and, crucially, without a second
//! implementation of any query.
//!
//!   cargo run -p pw-cli --example dev_fixtures -- tests/e2e/fixtures/api.json

use pw_core::models::SearchFilters;
use pw_core::{diagnostics, personal, repo, search, staging, Store};
use serde_json::{json, Map, Value};

fn key(cmd: &str, args: Value) -> String {
    // Canonical key: serde_json::Map is a BTreeMap here, so ordering is stable.
    format!("{cmd}|{}", serde_json::to_string(&args).unwrap())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "tests/e2e/fixtures/api.json".into());

    let dir = std::env::temp_dir().join(format!("pw-fixtures-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut store = Store::open(dir.join("wiki.db"))?;
    pw_core::seed::ensure_seeded(&mut store)?;

    // A little personal state so the fixtures exercise favourites, notes,
    // links, recents and saved searches rather than only empty states.
    let favourite = repo::item_detail(&store, "claude-print-mode")?;
    personal::set_favorite(&store, &favourite.summary.id, true)?;
    personal::set_pin(&store, &favourite.summary.id, true)?;
    personal::record_open(&store, &favourite.summary.id)?;
    let doctor = repo::item_detail(&store, "claude-doctor")?;
    personal::record_open(&store, &doctor.summary.id)?;
    personal::save_note(
        &store,
        None,
        Some(&favourite.summary.id),
        "طريقتي في الاستخدام",
        "أستخدم هذا الأمر داخل سكربتات النشر.",
    )?;
    personal::save_link(
        &store,
        None,
        "توثيق Claude Code",
        "https://code.claude.com/docs",
        Some("المرجع الرسمي"),
        None,
        None,
        true,
    )?;
    personal::save_search(&store, "أوامر الوكلاء", "وكيل", &SearchFilters::default())?;
    personal::record_search(
        &store,
        "تشغيل وكيل في الخلفية",
        &SearchFilters::default(),
        12,
    )?;

    let mut fixtures: Map<String, Value> = Map::new();
    let mut put = |cmd: &str, args: Value, value: Value| {
        fixtures.insert(key(cmd, args), value);
    };

    put(
        "home_summary",
        json!({}),
        serde_json::to_value(repo::home_summary(&store)?)?,
    );
    put(
        "list_tools",
        json!({}),
        serde_json::to_value(repo::tool_cards(&store)?)?,
    );
    put(
        "list_tasks",
        json!({}),
        serde_json::to_value(repo::task_cards(&store)?)?,
    );
    put(
        "category_tree",
        json!({}),
        serde_json::to_value(repo::category_tree(&store, None)?)?,
    );
    put(
        "list_tags",
        json!({}),
        serde_json::to_value(repo::list_tags(&store)?)?,
    );
    put(
        "list_item_types",
        json!({}),
        serde_json::to_value(repo::list_item_types(&store)?)?,
    );
    put(
        "list_favorites",
        json!({}),
        serde_json::to_value(repo::favorites(&store)?)?,
    );
    put(
        "list_pinned",
        json!({}),
        serde_json::to_value(repo::pinned(&store)?)?,
    );
    put(
        "list_notes",
        json!({}),
        serde_json::to_value(personal::list_notes(&store)?)?,
    );
    put(
        "list_links",
        json!({}),
        serde_json::to_value(personal::list_links(&store)?)?,
    );
    put(
        "list_sources",
        json!({}),
        serde_json::to_value(repo::list_sources(&store)?)?,
    );
    put(
        "list_batches",
        json!({}),
        serde_json::to_value(staging::list_batches(&store, 50)?)?,
    );
    put(
        "list_backups",
        json!({}),
        serde_json::to_value(pw_core::backup::list(&store)?)?,
    );
    put(
        "list_audit",
        json!({ "limit": 12 }),
        serde_json::to_value(pw_core::audit::list(&store, 12)?)?,
    );
    put(
        "list_audit",
        json!({ "limit": 40 }),
        serde_json::to_value(pw_core::audit::list(&store, 40)?)?,
    );
    put(
        "diagnostics",
        json!({}),
        serde_json::to_value(diagnostics::collect(&store)?)?,
    );
    put(
        "get_settings",
        json!({}),
        Value::Object(personal::all_settings(&store)?),
    );
    put(
        "search_history",
        json!({}),
        serde_json::to_value(personal::search_history(&store, 12)?)?,
    );
    put(
        "list_saved_searches",
        json!({}),
        serde_json::to_value(personal::list_saved_searches(&store)?)?,
    );
    put(
        "list_changes",
        json!({ "batchId": null, "status": "pending" }),
        serde_json::to_value(staging::list_changes(&store, None, Some("pending"))?)?,
    );

    for batch in staging::list_batches(&store, 50)? {
        put(
            "list_changes",
            json!({ "batchId": batch.id, "status": null }),
            serde_json::to_value(staging::list_changes(&store, Some(&batch.id), None)?)?,
        );
    }

    for tool in repo::list_tools(&store, false)? {
        put(
            "tool_page",
            json!({ "slug": tool.slug }),
            serde_json::to_value(repo::tool_page(&store, &tool.slug)?)?,
        );
        for sort in ["recent", "alpha", "popular"] {
            put(
                "items_for_tool",
                json!({ "slug": tool.slug, "section": null, "sort": sort }),
                serde_json::to_value(repo::items_for_tool(
                    &store,
                    &tool.slug,
                    None,
                    repo::ItemSort::parse(Some(sort)),
                    500,
                )?)?,
            );
        }
        fn flatten(nodes: &[pw_core::models::CategoryNode], out: &mut Vec<String>) {
            for node in nodes {
                out.push(node.category.slug.clone());
                flatten(&node.children, out);
            }
        }
        let mut sections = Vec::new();
        flatten(
            &repo::tool_page(&store, &tool.slug)?.sections,
            &mut sections,
        );
        for section in sections {
            put(
                "items_for_tool",
                json!({ "slug": tool.slug, "section": section, "sort": "recent" }),
                serde_json::to_value(repo::items_for_tool(
                    &store,
                    &tool.slug,
                    Some(&section),
                    repo::ItemSort::Recent,
                    500,
                )?)?,
            );
        }
    }

    for task in repo::list_tasks(&store, false)? {
        put(
            "task_detail",
            json!({ "slug": task.slug }),
            serde_json::to_value(&task)?,
        );
        put(
            "items_for_task",
            json!({ "slug": task.slug, "sort": "recent" }),
            serde_json::to_value(repo::items_for_task(
                &store,
                &task.slug,
                repo::ItemSort::Recent,
                500,
            )?)?,
        );
    }

    fn all_categories(nodes: &[pw_core::models::CategoryNode], out: &mut Vec<String>) {
        for node in nodes {
            out.push(node.category.slug.clone());
            all_categories(&node.children, out);
        }
    }
    let mut categories = Vec::new();
    all_categories(&repo::category_tree(&store, None)?, &mut categories);
    for slug in categories {
        put(
            "category_detail",
            json!({ "slug": slug }),
            serde_json::to_value(repo::category_by_slug(&store, &slug)?)?,
        );
        put(
            "category_path",
            json!({ "slug": slug }),
            serde_json::to_value(repo::category_path(&store, &slug)?)?,
        );
        put(
            "items_for_category",
            json!({ "slug": slug, "sort": "recent" }),
            serde_json::to_value(repo::items_for_category(
                &store,
                &slug,
                repo::ItemSort::Recent,
                500,
            )?)?,
        );
    }

    let mut item_slugs = Vec::new();
    {
        let mut stmt = store
            .conn()
            .prepare("SELECT slug FROM items WHERE is_active = 1")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        for row in rows {
            item_slugs.push(row?);
        }
    }
    for slug in &item_slugs {
        put(
            "item_detail",
            json!({ "slug": slug }),
            serde_json::to_value(repo::item_detail(&store, slug)?)?,
        );
    }

    // The queries the E2E suite types.
    for query in [
        "وكيل",
        "تشغيل وكيل في الخلفية",
        "claude",
        "claude doctor",
        "stash",
        "زقموطةزقموطةزقموطة",
    ] {
        for limit in [20, 25, 40] {
            let response = search::search(&store, query, &SearchFilters::default(), limit, 0)?;
            put(
                "search",
                json!({ "query": query, "filters": SearchFilters::default(), "limit": limit, "offset": 0 }),
                serde_json::to_value(&response)?,
            );
        }
        put(
            "search_suggestions",
            json!({ "query": query }),
            serde_json::to_value(search::suggestions(&store, query, 6)?)?,
        );
    }
    // One filtered query, to exercise the filter bar.
    let mut filters = SearchFilters::default();
    filters.tool_slugs = vec!["git-github".into()];
    for limit in [20, 25, 40] {
        put(
            "search",
            json!({ "query": "stash", "filters": filters, "limit": limit, "offset": 0 }),
            serde_json::to_value(search::search(&store, "stash", &filters, limit, 0)?)?,
        );
    }

    let payload = json!({
        "generatedBy": "cargo run -p pw-cli --example dev_fixtures",
        "note": "Recorded from the real pw-core read model. Regenerate after changing queries or seed content.",
        "itemSlugs": item_slugs,
        "responses": Value::Object(fixtures),
    });
    if let Some(parent) = std::path::Path::new(&out).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&out, serde_json::to_string(&payload)?)?;
    let count = payload["responses"].as_object().unwrap().len();
    println!("wrote {count} recorded responses to {out}");
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}
