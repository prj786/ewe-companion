//! Timers and reminders: "set a timer for 5 minutes", "remind me in 10
//! minutes to call mum", "remind me at 17:30 to stretch".
//!
//! Each one is a row in `companion.db` (migration 002), so it survives a
//! daemon restart; one that fell due while the daemon was down fires at
//! the next start and says when it was due. When one is due, the desktop's
//! notification server shows it (`notify-send` as Shaun, a banner for 30 s
//! instead of the usual 5, and kept in Quick settings → Notifications; Do
//! not disturb hides the banner like any other); without a server, the
//! island says it. Setting one is T1 (Undo cancels it); the list has Cancel
//! on each row.
//!
//! The text is the user's own words: it is kept in the store until the
//! timer fires, and never logged (ids and counts only).

use std::sync::Arc;
use std::time::Duration;

use companion_core::commands::{Call, clock};
use companion_core::id::new_id;
use companion_core::proto::{self, Peek};
use serde_json::{Value, json};
use tracing::info;

use crate::commands::{Outcome, done, failed, run_argv};
use crate::hub::Hub;
use crate::store::Timer;

/// The longest the scheduler sleeps between looks at the store.
const NAP: Duration = Duration::from_secs(60);

/// Now, in epoch milliseconds (signed: dues are compared and subtracted).
fn now_ms() -> i64 {
    companion_core::id::now_ms() as i64
}

// ── local time (libc; the daemon has no date crate) ─────────────────────────

fn local_tm(epoch_s: i64) -> Option<libc::tm> {
    let t = epoch_s as libc::time_t;
    // SAFETY: tm is plain data; localtime_r writes only into it.
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    let r = unsafe { libc::localtime_r(&t, &mut tm) };
    (!r.is_null()).then_some(tm)
}

/// `17:35`, in this computer's time zone.
pub fn clock_of_ms(ms: i64) -> String {
    match local_tm(ms / 1000) {
        Some(tm) => format!("{:02}:{:02}", tm.tm_hour, tm.tm_min),
        None => "?".into(),
    }
}

/// The next moment the clock shows `h:m` (today if it is still ahead,
/// else tomorrow), in epoch milliseconds.
fn next_at(now: i64, h: u32, m: u32) -> Option<i64> {
    let at_day = |plus_days: i32| -> Option<i64> {
        let mut tm = local_tm(now / 1000)?;
        tm.tm_mday += plus_days;
        tm.tm_hour = h as i32;
        tm.tm_min = m as i32;
        tm.tm_sec = 0;
        tm.tm_isdst = -1;
        // SAFETY: mktime normalises the fields it was given in place.
        let t = unsafe { libc::mktime(&mut tm) };
        (t != -1).then_some(t as i64 * 1000)
    };
    let today = at_day(0)?;
    if today > now { Some(today) } else { at_day(1) }
}

/// "5 minutes", "1 hour 30 minutes", "45 seconds".
pub fn span(secs: i64) -> String {
    let unit = |n: i64, one: &str| {
        if n == 1 {
            format!("1 {one}")
        } else {
            format!("{n} {one}s")
        }
    };
    let (h, m, s) = (secs / 3600, secs % 3600 / 60, secs % 60);
    let parts: Vec<String> = [(h, "hour"), (m, "minute"), (s, "second")]
        .into_iter()
        .filter(|(n, _)| *n > 0)
        .map(|(n, u)| unit(n, u))
        .collect();
    if parts.is_empty() {
        "0 seconds".into()
    } else {
        parts.join(" ")
    }
}

/// How long until `due`, roughly: "in 4 min", "in 1 h 20 min", "now".
fn left(due: i64, now: i64) -> String {
    let s = (due - now).max(0) / 1000;
    match s {
        0 => "now".into(),
        1..60 => format!("in {s} s"),
        60..3600 => format!("in {} min", (s + 59) / 60),
        _ => format!("in {} h {} min", s / 3600, s % 3600 / 60),
    }
}

/// What a timer says: its title and its line.
fn words(t: &Timer) -> (&'static str, String) {
    match (t.text.trim(), t.seconds, t.reminder) {
        ("", Some(s), false) => (
            "Timer",
            format!("{} are up.", span(s))
                .replace("1 minute are", "1 minute is")
                .replace("1 hour are", "1 hour is")
                .replace("1 second are", "1 second is"),
        ),
        // "remind me in 5 minutes", "remind me at 5pm": no words of its own
        ("", _, _) => (
            "Reminder",
            format!("You asked to be reminded at {}.", clock_of_ms(t.due)),
        ),
        (text, _, _) => ("Reminder", text.to_string()),
    }
}

/// The most timers and reminders that wait at once.
pub const MAX_LIVE: usize = 50;

// ── the commands ────────────────────────────────────────────────────────────

