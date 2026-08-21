//! The safety contract around imported knowledge:
//! malformed packs cannot reach staging, a failed apply rolls back, backups
//! survive, undo restores, and the personal overlay is never overwritten.

mod support;
use pw_core::{apply, backup, pack, personal, repo, staging};

fn issues(json: &str, store: &pw_core::Store) -> Vec<pw_core::ValidationIssue> {
    match pack::parse_and_validate(json, Some(store)) {
        Err(pw_core::CoreError::Validation(issues)) => issues,
        Err(other) => panic!("expected validation issues, got {other}"),
        Ok(_) => panic!("expected the pack to be rejected"),
    }
}

fn codes(issues: &[pw_core::ValidationIssue]) -> Vec<&str> {
    issues.iter().map(|i| i.code.as_str()).collect()
}

#[test]
fn malformed_json_is_rejected_with_a_useful_location() {
    let store = support::empty_store();
    let found = issues("{ not json", &store);
    assert_eq!(codes(&found), vec!["malformed_json"]);
    assert!(
        found[0].path.starts_with("$"),
        "the error points at a place in the file"
    );
}

#[test]
fn an_unsupported_schema_version_is_rejected() {
    let store = support::store();
    let json = r#"{ "schemaVersion": "99.0",
        "batch": { "producer": "x", "generatedAt": "2026-08-21T12:00:00Z" } }"#;
    assert!(codes(&issues(json, &store)).contains(&"unsupported_schema_version"));
}

#[test]
fn an_unknown_field_is_rejected_rather_than_silently_dropped() {
    let store = support::store();
    let json = r#"{ "schemaVersion": "1.0",
        "batch": { "producer": "x", "generatedAt": "2026-08-21T12:00:00Z" },
        "itemz": [] }"#;
    // A typo must fail loudly: silently ignoring it would lose content.
    assert_eq!(codes(&issues(json, &store)), vec!["malformed_json"]);
}

#[test]
fn duplicate_external_identity_is_rejected_deterministically() {
    let store = support::store();
    let one = r#"{ "externalKey": "t:dup", "slug": "dup-a", "itemType": "concept", "title": "A" }"#;
    let two = r#"{ "externalKey": "t:dup", "slug": "dup-b", "itemType": "concept", "title": "B" }"#;
    let json = support::pack_json(&format!("{one},{two}"), "");
    assert!(codes(&issues(&json, &store)).contains(&"duplicate_external_key"));
}

#[test]
fn broken_references_are_rejected() {
    let store = support::store();
    let json = support::pack_json(
        r#"{ "externalKey": "t:x", "slug": "x", "itemType": "concept", "title": "X",
             "toolSlug": "no-such-tool", "tasks": ["no-such-task"], "categories": ["no-such-category"] }"#,
        "",
    );
    let found = issues(&json, &store);
    assert!(
        found
            .iter()
            .filter(|i| i.code == "broken_reference")
            .count()
            >= 3
    );
    assert!(found.iter().any(|i| i.path.ends_with(".toolSlug")));
}

#[test]
fn an_unsafe_url_is_rejected() {
    let store = support::store();
    for bad in [
        "javascript:alert(1)",
        "file:///etc/passwd",
        "data:text/html,<script>",
        "//evil.example",
    ] {
        let json = format!(
            r#"{{ "schemaVersion": "1.0",
                  "batch": {{ "producer": "x", "generatedAt": "2026-08-21T12:00:00Z" }},
                  "tools": [{{ "slug": "t", "name": "T", "officialHomeUrl": "{bad}" }}] }}"#
        );
        assert!(
            codes(&issues(&json, &store)).contains(&"invalid_url"),
            "expected {bad:?} to be rejected"
        );
    }
}

#[test]
fn markup_in_a_display_field_is_refused() {
    let store = support::store();
    let json = support::pack_json(
        r#"{ "externalKey": "t:x", "slug": "x", "itemType": "concept", "title": "X",
             "fullDescription": "hello <script>steal()</script>" }"#,
        "",
    );
    assert!(codes(&issues(&json, &store)).contains(&"unsafe_content"));
}

