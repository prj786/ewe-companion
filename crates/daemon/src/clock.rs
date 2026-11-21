//! The clock and dates: "what time is it", "what time is it in tokyo",
//! "how many days until christmas", "what's the date in 30 days".
//!
//! Here: this computer's local time (libc's `localtime_r`, like the
//! timers). In a place: the zone `companion_core::dates::zone_for` picks
//! from `/usr/share/zoneinfo`'s names, and `date` run with `TZ=<zone>` in
//! the child's environment (argv, never a shell; the daemon's own `TZ` is
//! never touched). Date arithmetic is the core's, with today's local date
//! passed in. All T0, offline; the line is the facts (a time is no secret).

use std::path::Path;
use std::process::Stdio;
use std::sync::OnceLock;
use std::time::Duration;

use companion_core::dates::{self, MONTHS, WEEKDAYS};
use serde_json::Value;
use tokio::process::Command;

use crate::commands::{Outcome, done, failed, missed};

const ZONEINFO: &str = "/usr/share/zoneinfo";

/// Local time now.
fn local_now() -> Option<libc::tm> {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs() as libc::time_t;
    // SAFETY: tm is plain data; localtime_r writes only into it.
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    let r = unsafe { libc::localtime_r(&t, &mut tm) };
    (!r.is_null()).then_some(tm)
}

/// Today's local date as a day number (days since 1970-01-01).
pub fn today() -> Option<i64> {
    let tm = local_now()?;
    Some(dates::days_from_civil(
        tm.tm_year as i64 + 1900,
        tm.tm_mon as u32 + 1,
        tm.tm_mday as u32,
    ))
}

/// The zones this computer knows ("Asia/Tokyo", "UTC" …), read once.
fn zones() -> &'static Vec<String> {
    static Z: OnceLock<Vec<String>> = OnceLock::new();
    Z.get_or_init(|| {
        let mut out = Vec::new();
        walk(Path::new(ZONEINFO), "", 0, &mut out);
        out.sort();
        out
    })
}

fn walk(dir: &Path, prefix: &str, depth: usize, out: &mut Vec<String>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        // zone names start with a capital; the rest are tables and the
        // posix/right copies of the same zones
        if !name.starts_with(|c: char| c.is_ascii_uppercase())
            || name.contains('.')
            || matches!(name.as_str(), "Etc" | "SystemV" | "Factory")
        {
            continue;
        }
        let full = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        match e.file_type() {
            Ok(t) if t.is_dir() && depth < 3 => walk(&e.path(), &full, depth + 1, out),
            Ok(t) if t.is_file() || t.is_symlink() => out.push(full),
            _ => {}
        }
    }
}

/// "18:42, Sunday 8 November 2026".
fn local_line(tm: &libc::tm) -> String {
    format!(
        "{:02}:{:02}, {} {} {} {}",
        tm.tm_hour,
        tm.tm_min,
        WEEKDAYS[((tm.tm_wday + 6) % 7) as usize],
        tm.tm_mday,
        MONTHS[tm.tm_mon as usize],
        tm.tm_year + 1900
    )
}

/// `date` in a zone, as (HH:MM, weekday, day, month, UTC offset in
/// minutes). The zone goes to the child alone.
async fn date_in(zone: &str) -> Option<(String, String, String, String, i64)> {
    let mut c = Command::new("date");
    c.arg("+%H:%M|%A|%-d|%B|%z")
        .env("TZ", zone)
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let out = tokio::time::timeout(Duration::from_secs(2), c.output())
        .await
        .ok()?
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let p: Vec<&str> = s.split('|').collect();
    let [hm, wd, d, m, z] = p[..] else {
        return None;
    };
    Some((
        hm.into(),
        wd.into(),
        d.into(),
        m.into(),
        dates::utc_offset_min(z)?,
    ))
}

/// `clock {place?}`: the time and date here, or in a place.
pub async fn now(place: Option<&str>) -> Outcome {
    // the tm holds a pointer (its zone's name): read what is needed before
    // any await
    let Some((local, here)) = local_now().map(|tm| (local_line(&tm), tm.tm_gmtoff / 60)) else {
        return failed("I could not read the clock.");
    };
    let place = place.map(str::trim).filter(|p| !p.is_empty());
    let Some(place) = place else {
        return done(format!("It's {local}"), None).line_is_facts();
    };
    let Some(zone) = dates::zone_for(place, zones()) else {
        return missed(format!("I don't know a time zone for {place}."));
    };
    let Some((hm, wd, d, m, there)) = date_in(&zone).await else {
        return failed(format!("I could not read the time in {zone}."));
    };
    let city = if zone == "UTC" {
        "UTC".to_string()
    } else {
        dates::zone_city(&zone)
    };
    done(
        format!(
            "In {city} it's {hm}, {wd} {d} {m} ({zone}, {})",
            dates::offset_words(there, here)
        ),
        None,
    )
    .line_is_facts()
}

/// `date_math {op, date | days}`: days until or since, a weekday, a date
/// from today.
pub fn date_math(a: &Value) -> Outcome {
    let Some(today) = today() else {
        return failed("I could not read the clock.");
    };
    let op = a["op"].as_str().unwrap_or("");
    match dates::answer(
        op,
        a.get("date").and_then(Value::as_str),
        a.get("days").and_then(Value::as_i64),
        today,
    ) {
        Some(line) => done(line, None).line_is_facts(),
        None => missed("I can't tell which day that is."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn this_computer_knows_its_zones() {
        // read-only: /usr/share/zoneinfo on the build machine (tzdata)
        if !Path::new(ZONEINFO).join("Asia/Tokyo").exists() {
            return;
        }
        let z = zones();
        assert!(z.iter().any(|x| x == "Asia/Tokyo"));
        assert!(
            !z.iter()
                .any(|x| x.starts_with("posix/") || x.contains(".tab"))
        );
        assert_eq!(dates::zone_for("tokyo", z).as_deref(), Some("Asia/Tokyo"));
        assert_eq!(
            dates::zone_for("new york", z).as_deref(),
            Some("America/New_York")
        );
    }

    #[tokio::test]
    async fn the_time_here_and_in_a_place() {
        let o = now(None).await;
        assert!(
            o.ok && o.line.starts_with("It's ") && o.facts.is_some(),
            "{}",
            o.line
        );
        let o = now(Some("narnia")).await;
        assert!(o.miss);
        if Path::new(ZONEINFO).join("UTC").exists() {
            let o = now(Some("utc")).await;
            assert!(o.line.starts_with("In UTC it's "), "{}", o.line);
        }
    }

    #[test]
    fn date_math_uses_today() {
        let o = date_math(&serde_json::json!({"op": "offset", "days": 1}));
        assert!(o.ok && o.line.starts_with("Tomorrow is "), "{}", o.line);
        let o = date_math(&serde_json::json!({"op": "until", "date": "christmas"}));
        assert!(o.line.starts_with("Christmas is "), "{}", o.line);
    }
}
