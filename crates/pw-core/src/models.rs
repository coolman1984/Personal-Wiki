//! Domain types shared by the desktop app, the CLI, and the tests.
//! Serialized as camelCase because the Svelte frontend consumes them directly.

use serde::{Deserialize, Serialize};

macro_rules! camel {
    ($(#[$m:meta])* pub struct $name:ident { $($body:tt)* }) => {
        $(#[$m])*
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        #[serde(rename_all = "camelCase")]
        #[derive(Default)]
pub struct $name { $($body)* }
    };
}

camel! {
/// A tool family (Claude Code, Codex CLI, PowerShell…). Rows here become cards,
/// pages, and filters with no frontend change.
pub struct Tool {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub short_name: Option<String>,
    pub description: Option<String>,
    pub icon_key: String,
    pub accent_key: String,
    pub official_home_url: Option<String>,
    pub sort_order: i64,
    pub is_active: bool,
    pub external_key: Option<String>,
    pub managed_by: String,
    pub updated_at: String,
}}

camel! {
/// A tool card as rendered on Home: the record plus its computed counters.
pub struct ToolCard {
    #[serde(flatten)]
    pub tool: Tool,
    pub item_count: i64,
    pub example_count: i64,
    pub note_count: i64,
    pub source_count: i64,
    pub last_updated_at: Option<String>,
}}

camel! {
pub struct Task {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub icon_key: String,
    pub sort_order: i64,
    pub is_active: bool,
    pub external_key: Option<String>,
}}

camel! {
pub struct TaskCard {
    #[serde(flatten)]
    pub task: Task,
    pub item_count: i64,
    pub tool_count: i64,
    pub tool_icons: Vec<String>,
}}

camel! {
pub struct Category {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub icon_key: String,
    pub accent_key: String,
    pub parent_id: Option<String>,
    pub sort_order: i64,
    pub is_active: bool,
    pub external_key: Option<String>,
}}

camel! {
/// Category tree node. Built from `parent_id` at any depth — the UI never assumes
/// two levels.
pub struct CategoryNode {
    #[serde(flatten)]
    pub category: Category,
    pub item_count: i64,
    pub children: Vec<CategoryNode>,
}}

camel! {
pub struct Tag {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub accent_key: String,
}}

camel! {
pub struct Example {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub code: String,
    pub expected_output: Option<String>,
    pub sort_order: i64,
}}

camel! {
pub struct Parameter {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_required: bool,
    pub default_value: Option<String>,
    pub value_hint: Option<String>,
    pub sort_order: i64,
}}

camel! {
pub struct ItemSource {
    pub source_id: String,
    pub name: String,
    pub source_type: String,
    pub is_official: bool,
    pub url: Option<String>,
    pub source_url: Option<String>,
    pub source_anchor: Option<String>,
    pub source_version: Option<String>,
    pub verified_at: Option<String>,
}}

camel! {
pub struct RelatedItem {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub relationship_type: String,
    pub tool_slug: Option<String>,
    pub icon_key: Option<String>,
    pub accent_key: Option<String>,
}}

camel! {
/// The summary shape used by every card, list row, and search result.
pub struct ItemSummary {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub short_description: Option<String>,
    pub item_type: String,
    pub syntax: Option<String>,
    pub verification_status: String,
    pub lifecycle_status: String,
    pub icon_key: Option<String>,
    pub accent_key: Option<String>,
    pub tool_id: Option<String>,
    pub tool_slug: Option<String>,
    pub tool_name: Option<String>,
    pub tags: Vec<Tag>,
    pub is_favorite: bool,
    pub is_pinned: bool,
    pub has_examples: bool,
    pub has_sources: bool,
    pub updated_at: String,
}}

camel! {
/// Everything the detail screen can render. Sections whose data is absent are
/// simply empty here, and the UI omits them.
pub struct ItemDetail {
    #[serde(flatten)]
    pub summary: ItemSummary,
    pub full_description: Option<String>,
    pub when_to_use: Option<String>,
    pub expected_result: Option<String>,
    pub warning_text: Option<String>,
    pub result_diagram: Option<ResultDiagram>,
    pub difficulty: Option<String>,
    pub last_verified_at: Option<String>,
    pub managed_by: String,
    pub aliases: Vec<String>,
    pub examples: Vec<Example>,
    pub parameters: Vec<Parameter>,
    pub sources: Vec<ItemSource>,
    pub related: Vec<RelatedItem>,
    pub tasks: Vec<Task>,
    pub categories: Vec<Category>,
    pub notes: Vec<Note>,
}}

/// Typed, validated diagram payload for "expected result" flows.
/// Deliberately tiny and closed: nodes and edges only, no markup of any kind.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResultDiagram {
    pub kind: DiagramKind,
    pub nodes: Vec<DiagramNode>,
    #[serde(default)]
    pub edges: Vec<DiagramEdge>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagramKind {
    /// left-to-right (logical: start-to-end) flow of steps
    Flow,
    /// one parent fanning out to children
    FanOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiagramNode {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sublabel: Option<String>,
    #[serde(default)]
    pub accent_key: Option<String>,
    #[serde(default)]
    pub status: Option<DiagramStatus>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagramStatus {
    Ok,
    Pending,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiagramEdge {
    pub from: String,
    pub to: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

camel! {
pub struct Note {
    pub id: String,
    pub item_id: Option<String>,
    pub item_title: Option<String>,
    pub title: String,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
}}

camel! {
pub struct SavedLink {
    pub id: String,
    pub title: String,
    pub url: String,
    pub description: Option<String>,
    pub tool_id: Option<String>,
    pub tool_name: Option<String>,
    pub category_id: Option<String>,
    pub is_favorite: bool,
    pub sort_order: i64,
    pub updated_at: String,
}}

camel! {
pub struct Source {
    pub id: String,
    pub tool_id: Option<String>,
    pub tool_name: Option<String>,
    pub name: String,
    pub source_type: String,
    pub url: Option<String>,
    pub is_official: bool,
    pub sync_strategy: String,
    pub is_active: bool,
    pub check_interval_minutes: i64,
    pub last_checked_at: Option<String>,
    pub last_success_at: Option<String>,
    pub last_failure_at: Option<String>,
    pub last_error: Option<String>,
    pub next_check_at: Option<String>,
    /// Derived, never stored: ok | stale | failing | disabled | unchecked
    pub health: String,
    pub notes: Option<String>,
}}

camel! {
pub struct SearchFilters {
    #[serde(default)]
    pub tool_slugs: Vec<String>,
    #[serde(default)]
    pub task_slugs: Vec<String>,
    #[serde(default)]
    pub category_slugs: Vec<String>,
    #[serde(default)]
    pub item_types: Vec<String>,
    #[serde(default)]
    pub tag_slugs: Vec<String>,
    #[serde(default)]
    pub verification_statuses: Vec<String>,
    #[serde(default)]
    pub lifecycle_statuses: Vec<String>,
    #[serde(default)]
    pub favorites_only: bool,
}}


camel! {
pub struct SearchHit {
    #[serde(flatten)]
    pub item: ItemSummary,
    pub score: f64,
    /// 0..100, what the UI shows as the relevance chip.
    pub relevance: i64,
    pub matched_alias: Option<String>,
}}

camel! {
pub struct SearchResponse {
    pub query: String,
    pub normalized_query: String,
    pub total: i64,
    pub took_ms: f64,
    pub hits: Vec<SearchHit>,
}}

camel! {
pub struct UpdateBatch {
    pub id: String,
    pub source_id: Option<String>,
    pub created_by: String,
    pub producer: Option<String>,
    pub summary: Option<String>,
    pub notes: Option<String>,
    pub status: String,
    pub backup_id: Option<String>,
    pub created_at: String,
    pub applied_at: Option<String>,
    pub reverted_at: Option<String>,
    pub pending_count: i64,
    pub approved_count: i64,
    pub rejected_count: i64,
    pub applied_count: i64,
    pub add_count: i64,
    pub update_count: i64,
    pub remove_count: i64,
}}

camel! {
pub struct StagedChange {
    pub id: String,
    pub batch_id: String,
    pub entity_type: String,
    pub entity_key: String,
    pub change_type: String,
    pub title: Option<String>,
    pub detail: Option<String>,
    pub before_json: Option<serde_json::Value>,
    pub after_json: Option<serde_json::Value>,
    pub field_diff: Vec<FieldDiff>,
    pub status: String,
    pub confidence: Option<f64>,
    pub error_text: Option<String>,
    pub detected_at: String,
}}

camel! {
pub struct FieldDiff {
    pub field: String,
    pub before: Option<String>,
    pub after: Option<String>,
}}

camel! {
pub struct ApplyOutcome {
    pub batch_id: String,
    pub applied: i64,
    pub failed: i64,
    pub skipped: i64,
    pub backup_id: Option<String>,
    pub status: String,
}}

camel! {
pub struct BackupRecord {
    pub id: String,
    pub path: String,
    pub reason: String,
    pub batch_id: Option<String>,
    pub size_bytes: i64,
    pub schema_version: i64,
    pub item_count: i64,
    pub created_at: String,
}}

camel! {
pub struct AuditEntry {
    pub id: String,
    pub at: String,
    pub actor: String,
    pub action: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub batch_id: Option<String>,
    pub detail: Option<serde_json::Value>,
}}

camel! {
pub struct SavedSearch {
    pub id: String,
    pub name: String,
    pub query: String,
    pub filters: SearchFilters,
    pub created_at: String,
    pub last_run_at: Option<String>,
}}

camel! {
pub struct SearchHistoryEntry {
    pub id: String,
    pub query: String,
    pub result_count: i64,
    pub executed_at: String,
}}

camel! {
pub struct ActivityEntry {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub detail: Option<String>,
    pub item_slug: Option<String>,
    pub tool_slug: Option<String>,
    pub icon_key: Option<String>,
    pub accent_key: Option<String>,
    pub at: String,
}}

camel! {
pub struct HomeSummary {
    pub tools: Vec<ToolCard>,
    pub tasks: Vec<TaskCard>,
    pub recent_items: Vec<ItemSummary>,
    pub recent_activity: Vec<ActivityEntry>,
    pub pending_changes: i64,
    pub total_items: i64,
    pub total_examples: i64,
    pub last_sync_at: Option<String>,
    pub last_sync_status: Option<String>,
}}

camel! {
pub struct ToolPage {
    pub tool: Tool,
    pub stats: ToolStats,
    pub sections: Vec<CategoryNode>,
    pub tags: Vec<Tag>,
}}

camel! {
pub struct ToolStats {
    pub item_count: i64,
    pub example_count: i64,
    pub note_count: i64,
    pub source_count: i64,
    pub last_updated_at: Option<String>,
}}

camel! {
pub struct Diagnostics {
    pub app_version: String,
    pub schema_version: i64,
    pub expected_schema_version: i64,
    pub db_path: String,
    pub db_size_bytes: i64,
    pub integrity: String,
    pub foreign_key_violations: i64,
    pub fts_status: String,
    pub fts_row_count: i64,
    pub counts: std::collections::BTreeMap<String, i64>,
    pub last_batch: Option<UpdateBatch>,
    pub last_backup: Option<BackupRecord>,
    pub problems: Vec<String>,
}}
