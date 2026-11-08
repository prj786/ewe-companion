//! Windows and workspaces by asking: "switch to workspace 3", "move this
//! window to workspace 2", "make this window fullscreen", "minimise this
//! window", "close firefox", "firefox is frozen".
//!
//! Hyprland 0.56+ dispatches in Lua: `hyprctl dispatch "hl.dsp.…"`, the
//! same dispatchers ewe's `hyprland.lua` binds (Super+F, Super+Shift+F,
//! Super+V, Super+Z, Super+Q, Super+N). Every call passes the shell's
//! `HYPRLAND_INSTANCE_SIGNATURE` (`hub.env().hypr`): a user unit may not
//! have one. State comes from `hyprctl -j monitors|activewindow|clients|
//! workspaces`.
//!
//! Workspaces follow ewe's `workspaces.lua`: every display owns a block of
//! ids (the first display 1–10, the second 11–20 …), so "workspace 3" is id
//! `block * 10 + 3` of the display in focus.
//!
//! "This window" is the active one: the island is a layer surface, so the
//! last toplevel stays active while the user types. Lines name the app (its
//! class), never a window's title.
//!
//! T1 with Undo, except two that ask first: closing (the owner's choice) is
//! a "Close Firefox" button whose call carries a one-time token kept here
//! for two minutes, so a model can never close a window by itself (a
//! `window close` without a good token only offers the button); and
//! ending a frozen app (`force_quit`) is a proposal, `pkill -x <name>`
//! shown with Run.

use companion_core::commands::Call;
use serde_json::{Value, json};

use crate::commands::{Offers, Outcome, done, failed, missed, run_argv};

/// What a Close button closes: the windows' addresses and the app's name.
static CLOSES: Offers<(Vec<String>, String)> = Offers::new();

/// Ids per display (workspaces.lua's SPAN).
const SPAN: i64 = 10;

async fn hypr_json(what: &str, hypr: &str) -> Option<Value> {
    let out = run_argv("hyprctl", &["-j", what], hypr).await.ok()?;
    serde_json::from_str(&out).ok()
}

/// Run one Lua dispatcher. hyprctl answers "ok"; what reads like an error
/// is Hyprland's complaint (anything else counts as done: the wording of
/// a success was not seen on a live 0.56 compositor).
async fn dispatch(lua: &str, hypr: &str) -> Result<(), String> {
    let out = run_argv("hyprctl", &["dispatch", lua], hypr).await?;
    let low = out.to_lowercase();
    let bad = [
        "error",
        "invalid",
        "unknown",
        "expected",
        "not found",
        "failed",
        "no target",
        "couldn't",
    ];
    if bad.iter().any(|b| low.contains(b)) {
        Err(out.trim().to_string())
    } else {
        Ok(())
    }
}

/// The display in focus: (its name, the id of the workspace it shows).
async fn focused(hypr: &str) -> Option<(String, i64)> {
    let mons = hypr_json("monitors", hypr).await?;
    let m = mons
        .as_array()?
        .iter()
        .find(|m| m["focused"] == true)
        .or_else(|| mons.as_array()?.first())?;
    Some((
        m["name"].as_str().unwrap_or("").to_string(),
        m["activeWorkspace"]["id"].as_i64()?,
    ))
}

fn block_of(id: i64) -> i64 {
    if id >= 1 { (id - 1) / SPAN } else { 0 }
}

/// The number people see: id 13 is the second display's workspace 3.
fn number_of(id: i64) -> i64 {
    (id - 1).rem_euclid(SPAN) + 1
}

/// The next or previous workspace in use on this display, round the ends
/// (workspaces.lua's `cycle`, what Super+wheel does).
fn cycle(used: &[i64], current: i64, next: bool) -> i64 {
    let mut ns: Vec<i64> = used.iter().map(|id| number_of(*id)).collect();
    ns.sort();
    ns.dedup();
    let cur = number_of(current);
    if ns.is_empty() {
        return cur;
    }
    let at = ns.iter().position(|n| *n == cur).unwrap_or(0) as i64;
    let len = ns.len() as i64;
    ns[((at + if next { 1 } else { -1 }).rem_euclid(len)) as usize]
}

