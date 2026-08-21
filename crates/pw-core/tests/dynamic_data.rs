//! Proof that the UI grows from data alone.
//!
//! Every assertion here goes through the same read model the Svelte frontend
//! calls, so "a new tool appears without a code change" is verified against the
//! exact queries that render the screens.

mod support;
use pw_core::repo;

const TOOL_PACK: &str = r#"{
  "schemaVersion": "1.0",
  "batch": { "producer": "test-agent", "generatedAt": "2026-08-21T12:00:00Z" },
  "tools": [{ "slug": "powershell", "name": "PowerShell", "iconKey": "terminal",
              "accentKey": "blue", "externalKey": "tool:powershell",
              "description": "صدفة أوامر ويندوز." }],
  "categories": [
    { "slug": "automation", "name": "الأتمتة", "externalKey": "category:automation" },
    { "slug": "scheduled-tasks", "name": "المهام المجدولة", "parentSlug": "automation",
      "externalKey": "category:scheduled-tasks" },
    { "slug": "daily-jobs", "name": "المهام اليومية", "parentSlug": "scheduled-tasks",
      "externalKey": "category:daily-jobs" }
  ],
  "items": [{
    "externalKey": "powershell:new-scheduled-task",
    "slug": "powershell-new-scheduled-task",
    "toolSlug": "powershell",
    "itemType": "cli_command",
    "title": "New-ScheduledTask",
    "shortDescription": "إنشاء مهمة مجدولة.",
    "verificationStatus": "demo",
    "categories": ["daily-jobs"],
    "tasks": ["automation"],
    "aliases": ["جدولة مهمة", "schedule a job"]
  }]
}"#;

#[test]
fn a_new_tool_appears_on_home_and_in_the_tool_browser() {
    let mut store = support::store();
    let before = repo::tool_cards(&store).unwrap().len();

    support::install(&mut store, TOOL_PACK);

    let cards = repo::tool_cards(&store).unwrap();
    assert_eq!(cards.len(), before + 1);
    let card = cards.iter().find(|c| c.tool.slug == "powershell").expect("new tool card");
    assert_eq!(card.item_count, 1, "the card's counter is a query, not a stored number");

    // The home summary is what the first screen actually renders.
    let home = repo::home_summary(&store).unwrap();
    assert!(home.tools.iter().any(|c| c.tool.slug == "powershell"));

    // And the tool page opens.
    let page = repo::tool_page(&store, "powershell").unwrap();
    assert_eq!(page.tool.name, "PowerShell");
    assert_eq!(page.stats.item_count, 1);
}

#[test]
fn nested_categories_render_at_arbitrary_depth() {
    let mut store = support::store();
    support::install(&mut store, TOOL_PACK);

    // Three levels deep, built from parent_id at runtime.
    let tree = repo::category_tree(&store, None).unwrap();
    let automation = tree.iter().find(|n| n.category.slug == "automation").expect("root");
    let scheduled = automation
        .children
        .iter()
        .find(|n| n.category.slug == "scheduled-tasks")
        .expect("second level");
    let daily = scheduled
        .children
        .iter()
        .find(|n| n.category.slug == "daily-jobs")
        .expect("third level");
    assert_eq!(daily.item_count, 1);
    // Counts roll up the subtree, which is what the rail and tabs display.
    assert_eq!(scheduled.item_count, 1);
    assert_eq!(automation.item_count, 1);

    // Breadcrumbs come from the same tree.
    let path = repo::category_path(&store, "daily-jobs").unwrap();
    let slugs: Vec<_> = path.iter().map(|c| c.slug.as_str()).collect();
    assert_eq!(slugs, vec!["automation", "scheduled-tasks", "daily-jobs"]);

    // The tool page's section tabs are pruned to branches that hold its items.
    let page = repo::tool_page(&store, "powershell").unwrap();
    assert!(page.sections.iter().any(|n| n.category.slug == "automation"));
}

