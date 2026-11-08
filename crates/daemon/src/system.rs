//! This computer now, read-only and offline (T0): the battery, memory, the
//! CPU and the apps using it, how long it has been on, the network, free
//! disk space, all of it at once ("why is my laptop slow"), and what takes
//! the disk's space.
//!
//! Sources: `/sys/class/power_supply`, `/proc/meminfo`, `/proc/loadavg`,
//! `/proc/stat`, `/proc/uptime`, `/proc/[pid]/{stat,statm,comm,exe}` (never
//! `cmdline`: it can hold file names and secrets), `statvfs`, `nmcli`, `ip
//! -j addr`, `journalctl --disk-usage`.
//!
//! Apps are grouped by program: the basename of `/proc/[pid]/exe` (so
//! Firefox's content processes count as firefox), or the process name
//! (`comm`) when the program is an interpreter or unreadable. Chrome's many
//! processes are one "chrome".
//!
//! Sizes: disks count in 1000s (like the disk on the box and the file
//! manager: 1 GB = 1000 MB), memory in 1024s (like the RAM on the box).
//!
//! What goes to a model (`Outcome::facts`): numbers and program names.
//! The network's name, local addresses and Downloads' file names are the
//! user's: rows only, with a `private` line.

use std::collections::HashMap;
use std::ffi::CString;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use companion_core::commands::{self as catalog, Call};
use serde_json::{Value, json};

use crate::commands::{Outcome, done, run_argv_within};
use crate::hub::Hub;

const GIB: f64 = 1_073_741_824.0;
/// How long the CPU is watched for "what's using my cpu".
const SAMPLE: Duration = Duration::from_millis(350);

// ── sizes ───────────────────────────────────────────────────────────────────

/// A disk size, in 1000s: "4.5 GB", "120 MB".
pub fn size(bytes: u64) -> String {
    let b = bytes as f64;
    match bytes {
        0..1_000 => format!("{bytes} B"),
        1_000..1_000_000 => format!("{:.0} KB", b / 1e3),
        1_000_000..1_000_000_000 => format!("{:.0} MB", b / 1e6),
        _ if b < 1e11 => format!("{:.1} GB", b / 1e9),
        _ if b < 1e12 => format!("{:.0} GB", b / 1e9),
        _ => format!("{:.2} TB", b / 1e12),
    }
}

/// A memory size, in 1024s: "7.8 GB", "512 MB".
fn mem_size(bytes: u64) -> String {
    let b = bytes as f64;
    if b >= GIB {
        format!("{:.1} GB", b / GIB)
    } else {
        format!("{:.0} MB", b / 1_048_576.0)
    }
}

/// "3 h 10 min", "45 min", "2 days 4 h".
fn span_min(min: u64) -> String {
    let (d, h, m) = (min / 1440, min % 1440 / 60, min % 60);
    match (d, h, m) {
        (0, 0, m) => format!("{m} min"),
        (0, h, 0) => format!("{h} h"),
        (0, h, m) => format!("{h} h {m} min"),
        (1, 0, _) => "1 day".into(),
        (1, h, _) => format!("1 day {h} h"),
        (d, 0, _) => format!("{d} days"),
        (d, h, _) => format!("{d} days {h} h"),
    }
}

fn read_trim(p: &Path) -> String {
    std::fs::read_to_string(p)
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn read_num(p: &Path) -> Option<f64> {
    read_trim(p).parse().ok()
}

// ── battery ─────────────────────────────────────────────────────────────────

/// One battery: its percent, status, and now/full/rate in one unit family
/// (µWh and µW, or µAh and µA): the times are ratios, so either works.
#[derive(Debug, Clone, Default, PartialEq)]
struct Battery {
    capacity: Option<f64>,
    status: String,
    now: Option<f64>,
    full: Option<f64>,
    rate: Option<f64>,
}

/// The computer's own batteries under `root` (`/sys/class/power_supply`):
/// not a mouse's or a headset's (`scope` Device).
fn batteries(root: &Path) -> Vec<Battery> {
    let Ok(rd) = std::fs::read_dir(root) else {
        return vec![];
    };
    let mut out = Vec::new();
    let mut dirs: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
    dirs.sort();
    for d in dirs {
        if read_trim(&d.join("type")) != "Battery" || read_trim(&d.join("scope")) == "Device" {
            continue;
        }
        let energy = read_num(&d.join("energy_now")).is_some();
        let (now, full, rate) = if energy {
            ("energy_now", "energy_full", "power_now")
        } else {
            ("charge_now", "charge_full", "current_now")
        };
        out.push(Battery {
            capacity: read_num(&d.join("capacity")),
            status: read_trim(&d.join("status")),
            now: read_num(&d.join(now)),
            full: read_num(&d.join(full)),
            rate: read_num(&d.join(rate)).map(f64::abs).filter(|r| *r > 0.0),
        });
    }
    out
}

/// "Battery 54 %, on battery · about 3 h 10 min left". None without one.
fn battery_line(bats: &[Battery]) -> Option<String> {
    if bats.is_empty() {
        return None;
    }
    let sum =
        |f: fn(&Battery) -> Option<f64>| -> Option<f64> { bats.iter().map(f).sum::<Option<f64>>() };
    let (now, full, rate) = (sum(|b| b.now), sum(|b| b.full), sum(|b| b.rate));
    let pct = match (now, full) {
        (Some(n), Some(f)) if f > 0.0 => (n / f * 100.0).clamp(0.0, 100.0),
        _ => {
            let caps: Vec<f64> = bats.iter().filter_map(|b| b.capacity).collect();
            if caps.is_empty() {
                return Some("There is a battery, but it does not say how full it is.".into());
            }
            caps.iter().sum::<f64>() / caps.len() as f64
        }
    };
    let pct = pct.round() as u64;
    let has = |s: &str| bats.iter().any(|b| b.status == s);
    let minutes = |num: Option<f64>| -> Option<u64> {
        let m = num? / rate? * 60.0;
        (m.is_finite() && m > 0.0 && m < 60.0 * 48.0).then_some(m.round() as u64)
    };
    Some(if has("Charging") {
        match minutes(full.zip(now).map(|(f, n)| f - n)) {
            Some(m) => format!("Battery {pct} %, charging · full in about {}", span_min(m)),
            None => format!("Battery {pct} %, charging"),
        }
    } else if has("Discharging") {
        match minutes(now) {
            Some(m) => format!("Battery {pct} %, on battery · about {} left", span_min(m)),
            None => format!("Battery {pct} %, on battery"),
        }
    } else if bats.iter().all(|b| b.status == "Full") {
        format!("Battery {pct} %, full and plugged in")
    } else if has("Not charging") {
        format!("Battery {pct} %, plugged in but not charging (a charge limit may hold it there)")
    } else {
        format!("Battery {pct} %")
    })
}

fn battery() -> Outcome {
    match battery_line(&batteries(Path::new("/sys/class/power_supply"))) {
        Some(l) => done(l, None).line_is_facts(),
        None => done(
            "This computer has no battery: it runs on mains power.",
            None,
        )
        .line_is_facts(),
    }
}

// ── memory ──────────────────────────────────────────────────────────────────

/// `/proc/meminfo`'s numbers, in bytes.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Mem {
    total: u64,
    available: u64,
    swap_total: u64,
    swap_free: u64,
}

