//! This computer as it is now, for a model's question: what makes "knows
//! everything about ewe" true of *this* ewe, not only of ewe in general.
//!
//! Only what helps an answer, and nothing that names the user's accounts:
//! the versions, which ewe apps and plugins are here, the plugins' and the
//! user's own shortcuts, the look (scheme, accent, Glass), then the live
//! part (disks, waiting updates, battery, sound, Do not disturb, game mode,
//! the agents). Never ewe.conf as a whole (its account and mail sections
//! name servers and users), never a secret.
//!
//! Every source is a quick read-only command (ewe's tools print JSON) or a
//! file, run in parallel within 2 s; the installed part is kept a minute.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde_json::Value;
use tokio::process::Command;

use crate::commands;
use crate::hub::Hub;

const BUDGET: Duration = Duration::from_secs(2);
const KEEP: Duration = Duration::from_secs(60);

fn kept() -> &'static Mutex<Option<(Instant, String)>> {
    static K: OnceLock<Mutex<Option<(Instant, String)>>> = OnceLock::new();
    K.get_or_init(|| Mutex::new(None))
}

fn is_exec(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(p).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

/// A tool on PATH, else in ~/.local/bin: a git-clone ewe keeps its tools
/// there, and a user unit's PATH may not have it.
pub fn tool(name: &str) -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let p = dir.join(name);
            if is_exec(&p) {
                return Some(p);
            }
        }
    }
    let p = PathBuf::from(std::env::var_os("HOME")?)
        .join(".local/bin")
        .join(name);
    is_exec(&p).then_some(p)
}

async fn json_of(name: &str, args: &[&str]) -> Option<Value> {
    let mut c = Command::new(tool(name)?);
    c.args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let out = tokio::time::timeout(BUDGET, c.output()).await.ok()?.ok()?;
    serde_json::from_slice(&out.stdout).ok()
}

fn read_trim(p: &str) -> Option<String> {
    let s = std::fs::read_to_string(p).ok()?.trim().to_string();
    (!s.is_empty()).then_some(s)
}