/// `timer {seconds | at, text?}`: kept, then the Undo cancels it.
pub fn set(hub: &Arc<Hub>, a: &Value) -> Outcome {
    let now = now_ms();
    let text = a
        .get("text")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let (due, seconds) = match (
        a.get("seconds").and_then(Value::as_i64),
        a.get("at").and_then(Value::as_str),
    ) {
        (Some(s), _) => (now + s * 1000, Some(s)),
        (None, Some(at)) => match clock(at).and_then(|(h, m)| next_at(now, h, m)) {
            Some(due) => (due, None),
            None => return failed("That is not a time I can set."),
        },
        _ => return failed("A timer needs a length or a time."),
    };
    if hub.store().timers().len() >= MAX_LIVE {
        return failed(format!(
            "{MAX_LIVE} timers and reminders are waiting already: cancel one first (\"show my timers\")."
        ));
    }
    let reminder = a.get("reminder").and_then(Value::as_bool).unwrap_or(false) || seconds.is_none();
    let t = Timer {
        id: new_id("t"),
        due,
        created: now,
        seconds,
        reminder,
        text: text.clone(),
    };
    if !hub.store().timer_add(&t) {
        return failed("I could not keep the timer.");
    }
    hub.timers.notify_one();
    info!(id = %t.id, at_time = seconds.is_none(), with_text = !text.is_empty(), "timer set");
    let at = clock_of_ms(due);
    let line = match (seconds, text.is_empty()) {
        (Some(s), true) if !reminder => format!("Timer set for {} · until {at}", span(s)),
        (Some(s), true) => format!("I'll remind you in {} ({at})", span(s)),
        (Some(s), false) => format!("I'll remind you in {} ({at}): {text}", span(s)),
        (None, true) => format!("I'll remind you at {at}"),
        (None, false) => format!("I'll remind you at {at}: {text}"),
    };
    done(line, Some(Call::new("timer_cancel", json!({"id": t.id}))))
}

fn row(t: &Timer, now: i64) -> Value {
    let (title, _) = words(t);
    let label = match (t.text.trim(), t.seconds) {
        ("", Some(s)) => format!("{title} · {}", span(s)),
        ("", None) => title.to_string(),
        (text, _) => text.to_string(),
    };
    json!({"title": label, "detail": format!("at {} · {}", clock_of_ms(t.due), left(t.due, now)),
           "icon": "timer",
           "actions": [{"label": "Cancel", "command": "timer_cancel", "args": {"id": t.id}}]})
}

/// `timers {}`: what waits, soonest first, with Cancel.
pub fn list(hub: &Arc<Hub>) -> Outcome {
    let all = hub.store().timers();
    let now = now_ms();
    let n = all.len();
    Outcome {
        rows: all.iter().map(|t| row(t, now)).collect(),
        ..done(
            match n {
                0 => "No timers or reminders are waiting.".to_string(),
                1 => "One is waiting:".to_string(),
                n => format!("{n} are waiting:"),
            },
            None,
        )
    }
}

/// `timer_cancel {id?}`: that one; with no id, the only one (several
/// become rows to pick from).
pub fn cancel(hub: &Arc<Hub>, id: Option<&str>) -> Outcome {
    let all = hub.store().timers();
    let pick = match id {
        Some(id) => all.into_iter().find(|t| t.id == id),
        None => match all.len() {
            0 => return done("No timers or reminders are waiting.", None),
            1 => all.into_iter().next(),
            n => {
                let now = now_ms();
                return Outcome {
                    rows: all.iter().map(|t| row(t, now)).collect(),
                    ..done(format!("{n} are waiting. Which one?"), None)
                };
            }
        },
    };
    let Some(t) = pick else {
        return failed("That timer is not waiting any more.");
    };
    hub.store().timer_remove(&t.id);
    hub.timers.notify_one();
    info!(id = %t.id, "timer cancelled");
    let (title, _) = words(&t);
    done(
        match t.text.trim() {
            "" => format!(
                "Cancelled the {} ({})",
                title.to_lowercase(),
                clock_of_ms(t.due)
            ),
            text => format!("Cancelled the reminder: {text}"),
        },
        None,
    )
}

// ── firing ──────────────────────────────────────────────────────────────────

/// What the notification says; one that fell due while the daemon was
/// down (more than a minute ago) says when it was due.
fn notice(t: &Timer, now: i64) -> (&'static str, String) {
    let (title, mut body) = words(t);
    if now - t.due > 60_000 {
        body.push_str(&format!(" (due at {})", clock_of_ms(t.due)));
    }
    (title, body)
}

/// One notification for several at once (a backlog after the daemon was
/// down, or many due together): never a storm of banners.
fn summary(due: &[Timer], now: i64) -> (&'static str, String) {
    let names: Vec<String> = due
        .iter()
        .take(3)
        .map(|t| match t.text.trim() {
            "" => words(t).0.to_lowercase(),
            text => text.to_string(),
        })
        .collect();
    let more = due.len().saturating_sub(names.len());
    let mut body = format!("{} are due: {}", due.len(), names.join("; "));
    if more > 0 {
        body.push_str(&format!(" and {more} more"));
    }
    body.push('.');
    if let Some(first) = due.iter().map(|t| t.due).min().filter(|d| now - d > 60_000) {
        body.push_str(&format!(" (since {})", clock_of_ms(first)));
    }
    ("Timers and reminders", body)
}

