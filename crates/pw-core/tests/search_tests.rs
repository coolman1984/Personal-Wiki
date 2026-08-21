//! Search: normalization, alias discovery, ranking, filters, index maintenance.

mod support;
use pw_core::models::SearchFilters;
use pw_core::{repo, search};

fn titles(response: &pw_core::models::SearchResponse) -> Vec<&str> {
    response.hits.iter().map(|h| h.item.title.as_str()).collect()
}

#[test]
fn arabic_normalization_folds_orthographic_variants() {
    // hamza forms, ta marbuta, alef maqsura, tatweel, diacritics, arabic-indic digits
    assert_eq!(search::normalize("أحمد"), search::normalize("احمد"));
    assert_eq!(search::normalize("إعداد"), search::normalize("اعداد"));
    assert_eq!(search::normalize("مكتبة"), search::normalize("مكتبه"));
    assert_eq!(search::normalize("علي"), search::normalize("على"));
    assert_eq!(search::normalize("جـــلسة"), search::normalize("جلسه"));
    assert_eq!(search::normalize("مُحَدَّث"), search::normalize("محدث"));
    assert_eq!(search::normalize("٢٠٢٦"), "2026");
    assert_eq!(search::normalize("  A   B  "), "a b");
}

#[test]
fn an_alias_that_appears_nowhere_else_still_finds_its_item() {
    let mut store = support::store();
    let json = support::pack_json(
        r#"{ "externalKey": "claude-code:alias-target", "slug": "alias-target",
             "toolSlug": "claude-code", "itemType": "cli_command", "title": "zzqx --flag",
             "shortDescription": "أمر باسم غير مألوف.", "syntax": "zzqx --flag",
             "verificationStatus": "demo",
             "aliases": ["فرمتة السحابة الزرقاء"] }"#,
        "",
    );
    support::install(&mut store, &json);

    // The phrase is in no title, description or syntax — only in the alias.
    let response = search::search(&store, "فرمتة السحابة الزرقاء", &SearchFilters::default(), 10, 0).unwrap();
    assert!(!response.hits.is_empty(), "the alias must be searchable");
    assert_eq!(response.hits[0].item.slug, "alias-target");
    assert_eq!(response.hits[0].matched_alias.as_deref(), Some("فرمتة السحابة الزرقاء"));
}

#[test]
fn an_exact_title_match_outranks_a_body_mention() {
    let store = support::store();
    let response = search::search(&store, "claude doctor", &SearchFilters::default(), 10, 0).unwrap();
    assert!(!response.hits.is_empty());
    assert_eq!(response.hits[0].item.title, "claude doctor");
}

#[test]
fn a_deprecated_item_ranks_below_an_active_equivalent() {
    let mut store = support::store();
    let json = support::pack_json(
        r#"{ "externalKey": "claude-code:rank-active", "slug": "rank-active", "toolSlug": "claude-code",
             "itemType": "cli_command", "title": "widgetize", "shortDescription": "النسخة الحالية.",
             "verificationStatus": "demo", "lifecycleStatus": "active" },
           { "externalKey": "claude-code:rank-old", "slug": "rank-old", "toolSlug": "claude-code",
             "itemType": "cli_command", "title": "widgetize", "shortDescription": "النسخة القديمة.",
             "verificationStatus": "demo", "lifecycleStatus": "deprecated" }"#,
        "",
    );
    support::install(&mut store, &json);

    let response = search::search(&store, "widgetize", &SearchFilters::default(), 10, 0).unwrap();
    let slugs: Vec<_> = response.hits.iter().map(|h| h.item.slug.as_str()).collect();
    let active = slugs.iter().position(|s| *s == "rank-active").expect("active hit");
    let old = slugs.iter().position(|s| *s == "rank-old").expect("deprecated hit");
    assert!(active < old, "active must outrank deprecated: {slugs:?}");
}

#[test]
fn filters_narrow_the_result_set() {
    let store = support::store();
    let all = search::search(&store, "install", &SearchFilters::default(), 50, 0).unwrap();
    assert!(all.total > 0);

    let mut filters = SearchFilters::default();
    filters.tool_slugs = vec!["codex-cli".into()];
    let scoped = search::search(&store, "install", &filters, 50, 0).unwrap();
    assert!(scoped.total > 0);
    assert!(scoped.total < all.total);
    assert!(scoped.hits.iter().all(|h| h.item.tool_slug.as_deref() == Some("codex-cli")));

    let mut type_filter = SearchFilters::default();
    type_filter.item_types = vec!["slash_command".into()];
    let slashes = search::search(&store, "claude", &type_filter, 50, 0).unwrap();
    assert!(slashes.hits.iter().all(|h| h.item.item_type == "slash_command"));
}

