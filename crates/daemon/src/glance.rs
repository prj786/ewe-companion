//! The system glance: the few facts the home panel shows only when they
//! need you (the owner, 2026-10-14: "if memory is okay I don't care").
//! The daemon reports the numbers; the island decides what is worth a line.
//!
//! - disk: `statvfs` on `/` and the home directory (one row when they are
//!   the same file system), used as `df` counts it;
//! - updates: `~/.local/state/ewe/updates.json` (`{repo, aur}`), written by
//!   the shell's updates checker.

use std::ffi::CString;
use std::path::PathBuf;

use serde_json::{Value, json};

struct Disk {
    path: String,
    fsid: u64,
    used_pct: u64,
    free_bytes: u64,
}

fn statvfs(path: &str) -> Option<Disk> {
    let c = CString::new(path).ok()?;
    // SAFETY: statvfs writes one struct statvfs into `st` for a valid path.
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c.as_ptr(), &mut st) } != 0 {
        return None;
    }
    let frsize = st.f_frsize as u64;
    let used = (st.f_blocks as u64).saturating_sub(st.f_bfree as u64);
    let avail = st.f_bavail as u64;
    let total = used + avail;
    if total == 0 {
        return None;
    }
    Some(Disk {
        path: path.to_string(),
        fsid: st.f_fsid as u64,
        used_pct: (used * 100).div_ceil(total),
        free_bytes: avail * frsize,
    })
}

fn updates() -> Value {
    let file: PathBuf = companion_core::paths::state_dir()
        .parent()
        .map(|p| p.join("updates.json"))
        .unwrap_or_default();
    let v: Value = std::fs::read_to_string(file)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Value::Null);
    let n = |k: &str| v.get(k).and_then(Value::as_u64).unwrap_or(0);
    json!({"repo": n("repo"), "aur": n("aur"), "total": n("repo") + n("aur")})
}

pub fn read() -> Value {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
    let mut disks: Vec<Disk> = Vec::new();
    for p in ["/", home.as_str()] {
        if let Some(d) = statvfs(p)
            && !disks.iter().any(|x| x.fsid == d.fsid)
        {
            disks.push(d);
        }
    }
    let disks: Vec<Value> = disks
        .iter()
        .map(|d| json!({"path": if d.path == home { "~" } else { d.path.as_str() }, "used_pct": d.used_pct, "free_gb": d.free_bytes / 1_000_000_000}))
        .collect();
    json!({"disks": disks, "updates": updates()})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_root_disk_reads() {
        let d = statvfs("/").unwrap();
        assert!(d.used_pct <= 100);
        assert!(!read()["disks"].as_array().unwrap().is_empty());
    }
}