/// Say it: the desktop's notification server (`notify-send`, as Shaun),
/// else a line on the island. Neither path stores it: the user's words
/// never reach the events table or a log.
async fn notify(hub: &Arc<Hub>, title: &str, body: &str, key: &str) -> bool {
    let shown = run_argv(
        "notify-send",
        &[
            "--app-name=Shaun",
            "--icon=alarm-symbolic",
            "--urgency=normal",
            "--expire-time=30000",
            "--",
            title,
            body,
        ],
        "",
    )
    .await
    .is_ok();
    if !shown {
        let p = Peek {
            key: key.to_string(),
            line: format!("{title} · {body}"),
            level: "info".into(),
            actions: vec![],
            ttl_ms: 0,
        };
        hub.broadcast(proto::line(
            "peek",
            serde_json::to_value(&p).unwrap_or_default(),
        ));
    }
    shown
}

/// The scheduler: fire what is due, then sleep until the next one (at
/// most a minute; a new or cancelled timer wakes it).
pub async fn run(hub: Arc<Hub>) {
    loop {
        let now = now_ms();
        // gone first, so a slow notification never fires one twice. The
        // list is taken out of the store before each removal locks it again
        // (one statement holding the guard would deadlock the daemon)
        let pending = hub.store().timers_due(now);
        let due: Vec<Timer> = pending
            .into_iter()
            .filter(|t| hub.store().timer_remove(&t.id))
            .collect();
        match due.as_slice() {
            [] => {}
            [t] => {
                let (title, body) = notice(t, now);
                let shown = notify(&hub, title, &body, &format!("timer-{}", t.id)).await;
                info!(id = %t.id, notified = shown, late_s = (now - t.due) / 1000, "timer fired");
            }
            many => {
                let (title, body) = summary(many, now);
                let shown = notify(&hub, title, &body, "timers").await;
                info!(
                    count = many.len(),
                    notified = shown,
                    "timers fired together"
                );
            }
        }
        let next = hub.store().timers().first().map(|t| t.due);
        let wait = next
            .map(|d| Duration::from_millis((d - now_ms()).clamp(0, NAP.as_millis() as i64) as u64))
            .unwrap_or(NAP);
        tokio::select! {
            _ = tokio::time::sleep(wait) => {}
            _ = hub.timers.notified() => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_and_what_is_left() {
        assert_eq!(span(300), "5 minutes");
        assert_eq!(span(5400), "1 hour 30 minutes");
        assert_eq!(span(61), "1 minute 1 second");
        assert_eq!(left(10_000 + 240_000, 10_000), "in 4 min");
        assert_eq!(left(0, 5), "now");
    }

    #[test]
    fn a_time_of_day_is_the_next_one() {
        let now = now_ms();
        let tm = local_tm(now / 1000).unwrap();
        let (h, m) = (tm.tm_hour as u32, tm.tm_min as u32);
        // this very minute has begun: the next one is tomorrow
        let due = next_at(now, h, m).unwrap();
        assert!(due > now && due - now <= 25 * 3600 * 1000, "{}", due - now);
        assert_eq!(clock_of_ms(due), format!("{h:02}:{m:02}"));
    }

    #[test]
    fn a_timer_says_what_it_was() {
        let t = Timer {
            id: "t1".into(),
            due: 0,
            created: 0,
            seconds: Some(300),
            reminder: false,
            text: String::new(),
        };
        // "remind me in 5 minutes": a reminder, not a timer
        let bare = Timer {
            reminder: true,
            ..t.clone()
        };
        assert_eq!(
            words(&bare),
            (
                "Reminder",
                format!("You asked to be reminded at {}.", clock_of_ms(0))
            )
        );
        assert_eq!(words(&t), ("Timer", "5 minutes are up.".to_string()));
        assert_eq!(
            words(&Timer {
                seconds: Some(60),
                ..t.clone()
            })
            .1,
            "1 minute is up."
        );
        let late = Timer {
            text: "call mum".into(),
            ..t
        };
        assert_eq!(words(&late), ("Reminder", "call mum".to_string()));
        // on time: as it is; late (the daemon was down): when it was due
        assert_eq!(notice(&late, 30_000).1, "call mum");
        assert_eq!(
            notice(&late, 120_000).1,
            format!("call mum (due at {})", clock_of_ms(0))
        );
    }

    #[test]
    fn a_backlog_is_one_notification() {
        let t = |id: &str, text: &str| Timer {
            id: id.into(),
            due: 0,
            created: 0,
            seconds: Some(60),
            reminder: false,
            text: text.into(),
        };
        let due = [
            t("a", "call mum"),
            t("b", ""),
            t("c", "stretch"),
            t("d", "tea"),
            t("e", ""),
        ];
        let (title, body) = summary(&due, 600_000);
        assert_eq!(title, "Timers and reminders");
        assert_eq!(
            body,
            format!(
                "5 are due: call mum; timer; stretch and 2 more. (since {})",
                clock_of_ms(0)
            )
        );
    }
}