fn meminfo(text: &str) -> Option<Mem> {
    let mut m = Mem::default();
    let mut seen = 0;
    for l in text.lines() {
        let mut it = l.split_whitespace();
        let (Some(k), Some(v)) = (it.next(), it.next()) else {
            continue;
        };
        let Ok(kb) = v.parse::<u64>() else {
            continue;
        };
        let b = kb * 1024;
        match k {
            "MemTotal:" => {
                m.total = b;
                seen += 1
            }
            "MemAvailable:" => {
                m.available = b;
                seen += 1
            }
            "SwapTotal:" => m.swap_total = b,
            "SwapFree:" => m.swap_free = b,
            _ => {}
        }
    }
    (seen == 2 && m.total > 0).then_some(m)
}

/// "6.2 of 15.5 GB in use, 9.3 GB free · swap 0.2 of 8.0 GB".
fn mem_line(m: &Mem) -> String {
    let used = m.total.saturating_sub(m.available);
    let mut s = format!(
        "Memory: {} of {} in use, {} free",
        mem_size(used).trim_end_matches(" GB"),
        mem_size(m.total),
        mem_size(m.available)
    );
    // "512 MB of 15.5 GB": the unit only once when both are GB
    if !mem_size(used).ends_with(" GB") {
        s = format!(
            "Memory: {} of {} in use, {} free",
            mem_size(used),
            mem_size(m.total),
            mem_size(m.available)
        );
    }
    let swap_used = m.swap_total.saturating_sub(m.swap_free);
    if m.swap_total > 0 && swap_used > 0 {
        s.push_str(&format!(
            " · swap {} of {}",
            mem_size(swap_used),
            mem_size(m.swap_total)
        ));
    }
    s
}

fn read_mem() -> Option<Mem> {
    meminfo(&std::fs::read_to_string("/proc/meminfo").ok()?)
}

// ── processes ───────────────────────────────────────────────────────────────

/// One process: its program, resident memory and CPU time so far.
#[derive(Debug, Clone)]
struct Proc {
    pid: u32,
    app: String,
    rss: u64,
    ticks: u64,
}

/// `/proc/[pid]/stat`: (comm, utime + stime). The name sits in brackets
/// and may hold spaces and brackets itself, so the fields start after the
/// last `)`.
fn parse_stat(s: &str) -> Option<(String, u64)> {
    let open = s.find('(')?;
    let close = s.rfind(')')?;
    let comm = s.get(open + 1..close)?.to_string();
    let rest: Vec<&str> = s.get(close + 1..)?.split_whitespace().collect();
    // rest[0] is field 3 (state): utime is field 14, stime 15
    let utime: u64 = rest.get(11)?.parse().ok()?;
    let stime: u64 = rest.get(12)?.parse().ok()?;
    Some((comm, utime + stime))
}

/// Programs that run other programs: their processes go by their own
/// name (`comm`), not the interpreter's.
fn generic(exe: &str) -> bool {
    const RUNTIMES: &[&str] = &[
        "python",
        "python3",
        "node",
        "bun",
        "deno",
        "java",
        "bash",
        "sh",
        "zsh",
        "fish",
        "dash",
        "perl",
        "ruby",
        "lua",
        "luajit",
        "mono",
        "dotnet",
        "gjs",
        "electron",
        "bwrap",
        "wine",
        "wine64",
        "wine-preloader",
        "wine64-preloader",
        "env",
        "qemu-system-x86_64",
    ];
    RUNTIMES.contains(&exe)
        || exe.starts_with("python3.")
        || exe.starts_with("electron")
        || exe.starts_with("ld-linux")
}

/// The program a process belongs to: kworker/u16:2 → kworker.
fn app_of(pid: u32, comm: &str) -> String {
    let comm = comm.split('/').next().unwrap_or(comm).trim();
    let exe = std::fs::read_link(format!("/proc/{pid}/exe"))
        .ok()
        .and_then(|p| {
            p.file_name().map(|n| {
                n.to_string_lossy()
                    .trim_end_matches(" (deleted)")
                    .to_string()
            })
        })
        .filter(|e| !e.is_empty() && !generic(e));
    match exe {
        Some(e) => e,
        None if comm.is_empty() => "unknown".into(),
        None => comm.to_string(),
    }
}

fn page_size() -> u64 {
    // SAFETY: sysconf only reads a constant.
    let p = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    if p > 0 { p as u64 } else { 4096 }
}