#[test]
fn official_status_requires_official_provenance() {
    let store = support::store();
    let json = support::pack_json(
        r#"{ "externalKey": "t:x", "slug": "x", "itemType": "cli_command", "title": "X",
             "verificationStatus": "official" }"#,
        "",
    );
    assert!(codes(&issues(&json, &store)).contains(&"missing_provenance"));

    // With a real official source it passes.
    let ok = support::pack_json(
        r#"{ "externalKey": "t:y", "slug": "y", "itemType": "cli_command", "title": "Y",
             "verificationStatus": "official",
             "sources": [{ "sourceKey": "docs", "verifiedAt": "2026-08-21T12:00:00Z" }] }"#,
        r#""sources": [{ "key": "docs", "name": "Docs", "sourceType": "official_docs",
                        "url": "https://example.com/docs", "isOfficial": true }],"#,
    );
    assert!(pack::parse_and_validate(&ok, Some(&store)).is_ok());
}

#[test]
fn a_category_cycle_is_rejected() {
    let store = support::store();
    let json = r#"{ "schemaVersion": "1.0",
        "batch": { "producer": "x", "generatedAt": "2026-08-21T12:00:00Z" },
        "categories": [
          { "slug": "a", "name": "A", "parentSlug": "b" },
          { "slug": "b", "name": "B", "parentSlug": "a" }
        ] }"#;
    assert!(codes(&issues(json, &store)).contains(&"category_cycle"));
}

#[test]
fn a_rejected_pack_leaves_the_trusted_database_untouched() {
    let store = support::store();
    let before = repo::table_counts(&store).unwrap();
    let _ = pack::parse_and_validate("{ broken", Some(&store));
    let json = support::pack_json(
        r#"{ "externalKey": "t:x", "slug": "x", "itemType": "concept", "title": "X",
             "toolSlug": "nope" }"#,
        "",
    );
    let _ = pack::parse_and_validate(&json, Some(&store));
    assert_eq!(repo::table_counts(&store).unwrap(), before);
    assert_eq!(store.integrity_check().unwrap(), "ok");
}

#[test]
fn importing_stages_a_diff_and_changes_nothing_until_approval() {
    let mut store = support::store();
    let items_before = store.count("items").unwrap();

    let json = support::pack_json(
        r#"{ "externalKey": "claude-code:staged-only", "slug": "staged-only",
             "toolSlug": "claude-code", "itemType": "concept", "title": "عنصر مرحّلي",
             "shortDescription": "لم يُعتمد بعد.", "verificationStatus": "demo" }"#,
        "",
    );
    let parsed = pack::parse_and_validate(&json, Some(&store)).unwrap();
    let batch_id = staging::stage_pack(&store, &parsed, "test-agent").unwrap();

    // Staged, visible for review, but nothing was written to the knowledge tables.
    let changes = staging::list_changes(&store, Some(&batch_id), None).unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].change_type, "add");
    assert_eq!(changes[0].status, "pending");
    assert_eq!(store.count("items").unwrap(), items_before);
    assert!(repo::item_detail(&store, "staged-only").is_err());

    // Applying without approving is refused: staging is the safe default.
    let refused = apply::apply_batch(&mut store, &batch_id, "test");
    assert!(matches!(refused, Err(pw_core::CoreError::Refused(_))));
    assert_eq!(store.count("items").unwrap(), items_before);

    // Approve, then apply.
    staging::set_review_status(&store, &batch_id, None, "approved").unwrap();
    let outcome = apply::apply_batch(&mut store, &batch_id, "test").unwrap();
    assert_eq!(outcome.applied, 1);
    assert!(
        outcome.backup_id.is_some(),
        "a backup is taken before applying"
    );
    assert_eq!(store.count("items").unwrap(), items_before + 1);
    assert!(repo::item_detail(&store, "staged-only").is_ok());
}