/// `workspace {number | step | id}`: Undo goes back to the one it left.
pub async fn workspace(a: &Value, hypr: &str) -> Outcome {
    let Some((mon, cur)) = focused(hypr).await else {
        return failed("I can't reach Hyprland to switch workspaces.");
    };
    let block = block_of(cur);
    let target = if let Some(n) = a.get("number").and_then(Value::as_i64) {
        block * SPAN + n
    } else if let Some(id) = a.get("id").and_then(Value::as_i64) {
        id
    } else {
        let used: Vec<i64> = hypr_json("workspaces", hypr)
            .await
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default()
            .iter()
            .filter(|w| w["monitor"] == mon.as_str())
            .filter_map(|w| w["id"].as_i64())
            .filter(|id| *id >= 1)
            .collect();
        block * SPAN + cycle(&used, cur, a["step"] == "next")
    };
    if target == cur {
        return done(
            format!("You're on workspace {} already", number_of(cur)),
            None,
        )
        .line_is_facts();
    }
    match dispatch(&format!("hl.dsp.focus({{ workspace = {target} }})"), hypr).await {
        Ok(()) => done(
            format!("Workspace {}", number_of(target)),
            (cur >= 1).then(|| Call::new("workspace", json!({"id": cur}))),
        )
        .line_is_facts(),
        Err(e) => failed(format!("Hyprland did not switch: {e}")),
    }
}

// ── windows ─────────────────────────────────────────────────────────────────

/// An app's name as people know it, from its class: `org.mozilla.firefox`
/// → Firefox, `ewe-settings` → ewe Settings.
pub fn app_name(class: &str) -> String {
    let c = class.trim();
    let low = c.to_lowercase();
    let known = match low.as_str() {
        "ewe-settings" => Some("ewe Settings"),
        "ewe-browser" => Some("ewe Browser"),
        "ewe-safe" => Some("ewe Safe"),
        "ewe-sync" => Some("ewe Sync"),
        "komble" | "komble-arch" => Some("Komble"),
        "code" | "code-oss" | "code-url-handler" => Some("VS Code"),
        "google-chrome" | "chrome" => Some("Chrome"),
        "kitty" => Some("kitty"),
        _ => None,
    };
    if let Some(k) = known {
        return k.into();
    }
    let last = c.rsplit('.').next().unwrap_or(c);
    let last = if last.is_empty() { c } else { last };
    let mut s = last.to_string();
    if let Some(f) = s.get_mut(..1) {
        f.make_ascii_uppercase();
    }
    if s.is_empty() { "the app".into() } else { s }
}

/// Does this window belong to the app the words name? Its class, or the
/// kinds people say ("the browser", "the terminal").
fn fits(c: &Value, words: &str) -> bool {
    let w = words.trim().to_lowercase();
    let joined = w.replace(' ', "");
    let kinds: &[&str] = match w.as_str() {
        "browser" | "web browser" | "the browser" => &[
            "browser",
            "firefox",
            "chrom",
            "brave",
            "zen",
            "librewolf",
            "vivaldi",
            "opera",
        ],
        "terminal" | "terminals" => &[
            "kitty",
            "alacritty",
            "foot",
            "wezterm",
            "terminal",
            "ghostty",
        ],
        "files" | "file manager" => &["nemo", "nautilus", "thunar", "dolphin"],
        "editor" | "text editor" => &["zed", "code", "gedit", "kate", "texteditor"],
        _ => &[],
    };
    ["class", "initialClass"].iter().any(|k| {
        let cls = c[*k].as_str().unwrap_or("").to_lowercase();
        if cls.is_empty() {
            return false;
        }
        let last = cls.rsplit('.').next().unwrap_or(&cls);
        cls == joined
            || last == joined
            || (joined.len() >= 3 && cls.contains(&joined))
            || (last.len() >= 3 && w.split(' ').any(|x| x == last))
            || kinds.iter().any(|k| cls.contains(k))
    })
}

/// The windows an outcome is about: one by its address (a button, an
/// Undo), an app's, or the window in front.
async fn targets(a: &Value, hypr: &str) -> Result<Vec<Value>, Box<Outcome>> {
    if let Some(addr) = a.get("address").and_then(Value::as_str) {
        let all = hypr_json("clients", hypr).await.unwrap_or(Value::Null);
        return match all
            .as_array()
            .and_then(|l| l.iter().find(|c| c["address"] == addr))
        {
            Some(c) => Ok(vec![c.clone()]),
            None => Err(Box::new(failed("That window is not open any more."))),
        };
    }
    if let Some(app) = a.get("app").and_then(Value::as_str) {
        let all = hypr_json("clients", hypr).await.unwrap_or(Value::Null);
        let found: Vec<Value> = all
            .as_array()
            .into_iter()
            .flatten()
            .filter(|c| c["mapped"] != false && fits(c, app))
            .cloned()
            .collect();
        return if found.is_empty() {
            Err(Box::new(missed(format!("No {app} window is open."))))
        } else {
            Ok(found)
        };
    }
    match hypr_json("activewindow", hypr).await {
        Some(w) if w["address"].as_str().is_some_and(|s| !s.is_empty()) => Ok(vec![w]),
        Some(_) => Err(Box::new(failed("No window is in front."))),
        None => Err(Box::new(failed(
            "I can't reach Hyprland to see the windows.",
        ))),
    }
}

