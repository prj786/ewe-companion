//! Short ids for cards and events: time-ordered, unique within a process.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Milliseconds since the Unix epoch.
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Seconds since the Unix epoch.
pub fn now_s() -> i64 {
    (now_ms() / 1000) as i64
}

/// `<prefix><ms-hex><counter-hex>`, e.g. `c19a3f0b2e1003`.
pub fn new_id(prefix: &str) -> String {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}{:x}{:03x}", now_ms(), n & 0xfff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_prefixed() {
        let a = new_id("c");
        let b = new_id("c");
        assert!(a.starts_with('c'));
        assert_ne!(a, b);
    }
}