/// Every process now (blocking: a few hundred small reads).
fn procs() -> Vec<Proc> {
    let page = page_size();
    let Ok(rd) = std::fs::read_dir("/proc") else {
        return vec![];
    };
    let mut out = Vec::new();
    for e in rd.flatten() {
        let Some(pid) = e.file_name().to_str().and_then(|n| n.parse::<u32>().ok()) else {
            continue;
        };
        let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) else {
            continue;
        };
        let Some((comm, ticks)) = parse_stat(&stat) else {
            continue;
        };
        let rss = std::fs::read_to_string(format!("/proc/{pid}/statm"))
            .ok()
            .and_then(|s| s.split_whitespace().nth(1)?.parse::<u64>().ok())
            .unwrap_or(0)
            * page;
        out.push(Proc {
            pid,
            app: app_of(pid, &comm),
            rss,
            ticks,
        });
    }
    out
}

/// Programs by resident memory: (name, bytes, processes), biggest first.
fn top_memory(ps: &[Proc], n: usize) -> Vec<(String, u64, usize)> {
    let mut by: HashMap<&str, (u64, usize)> = HashMap::new();
    for p in ps.iter().filter(|p| p.rss > 0) {
        let e = by.entry(p.app.as_str()).or_default();
        e.0 += p.rss;
        e.1 += 1;
    }
    let mut v: Vec<(String, u64, usize)> = by
        .into_iter()
        .map(|(k, (b, c))| (k.to_string(), b, c))
        .collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    v.truncate(n);
    v
}

/// `/proc/stat`'s first line: (busy, total) jiffies.
fn cpu_total() -> Option<(u64, u64)> {
    let s = std::fs::read_to_string("/proc/stat").ok()?;
    let nums: Vec<u64> = s
        .lines()
        .next()?
        .split_whitespace()
        .skip(1)
        .take(8)
        .filter_map(|x| x.parse().ok())
        .collect();
    if nums.len() < 5 {
        return None;
    }
    let total: u64 = nums.iter().sum();
    let idle = nums[3] + nums[4];
    Some((total - idle, total))
}

/// What the CPU did over [`SAMPLE`]: how busy, and the programs by
/// percent of one core (100 = one core all the time).
struct CpuSample {
    busy: Option<f64>,
    cores: usize,
    load: Option<f64>,
    apps: Vec<(String, f64, usize)>,
    /// the processes at the end, for their memory
    procs: Vec<Proc>,
}

async fn cpu_sample() -> CpuSample {
    let look = || (procs(), cpu_total(), Instant::now());
    let empty = |_| (vec![], None, Instant::now());
    let first = tokio::task::spawn_blocking(look)
        .await
        .unwrap_or_else(empty);
    tokio::time::sleep(SAMPLE).await;
    let second = tokio::task::spawn_blocking(look)
        .await
        .unwrap_or_else(empty);
    let dt = second.2.duration_since(first.2).as_secs_f64().max(0.05);
    // SAFETY: sysconf only reads a constant.
    let tck = unsafe { libc::sysconf(libc::_SC_CLK_TCK) }.max(1) as f64;
    let cores = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) }.max(1) as usize;
    let before: HashMap<u32, (&str, u64)> = first
        .0
        .iter()
        .map(|p| (p.pid, (p.app.as_str(), p.ticks)))
        .collect();
    let mut by: HashMap<String, (f64, usize)> = HashMap::new();
    let me = std::process::id();
    // Shaun's own look at /proc is not what keeps the computer busy
    for p in second.0.iter().filter(|p| p.pid != me) {
        let Some((app, t0)) = before.get(&p.pid) else {
            continue;
        };
        if *app != p.app || p.ticks < *t0 {
            continue;
        }
        let pct = (p.ticks - t0) as f64 / tck / dt * 100.0;
        let e = by.entry(p.app.clone()).or_default();
        e.0 += pct;
        e.1 += 1;
    }
    let mut apps: Vec<(String, f64, usize)> = by
        .into_iter()
        .filter(|(_, (pct, _))| *pct >= 0.5)
        .map(|(k, (pct, n))| (k, pct, n))
        .collect();
    apps.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    apps.truncate(5);
    let busy = match (first.1, second.1) {
        (Some((b0, t0)), Some((b1, t1))) if t1 > t0 => {
            Some(((b1 - b0) as f64 / (t1 - t0) as f64 * 100.0).clamp(0.0, 100.0))
        }
        _ => None,
    };
    let load = std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse().ok());
    CpuSample {
        busy,
        cores,
        load,
        apps,
        procs: second.0,
    }
}

fn cpu_line(s: &CpuSample) -> String {
    let mut parts = Vec::new();
    match s.busy {
        Some(b) => parts.push(format!(
            "The processor is {:.0} % busy ({} cores)",
            b, s.cores
        )),
        None => parts.push(format!("{} cores", s.cores)),
    }
    if let Some(l) = s.load {
        parts.push(format!("load {l:.2}"));
    }
    let mut line = parts.join(" · ");
    match s.apps.first() {
        Some((app, pct, _)) => {
            line.push_str(&format!(". {app} uses the most: {pct:.0} % of one core"))
        }
        None => line.push_str(". Nothing stands out: the apps are idle"),
    }
    line
}

/// A program's row's Quit: `force_quit`, a proposal (`pkill -x`) that runs
/// only from its Run click. None for a name the catalog would refuse.
fn quit_actions(app: &str) -> Value {
    let call = Call::new("force_quit", json!({"app": app, "force": false}));
    match catalog::validate(&call) {
        Ok(()) => json!([{"label": "Quit", "command": call.command, "args": call.args}]),
        Err(_) => json!([]),
    }
}

fn cpu_rows(s: &CpuSample) -> Vec<Value> {
    s.apps
        .iter()
        .map(|(app, pct, n)| {
            json!({"title": app, "icon": "cpu", "actions": quit_actions(app),
                   "detail": format!("{pct:.0} % of one core{}", if *n > 1 { format!(" · {n} processes") } else { String::new() })})
        })
        .collect()
}

fn mem_rows(top: &[(String, u64, usize)]) -> Vec<Value> {
    top.iter()
        .map(|(app, b, n)| {
            json!({"title": app, "icon": "memory", "actions": quit_actions(app),
                   "detail": format!("{}{}", mem_size(*b), if *n > 1 { format!(" · {n} processes") } else { String::new() })})
        })
        .collect()
}

