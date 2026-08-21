//! Personal Wiki shared core.
//!
//! Everything that touches the trusted knowledge database lives here: schema and
//! migrations, repositories, FTS5 search, Knowledge Pack validation, staging and
//! diffing, transactional apply with backup and audit, restore/undo, and
//! diagnostics. The Tauri desktop application and the `personal-wiki` CLI are both
//! thin shells over this crate — the rules are implemented exactly once.

pub mod apply;
pub mod audit;
pub mod backup;
pub mod clock;
pub mod db;
pub mod diagnostics;
pub mod edit;
pub mod error;
pub mod ids;
pub mod models;
pub mod pack;
pub mod paths;
pub mod personal;
pub mod repo;
pub mod search;
pub mod seed;
pub mod staging;
pub mod url;

pub use db::{latest_schema_version, Store};
pub use error::{CoreError, Result, ValidationIssue};
