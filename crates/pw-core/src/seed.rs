//! Bootstrap content.
//!
//! The starter content is a Knowledge Pack and it is installed through the exact
//! same pipeline as any other pack — validate, stage, approve, apply — so the
//! shipped data cannot take a shortcut the update engine does not allow, and the
//! pipeline is exercised on every fresh install.

use crate::error::Result;
use crate::{apply, pack, staging, Store};

/// The taxonomy (tools, tasks, categories, tags, sources) comes first; the
/// content packs then reference it by slug.
pub const STARTER_PACKS: &[(&str, &str)] = &[
    ("taxonomy", include_str!("../../../seed/starter-pack.json")),
    ("claude-code", include_str!("../../../seed/packs/claude-code.json")),
    ("codex-cli", include_str!("../../../seed/packs/codex-cli.json")),
    ("opencode", include_str!("../../../seed/packs/opencode.json")),
    ("gemini-cli", include_str!("../../../seed/packs/gemini-cli.json")),
    ("git-github", include_str!("../../../seed/packs/git-github.json")),
];

/// Install a pack immediately (used for seeding a fresh database and by
/// `--apply` in tests). Returns the batch id.
pub fn install_pack(store: &mut Store, json: &str, actor: &str) -> Result<String> {
    let parsed = pack::parse_and_validate(json, Some(store))?;
    let batch_id = staging::stage_pack(store, &parsed, actor)?;
    staging::set_review_status(store, &batch_id, None, "approved")?;
    apply::apply_batch(store, &batch_id, actor)?;
    Ok(batch_id)
}

/// Seed a database that has no knowledge yet. Does nothing when content exists.
pub fn ensure_seeded(store: &mut Store) -> Result<bool> {
    let tools: i64 = store.count("tools")?;
    let items: i64 = store.count("items")?;
    if tools > 0 || items > 0 {
        return Ok(false);
    }
    for (name, json) in STARTER_PACKS {
        install_pack(store, json, "seed").map_err(|err| match err {
            crate::CoreError::Validation(issues) => crate::CoreError::Invalid(format!(
                "seed pack {name} is invalid: {}",
                issues.iter().map(|i| i.to_string()).collect::<Vec<_>>().join("; ")
            )),
            other => other,
        })?;
    }
    Ok(true)
}