fn apps_facts(top: &[(String, u64, usize)]) -> String {
    top.iter()
        .map(|(a, b, _)| format!("{a} {}", mem_size(*b)))
        .collect::<Vec<_>>()
        .join(", ")
}

async fn memory(apps: bool) -> Outcome {
    let Some(m) = read_mem() else {
        return done("I could not read the memory figures (/proc/meminfo).", None);
    };
    let ps = tokio::task::spawn_blocking(procs).await.unwrap_or_default();
    let top = top_memory(&ps, 5);
    let summary = mem_line(&m);
    let line = match (apps, top.first()) {
        (true, Some((app, b, _))) => {
            format!("{app} uses the most memory: {}. {summary}", mem_size(*b))
        }
        _ => summary,
    };
    let facts = if top.is_empty() {
        line.clone()
    } else {
        format!("{line}. The programs using the most: {}", apps_facts(&top))
    };
    Outcome {
        rows: mem_rows(&top),
        ..done(line, None)
    }
    .facts(facts)
}

async fn cpu() -> Outcome {
    let s = cpu_sample().await;
    let line = cpu_line(&s);
    let facts = if s.apps.is_empty() {
        line.clone()
    } else {
        format!(
            "{line}. The programs using it: {}",
            s.apps
                .iter()
                .map(|(a, p, _)| format!("{a} {p:.0} %"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    Outcome {
        rows: cpu_rows(&s),
        ..done(line, None)
    }
    .facts(facts)
}

// ── uptime ──────────────────────────────────────────────────────────────────

fn local_tm(epoch_s: i64) -> Option<libc::tm> {
    let t = epoch_s as libc::time_t;
    // SAFETY: tm is plain data; localtime_r writes only into it.
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    let r = unsafe { libc::localtime_r(&t, &mut tm) };
    (!r.is_null()).then_some(tm)
}

/// "since 09:12 today", "since yesterday 22:40", "since Saturday 14:12",
/// "since 1 November 14:12".
fn since_words(start: i64, now: i64) -> String {
    let (Some(a), Some(b)) = (local_tm(start), local_tm(now)) else {
        return String::new();
    };
    let day = |tm: &libc::tm| {
        companion_core::dates::days_from_civil(
            tm.tm_year as i64 + 1900,
            tm.tm_mon as u32 + 1,
            tm.tm_mday as u32,
        )
    };
    let hm = format!("{:02}:{:02}", a.tm_hour, a.tm_min);
    match day(&b) - day(&a) {
        0 => format!("since {hm} today"),
        1 => format!("since yesterday {hm}"),
        2..=6 => format!(
            "since {} {hm}",
            companion_core::dates::WEEKDAYS[((a.tm_wday + 6) % 7) as usize]
        ),
        _ => format!(
            "since {} {} {hm}",
            a.tm_mday,
            companion_core::dates::MONTHS[a.tm_mon as usize]
        ),
    }
}

fn uptime_line(up_s: f64, now: i64) -> String {
    let up = up_s.max(0.0) as i64;
    format!(
        "Up {}, {}",
        span_min((up / 60) as u64),
        since_words(now - up, now)
    )
}

fn uptime() -> Outcome {
    let up = std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse::<f64>().ok());
    match up {
        Some(u) => done(uptime_line(u, companion_core::id::now_s()), None).line_is_facts(),
        None => done("I could not read how long this computer has been on.", None),
    }
}

// ── the network ─────────────────────────────────────────────────────────────

/// One `nmcli -t` line's fields: `:` separates them, `\:` is a colon
/// inside one.
fn terse_fields(line: &str) -> Vec<String> {
    let mut out = vec![String::new()];
    let mut esc = false;
    for c in line.chars() {
        match (esc, c) {
            (true, c) => {
                out.last_mut().unwrap().push(c);
                esc = false;
            }
            (false, '\\') => esc = true,
            (false, ':') => out.push(String::new()),
            (false, c) => out.last_mut().unwrap().push(c),
        }
    }
    out
}

/// What the network is: (connectivity, the kinds of link up, their
/// connection names, a VPN on).
#[derive(Debug, Default, PartialEq)]
struct Net {
    connectivity: Option<String>,
    links: Vec<&'static str>,
    names: Vec<(String, String)>,
    vpn: bool,
}

fn net_of(connectivity: Option<&str>, devices: Option<&str>) -> Net {
    let mut n = Net {
        connectivity: connectivity.map(|c| c.trim().to_lowercase()),
        ..Default::default()
    };
    for l in devices.unwrap_or("").lines() {
        let f = terse_fields(l);
        let (Some(kind), Some(state)) = (f.first(), f.get(1)) else {
            continue;
        };
        if !state.starts_with("connected") || state.contains("externally") {
            continue;
        }
        let label = match kind.as_str() {
            "wifi" => "Wi-Fi",
            "ethernet" => "Ethernet",
            "gsm" | "cdma" | "modem" => "mobile data",
            "vpn" | "wireguard" | "tun" => {
                n.vpn = true;
                "VPN"
            }
            _ => continue,
        };
        if label != "VPN" && !n.links.contains(&label) {
            n.links.push(label);
        }
        if let Some(name) = f.get(2).filter(|s| !s.is_empty()) {
            n.names.push((label.to_string(), name.clone()));
        }
    }
    n
}

/// "You're online, over Wi-Fi, with a VPN on." and its model-safe facts.
fn net_line(n: &Net) -> (String, String) {
    let how = match n.links.as_slice() {
        [] => String::new(),
        [one] => format!(", over {one}"),
        many => format!(", over {}", many.join(" and ")),
    };
    let vpn = if n.vpn { ", with a VPN on" } else { "" };
    let (line, state) = match n.connectivity.as_deref() {
        Some("full") => (format!("You're online{how}{vpn}."), "online"),
        Some("limited") => (
            format!("Connected{how}{vpn}, but the internet is not reachable."),
            "connected, internet not reachable",
        ),
        Some("portal") => (
            format!(
                "Connected{how}, but a sign-in page is in the way: open any website to sign in."
            ),
            "behind a sign-in page (captive portal)",
        ),
        Some("none") => ("You're offline.".to_string(), "offline"),
        _ if n.links.is_empty() => (
            "NetworkManager is not answering, so I can't tell whether you're online.".to_string(),
            "unknown",
        ),
        _ => (
            format!(
                "Connected{how}{vpn}; whether the internet is reachable, NetworkManager does not say."
            ),
            "connected, internet unknown",
        ),
    };
    let facts = format!(
        "Network: {state}{}{}",
        if n.links.is_empty() {
            String::new()
        } else {
            format!(" over {}", n.links.join(" and "))
        },
        if n.vpn { "; a VPN is on" } else { "" }
    );
    (line, facts)
}

/// `ip -j addr`'s global addresses: (interface, address, family).
fn addresses(json_text: &str) -> Vec<(String, String, &'static str)> {
    let v: Value = serde_json::from_str(json_text).unwrap_or(Value::Null);
    let mut out = Vec::new();
    for i in v.as_array().into_iter().flatten() {
        let ifname = i["ifname"].as_str().unwrap_or("");
        if ifname == "lo" {
            continue;
        }
        for a in i["addr_info"].as_array().into_iter().flatten() {
            if a["scope"].as_str() != Some("global") {
                continue;
            }
            let fam = if a["family"] == "inet6" {
                "IPv6"
            } else {
                "IPv4"
            };
            if let Some(local) = a["local"].as_str() {
                out.push((ifname.to_string(), local.to_string(), fam));
            }
        }
    }
    out
}

async fn network() -> Outcome {
    let budget = Duration::from_secs(3);
    let (conn, devs, ip) = tokio::join!(
        run_argv_within(
            "nmcli",
            &["-t", "-f", "CONNECTIVITY", "general"],
            "",
            budget
        ),
        run_argv_within(
            "nmcli",
            &["-t", "-f", "TYPE,STATE,CONNECTION", "device"],
            "",
            budget
        ),
        run_argv_within("ip", &["-j", "addr"], "", budget),
    );
    let net = net_of(conn.ok().as_deref(), devs.ok().as_deref());
    let addrs = addresses(ip.as_deref().unwrap_or("[]"));
    let (mut line, facts) = net_line(&net);
    if !addrs.is_empty() {
        line.push_str(
            " Your local addresses are below; your public address would need asking a website, so I don't look it up.",
        );
    }
    let mut rows: Vec<Value> = net
        .names
        .iter()
        .map(|(kind, name)| json!({"title": kind, "detail": name, "icon": "network"}))
        .collect();
    rows.extend(addrs.iter().map(|(ifname, a, fam)| {
        json!({"title": a, "detail": format!("{fam} · {ifname}"), "icon": "network",
               "actions": [{"label": "Copy", "copy": a}]})
    }));
    let facts = format!(
        "{facts}; {} local address{} shown on the island; the public address is not looked up",
        addrs.len(),
        if addrs.len() == 1 { "" } else { "es" }
    );
    Outcome {
        rows,
        ..done(line, None)
    }
    .private(facts.clone())
    .facts(facts)
}

// ── disks ───────────────────────────────────────────────────────────────────

/// (where, used %, free bytes, total bytes) for `/` and the home folder,
/// once per file system.
fn disks() -> Vec<(String, u64, u64, u64)> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
    let mut seen = Vec::new();
    let mut out = Vec::new();
    for p in ["/", home.as_str()] {
        let Ok(c) = CString::new(p) else { continue };
        // SAFETY: statvfs writes one struct statvfs into `st` for a valid path.
        let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
        if unsafe { libc::statvfs(c.as_ptr(), &mut st) } != 0 {
            continue;
        }
        let fsid = st.f_fsid as u64;
        if seen.contains(&fsid) {
            continue;
        }
        seen.push(fsid);
        let fr = st.f_frsize as u64;
        let used = (st.f_blocks as u64).saturating_sub(st.f_bfree as u64) * fr;
        let free = st.f_bavail as u64 * fr;
        let total = used + free;
        if total == 0 {
            continue;
        }
        let label = if p == "/" {
            "/".to_string()
        } else {
            "~".to_string()
        };
        out.push((label, (used * 100).div_ceil(total), free, total));
    }
    out
}

fn disk_line(ds: &[(String, u64, u64, u64)]) -> String {
    match ds {
        [] => "I could not read the disks.".into(),
        [(_, pct, free, total)] => format!(
            "The disk is {pct} % full: {} free of {}",
            size(*free),
            size(*total)
        ),
        many => many
            .iter()
            .map(|(p, pct, free, total)| {
                format!(
                    "{p} is {pct} % full ({} free of {})",
                    size(*free),
                    size(*total)
                )
            })
            .collect::<Vec<_>>()
            .join(" · "),
    }
}

fn disk() -> Outcome {
    let ds = disks();
    let mut o = done(disk_line(&ds), None).line_is_facts();
    if ds.iter().any(|d| d.1 >= 85) {
        o.action = Some((
            "What takes space".into(),
            companion_core::commands::Call::new("disk_usage", json!({})),
        ));
    }
    o
}

// ── all of it: "why is my laptop slow" ─────────────────────────────────────

async fn overview() -> Outcome {
    let s = cpu_sample().await;
    let mem = read_mem();
    let ds = disks();
    let bat = battery_line(&batteries(Path::new("/sys/class/power_supply")));
    let top_mem = top_memory(&s.procs, 3);
    let mut worries: Vec<String> = Vec::new();
    if let (Some(b), Some((app, pct, _))) = (s.busy, s.apps.first())
        && (b >= 80.0 || *pct >= 90.0)
    {
        worries.push(format!(
            "the processor is busy ({b:.0} %): {app} uses the most, {pct:.0} % of one core"
        ));
    }
    if let Some(l) = s.load
        && l > s.cores as f64 * 1.5
        && worries.is_empty()
    {
        worries.push(format!("the load is high ({l:.1} on {} cores)", s.cores));
    }
    if let Some(m) = mem
        && m.available * 10 < m.total
    {
        let who = top_mem
            .first()
            .map(|(a, b, _)| format!(": {a} uses the most, {}", mem_size(*b)))
            .unwrap_or_default();
        worries.push(format!(
            "memory is nearly full ({} free){who}",
            mem_size(m.available)
        ));
    }
    if let Some(m) = mem {
        let swap_used = m.swap_total.saturating_sub(m.swap_free);
        if m.swap_total > 0 && swap_used * 2 > m.swap_total {
            worries.push(format!("swap is in heavy use ({})", mem_size(swap_used)));
        }
    }
    for (p, pct, free, _) in &ds {
        if *pct >= 90 {
            worries.push(format!(
                "the disk {p} is {pct} % full ({} free)",
                size(*free)
            ));
        }
    }
    if let Some(b) = &bat
        && b.contains("on battery")
        && let Some(pct) = b
            .strip_prefix("Battery ")
            .and_then(|r| r.split(' ').next())
            .and_then(|n| n.parse::<u64>().ok())
        && pct <= 15
    {
        worries.push(format!(
            "the battery is low ({pct} %): a laptop may slow down to save it"
        ));
    }
    let verdict = match worries.as_slice() {
        [] => {
            "Nothing looks wrong: the processor has room, memory has room and the disk has space."
                .to_string()
        }
        [one] => format!("{}{}.", one[..1].to_uppercase(), &one[1..]),
        many => {
            let s = many.join("; ");
            format!("{}{}.", s[..1].to_uppercase(), &s[1..])
        }
    };
    let mut rows = vec![json!({"title": "Processor", "detail": cpu_line(&s), "icon": "cpu"})];
    if let Some(m) = &mem {
        rows.push(json!({"title": "Memory", "detail": mem_line(m).trim_start_matches("Memory: ").to_string(), "icon": "memory"}));
    }
    if !ds.is_empty() {
        rows.push(json!({"title": "Disk", "detail": disk_line(&ds), "icon": "disk"}));
    }
    if let Some(b) = &bat {
        rows.push(json!({"title": "Battery", "detail": b, "icon": "battery"}));
    }
    let mut facts = vec![verdict.clone(), cpu_line(&s)];
    if let Some(m) = &mem {
        facts.push(mem_line(m));
    }
    if !top_mem.is_empty() {
        facts.push(format!("most memory: {}", apps_facts(&top_mem)));
    }
    facts.push(disk_line(&ds));
    if let Some(b) = bat {
        facts.push(b);
    }
    Outcome {
        rows,
        ..done(verdict, None)
    }
    .facts(facts.join(". "))
}

/// `system_info {what, apps?}`.
pub async fn info(_hub: &Arc<Hub>, a: &Value) -> Outcome {
    let apps = a.get("apps").and_then(Value::as_bool).unwrap_or(false);
    match a["what"].as_str().unwrap_or("") {
        "battery" => battery(),
        "memory" => memory(apps).await,
        "cpu" => cpu().await,
        "uptime" => uptime(),
        "network" => network().await,
        "disk" => disk(),
        _ => overview().await,
    }
}

// ── what takes the disk's space ─────────────────────────────────────────────

/// The whole look at the disk, at most.
const SCAN_BUDGET: Duration = Duration::from_secs(5);
/// How long a look is kept.
const SCAN_KEEP: Duration = Duration::from_secs(600);

#[derive(Clone)]
struct Usage {
    at: Instant,
    /// (what, its key, bytes, whether the look finished)
    cats: Vec<(&'static str, &'static str, u64, bool)>,
    /// Downloads' biggest files: (path, bytes)
    downloads: Vec<(PathBuf, u64)>,
}

fn kept() -> &'static Mutex<Option<Usage>> {
    static K: Mutex<Option<Usage>> = Mutex::new(None);
    &K
}

/// The bytes under `root` (files' own sizes, links not followed, like `du
/// -sb`), and whether the walk finished before `deadline`. With `biggest`,
/// the largest files too.
fn dir_size(
    root: &Path,
    deadline: Instant,
    biggest: Option<&mut Vec<(PathBuf, u64)>>,
) -> (u64, bool) {
    let mut total = 0u64;
    let mut stack = vec![root.to_path_buf()];
    let mut files: Vec<(PathBuf, u64)> = Vec::new();
    let keep = biggest.is_some();
    let mut n = 0u32;
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            n += 1;
            if n.is_multiple_of(256) && Instant::now() > deadline {
                if let Some(b) = biggest {
                    files.sort_by_key(|f| std::cmp::Reverse(f.1));
                    files.truncate(10);
                    *b = files;
                }
                return (total, false);
            }
            let Ok(m) = e.metadata() else { continue };
            if m.is_dir() {
                stack.push(e.path());
            } else if m.is_file() {
                total += m.len();
                if keep {
                    files.push((e.path(), m.len()));
                    if files.len() > 200 {
                        files.sort_by_key(|f| std::cmp::Reverse(f.1));
                        files.truncate(10);
                    }
                }
            }
        }
    }
    if let Some(b) = biggest {
        files.sort_by_key(|f| std::cmp::Reverse(f.1));
        files.truncate(10);
        *b = files;
    }
    (total, true)
}