#[test]
fn a_favorites_filter_only_returns_favorites() {
    let store = support::store();
    let item = repo::item_detail(&store, "claude-doctor").unwrap();
    pw_core::personal::set_favorite(&store, &item.summary.id, true).unwrap();

    let mut filters = SearchFilters::default();
    filters.favorites_only = true;
    let response = search::search(&store, "claude", &filters, 50, 0).unwrap();
    assert!(!response.hits.is_empty());
    assert!(response.hits.iter().all(|h| h.item.is_favorite));
}

#[test]
fn searching_an_arabic_task_phrase_finds_commands_from_several_tools() {
    let store = support::store();
    let response = search::search(&store, "تثبيت", &SearchFilters::default(), 30, 0).unwrap();
    let tools: std::collections::HashSet<_> =
        response.hits.iter().filter_map(|h| h.item.tool_slug.clone()).collect();
    assert!(tools.len() >= 2, "a task phrase should cross tools, got {tools:?}");
    assert!(!titles(&response).is_empty());
}

#[test]
fn the_index_is_maintained_on_write_and_can_be_rebuilt_after_damage() {
    let mut store = support::store();
    let json = support::pack_json(
        r#"{ "externalKey": "claude-code:indexed", "slug": "indexed", "toolSlug": "claude-code",
             "itemType": "concept", "title": "مفهوم مفهرس", "shortDescription": "كلمة نادرة: زقموط.",
             "verificationStatus": "demo" }"#,
        "",
    );
    support::install(&mut store, &json);
    assert!(!search::search(&store, "زقموط", &SearchFilters::default(), 5, 0).unwrap().hits.is_empty());

    // Simulate a damaged index.
    store.conn().execute("DELETE FROM items_fts", []).unwrap();
    let (status, _) = search::health(store.conn());
    assert_eq!(status, "stale");
    assert!(search::search(&store, "زقموط", &SearchFilters::default(), 5, 0).unwrap().hits.is_empty());

    let rebuilt = search::rebuild(store.conn()).unwrap();
    assert!(rebuilt > 0);
    let (status, rows) = search::health(store.conn());
    assert_eq!(status, "ok");
    assert_eq!(rows, store.count("items").unwrap());
    assert!(!search::search(&store, "زقموط", &SearchFilters::default(), 5, 0).unwrap().hits.is_empty());
}

#[test]
fn a_query_full_of_punctuation_cannot_break_the_match_expression() {
    let store = support::store();
    for hostile in ["\"", "AND OR NOT", "*", "a\" OR items_fts MATCH \"b", "()", "^^^", "--"] {
        let response = search::search(&store, hostile, &SearchFilters::default(), 5, 0);
        assert!(response.is_ok(), "query {hostile:?} must not error");
    }
}

#[test]
fn suggestions_complete_aliases_and_task_names() {
    let store = support::store();
    let suggestions = search::suggestions(&store, "تشغيل", 6).unwrap();
    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().all(|s| !s.is_empty()));
}

#[test]
fn an_empty_query_returns_nothing_rather_than_everything() {
    let store = support::store();
    let response = search::search(&store, "   ", &SearchFilters::default(), 10, 0).unwrap();
    assert_eq!(response.total, 0);
    assert!(response.hits.is_empty());
}

#[test]
fn pagination_walks_the_result_set_without_repeating() {
    let store = support::store();
    let first = search::search(&store, "claude", &SearchFilters::default(), 5, 0).unwrap();
    let second = search::search(&store, "claude", &SearchFilters::default(), 5, 5).unwrap();
    assert_eq!(first.hits.len(), 5);
    assert_eq!(first.total, second.total);
    let a: std::collections::HashSet<_> = first.hits.iter().map(|h| h.item.id.clone()).collect();
    let b: std::collections::HashSet<_> = second.hits.iter().map(|h| h.item.id.clone()).collect();
    assert!(a.is_disjoint(&b), "pages must not overlap");
}