#[test]
fn a_category_query_includes_the_whole_subtree() {
    let mut store = support::store();
    support::install(&mut store, TOOL_PACK);

    let items = repo::items_for_category(&store, "automation", repo::ItemSort::Recent, 100).unwrap();
    assert_eq!(items.len(), 1, "an ancestor category shows items stored on its descendants");
}

#[test]
fn one_item_can_belong_to_several_tasks_without_being_duplicated() {
    let mut store = support::store();
    let pack = r#"{
      "schemaVersion": "1.0",
      "batch": { "producer": "test-agent", "generatedAt": "2026-08-21T12:00:00Z" },
      "items": [{
        "externalKey": "claude-code:shared-item",
        "slug": "shared-item",
        "toolSlug": "claude-code",
        "itemType": "concept",
        "title": "عنصر مشترك",
        "shortDescription": "ينتمي إلى مهمتين.",
        "verificationStatus": "demo",
        "tasks": ["agents", "background-work"]
      }]
    }"#;
    support::install(&mut store, pack);

    let agents = repo::items_for_task(&store, "agents", repo::ItemSort::Recent, 100).unwrap();
    let background = repo::items_for_task(&store, "background-work", repo::ItemSort::Recent, 100).unwrap();
    assert!(agents.iter().any(|i| i.slug == "shared-item"));
    assert!(background.iter().any(|i| i.slug == "shared-item"));

    let stored: i64 = store
        .conn()
        .query_row("SELECT COUNT(*) FROM items WHERE slug = 'shared-item'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(stored, 1, "linked to two tasks, stored once");
}

#[test]
fn a_cross_tool_task_groups_items_from_several_tool_families() {
    let store = support::store();
    // The shipped content links several tools to the setup task.
    let items = repo::items_for_task(&store, "setup", repo::ItemSort::Recent, 200).unwrap();
    let tools: std::collections::HashSet<_> =
        items.iter().filter_map(|i| i.tool_slug.clone()).collect();
    assert!(
        tools.len() >= 4,
        "expected one task page to gather at least four tool families, got {tools:?}"
    );
}

#[test]
fn deactivating_a_record_hides_it_without_deleting_anything() {
    let store = support::store();
    let tools = repo::tool_cards(&store).unwrap();
    let target = tools.first().expect("a tool").tool.clone();

    pw_core::edit::set_active(&store, "tool", &target.id, false).unwrap();

    let visible = repo::tool_cards(&store).unwrap();
    assert!(!visible.iter().any(|c| c.tool.id == target.id), "hidden from browsing");

    let rows: i64 = store
        .conn()
        .query_row("SELECT COUNT(*) FROM tools WHERE id = ?1", [&target.id], |r| r.get(0))
        .unwrap();
    assert_eq!(rows, 1, "the record is retained, not deleted");

    // And it comes back.
    pw_core::edit::set_active(&store, "tool", &target.id, true).unwrap();
    assert!(repo::tool_cards(&store).unwrap().iter().any(|c| c.tool.id == target.id));
}

#[test]
fn item_detail_only_carries_the_sections_the_record_actually_has() {
    let mut store = support::store();
    let pack = r#"{
      "schemaVersion": "1.0",
      "batch": { "producer": "test-agent", "generatedAt": "2026-08-21T12:00:00Z" },
      "items": [{
        "externalKey": "claude-code:bare-link",
        "slug": "bare-link",
        "toolSlug": "claude-code",
        "itemType": "link",
        "title": "رابط بلا صيغة",
        "shortDescription": "عنصر بسيط.",
        "verificationStatus": "demo"
      }]
    }"#;
    support::install(&mut store, pack);

    let bare = repo::item_detail(&store, "bare-link").unwrap();
    assert!(bare.summary.syntax.is_none());
    assert!(bare.parameters.is_empty());
    assert!(bare.examples.is_empty());
    assert!(bare.related.is_empty());

    // A fully populated command carries every section.
    let full = repo::item_detail(&store, "claude-print-mode").unwrap();
    assert!(full.summary.syntax.is_some());
    assert!(!full.parameters.is_empty());
    assert!(!full.examples.is_empty());
    assert!(!full.sources.is_empty());
    assert!(!full.aliases.is_empty());
}
