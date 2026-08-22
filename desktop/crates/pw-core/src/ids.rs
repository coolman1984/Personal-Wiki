//! ULID generation without pulling in a dependency.
//!
//! Crockford base32, 48-bit millisecond timestamp + 80 bits of randomness, so ids
//! are stable, unique, and lexicographically sortable by creation time.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
static COUNTER: AtomicU64 = AtomicU64::new(0);

fn seed() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut x = nanos ^ (n.wrapping_mul(0x9E37_79B9_7F4A_7C15)) ^ (std::process::id() as u64) << 32;
    // xorshift64*
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    x.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

pub fn ulid() -> String {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let r1 = seed();
    let r2 = seed();

    let mut out = String::with_capacity(26);
    // 10 chars of timestamp (48 bits)
    for i in (0..10).rev() {
        let shift = i * 5;
        let v = if shift >= 48 { 0 } else { (ms >> shift) & 0x1F };
        out.push(ALPHABET[v as usize] as char);
    }
    // 16 chars of randomness (80 bits)
    for i in (0..8).rev() {
        out.push(ALPHABET[((r1 >> (i * 5)) & 0x1F) as usize] as char);
    }
    for i in (0..8).rev() {
        out.push(ALPHABET[((r2 >> (i * 5)) & 0x1F) as usize] as char);
    }
    out
}

/// Turn arbitrary text into a url/database friendly slug. Arabic and other
/// non-ASCII letters are preserved (they are valid in slugs here) while
/// separators and punctuation collapse to single dashes.
pub fn slugify(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut last_dash = true;
    for ch in input.chars() {
        if ch.is_alphanumeric() {
            for l in ch.to_lowercase() {
                out.push(l);
            }
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}