fn addr_of(w: &Value) -> String {
    w["address"].as_str().unwrap_or("").to_string()
}

fn name_of(w: &Value) -> String {
    let class = w["class"]
        .as_str()
        .filter(|s| !s.is_empty())
        .or_else(|| w["initialClass"].as_str())
        .unwrap_or("");
    app_name(class)
}

/// 0 none, 1 maximised, 2 fullscreen (`hyprctl -j`'s `fullscreen`).
fn fullscreen_state(w: &Value) -> i64 {
    match &w["fullscreen"] {
        Value::Bool(true) => 2,
        v => v.as_i64().unwrap_or(0),
    }
}

/// `window {action, …}`.
pub async fn window(a: &Value, hypr: &str) -> Outcome {
    let action = a["action"].as_str().unwrap_or("");
    if action == "close" {
        return close(a, hypr).await;
    }
    if action == "restore" {
        return restore(hypr).await;
    }
    let wins = match targets(a, hypr).await {
        Ok(w) => w,
        Err(o) => return *o,
    };
    // an app with several windows: the most recently used one
    let w = wins
        .iter()
        .min_by_key(|w| w["focusHistoryID"].as_i64().unwrap_or(i64::MAX))
        .cloned()
        .unwrap_or(Value::Null);
    let addr = addr_of(&w);
    let name = name_of(&w);
    let at = format!("window = \"address:{addr}\"");
    let on = a.get("on").and_then(Value::as_bool).unwrap_or(true);
    let ws_id = w["workspace"]["id"].as_i64().unwrap_or(0);
    let result = match action {
        "move" => {
            let to = match (
                a.get("to").and_then(Value::as_i64),
                a.get("to_id").and_then(Value::as_i64),
            ) {
                (_, Some(id)) => id,
                (Some(n), None) => match focused(hypr).await {
                    Some((_, cur)) => block_of(cur) * SPAN + n,
                    None => return failed("I can't reach Hyprland to move the window."),
                },
                _ => return failed("Move it to which workspace?"),
            };
            if to == ws_id {
                return done(
                    format!("{name} is on workspace {} already", number_of(to)),
                    None,
                )
                .line_is_facts();
            }
            dispatch(
                &format!("hl.dsp.window.move({{ workspace = {to}, {at}, follow = false }})"),
                hypr,
            )
            .await
            .map(|()| {
                (
                    format!("Moved {name} to workspace {}", number_of(to)),
                    (ws_id >= 1).then(|| {
                        Call::new(
                            "window",
                            json!({"action": "move", "to_id": ws_id, "address": addr}),
                        )
                    }),
                )
            })
        }
        "fullscreen" | "maximize" => {
            let (mode, want, label) = if action == "fullscreen" {
                (0, 2, "fullscreen")
            } else {
                (1, 1, "maximised")
            };
            let now = fullscreen_state(&w);
            if (now == want) == on {
                return done(
                    if on {
                        format!("{name} is {label} already")
                    } else {
                        format!("{name} is not {label}")
                    },
                    None,
                )
                .line_is_facts();
            }
            let verb = if on { "set" } else { "unset" };
            dispatch(
                &format!(
                    "hl.dsp.window.fullscreen({{ mode = {mode}, action = \"{verb}\", {at} }})"
                ),
                hypr,
            )
            .await
            .map(|()| {
                (
                    if on {
                        format!("{name} is {label}")
                    } else {
                        format!("{name} is not {label} any more")
                    },
                    Some(Call::new(
                        "window",
                        json!({"action": action, "on": !on, "address": addr}),
                    )),
                )
            })
        }
        "float" => {
            let floating = w["floating"] == true;
            if floating == on {
                return done(
                    if on {
                        format!("{name} floats already")
                    } else {
                        format!("{name} is tiled already")
                    },
                    None,
                )
                .line_is_facts();
            }
            dispatch(
                &format!("hl.dsp.window.float({{ action = \"toggle\", {at} }})"),
                hypr,
            )
            .await
            .map(|()| {
                (
                    if on {
                        format!("{name} floats")
                    } else {
                        format!("{name} is tiled again")
                    },
                    Some(Call::new(
                        "window",
                        json!({"action": "float", "on": !on, "address": addr}),
                    )),
                )
            })
        }
        "minimize" => dispatch(
            &format!(
                "hl.dsp.window.move({{ workspace = \"special:minimized\", {at}, follow = false }})"
            ),
            hypr,
        )
        .await
        .map(|()| {
            (
                format!("Minimised {name} (Super+Shift+Z or the dock brings it back)"),
                (ws_id >= 1).then(|| {
                    Call::new(
                        "window",
                        json!({"action": "move", "to_id": ws_id, "address": addr}),
                    )
                }),
            )
        }),
        _ => return failed(format!("No window action called {action}.")),
    };
    match result {
        Ok((line, undo)) => done(line, undo).line_is_facts(),
        Err(e) => failed(format!("Hyprland did not do it: {e}")),
    }
}

