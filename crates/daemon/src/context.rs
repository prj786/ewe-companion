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
        ("ewe-settings", "ewe Settings"),
        ("komble", "Komble"),
        ("ewe-sync", "ewe Sync"),
        ("ewe-browser", "ewe Browser"),
        ("ewe-safe", "ewe Safe (passwords)"),
        ("ewe-castd", "ewe Cast"),
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

const DAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// "Sunday 8 November 2026, 18:42 (UTC+04:00, Asia/Tbilisi)": a model
/// knows no date of its own, and without this it guesses.
pub fn now_line() -> Option<String> {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs() as libc::time_t;
    // SAFETY: tm is plain data; localtime_r writes only into it.
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    if unsafe { libc::localtime_r(&t, &mut tm) }.is_null() {
        return None;
    }
    let off = tm.tm_gmtoff / 60;
    let zone = std::fs::read_link("/etc/localtime")
        .ok()
        .and_then(|p| {
            let s = p.to_string_lossy().to_string();
            s.split_once("zoneinfo/").map(|(_, z)| z.to_string())
        })
        .map(|z| format!(", {z}"))
        .unwrap_or_default();
    Some(format!(
        "{} {} {} {}, {:02}:{:02} (UTC{}{:02}:{:02}{zone})",
        DAYS.get(tm.tm_wday as usize)?,
        tm.tm_mday,
        MONTHS.get(tm.tm_mon as usize)?,
        tm.tm_year + 1900,
        tm.tm_hour,
        tm.tm_min,
        if off < 0 { '-' } else { '+' },
        off.abs() / 60,
        off.abs() % 60,
    ))
}

/// "1 day 4 h", "35 min": how long since the computer started.
fn uptime() -> Option<String> {
    let s: f64 = read_trim("/proc/uptime")?
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;
    let m = (s / 60.0) as u64;
    let (d, h, m) = (m / 1440, m % 1440 / 60, m % 60);
    Some(match (d, h) {
        (0, 0) => format!("{m} min"),
        (0, _) => format!("{h} h {m} min"),
        (1, _) => format!("1 day {h} h"),
        _ => format!("{d} days {h} h"),
    })
}

/// "9.1 of 15.5 GB in use": `MemTotal` less `MemAvailable`.
fn memory() -> Option<String> {
    let info = std::fs::read_to_string("/proc/meminfo").ok()?;
    let kb = |key: &str| -> Option<f64> {
        info.lines()
            .find(|l| l.starts_with(key))?
            .split_whitespace()
            .nth(1)?
            .parse()
            .ok()
    };
    let (total, avail) = (kb("MemTotal:")?, kb("MemAvailable:")?);
    let gb = |k: f64| k / 1024.0 / 1024.0;
    Some(format!(
        "{:.1} of {:.1} GB in use",
        gb(total - avail),
        gb(total)
    ))
}

/// "1.20 over the last minute, 8 cores".
fn load() -> Option<String> {
    let one = read_trim("/proc/loadavg")?
        .split_whitespace()
        .next()?
        .to_string();
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
    Some(format!("{one} over the last minute, {cores} cores"))
}

/// Online or not, from NetworkManager (`full`, `limited`, `portal`, `none`).
async fn online() -> Option<String> {
    let out = first_line("nmcli", &["-t", "-f", "CONNECTIVITY", "general"]).await?;
    Some(
        match out.trim() {
            "full" => "online",
            "limited" => "connected, but no internet",
            "portal" => "behind a sign-in page (captive portal)",
            "none" => "offline",
            _ => return None,
        }
        .to_string(),
    )
}

/// A quick command's first line (not JSON).
async fn first_line(name: &str, args: &[&str]) -> Option<String> {
    let mut c = Command::new(tool(name)?);
    c.args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let out = tokio::time::timeout(BUDGET, c.output()).await.ok()?.ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    Some(s.lines().next()?.trim().to_string())
}

/// The live part: what changes from one question to the next.
async fn live(hub: &Arc<Hub>) -> String {
    let mut lines = Vec::new();
    if let Some(now) = now_line() {
        lines.push(format!("- now: {now}"));
    }
    if let Some(up) = uptime() {
        lines.push(format!("- up for {up}"));
    }
    if let Some(m) = memory() {
        lines.push(format!("- memory: {m}"));
    }
    if let Some(l) = load() {
        lines.push(format!("- CPU load: {l}"));
    }
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
    let (vol, light, net) = tokio::join!(commands::volume(), commands::brightness(), online());
    if let Some(n) = net {
        lines.push(format!("- network: {n}"));
    }
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

/// The installed part, from the last minute's reading or read now.
async fn fixed() -> String {
    let fresh = kept().lock().ok().and_then(|k| {
        k.as_ref()
            .filter(|(at, _)| at.elapsed() < KEEP)
            .map(|(_, s)| s.clone())
    });
    match fresh {
        Some(s) => s,
        None => {
            let s = installed().await;
            if let Ok(mut k) = kept().lock() {
                *k = Some((Instant::now(), s.clone()));
            }
            s
        }
    }
}

/// Read the installed part ahead of a question (the chat field got
/// focus): its tools take a moment, and a question should not wait.
pub async fn prefetch() {
    fixed().await;
}

/// This computer now, as a model reads it.
pub async fn snapshot(hub: &Arc<Hub>) -> String {
    format!("{}\n{}", fixed().await, live(hub).await)
}
