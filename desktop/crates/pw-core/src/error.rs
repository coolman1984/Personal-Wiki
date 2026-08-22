use std::fmt;

/// One error type for the whole core so the CLI and the Tauri layer report
/// failures identically.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("knowledge pack is invalid ({} problem(s))", .0.len())]
    Validation(Vec<ValidationIssue>),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid input: {0}")]
    Invalid(String),
    #[error("operation refused: {0}")]
    Refused(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;

/// A validation problem carrying the JSON path that caused it, so a human (or an
/// agent regenerating the pack) can fix exactly the right field.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ValidationIssue {
    pub path: String,
    pub code: String,
    pub message: String,
}

impl ValidationIssue {
    pub fn new(path: impl Into<String>, code: &str, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            code: code.to_string(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}]: {}", self.path, self.code, self.message)
    }
}
