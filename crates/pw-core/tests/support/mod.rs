#![allow(dead_code)]
use pw_core::Store;

pub fn store() -> Store {
    let mut s = Store::open_in_memory().expect("open in-memory store");
    pw_core::seed::ensure_seeded(&mut s).expect("seed");
    s
}

pub fn empty_store() -> Store {
    Store::open_in_memory().expect("open in-memory store")
}

/// A minimal well-formed pack, used across the tests.
pub fn pack_json(items: &str, extra: &str) -> String {
    format!(
        r#"{{
  "schemaVersion": "1.0",
  "batch": {{ "producer": "test-agent", "generatedAt": "2026-08-21T12:00:00Z" }},
  {extra}
  "items": [{items}]
}}"#
    )
}

/// Stage + approve + apply, the way the app does it.
pub fn install(store: &mut Store, json: &str) -> String {
    pw_core::seed::install_pack(store, json, "test").expect("install pack")
}

/// A real on-disk database (backup/restore needs actual files).
pub fn temp_db() -> tempfile::TempDir {
    tempfile::tempdir().expect("temp dir")
}
