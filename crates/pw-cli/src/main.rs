//! `personal-wiki` — the agent-facing surface of Personal Wiki.
//!
//! It is a thin shell over `pw-core`: the same storage, validation, staging,
//! apply, backup and diagnostics code the desktop app uses. Nothing here
//! re-implements a rule.
//!
//! Exit codes:
//!   0 success · 1 usage error · 2 not found · 3 validation failed
//!   4 refused (would be unsafe) · 5 internal/database error

use pw_core::models::SearchFilters;
use pw_core::{apply, backup, diagnostics, pack, paths, personal, repo, search, seed, staging, Store};
use std::process::ExitCode;

const USAGE: &str = r#"personal-wiki — Personal Wiki knowledge tool

USAGE
  personal-wiki [--db <path>] [--json] <command> [args]

COMMANDS
  status                        database, counts, pending changes
  search "<query>"              search the knowledge base
  tools list                    list tool families
  sources list                  list configured sources
  sources manifest --json       machine-readable manifest for research agents
  validate <pack.json>          validate a Knowledge Pack (no writes)
  import <pack.json> --stage    validate and stage a pack for review (default)
  import <pack.json> --apply    stage, approve and apply in one step (explicit)
  changes list                  list update batches
  changes diff <batch-id>       show the staged diff of a batch
  changes approve <batch-id>    approve every pending change in a batch
  changes reject <batch-id>     reject every pending change in a batch
  changes apply <batch-id>      apply approved changes transactionally
  changes undo [batch-id]       reverse an applied batch (defaults to the last)
  backup                        create a snapshot
  backups list                  list snapshots
  restore <backup-id>           restore a snapshot (a safety snapshot is taken first)
  export <output.json>          export the whole knowledge base
  doctor [--repair]             check for problems and optionally fix them

GLOBAL FLAGS
  --db <path>    use a specific database file (default: platform data directory,
                 overridable with PERSONAL_WIKI_DB)
  --json         machine-readable output where the command supports it
"#;

struct Args {
    json: bool,
    db: Option<String>,
    rest: Vec<String>,
    flags: Vec<String>,
}

fn parse_args() -> Args {
    let mut json = false;
    let mut db = None;
    let mut rest = Vec::new();
    let mut flags = Vec::new();
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--db" => db = it.next(),
            other if other.starts_with("--") => flags.push(other.to_string()),
            other => rest.push(other.to_string()),
        }
    }
    Args { json, db, rest, flags }
}

fn main() -> ExitCode {
    let args = parse_args();
    match run(&args) {
        Ok(code) => code,
        Err(err) => {
            report_error(&err, args.json);
            match err {
                pw_core::CoreError::NotFound(_) => ExitCode::from(2),
                pw_core::CoreError::Validation(_) => ExitCode::from(3),
                pw_core::CoreError::Refused(_) => ExitCode::from(4),
                pw_core::CoreError::Invalid(_) => ExitCode::from(1),
                _ => ExitCode::from(5),
            }
        }
    }
}

fn report_error(err: &pw_core::CoreError, json: bool) {
    if let pw_core::CoreError::Validation(issues) = err {
        if json {
            let payload = serde_json::json!({ "ok": false, "error": err.to_string(), "issues": issues });
            println!("{}", serde_json::to_string_pretty(&payload).unwrap_or_default());
        } else {
            eprintln!("✗ {err}");
            for issue in issues {
                eprintln!("  {issue}");
            }
        }
        return;
    }
    if json {
        let payload = serde_json::json!({ "ok": false, "error": err.to_string() });
        println!("{}", serde_json::to_string_pretty(&payload).unwrap_or_default());
    } else {
        eprintln!("✗ {err}");
    }
}

fn open_store(args: &Args) -> pw_core::Result<Store> {
    let path = args
        .db
        .clone()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(paths::default_db_path);
    let mut store = Store::open(path)?;
    seed::ensure_seeded(&mut store)?;
    Ok(store)
}