/// `journalctl --disk-usage`: "… take up 1.2G in the file system." (1024s).
fn journal_bytes(out: &str) -> Option<u64> {
    let w = out.split("take up ").nth(1)?.split_whitespace().next()?;
    let (num, unit) = w.split_at(w.find(|c: char| c.is_ascii_alphabetic()).unwrap_or(w.len()));
    let n: f64 = num.parse().ok()?;
    let mult = match unit.chars().next() {
        None | Some('B') => 1.0,
        Some('K') => 1024.0,
        Some('M') => 1_048_576.0,
        Some('G') => GIB,
        Some('T') => GIB * 1024.0,
        _ => return None,
    };
    Some((n * mult) as u64)
}

fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var_os("HOME").unwrap_or_else(|| "/".into()))
}

fn xdg(var: &str, under_home: &str) -> PathBuf {
    std::env::var_os(var)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| home_dir().join(under_home))
}

async fn scan() -> Usage {
    let deadline = Instant::now() + SCAN_BUDGET;
    let journal = run_argv_within("journalctl", &["--disk-usage"], "", Duration::from_secs(2))
        .await
        .ok()
        .and_then(|o| journal_bytes(&o));
    let downloads_dir = crate::places::resolve("downloads");
    tokio::task::spawn_blocking(move || {
        let mut cats = Vec::new();
        let mut downloads = Vec::new();
        let dirs: [(&'static str, &'static str, Option<PathBuf>); 5] = [
            ("Downloads", "downloads", downloads_dir),
            (
                "the package cache",
                "pacman",
                Some(PathBuf::from("/var/cache/pacman/pkg")),
            ),
            ("~/.cache", "cache", Some(xdg("XDG_CACHE_HOME", ".cache"))),
            (
                "the trash",
                "trash",
                Some(xdg("XDG_DATA_HOME", ".local/share").join("Trash")),
            ),
            (
                "crash dumps",
                "coredump",
                Some(PathBuf::from("/var/lib/systemd/coredump")),
            ),
        ];
        for (label, key, dir) in dirs {
            let Some(dir) = dir.filter(|d| d.is_dir()) else {
                continue;
            };
            let (bytes, whole) = if key == "downloads" {
                dir_size(&dir, deadline, Some(&mut downloads))
            } else {
                dir_size(&dir, deadline, None)
            };
            cats.push((label, key, bytes, whole));
        }
        if let Some(j) = journal {
            cats.push(("the system journal", "journal", j, true));
        }
        cats.sort_by_key(|c| std::cmp::Reverse(c.2));
        Usage {
            at: Instant::now(),
            cats,
            downloads,
        }
    })
    .await
    .unwrap_or_else(|_| Usage {
        at: Instant::now(),
        cats: vec![],
        downloads: vec![],
    })
}