#[test]
fn an_update_produces_a_field_level_diff_and_no_row_for_unchanged_content() {
    let mut store = support::store();
    let first = support::pack_json(
        r#"{ "externalKey": "claude-code:diffable", "slug": "diffable", "toolSlug": "claude-code",
             "itemType": "concept", "title": "قبل", "shortDescription": "الوصف الأول.",
             "verificationStatus": "demo" }"#,
        "",
    );
    support::install(&mut store, &first);

    // Re-importing the identical content is not a change.
    let parsed = pack::parse_and_validate(&first, Some(&store)).unwrap();
    let same_batch = staging::stage_pack(&store, &parsed, "test").unwrap();
    assert!(staging::list_changes(&store, Some(&same_batch), None)
        .unwrap()
        .is_empty());

    // A real edit produces an update with a field diff.
    let second = first.replace("\"title\": \"قبل\"", "\"title\": \"بعد\"");
    let parsed = pack::parse_and_validate(&second, Some(&store)).unwrap();
    let batch = staging::stage_pack(&store, &parsed, "test").unwrap();
    let changes = staging::list_changes(&store, Some(&batch), None).unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].change_type, "update");
    let title_diff = changes[0]
        .field_diff
        .iter()
        .find(|f| f.field == "title")
        .expect("title diff");
    assert_eq!(title_diff.before.as_deref(), Some("قبل"));
    assert_eq!(title_diff.after.as_deref(), Some("بعد"));
}

#[test]
fn a_failed_apply_rolls_back_and_keeps_the_backup() {
    let mut store = support::store();
    let before_counts = repo::table_counts(&store).unwrap();

    // Stage two changes, then corrupt the second one's payload so apply fails
    // partway through — exactly the case the transaction exists for.
    let json = support::pack_json(
        r#"{ "externalKey": "claude-code:ok-item", "slug": "ok-item", "toolSlug": "claude-code",
             "itemType": "concept", "title": "سليم", "verificationStatus": "demo" },
           { "externalKey": "claude-code:doomed-item", "slug": "doomed-item", "toolSlug": "claude-code",
             "itemType": "concept", "title": "معطوب", "verificationStatus": "demo" }"#,
        "",
    );
    let parsed = pack::parse_and_validate(&json, Some(&store)).unwrap();
    let batch_id = staging::stage_pack(&store, &parsed, "test").unwrap();
    store
        .conn()
        .execute(
            "UPDATE staged_changes SET after_json = json_set(after_json, '$.toolSlug', 'ghost-tool')
             WHERE entity_key = 'claude-code:doomed-item'",
            [],
        )
        .unwrap();
    staging::set_review_status(&store, &batch_id, None, "approved").unwrap();

    let backups_before = backup::list(&store).unwrap().len();
    let result = apply::apply_batch(&mut store, &batch_id, "test");
    assert!(result.is_err(), "the apply must fail");

    // Nothing from the batch landed — not even the change that would have worked.
    assert_eq!(
        repo::table_counts(&store).unwrap().get("items"),
        before_counts.get("items")
    );
    assert!(repo::item_detail(&store, "ok-item").is_err());
    assert_eq!(store.integrity_check().unwrap(), "ok");
    assert_eq!(store.foreign_key_check().unwrap(), 0);

    // The pre-apply backup is kept, and the failure is recorded.
    assert_eq!(backup::list(&store).unwrap().len(), backups_before + 1);
    let batch = staging::get_batch(&store, &batch_id).unwrap();
    assert_eq!(batch.status, "failed");
}

#[test]
fn undo_reverses_an_applied_batch_and_rebuilds_the_index() {
    let mut store = support::store();
    let json = support::pack_json(
        r#"{ "externalKey": "claude-code:undo-me", "slug": "undo-me", "toolSlug": "claude-code",
             "itemType": "concept", "title": "قابل للتراجع",
             "shortDescription": "أُضيف بواسطة دفعة.", "verificationStatus": "demo",
             "aliases": ["تراجع تجريبي"] }"#,
        "",
    );
    let batch_id = support::install(&mut store, &json);

    assert!(repo::item_detail(&store, "undo-me").is_ok());
    let hits = pw_core::search::search(&store, "تراجع تجريبي", &Default::default(), 10, 0).unwrap();
    assert!(hits.hits.iter().any(|h| h.item.slug == "undo-me"));

    apply::revert_batch(&mut store, &batch_id, "test").unwrap();

    // Added rows are retired, never hard-deleted, and drop out of browsing.
    let detail = repo::item_detail(&store, "undo-me").unwrap();
    assert_eq!(detail.summary.lifecycle_status, "removed");
    let listed =
        repo::items_for_tool(&store, "claude-code", None, repo::ItemSort::Recent, 500).unwrap();
    assert!(
        !listed.iter().any(|i| i.slug == "undo-me"),
        "a retired item leaves browsing"
    );
    assert_eq!(
        staging::get_batch(&store, &batch_id).unwrap().status,
        "reverted"
    );
    assert_eq!(store.integrity_check().unwrap(), "ok");
}