fn print_json<T: serde::Serialize>(value: &T) {
    println!("{}", serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".into()));
}

fn run(args: &Args) -> pw_core::Result<ExitCode> {
    let Some(command) = args.rest.first().map(|s| s.as_str()) else {
        print!("{USAGE}");
        return Ok(ExitCode::from(1));
    };
    let sub = args.rest.get(1).map(|s| s.as_str());
    let third = args.rest.get(2).map(|s| s.as_str());

    match (command, sub) {
        ("help", _) | ("--help", _) => {
            print!("{USAGE}");
            Ok(ExitCode::SUCCESS)
        }

        ("status", _) => {
            let store = open_store(args)?;
            let diag = diagnostics::collect(&store)?;
            if args.json {
                print_json(&diag);
            } else {
                println!("Personal Wiki {}", diag.app_version);
                println!("database   {}", diag.db_path);
                println!("schema     v{} (expected v{})", diag.schema_version, diag.expected_schema_version);
                println!("integrity  {}", diag.integrity);
                println!("search     {} ({} rows)", diag.fts_status, diag.fts_row_count);
                for (table, count) in &diag.counts {
                    println!("  {table:<16} {count}");
                }
                if let Some(batch) = &diag.last_batch {
                    println!("last batch {} [{}] {} pending", batch.id, batch.status, batch.pending_count);
                }
            }
            Ok(ExitCode::SUCCESS)
        }

        ("search", _) => {
            let Some(query) = args.rest.get(1) else {
                eprintln!("usage: personal-wiki search \"<query>\"");
                return Ok(ExitCode::from(1));
            };
            let store = open_store(args)?;
            let response = search::search(&store, query, &SearchFilters::default(), 20, 0)?;
            if args.json {
                print_json(&response);
            } else {
                println!("{} result(s) in {:.1} ms", response.total, response.took_ms);
                for hit in &response.hits {
                    println!(
                        "  [{:>3}%] {:<40} {}",
                        hit.relevance,
                        hit.item.title,
                        hit.item.tool_name.clone().unwrap_or_default()
                    );
                    if let Some(syntax) = &hit.item.syntax {
                        println!("         {syntax}");
                    }
                }
            }
            Ok(ExitCode::SUCCESS)
        }

        ("tools", Some("list")) => {
            let store = open_store(args)?;
            let tools = repo::tool_cards(&store)?;
            if args.json {
                print_json(&tools);
            } else {
                for card in &tools {
                    println!(
                        "{:<16} {:<22} {} item(s), {} example(s)",
                        card.tool.slug, card.tool.name, card.item_count, card.example_count
                    );
                }
            }
            Ok(ExitCode::SUCCESS)
        }

        ("sources", Some("list")) => {
            let store = open_store(args)?;
            let sources = repo::list_sources(&store)?;
            if args.json {
                print_json(&sources);
            } else {
                for source in &sources {
                    println!(
                        "{:<10} {:<34} {:<16} {}",
                        source.health,
                        source.name,
                        source.source_type,
                        source.url.clone().unwrap_or_default()
                    );
                }
            }
            Ok(ExitCode::SUCCESS)
        }

        // The discovery surface for research agents: what should be checked, and
        // exactly what to produce.
        ("sources", Some("manifest")) => {
            let store = open_store(args)?;
            let sources = repo::list_sources(&store)?;
            let tools = repo::list_tools(&store, false)?;
            let manifest = serde_json::json!({
                "schemaVersion": pack::SUPPORTED_SCHEMA_VERSIONS[0],
                "generatedAt": pw_core::clock::now_iso(),
                "packSchema": "schemas/knowledge-pack.schema.json",
                "workflow": [
                    "read this manifest",
                    "research the listed official sources",
                    "produce a Knowledge Pack",
                    "personal-wiki validate <pack.json>",
                    "personal-wiki import <pack.json> --stage",
                    "a human reviews and approves in the Update Center"
                ],
                "rules": [
                    "verificationStatus \"official\" requires a real official source url",
                    "never invent syntax; screenshots are not sources",
                    "externalKey must be stable across refreshes, e.g. \"codex-cli:agent-create\""
                ],
                "tools": tools.iter().map(|t| serde_json::json!({
                    "slug": t.slug, "name": t.name, "officialHomeUrl": t.official_home_url
                })).collect::<Vec<_>>(),
                "sources": sources,
                "taxonomy": {
                    "tasks": repo::list_tasks(&store, false)?,
                    "categories": repo::category_tree(&store, None)?,
                    "itemTypes": pack::ITEM_TYPES,
                    "verificationStatuses": pack::VERIFICATION_STATUSES,
                    "lifecycleStatuses": pack::LIFECYCLE_STATUSES
                }
            });
            print_json(&manifest);
            Ok(ExitCode::SUCCESS)
        }

        ("validate", Some(path)) => {
            let store = open_store(args)?;
            let json = std::fs::read_to_string(path)?;
            let parsed = pack::parse_and_validate(&json, Some(&store))?;
            let summary = serde_json::json!({
                "ok": true,
                "schemaVersion": parsed.schema_version,
                "producer": parsed.batch.producer,
                "tools": parsed.tools.len(), "tasks": parsed.tasks.len(),
                "categories": parsed.categories.len(), "tags": parsed.tags.len(),
                "sources": parsed.sources.len(), "items": parsed.items.len(),
                "removals": parsed.removals.len()
            });
            if args.json {
                print_json(&summary);
            } else {
                println!("✓ pack is valid — {} item(s) from {}", parsed.items.len(), parsed.batch.producer);
            }
            Ok(ExitCode::SUCCESS)
        }

        ("import", Some(path)) => {
            let mut store = open_store(args)?;
            let json = std::fs::read_to_string(path)?;
            let parsed = pack::parse_and_validate(&json, Some(&store))?;
            // Staging is the default. Applying takes an explicit, separate flag.
            let apply_now = args.flags.iter().any(|f| f == "--apply");
            let batch_id = staging::stage_pack(&store, &parsed, "cli-agent")?;
            if apply_now {
                staging::set_review_status(&store, &batch_id, None, "approved")?;
                let outcome = apply::apply_batch(&mut store, &batch_id, "cli-agent")?;
                if args.json {
                    print_json(&outcome);
                } else {
                    println!("✓ applied {} change(s) — batch {batch_id}", outcome.applied);
                }
            } else {
                let changes = staging::list_changes(&store, Some(&batch_id), None)?;
                if args.json {
                    print_json(&serde_json::json!({ "ok": true, "batchId": batch_id, "staged": changes.len() }));
                } else {
                    println!("✓ staged {} change(s) for review — batch {batch_id}", changes.len());
                    println!("  review:  personal-wiki changes diff {batch_id}");
                    println!("  approve: personal-wiki changes approve {batch_id}");
                }
            }
            Ok(ExitCode::SUCCESS)
        }

        ("changes", Some("list")) => {
            let store = open_store(args)?;
            let batches = staging::list_batches(&store, 50)?;
            if args.json {
                print_json(&batches);
            } else {
                for batch in &batches {
                    println!(
                        "{}  {:<18} pending {:<4} approved {:<4} applied {:<4}  {}",
                        batch.id, batch.status, batch.pending_count, batch.approved_count,
                        batch.applied_count, batch.summary.clone().unwrap_or_default()
                    );
                }
            }
            Ok(ExitCode::SUCCESS)
        }

        ("changes", Some("diff")) => {
            let Some(batch_id) = third else {
                eprintln!("usage: personal-wiki changes diff <batch-id>");
                return Ok(ExitCode::from(1));
            };
            let store = open_store(args)?;
            let batch = staging::get_batch(&store, batch_id)?;
            let changes = staging::list_changes(&store, Some(batch_id), None)?;
            if args.json {
                print_json(&serde_json::json!({ "batch": batch, "changes": changes }));
            } else {
                println!("batch {} [{}]", batch.id, batch.status);
                for change in &changes {
                    println!(
                        "  {:<18} {:<10} {:<8} {}",
                        change.change_type, change.entity_type, change.status,
                        change.title.clone().unwrap_or_else(|| change.entity_key.clone())
                    );
                    for field in &change.field_diff {
                        println!(
                            "      {:<20} {:?} → {:?}",
                            field.field,
                            field.before.clone().unwrap_or_default(),
                            field.after.clone().unwrap_or_default()
                        );
                    }
                }
            }
            Ok(ExitCode::SUCCESS)
        }

        ("changes", Some(action @ ("approve" | "reject"))) => {
            let Some(batch_id) = third else {
                eprintln!("usage: personal-wiki changes {action} <batch-id>");
                return Ok(ExitCode::from(1));
            };
            let store = open_store(args)?;
            staging::get_batch(&store, batch_id)?;
            let status = if action == "approve" { "approved" } else { "rejected" };
            let n = staging::set_review_status(&store, batch_id, None, status)?;
            if args.json {
                print_json(&serde_json::json!({ "ok": true, "batchId": batch_id, "changed": n, "status": status }));
            } else {
                println!("✓ {n} change(s) {status}");
            }
            Ok(ExitCode::SUCCESS)
        }

        ("changes", Some("apply")) => {
            let Some(batch_id) = third else {
                eprintln!("usage: personal-wiki changes apply <batch-id>");
                return Ok(ExitCode::from(1));
            };
            let mut store = open_store(args)?;
            let outcome = apply::apply_batch(&mut store, batch_id, "cli-user")?;
            if args.json {
                print_json(&outcome);
            } else {
                println!(
                    "✓ applied {} change(s); backup {}",
                    outcome.applied,
                    outcome.backup_id.clone().unwrap_or_default()
                );
            }
            Ok(ExitCode::SUCCESS)
        }

        ("changes", Some("undo")) => {
            let mut store = open_store(args)?;
            let batch_id = match third {
                Some(id) => id.to_string(),
                None => apply::last_applied_batch(&store)?
                    .ok_or_else(|| pw_core::CoreError::NotFound("no applied batch to undo".into()))?,
            };
            let outcome = apply::revert_batch(&mut store, &batch_id, "cli-user")?;
            if args.json {
                print_json(&outcome);
            } else {
                println!("✓ reverted batch {batch_id}");
            }
            Ok(ExitCode::SUCCESS)
        }

        ("backup", _) => {
            let store = open_store(args)?;
            let record = backup::create(&store, "manual", None)?;
            if args.json {
                print_json(&record);
            } else {
                println!("✓ backup {} → {}", record.id, record.path);
            }
            Ok(ExitCode::SUCCESS)
        }

        ("backups", Some("list")) => {
            let store = open_store(args)?;
            let records = backup::list(&store)?;
            if args.json {
                print_json(&records);
            } else {
                for record in &records {
                    println!(
                        "{}  {:<12} {:>10} bytes  {}",
                        record.id, record.reason, record.size_bytes, record.created_at
                    );
                }
            }
            Ok(ExitCode::SUCCESS)
        }

        ("restore", Some(backup_id)) => {
            let mut store = open_store(args)?;
            let record = backup::restore(&mut store, backup_id)?;
            if args.json {
                print_json(&record);
            } else {
                println!("✓ restored backup {} ({} items)", record.id, record.item_count);
            }
            Ok(ExitCode::SUCCESS)
        }

        ("export", Some(path)) => {
            let store = open_store(args)?;
            let export = pw_core::pack::export_all(&store)?;
            std::fs::write(path, serde_json::to_string_pretty(&export)?)?;
            if args.json {
                print_json(&serde_json::json!({ "ok": true, "path": path, "items": export.items.len() }));
            } else {
                println!("✓ exported {} item(s) to {path}", export.items.len());
            }
            Ok(ExitCode::SUCCESS)
        }

        ("doctor", _) => {
            let store = open_store(args)?;
            let mut diag = diagnostics::collect(&store)?;
            let mut fixed = Vec::new();
            if args.flags.iter().any(|f| f == "--repair") {
                fixed = diagnostics::repair(&store)?;
                diag = diagnostics::collect(&store)?;
            }
            if args.json {
                print_json(&serde_json::json!({ "diagnostics": diag, "repaired": fixed }));
            } else {
                println!("Personal Wiki {} — {}", diag.app_version, diag.db_path);
                println!("integrity {} · foreign keys {} · search {}", diag.integrity, diag.foreign_key_violations, diag.fts_status);
                for done in &fixed {
                    println!("  fixed: {done}");
                }
                if diag.problems.is_empty() {
                    println!("✓ no problems found");
                } else {
                    for problem in &diag.problems {
                        println!("  ! {problem}");
                    }
                }
            }
            if diag.problems.is_empty() {
                Ok(ExitCode::SUCCESS)
            } else {
                Ok(ExitCode::from(4))
            }
        }

        // Small conveniences an agent or a test needs.
        ("notes", Some("list")) => {
            let store = open_store(args)?;
            print_json(&personal::list_notes(&store)?);
            Ok(ExitCode::SUCCESS)
        }

        _ => {
            eprintln!("unknown command: {}", args.rest.join(" "));
            eprint!("{USAGE}");
            Ok(ExitCode::from(1))
        }
    }
}
