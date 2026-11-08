//! What the user asks Shaun to remember: "remember that my dentist is on
//! friday", "what do you remember", "forget about my dentist".
//!
//! Each memory is a row in `companion.db` (migration 003), the user's own
//! words, at most [`MAX`]. Remembering and forgetting one are T1 with Undo;
//! "forget everything" asks first (a button whose call carries a one-time
//! token, like closing a window). Memories are meant for the model: the
//! user asked for them, so [`prompt_block`] hands them over, and a list's
//! facts carry them. They are never logged (ids and counts only).
//!
//! A secret is never kept (C3): a password, a PIN or a code, a key or a
//! token, a card or bank number, a long run of digits or a random-looking
//! string is refused, with ewe Safe named as the place for it.

use std::sync::Arc;

use companion_core::commands::Call;
use companion_core::id::{new_id, now_ms};
use serde_json::{Value, json};
use tracing::info;

use crate::commands::{Offers, Outcome, done, failed, missed};
use crate::hub::Hub;
use crate::store::Memory;

/// The most memories kept.
pub const MAX: usize = 100;
/// What [`prompt_block`] hands a model at most.
const PROMPT_LINES: usize = 30;
const PROMPT_BYTES: usize = 2048;

pub const REFUSED: &str =
    "I won't keep that: it looks like a secret. ewe Safe is the place for it.";

/// What a "Forget all" button forgets: how many there were when offered.
static FORGET_ALL: Offers<usize> = Offers::new();

// ── secrets ─────────────────────────────────────────────────────────────────

/// Words that say a secret follows (whole words or phrases).
const SECRET_WORDS: &[&str] = &[
    "password",
    "passwords",
    "passwd",
    "passcode",
    "passphrase",
    "pin",
    "pin code",
    "pin number",
    "cvv",
    "cvc",
    "otp",
    "2fa",
    "totp",
    "token",
    "api key",
    "apikey",
    "api-key",
    "secret key",
    "private key",
    "ssh key",
    "seed phrase",
    "recovery phrase",
    "recovery code",
    "recovery codes",
    "backup code",
    "backup codes",
    "security code",
    "door code",
    "alarm code",
    "access code",
    "login code",
    "verification code",
    "unlock code",
    "iban",
    "card number",
    "credit card",
    "debit card",
    "bank card",
    "account number",
    "routing number",
    "sort code",
    "social security",
    "ssn",
    "wifi key",
    "wi-fi key",
    "master password",
];

/// Does this look like something for ewe Safe, not for a model?
pub fn looks_secret(text: &str) -> bool {
    let low = text.to_lowercase();
    // whole words and phrases: "pin" but not "spinach"
    let words: Vec<&str> = low
        .split(|c: char| !(c.is_alphanumeric() || c == '-'))
        .filter(|w| !w.is_empty())
        .collect();
    let joined = format!(" {} ", words.join(" "));
    if SECRET_WORDS
        .iter()
        .any(|w| joined.contains(&format!(" {w} ")))
    {
        return true;
    }
    // a long number, also in groups ("4111 1111 1111 1111"): a card, an
    // account; a date or a phone number is shorter
    let mut run = 0usize;
    let mut longest = 0usize;
    let mut gap = 0usize;
    for c in text.chars() {
        if c.is_ascii_digit() {
            run += 1;
            gap = 0;
        } else if (c == ' ' || c == '-') && run > 0 && gap == 0 {
            gap = 1;
        } else {
            run = 0;
            gap = 0;
        }
        longest = longest.max(run);
    }
    if longest >= 12 {
        return true;
    }
    // a token: long letters mixed with digits ("ghp_x8K2…", "sk-ant-…")
    text.split_whitespace().any(|t| {
        let t = t.trim_matches(|c: char| ".,;:!?\"'()[]{}<>".contains(c));
        if t.contains("://")
            || t.starts_with("www.")
            || t.starts_with('/')
            || t.starts_with('~')
            || t.contains('@')
        {
            return false;
        }
        let digits = t.chars().filter(char::is_ascii_digit).count();
        let lower = t.chars().any(|c| c.is_ascii_lowercase());
        let upper = t.chars().any(|c| c.is_ascii_uppercase());
        let symbol = t.chars().any(|c| !c.is_ascii_alphanumeric());
        let classes = [digits > 0, lower, upper, symbol]
            .iter()
            .filter(|x| **x)
            .count();
        (t.len() >= 16 && digits > 0 && (lower || upper))
            || (t.len() >= 10 && classes >= 3 && digits >= 2)
    })
}

