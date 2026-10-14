//! Running a [`Call`] from the catalog (`companion_core::commands`).
//!
//! Every command here is a desktop action (T1, decision 6): it runs at
//! once, says in one line what it did, and offers Undo where there is a
//! way back. Commands go through argv only, never a shell, with a short
//! budget. The ones only the shell can do (Do not disturb, opening apps,
//! screenshots, the browser) are sent to the island as `do` messages, so an
//! app the companion opens belongs to the desktop, not to this daemon's
//! unit, and survives a daemon restart.

use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use companion_core::commands::{self as catalog, Call};
use serde_json::{Value, json};
use tokio::process::Command;

use crate::hub::Hub;
use crate::places;

const BUDGET: Duration = Duration::from_secs(5);

pub struct Outcome {
    pub ok: bool,
    pub line: String,
    pub undo: Option<Call>,
    /// the words named nothing here (no such app, no such folder): when
    /// they came from the offline matcher, the chat treats them as a
    /// question instead ("open the pod bay doors")
    pub miss: bool,
}

fn done(line: impl Into<String>, undo: Option<Call>) -> Outcome {
    Outcome {
        ok: true,
        line: line.into(),
        undo,
        miss: false,
    }
}

fn failed(line: impl Into<String>) -> Outcome {
    Outcome {
        ok: false,
        line: line.into(),
        undo: None,
        miss: false,
    }
}

fn missed(line: impl Into<String>) -> Outcome {
    Outcome {
        miss: true,
        ..failed(line)
    }
}

/// What the island answers when no app goes by that name.
const NOT_FOUND: &str = "not-found";

async fn run_argv(cmd: &str, args: &[&str], hypr: &str) -> Result<String, String> {
    let mut c = Command::new(cmd);
    c.args(args).stdin(Stdio::null()).kill_on_drop(true);
    if !hypr.is_empty() {
        c.env("HYPRLAND_INSTANCE_SIGNATURE", hypr);
    }
    match tokio::time::timeout(BUDGET, c.output()).await {
        Ok(Ok(o)) if o.status.success() => {
            Ok(String::from_utf8_lossy(&o.stdout).trim().to_string())
        }
        Ok(Ok(o)) => Err(String::from_utf8_lossy(&o.stderr).trim().to_string()),
        Ok(Err(_)) => Err(format!("{cmd} is not installed")),
        Err(_) => Err(format!("{cmd} did not answer")),
    }
}

// ── sound (PipeWire's wpctl) ───────────────────────────────────────────────

const SINK: &str = "@DEFAULT_AUDIO_SINK@";

/// (percent, muted) from `Volume: 0.40 [MUTED]`
pub(crate) async fn volume() -> Option<(i64, bool)> {
    let out = run_argv("wpctl", &["get-volume", SINK], "").await.ok()?;
    let frac: f64 = out.split_whitespace().nth(1)?.parse().ok()?;
    Some(((frac * 100.0).round() as i64, out.contains("MUTED")))
}

async fn volume_line() -> String {
    match volume().await {
        Some((_, true)) => "Muted".into(),
        Some((p, false)) => format!("Volume {p} %"),
        None => "Volume changed".into(),
    }
}

// ── brightness (brightnessctl) ─────────────────────────────────────────────

/// the percent from `intel_backlight,backlight,240,25%,960`
pub(crate) async fn brightness() -> Option<i64> {
    let out = run_argv("brightnessctl", &["-m"], "").await.ok()?;
    out.lines()
        .next()?
        .split(',')
        .nth(3)?
        .trim_end_matches('%')
        .parse()
        .ok()
}

/// What an `open` says it did. Settings 0.18 opens on Appearance whatever
/// page is asked for, so the line names the page to pick.
fn open_line(what: &str, page: &str) -> String {
    let label = catalog::page_label(what, page);
    match (what, label) {
        ("settings", Some("Appearance") | None) => "Opened Settings".into(),
        ("settings", Some(l)) => format!("Opened Settings: pick {l} in the sidebar"),
        ("komble", Some(l)) if page.starts_with("search=") => {
            format!("Opened Komble's {l}, searching for {}", &page[7..])
        }
        ("komble", Some(l)) => format!("Opened {l} in Komble"),
        ("komble", None) => "Opened Komble".into(),
        ("quicksettings", Some("Quick settings") | None) => "Opened Quick settings".into(),
        ("quicksettings", Some(l)) => format!("Opened {l} in Quick settings"),
        ("overview", _) => "Opened the Overview".into(),
        ("sync", _) => "Opened ewe-sync".into(),
        ("terminal", _) => "Opened a terminal".into(),
        _ => "Opened Files".into(),
    }
}

