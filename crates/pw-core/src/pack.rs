//! Knowledge Pack: the only way external content enters Personal Wiki.
//!
//! A pack is untrusted input. It is parsed strictly (unknown fields are an error,
//! so a typo never silently drops content), then validated against every rule the
//! trusted database relies on, and only then staged for human review.

use crate::error::{CoreError, Result, ValidationIssue};
use crate::models::ResultDiagram;
use crate::url::validate_web_url;
use crate::{clock, Store};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub const SUPPORTED_SCHEMA_VERSIONS: &[&str] = &["1.0"];
const MAX_TEXT: usize = 20_000;
const MAX_TITLE: usize = 300;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KnowledgePack {
    pub schema_version: String,
    pub batch: BatchMeta,
    #[serde(default)]
    pub tools: Vec<PackTool>,
    #[serde(default)]
    pub tasks: Vec<PackTask>,
    #[serde(default)]
    pub categories: Vec<PackCategory>,
    #[serde(default)]
    pub tags: Vec<PackTag>,
    #[serde(default)]
    pub sources: Vec<PackSource>,
    #[serde(default)]
    pub items: Vec<PackItem>,
    /// Items a source no longer mentions. Never a hard delete: they become
    /// `remove_candidate` staged changes that a human must approve.
    #[serde(default)]
    pub removals: Vec<PackRemoval>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BatchMeta {
    pub producer: String,
    pub generated_at: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackTool {
    pub slug: String,
    pub name: String,
    #[serde(default)] pub external_key: Option<String>,
    #[serde(default)] pub short_name: Option<String>,
    #[serde(default)] pub description: Option<String>,
    #[serde(default)] pub icon_key: Option<String>,
    #[serde(default)] pub accent_key: Option<String>,
    #[serde(default)] pub official_home_url: Option<String>,
    #[serde(default)] pub sort_order: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackTask {
    pub slug: String,
    pub name: String,
    #[serde(default)] pub external_key: Option<String>,
    #[serde(default)] pub description: Option<String>,
    #[serde(default)] pub icon_key: Option<String>,
    #[serde(default)] pub sort_order: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackCategory {
    pub slug: String,
    pub name: String,
    #[serde(default)] pub external_key: Option<String>,
    #[serde(default)] pub description: Option<String>,
    #[serde(default)] pub icon_key: Option<String>,
    #[serde(default)] pub accent_key: Option<String>,
    #[serde(default)] pub parent_slug: Option<String>,
    #[serde(default)] pub sort_order: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackTag {
    pub slug: String,
    pub name: String,
    #[serde(default)] pub accent_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackSource {
    /// Stable key used by items to reference this source inside the pack.
    pub key: String,
    pub name: String,
    pub source_type: String,
    #[serde(default)] pub url: Option<String>,
    #[serde(default)] pub is_official: Option<bool>,
    #[serde(default)] pub tool_slug: Option<String>,
    #[serde(default)] pub sync_strategy: Option<String>,
    #[serde(default)] pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackItem {
    /// Stable external identity. `tool-slug:item-slug` by convention; required for
    /// every externally managed item so refreshes match on identity, not on title.
    pub external_key: String,
    pub slug: String,
    pub item_type: String,
    pub title: String,
    #[serde(default)] pub tool_slug: Option<String>,
    #[serde(default)] pub subtitle: Option<String>,
    #[serde(default)] pub short_description: Option<String>,
    #[serde(default)] pub full_description: Option<String>,
    #[serde(default)] pub syntax: Option<String>,
    #[serde(default)] pub when_to_use: Option<String>,
    #[serde(default)] pub expected_result: Option<String>,
    #[serde(default)] pub warning_text: Option<String>,
    #[serde(default)] pub result_diagram: Option<ResultDiagram>,
    #[serde(default)] pub verification_status: Option<String>,
    #[serde(default)] pub lifecycle_status: Option<String>,
    #[serde(default)] pub difficulty: Option<String>,
    #[serde(default)] pub icon_key: Option<String>,
    #[serde(default)] pub accent_key: Option<String>,
    #[serde(default)] pub sort_order: Option<i64>,
    #[serde(default)] pub last_verified_at: Option<String>,
    #[serde(default)] pub aliases: Vec<String>,
    #[serde(default)] pub tasks: Vec<String>,
    #[serde(default)] pub categories: Vec<String>,
    #[serde(default)] pub tags: Vec<String>,
    #[serde(default)] pub examples: Vec<PackExample>,
    #[serde(default)] pub parameters: Vec<PackParameter>,
    #[serde(default)] pub sources: Vec<PackItemSource>,
    #[serde(default)] pub relationships: Vec<PackRelationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackExample {
    pub code: String,
    #[serde(default)] pub title: Option<String>,
    #[serde(default)] pub description: Option<String>,
    #[serde(default)] pub expected_output: Option<String>,
    #[serde(default)] pub sort_order: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackParameter {
    pub name: String,
    #[serde(default)] pub description: Option<String>,
    #[serde(default)] pub is_required: Option<bool>,
    #[serde(default)] pub default_value: Option<String>,
    #[serde(default)] pub value_hint: Option<String>,
    #[serde(default)] pub sort_order: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackItemSource {
    /// References `sources[].key` in this pack.
    pub source_key: String,
    #[serde(default)] pub url: Option<String>,
    #[serde(default)] pub anchor: Option<String>,
    #[serde(default)] pub version: Option<String>,
    #[serde(default)] pub verified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackRelationship {
    pub to_external_key: String,
    pub relationship_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackRemoval {
    pub entity_type: String,
    pub external_key: String,
    #[serde(default)] pub reason: Option<String>,
}

pub const ITEM_TYPES: &[&str] = &[
    "command", "slash_command", "cli_command", "setting", "shortcut", "workflow",
    "link", "extension", "concept", "note", "snippet", "troubleshooting",
];
pub const VERIFICATION_STATUSES: &[&str] = &["official", "verified", "community", "unverified", "demo"];
pub const LIFECYCLE_STATUSES: &[&str] = &["active", "experimental", "deprecated", "removed"];
pub const SOURCE_TYPES: &[&str] = &[
    "official_docs", "official_repo", "official_changelog", "official_blog",
    "community", "local_file", "manual", "agent_managed",
];
pub const SYNC_STRATEGIES: &[&str] = &["agent", "manual", "local_file"];
pub const RELATIONSHIP_TYPES: &[&str] =
    &["related", "alternative", "prerequisite", "replaced_by", "similar_to", "follows", "part_of"];
pub const ACCENT_KEYS: &[&str] = &["blue", "green", "purple", "orange", "teal", "red", "slate"];

/// Parse JSON text into a pack. Parse errors are validation issues with a path,
/// not opaque serde messages.
pub fn parse(json: &str) -> Result<KnowledgePack> {
    match serde_json::from_str::<KnowledgePack>(json) {
        Ok(pack) => Ok(pack),
        Err(err) => {
            let path = if err.line() > 0 {
                format!("$ (line {}, column {})", err.line(), err.column())
            } else {
                "$".to_string()
            };
            Err(CoreError::Validation(vec![ValidationIssue::new(
                path,
                "malformed_json",
                format!("the file is not a valid Knowledge Pack: {err}"),
            )]))
        }
    }
}

struct Ctx<'a> {
    issues: Vec<ValidationIssue>,
    store: Option<&'a Store>,
}

impl Ctx<'_> {
    fn add(&mut self, path: impl Into<String>, code: &str, message: impl Into<String>) {
        self.issues.push(ValidationIssue::new(path, code, message));
    }

    fn text(&mut self, path: &str, value: &str, max: usize, required: bool) {
        if required && value.trim().is_empty() {
            self.add(path, "required", "this field must not be empty");
            return;
        }
        if value.len() > max {
            self.add(path, "too_long", format!("must be at most {max} characters"));
        }
        if value.contains('\0') {
            self.add(path, "unsafe_content", "text contains a NUL byte");
        }
        let lower = value.to_ascii_lowercase();
        // Item text is rendered as plain text, never as markup. Anything that looks
        // like injected markup or a script URL is refused rather than sanitized,
        // so a pack cannot smuggle active content through a display field.
        for needle in ["<script", "</script", "javascript:", "data:text/html", "<iframe", "onerror=", "onload="] {
            if lower.contains(needle) {
                self.add(path, "unsafe_content", format!("text contains disallowed content: {needle}"));
                break;
            }
        }
    }

    fn slug(&mut self, path: &str, value: &str) {
        if value.trim().is_empty() {
            self.add(path, "required", "slug must not be empty");
            return;
        }
        if value.len() > 120 {
            self.add(path, "too_long", "slug must be at most 120 characters");
        }
        let ok = value
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_');
        if !ok {
            self.add(path, "invalid_slug", "slug may contain letters, digits, '-' and '_' only");
        }
    }

    fn enum_value(&mut self, path: &str, value: &str, allowed: &[&str]) {
        if !allowed.contains(&value) {
            self.add(
                path,
                "invalid_enum",
                format!("{value:?} is not one of: {}", allowed.join(", ")),
            );
        }
    }

    fn url(&mut self, path: &str, value: &str) {
        if let Err(err) = validate_web_url(value) {
            self.add(path, "invalid_url", err.to_string());
        }
    }

    fn timestamp(&mut self, path: &str, value: &str) {
        if clock::parse_iso(value).is_none() {
            self.add(path, "invalid_timestamp", "must be an RFC3339 timestamp, e.g. 2026-08-21T12:00:00Z");
        }
    }
}

/// Full validation. `store` is optional: when provided, references may also
/// resolve against records already in the database (an incremental pack does not
/// have to restate the whole taxonomy).
pub fn validate(pack: &KnowledgePack, store: Option<&Store>) -> std::result::Result<(), Vec<ValidationIssue>> {
    let mut ctx = Ctx { issues: Vec::new(), store };

    if !SUPPORTED_SCHEMA_VERSIONS.contains(&pack.schema_version.as_str()) {
        ctx.add(
            "$.schemaVersion",
            "unsupported_schema_version",
            format!(
                "{:?} is not supported; this build accepts: {}",
                pack.schema_version,
                SUPPORTED_SCHEMA_VERSIONS.join(", ")
            ),
        );
    }
    ctx.text("$.batch.producer", &pack.batch.producer, 200, true);
    ctx.timestamp("$.batch.generatedAt", &pack.batch.generated_at);

    // ---- taxonomy, with in-pack uniqueness
    let mut tool_slugs = HashSet::new();
    for (i, tool) in pack.tools.iter().enumerate() {
        let p = format!("$.tools[{i}]");
        ctx.slug(&format!("{p}.slug"), &tool.slug);
        ctx.text(&format!("{p}.name"), &tool.name, MAX_TITLE, true);
        if let Some(desc) = &tool.description {
            ctx.text(&format!("{p}.description"), desc, MAX_TEXT, false);
        }
        if let Some(url) = &tool.official_home_url {
            ctx.url(&format!("{p}.officialHomeUrl"), url);
        }
        if let Some(accent) = &tool.accent_key {
            ctx.enum_value(&format!("{p}.accentKey"), accent, ACCENT_KEYS);
        }
        if !tool_slugs.insert(tool.slug.clone()) {
            ctx.add(format!("{p}.slug"), "duplicate_key", format!("tool slug {:?} appears twice in this pack", tool.slug));
        }
    }

    let mut task_slugs = HashSet::new();
    for (i, task) in pack.tasks.iter().enumerate() {
        let p = format!("$.tasks[{i}]");
        ctx.slug(&format!("{p}.slug"), &task.slug);
        ctx.text(&format!("{p}.name"), &task.name, MAX_TITLE, true);
        if !task_slugs.insert(task.slug.clone()) {
            ctx.add(format!("{p}.slug"), "duplicate_key", format!("task slug {:?} appears twice", task.slug));
        }
    }

    let mut category_slugs = HashSet::new();
    let mut parent_of: HashMap<String, String> = HashMap::new();
    for (i, cat) in pack.categories.iter().enumerate() {
        let p = format!("$.categories[{i}]");
        ctx.slug(&format!("{p}.slug"), &cat.slug);
        ctx.text(&format!("{p}.name"), &cat.name, MAX_TITLE, true);
        if !category_slugs.insert(cat.slug.clone()) {
            ctx.add(format!("{p}.slug"), "duplicate_key", format!("category slug {:?} appears twice", cat.slug));
        }
        if let Some(parent) = &cat.parent_slug {
            if parent == &cat.slug {
                ctx.add(format!("{p}.parentSlug"), "category_cycle", "a category cannot be its own parent");
            }
            parent_of.insert(cat.slug.clone(), parent.clone());
        }
    }
    // Cycle detection across the pack's own parent links.
    for start in parent_of.keys() {
        let mut seen = HashSet::new();
        let mut cursor = start.clone();
        while let Some(parent) = parent_of.get(&cursor) {
            if !seen.insert(parent.clone()) || parent == start {
                ctx.add(
                    format!("$.categories[slug={start}].parentSlug"),
                    "category_cycle",
                    "category parent chain forms a cycle",
                );
                break;
            }
            cursor = parent.clone();
        }
    }

    let mut tag_slugs = HashSet::new();
    for (i, tag) in pack.tags.iter().enumerate() {
        let p = format!("$.tags[{i}]");
        ctx.slug(&format!("{p}.slug"), &tag.slug);
        ctx.text(&format!("{p}.name"), &tag.name, MAX_TITLE, true);
        if let Some(accent) = &tag.accent_key {
            ctx.enum_value(&format!("{p}.accentKey"), accent, ACCENT_KEYS);
        }
        tag_slugs.insert(tag.slug.clone());
    }

    let mut source_keys = HashSet::new();
    let mut official_sources: HashMap<String, bool> = HashMap::new();
    for (i, src) in pack.sources.iter().enumerate() {
        let p = format!("$.sources[{i}]");
        ctx.slug(&format!("{p}.key"), &src.key);
        ctx.text(&format!("{p}.name"), &src.name, MAX_TITLE, true);
        ctx.enum_value(&format!("{p}.sourceType"), &src.source_type, SOURCE_TYPES);
        if let Some(strategy) = &src.sync_strategy {
            ctx.enum_value(&format!("{p}.syncStrategy"), strategy, SYNC_STRATEGIES);
        }
        let has_url = match &src.url {
            Some(url) => {
                ctx.url(&format!("{p}.url"), url);
                validate_web_url(url).is_ok()
            }
            None => false,
        };
        let is_official = src.is_official.unwrap_or(false);
        if is_official && !has_url && src.source_type != "local_file" {
            ctx.add(
                format!("{p}.url"),
                "missing_provenance",
                "a source marked official needs a valid https url",
            );
        }
        if !source_keys.insert(src.key.clone()) {
            ctx.add(format!("{p}.key"), "duplicate_key", format!("source key {:?} appears twice", src.key));
        }
        official_sources.insert(src.key.clone(), is_official && has_url);
    }

    // ---- items
    let mut item_keys = HashSet::new();
    let mut item_slugs = HashSet::new();
    for (i, item) in pack.items.iter().enumerate() {
        let p = format!("$.items[{i}]");
        validate_item(&mut ctx, &p, item, &tool_slugs, &task_slugs, &category_slugs, &tag_slugs, &source_keys, &official_sources);
        if !item_keys.insert(item.external_key.clone()) {
            ctx.add(
                format!("{p}.externalKey"),
                "duplicate_external_key",
                format!("external key {:?} appears more than once in this pack", item.external_key),
            );
        }
        if !item_slugs.insert(item.slug.clone()) {
            ctx.add(format!("{p}.slug"), "duplicate_key", format!("item slug {:?} appears twice", item.slug));
        }
    }
    // Relationship targets must exist somewhere: in this pack or already stored.
    for (i, item) in pack.items.iter().enumerate() {
        for (j, rel) in item.relationships.iter().enumerate() {
            let p = format!("$.items[{i}].relationships[{j}]");
            ctx.enum_value(&format!("{p}.relationshipType"), &rel.relationship_type, RELATIONSHIP_TYPES);
            if rel.to_external_key == item.external_key {
                ctx.add(format!("{p}.toExternalKey"), "self_reference", "an item cannot relate to itself");
                continue;
            }
            let in_pack = item_keys.contains(&rel.to_external_key);
            let in_db = ctx
                .store
                .map(|s| external_key_exists(s, "items", &rel.to_external_key))
                .unwrap_or(false);
            if !in_pack && !in_db {
                ctx.add(
                    format!("{p}.toExternalKey"),
                    "broken_reference",
                    format!("no item with external key {:?} in this pack or in the database", rel.to_external_key),
                );
            }
        }
    }

    for (i, removal) in pack.removals.iter().enumerate() {
        let p = format!("$.removals[{i}]");
        ctx.enum_value(
            &format!("{p}.entityType"),
            &removal.entity_type,
            &["item", "tool", "task", "category"],
        );
        ctx.text(&format!("{p}.externalKey"), &removal.external_key, 200, true);
    }

    if ctx.issues.is_empty() {
        Ok(())
    } else {
        Err(ctx.issues)
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_item(
    ctx: &mut Ctx,
    p: &str,
    item: &PackItem,
    tool_slugs: &HashSet<String>,
    task_slugs: &HashSet<String>,
    category_slugs: &HashSet<String>,
    tag_slugs: &HashSet<String>,
    source_keys: &HashSet<String>,
    official_sources: &HashMap<String, bool>,
) {
    ctx.text(&format!("{p}.externalKey"), &item.external_key, 200, true);
    if !item.external_key.contains(':') {
        ctx.add(
            format!("{p}.externalKey"),
            "invalid_external_key",
            "external key must be namespaced, e.g. \"codex-cli:agent-create\"",
        );
    }
    ctx.slug(&format!("{p}.slug"), &item.slug);
    ctx.text(&format!("{p}.title"), &item.title, MAX_TITLE, true);
    ctx.enum_value(&format!("{p}.itemType"), &item.item_type, ITEM_TYPES);

    for (field, value) in [
        ("subtitle", &item.subtitle),
        ("shortDescription", &item.short_description),
        ("fullDescription", &item.full_description),
        ("syntax", &item.syntax),
        ("whenToUse", &item.when_to_use),
        ("expectedResult", &item.expected_result),
        ("warningText", &item.warning_text),
    ] {
        if let Some(text) = value {
            ctx.text(&format!("{p}.{field}"), text, MAX_TEXT, false);
        }
    }

    let verification = item.verification_status.as_deref().unwrap_or("unverified");
    ctx.enum_value(&format!("{p}.verificationStatus"), verification, VERIFICATION_STATUSES);
    if let Some(lifecycle) = &item.lifecycle_status {
        ctx.enum_value(&format!("{p}.lifecycleStatus"), lifecycle, LIFECYCLE_STATUSES);
    }
    if let Some(difficulty) = &item.difficulty {
        ctx.enum_value(&format!("{p}.difficulty"), difficulty, &["beginner", "intermediate", "advanced"]);
    }
    if let Some(accent) = &item.accent_key {
        ctx.enum_value(&format!("{p}.accentKey"), accent, ACCENT_KEYS);
    }
    if let Some(ts) = &item.last_verified_at {
        ctx.timestamp(&format!("{p}.lastVerifiedAt"), ts);
    }

    if let Some(tool) = &item.tool_slug {
        let known = tool_slugs.contains(tool)
            || ctx.store.map(|s| slug_exists(s, "tools", tool)).unwrap_or(false);
        if !known {
            ctx.add(format!("{p}.toolSlug"), "broken_reference", format!("unknown tool slug {tool:?}"));
        }
    }
    for (j, task) in item.tasks.iter().enumerate() {
        let known = task_slugs.contains(task) || ctx.store.map(|s| slug_exists(s, "tasks", task)).unwrap_or(false);
        if !known {
            ctx.add(format!("{p}.tasks[{j}]"), "broken_reference", format!("unknown task slug {task:?}"));
        }
    }
    for (j, cat) in item.categories.iter().enumerate() {
        let known = category_slugs.contains(cat) || ctx.store.map(|s| slug_exists(s, "categories", cat)).unwrap_or(false);
        if !known {
            ctx.add(format!("{p}.categories[{j}]"), "broken_reference", format!("unknown category slug {cat:?}"));
        }
    }
    for (j, tag) in item.tags.iter().enumerate() {
        let known = tag_slugs.contains(tag) || ctx.store.map(|s| slug_exists(s, "tags", tag)).unwrap_or(false);
        if !known {
            ctx.add(format!("{p}.tags[{j}]"), "broken_reference", format!("unknown tag slug {tag:?}"));
        }
    }

    for (j, alias) in item.aliases.iter().enumerate() {
        ctx.text(&format!("{p}.aliases[{j}]"), alias, 200, true);
    }
    for (j, example) in item.examples.iter().enumerate() {
        ctx.text(&format!("{p}.examples[{j}].code"), &example.code, MAX_TEXT, true);
        if let Some(title) = &example.title {
            ctx.text(&format!("{p}.examples[{j}].title"), title, MAX_TITLE, false);
        }
    }
    for (j, param) in item.parameters.iter().enumerate() {
        ctx.text(&format!("{p}.parameters[{j}].name"), &param.name, 120, true);
    }

    let mut has_official_provenance = false;
    for (j, src) in item.sources.iter().enumerate() {
        let sp = format!("{p}.sources[{j}]");
        if !source_keys.contains(&src.source_key) {
            ctx.add(
                format!("{sp}.sourceKey"),
                "broken_reference",
                format!("no source with key {:?} declared in $.sources", src.source_key),
            );
            continue;
        }
        if let Some(url) = &src.url {
            ctx.url(&format!("{sp}.url"), url);
        }
        if let Some(ts) = &src.verified_at {
            ctx.timestamp(&format!("{sp}.verifiedAt"), ts);
        }
        if official_sources.get(&src.source_key).copied().unwrap_or(false) {
            has_official_provenance = true;
        }
    }
    // The rule that keeps the wiki honest: nothing may claim to be official
    // without a real official source behind it.
    if verification == "official" && !has_official_provenance {
        ctx.add(
            format!("{p}.verificationStatus"),
            "missing_provenance",
            "an item marked \"official\" must reference at least one official source with a valid url",
        );
    }

    if let Some(diagram) = &item.result_diagram {
        let ids: HashSet<&str> = diagram.nodes.iter().map(|n| n.id.as_str()).collect();
        if diagram.nodes.is_empty() {
            ctx.add(format!("{p}.resultDiagram.nodes"), "required", "a diagram needs at least one node");
        }
        if diagram.nodes.len() > 12 {
            ctx.add(format!("{p}.resultDiagram.nodes"), "too_long", "a diagram may have at most 12 nodes");
        }
        for (j, node) in diagram.nodes.iter().enumerate() {
            ctx.text(&format!("{p}.resultDiagram.nodes[{j}].label"), &node.label, 120, true);
            if let Some(accent) = &node.accent_key {
                ctx.enum_value(&format!("{p}.resultDiagram.nodes[{j}].accentKey"), accent, ACCENT_KEYS);
            }
        }
        for (j, edge) in diagram.edges.iter().enumerate() {
            if !ids.contains(edge.from.as_str()) {
                ctx.add(format!("{p}.resultDiagram.edges[{j}].from"), "broken_reference", format!("unknown node id {:?}", edge.from));
            }
            if !ids.contains(edge.to.as_str()) {
                ctx.add(format!("{p}.resultDiagram.edges[{j}].to"), "broken_reference", format!("unknown node id {:?}", edge.to));
            }
        }
    }
}

fn slug_exists(store: &Store, table: &str, slug: &str) -> bool {
    store
        .conn()
        .query_row(
            &format!("SELECT COUNT(*) FROM {table} WHERE slug = ?1"),
            rusqlite::params![slug],
            |r| r.get::<_, i64>(0),
        )
        .map(|n| n > 0)
        .unwrap_or(false)
}

fn external_key_exists(store: &Store, table: &str, key: &str) -> bool {
    store
        .conn()
        .query_row(
            &format!("SELECT COUNT(*) FROM {table} WHERE external_key = ?1"),
            rusqlite::params![key],
            |r| r.get::<_, i64>(0),
        )
        .map(|n| n > 0)
        .unwrap_or(false)
}

/// Convenience: parse + validate in one step.
pub fn parse_and_validate(json: &str, store: Option<&Store>) -> Result<KnowledgePack> {
    let pack = parse(json)?;
    validate(&pack, store).map_err(CoreError::Validation)?;
    Ok(pack)
}

/// Export the entire knowledge base as a Knowledge Pack.
///
/// The export is a valid pack: it can be validated and re-imported into a fresh
/// database, which is what makes the backup story understandable without SQLite.
pub fn export_all(store: &Store) -> Result<KnowledgePack> {
    use rusqlite::params;
    let conn = store.conn();

    let mut stmt = conn.prepare(
        "SELECT slug, name, short_name, description, icon_key, accent_key, official_home_url,
                sort_order, external_key FROM tools ORDER BY sort_order, name",
    )?;
    let tools = stmt
        .query_map([], |r| {
            Ok(PackTool {
                slug: r.get(0)?,
                name: r.get(1)?,
                short_name: r.get(2)?,
                description: r.get(3)?,
                icon_key: r.get(4)?,
                accent_key: r.get(5)?,
                official_home_url: r.get(6)?,
                sort_order: r.get(7)?,
                external_key: r.get(8)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT slug, name, description, icon_key, sort_order, external_key FROM tasks ORDER BY sort_order, name",
    )?;
    let tasks = stmt
        .query_map([], |r| {
            Ok(PackTask {
                slug: r.get(0)?,
                name: r.get(1)?,
                description: r.get(2)?,
                icon_key: r.get(3)?,
                sort_order: r.get(4)?,
                external_key: r.get(5)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    // Parents before children so a re-import can resolve parentSlug in one pass.
    let mut stmt = conn.prepare(
        "WITH RECURSIVE tree(id, slug, name, description, icon_key, accent_key, parent_slug, sort_order, external_key, depth) AS (
           SELECT id, slug, name, description, icon_key, accent_key, NULL, sort_order, external_key, 0
             FROM categories WHERE parent_id IS NULL
           UNION ALL
           SELECT c.id, c.slug, c.name, c.description, c.icon_key, c.accent_key, t.slug, c.sort_order, c.external_key, t.depth + 1
             FROM categories c JOIN tree t ON c.parent_id = t.id
         ) SELECT slug, name, description, icon_key, accent_key, parent_slug, sort_order, external_key FROM tree ORDER BY depth, sort_order, name",
    )?;
    let categories = stmt
        .query_map([], |r| {
            Ok(PackCategory {
                slug: r.get(0)?,
                name: r.get(1)?,
                description: r.get(2)?,
                icon_key: r.get(3)?,
                accent_key: r.get(4)?,
                parent_slug: r.get(5)?,
                sort_order: r.get(6)?,
                external_key: r.get(7)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut stmt = conn.prepare("SELECT slug, name, accent_key FROM tags ORDER BY name")?;
    let tags = stmt
        .query_map([], |r| Ok(PackTag { slug: r.get(0)?, name: r.get(1)?, accent_key: r.get(2)? }))?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT s.external_key, s.name, s.source_type, s.url, s.is_official, t.slug, s.sync_strategy, s.notes
         FROM sources s LEFT JOIN tools t ON t.id = s.tool_id ORDER BY s.name",
    )?;
    let sources = stmt
        .query_map([], |r| {
            let external: Option<String> = r.get(0)?;
            Ok(PackSource {
                key: external.unwrap_or_default().trim_start_matches("source:").to_string(),
                name: r.get(1)?,
                source_type: r.get(2)?,
                url: r.get(3)?,
                is_official: Some(r.get::<_, i64>(4)? == 1),
                tool_slug: r.get(5)?,
                sync_strategy: r.get(6)?,
                notes: r.get(7)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|s| !s.key.is_empty())
        .collect::<Vec<_>>();

    let mut stmt = conn.prepare(
        "SELECT i.id, i.external_key, i.slug, t.slug, i.item_type, i.title, i.subtitle,
                i.short_description, i.full_description, i.syntax, i.when_to_use, i.expected_result,
                i.warning_text, i.result_diagram, i.verification_status, i.lifecycle_status,
                i.difficulty, i.icon_key, i.accent_key, i.sort_order, i.last_verified_at
         FROM items i LEFT JOIN tools t ON t.id = i.tool_id ORDER BY i.sort_order, i.title",
    )?;
    let raw = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                PackItem {
                    external_key: String::new(),
                    slug: r.get(2)?,
                    tool_slug: r.get(3)?,
                    item_type: r.get(4)?,
                    title: r.get(5)?,
                    subtitle: r.get(6)?,
                    short_description: r.get(7)?,
                    full_description: r.get(8)?,
                    syntax: r.get(9)?,
                    when_to_use: r.get(10)?,
                    expected_result: r.get(11)?,
                    warning_text: r.get(12)?,
                    result_diagram: r
                        .get::<_, Option<String>>(13)?
                        .and_then(|d| serde_json::from_str(&d).ok()),
                    verification_status: r.get(14)?,
                    lifecycle_status: r.get(15)?,
                    difficulty: r.get(16)?,
                    icon_key: r.get(17)?,
                    accent_key: r.get(18)?,
                    sort_order: r.get(19)?,
                    last_verified_at: r.get(20)?,
                    aliases: vec![],
                    tasks: vec![],
                    categories: vec![],
                    tags: vec![],
                    examples: vec![],
                    parameters: vec![],
                    sources: vec![],
                    relationships: vec![],
                },
            ))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut items = Vec::with_capacity(raw.len());
    for (id, external_key, mut item) in raw {
        item.external_key = external_key.unwrap_or_else(|| format!("local:{}", item.slug));

        let strings = |sql: &str| -> Result<Vec<String>> {
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map(params![id], |r| r.get::<_, String>(0))?;
            Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
        };
        item.aliases = strings("SELECT alias FROM aliases WHERE item_id = ?1 ORDER BY alias")?;
        item.tasks = strings("SELECT t.slug FROM tasks t JOIN item_tasks it ON it.task_id = t.id WHERE it.item_id = ?1")?;
        item.categories = strings("SELECT c.slug FROM categories c JOIN item_categories ic ON ic.category_id = c.id WHERE ic.item_id = ?1")?;
        item.tags = strings("SELECT tg.slug FROM tags tg JOIN item_tags it ON it.tag_id = tg.id WHERE it.item_id = ?1")?;

        let mut stmt = conn.prepare(
            "SELECT title, description, code, expected_output, sort_order FROM examples WHERE item_id = ?1 ORDER BY sort_order",
        )?;
        item.examples = stmt
            .query_map(params![id], |r| {
                Ok(PackExample {
                    title: r.get(0)?,
                    description: r.get(1)?,
                    code: r.get(2)?,
                    expected_output: r.get(3)?,
                    sort_order: r.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let mut stmt = conn.prepare(
            "SELECT name, description, is_required, default_value, value_hint, sort_order
             FROM parameters WHERE item_id = ?1 ORDER BY sort_order",
        )?;
        item.parameters = stmt
            .query_map(params![id], |r| {
                Ok(PackParameter {
                    name: r.get(0)?,
                    description: r.get(1)?,
                    is_required: Some(r.get::<_, i64>(2)? == 1),
                    default_value: r.get(3)?,
                    value_hint: r.get(4)?,
                    sort_order: r.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let mut stmt = conn.prepare(
            "SELECT s.external_key, isr.source_url, isr.source_anchor, isr.source_version, isr.verified_at
             FROM item_sources isr JOIN sources s ON s.id = isr.source_id WHERE isr.item_id = ?1",
        )?;
        item.sources = stmt
            .query_map(params![id], |r| {
                let key: Option<String> = r.get(0)?;
                Ok(PackItemSource {
                    source_key: key.unwrap_or_default().trim_start_matches("source:").to_string(),
                    url: r.get(1)?,
                    anchor: r.get(2)?,
                    version: r.get(3)?,
                    verified_at: r.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let mut stmt = conn.prepare(
            "SELECT i2.external_key, r.relationship_type FROM relationships r
             JOIN items i2 ON i2.id = r.to_item_id WHERE r.from_item_id = ?1",
        )?;
        item.relationships = stmt
            .query_map(params![id], |r| {
                let key: Option<String> = r.get(0)?;
                Ok(PackRelationship {
                    to_external_key: key.unwrap_or_default(),
                    relationship_type: r.get(1)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|r| !r.to_external_key.is_empty())
            .collect();

        items.push(item);
    }

    Ok(KnowledgePack {
        schema_version: SUPPORTED_SCHEMA_VERSIONS[0].to_string(),
        batch: BatchMeta {
            producer: "personal-wiki-export".into(),
            generated_at: crate::clock::now_iso(),
            summary: Some(format!("full export: {} item(s)", items.len())),
            notes: None,
        },
        tools,
        tasks,
        categories,
        tags,
        sources,
        items,
        removals: vec![],
    })
}