// ── the commands ────────────────────────────────────────────────────────────

/// One line, no doubled spaces, no full stop at the end.
fn tidy(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_end_matches('.')
        .to_string()
}

/// "8 Nov" for when it was said, in local time (a memory like "on friday"
/// needs it).
fn said_on(ms: i64) -> String {
    let t = (ms / 1000) as libc::time_t;
    // SAFETY: tm is plain data; localtime_r writes only into it.
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    if unsafe { libc::localtime_r(&t, &mut tm) }.is_null() {
        return "?".into();
    }
    format!(
        "{} {}",
        tm.tm_mday,
        &companion_core::dates::MONTHS[tm.tm_mon.clamp(0, 11) as usize][..3]
    )
}

/// `remember {text}`: kept; Undo forgets it.
pub fn remember(hub: &Arc<Hub>, a: &Value) -> Outcome {
    let text = tidy(a["text"].as_str().unwrap_or(""));
    if text.is_empty() {
        return failed("Remember what?");
    }
    if looks_secret(&text) {
        info!("memory refused (looked like a secret)");
        return failed(REFUSED).private("I refused to keep something that looked like a secret.");
    }
    let all = hub.store().memories();
    if let Some(m) = all.iter().find(|m| m.text.eq_ignore_ascii_case(&text)) {
        return done(format!("I remember that already: {}", m.text), None)
            .facts(format!("Already remembered: {}", m.text));
    }
    if all.len() >= MAX {
        return failed(format!(
            "I keep {MAX} things at most: forget one first (\"what do you remember\")."
        ));
    }
    let m = Memory {
        id: new_id("m"),
        text: text.clone(),
        created: now_ms() as i64,
    };
    if !hub.store().memory_add(&m) {
        return failed("I could not keep that.");
    }
    info!(id = %m.id, "memory kept");
    done(
        format!("I'll remember that: {text}"),
        Some(Call::new("forget", json!({"id": m.id}))),
    )
    .facts(format!("Remembered: {text}"))
}

/// The words that tell memories apart ("my", "the", "about" count for
/// nothing).
fn key_words(q: &str) -> Vec<String> {
    const STOP: &[&str] = &[
        "my", "the", "a", "an", "about", "that", "this", "me", "i", "is", "are", "was", "on", "in",
        "at", "to", "of", "for", "and", "it", "what", "you", "your",
    ];
    q.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty() && !STOP.contains(w))
        .map(str::to_string)
        .collect()
}

/// Memories with every key word of the query (a word fits by its start:
/// "dentist" fits "dentist's").
fn matching(all: Vec<Memory>, q: &str) -> Vec<Memory> {
    let ks = key_words(q);
    if ks.is_empty() {
        return all;
    }
    all.into_iter()
        .filter(|m| {
            let words = key_words(&m.text);
            ks.iter().all(|k| {
                words.iter().any(|w| {
                    w.starts_with(k.as_str()) || (w.len() >= 4 && k.starts_with(w.as_str()))
                })
            })
        })
        .collect()
}

fn row(m: &Memory) -> Value {
    json!({"title": m.text, "detail": format!("said {}", said_on(m.created)), "icon": "remember",
           "actions": [{"label": "Forget", "command": "forget", "args": {"id": m.id}}]})
}

/// The memories as a model reads them: "- my dentist is on friday (said 8
/// Nov)", newest first, at most [`PROMPT_LINES`] lines and
/// [`PROMPT_BYTES`] bytes.
fn lines(ms: &[Memory]) -> String {
    let mut out = String::new();
    for (i, m) in ms.iter().enumerate() {
        let l = format!("- {} (said {})\n", m.text, said_on(m.created));
        if i >= PROMPT_LINES || out.len() + l.len() > PROMPT_BYTES {
            break;
        }
        out.push_str(&l);
    }
    out.trim_end().to_string()
}

/// What the user asked Shaun to remember, for a model's prompt: one short
/// line each, newest first, at most 30 lines and about 2 KB; empty when
/// there are none. The user's words as said, with the day they said them
/// ("friday" is relative).
// chat.rs puts it in the model's prompt (the look-up loop's change)
pub fn prompt_block(hub: &Hub) -> String {
    let all = hub.store().memories();
    lines(&all)
}