/// Bring back the window minimised last (Super+Shift+Z), onto the
/// workspace in focus.
async fn restore(hypr: &str) -> Outcome {
    let all = hypr_json("clients", hypr).await.unwrap_or(Value::Null);
    let w = all
        .as_array()
        .into_iter()
        .flatten()
        .filter(|c| {
            matches!(
                c["workspace"]["name"].as_str(),
                Some("special:minimized" | "special:pen")
            )
        })
        .min_by_key(|c| c["focusHistoryID"].as_i64().unwrap_or(i64::MAX))
        .cloned();
    let Some(w) = w else {
        return done("No window is minimised.", None).line_is_facts();
    };
    let Some((_, cur)) = focused(hypr).await else {
        return failed("I can't reach Hyprland to bring it back.");
    };
    let addr = addr_of(&w);
    let name = name_of(&w);
    let at = format!("window = \"address:{addr}\"");
    let moved = dispatch(
        &format!("hl.dsp.window.move({{ workspace = {cur}, {at}, follow = false }})"),
        hypr,
    )
    .await;
    match moved {
        Ok(()) => {
            let _ = dispatch(&format!("hl.dsp.focus({{ {at} }})"), hypr).await;
            done(
                format!("Brought back {name}"),
                Some(Call::new(
                    "window",
                    json!({"action": "minimize", "address": addr}),
                )),
            )
            .line_is_facts()
        }
        Err(e) => failed(format!("Hyprland did not bring it back: {e}")),
    }
}

/// Closing asks first: without a good token, a Close button; with one (the
/// click), the windows close gracefully (the app may ask to save).
async fn close(a: &Value, hypr: &str) -> Outcome {
    if let Some((addrs, name)) = a
        .get("token")
        .and_then(Value::as_str)
        .and_then(|t| CLOSES.take(t))
    {
        let mut closed = 0;
        let mut err = None;
        for addr in &addrs {
            match dispatch(
                &format!("hl.dsp.window.close({{ window = \"address:{addr}\" }})"),
                hypr,
            )
            .await
            {
                Ok(()) => closed += 1,
                Err(e) => err = Some(e),
            }
        }
        return match (closed, err) {
            (0, Some(e)) => failed(format!("Hyprland did not close {name}: {e}")),
            (0, None) => failed(format!("{name} is not open any more.")),
            (1, _) => done(format!("Closed {name}"), None).line_is_facts(),
            (n, _) => done(format!("Closed {n} {name} windows"), None).line_is_facts(),
        };
    }
    // a model's call, a typed request, or a button gone stale: ask first
    let wins = match targets(a, hypr).await {
        Ok(w) => w,
        Err(o) => return *o,
    };
    let name = name_of(&wins[0]);
    let addrs: Vec<String> = wins.iter().map(addr_of).collect();
    let n = addrs.len();
    let token = CLOSES.keep((addrs, name.clone()));
    let (line, label) = if n == 1 {
        (
            format!("Close {name}? It may ask you to save first."),
            format!("Close {name}"),
        )
    } else {
        (
            format!("Close all {n} {name} windows? Each may ask you to save first."),
            format!("Close {n} windows"),
        )
    };
    Outcome {
        action: Some((
            label,
            Call::new("window", json!({"action": "close", "token": token})),
        )),
        ..done(line, None)
    }
    .facts(format!(
        "Offered a Close button for {name}; nothing closes unless the user clicks it."
    ))
}

// ── a frozen app: a proposal ────────────────────────────────────────────────

/// A process name `pkill -x` can take as it is.
fn plain_name(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with('-')
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "._+-".contains(c))
}

