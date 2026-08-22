//! End-to-end CLI tests. These run the real `personal-wiki` binary against a
//! scratch database, which is exactly how an external agent uses it.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Cli {
    _dir: tempfile::TempDir,
    db: PathBuf,
}

impl Cli {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("wiki.db");
        Self { _dir: dir, db }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_personal-wiki"))
            .arg("--db")
            .arg(&self.db)
            .args(args)
            .output()
            .expect("run personal-wiki")
    }

    fn json(&self, args: &[&str]) -> Value {
        let mut full = vec!["--json"];
        full.extend_from_slice(args);
        let out = self.run(&full);
        let text = String::from_utf8_lossy(&out.stdout);
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("not json: {e}\n{text}"))
    }

    fn write_pack(&self, name: &str, body: &str) -> PathBuf {
        let path = self._dir.path().join(name);
        std::fs::write(&path, body).unwrap();
        path
    }
}

fn code(output: &Output) -> i32 {
    output.status.code().unwrap_or(-1)
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn status_seeds_a_fresh_database_and_reports_it() {
    let cli = Cli::new();
    let out = cli.run(&["status"]);
    assert_eq!(code(&out), 0);
    let text = stdout(&out);
    assert!(text.contains("integrity  ok"), "{text}");
    assert!(Path::new(&cli.db).exists());

    let json = cli.json(&["status"]);
    assert_eq!(json["integrity"], "ok");
    assert_eq!(json["ftsStatus"], "ok");
    assert!(json["counts"]["items"].as_i64().unwrap() > 0);
}

#[test]
fn tools_list_and_search_return_the_seeded_knowledge() {
    let cli = Cli::new();
    let tools = cli.json(&["tools", "list"]);
    assert!(tools.as_array().unwrap().len() >= 8);

    let results = cli.json(&["search", "claude doctor"]);
    assert!(results["total"].as_i64().unwrap() > 0);
    assert_eq!(results["hits"][0]["title"], "claude doctor");
}

#[test]
fn sources_manifest_gives_an_agent_what_it_needs_to_produce_a_pack() {
    let cli = Cli::new();
    let manifest = cli.json(&["sources", "manifest"]);
    assert_eq!(manifest["schemaVersion"], "1.0");
    assert!(manifest["sources"].as_array().unwrap().len() >= 5);
    assert!(manifest["tools"].as_array().unwrap().len() >= 8);
    assert!(manifest["workflow"].as_array().unwrap().len() >= 4);
    // The rules an agent must follow travel with the manifest.
    let rules = manifest["rules"].to_string();
    assert!(rules.contains("official"));
    assert!(manifest["taxonomy"]["itemTypes"]
        .as_array()
        .unwrap()
        .contains(&Value::from("cli_command")));
}

const GOOD_PACK: &str = r#"{
  "schemaVersion": "1.0",
  "batch": { "producer": "cli-test", "generatedAt": "2026-08-21T12:00:00Z",
             "summary": "دفعة اختبار" },
  "items": [{
    "externalKey": "claude-code:cli-test-item",
    "slug": "cli-test-item",
    "toolSlug": "claude-code",
    "itemType": "cli_command",
    "title": "cli-test-item",
    "shortDescription": "عنصر أضافه اختبار سطر الأوامر.",
    "syntax": "cli-test-item --run",
    "verificationStatus": "demo",
    "aliases": ["عنصر اختبار سطر الأوامر"]
  }]
}"#;

#[test]
fn validate_accepts_a_good_pack_and_rejects_a_bad_one_with_field_paths() {
    let cli = Cli::new();
    let good = cli.write_pack("good.json", GOOD_PACK);
    let out = cli.run(&["validate", good.to_str().unwrap()]);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));

    let bad = cli.write_pack(
        "bad.json",
        r#"{ "schemaVersion": "1.0",
             "batch": { "producer": "x", "generatedAt": "2026-08-21T12:00:00Z" },
             "items": [{ "externalKey": "x:y", "slug": "y", "itemType": "not_a_type",
                         "title": "Y", "toolSlug": "ghost" }] }"#,
    );
    let out = cli.run(&["--json", "validate", bad.to_str().unwrap()]);
    assert_eq!(code(&out), 3, "validation failures use exit code 3");
    let report: Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(report["ok"], false);
    let issues = report["issues"].as_array().unwrap();
    assert!(issues.iter().any(|i| i["code"] == "invalid_enum"));
    assert!(issues.iter().any(|i| i["code"] == "broken_reference"));
    assert!(issues
        .iter()
        .any(|i| i["path"].as_str().unwrap().contains("$.items[0]")));
}