/// `memories {query?}`: rows with Forget.
pub fn list(hub: &Arc<Hub>, a: &Value) -> Outcome {
    let q = a.get("query").and_then(Value::as_str).unwrap_or("").trim();
    let all = hub.store().memories();
    let total = all.len();
    let found = matching(all, q);
    let line = match (found.len(), q.is_empty()) {
        (0, true) => {
            "I don't remember anything yet. Say \"remember that …\" and I will.".to_string()
        }
        (0, false) => format!("I don't remember anything about {q}."),
        (1, _) => "One thing:".to_string(),
        (n, _) => format!("{n} things:"),
    };
    let facts = if found.is_empty() {
        format!("{line} ({total} memories in all)")
    } else {
        format!("What the user asked me to remember:\n{}", lines(&found))
    };
    Outcome {
        rows: found.iter().map(row).collect(),
        ..done(line, None)
    }
    .facts(facts)
}

/// `forget {id | words | all}`: one memory (Undo puts it back), the one
/// that fits the words (several become rows), or all of them (a button
/// first).
pub fn forget(hub: &Arc<Hub>, a: &Value) -> Outcome {
    if a.get("all").and_then(Value::as_bool) == Some(true) {
        let n = hub.store().memories().len();
        if let Some(offered) = a
            .get("token")
            .and_then(Value::as_str)
            .and_then(|t| FORGET_ALL.take(t))
        {
            let gone = hub.store().memory_clear();
            info!(count = gone, offered, "memories forgotten");
            return done(
                match gone {
                    0 => "There was nothing to forget.".to_string(),
                    1 => "Forgot the one thing I remembered.".to_string(),
                    n => format!("Forgot all {n} things I remembered."),
                },
                None,
            )
            .line_is_facts();
        }
        if n == 0 {
            return done(
                "I don't remember anything, so there is nothing to forget.",
                None,
            )
            .line_is_facts();
        }
        let token = FORGET_ALL.keep(n);
        let (line, label) = if n == 1 {
            (
                "Forget the one thing I remember? There is no Undo.".to_string(),
                "Forget it".to_string(),
            )
        } else {
            (
                format!("Forget all {n} things I remember? There is no Undo."),
                format!("Forget all {n}"),
            )
        };
        return Outcome {
            action: Some((label, Call::new("forget", json!({"all": true, "token": token})))),
            ..done(line, None)
        }
        .facts("Offered a button to forget everything; nothing is forgotten unless the user clicks it.");
    }
    let all = hub.store().memories();
    let pick: Vec<Memory> = match a.get("id").and_then(Value::as_str) {
        Some(id) => all.into_iter().filter(|m| m.id == id).collect(),
        None => {
            let w = a.get("words").and_then(Value::as_str).unwrap_or("").trim();
            let found = matching(all, w);
            if key_words(w).is_empty() || found.is_empty() {
                return missed(format!("I don't remember anything about {w}."));
            }
            found
        }
    };
    match pick.as_slice() {
        [] => failed("I don't remember that any more."),
        [m] => {
            hub.store().memory_remove(&m.id);
            info!(id = %m.id, "memory forgotten");
            done(
                format!("Forgot: {}", m.text),
                Some(Call::new("remember", json!({"text": m.text}))),
            )
            .facts(format!("Forgot: {}", m.text))
        }
        many => Outcome {
            rows: many.iter().map(row).collect(),
            ..done(format!("{} things fit. Which one?", many.len()), None)
        }
        .facts(format!(
            "{} memories fit; the user picks one on the island.",
            many.len()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    fn hub() -> Arc<Hub> {
        Arc::new(Hub::new(Store::memory(), false))
    }

    #[test]
    fn secrets_are_never_kept() {
        for t in [
            "my password is hunter2",
            "the wifi password is sheep-1234",
            "my pin is 4821",
            "the door code is 1234",
            "my card number is 4111 1111 1111 1111",
            "4111-1111-1111-1111",
            "my iban is GE29NB0000000101904917",
            "api key sk-ant-api03-abcdefghijklmnop",
            "ghp_x8K2mP9qL3vN7wR4tY6uZ1aB",
            "the token is Xk9$mP2!qz",
            "my ssn is 078-05-1120",
            "my seed phrase is apple banana cherry",
        ] {
            assert!(looks_secret(t), "{t:?}");
        }
        for t in [
            "my dentist is on friday",
            "i like dark mode",
            "my mum's number is 555 123 4567",
            "the meeting is on 2026-12-25 at 10:30",
            "spinach is my favourite",
            "the wiki is at https://wiki.archlinux.org/title/Hyprland2026",
            "my email is sam@example.com",
            "i parked on level 3, spot 42",
            "the project lives in ~/Projects/ewe-companion2",
        ] {
            assert!(!looks_secret(t), "{t:?}");
        }
    }

    #[test]
    fn remember_list_and_forget() {
        let h = hub();
        let o = remember(&h, &json!({"text": "my dentist is on friday."}));
        assert!(o.ok, "{}", o.line);
        assert_eq!(o.line, "I'll remember that: my dentist is on friday");
        let undo = o.undo.unwrap();
        assert_eq!(undo.command, "forget");
        // twice is once
        assert!(
            remember(&h, &json!({"text": "My dentist is on Friday"}))
                .line
                .contains("already")
        );
        assert!(remember(&h, &json!({"text": "i like dark mode"})).ok);
        let o = remember(&h, &json!({"text": "my pin is 1234"}));
        assert!(!o.ok && o.line == REFUSED && o.private.is_some());
        assert_eq!(h.store().memories().len(), 2);
        // the list: newest first, with Forget
        let o = list(&h, &json!({}));
        assert_eq!(o.line, "2 things:");
        assert_eq!(o.rows[0]["title"], "i like dark mode");
        assert_eq!(o.rows[1]["actions"][0]["command"], "forget");
        assert!(
            o.facts
                .unwrap()
                .contains("- my dentist is on friday (said ")
        );
        assert_eq!(list(&h, &json!({"query": "my dentist"})).line, "One thing:");
        assert!(
            list(&h, &json!({"query": "cats"}))
                .line
                .contains("anything about cats")
        );
        // forget by words; Undo puts it back
        let o = forget(&h, &json!({"words": "my dentist"}));
        assert_eq!(o.line, "Forgot: my dentist is on friday");
        let back = o.undo.unwrap();
        assert_eq!(back.command, "remember");
        assert!(remember(&h, &back.args).ok);
        assert!(forget(&h, &json!({"words": "zebras"})).miss);
        // forget by id (the Undo of remember)
        let m = h.store().memories()[0].clone();
        assert!(forget(&h, &json!({"id": m.id})).ok);
        assert_eq!(h.store().memories().len(), 1);
    }

    #[test]
    fn forgetting_everything_asks_first() {
        let h = hub();
        remember(&h, &json!({"text": "a"}));
        remember(&h, &json!({"text": "b"}));
        let o = forget(&h, &json!({"all": true}));
        assert_eq!(o.line, "Forget all 2 things I remember? There is no Undo.");
        assert_eq!(h.store().memories().len(), 2, "nothing forgotten yet");
        let (label, call) = o.action.unwrap();
        assert_eq!(label, "Forget all 2");
        // a made-up token only offers the button again
        let o = forget(&h, &json!({"all": true, "token": "deadbeefdeadbeef"}));
        assert!(o.action.is_some());
        assert_eq!(h.store().memories().len(), 2);
        // the click
        let o = forget(&h, &call.args);
        assert_eq!(o.line, "Forgot all 2 things I remembered.");
        assert!(h.store().memories().is_empty());
        // the same token twice does nothing more
        assert!(forget(&h, &call.args).action.is_none());
    }

    #[test]
    fn the_prompt_block_is_short_and_newest_first() {
        let h = hub();
        assert_eq!(prompt_block(&h), "");
        for i in 0..40 {
            h.store().memory_add(&Memory {
                id: format!("m{i:03}"),
                text: format!("fact number {i}"),
                created: i,
            });
        }
        let b = prompt_block(&h);
        assert_eq!(b.lines().count(), PROMPT_LINES);
        assert!(b.starts_with("- fact number 39 (said "), "{b}");
        assert!(b.len() <= PROMPT_BYTES);
    }

    #[test]
    fn a_hundred_is_the_most() {
        let h = hub();
        for i in 0..MAX {
            assert!(remember(&h, &json!({"text": format!("thing {i}")})).ok);
        }
        let o = remember(&h, &json!({"text": "one more"}));
        assert!(!o.ok && o.line.contains("100 things at most"));
    }
}
