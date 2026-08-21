//! Schema/migration integrity, the personal layer's CRUD, and the security rules
//! that keep imported content inert.

mod support;
use pw_core::{db, personal, repo, url, Store};

#[test]
fn a_clean_install_reaches_the_latest_schema_with_foreign_keys_on() {
    let store = Store::open_in_memory().unwrap();
    assert_eq!(store.schema_version().unwrap(), db::latest_schema_version());
    assert_eq!(store.integrity_check().unwrap(), "ok");
    let fk: i64 = store.conn().query_row("PRAGMA foreign_keys", [], |r| r.get(0)).unwrap();
    assert_eq!(fk, 1, "foreign keys must be enforced");
}

#[test]
fn migrating_is_idempotent_and_upgrades_an_existing_database() {
    let dir = support::temp_db();
    let path = dir.path().join("wiki.db");
    {
        let mut store = Store::open(&path).unwrap();
        pw_core::seed::ensure_seeded(&mut store).unwrap();
        store.migrate().unwrap();
        store.migrate().unwrap();
    }
    // Reopening an existing database migrates forward and keeps the content.
    let mut store = Store::open(&path).unwrap();
    store.migrate().unwrap();
    assert_eq!(store.schema_version().unwrap(), db::latest_schema_version());
    assert!(store.count("items").unwrap() > 0);
    assert_eq!(store.integrity_check().unwrap(), "ok");

    let applied: i64 = store
        .conn()
        .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
        .unwrap();
    assert_eq!(applied, db::latest_schema_version(), "each migration is recorded once");
}

#[test]
fn every_migration_version_is_unique_and_ordered() {
    let mut seen = Vec::new();
    for (version, name, sql) in db::MIGRATIONS {
        assert!(!seen.contains(version), "duplicate migration version {version}");
        assert!(seen.last().map(|last| last < version).unwrap_or(true), "{name} is out of order");
        assert!(!sql.trim().is_empty(), "{name} is empty");
        seen.push(*version);
    }
}

#[test]
fn deleting_an_item_cascades_to_its_children_but_never_to_a_tool() {
    let store = support::store();
    let item = repo::item_detail(&store, "claude-print-mode").unwrap();
    let id = item.summary.id.clone();
    assert!(!item.parameters.is_empty());

    store.conn().execute("DELETE FROM items WHERE id = ?1", [&id]).unwrap();

    for table in ["aliases", "examples", "parameters", "item_tasks", "item_categories", "item_tags"] {
        let left: i64 = store
            .conn()
            .query_row(&format!("SELECT COUNT(*) FROM {table} WHERE item_id = ?1"), [&id], |r| r.get(0))
            .unwrap();
        assert_eq!(left, 0, "{table} should cascade");
    }
    assert!(repo::tool_by_slug(&store, "claude-code").is_ok(), "the tool is untouched");
    assert_eq!(store.foreign_key_check().unwrap(), 0);
}

#[test]
fn only_absolute_http_urls_are_accepted() {
    for good in ["https://example.com/docs", "http://localhost:3000/x", "https://a.b.c/d?e=f#g"] {
        assert!(url::validate_web_url(good).is_ok(), "{good} should be accepted");
    }
    for bad in [
        "javascript:alert(1)", "JavaScript:alert(1)", "data:text/html,<script>",
        "file:///etc/passwd", "vbscript:msgbox", "//evil.example", "https://", "", "   ",
        "https://has space/x", "https://nohost",
    ] {
        assert!(url::validate_web_url(bad).is_err(), "{bad:?} should be refused");
    }
}

#[test]
fn a_saved_link_with_an_unsafe_url_is_refused() {
    let store = support::store();
    let refused = personal::save_link(&store, None, "خبيث", "javascript:alert(1)", None, None, None, false);
    assert!(refused.is_err());
    assert!(personal::list_links(&store).unwrap().is_empty());

    let ok = personal::save_link(&store, None, "توثيق", "https://example.com/docs", None, None, None, false);
    assert!(ok.is_ok());
    assert_eq!(personal::list_links(&store).unwrap().len(), 1);
}