fn usage_outcome(u: &Usage) -> Outcome {
    let amount = |b: u64, whole: bool| {
        if whole {
            size(b)
        } else {
            format!("at least {}", size(b))
        }
    };
    let list: Vec<String> = u
        .cats
        .iter()
        .map(|(label, _, b, whole)| format!("{label} {}", amount(*b, *whole)))
        .collect();
    if list.is_empty() {
        return done("I could not look at the disk.", None);
    }
    let age = u.at.elapsed().as_secs() / 60;
    let mut line = format!("What takes space, biggest first: {}.", list.join(", "));
    if age >= 1 {
        line.push_str(&format!(" (As I looked {age} min ago.)"));
    }
    let mut rows: Vec<Value> = u
        .cats
        .iter()
        .map(|(label, key, b, whole)| {
            let (hint, actions) = match *key {
                "downloads" => (
                    "your downloads; the biggest files are below",
                    json!([{"label": "Open", "command": "open_folder", "args": {"path": "downloads"}}]),
                ),
                "pacman" => (
                    "old packages: sudo paccache -rk1 keeps one old version",
                    json!([]),
                ),
                "cache" => (
                    "apps' caches: they rebuild them",
                    json!([{"label": "Open", "command": "open_folder", "args": {"path": "~/.cache"}}]),
                ),
                "trash" => (
                    "what you deleted",
                    json!([{"label": "Empty", "command": "empty_trash", "args": {}}]),
                ),
                "journal" => ("the system's logs: journalctl --vacuum-time=2weeks trims them", json!([])),
                _ => ("from programs that crashed: coredumpctl lists them", json!([])),
            };
            let mut s = label.to_string();
            if let Some(f) = s.get_mut(..1) {
                f.make_ascii_uppercase();
            }
            json!({"title": s, "detail": format!("{} · {hint}", amount(*b, *whole)),
                   "icon": "disk", "actions": actions})
        })
        .collect();
    rows.extend(u.downloads.iter().take(5).map(|(p, b)| {
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        json!({"title": name, "detail": format!("{} · in Downloads", size(*b)), "icon": "file",
               "actions": [{"label": "Open", "command": "open_file", "args": {"path": p.to_string_lossy()}},
                           {"label": "Show in folder", "command": "open_folder",
                            "args": {"path": p.parent().map(|d| d.to_string_lossy().to_string()).unwrap_or_default()}}]})
    }));
    // the categories are no one's names; Downloads' files are, so they
    // stay in the rows
    let facts = format!("Disk use, biggest first: {}", list.join(", "));
    Outcome {
        rows,
        ..done(line, None)
    }
    .private(facts.clone())
    .facts(facts)
}