#[test]
fn personal_notes_favorites_and_pins_survive_an_official_refresh() {
    let mut store = support::store();
    let first = support::pack_json(
        r#"{ "externalKey": "claude-code:overlay-item", "slug": "overlay-item",
             "toolSlug": "claude-code", "itemType": "cli_command", "title": "قبل التحديث",
             "shortDescription": "النسخة الأولى.", "syntax": "tool --old",
             "verificationStatus": "demo" }"#,
        "",
    );
    support::install(&mut store, &first);
    let item = repo::item_detail(&store, "overlay-item").unwrap();

    // The user makes it theirs.
    personal::set_favorite(&store, &item.summary.id, true).unwrap();
    personal::set_pin(&store, &item.summary.id, true).unwrap();
    personal::save_note(
        &store,
        None,
        Some(&item.summary.id),
        "ملاحظتي",
        "أستخدمه هكذا",
    )
    .unwrap();

    // The vendor content is refreshed on top of it.
    let second = first
        .replace("قبل التحديث", "بعد التحديث")
        .replace("tool --old", "tool --new")
        .replace("النسخة الأولى.", "النسخة الثانية.");
    support::install(&mut store, &second);

    let after = repo::item_detail(&store, "overlay-item").unwrap();
    assert_eq!(
        after.summary.title, "بعد التحديث",
        "official content did update"
    );
    assert_eq!(after.summary.syntax.as_deref(), Some("tool --new"));
    assert!(after.summary.is_favorite, "the favorite survived");
    assert!(after.summary.is_pinned, "the pin survived");
    assert_eq!(after.notes.len(), 1, "the personal note survived");
    assert_eq!(after.notes[0].body, "أستخدمه هكذا");
}

#[test]
fn a_removal_candidate_retires_an_item_instead_of_deleting_it() {
    let mut store = support::store();
    let json = support::pack_json(
        r#"{ "externalKey": "claude-code:retire-me", "slug": "retire-me", "toolSlug": "claude-code",
             "itemType": "concept", "title": "سيُسحب", "verificationStatus": "demo" }"#,
        "",
    );
    support::install(&mut store, &json);
    let id = repo::item_detail(&store, "retire-me").unwrap().summary.id;

    let removal = r#"{
      "schemaVersion": "1.0",
      "batch": { "producer": "test-agent", "generatedAt": "2026-08-21T12:00:00Z" },
      "removals": [{ "entityType": "item", "externalKey": "claude-code:retire-me",
                     "reason": "لم يعد مذكورًا في المصدر" }]
    }"#;
    support::install(&mut store, removal);

    let rows: i64 = store
        .conn()
        .query_row("SELECT COUNT(*) FROM items WHERE id = ?1", [&id], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(rows, 1, "history is preserved");
    let detail = repo::item_detail(&store, "retire-me").unwrap();
    assert_eq!(detail.summary.lifecycle_status, "removed");
}

#[test]
fn a_backup_can_be_restored_and_the_database_stays_valid() {
    let dir = support::temp_db();
    let mut store = pw_core::Store::open(dir.path().join("wiki.db")).unwrap();
    pw_core::seed::ensure_seeded(&mut store).unwrap();
    let snapshot = backup::create(&store, "test", None).unwrap();
    let items_at_snapshot = store.count("items").unwrap();

    support::install(
        &mut store,
        &support::pack_json(
            r#"{ "externalKey": "claude-code:after-snapshot", "slug": "after-snapshot",
                 "toolSlug": "claude-code", "itemType": "concept", "title": "بعد النسخة",
                 "verificationStatus": "demo" }"#,
            "",
        ),
    );
    assert_eq!(store.count("items").unwrap(), items_at_snapshot + 1);

    backup::restore(&mut store, &snapshot.id).unwrap();

    assert_eq!(store.count("items").unwrap(), items_at_snapshot);
    assert_eq!(store.integrity_check().unwrap(), "ok");
    assert_eq!(
        store.schema_version().unwrap(),
        pw_core::latest_schema_version()
    );
}
