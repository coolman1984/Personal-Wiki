use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

/// Every timestamp stored by Personal Wiki is RFC3339 UTC text, so ordering is
/// lexicographic and the value is readable in the database and in exports.
pub fn now_iso() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into())
}

pub fn parse_iso(value: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(value, &Rfc3339).ok()
}

pub fn plus_minutes_iso(minutes: i64) -> String {
    let t = OffsetDateTime::now_utc() + time::Duration::minutes(minutes);
    t.format(&Rfc3339).unwrap_or_else(|_| now_iso())
}