/// The process names that fit the words: the app's windows' processes,
/// else a running process of that name.
async fn process_names(app: &str, hypr: &str) -> Vec<String> {
    let mut names = Vec::new();
    if let Some(all) = hypr_json("clients", hypr).await {
        for c in all
            .as_array()
            .into_iter()
            .flatten()
            .filter(|c| fits(c, app))
        {
            if let Some(pid) = c["pid"].as_i64()
                && let Ok(comm) = std::fs::read_to_string(format!("/proc/{pid}/comm"))
            {
                let comm = comm.trim().to_string();
                if !names.contains(&comm) {
                    names.push(comm);
                }
            }
        }
    }
    if names.is_empty() {
        let want = app.trim().to_lowercase().replace(' ', "");
        let found = tokio::task::spawn_blocking(move || {
            let mut v: Vec<String> = Vec::new();
            for e in std::fs::read_dir("/proc").into_iter().flatten().flatten() {
                if !e
                    .file_name()
                    .to_string_lossy()
                    .chars()
                    .all(|c| c.is_ascii_digit())
                {
                    continue;
                }
                let Ok(comm) = std::fs::read_to_string(e.path().join("comm")) else {
                    continue;
                };
                let comm = comm.trim().to_string();
                if comm.to_lowercase() == want && !v.contains(&comm) {
                    v.push(comm);
                }
            }
            v
        })
        .await
        .unwrap_or_default();
        names = found;
    }
    names
}

/// `force_quit {app, force}`: never runs; shows `pkill -x <name>` with Run.
pub async fn force_quit(a: &Value, hypr: &str) -> Outcome {
    let app = a["app"].as_str().unwrap_or("").trim();
    let force = a.get("force").and_then(Value::as_bool).unwrap_or(true);
    let names: Vec<String> = process_names(app, hypr)
        .await
        .into_iter()
        .filter(|n| plain_name(n))
        .collect();
    let Some(proc_name) = names.first() else {
        return missed(format!("Nothing called {app} is running."));
    };
    let shown = app_name(app);
    let command = if force {
        format!("pkill -KILL -x {proc_name}")
    } else {
        format!("pkill -x {proc_name}")
    };
    let (line, about) = if force {
        (
            format!("Force quit {shown}? It ends at once: what it did not save is lost."),
            format!("Force quit {shown}"),
        )
    } else {
        (
            format!("End {shown}? It closes without asking to save."),
            format!("End {shown}"),
        )
    };
    Outcome {
        ok: true,
        line,
        propose: Some((command, about)),
        ..Default::default()
    }
    .facts(format!(
        "Offered to end {shown} with Run; nothing runs unless the user clicks it."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspaces_are_numbers_of_a_display() {
        assert_eq!(block_of(3), 0);
        assert_eq!(block_of(13), 1);
        assert_eq!(number_of(13), 3);
        assert_eq!(number_of(10), 10);
        assert_eq!(cycle(&[1, 2, 5], 2, true), 5);
        assert_eq!(cycle(&[1, 2, 5], 5, true), 1);
        assert_eq!(cycle(&[1, 2, 5], 1, false), 5);
        assert_eq!(cycle(&[11, 12], 12, true), 1);
        assert_eq!(cycle(&[], 4, true), 4);
    }

    #[test]
    fn apps_go_by_their_class() {
        assert_eq!(app_name("org.mozilla.firefox"), "Firefox");
        assert_eq!(app_name("firefox"), "Firefox");
        assert_eq!(app_name("ewe-settings"), "ewe Settings");
        assert_eq!(app_name("kitty"), "kitty");
        let ff = json!({"class": "firefox", "initialClass": "firefox"});
        assert!(fits(&ff, "firefox"));
        assert!(fits(&ff, "the browser"));
        assert!(fits(&json!({"class": "dev.zed.Zed"}), "zed"));
        assert!(
            fits(&json!({"class": "code-oss"}), "visual studio code")
                || fits(&json!({"class": "code"}), "visual studio code")
        );
        assert!(!fits(&ff, "steam"));
        assert!(!fits(&json!({"class": ""}), "firefox"));
        assert!(
            plain_name("firefox")
                && plain_name("Web.Content")
                && !plain_name("Isolated Web Co")
                && !plain_name("-9")
        );
        assert_eq!(fullscreen_state(&json!({"fullscreen": 2})), 2);
        assert_eq!(fullscreen_state(&json!({"fullscreen": true})), 2);
        assert_eq!(fullscreen_state(&json!({})), 0);
    }
}