#[test]
fn a_locally_created_item_cannot_claim_official_status() {
    let store = support::store();
    let input: pw_core::edit::ItemInput = serde_json::from_str(
        r#"{ "title": "ادعاء رسمي", "itemType": "cli_command", "verificationStatus": "official" }"#,
    )
    .unwrap();
    let refused = pw_core::edit::save_item(&store, input);
    assert!(matches!(refused, Err(pw_core::CoreError::Refused(_))));
}

#[test]
fn personal_crud_round_trips() {
    let store = support::store();
    let item = repo::item_detail(&store, "claude-doctor").unwrap();

    // favorites + pins
    personal::set_favorite(&store, &item.summary.id, true).unwrap();
    personal::set_pin(&store, &item.summary.id, true).unwrap();
    assert_eq!(repo::favorites(&store).unwrap().len(), 1);
    assert_eq!(repo::pinned(&store).unwrap().len(), 1);
    personal::set_favorite(&store, &item.summary.id, false).unwrap();
    assert!(repo::favorites(&store).unwrap().is_empty());

    // notes
    let note = personal::save_note(&store, None, Some(&item.summary.id), "عنوان", "نص").unwrap();
    let edited = personal::save_note(&store, Some(&note.id), Some(&item.summary.id), "عنوان", "نص محدّث").unwrap();
    assert_eq!(edited.body, "نص محدّث");
    assert_eq!(personal::list_notes(&store).unwrap().len(), 1);
    personal::delete_note(&store, &note.id).unwrap();
    assert!(personal::list_notes(&store).unwrap().is_empty());

    // links
    let link = personal::save_link(&store, None, "رابط", "https://example.com", None, None, None, true).unwrap();
    personal::delete_link(&store, &link.id).unwrap();
    assert!(personal::list_links(&store).unwrap().is_empty());

    // settings + saved searches + history
    personal::set_setting(&store, "startupScreen", &serde_json::json!("/tools")).unwrap();
    assert_eq!(
        personal::get_setting(&store, "startupScreen").unwrap(),
        Some(serde_json::json!("/tools"))
    );
    personal::save_search(&store, "أوامر الوكلاء", "وكيل", &Default::default()).unwrap();
    assert_eq!(personal::list_saved_searches(&store).unwrap().len(), 1);
    personal::record_search(&store, "وكيل", &Default::default(), 3).unwrap();
    assert_eq!(personal::search_history(&store, 10).unwrap().len(), 1);
    personal::clear_search_history(&store).unwrap();
    assert!(personal::search_history(&store, 10).unwrap().is_empty());
}

#[test]
fn a_category_may_not_become_its_own_ancestor() {
    let store = support::store();
    let tree = repo::category_tree(&store, None).unwrap();
    let parent = tree
        .iter()
        .find(|n| !n.children.is_empty())
        .expect("a category with children");
    let child = parent.children[0].category.clone();

    // Making the parent a child of its own descendant would create a cycle.
    let input: pw_core::edit::CategoryInput = serde_json::from_str(&format!(
        r#"{{ "id": "{}", "name": "{}", "parentId": "{}" }}"#,
        parent.category.id, parent.category.name, child.id
    ))
    .unwrap();
    assert!(pw_core::edit::save_category(&store, input).is_err());

    // A category cannot be its own parent either.
    let self_parent: pw_core::edit::CategoryInput = serde_json::from_str(&format!(
        r#"{{ "id": "{}", "name": "{}", "parentId": "{}" }}"#,
        child.id, child.name, child.id
    ))
    .unwrap();
    assert!(pw_core::edit::save_category(&store, self_parent).is_err());
}

#[test]
fn diagnostics_reports_a_healthy_database() {
    let store = support::store();
    let report = pw_core::diagnostics::collect(&store).unwrap();
    assert_eq!(report.integrity, "ok");
    assert_eq!(report.foreign_key_violations, 0);
    assert_eq!(report.fts_status, "ok");
    assert_eq!(report.schema_version, report.expected_schema_version);
    assert!(report.problems.is_empty(), "unexpected problems: {:?}", report.problems);
    assert!(report.counts.get("items").copied().unwrap_or(0) > 0);
}
