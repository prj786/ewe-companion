//! Jump to the terminal an agent runs in (PLAN §10.1, decision 7).
//!
//! 1. The OS window: walk the hook's parent-pid chain against
//!    `hyprctl clients -j` and focus that window **by address**. Hyprland
//!    0.56's Lua config rejects the old `focuswindow pid:` dispatcher, so
//!    it is `hl.dsp.focus({ window = "address:0x…" })`, as the shell's own
//!    `Globals.focusToplevel` does.
//! 2. The kitty tab or split: `kitty @ --to $KITTY_LISTEN_ON focus-window`,
//!    when ewe's kitty config allows socket-only remote control.
//! 3. The tmux pane: `tmux select-window` + `select-pane` on its server.
//!
//! `hyprctl` needs the compositor's instance signature; the island sends
//! the shell's in its `env` message, because a user unit may not have it.

use std::process::Stdio;
use std::time::Duration;

use serde_json::{Value, json};
use tokio::process::Command;

const STEP: Duration = Duration::from_secs(3);

async fn run(cmd: &str, args: &[&str], hypr: &str) -> Option<String> {
    let mut c = Command::new(cmd);
    c.args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    if !hypr.is_empty() {
        c.env("HYPRLAND_INSTANCE_SIGNATURE", hypr);
    }
    let out = tokio::time::timeout(STEP, c.output()).await.ok()?.ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

fn chain(term: &Value) -> Vec<u64> {
    term.get("ppid_chain")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_u64).collect())
        .unwrap_or_default()
}

fn s<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}

/// The Hyprland window (address, pid) hosting this pid chain: the nearest
/// ancestor that owns a window.
pub fn window_for(clients: &Value, chain: &[u64]) -> Option<(String, u64)> {
    let list = clients.as_array()?;
    chain.iter().find_map(|pid| {
        list.iter()
            .find(|c| c.get("pid").and_then(Value::as_u64) == Some(*pid))
            .map(|c| (s(c, "address").to_string(), *pid))
    })
}

async fn clients(hypr: &str) -> Value {
    run("hyprctl", &["clients", "-j"], hypr)
        .await
        .and_then(|o| serde_json::from_str(&o).ok())
        .unwrap_or(Value::Null)
}

/// Is the agent's terminal the focused window right now?
pub async fn is_focused(term: &Value, hypr: &str) -> bool {
    let Some(active) = run("hyprctl", &["activewindow", "-j"], hypr).await else {
        return false;
    };
    let Ok(active) = serde_json::from_str::<Value>(&active) else {
        return false;
    };
    let Some(pid) = active.get("pid").and_then(Value::as_u64) else {
        return false;
    };
    chain(term).contains(&pid)
}

pub async fn jump(term: &Value, hypr: &str) -> Value {
    let mut window = false;
    if let Some((addr, _)) = window_for(&clients(hypr).await, &chain(term)) {
        let addr = if addr.starts_with("0x") {
            addr
        } else {
            format!("0x{addr}")
        };
        let dsp = format!("hl.dsp.focus({{ window = \"address:{addr}\" }})");
        window = run("hyprctl", &["dispatch", &dsp], hypr).await.is_some();
    }
    let mut tab = false;
    let (listen, win) = (s(term, "kitty_listen_on"), s(term, "kitty_window_id"));
    if !listen.is_empty() && !win.is_empty() {
        let m = format!("id:{win}");
        tab = run(
            "kitty",
            &["@", "--to", listen, "focus-window", "--match", &m],
            hypr,
        )
        .await
        .is_some();
    }
    let pane = s(term, "tmux_pane");
    if !pane.is_empty() {
        // $TMUX is "<socket>,<pid>,<session>": talk to that server
        let sock = s(term, "tmux").split(',').next().unwrap_or("");
        let mut base = vec![];
        if !sock.is_empty() {
            base.extend(["-S", sock]);
        }
        let sel_w = [base.clone(), vec!["select-window", "-t", pane]].concat();
        let sel_p = [base, vec!["select-pane", "-t", pane]].concat();
        tab =
            run("tmux", &sel_w, hypr).await.is_some() && run("tmux", &sel_p, hypr).await.is_some();
    }
    json!({"ok": window || tab, "window": window, "tab": tab})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_window_wins() {
        let clients = json!([{"address": "0xaaa", "pid": 10}, {"address": "0xbbb", "pid": 4242}]);
        // the shell (51200) has no window; kitty (4242) does
        assert_eq!(
            window_for(&clients, &[51234, 51200, 4242, 10]),
            Some(("0xbbb".into(), 4242))
        );
        assert_eq!(window_for(&clients, &[1, 2]), None);
        assert_eq!(window_for(&Value::Null, &[10]), None);
    }
}
