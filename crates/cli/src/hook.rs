//! `ewe-companion hook <agent> [<Event>]`: the agent hook relay (PLAN §10).
//!
//! It runs on every hook event, so it is std-only and quick, and it has one
//! rule above the others: **on any failure it prints nothing and exits 0**,
//! which Claude Code reads as "no opinion" and asks in its own terminal (C2).
//! It never prints an allow the daemon did not send, and the daemon sends
//! one only for a human's click (C1).
//!
//! Budgets: 300 ms to connect, 2 s to hand over an event that does not
//! wait, 110 s for an answer (the daemon gives up at 108 s, the hook's own
//! timeout is 120 s).

use std::io::{Read, Write};
use std::time::{Duration, Instant};

use companion_core::{claude, proto, trim};
use serde_json::{Value, json};

use crate::client;

/// Payloads past this are refused; anything under it is trimmed to well
/// under the socket's 1 MiB line before it is sent.
const STDIN_CAP: usize = 8 << 20;
const CONNECT: Duration = Duration::from_millis(300);
const HAND_OVER: Duration = Duration::from_secs(2);
const ANSWER: Duration = Duration::from_secs(110);

fn env(k: &str) -> Value {
    std::env::var(k)
        .ok()
        .filter(|v| !v.is_empty())
        .map(Value::String)
        .unwrap_or(Value::Null)
}

/// The parent of `pid`, from `/proc/<pid>/stat` (field 4, after the
/// parenthesised command name, which may itself hold spaces or parens).
fn ppid_of(pid: u32) -> Option<u32> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let after = &stat[stat.rfind(')')? + 1..];
    after.split_whitespace().nth(1)?.parse().ok()
}

/// Our ancestors, nearest first, up to `depth`: the terminal window that
/// hosts the agent is one of them.
fn ppid_chain(depth: usize) -> Vec<u32> {
    let mut chain = Vec::new();
    // SAFETY: getppid has no preconditions and cannot fail.
    let mut pid = unsafe { libc::getppid() } as u32;
    while pid > 1 && chain.len() < depth {
        chain.push(pid);
        match ppid_of(pid) {
            Some(p) => pid = p,
            None => break,
        }
    }
    chain
}

pub fn term_identity() -> Value {
    json!({
        "kitty_window_id": env("KITTY_WINDOW_ID"),
        "kitty_listen_on": env("KITTY_LISTEN_ON"),
        "kitty_pid": env("KITTY_PID"),
        "tmux": env("TMUX"),
        "tmux_pane": env("TMUX_PANE"),
        "term_program": env("TERM_PROGRAM"),
        "wezterm_pane": env("WEZTERM_PANE"),
        "ppid_chain": ppid_chain(8),
    })
}

/// `EWE_COMPANION_HOOK_LOG=<file>`: append every raw payload, for recording
/// test fixtures from a scratch project. Off unless set.
fn log_payload(event: &str, raw: &[u8]) {
    let Some(path) = std::env::var_os("EWE_COMPANION_HOOK_LOG").filter(|p| !p.is_empty()) else {
        return;
    };
    let Ok(payload) = serde_json::from_slice::<Value>(raw) else {
        return;
    };
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = writeln!(f, "{}", json!({"event": event, "payload": payload}));
    }
}

/// The relay, start to finish. Returns what to print (usually nothing).
pub fn relay(agent: &str, event_arg: Option<&str>, raw: &[u8]) -> Option<String> {
    if agent != "claude" || raw.len() > STDIN_CAP {
        return None; // other dialects arrive with P6
    }
    // the companion's own chat runs Claude Code too: those sessions are not
    // yours to watch, and never ask anything (they have no tools)
    if std::env::var_os("EWE_COMPANION_INTERNAL").is_some() {
        return None;
    }
    let payload: Value = serde_json::from_slice(raw).ok()?;
    let event = event_arg
        .filter(|e| !e.is_empty())
        .or_else(|| payload.get("hook_event_name").and_then(Value::as_str))?
        .to_string();
    log_payload(&event, raw);
    let wait = claude::waits(&event, &payload);
    let msg = json!({
        "v": proto::PROTOCOL, "type": "agent", "agent": agent, "event": event, "wait": wait,
        "term": term_identity(), "payload": trim::trim_payload(&payload),
    });
    let mut stream = client::connect(CONNECT)?;
    client::send_line(&mut stream, &msg, HAND_OVER)?;
    if !wait {
        return None;
    }
    let line = client::read_line(&mut stream, Instant::now() + ANSWER, 64 * 1024)?;
    let reply: proto::Reply = serde_json::from_str(&line).ok()?;
    claude::render(&event, &payload, &reply)
}

pub fn run(args: &[String]) {
    let agent = args.first().map(String::as_str).unwrap_or("");
    let mut raw = Vec::new();
    if std::io::stdin()
        .take(STDIN_CAP as u64 + 1)
        .read_to_end(&mut raw)
        .is_err()
    {
        return;
    }
    if let Some(out) = relay(agent, args.get(1).map(String::as_str), &raw) {
        let mut so = std::io::stdout().lock();
        let _ = writeln!(so, "{out}");
        let _ = so.flush();
    }
}