/// `disk_usage {}`: what takes space, looked at once per 10 minutes.
pub async fn disk_usage() -> Outcome {
    let fresh = kept()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clone()
        .filter(|u| u.at.elapsed() < SCAN_KEEP);
    let u = match fresh {
        Some(u) => u,
        None => {
            let u = scan().await;
            *kept().lock().unwrap_or_else(|p| p.into_inner()) = Some(u.clone());
            u
        }
    };
    usage_outcome(&u)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "companion-system-{name}-{}-{}",
            std::process::id(),
            companion_core::id::new_id("")
        ));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn put(dir: &Path, sub: &str, files: &[(&str, &str)]) {
        let d = dir.join(sub);
        std::fs::create_dir_all(&d).unwrap();
        for (k, v) in files {
            std::fs::write(d.join(k), v).unwrap();
        }
    }

    #[test]
    fn sizes_read_like_the_box_says() {
        assert_eq!(size(512), "512 B");
        assert_eq!(size(4_500_000_000), "4.5 GB");
        assert_eq!(size(120_000_000), "120 MB");
        assert_eq!(mem_size(16 * 1_073_741_824), "16.0 GB");
        assert_eq!(mem_size(512 * 1_048_576), "512 MB");
        assert_eq!(span_min(190), "3 h 10 min");
        assert_eq!(span_min(45), "45 min");
        assert_eq!(span_min(1440 + 240), "1 day 4 h");
    }

    #[test]
    fn batteries_say_how_full_and_how_long() {
        let root = scratch("bat");
        // no battery
        assert_eq!(battery_line(&batteries(&root)), None);
        // a mouse's battery is not the computer's
        put(
            &root,
            "hidpp_battery_0",
            &[("type", "Battery"), ("scope", "Device"), ("capacity", "40")],
        );
        assert_eq!(battery_line(&batteries(&root)), None);
        put(
            &root,
            "BAT0",
            &[
                ("type", "Battery"),
                ("status", "Discharging"),
                ("capacity", "54"),
                ("energy_now", "27000000"),
                ("energy_full", "50000000"),
                ("power_now", "9000000"),
            ],
        );
        assert_eq!(
            battery_line(&batteries(&root)).as_deref(),
            Some("Battery 54 %, on battery · about 3 h left")
        );
        put(
            &root,
            "BAT0",
            &[("status", "Charging"), ("power_now", "23000000")],
        );
        assert_eq!(
            battery_line(&batteries(&root)).as_deref(),
            Some("Battery 54 %, charging · full in about 1 h")
        );
        put(
            &root,
            "BAT0",
            &[("status", "Not charging"), ("power_now", "0")],
        );
        assert!(
            battery_line(&batteries(&root))
                .unwrap()
                .contains("not charging")
        );
        // charge instead of energy
        put(
            &root,
            "BAT0",
            &[
                ("status", "Discharging"),
                ("energy_now", ""),
                ("charge_now", "2000000"),
                ("charge_full", "4000000"),
                ("current_now", "1000000"),
            ],
        );
        std::fs::remove_file(root.join("BAT0/energy_now")).unwrap();
        assert_eq!(
            battery_line(&batteries(&root)).as_deref(),
            Some("Battery 50 %, on battery · about 2 h left")
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn meminfo_reads() {
        let m = meminfo(
            "MemTotal:       16000000 kB\nMemFree:  100 kB\nMemAvailable:    9000000 kB\nSwapTotal: 8388608 kB\nSwapFree: 8000000 kB\n",
        )
        .unwrap();
        assert_eq!(m.total, 16_000_000 * 1024);
        assert_eq!(
            mem_line(&m),
            "Memory: 6.7 of 15.3 GB in use, 8.6 GB free · swap 380 MB of 8.0 GB"
        );
        assert_eq!(meminfo("nonsense"), None);
    }

    #[test]
    fn stat_lines_with_odd_names_parse() {
        let s = "1234 (Web Content (x)) S 1 2 3 4 5 6 7 8 9 10 250 50 0 0 20 0 1 0 100 0 0";
        assert_eq!(parse_stat(s), Some(("Web Content (x)".to_string(), 300)));
        assert_eq!(parse_stat("garbage"), None);
        let ps = vec![
            Proc {
                pid: 1,
                app: "chrome".into(),
                rss: 100,
                ticks: 0,
            },
            Proc {
                pid: 2,
                app: "chrome".into(),
                rss: 300,
                ticks: 0,
            },
            Proc {
                pid: 3,
                app: "zed".into(),
                rss: 350,
                ticks: 0,
            },
            Proc {
                pid: 4,
                app: "kworker".into(),
                rss: 0,
                ticks: 0,
            },
        ];
        assert_eq!(
            top_memory(&ps, 5),
            vec![("chrome".to_string(), 400, 2), ("zed".to_string(), 350, 1)]
        );
        assert!(generic("python3.13") && generic("electron34") && !generic("firefox"));
        // this very process has a program
        assert!(!app_of(std::process::id(), "x").is_empty());
    }

    #[test]
    fn the_network_says_how_without_names() {
        assert_eq!(
            terse_fields(r"wifi:connected:Home\:Net"),
            vec!["wifi", "connected", "Home:Net"]
        );
        let n = net_of(
            Some("full\n"),
            Some(
                "wifi:connected:HomeNet-7Q2X\nethernet:unavailable:\nloopback:connected (externally):lo\nwireguard:connected:wg0\n",
            ),
        );
        assert_eq!(n.links, vec!["Wi-Fi"]);
        assert!(n.vpn);
        let (line, facts) = net_line(&n);
        assert_eq!(line, "You're online, over Wi-Fi, with a VPN on.");
        assert!(!facts.contains("HomeNet"), "{facts}");
        assert_eq!(
            net_line(&net_of(Some("none"), Some(""))).0,
            "You're offline."
        );
        assert!(
            net_line(&net_of(Some("portal"), Some("wifi:connected:Cafe")))
                .0
                .contains("sign-in page")
        );
        assert!(net_line(&net_of(None, None)).0.contains("not answering"));
        let a = addresses(
            r#"[{"ifname":"lo","addr_info":[{"family":"inet","local":"127.0.0.1","scope":"host"}]},
                {"ifname":"wlan0","addr_info":[{"family":"inet","local":"192.168.1.23","scope":"global"},
                                               {"family":"inet6","local":"fe80::1","scope":"link"}]}]"#,
        );
        assert_eq!(
            a,
            vec![("wlan0".to_string(), "192.168.1.23".to_string(), "IPv4")]
        );
    }

    #[test]
    fn the_journal_and_folders_have_sizes() {
        assert_eq!(
            journal_bytes("Archived and active journals take up 1.5G in the file system."),
            Some((1.5 * GIB) as u64)
        );
        assert_eq!(journal_bytes("No journal files were found."), None);
        let root = scratch("du");
        put(
            &root,
            "a",
            &[("big.iso", &"x".repeat(5000)), ("small.txt", "hi")],
        );
        put(&root, "a/b", &[("mid.bin", &"y".repeat(300))]);
        let mut top = Vec::new();
        let (bytes, whole) = dir_size(&root, Instant::now() + SCAN_BUDGET, Some(&mut top));
        assert_eq!((bytes, whole), (5302, true));
        assert_eq!(top[0].1, 5000);
        assert!(top[0].0.ends_with("big.iso"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn uptime_says_since_when() {
        let now = companion_core::id::now_s();
        let l = uptime_line(3600.0 * 28.0, now);
        assert!(l.starts_with("Up 1 day 4 h, since "), "{l}");
        assert!(uptime_line(600.0, now).starts_with("Up 10 min, since "));
    }
}
