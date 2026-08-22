//! URL safety. Personal Wiki opens links in the user's browser and stores links
//! that came from untrusted imported packs, so every URL passes through here.

use crate::error::{CoreError, Result};

const ALLOWED_SCHEMES: [&str; 2] = ["https://", "http://"];

/// Accept only absolute http(s) URLs with a plausible host. Everything else —
/// `javascript:`, `data:`, `file:`, `vbscript:`, UNC paths, scheme-relative URLs,
/// control characters — is refused.
pub fn validate_web_url(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(CoreError::Invalid("url is empty".into()));
    }
    if trimmed
        .chars()
        .any(|c| c.is_control() || c == '\u{2028}' || c == '\u{2029}')
    {
        return Err(CoreError::Invalid("url contains control characters".into()));
    }
    if trimmed.starts_with("//") {
        return Err(CoreError::Invalid(
            "url must be absolute with an http(s) scheme".into(),
        ));
    }
    let lower = trimmed.to_ascii_lowercase();
    let scheme_ok = ALLOWED_SCHEMES.iter().any(|s| lower.starts_with(s));
    if !scheme_ok {
        return Err(CoreError::Invalid(format!(
            "unsupported url scheme in {trimmed:?}: only http and https are allowed"
        )));
    }
    let rest = match lower.split_once("://") {
        Some((_, rest)) => rest,
        None => return Err(CoreError::Invalid("malformed url".into())),
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host = host.rsplit('@').next().unwrap_or(host);
    let host_no_port = host.split(':').next().unwrap_or(host);
    if host_no_port.is_empty() || !host_no_port.contains('.') && host_no_port != "localhost" {
        return Err(CoreError::Invalid(format!(
            "url has no valid host: {trimmed:?}"
        )));
    }
    if host_no_port.contains(' ') {
        return Err(CoreError::Invalid("url host contains a space".into()));
    }
    Ok(trimmed.to_string())
}

pub fn is_safe_web_url(raw: &str) -> bool {
    validate_web_url(raw).is_ok()
}