#[test]
fn import_stages_by_default_and_never_applies_silently() {
    let cli = Cli::new();
    let before = cli.json(&["status"])["counts"]["items"].as_i64().unwrap();
    let pack = cli.write_pack("pack.json", GOOD_PACK);

    let staged = cli.json(&["import", pack.to_str().unwrap(), "--stage"]);
    assert_eq!(staged["ok"], true);
    let batch_id = staged["batchId"].as_str().unwrap().to_string();
    assert_eq!(staged["staged"], 1);

    // Still not in the trusted data.
    assert_eq!(
        cli.json(&["status"])["counts"]["items"].as_i64().unwrap(),
        before
    );
    let hits = cli.json(&["search", "cli-test-item"]);
    assert!(
        !hits["hits"]
            .as_array()
            .unwrap()
            .iter()
            .any(|h| h["slug"] == "cli-test-item"),
        "a staged item must not be searchable before it is approved"
    );

    // The diff is reviewable.
    let diff = cli.json(&["changes", "diff", &batch_id]);
    assert_eq!(diff["changes"][0]["changeType"], "add");
    assert_eq!(diff["changes"][0]["status"], "pending");

    // Applying without approving is refused.
    let refused = cli.run(&["changes", "apply", &batch_id]);
    assert_eq!(code(&refused), 4, "refusals use exit code 4");
    assert_eq!(
        cli.json(&["status"])["counts"]["items"].as_i64().unwrap(),
        before
    );

    // Approve, apply, and now it is browsable and searchable.
    assert_eq!(code(&cli.run(&["changes", "approve", &batch_id])), 0);
    let applied = cli.json(&["changes", "apply", &batch_id]);
    assert_eq!(applied["applied"], 1);
    assert!(
        applied["backupId"].is_string(),
        "a backup is taken before applying"
    );
    assert_eq!(
        cli.json(&["status"])["counts"]["items"].as_i64().unwrap(),
        before + 1
    );
    assert!(
        cli.json(&["search", "عنصر اختبار سطر الأوامر"])["total"]
            .as_i64()
            .unwrap()
            > 0
    );

    // And it can be undone.
    assert_eq!(code(&cli.run(&["changes", "undo", &batch_id])), 0);
    let batches = cli.json(&["changes", "list"]);
    let batch = batches
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == batch_id.as_str())
        .unwrap();
    assert_eq!(batch["status"], "reverted");
}

#[test]
fn reject_marks_a_batch_rejected_without_touching_trusted_data() {
    let cli = Cli::new();
    let before = cli.json(&["status"])["counts"]["items"].as_i64().unwrap();
    let pack = cli.write_pack("pack.json", GOOD_PACK);
    let batch_id = cli.json(&["import", pack.to_str().unwrap(), "--stage"])["batchId"]
        .as_str()
        .unwrap()
        .to_string();

    assert_eq!(code(&cli.run(&["changes", "reject", &batch_id])), 0);
    assert_eq!(
        cli.json(&["status"])["counts"]["items"].as_i64().unwrap(),
        before
    );
    let diff = cli.json(&["changes", "diff", &batch_id]);
    assert_eq!(diff["changes"][0]["status"], "rejected");
}

#[test]
fn import_with_apply_is_explicit_and_immediate() {
    let cli = Cli::new();
    let pack = cli.write_pack("pack.json", GOOD_PACK);
    let applied = cli.json(&["import", pack.to_str().unwrap(), "--apply"]);
    assert_eq!(applied["applied"], 1);
    let hits = cli.json(&["search", "cli-test-item"]);
    assert!(hits["hits"]
        .as_array()
        .unwrap()
        .iter()
        .any(|h| h["slug"] == "cli-test-item"));
}

#[test]
fn backup_list_and_restore_round_trip() {
    let cli = Cli::new();
    let snapshot = cli.json(&["backup"]);
    let backup_id = snapshot["id"].as_str().unwrap().to_string();
    assert!(Path::new(snapshot["path"].as_str().unwrap()).exists());

    let pack = cli.write_pack("pack.json", GOOD_PACK);
    cli.json(&["import", pack.to_str().unwrap(), "--apply"]);
    let after_import = cli.json(&["status"])["counts"]["items"].as_i64().unwrap();

    assert_eq!(code(&cli.run(&["restore", &backup_id])), 0);
    let after_restore = cli.json(&["status"])["counts"]["items"].as_i64().unwrap();
    assert!(
        after_restore < after_import,
        "restore rolled the knowledge back"
    );
    assert_eq!(cli.json(&["status"])["integrity"], "ok");

    let backups = cli.json(&["backups", "list"]);
    assert!(
        backups.as_array().unwrap().len() >= 2,
        "a safety snapshot is taken before restoring"
    );
}

#[test]
fn export_produces_a_pack_that_validates_and_re_imports() {
    let cli = Cli::new();
    let out_path = cli._dir.path().join("export.json");
    let exported = cli.json(&["export", out_path.to_str().unwrap()]);
    assert_eq!(exported["ok"], true);
    assert!(exported["items"].as_i64().unwrap() > 0);

    // Round trip: the export is itself a valid Knowledge Pack.
    let fresh = Cli::new();
    let out = fresh.run(&["validate", out_path.to_str().unwrap()]);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn doctor_reports_a_clean_database_and_repairs_a_damaged_index() {
    let cli = Cli::new();
    let out = cli.run(&["doctor"]);
    assert_eq!(code(&out), 0);
    assert!(stdout(&out).contains("no problems"), "{}", stdout(&out));

    // Break the index, then let doctor fix it.
    let store = pw_core::Store::open(&cli.db).unwrap();
    store.conn().execute("DELETE FROM items_fts", []).unwrap();
    drop(store);

    let broken = cli.run(&["doctor"]);
    assert_eq!(
        code(&broken),
        4,
        "problems are reported with a non-zero exit code"
    );

    let repaired = cli.json(&["doctor", "--repair"]);
    assert_eq!(repaired["diagnostics"]["ftsStatus"], "ok");
    assert_eq!(code(&cli.run(&["doctor"])), 0);
}

#[test]
fn an_unknown_command_and_a_missing_file_use_distinct_exit_codes() {
    let cli = Cli::new();
    assert_eq!(code(&cli.run(&["not-a-command"])), 1);
    assert_eq!(code(&cli.run(&["changes", "diff", "no-such-batch"])), 2);
    assert_eq!(code(&cli.run(&["validate", "/nonexistent/pack.json"])), 5);
}
