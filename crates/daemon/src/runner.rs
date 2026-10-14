//! Running a proposed shell command, after the user's Run click (the
//! proposal and its check: `companion_core::shell`, `chat.rs`).
//!
//! `bash -c` in the home folder, with the session the daemon was started in
//! (systemd's user environment: the Wayland display, the bus) and the
//! compositor the island lives in. Output and errors come back together,
//! the last lines kept for the chat. Two minutes at most: a long job belongs
//! in a terminal, which In terminal opens.

use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::AsyncReadExt;
use tokio::process::Command;

use crate::hub::Hub;

const BUDGET: Duration = Duration::from_secs(120);
/// what is read of the output at most
const CAP: usize = 64 * 1024;
/// what the chat shows of it
const LINES: usize = 12;

pub struct Ran {
    pub ok: bool,
    /// the exit code, or None when it was stopped
    pub code: Option<i32>,
    /// the last lines of what it printed
    pub output: String,
    pub timed_out: bool,
}

pub async fn run(hub: &Arc<Hub>, command: &str) -> Ran {
    let home = std::env::var_os("HOME").unwrap_or_else(|| "/".into());
    let mut c = Command::new("bash");
    // errors interleave with output, as in a terminal
    c.arg("-c")
        .arg(format!("exec 2>&1\n{command}"))
        .current_dir(&home)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        // its own process group, so a stop takes the whole pipeline with it
        .process_group(0)
        .kill_on_drop(true);
    let hypr = hub.env().hypr;
    if !hypr.is_empty() {
        c.env("HYPRLAND_INSTANCE_SIGNATURE", hypr);
    }
    let mut child = match c.spawn() {
        Ok(ch) => ch,
        Err(e) => {
            return Ran {
                ok: false,
                code: None,
                output: format!("bash would not start: {e}"),
                timed_out: false,
            };
        }
    };
    let pid = child.id();
    let mut out = child.stdout.take();
    let read = async {
        let mut buf = Vec::new();
        if let Some(o) = out.as_mut() {
            let mut chunk = [0u8; 8192];
            while let Ok(n) = o.read(&mut chunk).await {
                if n == 0 {
                    break;
                }
                // keep the end: the last lines are what the chat shows
                buf.extend_from_slice(&chunk[..n]);
                if buf.len() > CAP {
                    buf.drain(..buf.len() - CAP);
                }
            }
        }
        buf
    };
    let both = async {
        let buf = read.await;
        let status = child.wait().await;
        (buf, status)
    };
    match tokio::time::timeout(BUDGET, both).await {
        Ok((buf, status)) => {
            let code = status.ok().and_then(|s| s.code());
            Ran {
                ok: code == Some(0),
                code,
                output: tail(&buf),
                timed_out: false,
            }
        }
        Err(_) => {
            if let Some(pid) = pid.and_then(|p| i32::try_from(p).ok()) {
                // SAFETY: kill(2) on the process group this call started
                unsafe { libc::kill(-pid, libc::SIGKILL) };
            }
            Ran {
                ok: false,
                code: None,
                output: String::new(),
                timed_out: true,
            }
        }
    }
}

/// The last lines, without terminal colour codes.
fn tail(buf: &[u8]) -> String {
    let text = String::from_utf8_lossy(buf);
    let clean = strip_ansi(&text);
    let lines: Vec<&str> = clean
        .lines()
        .map(str::trim_end)
        .filter(|l| !l.is_empty())
        .collect();
    lines[lines.len().saturating_sub(LINES)..].join("\n")
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // CSI: ESC [ … final byte in @–~
            if chars.peek() == Some(&'[') {
                chars.next();
                for d in chars.by_ref() {
                    if ('@'..='~').contains(&d) {
                        break;
                    }
                }
            }
            continue;
        }
        if c == '\r' {
            continue;
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tail_is_the_last_lines_without_colours() {
        let mut s = String::new();
        for i in 0..20 {
            s.push_str(&format!("\u{1b}[32mline {i}\u{1b}[0m\r\n"));
        }
        let t = tail(s.as_bytes());
        assert_eq!(t.lines().count(), LINES);
        assert!(t.starts_with("line 8") && t.ends_with("line 19"), "{t}");
    }
}
