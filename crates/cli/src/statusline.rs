//! `ewe-companion statusline`: Claude Code's plan usage for the island
//! (PLAN §10.1, the usage pill).
//!
//! Claude Code runs its status-line command after each message and hands it
//! the session as JSON. This relay forwards **only `rate_limits`** (the
//! 5-hour and 7-day windows, Pro and Max plans) to the daemon within 300 ms,
//! then runs the command you had before (saved by `agents usage claude on`)
//! with the same input and prints its output, so your status line looks
//! exactly as it did. No daemon: it just runs your command.

use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use companion_core::{paths, proto};
use serde_json::{Value, json};

use crate::client;

pub const PREVIOUS: &str = "statusline-previous.json";
const OUT_CAP: usize = 64 * 1024;

pub fn previous_command() -> Option<String> {
    let v: Value =
        serde_json::from_str(&std::fs::read_to_string(paths::state_dir().join(PREVIOUS)).ok()?)
            .ok()?;
    v.get("statusLine")?
        .get("command")?
        .as_str()
        .map(str::to_string)
        .filter(|c| !c.trim().is_empty())
}

fn forward(input: &Value) {
    let Some(limits) = input.get("rate_limits").filter(|l| l.is_object()) else {
        return;
    };
    let Some(mut s) = client::connect(Duration::from_millis(300)) else {
        return;
    };
    let msg = json!({"v": proto::PROTOCOL, "type": "cli", "verb": "usage", "args": {"agent": "claude", "rate_limits": limits}});
    if client::send_line(&mut s, &msg, Duration::from_millis(300)).is_some() {
        let _ = client::read_line(&mut s, Instant::now() + Duration::from_millis(300), 4096);
    }
}

/// Run the user's own status-line command with the same stdin; its output,
/// capped, within 10 s.
fn run_previous(cmd: &str, raw: &[u8]) -> Vec<u8> {
    let Ok(mut child) = Command::new("sh")
        .args(["-c", cmd])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return Vec::new();
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(raw);
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
            _ => {
                let _ = child.kill();
                break;
            }
        }
    }
    let mut out = Vec::new();
    if let Some(mut so) = child.stdout.take() {
        let _ = so.by_ref().take(OUT_CAP as u64).read_to_end(&mut out);
    }
    out
}

pub fn run() {
    let mut raw = Vec::new();
    if std::io::stdin()
        .take(1 << 20)
        .read_to_end(&mut raw)
        .is_err()
    {
        return;
    }
    if let Ok(input) = serde_json::from_slice::<Value>(&raw) {
        forward(&input);
    }
    if let Some(cmd) = previous_command() {
        let out = run_previous(&cmd, &raw);
        let _ = std::io::stdout().write_all(&out);
    }
}