/// The installed part: versions, apps, plugins, shortcuts, the look.
async fn installed() -> String {
    let (plugins, binds, theme) = tokio::join!(
        json_of("ewe-plugin", &["list", "--json"]),
        json_of("ewe-conf", &["get", "desktop.keybinds"]),
        json_of("ewe-conf", &["get", "desktop.theme"]),
    );
    let mut lines = Vec::new();

    let ewe = read_trim("/usr/share/ewe/VERSION");
    let kernel = read_trim("/proc/sys/kernel/osrelease");
    lines.push(format!(
        "- ewe {}, Linux {}",
        ewe.as_deref().unwrap_or("(version unknown)"),
        kernel.as_deref().unwrap_or("?")
    ));

    let apps = [
        ("ewe-settings", "Settings"),
        ("komble", "Komble"),
        ("ewe-sync", "ewe-sync"),
        ("ewe-browser", "ewe browser"),
        ("ewe-castd", "ewe-cast"),
    ];
    let (have, missing): (Vec<_>, Vec<_>) = apps.iter().partition(|(bin, _)| tool(bin).is_some());
    let names = |v: &[&(&str, &str)]| v.iter().map(|(_, n)| *n).collect::<Vec<_>>().join(", ");
    lines.push(format!("- apps installed: {}", names(&have)));
    if !missing.is_empty() {
        lines.push(format!("- not installed: {}", names(&missing)));
    }

    if let Some(list) = plugins.as_ref().and_then(|v| v["plugins"].as_array()) {
        let on: Vec<&Value> = list
            .iter()
            .filter(|p| {
                p["enabled"].as_bool() == Some(true)
                    && p["installed"].as_bool() == Some(true)
                    && p["valid"].as_bool() != Some(false)
            })
            .collect();
        let name = |p: &Value| {
            p["name"]
                .as_str()
                .or(p["id"].as_str())
                .unwrap_or("?")
                .to_string()
        };
        if on.is_empty() {
            lines.push("- plugins on: none".into());
        } else {
            lines.push(format!(
                "- plugins on: {}",
                on.iter().map(|p| name(p)).collect::<Vec<_>>().join(", ")
            ));
        }
        let keys: Vec<String> = on
            .iter()
            .flat_map(|p| {
                let n = name(p);
                p["keybinds"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(move |k| Some(format!("{} ({n})", k["combo"].as_str()?)))
            })
            .collect();
        if !keys.is_empty() {
            lines.push(format!("- plugin shortcuts: {}", keys.join(", ")));
        }
    }

    if let Some(list) = binds.as_ref().and_then(Value::as_array) {
        let own: Vec<String> = list
            .iter()
            .take(12)
            .filter_map(|b| {
                Some(format!(
                    "{} runs {}",
                    b["combo"].as_str()?,
                    b["exec"].as_str()?
                ))
            })
            .collect();
        if !own.is_empty() {
            lines.push(format!("- the user's own shortcuts: {}", own.join("; ")));
        }
    }

    if let Some(t) = theme.as_ref().filter(|t| t.is_object()) {
        let mut look = Vec::new();
        if let Some(s) = t["scheme"].as_str() {
            look.push(format!("scheme {s}"));
        }
        if let Some(a) = t["accent"].as_str() {
            look.push(format!("accent {a}"));
        }
        match t["bar_opacity"].as_i64() {
            Some(n) if n < 100 => look.push(format!("Glass on (bar opacity {n} %)")),
            Some(_) => look.push("Glass off".into()),
            None => {}
        }
        if !look.is_empty() {
            lines.push(format!("- the look: {}", look.join(", ")));
        }
    }
    lines.join("\n")
}

/// The first battery, if the machine has one: "64 %, discharging".
fn battery() -> Option<String> {
    for e in std::fs::read_dir("/sys/class/power_supply").ok()?.flatten() {
        let p = e.path();
        if read_trim(&p.join("type").to_string_lossy()).as_deref() != Some("Battery") {
            continue;
        }
        let cap = read_trim(&p.join("capacity").to_string_lossy())?;
        let status = read_trim(&p.join("status").to_string_lossy()).unwrap_or_default();
        return Some(format!("{cap} %, {}", status.to_lowercase()));
    }
    None
}

/// The live part: what changes from one question to the next.
async fn live(hub: &Arc<Hub>) -> String {
    let mut lines = Vec::new();
    let snap = hub.snapshot();
    for d in snap["glance"]["disks"]
        .as_array()
        .cloned()
        .unwrap_or_default()
    {
        lines.push(format!(
            "- disk {}: {} % used, {} GB free",
            d["path"].as_str().unwrap_or("?"),
            d["used_pct"],
            d["free_gb"]
        ));
    }
    let up = snap["glance"]["updates"]["total"].as_u64().unwrap_or(0);
    lines.push(match up {
        0 => "- no updates waiting".to_string(),
        n => format!("- {n} updates waiting (Komble → Updates)"),
    });
    if let Some(b) = battery() {
        lines.push(format!("- battery {b}"));
    }
    let (vol, light) = tokio::join!(commands::volume(), commands::brightness());
    if let Some((v, muted)) = vol {
        lines.push(format!(
            "- volume {v} %{}",
            if muted { ", muted" } else { "" }
        ));
    }
    if let Some(b) = light {
        lines.push(format!("- screen brightness {b} %"));
    }
    let env = hub.env();
    lines.push(format!(
        "- Do not disturb {}, game mode {}",
        if env.dnd { "on" } else { "off" },
        if hub.game_mode_on() { "on" } else { "off" }
    ));
    // the snapshot's sessions: the recent ones the island lists
    let agents: Vec<String> = snap["sessions"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter(|s| matches!(s["state"].as_str(), Some("working" | "waiting")))
        .map(|s| {
            let str_of = |k: &str| s[k].as_str().unwrap_or("").to_string();
            let last = str_of("last");
            let what = if last.is_empty() {
                String::new()
            } else {
                format!(": {last}")
            };
            format!(
                "{} in {} ({}{what})",
                str_of("agent"),
                str_of("project"),
                str_of("state")
            )
        })
        .collect();
    if !agents.is_empty() {
        lines.push(format!("- agents at work: {}", agents.join("; ")));
    }
    lines.join("\n")
}

/// This computer now, as a model reads it.
pub async fn snapshot(hub: &Arc<Hub>) -> String {
    let fresh = kept().lock().ok().and_then(|k| {
        k.as_ref()
            .filter(|(at, _)| at.elapsed() < KEEP)
            .map(|(_, s)| s.clone())
    });
    let fixed = match fresh {
        Some(s) => s,
        None => {
            let s = installed().await;
            if let Ok(mut k) = kept().lock() {
                *k = Some((Instant::now(), s.clone()));
            }
            s
        }
    };
    format!("{fixed}\n{}", live(hub).await)
}