pub async fn run(hub: &Arc<Hub>, call: &Call) -> Outcome {
    if let Err(e) = catalog::validate(call) {
        return failed(e);
    }
    let a = &call.args;
    let hypr = hub.env().hypr;
    match call.command.as_str() {
        "volume_set" | "volume_change" => {
            let before = volume().await;
            let arg = if call.command == "volume_set" {
                format!("{:.2}", a["percent"].as_i64().unwrap_or(0) as f64 / 100.0)
            } else {
                let d = a["delta"].as_i64().unwrap_or(0);
                format!("{}%{}", d.abs(), if d >= 0 { "+" } else { "-" })
            };
            match run_argv("wpctl", &["set-volume", "-l", "1.5", SINK, &arg], "").await {
                Ok(_) => {
                    // turning it up while muted means "I want to hear it"
                    if before.is_some_and(|(_, m)| m) && call.command == "volume_set" {
                        let _ = run_argv("wpctl", &["set-mute", SINK, "0"], "").await;
                    }
                    done(
                        volume_line().await,
                        before.map(|(p, _)| Call::new("volume_set", json!({"percent": p}))),
                    )
                }
                Err(_) => failed("Could not change the volume (wpctl)."),
            }
        }
        "mute" => {
            let state = match a.get("on").and_then(Value::as_bool) {
                Some(true) => "1",
                Some(false) => "0",
                None => "toggle",
            };
            match run_argv("wpctl", &["set-mute", SINK, state], "").await {
                Ok(_) => {
                    let muted = volume().await.is_some_and(|(_, m)| m);
                    done(
                        if muted { "Muted" } else { "Sound on" },
                        Some(Call::new("mute", json!({"on": !muted}))),
                    )
                }
                Err(_) => failed("Could not mute (wpctl)."),
            }
        }
        "brightness_set" | "brightness_change" => {
            let before = brightness().await;
            let arg = if call.command == "brightness_set" {
                format!("{}%", a["percent"].as_i64().unwrap_or(50))
            } else {
                let d = a["delta"].as_i64().unwrap_or(0);
                format!("{}%{}", d.abs(), if d >= 0 { "+" } else { "-" })
            };
            match run_argv("brightnessctl", &["-q", "set", &arg], "").await {
                Ok(_) => {
                    let now = brightness()
                        .await
                        .map(|p| format!("Brightness {p} %"))
                        .unwrap_or("Brightness changed".into());
                    done(
                        now,
                        before.map(|p| Call::new("brightness_set", json!({"percent": p.max(1)}))),
                    )
                }
                Err(_) => {
                    failed("This screen's brightness cannot be changed here (brightnessctl).")
                }
            }
        }
        "media" => {
            let verb = match a["action"].as_str().unwrap_or("") {
                "next" => "next",
                "previous" => "previous",
                _ => "play-pause",
            };
            match run_argv("playerctl", &[verb], "").await {
                Ok(_) if verb == "play-pause" => {
                    tokio::time::sleep(Duration::from_millis(150)).await;
                    let st = run_argv("playerctl", &["status"], "")
                        .await
                        .unwrap_or_default();
                    done(
                        if st == "Playing" { "Playing" } else { "Paused" },
                        Some(call.clone()),
                    )
                }
                Ok(_) => done(
                    if verb == "next" {
                        "Next track"
                    } else {
                        "Previous track"
                    },
                    None,
                ),
                Err(_) => failed("Nothing is playing."),
            }
        }
        "dnd" => {
            let on = a["on"].as_bool().unwrap_or(true);
            let was = hub.env().dnd;
            match hub.shell_do("dnd", json!({"on": on})).await {
                Ok(_) => done(
                    if on {
                        "Do not disturb on"
                    } else {
                        "Do not disturb off"
                    },
                    (was != on).then(|| Call::new("dnd", json!({"on": was}))),
                ),
                Err(e) => failed(e),
            }
        }
        "game_mode" => game_mode(hub, a["on"].as_bool().unwrap_or(true), &hypr).await,
        "screenshot" => {
            let mode = match a["mode"].as_str().unwrap_or("region") {
                "window" => "activewindow",
                m => m,
            };
            match hub.shell_do("screenshot", json!({"mode": mode})).await {
                Ok(_) => done("Screenshot", None),
                Err(e) => failed(e),
            }
        }
        "lock" => match run_argv("loginctl", &["lock-session"], "").await {
            Ok(_) => done("Locked", None),
            Err(_) => failed("Could not lock the screen."),
        },
        "open" => {
            let what = a["what"].as_str().unwrap_or("");
            let page = a.get("page").and_then(Value::as_str).unwrap_or("");
            match hub
                .shell_do("open", json!({"what": what, "page": page}))
                .await
            {
                Ok(_) => done(open_line(what, page), None),
                Err(e) => failed(e),
            }
        }
        "web_search" | "play_music" | "open_url" => {
            let q = a.get("query").and_then(Value::as_str).unwrap_or("").trim();
            let (url, line) = match call.command.as_str() {
                "web_search" => (catalog::search_url(q), format!("Searching the web for {q}")),
                "play_music" => (
                    catalog::music_url(q),
                    format!("Looking up {q} in YouTube Music"),
                ),
                _ => (
                    a["url"].as_str().unwrap_or("").to_string(),
                    "Opened the page".to_string(),
                ),
            };
            match hub.shell_do("browse", json!({"url": url})).await {
                Ok(r) => {
                    // ewe-browser when it is installed, else the default browser
                    let via = r.get("via").and_then(Value::as_str).unwrap_or("");
                    done(
                        if via.is_empty() {
                            line
                        } else {
                            format!("{line} · {via}")
                        },
                        None,
                    )
                }
                Err(e) => failed(e),
            }
        }
        "launch_app" => {
            let app = a["app"].as_str().unwrap_or("").trim().to_string();
            let words = a
                .get("path")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .to_string();
            // "open kitty in documents", or the other way round: "open
            // documents in nemo"
            let (app, path) = if words.is_empty() {
                (app, None)
            } else if let Some(p) = places::resolve(&words) {
                (app, Some(p))
            } else if let Some(p) = places::resolve(&app) {
                (words, Some(p))
            } else {
                return missed(format!("I can't find a folder called {words}."));
            };
            let id = places::default_app(&app)
                .await
                .unwrap_or_else(|| app.clone());
            // a terminal starts in a folder, so a file gives its folder
            let dir = path.as_ref().map(|p| {
                if p.is_dir() {
                    p.clone()
                } else {
                    p.parent().unwrap_or(p).to_path_buf()
                }
            });
            let args = json!({
                "app": id,
                "path": path.as_ref().map(|p| p.to_string_lossy().to_string()),
                "dir": dir.as_ref().map(|p| p.to_string_lossy().to_string()),
            });
            match hub.shell_do("launch", args).await {
                Ok(r) => {
                    let name = r
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or(&app)
                        .to_string();
                    done(
                        match &path {
                            Some(p) => format!("Opened {name} in {}", places::show(p)),
                            None => format!("Opened {name}"),
                        },
                        None,
                    )
                }
                Err(e) if e == NOT_FOUND => missed(format!("There is no app called {app} here.")),
                Err(e) => failed(e),
            }
        }
        "open_folder" => {
            let words = a["path"].as_str().unwrap_or("").trim();
            let Some(p) = places::resolve(words) else {
                return missed(format!("I can't find a folder called {words}."));
            };
            // a file shows in its folder; the file manager is the one
            // Settings → Default apps chose, else Nemo (Super+E)
            let dir = if p.is_dir() {
                p.clone()
            } else {
                p.parent().unwrap_or(&p).to_path_buf()
            };
            let fm = places::default_app("file manager")
                .await
                .unwrap_or_else(|| "nemo".into());
            let d = dir.to_string_lossy().to_string();
            match hub
                .shell_do("launch", json!({"app": fm, "path": d, "dir": d}))
                .await
            {
                Ok(_) => done(format!("Opened {}", places::show(&dir)), None),
                Err(e) if e == NOT_FOUND => failed("There is no file manager here."),
                Err(e) => failed(e),
            }
        }
        // never from here: a proposal waits for its Run click (chat.rs)
        "run_shell" => failed("A shell command runs from its Run button in the chat."),
        other => failed(format!("No command called {other}.")),
    }
}

