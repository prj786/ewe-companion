//! Shrinking a hook payload before it crosses the socket (PLAN §10).
//!
//! - dropped: `tool_response` (often huge, never shown) and
//!   `transcript_path` (nothing of ours reads transcripts);
//! - every string: at most [`STRING_CAP`] bytes;
//! - except the diff-preview fields of `tool_input` (`old_string`,
//!   `new_string`, `content`, and the same inside `edits[]`): up to
//!   [`DIFF_CAP`] each and [`DIFF_TOTAL`] together.

use serde_json::Value;

pub const STRING_CAP: usize = 2 * 1024;
pub const DIFF_CAP: usize = 256 * 1024;
pub const DIFF_TOTAL: usize = 512 * 1024;

const DROPPED: &[&str] = &["tool_response", "transcript_path"];
const DIFF_FIELDS: &[&str] = &["old_string", "new_string", "content"];

/// Cut a string to at most `cap` bytes on a char boundary, marking the cut.
pub fn cut(s: &str, cap: usize) -> String {
    if s.len() <= cap {
        return s.to_string();
    }
    let mut end = cap;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &s[..end])
}

pub fn trim_payload(payload: &Value) -> Value {
    let mut budget = DIFF_TOTAL;
    match payload {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                if DROPPED.contains(&k.as_str()) {
                    continue;
                }
                let v = if k == "tool_input" {
                    trim_tool_input(v, &mut budget)
                } else {
                    trim_value(v)
                };
                out.insert(k.clone(), v);
            }
            Value::Object(out)
        }
        other => trim_value(other),
    }
}

fn trim_tool_input(v: &Value, budget: &mut usize) -> Value {
    match v {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, val) in map {
                let t = match (k.as_str(), val) {
                    (f, Value::String(s)) if DIFF_FIELDS.contains(&f) => {
                        Value::String(take_diff(s, budget))
                    }
                    ("edits", Value::Array(items)) => {
                        Value::Array(items.iter().map(|e| trim_tool_input(e, budget)).collect())
                    }
                    _ => trim_value(val),
                };
                out.insert(k.clone(), t);
            }
            Value::Object(out)
        }
        other => trim_value(other),
    }
}

fn take_diff(s: &str, budget: &mut usize) -> String {
    let cap = DIFF_CAP.min(*budget).max(STRING_CAP);
    let out = cut(s, cap);
    *budget = budget.saturating_sub(out.len());
    out
}

fn trim_value(v: &Value) -> Value {
    match v {
        Value::String(s) => Value::String(cut(s, STRING_CAP)),
        Value::Array(a) => Value::Array(a.iter().map(trim_value).collect()),
        Value::Object(m) => {
            Value::Object(m.iter().map(|(k, v)| (k.clone(), trim_value(v))).collect())
        }
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn drops_and_caps() {
        let p = json!({
            "transcript_path": "/x", "tool_response": {"big": "y"},
            "cwd": "c".repeat(5000),
            "tool_input": {"command": "z".repeat(5000), "new_string": "n".repeat(300_000)}
        });
        let t = trim_payload(&p);
        assert!(t.get("transcript_path").is_none());
        assert!(t.get("tool_response").is_none());
        assert!(t["cwd"].as_str().unwrap().len() <= STRING_CAP + 3);
        assert!(t["tool_input"]["command"].as_str().unwrap().len() <= STRING_CAP + 3);
        let n = t["tool_input"]["new_string"].as_str().unwrap();
        assert!(n.len() > STRING_CAP && n.len() <= DIFF_CAP + 3);
    }

    #[test]
    fn diff_total_is_shared() {
        let edits: Vec<Value> = (0..4)
            .map(|_| json!({"old_string": "o".repeat(200_000), "new_string": "n".repeat(200_000)}))
            .collect();
        let t = trim_payload(&json!({"tool_input": {"edits": edits}}));
        let total: usize = t["tool_input"]["edits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| {
                e["old_string"].as_str().unwrap().len() + e["new_string"].as_str().unwrap().len()
            })
            .sum();
        // the budget runs out, then each field still keeps STRING_CAP
        assert!(total <= DIFF_TOTAL + 8 * (STRING_CAP + 3));
    }

    #[test]
    fn cut_respects_char_boundaries() {
        let s = "ქართული".repeat(1000);
        let c = cut(&s, 100);
        assert!(c.ends_with('…'));
    }
}