/// Game mode: Do not disturb, Hyprland's animations and blur off (at
/// runtime only; `hyprctl reload` puts your config back), and Feral
/// GameMode's CPU boost held while it is on.
async fn game_mode(hub: &Arc<Hub>, on: bool, hypr: &str) -> Outcome {
    if on == hub.game_mode_on() {
        return done(
            if on {
                "Game mode is already on"
            } else {
                "Game mode is already off"
            },
            None,
        );
    }
    if on {
        let was_dnd = hub.env().dnd;
        let _ = hub.shell_do("dnd", json!({"on": true})).await;
        let quiet = r#"hl.config({ animations = { enabled = false }, decoration = { blur = { enabled = false } } })"#;
        let hypr_ok = run_argv("hyprctl", &["eval", quiet], hypr).await.is_ok();
        let boost = Command::new("gamemoded")
            .arg("-r")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .ok();
        let mut parts = vec!["notifications quiet"];
        if hypr_ok {
            parts.push("no animations or blur");
        }
        if boost.is_some() {
            parts.push("CPU boost");
        }
        hub.set_game_mode(Some((was_dnd, boost)));
        done(
            format!("Game mode on · {}", parts.join(", ")),
            Some(Call::new("game_mode", json!({"on": false}))),
        )
    } else {
        let was_dnd = hub.set_game_mode(None);
        let _ = run_argv("hyprctl", &["reload", "config-only"], hypr).await;
        let _ = hub.shell_do("dnd", json!({"on": was_dnd})).await;
        done("Game mode off", None)
    }
}
