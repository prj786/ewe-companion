//! The offline matcher: plain words → a [`Call`], with no model at all.
//!
//! Commands must work with no AI configured (C5), instantly and the same
//! way every time, so the common ones are matched here first: "set volume to
//! 60%", "brightness up", "next song", "game mode on", "search for …". Only
//! what this misses goes to a model (and only when one is configured).
//!
//! Matching is deliberately literal: whole-sentence patterns, case- and
//! punctuation-insensitive. A question ("what is the volume?") is not a
//! command and falls through.

use std::sync::OnceLock;

use regex::Regex;
use serde_json::json;

use crate::commands::Call;

/// How much "up" / "down" moves volume and brightness.
const STEP: i64 = 10;

fn clean(text: &str) -> String {
    let t = text
        .trim()
        .to_lowercase()
        .replace(['\u{2019}', '\u{2018}'], "'");
    let t = t.trim_end_matches(['.', '!', '?']).trim();
    let t = t.strip_prefix("please ").unwrap_or(t);
    let t = t.strip_suffix(" please").unwrap_or(t);
    t.split_whitespace().collect::<Vec<_>>().join(" ")
}

struct Rule {
    re: Regex,
    make: fn(&regex::Captures) -> Option<Call>,
}

fn num(c: &regex::Captures, i: usize) -> Option<i64> {
    c.get(i)?.as_str().parse().ok()
}

fn on_word(w: &str) -> Option<bool> {
    match w {
        "on" | "enable" | "start" | "enter" | "turn on" | "switch on" => Some(true),
        "off" | "disable" | "stop" | "exit" | "leave" | "quit" | "turn off" | "switch off" => {
            Some(false)
        }
        _ => None,
    }
}

/// What Safe copies, from the word people use for it.
fn safe_field(w: &str) -> &'static str {
    match w {
        "password" | "pass" => "password",
        w if w.contains("code") || w.contains("otp") || w == "2fa" => "otp",
        _ => "username",
    }
}

/// The site or app a Safe request names ("my bank" → "bank"); "the" or
/// "my" alone names nothing.
fn safe_query(q: &str) -> Option<String> {
    let mut q = q.trim();
    for pre in ["my ", "the ", "your "] {
        q = q.strip_prefix(pre).unwrap_or(q).trim();
    }
    for suf in [" website", " site", " account"] {
        q = q.strip_suffix(suf).unwrap_or(q).trim();
    }
    (!q.is_empty() && !matches!(q, "the" | "my" | "a" | "it" | "this" | "that"))
        .then(|| q.to_string())
}

fn safe_copy(query: &str, field: &str) -> Option<Call> {
    let q = safe_query(query)?;
    Some(Call::new(
        "safe_copy",
        json!({"query": q, "field": safe_field(field)}),
    ))
}

/// The site a search names ("on zara", "on h&m's website", "zara.com"):
/// short (one to three words, or a domain), and not a place on this
/// computer or the web as a whole, which keep their old meaning.
fn site_of(y: &str) -> Option<String> {
    let mut y = y.trim().trim_end_matches('\'').trim();
    y = y.strip_prefix("the ").unwrap_or(y).trim();
    for suf in [" website", " web site", " site", " app", " store", " shop"] {
        y = y.strip_suffix(suf).unwrap_or(y).trim();
    }
    y = y.strip_suffix("'s").unwrap_or(y).trim();
    if y.is_empty() || y.len() > 64 {
        return None;
    }
    let words: Vec<&str> = y.split_whitespace().collect();
    // "my desktop", "this computer", "a cold day": not a site
    if matches!(
        words[0],
        "my" | "your"
            | "our"
            | "this"
            | "that"
            | "these"
            | "a"
            | "an"
            | "it"
            | "here"
            | "there"
            | "for"
            | "in"
            | "at"
            | "to"
    ) {
        return None;
    }
    const NOT_SITES: &[&str] = &[
        "web",
        "internet",
        "online",
        "google",
        "line",
        "computer",
        "laptop",
        "pc",
        "machine",
        "system",
        "disk",
        "drive",
        "desktop",
        "home",
        "documents",
        "docs",
        "downloads",
        "pictures",
        "photos",
        "videos",
        "movies",
        "screen",
        "files",
        "folder",
        "usb",
        "phone",
        "sale",
        "time",
        "tv",
        "today",
        "tomorrow",
        "monday",
        "tuesday",
        "wednesday",
        "thursday",
        "friday",
        "saturday",
        "sunday",
        "weekend",
    ];
    if words.len() == 1 && NOT_SITES.contains(&words[0]) {
        return None;
    }
    let domain = words.len() == 1 && y.contains('.') && !y.starts_with('.') && !y.ends_with('.');
    let fine = words.iter().all(|w| {
        w.chars()
            .all(|c| c.is_alphanumeric() || "&'+.-".contains(c))
    });
    (fine && !y.starts_with('-') && (domain || (1..=3).contains(&words.len())))
        .then(|| y.to_string())
}

fn site_search(query: &str, site: &str) -> Option<Call> {
    let q = query.trim();
    let site = site_of(site)?;
    (!q.is_empty()).then(|| Call::new("web_search", json!({"query": q, "site": site})))
}

/// Sites people name by name, for the looser phrasings ("find X on Y",
/// "show me X on Y", "search Y for X"), where Y is just as often not a
/// site ("search gifts for mum", "find the report on dropbox"). ewe
/// browser's own table is bigger; these are the ones worth the words.
const KNOWN_SITES: &[&str] = &[
    "zara",
    "h&m",
    "hm",
    "amazon",
    "ebay",
    "etsy",
    "ikea",
    "aliexpress",
    "temu",
    "asos",
    "youtube",
    "youtube music",
    "spotify",
    "soundcloud",
    "bandcamp",
    "netflix",
    "twitch",
    "wikipedia",
    "imdb",
    "goodreads",
    "reddit",
    "github",
    "gitlab",
    "stack overflow",
    "stackoverflow",
    "arch wiki",
    "archwiki",
    "aur",
    "arch packages",
    "npm",
    "crates",
    "pypi",
    "mdn",
    "google maps",
    "openstreetmap",
    "google flights",
    "booking",
    "airbnb",
    "kayak",
    "skyscanner",
    "instagram",
    "facebook",
    "tiktok",
    "linkedin",
    "pinterest",
    "bbc",
    "steam",
    "myauto",
];

/// A site's search only when the site is one people name (`KNOWN_SITES`)
/// or a domain ("zara.com").
fn known_site_search(query: &str, site: &str) -> Option<Call> {
    let call = site_search(query, site)?;
    let s = call.args["site"].as_str().unwrap_or("");
    (KNOWN_SITES.contains(&s) || (s.contains('.') && !s.contains(' '))).then_some(call)
}

/// A number people say: "5", "1.5", "a", "an", "five", "half an".
fn amount(w: &str) -> Option<f64> {
    Some(match w {
        "a" | "an" | "one" => 1.0,
        "two" => 2.0,
        "three" => 3.0,
        "four" => 4.0,
        "five" => 5.0,
        "six" => 6.0,
        "seven" => 7.0,
        "eight" => 8.0,
        "nine" => 9.0,
        "ten" => 10.0,
        "fifteen" => 15.0,
        "twenty" => 20.0,
        "thirty" => 30.0,
        "forty" => 40.0,
        "forty-five" => 45.0,
        "fifty" => 50.0,
        "sixty" => 60.0,
        "ninety" => 90.0,
        w => w.parse().ok()?,
    })
}

/// "5 minutes", "1 hour 30 minutes", "an hour and a half", "half an
/// hour", "90 s", "10min", "2h" → seconds.
pub fn duration_secs(s: &str) -> Option<i64> {
    let s = s
        .trim()
        .replace(" and a half", " +half")
        .replace(" and ", " ");
    // "10min" → "10 min"
    let mut spaced = String::new();
    let mut prev_digit = false;
    for c in s.chars() {
        if prev_digit && c.is_alphabetic() {
            spaced.push(' ');
        }
        prev_digit = c.is_ascii_digit();
        spaced.push(c);
    }
    let unit = |w: &str| -> Option<f64> {
        Some(match w {
            "s" | "sec" | "secs" | "second" | "seconds" => 1.0,
            "m" | "min" | "mins" | "minute" | "minutes" => 60.0,
            "h" | "hr" | "hrs" | "hour" | "hours" => 3600.0,
            _ => return None,
        })
    };
    let words: Vec<&str> = spaced.split_whitespace().collect();
    let (mut total, mut i, mut last_unit) = (0.0f64, 0, 0.0f64);
    while i < words.len() {
        match words[i] {
            // "half an hour"
            "half" if words.get(i + 1).is_some_and(|w| *w == "an" || *w == "a") => {
                total += 0.5 * unit(words.get(i + 2)?)?;
                i += 3;
            }
            // "an hour and a half"
            "+half" if last_unit > 0.0 => {
                total += 0.5 * last_unit;
                i += 1;
            }
            w => {
                let n = amount(w)?;
                let u = unit(words.get(i + 1)?)?;
                total += n * u;
                last_unit = u;
                i += 2;
            }
        }
    }
    let secs = total.round() as i64;
    ((1..=crate::commands::TIMER_MAX_S).contains(&secs) && i > 0).then_some(secs)
}

/// "5pm", "5:30 pm", "17:30", "09:00", "noon", "midnight" → "HH:MM". A
/// lone "5" or "5:30" could be morning or evening: not a time here.
pub fn clock_of(s: &str) -> Option<String> {
    let s = s.trim().replace(['.', ' '], "");
    let (s, half) = match s.as_str() {
        "noon" | "midday" => return Some("12:00".into()),
        "midnight" => return Some("00:00".into()),
        _ if s.ends_with("am") => (s[..s.len() - 2].to_string(), Some(false)),
        _ if s.ends_with("pm") => (s[..s.len() - 2].to_string(), Some(true)),
        _ => (s, None),
    };
    let (h, m) = match s.split_once(':') {
        Some((h, m)) if m.len() == 2 => (h, m),
        Some(_) => return None,
        None => (s.as_str(), "00"),
    };
    let (mut h, m): (u32, u32) = (h.parse().ok()?, m.parse().ok()?);
    if m > 59 {
        return None;
    }
    match half {
        Some(pm) => {
            if !(1..=12).contains(&h) {
                return None;
            }
            h = match (h, pm) {
                (12, false) => 0,
                (12, true) => 12,
                (h, true) => h + 12,
                (h, false) => h,
            };
        }
        // 24-hour: two digits ("09:30", "17:30"), never a bare hour
        None if s.contains(':') && (h >= 13 || s.starts_with('0') || s.len() == 5) && h < 24 => {}
        None => return None,
    }
    Some(format!("{h:02}:{m:02}"))
}

/// A timer or a reminder: when (in some time, or at a time of day) and
/// what for.
fn timer(when: &str, at: bool, text: &str) -> Option<Call> {
    let text = text.trim();
    let mut args = if at {
        json!({"at": clock_of(when)?})
    } else {
        json!({"seconds": duration_secs(when)?})
    };
    if !text.is_empty() {
        args["text"] = json!(text);
    }
    Some(Call::new("timer", args))
}

/// The words for a tab that name no tab: "close the new tab" is not
/// about a tab called "new".
fn tab_words(w: Option<regex::Match>) -> Option<Option<String>> {
    match w.map(|m| m.as_str().trim()) {
        None | Some("") => Some(None),
        Some(
            "this" | "that" | "current" | "active" | "open" | "new" | "a new" | "next" | "last"
            | "previous" | "other" | "all" | "every",
        ) => None,
        Some(w) => Some(Some(w.to_string())),
    }
}

/// A workspace's number as said: "3", "three", "third" (1–10).
fn ws_number(w: &str) -> Option<i64> {
    let n = match w {
        "one" | "first" | "1st" => 1,
        "two" | "second" | "2nd" => 2,
        "three" | "third" | "3rd" => 3,
        "four" | "fourth" | "4th" => 4,
        "five" | "fifth" | "5th" => 5,
        "six" | "sixth" | "6th" => 6,
        "seven" | "seventh" | "7th" => 7,
        "eight" | "eighth" | "8th" => 8,
        "nine" | "ninth" | "9th" => 9,
        "ten" | "tenth" | "10th" => 10,
        w => w.parse().ok()?,
    };
    (1..=10).contains(&n).then_some(n)
}

/// A workspace number in a pattern (digits or a word), and an ordinal.
const WS_NUM: &str = r"(\d{1,2}|one|two|three|four|five|six|seven|eight|nine|ten)";
const WS_ORD: &str = r"(first|second|third|fourth|fifth|sixth|seventh|eighth|ninth|tenth|1st|2nd|3rd|4th|5th|6th|7th|8th|9th|10th)";

/// An app as said ("the firefox app" → "firefox"); None for words that
/// name no app (the computer, the screen, it).
fn app_words(w: &str) -> Option<String> {
    let mut w = w.trim();
    for pre in ["the ", "my ", "this ", "that ", "all "] {
        w = w.strip_prefix(pre).unwrap_or(w).trim();
    }
    for suf in [
        " app",
        " apps",
        " application",
        " window",
        " windows",
        " program",
        " process",
        " now",
        " right now",
        " again",
        " completely",
        " for me",
    ] {
        w = w.strip_suffix(suf).unwrap_or(w).trim();
    }
    const NOT_APPS: &[&str] = &[
        "computer",
        "laptop",
        "pc",
        "screen",
        "desktop",
        "system",
        "mouse",
        "keyboard",
        "shell",
        "bar",
        "top bar",
        "everything",
        "it",
        "this",
        "that",
        "window",
        "app",
        "machine",
        "internet",
        "wifi",
        "wi-fi",
        "bluetooth",
        "sound",
        "audio",
        "network",
        "island",
        "notch",
        "shaun",
        "you",
        "touchpad",
        "trackpad",
        "display",
        "monitor",
        "ewe",
        "os",
        "all",
        "session",
        "phone",
        "music",
        "video",
        "game",
        "page",
        "tab",
        "web page",
        "website",
        "site",
        "cursor",
        "pointer",
        "whole thing",
        "thing",
        "dock",
        "panel",
        "overview",
        "lock screen",
        "fan",
        "battery",
        "camera",
        "mic",
        "microphone",
        "speaker",
        "speakers",
        "update",
        "updates",
        "download",
        "the download",
        "trash",
        "bin",
        "quick settings",
        "launcher",
        "menu",
        "popup",
        "dialog",
        "safe",
        "ewe safe",
    ];
    // a tab or a page is ewe browser's ("close the other tab")
    let last = w.rsplit(' ').next().unwrap_or(w);
    if w.is_empty()
        || w.len() > 40
        || NOT_APPS.contains(&w)
        || matches!(
            last,
            "tab" | "tabs" | "page" | "pages" | "mode" | "notification" | "notifications"
        )
        || w.starts_with('-')
        || w.split_whitespace().count() > 3
        || !w
            .chars()
            .all(|c| c.is_alphanumeric() || " ._+-".contains(c))
    {
        return None;
    }
    Some(w.to_string())
}

/// "30 days", "2 weeks", "a week", "a fortnight", "ten days" → days.
fn span_days(w: &str) -> Option<i64> {
    let w = w.trim();
    if w == "a fortnight" || w == "one fortnight" {
        return Some(14);
    }
    let (n, unit) = w.split_once(' ')?;
    let n = amount(n)?;
    let per = match unit {
        "day" | "days" => 1.0,
        "week" | "weeks" => 7.0,
        "fortnight" | "fortnights" => 14.0,
        _ => return None,
    };
    let d = (n * per).round() as i64;
    (n.fract() == 0.0 && (1..=36_500).contains(&d)).then_some(d)
}

/// A date question's words, when they name a day (`dates::parse`).
fn date_call(op: &str, words: &str) -> Option<Call> {
    let w = words.trim();
    crate::dates::parse(w)?;
    Some(Call::new("date_math", json!({"op": op, "date": w})))
}

/// A memory as said, or a reminder when it has a time: "remember to call
/// mum at 5pm" is a reminder, "remember to call mum" a memory. A time that
/// could be morning or evening ("at 5") is the model's to ask about.
fn remember_or_remind(what: &str) -> Option<Call> {
    let what = what.trim().trim_start_matches(':').trim();
    if what.is_empty() || matches!(what, "that" | "this" | "it" | "me") {
        return None;
    }
    if let Some(todo) = what.strip_prefix("to ") {
        for sep in [" at ", " in "] {
            if let Some(i) = todo.rfind(sep) {
                let (text, when) = (&todo[..i], &todo[i + sep.len()..]);
                let at = sep == " at ";
                if let Some(mut call) = timer(when, at, text) {
                    call.args["reminder"] = json!(true);
                    return Some(call);
                }
                // "at 5", "in a bit": a time it cannot read
                if when.starts_with(|c: char| c.is_ascii_digit()) {
                    return None;
                }
            }
        }
    }
    Some(Call::new("remember", json!({"text": what})))
}

/// A url or a bare domain, as `open_url` takes it.
fn web_address(t: &str) -> String {
    if t.starts_with("http://") || t.starts_with("https://") {
        t.to_string()
    } else {
        format!("https://{t}")
    }
}

/// The endings a bare site may have ("open github.com"), so "open
/// notes.md" stays a file's business.
const TLDS: &str = "com|org|net|io|dev|app|ge|de|uk|co|ai|edu|gov|me|tv|fm|ly|gg|xyz|info|eu|us|ca|au|jp|fr|it|es|nl|ru|pl|ch|se|no|fi|dk|be|at|br|tr|ua|am|az|news|wiki|blog|tech|site|online|store|to|cc|is|social";

fn rules() -> &'static [Rule] {
    static RULES: OnceLock<Vec<Rule>> = OnceLock::new();
    RULES.get_or_init(|| {
        let r = |p: &str, make: fn(&regex::Captures) -> Option<Call>| Rule {
            re: Regex::new(&format!("^(?:{p})$")).expect("intent pattern"),
            make,
        };
        vec![
            // ── a sum: only when the words are one (calc::eval), so "what is
            //    the volume" falls through ──
            r(r"(?:what(?:'s| is)|calculate|calc|compute|how much is|work out|solve) (.+?)(?: ?=)?|(.+?) ?=?",
              |c| {
                  let e = c.get(1).or(c.get(2))?.as_str().trim().trim_end_matches('=').trim();
                  // "555-1234", "2026-10-18": a phone number or a date
                  // unless asked for as a sum ("what is 12-3", "12 - 3")
                  if c.get(2).is_some() && crate::calc::looks_like_a_number(e) {
                      return None;
                  }
                  crate::calc::eval(e)?;
                  Some(Call::new("calc", json!({"expr": e})))
              }),
            // ── units: "convert 10 km to miles", "20 c in f", "how many cm
            //    in 3 feet". Like the sum, it reads every line and declines
            //    unless both units are units of one kind (no currencies) ──
            r(r".+",
              |c| {
                  let (v, from, to) = crate::units::parse_question(&c[0])?;
                  let value = if v.fract() == 0.0 && v.abs() < 1e15 { json!(v as i64) } else { json!(v) };
                  Some(Call::new("convert", json!({"value": value, "from": from, "to": to})))
              }),
            // ── the clock: here, or in a place ──
            r(r"(?:what(?:'?s| is) )?(?:the )?(?:current |local |exact )?(?:time|date) (?:is it |now |right now )?in (.+?)(?: (?:now|right now|today))?|what time is it (?:now |right now )?in (.+?)(?: now| right now)?|what time (?:it is|is it now) in (.+)",
              |c| {
                  let p = c.get(1).or(c.get(2)).or(c.get(3))?.as_str().trim();
                  let p = p.strip_prefix("the ").unwrap_or(p);
                  // "the date in a week" is a date, not a place
                  if p.is_empty() || p.chars().any(|ch| ch.is_ascii_digit()) || p.split_whitespace().count() > 4
                      || span_days(p).is_some() || matches!(p, "tomorrow" | "yesterday" | "today")
                  {
                      return None;
                  }
                  Some(Call::new("clock", json!({"place": p})))
              }),
            r(r"(?:what(?:'?s| is) )?(?:the )?(?:current |exact )?time(?: now| right now| please)?|what time is it(?: now| right now| here)?|what time it is|(?:tell me|do you (?:know|have)|have you got|got) the time|(?:what(?:'?s| is) )?(?:the |today'?s )?date(?: today)?|what(?:'?s| is) (?:the date|today'?s date|the day)(?: today)?|what date is it(?: today)?|what day is (?:it|today)(?: today)?|what(?:'?s| is) today|which day is (?:it|today)|(?:what(?:'?s| is) )?the time and date|(?:the )?date and time",
              |_| Some(Call::new("clock", json!({})))),
            // ── dates: days until or since, a date's weekday, a date from
            //    today. Only words that name a day (dates::parse) ──
            r(r"(?:how many|how much) (?:days|weeks|sleeps|time) (?:is it |are there |are left |is left |left |do i have |have i got )?(?:until|till|til|to|before) (.+)|how long (?:is it |is left |do i have |have i got |is there )?(?:until|till|til|to|before) (.+)|(?:days|time|countdown) (?:left )?(?:until|till|to) (.+)|when is (.+)",
              |c| date_call("until", c.get(1).or(c.get(2)).or(c.get(3)).or(c.get(4))?.as_str())),
            r(r"how (?:many days|many weeks|long) (?:has it been |is it |ago was it |ago was )?since (.+)|how many days ago was (.+)|(?:days|time) since (.+)",
              |c| date_call("since", c.get(1).or(c.get(2)).or(c.get(3))?.as_str())),
            r(r"what(?:'?s| is| was| will be) the date (?:in |after )?(.+?)(?: (from now|from today|ago))?|what (?:date|day) (?:is it|will it be|was it) (?:in |after )?(.+?)(?: (from now|from today|ago))?|(?:the )?date in (.+)|(.+?) (from now|from today|ago)",
              |c| {
                  let (words, dir) = match (c.get(1), c.get(3), c.get(5), c.get(6)) {
                      (Some(w), ..) => (w.as_str(), c.get(2)),
                      (_, Some(w), ..) => (w.as_str(), c.get(4)),
                      (_, _, Some(w), _) => (w.as_str(), None),
                      (_, _, _, Some(w)) => (w.as_str(), c.get(7)),
                      _ => return None,
                  };
                  let back = dir.is_some_and(|d| d.as_str() == "ago");
                  let days = match words {
                      "tomorrow" if !back => 1,
                      "yesterday" if !back => -1,
                      w => span_days(w)? * if back { -1 } else { 1 },
                  };
                  Some(Call::new("date_math", json!({"op": "offset", "days": days})))
              }),
            r(r"what day (?:of the week )?(?:is|was|will be|falls on|does) (.+?)(?: fall on| on)?|(?:which|what) (?:day of the week|weekday) (?:is|was|will be) (.+)|what (?:day|weekday) (?:is|was) (.+?) on",
              |c| date_call("weekday", c.get(1).or(c.get(2)).or(c.get(3))?.as_str())),
            // ── this computer: battery, memory, CPU, uptime, the network,
            //    disk space; "why is my laptop slow" is all of it ──
            r(r"(?:how much |what(?:'?s| is) (?:my |the )?)?battery(?: (?:level|left|life|charge|percentage|percent|status|state))?(?: (?:do i have|is left|left|now))?|how much (?:battery|charge|power) (?:do i have|is left|have i got|left|is there)(?: (?:on|in) (?:my |the )?(?:laptop|computer|battery))?(?: left)?|how(?:'?s| is) (?:my |the )?battery(?: doing)?|is (?:my |the )?(?:laptop|computer|battery) (?:charging|plugged in|on battery|full)|how long (?:will|does|can) (?:my |the )?(?:battery|laptop|charge) last|am i (?:charging|on battery|plugged in)",
              |_| Some(Call::new("system_info", json!({"what": "battery"})))),
            r(r"what(?:'?s| is) (?:using|eating|taking|hogging) (?:up )?(?:all |so much |the |my |of )*(?:ram|memory)|(?:which|what) (?:apps?|programs?|processes) (?:use|uses|are using|is using) (?:the most |so much |all )?(?:of )?(?:my |the )?(?:ram|memory)|(?:top|biggest) (?:memory|ram) (?:apps|users|hogs)|what(?:'?s| is) using (?:the most|so much) (?:ram|memory)",
              |_| Some(Call::new("system_info", json!({"what": "memory", "apps": true})))),
            r(r"how much (?:free )?(?:ram|memory)(?: is)? (?:free|left|available|used|in use|do i have|am i using|have i got|is being used|is there)(?: left| now)?|(?:how much |what(?:'?s| is) )?(?:my |the )?(?:ram|memory) (?:usage|use)(?: now)?|(?:ram|memory) usage|is (?:my |the )?(?:ram|memory) full|(?:free|available) (?:ram|memory)|how much (?:ram|memory) (?:does (?:my |this )?(?:computer|laptop) have|have i)",
              |_| Some(Call::new("system_info", json!({"what": "memory"})))),
            r(r"what(?:'?s| is) (?:using|eating|hogging|taking) (?:up )?(?:all |so much |the |my |of )*(?:cpu|processor)(?: power| time)?|(?:which|what) (?:apps?|programs?|processes) (?:use|uses|are using|is using) (?:the most |so much |all )?(?:of )?(?:my |the )?(?:cpu|processor)|(?:how much |what(?:'?s| is) (?:my |the )?)?cpu(?: usage| use| load)?(?: now| right now)?|how busy is (?:my |the |this )?(?:cpu|computer|processor|laptop|pc)|(?:system |cpu )?load(?: average)?|top (?:processes|apps|cpu)|what(?:'?s| is) running(?: on my (?:computer|laptop|pc))?",
              |_| Some(Call::new("system_info", json!({"what": "cpu", "apps": true})))),
            r(r"how long (?:has|have) (?:my |the |this )?(?:computer|laptop|pc|machine|system|i) been (?:on|up|running|awake)|(?:what(?:'?s| is) (?:my |the )?)?uptime|when did (?:i|my computer|the computer|my laptop|my pc|this computer) (?:last )?(?:start|boot|turn on|switch on)(?: up)?|since when (?:is|has) (?:my |the )?(?:computer|laptop|pc) (?:been )?on",
              |_| Some(Call::new("system_info", json!({"what": "uptime"})))),
            r(r"am i (?:connected|online|offline)(?: to the internet)?|(?:is|do i have) (?:the |my )?internet(?: connection)?(?: working| on| up| down| connected)?|is (?:my |the )?(?:internet|network|wi-?fi|connection) (?:working|up|down|on|connected|ok|okay)|(?:what(?:'?s| is) )?my (?:local |public |internal )?ip(?: address)?|what(?:'?s| is) (?:my|the|this computer'?s) (?:local |public |internal )?ip(?: address)?|(?:what(?:'?s| is) )?(?:the |my )?ip address|(?:show )?(?:my )?network (?:status|info|details)|am i on (?:wi-?fi|ethernet|cable)|how am i connected(?: to the internet)?|what network am i (?:on|connected to)|(?:what(?:'?s| is) )?my network",
              |_| Some(Call::new("system_info", json!({"what": "network"})))),
            r(r"what(?:'?s| is) (?:eating|using|taking|filling|hogging) (?:up )?(?:all |so much |of )*(?:my |the )?(?:disk|disk space|space|storage|drive|ssd|hard drive)(?: space)?|where (?:did|has) (?:all )?my (?:disk )?space gone|where did my (?:disk )?space go|what(?:'?s| is) (?:big|large|the biggest|the largest) on (?:my |the )?(?:disk|drive|computer)|(?:show|find)(?: me)? (?:the )?(?:big|large|biggest|largest) (?:files|folders|things)(?: on (?:my |the )?(?:disk|computer))?|what can i (?:delete|clean|clear|remove) to (?:free|make|get) (?:up )?(?:some )?space|(?:clean ?up|free up) (?:some )?(?:my |the )?(?:disk|space|storage|disk space)|disk usage by folder",
              |_| Some(Call::new("disk_usage", json!({})))),
            r(r"how much (?:disk |storage |free |hard drive |ssd )*space (?:is (?:left|free|there|available)|do i have(?: left)?|have i got(?: left)?|left|free|available)(?: on (?:my |the |this )?(?:disk|drive|computer|laptop|ssd|pc))?|how much (?:free |disk )?space (?:does (?:my |the )?(?:disk|drive|computer) have)|how (?:full|big) is (?:my |the )?(?:disk|drive|ssd|hard drive|storage)|(?:disk|storage) (?:space|usage|status)|is (?:my |the )?(?:disk|drive|storage|ssd) (?:full|nearly full|almost full)|(?:free|available) (?:disk |storage )?space|how much storage (?:do i have|is left)(?: left)?",
              |_| Some(Call::new("system_info", json!({"what": "disk"})))),
            r(r"why is (?:my |the |this )?(?:computer|laptop|pc|machine|system|desktop|everything|it) (?:so |being |running )?(?:slow|sluggish|laggy|lagging|hot|loud|noisy)(?: today| now| right now| lately)?|(?:my |the )?(?:computer|laptop|pc|machine|system) (?:is|feels|seems) (?:so |really |very |a bit )?(?:slow|sluggish|laggy|hot|loud)|is (?:my |the |this )?(?:computer|laptop|pc|machine|system) (?:ok|okay|alright|all right|fine|healthy|slow|overloaded)|how(?:'?s| is) (?:my |the |this )?(?:computer|laptop|pc|machine|system)(?: doing| holding up)?|(?:system|computer|laptop|pc) (?:status|health|check|overview|report)|check (?:my |the |this )?(?:computer|laptop|system|pc)|what(?:'?s| is) slowing (?:down )?(?:my |the |this )?(?:computer|laptop|pc|machine)(?: down)?",
              |_| Some(Call::new("system_info", json!({"what": "overview"})))),
            // ── volume ──
            r(r"(?:set |change )?(?:the )?(?:volume|sound)(?: level)? (?:to |at )?(\d{1,3}) ?%?|(?:set |turn )?(?:the )?volume to (\d{1,3}) ?(?:%|percent)?",
              |c| Some(Call::new("volume_set", json!({"percent": num(c, 1).or_else(|| num(c, 2))?})))),
            r(r"(?:turn |make )?(?:the )?(?:volume|sound|it) (up|down)(?: by (\d{1,3}) ?%?)?|(louder|quieter|softer)",
              |c| {
                  let up = matches!(c.get(1).or(c.get(3))?.as_str(), "up" | "louder");
                  let by = num(c, 2).unwrap_or(STEP);
                  Some(Call::new("volume_change", json!({"delta": if up { by } else { -by }})))
              }),
            r(r"(?:turn |make )?(?:the )?(?:volume|sound) (?:a bit |a little )?(higher|lower)|(increase|decrease|raise|lower) (?:the )?(?:volume|sound)(?: by (\d{1,3}) ?%?)?",
              |c| {
                  let w = c.get(1).or(c.get(2))?.as_str();
                  let up = matches!(w, "higher" | "increase" | "raise");
                  let by = num(c, 3).unwrap_or(STEP);
                  Some(Call::new("volume_change", json!({"delta": if up { by } else { -by }})))
              }),
            r(r"(?:make |turn )?(?:it|the (?:volume|sound|music|audio)|things|everything)? ?(?:a (?:bit|little|tad|touch) |much |slightly |way )?(louder|quieter|softer)(?: (?:a bit|a little|please))?|(?:it'?s|it is|that'?s|that is|this is|the (?:music|sound|volume) is) (?:way |much |a bit |far )?too (loud|quiet|soft|low|high)",
              |c| {
                  let up = matches!(c.get(1).or(c.get(2))?.as_str(), "louder" | "quiet" | "soft" | "low");
                  Some(Call::new("volume_change", json!({"delta": if up { STEP } else { -STEP }})))
              }),
            r(r"(mute|unmute)(?: (?:the )?(?:sound|audio|speakers?|volume))?",
              |c| Some(Call::new("mute", json!({"on": &c[1] == "mute"})))),
            // ── brightness ──
            r(r"(?:set |change )?(?:the )?(?:screen )?brightness (?:to |at )?(\d{1,3}) ?(?:%|percent)?",
              |c| Some(Call::new("brightness_set", json!({"percent": num(c, 1)?.clamp(1, 100)})))),
            r(r"(?:turn |make )?(?:the )?(?:screen )?brightness (up|down)(?: by (\d{1,3}) ?%?)?|(increase|decrease|raise|lower) (?:the )?(?:screen )?brightness(?: by (\d{1,3}) ?%?)?|(?:make (?:the )?screen )?(brighter|dimmer|darker)|dim (?:the )?screen",
              |c| {
                  let w = c.get(1).or(c.get(3)).or(c.get(5)).map(|m| m.as_str()).unwrap_or("dim");
                  let up = matches!(w, "up" | "increase" | "raise" | "brighter");
                  let by = num(c, 2).or_else(|| num(c, 4)).unwrap_or(STEP);
                  Some(Call::new("brightness_change", json!({"delta": if up { by } else { -by }})))
              }),
            // ── media ──
            r(r"(?:play|pause|resume|stop)(?: (?:the )?(?:music|song|track|video|media))?|play ?pause|toggle (?:the )?music",
              |_| Some(Call::new("media", json!({"action": "play_pause"})))),
            r(r"(?:next|skip)(?: (?:the )?(?:song|track|one))?|skip (?:this )?(?:song|track)",
              |_| Some(Call::new("media", json!({"action": "next"})))),
            r(r"(?:previous|prev|last|go back)(?: (?:song|track|one))?|play (?:the )?previous(?: song| track)?",
              |_| Some(Call::new("media", json!({"action": "previous"})))),
            // ── do not disturb ──
            r(r"(turn on|turn off|switch on|switch off|enable|disable) (?:do not disturb|dnd|notifications)|(?:do not disturb|dnd) (on|off)|(?:be quiet|silence notifications)",
              |c| {
                  let w = c.get(1).or(c.get(2)).map(|m| m.as_str()).unwrap_or("on");
                  // "turn off notifications" means quiet, i.e. DND on
                  let on = if c[0].ends_with("notifications") { !on_word(w)? } else { on_word(w)? };
                  Some(Call::new("dnd", json!({"on": on})))
              }),
            // ── game mode ──
            r(r"(enter|start|turn on|switch on|enable|exit|leave|stop|quit|turn off|switch off|disable) (?:the )?game ?mode|game ?mode(?: (on|off))?",
              |c| {
                  let on = match (c.get(1), c.get(2)) {
                      (Some(w), _) => on_word(w.as_str())?,
                      (None, Some(w)) => on_word(w.as_str())?,
                      (None, None) => true,
                  };
                  Some(Call::new("game_mode", json!({"on": on})))
              }),
            // ── Wi-Fi, Bluetooth, airplane mode, keeping awake ──
            r(r"(turn on|turn off|switch on|switch off|enable|disable) (?:the )?wi-?fi|(?:turn|switch) (?:the )?wi-?fi (on|off)|wi-?fi (on|off)",
              |c| Some(Call::new("wifi", json!({"on": on_word(c.get(1).or(c.get(2)).or(c.get(3))?.as_str())?})))),
            r(r"(turn on|turn off|switch on|switch off|enable|disable) (?:the )?bluetooth|(?:turn|switch) (?:the )?bluetooth (on|off)|bluetooth (on|off)",
              |c| Some(Call::new("bluetooth", json!({"on": on_word(c.get(1).or(c.get(2)).or(c.get(3))?.as_str())?})))),
            r(r"(turn on|turn off|switch on|switch off|enable|disable|enter|exit|leave) (?:the )?(?:airplane|aeroplane|flight|plane) mode|(?:airplane|aeroplane|flight|plane) mode(?: (on|off))?",
              |c| {
                  let on = match c.get(1).or(c.get(2)) {
                      Some(w) => on_word(w.as_str())?,
                      None => true,
                  };
                  Some(Call::new("airplane_mode", json!({"on": on})))
              }),
            r(r"(dis)?connect(?: to)?(?: my| the)? (.+?)(?: (?:via|over|with|using) bluetooth)?",
              |c| {
                  let name = c[2].trim();
                  // the network, a VPN, an account: not a Bluetooth device
                  const NOT_DEVICES: &[&str] = &["wifi", "wi-fi", "internet", "network", "vpn", "ethernet",
                      "hotspot", "server", "ssh", "nextcloud", "account", "cloud", "bluetooth", "tv",
                      "monitor", "display", "screen", "printer", "web"];
                  if name.split_whitespace().any(|w| NOT_DEVICES.contains(&w)) || name.contains('.') {
                      return None;
                  }
                  Some(Call::new("bluetooth_device", json!({"name": name, "connect": c.get(1).is_none()})))
              }),
            r(r"keep (?:the |my )?(?:screen |computer |laptop |pc |display |machine )?awake|stay awake|(?:don'?t|do not) let (?:the |my )?(?:screen|computer|laptop|pc) (?:sleep|lock|dim)|(turn on|turn off|switch on|switch off|enable|disable|start|stop) (?:caffeine|insomnia|keep awake)|(?:keep awake|caffeine|insomnia) (on|off)|stop keeping (?:the |my )?(?:screen |computer |laptop )?awake|(?:let|allow) (?:the |my )?(?:screen|computer|laptop|pc) (?:to )?(?:sleep|lock|dim)(?: again)?",
              |c| {
                  let t = &c[0];
                  let on = match c.get(1).or(c.get(2)) {
                      Some(w) => on_word(w.as_str())?,
                      None => !(t.starts_with("stop") || t.starts_with("let") || t.starts_with("allow")),
                  };
                  Some(Call::new("keep_awake", json!({"on": on})))
              }),
            // ── timers and reminders ──
            r(r"(?:set |start )?(?:a |an )?(?:timer|alarm) (?:for |of )?(.+?)(?: (?:called|named|labelled|labeled) (.+))?|(?:set |start )?(?:a |an )?(.+?) timer(?: (?:called|named|labelled|labeled|for) (.+))?|timer (.+)",
              |c| {
                  let when = c.get(1).or(c.get(3)).or(c.get(5))?.as_str();
                  let text = c.get(2).or(c.get(4)).map_or("", |m| m.as_str());
                  timer(when, false, text)
              }),
            r(r"remind me (in|at) (.+?)(?: (?:to|that|about) (.+))?|remind me (?:to|that|about) (.+) (in|at) (.+)|(?:set |add )?(?:a )?reminder (in|at|for) (.+?)(?: (?:to|that|about) (.+))?",
              |c| {
                  let (how, when, text) = if c.get(1).is_some() {
                      (&c[1], &c[2], c.get(3).map_or("", |m| m.as_str()))
                  } else if c.get(4).is_some() {
                      (&c[5], &c[6], &c[4])
                  } else {
                      (&c[7], &c[8], c.get(9).map_or("", |m| m.as_str()))
                  };
                  let at = how != "in" && clock_of(when).is_some();
                  let mut call = timer(when, at, text)?;
                  call.args["reminder"] = json!(true);
                  Some(call)
              }),
            r(r"(?:cancel|stop|delete|remove|clear|turn off)(?: the| my| that| this| all(?: my| the)?)? (?:timers?|reminders?|alarms?)",
              |_| Some(Call::new("timer_cancel", json!({})))),
            r(r"(?:show|list|what are)(?: me)?(?: my| the| all)? (?:timers|reminders|timers and reminders|alarms)|(?:what|which) (?:timers|reminders) (?:do i have|are (?:set|running|on|there))|how (?:long|much time) is left(?: on (?:my|the) timer)?|(?:my )?(?:timers|reminders)",
              |_| Some(Call::new("timers", json!({})))),
            // ── memory: "remember that …" (a reminder when it has a time),
            //    "what do you remember", "forget …" ──
            r(r"(?:please )?remember(?: that|:)? (.+)|(?:please )?(?:don'?t|do not) forget(?: that)? (.+)|(?:keep in mind|make a note|take a note|note down)(?: that|:)? (.+)",
              |c| remember_or_remind(c.get(1).or(c.get(2)).or(c.get(3))?.as_str())),
            r(r"what (?:else )?do you (?:remember|know)(?: about me)?|what (?:have you|did you) remember(?:ed)?(?: about me)?|what (?:things )?(?:did|have) i (?:ask(?:ed)? you to|told you to) remember|(?:show|list|tell me)(?: me)?(?: my| your| all(?: my| your)?| the)? (?:memories|things you remember)|(?:your|my) memories|what do you remember about (.+)|do you remember (.+)",
              |c| {
                  let q = c.get(1).or(c.get(2)).map_or("", |m| m.as_str().trim());
                  let q = q.strip_suffix(" about me").unwrap_or(q);
                  let mut args = json!({});
                  if !q.is_empty() && q != "me" && q != "anything" {
                      args["query"] = json!(q);
                  }
                  Some(Call::new("memories", args))
              }),
            r(r"forget (?:everything|all(?: of it)?|it all|all (?:my |your |the )?memories|(?:everything |all |what )?you (?:know|remember)(?: about me)?)",
              |_| Some(Call::new("forget", json!({"all": true})))),
            r(r"forget (?:that |about |the memory |what i (?:said|told you) about )?(.+)",
              |c| {
                  let w = c[1].trim();
                  let w = w.strip_prefix("that ").unwrap_or(w).trim();
                  if matches!(w, "it" | "that" | "this" | "about it" | "me" | "it please" | "you") || w.len() > 100 {
                      return None;
                  }
                  Some(Call::new("forget", json!({"words": w})))
              }),
            // ── dark or light, power, the trash: proposals (asks first) ──
            r(r"(turn on|switch to|change to|go to|use|enable|turn off|disable) (?:the )?(light|dark) (?:mode|theme)|(light|dark) (?:mode|theme)(?: on)?|go (light|dark)",
              |c| {
                  let mode = c.get(2).or(c.get(3)).or(c.get(4))?.as_str();
                  let off = c.get(1).is_some_and(|w| matches!(w.as_str(), "turn off" | "disable"));
                  let mode = match (mode, off) {
                      ("light", true) => "dark",
                      ("dark", true) => "light",
                      (m, _) => m,
                  };
                  Some(Call::new("theme_mode", json!({"mode": mode})))
              }),
            r(r"(?:suspend|sleep)(?: the| my)?(?: computer| laptop| pc| system| machine)?(?: now)?|(?:go to|put (?:the |my )?(?:computer|laptop|pc|machine) to) sleep|(restart|reboot)(?: the| my)?(?: computer| laptop| pc| system| machine)?(?: now)?|(shut ?down|power off)(?: the| my)?(?: computer| laptop| pc| system| machine)?(?: now)?|(turn off) (?:the |my )?(?:computer|laptop|pc|machine)(?: now)?|(log ?out|sign ?out|log off)(?: of (?:ewe|the desktop|my session|the session))?",
              |c| {
                  let action = if c.get(1).is_some() {
                      "restart"
                  } else if c.get(2).is_some() || c.get(3).is_some() {
                      "shutdown"
                  } else if c.get(4).is_some() {
                      "logout"
                  } else {
                      "suspend"
                  };
                  Some(Call::new("power", json!({"action": action})))
              }),
            r(r"(?:empty|clear) (?:the |my )?(?:trash|bin|recycle bin|wastebasket|rubbish bin)(?: can| bin)?",
              |_| Some(Call::new("empty_trash", json!({})))),
            // ── screenshot, lock ──
            r(r"(?:take )?(?:a )?screen ?shot(?: of)?(?: (?:the )?(whole screen|full screen|screen|window|region|area))?|screenshot",
              |c| {
                  let mode = match c.get(1).map(|m| m.as_str()) {
                      Some("whole screen" | "full screen" | "screen") => "full",
                      Some("window") => "window",
                      _ => "region",
                  };
                  Some(Call::new("screenshot", json!({"mode": mode})))
              }),
            // ── Safe, the password manager: Safe copies, the companion
            //    never sees a value. Before "lock", so "lock my safe" is
            //    Safe's and "lock" stays the screen ──
            r(r"copy (?:my |the )?(.+?)(?:'s)? (password|pass|username|user ?name|login|e-?mail|2fa code|2fa|one[- ]time code|otp|totp|verification code|security code|code)(?: (?:from|in) (?:my |the )?safe)?",
              |c| safe_copy(&c[1], &c[2])),
            r(r"copy (?:my |the )?(password|username|user ?name|login|e-?mail|2fa code|one[- ]time code|otp|code) (?:for|of|from) (?:my |the )?(.+?)(?: (?:from|in) (?:my |the )?safe)?",
              |c| safe_copy(&c[2], &c[1])),
            r(r"(?:log|sign) me (?:in|into|on)(?: to| at)? (.+)|(?:log|sign) (?:in|into|on)(?: to| at)? (.+)",
              |c| {
                  let q = safe_query(c.get(1).or(c.get(2))?.as_str())?;
                  Some(Call::new("safe_login", json!({"query": q})))
              }),
            r(r"(?:lock|close) (?:my |the )?(?:ewe )?safe",
              |_| Some(Call::new("safe_lock", json!({})))),
            r(r"unlock (?:my |the )?(?:ewe )?safe",
              |_| Some(Call::new("safe_unlock", json!({})))),
            r(r"is (?:my |the )?safe (?:synced|in sync|up to date|locked|unlocked|open)|(?:my |the )?safe(?:'s)? status|how many (?:passwords|logins|items|accounts) (?:do i have|(?:are )?(?:there )?in (?:my |the )?safe|does (?:my |the )?safe (?:have|hold))(?: in (?:my |the )?safe)?",
              |_| Some(Call::new("safe_status", json!({})))),
            r(r"lock(?: (?:the )?(?:screen|computer|pc|laptop|session))?",
              |_| Some(Call::new("lock", json!({})))),
            // ── open an ewe app ──
            r(r"(?:open|launch|show)(?: the)? (settings|ewe settings|komble|store|app store|software|sync|ewe-sync|ewe sync|terminal|kitty|files|file manager)",
              |c| {
                  let what = match &c[1] {
                      "settings" | "ewe settings" => "settings",
                      "komble" | "store" | "app store" | "software" => "komble",
                      "sync" | "ewe-sync" | "ewe sync" => "sync",
                      "terminal" | "kitty" => "terminal",
                      _ => "files",
                  };
                  Some(Call::new("open", json!({"what": what})))
              }),
            // ── "install X": Komble's search for it (it only opens) ──
            r(r"install (.+)",
              |c| {
                  let w = c[1].trim();
                  let page = match w {
                      "updates" | "the updates" | "all updates" | "all the updates" | "update" => "updates".to_string(),
                      "plugins" | "a plugin" | "plugin" | "some plugins" => "plugins".to_string(),
                      w => {
                          let w = w.strip_prefix("the ").or_else(|| w.strip_prefix("an ")).or_else(|| w.strip_prefix("a ")).unwrap_or(w);
                          let w = w.strip_suffix(" app").unwrap_or(w).trim();
                          format!("search={w}")
                      }
                  };
                  let call = Call::new("open", json!({"what": "komble", "page": page}));
                  crate::commands::validate(&call).is_ok().then_some(call)
              }),
            // ── a place in ewe: a Quick settings tab, a Komble page, a
            //    Settings page, the Overview ──
            r(r"(?:open|show|go to)(?: the| my)? (quick settings|control center|control centre|wi-?fi|bluetooth|sound|audio|notifications|calendar)(?: settings| panel| page| tab)?",
              |c| {
                  let tab = match &c[1] {
                      "quick settings" | "control center" | "control centre" => "home",
                      "bluetooth" => "bt",
                      "sound" | "audio" => "audio",
                      "notifications" => "notifs",
                      "calendar" => "cal",
                      _ => "wifi",
                  };
                  Some(Call::new("open", json!({"what": "quicksettings", "page": tab})))
              }),
            r(r"(?:open|show)(?: the| my)? (updates|plugins)|check for updates",
              |c| Some(Call::new("open", json!({"what": "komble", "page": c.get(1).map_or("updates", |m| m.as_str())})))),
            r(r"(?:open|show)(?: the)? overview",
              |_| Some(Call::new("open", json!({"what": "overview"})))),
            r(r"(?:open|show|go to)(?: the| my)? (appearance|accessibility|animations?|layout|window rules?|displays?|monitors?|screens?|network(?:ing)?|wallpaper|keyboard|mouse|touchpad|screensaver|lock screen|power|battery|default apps?|startup(?: apps)?|(?:keyboard )?shortcuts|time|language|user|account) settings",
              |c| {
                  // the account (sign-in, backups, mail) is ewe Sync's since
                  // ewe Settings 0.22: there is no Account page
                  if &c[1] == "account" {
                      return Some(Call::new("open", json!({"what": "sync"})));
                  }
                  let page = match &c[1] {
                      "animation" | "animations" => "animations",
                      "window rule" | "window rules" => "windowrules",
                      "display" | "displays" | "monitor" | "monitors" | "screen" | "screens" => "displays",
                      "network" | "networking" => "network",
                      "keyboard" | "mouse" | "touchpad" => "input",
                      "screensaver" | "lock screen" => "saver",
                      "power" | "battery" => "power",
                      "default app" | "default apps" => "defaults",
                      "startup" | "startup apps" => "startup",
                      "shortcuts" | "keyboard shortcuts" => "shortcuts",
                      "time" | "language" => "time",
                      other => other,
                  };
                  Some(Call::new("open", json!({"what": "settings", "page": page})))
              }),
            // ── the browser (ewe browser's own command line when it has
            //    one) ──
            r(r"(?:open|go to|visit) (https?://\S+)",
              |c| Some(Call::new("open_url", json!({"url": &c[1]})))),
            // a site in one of ewe browser's workspaces: "open github.com in
            // my work workspace"
            r(r"(?:open|go to|visit) (https?://\S+|(?:www\.)?[a-z0-9-]+(?:\.[a-z0-9-]+)*\.[a-z]{2,}(?:/\S*)?) in (?:my |the )?(?:browser )?(.+?) workspace(?: in (?:the |ewe )?browser)?|(?:open|go to|visit) (https?://\S+|(?:www\.)?[a-z0-9-]+(?:\.[a-z0-9-]+)*\.[a-z]{2,}(?:/\S*)?) in (?:the )?(?:browser )?workspace (.+?)(?: in (?:the |ewe )?browser)?",
              |c| {
                  let url = web_address(c.get(1).or(c.get(3))?.as_str());
                  let ws = c.get(2).or(c.get(4))?.as_str().trim();
                  // "in workspace 2" is the desktop's unless "browser" is said
                  if ws.chars().all(|ch| ch.is_ascii_digit()) && !c[0].contains("browser") {
                      return None;
                  }
                  Some(Call::new("open_url", json!({"url": url, "workspace": ws})))
              }),
            // a bare site: "open github.com" (a known ending, so "open
            // notes.md" stays a file's business)
            r(&format!(r"(?:open|go to|visit|browse to) ((?:www\.)?[a-z0-9-]+(?:\.[a-z0-9-]+)*\.(?:{TLDS})(?:/\S*)?|www\.\S+)"),
              |c| Some(Call::new("open_url", json!({"url": format!("https://{}", &c[1])})))),
            // the browser's history: "find tax in my history", "that site
            // about sheep", "show my history"
            r(r"(?:find|search(?: for)?|look up|look for|show me) (.+?) in (?:my |the )?(?:browser |browsing |web )?history|(?:what was |find |open |show me |go to )?(?:that|the) (?:web ?site|site|page|article) (?:about|with|on) (.+)|(?:show|open|list)(?: me)?(?: my| the)? (?:browser |browsing |web )?history",
              |c| {
                  let q = c.get(1).or(c.get(2)).map_or("", |m| m.as_str()).trim();
                  Some(Call::new("browser_history", json!({"query": q})))
              }),
            // the browser's bookmarks: "find recipes in my bookmarks"
            r(r"(?:find|search(?: for)?|look for|look up|show me) (.+?) in (?:my |the )?(?:browser )?bookmarks|search (?:my |the )?(?:browser )?bookmarks for (.+)|(?:show|list|open)(?: me)?(?: my| the| all)? (?:browser )?bookmarks",
              |c| {
                  let q = c.get(1).or(c.get(2)).map_or("", |m| m.as_str()).trim();
                  Some(Call::new("browser_bookmarks", json!({"query": q})))
              }),
            // the browser's own pages: "open the browser's settings"
            r(r"(?:open|show|go to)(?: the| my)? (?:ewe )?browser(?:'s)? (home|history|settings|music|files)(?: page)?|(?:open|show|go to)(?: the| my)? (home|history|settings|music|files)(?: page)? (?:in|of) (?:the |ewe )?browser",
              |c| Some(Call::new("browser_page", json!({"page": c.get(1).or(c.get(2))?.as_str()})))),
            r(r"(?:show|list|what are)(?: me)?(?: all)?(?: my| the)?(?: open)? (?:browser )?tabs(?: (?:are )?open)?|(?:which|what) tabs are open",
              |_| Some(Call::new("browser_tabs", json!({})))),
            // a tab: "mute this tab", "pin the github tab", "close the tab"
            r(r"(mute|unmute|pin|unpin|close)(?: this| the| that| my)? (?:(.+?) )?tab(?: in (?:the |ewe )?browser)?",
              |c| {
                  let words = tab_words(c.get(2))?;
                  let mut args = json!({"action": &c[1]});
                  if let Some(w) = words {
                      args["words"] = json!(w);
                  }
                  Some(Call::new("browser_tab_do", args))
              }),
            // bookmark the page in front
            r(r"bookmark (?:this|the|that|it)(?: page| site| website| tab)?|add (?:this|the|that)(?: page| site| website| tab)? to (?:my )?bookmarks|save (?:this|the) page (?:to|in) (?:my )?bookmarks",
              |_| Some(Call::new("browser_bookmark", json!({})))),
            // back, forward, reload, reopen ("go back" alone stays the music's)
            r(r"(?:go )?back (?:a|one) page|(?:go )?back(?: a page| one page)? in (?:the |ewe )?browser|browser back|(?:go to )?(?:the )?previous page|(?:go )?back to the (?:previous|last) page",
              |_| Some(Call::new("browser_nav", json!({"action": "back"})))),
            r(r"go forward(?: a page| one page)?(?: in (?:the |ewe )?browser)?|forward (?:a|one) page|browser forward",
              |_| Some(Call::new("browser_nav", json!({"action": "forward"})))),
            r(r"(?:reload|refresh)(?: this| the)? (?:page|tab|web ?page|site|website)(?: in (?:the |ewe )?browser)?|(?:reload|refresh) the browser",
              |_| Some(Call::new("browser_nav", json!({"action": "reload"})))),
            r(r"reopen(?: the| my| that)?(?: last)?(?: closed)? tab(?: i (?:just )?closed)?|(?:bring back|restore|undo close|undo closing)(?: the| my| that)?(?: last)?(?: closed)? tab(?: i (?:just )?closed)?|open(?: the)?(?: last)? closed tab(?: again)?",
              |_| Some(Call::new("browser_nav", json!({"action": "reopen"})))),
            // ewe browser's own workspaces (the desktop's are numbers on
            // Super+1…8, so a number needs the word "browser")
            r(r"(?:show|list)(?: me)?(?: my| the| all)? (?:ewe )?browser workspaces|(?:show|list)(?: me)?(?: my| the| all)? workspaces in (?:the |ewe )?browser|(?:what|which) workspaces (?:do i have|are there) in (?:the |ewe )?browser",
              |_| Some(Call::new("browser_workspace", json!({"action": "list"})))),
            r(r"(?:go to |switch to )?(?:the )?(next|previous|prev|last) (?:ewe )?browser workspace|(?:go to |switch to )?(?:the )?(next|previous|prev) workspace in (?:the |ewe )?browser",
              |c| {
                  let w = c.get(1).or(c.get(2))?.as_str();
                  Some(Call::new("browser_workspace", json!({"action": if w == "next" { "next" } else { "prev" }})))
              }),
            r(r"(?:new|create|make|add|open)(?: a)?(?: new)? (?:ewe )?browser workspace(?: (?:called|named) (.+))?|(?:create |make |add |open )?(?:a )?new workspace (?:called|named) (.+?)(?: in (?:the |ewe )?browser)?|(?:new|create|make|add)(?: a)?(?: new)? workspace in (?:the |ewe )?browser(?: (?:called|named) (.+))?",
              |c| {
                  let mut args = json!({"action": "new"});
                  if let Some(n) = c.get(1).or(c.get(2)).or(c.get(3)) {
                      args["name"] = json!(n.as_str().trim());
                  }
                  Some(Call::new("browser_workspace", args))
              }),
            // switching needs the word "browser": "switch to the firefox
            // workspace", "go to the second workspace" are the desktop's
            r(r"(?:switch|go|move|change|jump) to (?:the |my )?(?:ewe )?browser workspace (.+)|(?:switch|go|move|change|jump) to (?:the |my )?workspace (.+?) in (?:the |ewe )?browser|(?:switch|go|move|change|jump) to (?:the |my )?(.+?) workspace in (?:the |ewe )?browser|(?:switch|go|move|change|jump) to (?:the |my )?(.+?) (?:ewe )?browser workspace|open (?:the |my )?(.+?) workspace in (?:the |ewe )?browser",
              |c| {
                  let name = c.get(1).or(c.get(2)).or(c.get(3)).or(c.get(4)).or(c.get(5))?.as_str().trim();
                  if name.contains(" in ")
                      || matches!(name, "next" | "previous" | "prev" | "last" | "new" | "a new" | "other" | "empty"
                                  | "an empty" | "pen" | "the pen" | "special" | "scratchpad")
                  {
                      return None;
                  }
                  Some(Call::new("browser_workspace", json!({"action": "switch", "name": name})))
              }),
            // the browser's downloads ("show my downloads" stays the folder)
            r(r"(?:show|list)(?: me)?(?: my| the)? (?:recent |latest |browser )+downloads|(?:show|list|open)(?: me)?(?: my| the)? downloads (?:in|from|of) (?:the |ewe )?browser|what did i (?:just )?download(?: lately| recently| today)?|my (?:recent|latest) downloads|(?:show me )?what i (?:just )?downloaded(?: recently| lately| today)?",
              |_| Some(Call::new("browser_downloads", json!({})))),
            r(r"(?:go|switch|jump|take me)(?: back)? to(?: the| my)? (.+?) tab|(?:open|show|find)(?: the| my)? (.+?) tab",
              |c| {
                  let w = c.get(1).or(c.get(2))?.as_str().trim();
                  (!matches!(w, "new" | "a new" | "next" | "last" | "previous" | "other"))
                      .then(|| Call::new("browser_tab", json!({"words": w})))
              }),
            // ── the desktop's workspaces (a number of the display you are
            //    on) and windows ("this window" is the one in front) ──
            r(&format!(r"(?:(?:switch|go|change|jump|move|take me|flip)(?: over| back)? to |show(?: me)? |open )?(?:the )?(?:desktop |virtual desktop )?workspace(?: number)? {WS_NUM}|(?:(?:switch|go|change|jump|move|take me|flip)(?: over| back)? to |show(?: me)? )?(?:the )?{WS_ORD} (?:desktop |virtual desktop )?workspace|(?:(?:switch|go|change|jump|move)(?: over)? to )?desktop {WS_NUM}"),
              |c| {
                  let n = ws_number(c.get(1).or(c.get(2)).or(c.get(3))?.as_str())?;
                  Some(Call::new("workspace", json!({"number": n})))
              }),
            r(r"(?:(?:switch|go|change|jump|move|flip)(?: over)? to |show(?: me)? )?(?:the )?(next|previous|prev|last|other) (?:desktop |virtual desktop )?workspace|(?:(?:switch|go|change|jump|move) to )?(?:the )?workspace (?:on the |to the )?(right|left)|(next|previous|prev) desktop",
              |c| {
                  let w = c.get(1).or(c.get(2)).or(c.get(3))?.as_str();
                  let step = if matches!(w, "next" | "right") { "next" } else { "prev" };
                  Some(Call::new("workspace", json!({"step": step})))
              }),
            r(&format!(r"(?:move|send|put|throw|take) (?:this |the |my |current |active |that )?(?:window|app)(?: over)? (?:to|onto|on|into) (?:the )?(?:workspace |desktop )?{WS_NUM}|(?:move|send|put|throw|take) (?:this |the |my |current |active |that )?(?:window |app )?(?:to|onto|on|into) (?:the )?{WS_ORD} workspace|(?:move|send|put|throw|take) it to (?:workspace |desktop )?{WS_NUM}|(?:move|send|put|throw|take) (.+?) (?:to|onto|on|into) (?:the )?workspace {WS_NUM}"),
              |c| {
                  let (n, app) = match (c.get(1).or(c.get(2)).or(c.get(3)), c.get(4), c.get(5)) {
                      (Some(n), _, _) => (n.as_str(), None),
                      (None, Some(a), Some(n)) => (n.as_str(), Some(app_words(a.as_str())?)),
                      _ => return None,
                  };
                  let mut args = json!({"action": "move", "to": ws_number(n)?});
                  if let Some(a) = app {
                      args["app"] = json!(a);
                  }
                  Some(Call::new("window", args))
              }),
            r(r"(exit|leave|quit|stop|undo|turn off|get out of|no more|un-?) ?full ?screen(?: mode)?(?: (?:for )?(?:this|the|that) (?:window|app))?|(?:make |put |set |turn )?(?:this |the |my |current |that )?(?:window |app |it )?(?:to |into |in |go )?full ?screen(?: mode)?(?: (?:this|the|that) (?:window|app))?|(?:go|make it|make this|make this window|make the window) full ?screen|full ?screen (?:this|the|that|my) (?:window|app)",
              |c| Some(Call::new("window", json!({"action": "fullscreen", "on": c.get(1).is_none()})))),
            r(r"(un-?)?maximi[sz]e(?: (?:this|the|that|my|current) (?:window|app))?|make (?:this |the |that )?(?:window|app|it) (?:bigger|maximi[sz]ed|fill the screen)",
              |c| Some(Call::new("window", json!({"action": "maximize", "on": c.get(1).is_none()})))),
            r(r"(?:make |set |turn )?(?:this |the |that |my )?(?:window |app |it )?(?:float|floating)(?: (?:this|the|that|my) (?:window|app))?|float (?:this|the|that|my) (?:window|app)|make (?:this |the |that )?(?:window |app |it )?(?:float|floating)|(unfloat|tile|stop floating)(?: (?:this|the|that|my) (?:window|app))?|make (?:this |the |that )?(?:window |app |it )?(tiled|tile|not float(?:ing)?)",
              |c| Some(Call::new("window", json!({"action": "float", "on": c.get(1).is_none() && c.get(2).is_none()})))),
            r(r"(?:unminimi[sz]e|restore|bring back|show) (?:the |my )?(?:last )?(?:minimi[sz]ed|hidden) (?:window|app)|unminimi[sz]e(?: (?:the|my|that) (?:last )?(?:window|app))?|bring back (?:the |my )?(?:last )?(?:window|app) i (?:minimi[sz]ed|hid)",
              |_| Some(Call::new("window", json!({"action": "restore"})))),
            r(r"(?:minimi[sz]e|hide)(?: (?:this|the|that|my|current|active) (?:window|app))?|(?:minimi[sz]e|hide) (?:the )?window|minimi[sz]e it",
              |_| Some(Call::new("window", json!({"action": "minimize"})))),
            // closing asks first: the reply is a button (windows.rs)
            r(r"(?:close|quit|exit|shut) (?:this|the|that|my|current|active) (?:window|app)|(?:close|quit|exit) (?:the )?window|close it",
              |_| Some(Call::new("window", json!({"action": "close"})))),
            r(r"(?:force[- ]?(?:quit|close|stop|kill)|kill|terminate) (?:the )?(.+?)(?: process)?|(?:the |my )?(.+?) (?:is|has|seems|looks) (?:frozen|froze|hung|hanging|stuck|crashed|not responding|unresponsive|locked up|stopped responding)|(?:the |my )?(.+?) (?:isn'?t|is not|won'?t|will not|doesn'?t|does not|stopped) (?:responding|respond|close|quit|closing|quitting)",
              |c| {
                  let force = !c[0].starts_with("kill") && !c[0].starts_with("terminate");
                  let app = app_words(c.get(1).or(c.get(2)).or(c.get(3))?.as_str())?;
                  Some(Call::new("force_quit", json!({"app": app, "force": force})))
              }),
            r(r"(?:close|quit|exit) (?:the |my |all )?(.+?)(?: app| apps| window| windows)?",
              |c| {
                  let app = app_words(&c[1])?;
                  Some(Call::new("window", json!({"action": "close", "app": app})))
              }),
            // ── files and folders: the daemon's finder. "find" is the
            //    finder; "find … online", "search", "google" and "look up"
            //    are the web. A finder that finds nothing hands the words
            //    on as a question ──
            r(r"search (?:my |the |in my )?(?:files|folders|computer|home(?: folder)?|disk) for (.+)|(?:search|look) for ((?:the |a |my )?(?:files?|folders?|directory|directories) .+)",
              |c| Some(Call::new("find_files", json!({"query": c.get(1).or(c.get(2))?.as_str().trim()})))),
            // a site's own search: "search men's shoes on zara", "search
            // zara for men's shoes", "go to zara and search for shoes". The
            // site is short (site_of); "on the web", "online", "on my
            // desktop" keep their old meaning
            r(r"(?:search|look up|shop|shop for)(?: for)? (.+) on (.+)",
              |c| site_search(&c[1], &c[2])),
            // looser words: only a site people name, or a domain ("find
            // the report on dropbox", "search gifts for mum" stay theirs)
            r(r"(?:find|show me)(?: for)? (.+) on (.+)",
              |c| known_site_search(&c[1], &c[2])),
            r(r"search (.+?) for (.+)",
              |c| known_site_search(&c[2], &c[1])),
            r(r"(?:go to|open|visit) (.+?) and (?:search|look) (?:for )?(.+)",
              |c| site_search(&c[2], &c[1])),
            r(r"(?:find|locate)(?: me)? (.+)",
              |c| {
                  let q = c[1].trim();
                  let web = [" online", " on the web", " on the internet", " on google"]
                      .iter()
                      .any(|s| q.ends_with(s));
                  (!web).then(|| Call::new("find_files", json!({"query": q})))
              }),
            r(r"where(?:'s| is| are| did i (?:put|save)) (my .+|(?:the |a )?.+? (?:folder|directory|dir|file|files))",
              |c| Some(Call::new("find_files", json!({"query": c[1].trim()})))),
            r(r"(?:search|google|look up|find)(?: the web| online| on the web)?(?: for)? (.+?)(?: online| on the web| on the internet| on google)?",
              |c| Some(Call::new("web_search", json!({"query": c[1].trim()})))),
            r(r"play (?:some |the song |the album |music by |songs by )?(.+?)(?: on (?:youtube|youtube music|the browser))?",
              |c| Some(Call::new("play_music", json!({"query": c[1].trim()})))),
            // ── apps and folders: "open nemo", "open kitty in documents",
            //    "open documents in nemo", "open my downloads folder". The
            //    words are only words here: the daemon finds the folder, the
            //    shell finds the app, and either side may be the app ("in
            //    nemo" or "kitty in"). Words that name neither fall through
            //    to the knowledge pack and the model. Last, so every command
            //    above wins over a generic "open …" ──
            r(r"(?:open|launch|start|run)(?: a| an| the| new| my)? (.+?) (?:in|at|inside|from|with)(?: the| my)? (.+?)(?: folder| directory| dir)?",
              |c| Some(Call::new("launch_app", json!({"app": c[1].trim(), "path": c[2].trim()})))),
            r(r"(?:open|show|browse|go to)(?: the| my)? (.+?) (?:folder|directory|dir)",
              |c| Some(Call::new("open_folder", json!({"path": c[1].trim()})))),
            r(r"(?:open|show|browse|go to)(?: the| my)? (~\S*|/\S*|documents|docs|downloads|pictures|photos|music|videos|movies|desktop|home|templates|public|projects|screenshots|nextcloud)",
              |c| Some(Call::new("open_folder", json!({"path": c[1].trim()})))),
            r(r"(?:open|launch|start|run)(?: the| an?| my)? ([a-z0-9][a-z0-9 ._+-]{0,60}?)(?: app)?",
              |c| Some(Call::new("launch_app", json!({"app": c[1].trim()})))),
        ]
    })
}

/// "What is this page about?": a question about the page open in ewe
/// browser. Not a command: the chat reads the page and asks the model,
/// with the page's text marked untrusted (C4).
pub fn about_page(text: &str) -> bool {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"^(?:what(?:'s| is) (?:this|the|that) (?:web ?)?(?:page|site|article|tab) about|summari[sz]e (?:this|the|that|my) (?:web ?)?(?:page|site|article|tab)|(?:explain|tl;?dr)(?: me)? (?:this|the) (?:web ?)?(?:page|article)|what does (?:this|the) (?:web ?)?(?:page|article) say)\b",
        )
        .expect("about_page pattern")
    })
    .is_match(&clean(text))
}

/// The command these words ask for, if they plainly ask for one.
pub fn match_text(text: &str) -> Option<Call> {
    let t = clean(text);
    if t.is_empty() || t.len() > 300 {
        return None;
    }
    let find = |t: &str| {
        rules()
            .iter()
            .find_map(|r| r.re.captures(t).and_then(|c| (r.make)(&c)))
    };
    // a miss: once more with the command words' typos fixed ("opne
    // firefox", "volme 50"); free text is never touched (typo.rs)
    find(&t).or_else(|| crate::typo::correct(&t).and_then(|c| find(&c)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn m(t: &str) -> Option<(String, Value)> {
        match_text(t).map(|c| (c.command, c.args))
    }
    fn is(t: &str, cmd: &str, args: Value) {
        assert_eq!(m(t), Some((cmd.to_string(), args)), "{t:?}");
    }

    #[test]
    fn volume() {
        is("set volume to 60%", "volume_set", json!({"percent": 60}));
        is(
            "Set the volume to 60 percent.",
            "volume_set",
            json!({"percent": 60}),
        );
        is("volume 35", "volume_set", json!({"percent": 35}));
        is("volume up", "volume_change", json!({"delta": 10}));
        is("turn it down", "volume_change", json!({"delta": -10}));
        is("louder please", "volume_change", json!({"delta": 10}));
        is(
            "increase volume by 20%",
            "volume_change",
            json!({"delta": 20}),
        );
        is("mute", "mute", json!({"on": true}));
        is("unmute the sound", "mute", json!({"on": false}));
    }

    #[test]
    fn brightness() {
        is(
            "increase brightness",
            "brightness_change",
            json!({"delta": 10}),
        );
        is(
            "brightness down by 30%",
            "brightness_change",
            json!({"delta": -30}),
        );
        is("brighter", "brightness_change", json!({"delta": 10}));
        is("dim the screen", "brightness_change", json!({"delta": -10}));
        is(
            "set brightness to 70%",
            "brightness_set",
            json!({"percent": 70}),
        );
    }

    #[test]
    fn modes_and_media() {
        is("enter gamemode", "game_mode", json!({"on": true}));
        is("game mode off", "game_mode", json!({"on": false}));
        is("Exit game mode!", "game_mode", json!({"on": false}));
        is("turn on do not disturb", "dnd", json!({"on": true}));
        is("dnd off", "dnd", json!({"on": false}));
        is("turn off notifications", "dnd", json!({"on": true}));
        is("next song", "media", json!({"action": "next"}));
        is("pause", "media", json!({"action": "play_pause"}));
        is("previous track", "media", json!({"action": "previous"}));
        is("take a screenshot", "screenshot", json!({"mode": "region"}));
        is(
            "screenshot of the whole screen",
            "screenshot",
            json!({"mode": "full"}),
        );
        is("lock the screen", "lock", json!({}));
        is("open komble", "open", json!({"what": "komble"}));
        is("open settings", "open", json!({"what": "settings"}));
        // the apps' display names (ewe Settings, ewe Sync) say the same
        is("open ewe Settings", "open", json!({"what": "settings"}));
        is("open ewe Sync", "open", json!({"what": "sync"}));
    }

    #[test]
    fn apps_and_folders() {
        is("open nemo", "launch_app", json!({"app": "nemo"}));
        is(
            "launch Visual Studio Code",
            "launch_app",
            json!({"app": "visual studio code"}),
        );
        is("start steam", "launch_app", json!({"app": "steam"}));
        is(
            "open kitty in documents",
            "launch_app",
            json!({"app": "kitty", "path": "documents"}),
        );
        is(
            "open a terminal in ~/projects/ewe",
            "launch_app",
            json!({"app": "terminal", "path": "~/projects/ewe"}),
        );
        // either side may be the app: the daemon tells the folder from the app
        is(
            "open documents in nemo",
            "launch_app",
            json!({"app": "documents", "path": "nemo"}),
        );
        is(
            "open zed in my projects folder",
            "launch_app",
            json!({"app": "zed", "path": "projects"}),
        );
        is(
            "open my downloads folder",
            "open_folder",
            json!({"path": "downloads"}),
        );
        is(
            "show the documents",
            "open_folder",
            json!({"path": "documents"}),
        );
        is(
            "open ~/projects/ewe",
            "open_folder",
            json!({"path": "~/projects/ewe"}),
        );
        // the specific commands still win over a generic "open …"
        is("open kitty", "open", json!({"what": "terminal"}));
        is("open settings", "open", json!({"what": "settings"}));
        is(
            "open wifi",
            "open",
            json!({"what": "quicksettings", "page": "wifi"}),
        );
        is("start game mode", "game_mode", json!({"on": true}));
        is(
            "open https://archlinux.org",
            "open_url",
            json!({"url": "https://archlinux.org"}),
        );
    }

    #[test]
    fn places() {
        let qs = |tab: &str| json!({"what": "quicksettings", "page": tab});
        is("open wifi", "open", qs("wifi"));
        is("show the bluetooth settings", "open", qs("bt"));
        is("open quick settings", "open", qs("home"));
        is("show notifications", "open", qs("notifs"));
        is(
            "check for updates",
            "open",
            json!({"what": "komble", "page": "updates"}),
        );
        is(
            "open plugins",
            "open",
            json!({"what": "komble", "page": "plugins"}),
        );
        is("open the overview", "open", json!({"what": "overview"}));
        is(
            "open display settings",
            "open",
            json!({"what": "settings", "page": "displays"}),
        );
        is(
            "go to keyboard settings",
            "open",
            json!({"what": "settings", "page": "input"}),
        );
        is(
            "open wallpaper settings",
            "open",
            json!({"what": "settings", "page": "wallpaper"}),
        );
        // ewe Settings 0.22 has no Account page: the account is ewe Sync's
        is("open account settings", "open", json!({"what": "sync"}));
        is(
            "open networking settings",
            "open",
            json!({"what": "settings", "page": "network"}),
        );
        // every page a rule makes is one `open` accepts
        for t in [
            "open animation settings",
            "open window rules settings",
            "open screensaver settings",
            "open battery settings",
            "open default apps settings",
            "open startup settings",
            "open shortcuts settings",
            "open language settings",
            "open account settings",
            "open networking settings",
            "open appearance settings",
            "open accessibility settings",
            "open layout settings",
            "open user settings",
            "open time settings",
        ] {
            let c = match_text(t).unwrap_or_else(|| panic!("{t:?}"));
            assert_eq!(crate::commands::validate(&c), Ok(()), "{t:?}");
        }
    }

    #[test]
    fn the_browser() {
        is(
            "search for hyprland lua config",
            "web_search",
            json!({"query": "hyprland lua config"}),
        );
        is(
            "google arch linux news",
            "web_search",
            json!({"query": "arch linux news"}),
        );
        is(
            "play massive attack teardrop",
            "play_music",
            json!({"query": "massive attack teardrop"}),
        );
        is(
            "play some jazz on youtube",
            "play_music",
            json!({"query": "jazz"}),
        );
        is(
            "open https://archlinux.org/news/",
            "open_url",
            json!({"url": "https://archlinux.org/news/"}),
        );
    }

    #[test]
    fn the_browser_s_own_powers() {
        is(
            "open github.com",
            "open_url",
            json!({"url": "https://github.com"}),
        );
        is(
            "go to www.archlinux.org/news",
            "open_url",
            json!({"url": "https://www.archlinux.org/news"}),
        );
        is(
            "find hyprland docs online",
            "web_search",
            json!({"query": "hyprland docs"}),
        );
        is(
            "look up sheep breeds",
            "web_search",
            json!({"query": "sheep breeds"}),
        );
        is(
            "find tax in my history",
            "browser_history",
            json!({"query": "tax"}),
        );
        is(
            "that site about sheep breeds",
            "browser_history",
            json!({"query": "sheep breeds"}),
        );
        is("show my history", "browser_history", json!({"query": ""}));
        is("show my tabs", "browser_tabs", json!({}));
        is("which tabs are open?", "browser_tabs", json!({}));
        is(
            "go to the YouTube tab",
            "browser_tab",
            json!({"words": "youtube"}),
        );
        is(
            "open the youtube tab",
            "browser_tab",
            json!({"words": "youtube"}),
        );
        assert_ne!(m("open a new tab").map(|c| c.0), Some("browser_tab".into()));
        // Quick settings' own tabs still win
        is(
            "go to the calendar tab",
            "open",
            json!({"what": "quicksettings", "page": "cal"}),
        );
        assert!(about_page("What is this page about?"));
        assert!(about_page("summarize this page in three points"));
        assert!(about_page("tldr this article"));
        assert!(!about_page("what is ewe about"));
        assert_eq!(m("what is this page about"), None);
    }

    #[test]
    fn files_and_folders() {
        is(
            "find my resume",
            "find_files",
            json!({"query": "my resume"}),
        );
        is(
            "Where is the ewe-safe folder?",
            "find_files",
            json!({"query": "the ewe-safe folder"}),
        );
        is(
            "find pdfs about tax",
            "find_files",
            json!({"query": "pdfs about tax"}),
        );
        is(
            "search my files for invoice",
            "find_files",
            json!({"query": "invoice"}),
        );
        is(
            "search for folders named photos",
            "find_files",
            json!({"query": "folders named photos"}),
        );
        is(
            "where's my passport scan",
            "find_files",
            json!({"query": "my passport scan"}),
        );
        // opening a folder by name is still open_folder (it finds it first)
        is(
            "open the ewe-safe folder",
            "open_folder",
            json!({"path": "ewe-safe"}),
        );
    }

    #[test]
    fn safe() {
        let copy = |q: &str, f: &str| json!({"query": q, "field": f});
        is(
            "copy my GitHub password",
            "safe_copy",
            copy("github", "password"),
        );
        is(
            "copy the password for github",
            "safe_copy",
            copy("github", "password"),
        );
        is(
            "copy my github username",
            "safe_copy",
            copy("github", "username"),
        );
        is("copy my github code", "safe_copy", copy("github", "otp"));
        is(
            "copy the one-time code for my bank",
            "safe_copy",
            copy("bank", "otp"),
        );
        is(
            "copy discord's 2fa code from safe",
            "safe_copy",
            copy("discord", "otp"),
        );
        assert_eq!(m("copy my password"), None, "no site named");
        is(
            "log me in to GitHub",
            "safe_login",
            json!({"query": "github"}),
        );
        is("sign in to my bank", "safe_login", json!({"query": "bank"}));
        is("lock my safe", "safe_lock", json!({}));
        is("unlock my safe", "safe_unlock", json!({}));
        is("lock ewe Safe", "safe_lock", json!({}));
        is("unlock ewe safe", "safe_unlock", json!({}));
        is("is my safe synced?", "safe_status", json!({}));
        is("how many passwords do I have", "safe_status", json!({}));
        is("how many logins are in my safe", "safe_status", json!({}));
        // the screen is still "lock"
        is("lock", "lock", json!({}));
        is("lock the screen", "lock", json!({}));
        for (t, _) in [("copy my GitHub password", ()), ("log me in to github", ())] {
            let c = match_text(t).unwrap();
            assert_eq!(crate::commands::validate(&c), Ok(()), "{t:?}");
        }
    }

    #[test]
    fn a_site_s_own_search() {
        let site = |q: &str, s: &str| json!({"query": q, "site": s});
        is(
            "search men's shoes on zara",
            "web_search",
            site("men's shoes", "zara"),
        );
        is(
            "Find a kettle on Amazon.",
            "web_search",
            site("a kettle", "amazon"),
        );
        is(
            "look up sheep on wikipedia",
            "web_search",
            site("sheep", "wikipedia"),
        );
        is(
            "search zara for men's shoes",
            "web_search",
            site("men's shoes", "zara"),
        );
        is(
            "shop for linen shirts on h&m",
            "web_search",
            site("linen shirts", "h&m"),
        );
        is(
            "show me cheap flights on google flights",
            "web_search",
            site("cheap flights", "google flights"),
        );
        is(
            "go to zara and search for men's shoes",
            "web_search",
            site("men's shoes", "zara"),
        );
        is(
            "search used cars on myauto.ge",
            "web_search",
            site("used cars", "myauto.ge"),
        );
        is(
            "search books on rust on the amazon website",
            "web_search",
            site("books on rust", "amazon"),
        );
        is(
            "search for hyprland plugins on github",
            "web_search",
            site("hyprland plugins", "github"),
        );
        // a long "site" is not one: the whole phrase is searched
        is(
            "search hats on a cold winter day",
            "web_search",
            json!({"query": "hats on a cold winter day"}),
        );
        for t in [
            "search men's shoes on zara",
            "go to zara.com and search for shoes",
        ] {
            let c = match_text(t).unwrap();
            assert_eq!(crate::commands::validate(&c), Ok(()), "{t:?}");
        }
    }

    #[test]
    fn site_words_steal_nothing() {
        // the plan's list: these keep their meaning
        is(
            "search my files for invoice",
            "find_files",
            json!({"query": "invoice"}),
        );
        is(
            "find tax in my history",
            "browser_history",
            json!({"query": "tax"}),
        );
        is(
            "search sheep breeds online",
            "web_search",
            json!({"query": "sheep breeds"}),
        );
        is(
            "search the web for sheep breeds",
            "web_search",
            json!({"query": "sheep breeds"}),
        );
        is(
            "search sheep breeds on the web",
            "web_search",
            json!({"query": "sheep breeds"}),
        );
        is(
            "find hyprland docs on google",
            "web_search",
            json!({"query": "hyprland docs"}),
        );
        assert_eq!(m("look for the youtube tab"), None);
        // "search Y for X", "find/show me X on Y": only a site people name
        // or a domain; anything else is the whole phrase, or the finder's
        is(
            "search gifts for mum",
            "web_search",
            json!({"query": "gifts for mum"}),
        );
        is(
            "search flights for tomorrow",
            "web_search",
            json!({"query": "flights for tomorrow"}),
        );
        is(
            "search weather for paris",
            "web_search",
            json!({"query": "weather for paris"}),
        );
        is(
            "find report on dropbox",
            "find_files",
            json!({"query": "report on dropbox"}),
        );
        assert!(
            m("show me errors on server").is_none_or(|c| c.1.get("site").is_none()),
            "not a site"
        );
        is(
            "search youtube for sheepdogs",
            "web_search",
            json!({"query": "sheepdogs", "site": "youtube"}),
        );
        is(
            "search zara.com for linen",
            "web_search",
            json!({"query": "linen", "site": "zara.com"}),
        );
        // "for …" and a day are not sites
        is(
            "search for cheap flights for paris",
            "web_search",
            json!({"query": "cheap flights for paris"}),
        );
        is(
            "find concerts on friday",
            "find_files",
            json!({"query": "concerts on friday"}),
        );
        // a place on this computer is the finder's
        is(
            "find the report on my desktop",
            "find_files",
            json!({"query": "the report on my desktop"}),
        );
        is(
            "search for folders named photos",
            "find_files",
            json!({"query": "folders named photos"}),
        );
        is(
            "play some jazz on youtube",
            "play_music",
            json!({"query": "jazz"}),
        );
    }

    #[test]
    fn the_browser_s_new_powers() {
        is("bookmark this page", "browser_bookmark", json!({}));
        is(
            "add this site to my bookmarks",
            "browser_bookmark",
            json!({}),
        );
        is(
            "find recipes in my bookmarks",
            "browser_bookmarks",
            json!({"query": "recipes"}),
        );
        is(
            "show my bookmarks",
            "browser_bookmarks",
            json!({"query": ""}),
        );
        is("go back a page", "browser_nav", json!({"action": "back"}));
        is("previous page", "browser_nav", json!({"action": "back"}));
        is("go forward", "browser_nav", json!({"action": "forward"}));
        is(
            "reload the page",
            "browser_nav",
            json!({"action": "reload"}),
        );
        is(
            "refresh this tab",
            "browser_nav",
            json!({"action": "reload"}),
        );
        is(
            "reopen the closed tab",
            "browser_nav",
            json!({"action": "reopen"}),
        );
        is(
            "bring back the tab i just closed",
            "browser_nav",
            json!({"action": "reopen"}),
        );
        // "go back" alone is still the music's
        is("go back", "media", json!({"action": "previous"}));
        is("mute this tab", "browser_tab_do", json!({"action": "mute"}));
        is(
            "unmute the youtube tab",
            "browser_tab_do",
            json!({"action": "unmute", "words": "youtube"}),
        );
        is(
            "pin the github tab",
            "browser_tab_do",
            json!({"action": "pin", "words": "github"}),
        );
        is(
            "close this tab",
            "browser_tab_do",
            json!({"action": "close"}),
        );
        assert_eq!(m("close the other tab"), None);
        // the speakers stay "mute"
        is("mute", "mute", json!({"on": true}));
        is(
            "switch to my work workspace in the browser",
            "browser_workspace",
            json!({"action": "switch", "name": "work"}),
        );
        is(
            "go to the travel browser workspace",
            "browser_workspace",
            json!({"action": "switch", "name": "travel"}),
        );
        // without "browser" a workspace is the desktop's
        for t in [
            "switch to my work workspace",
            "go to the second workspace",
            "switch to the firefox workspace",
            "open the music workspace",
            "open github.com in workspace 2",
        ] {
            assert_ne!(
                m(t).map(|c| c.0),
                Some("browser_workspace".to_string()),
                "{t:?}"
            );
            assert!(m(t).is_none_or(|c| c.1.get("workspace").is_none()), "{t:?}");
        }
        is(
            "go to browser workspace 2",
            "browser_workspace",
            json!({"action": "switch", "name": "2"}),
        );
        is(
            "next browser workspace",
            "browser_workspace",
            json!({"action": "next"}),
        );
        is(
            "previous workspace in the browser",
            "browser_workspace",
            json!({"action": "prev"}),
        );
        is(
            "new workspace called travel",
            "browser_workspace",
            json!({"action": "new", "name": "travel"}),
        );
        is(
            "new browser workspace",
            "browser_workspace",
            json!({"action": "new"}),
        );
        is(
            "show my browser workspaces",
            "browser_workspace",
            json!({"action": "list"}),
        );
        // the desktop's workspaces are not the browser's
        is("go to workspace 3", "workspace", json!({"number": 3}));
        is(
            "switch to the next workspace",
            "workspace",
            json!({"step": "next"}),
        );
        assert_eq!(m("go to the pen workspace"), None);
        is(
            "open github.com in my work workspace",
            "open_url",
            json!({"url": "https://github.com", "workspace": "work"}),
        );
        is("what did i download", "browser_downloads", json!({}));
        is("show my recent downloads", "browser_downloads", json!({}));
        // the Downloads folder keeps its words
        is(
            "show my downloads",
            "open_folder",
            json!({"path": "downloads"}),
        );
        is(
            "open the browser's settings",
            "browser_page",
            json!({"page": "settings"}),
        );
        is(
            "open history in the browser",
            "browser_page",
            json!({"page": "history"}),
        );
        is(
            "show my browser history",
            "browser_history",
            json!({"query": ""}),
        );
        is("open settings", "open", json!({"what": "settings"}));
        for t in [
            "pin the github tab",
            "new workspace called travel",
            "open github.com in my work workspace",
            "open the browser's music page",
        ] {
            let c = match_text(t).unwrap_or_else(|| panic!("{t:?}"));
            assert_eq!(crate::commands::validate(&c), Ok(()), "{t:?}");
        }
    }

    #[test]
    fn the_desktop_s_new_powers() {
        is("turn off wifi", "wifi", json!({"on": false}));
        is("wi-fi on", "wifi", json!({"on": true}));
        is("enable bluetooth", "bluetooth", json!({"on": true}));
        is("turn bluetooth off", "bluetooth", json!({"on": false}));
        is(
            "connect my airpods",
            "bluetooth_device",
            json!({"name": "airpods", "connect": true}),
        );
        is(
            "disconnect the sony headphones",
            "bluetooth_device",
            json!({"name": "sony headphones", "connect": false}),
        );
        assert_eq!(m("connect to wifi"), None);
        assert_eq!(m("connect to the vpn"), None);
        is("airplane mode on", "airplane_mode", json!({"on": true}));
        is(
            "turn off flight mode",
            "airplane_mode",
            json!({"on": false}),
        );
        is("airplane mode", "airplane_mode", json!({"on": true}));
        is("keep the screen awake", "keep_awake", json!({"on": true}));
        is("stay awake", "keep_awake", json!({"on": true}));
        is(
            "stop keeping the screen awake",
            "keep_awake",
            json!({"on": false}),
        );
        is(
            "let the screen sleep again",
            "keep_awake",
            json!({"on": false}),
        );
        is("caffeine off", "keep_awake", json!({"on": false}));
        is("what is 12*7", "calc", json!({"expr": "12*7"}));
        is("15% of 80", "calc", json!({"expr": "15% of 80"}));
        is("how much is 80 + 15%", "calc", json!({"expr": "80 + 15%"}));
        is("12 x 7 =", "calc", json!({"expr": "12 x 7"}));
        // a phone number or a date is not a sum, unless asked as one
        for t in ["555-1234", "2026-10-18", "1-2"] {
            assert_ne!(m(t).map(|c| c.0), Some("calc".to_string()), "{t:?}");
        }
        is("what is 12-3", "calc", json!({"expr": "12-3"}));
        is("12 - 3", "calc", json!({"expr": "12 - 3"}));
        is(
            "install obs studio",
            "open",
            json!({"what": "komble", "page": "search=obs studio"}),
        );
        is(
            "install the steam app",
            "open",
            json!({"what": "komble", "page": "search=steam"}),
        );
        is(
            "install updates",
            "open",
            json!({"what": "komble", "page": "updates"}),
        );
        is(
            "switch to light mode",
            "theme_mode",
            json!({"mode": "light"}),
        );
        is("dark mode", "theme_mode", json!({"mode": "dark"}));
        is("turn off dark mode", "theme_mode", json!({"mode": "light"}));
        is("suspend", "power", json!({"action": "suspend"}));
        is(
            "put the computer to sleep",
            "power",
            json!({"action": "suspend"}),
        );
        is(
            "restart the computer",
            "power",
            json!({"action": "restart"}),
        );
        is("shut down", "power", json!({"action": "shutdown"}));
        is(
            "turn off the laptop",
            "power",
            json!({"action": "shutdown"}),
        );
        is("log out", "power", json!({"action": "logout"}));
        assert_eq!(m("restart the shell"), None);
        assert_eq!(m("sign out of nextcloud"), None);
        is("empty the trash", "empty_trash", json!({}));
        // "make it light" stays a model's (the chat test's ACTION)
        assert_eq!(m("make it light"), None);
        // still the lock screen and the music
        is("lock", "lock", json!({}));
        is("stop", "media", json!({"action": "play_pause"}));
    }

    #[test]
    fn timers_and_reminders() {
        is(
            "set a timer for 5 minutes",
            "timer",
            json!({"seconds": 300}),
        );
        is("timer 90 seconds", "timer", json!({"seconds": 90}));
        is("set a 10 minute timer", "timer", json!({"seconds": 600}));
        is(
            "start a timer for 1 hour 30 minutes",
            "timer",
            json!({"seconds": 5400}),
        );
        is(
            "set a timer for an hour and a half",
            "timer",
            json!({"seconds": 5400}),
        );
        is(
            "set a timer for half an hour called pasta",
            "timer",
            json!({"seconds": 1800, "text": "pasta"}),
        );
        is(
            "remind me in 10 minutes to call mum",
            "timer",
            json!({"seconds": 600, "text": "call mum", "reminder": true}),
        );
        is(
            "remind me to take the pie out of the oven in 20 min",
            "timer",
            json!({"seconds": 1200, "text": "take the pie out of the oven", "reminder": true}),
        );
        is(
            "remind me at 5pm to call the bank",
            "timer",
            json!({"at": "17:00", "text": "call the bank", "reminder": true}),
        );
        is(
            "remind me to stretch at 17:30",
            "timer",
            json!({"at": "17:30", "text": "stretch", "reminder": true}),
        );
        // no words of its own: a reminder, never a made-up "reminder" text
        is(
            "remind me in 5 minutes",
            "timer",
            json!({"seconds": 300, "reminder": true}),
        );
        is("cancel the timer", "timer_cancel", json!({}));
        is("cancel my reminders", "timer_cancel", json!({}));
        is("show my timers", "timers", json!({}));
        is("how long is left on my timer", "timers", json!({}));
        // a time that could be morning or evening is the model's to ask
        assert_eq!(m("remind me at 5 to call mum"), None);
        assert_eq!(m("remind me to call mum"), None);
        assert_eq!(duration_secs("10min"), Some(600));
        assert_eq!(duration_secs("2h"), Some(7200));
        assert_eq!(duration_secs("1.5 hours"), Some(5400));
        assert_eq!(duration_secs("ten years"), None);
        assert_eq!(clock_of("5:30 pm").as_deref(), Some("17:30"));
        assert_eq!(clock_of("12am").as_deref(), Some("00:00"));
        assert_eq!(clock_of("noon").as_deref(), Some("12:00"));
        assert_eq!(clock_of("09:15").as_deref(), Some("09:15"));
        assert_eq!(clock_of("5:30"), None);
        assert_eq!(clock_of("13pm"), None);
    }

    #[test]
    fn the_owner_s_everyday_phrases() {
        // the ones that used to miss and wait 7 s for a model (2026-11-08)
        is("opne firefox", "launch_app", json!({"app": "firefox"}));
        is("volme 50", "volume_set", json!({"percent": 50}));
        is("make it quieter", "volume_change", json!({"delta": -10}));
        is("what time is it", "clock", json!({}));
        is("what's the date today", "clock", json!({}));
        is(
            "how much battery do i have",
            "system_info",
            json!({"what": "battery"}),
        );
        is(
            "how much ram is free",
            "system_info",
            json!({"what": "memory"}),
        );
        is(
            "what's using my cpu",
            "system_info",
            json!({"what": "cpu", "apps": true}),
        );
        is(
            "how long has my computer been on",
            "system_info",
            json!({"what": "uptime"}),
        );
        is(
            "convert 10 km to miles",
            "convert",
            json!({"value": 10, "from": "km", "to": "mi"}),
        );
        is(
            "20 c in f",
            "convert",
            json!({"value": 20, "from": "degC", "to": "degF"}),
        );
        is(
            "how many days until christmas",
            "date_math",
            json!({"op": "until", "date": "christmas"}),
        );
        is(
            "what time is it in tokyo",
            "clock",
            json!({"place": "tokyo"}),
        );
        is("switch to workspace 3", "workspace", json!({"number": 3}));
        is("close this window", "window", json!({"action": "close"}));
        is(
            "move this window to workspace 2",
            "window",
            json!({"action": "move", "to": 2}),
        );
        is(
            "make this window fullscreen",
            "window",
            json!({"action": "fullscreen", "on": true}),
        );
        is("what's my ip", "system_info", json!({"what": "network"}));
        is(
            "am i connected to the internet",
            "system_info",
            json!({"what": "network"}),
        );
        is(
            "how much disk space is left",
            "system_info",
            json!({"what": "disk"}),
        );
        is("what's eating my disk", "disk_usage", json!({}));
        is(
            "kill firefox",
            "force_quit",
            json!({"app": "firefox", "force": false}),
        );
        is(
            "firefox is frozen",
            "force_quit",
            json!({"app": "firefox", "force": true}),
        );
        is(
            "why is my laptop slow",
            "system_info",
            json!({"what": "overview"}),
        );
        is(
            "remember that my dentist is on friday",
            "remember",
            json!({"text": "my dentist is on friday"}),
        );
        is("what do you remember", "memories", json!({}));
        is("sqrt 144", "calc", json!({"expr": "sqrt 144"}));
    }

    #[test]
    fn clock_dates_and_units() {
        for t in [
            "what's the time",
            "whats the time",
            "time",
            "what is the date",
            "today's date",
            "what day is it",
            "what day is it today",
        ] {
            is(t, "clock", json!({}));
        }
        is(
            "what time is it in new york",
            "clock",
            json!({"place": "new york"}),
        );
        is("time in tokyo", "clock", json!({"place": "tokyo"}));
        is("what's the time in the uk", "clock", json!({"place": "uk"}));
        is(
            "days until 25 december",
            "date_math",
            json!({"op": "until", "date": "25 december"}),
        );
        is(
            "how long until new year",
            "date_math",
            json!({"op": "until", "date": "new year"}),
        );
        is(
            "when is easter",
            "date_math",
            json!({"op": "until", "date": "easter"}),
        );
        is(
            "how many days since 1 january",
            "date_math",
            json!({"op": "since", "date": "1 january"}),
        );
        is(
            "what day is 25 december",
            "date_math",
            json!({"op": "weekday", "date": "25 december"}),
        );
        is(
            "what day of the week is 2026-12-25",
            "date_math",
            json!({"op": "weekday", "date": "2026-12-25"}),
        );
        is(
            "what day is tomorrow",
            "date_math",
            json!({"op": "weekday", "date": "tomorrow"}),
        );
        is(
            "what's the date in 30 days",
            "date_math",
            json!({"op": "offset", "days": 30}),
        );
        is(
            "what's the date in 2 weeks",
            "date_math",
            json!({"op": "offset", "days": 14}),
        );
        is(
            "what's the date in a week",
            "date_math",
            json!({"op": "offset", "days": 7}),
        );
        is(
            "what was the date 10 days ago",
            "date_math",
            json!({"op": "offset", "days": -10}),
        );
        is(
            "what's the date tomorrow",
            "date_math",
            json!({"op": "offset", "days": 1}),
        );
        is(
            "10 km in miles",
            "convert",
            json!({"value": 10, "from": "km", "to": "mi"}),
        );
        is(
            "20°c to fahrenheit",
            "convert",
            json!({"value": 20, "from": "degC", "to": "degF"}),
        );
        is(
            "5 gb in mb",
            "convert",
            json!({"value": 5, "from": "gb", "to": "mb"}),
        );
        is(
            "3 feet in cm",
            "convert",
            json!({"value": 3, "from": "ft", "to": "cm"}),
        );
        is(
            "100 km/h in mph",
            "convert",
            json!({"value": 100, "from": "km/h", "to": "mph"}),
        );
        is(
            "70 kg in pounds",
            "convert",
            json!({"value": 70, "from": "kg", "to": "lb"}),
        );
        is(
            "2 cups in ml",
            "convert",
            json!({"value": 2, "from": "cup", "to": "ml"}),
        );
        is(
            "1 hour in seconds",
            "convert",
            json!({"value": 1, "from": "h", "to": "s"}),
        );
        is(
            "1.5 l in cups",
            "convert",
            json!({"value": 1.5, "from": "l", "to": "cup"}),
        );
        // a currency needs the network: the model's, never a guess
        assert_eq!(m("100 usd in eur"), None);
        is(
            "what is the square root of 144",
            "calc",
            json!({"expr": "the square root of 144"}),
        );
        is("12 squared", "calc", json!({"expr": "12 squared"}));
        // every call validates
        for t in [
            "how many days until christmas",
            "what's the date in 30 days",
            "20 c in f",
            "time in tokyo",
        ] {
            let c = match_text(t).unwrap_or_else(|| panic!("{t:?}"));
            assert_eq!(crate::commands::validate(&c), Ok(()), "{t:?}");
        }
    }

    #[test]
    fn this_computer() {
        for (t, what) in [
            ("battery", "battery"),
            ("is my laptop charging", "battery"),
            ("how much memory am i using", "memory"),
            ("ram usage", "memory"),
            ("cpu usage", "cpu"),
            ("uptime", "uptime"),
            ("is the internet working", "network"),
            ("what is my ip address", "network"),
            ("how full is my disk", "disk"),
            ("how much storage do i have left", "disk"),
            ("is my computer ok", "overview"),
            ("my computer is so slow", "overview"),
            ("why is everything so slow", "overview"),
        ] {
            assert_eq!(
                m(t).map(|c| c.1["what"].clone()),
                Some(json!(what)),
                "{t:?}"
            );
        }
        is(
            "what's using my memory",
            "system_info",
            json!({"what": "memory", "apps": true}),
        );
        is(
            "which apps use the most ram",
            "system_info",
            json!({"what": "memory", "apps": true}),
        );
        is(
            "what is taking up all my disk space",
            "disk_usage",
            json!({}),
        );
        is("free up space", "disk_usage", json!({}));
    }

    #[test]
    fn windows_and_workspaces() {
        is("go to workspace two", "workspace", json!({"number": 2}));
        is("workspace 4", "workspace", json!({"number": 4}));
        is(
            "go to the third workspace",
            "workspace",
            json!({"number": 3}),
        );
        is("previous workspace", "workspace", json!({"step": "prev"}));
        is("next workspace", "workspace", json!({"step": "next"}));
        assert_eq!(m("go to workspace 11"), None);
        is(
            "move it to workspace 5",
            "window",
            json!({"action": "move", "to": 5}),
        );
        is(
            "send this window to the second workspace",
            "window",
            json!({"action": "move", "to": 2}),
        );
        is(
            "move firefox to workspace 2",
            "window",
            json!({"action": "move", "to": 2, "app": "firefox"}),
        );
        is(
            "fullscreen",
            "window",
            json!({"action": "fullscreen", "on": true}),
        );
        is(
            "exit fullscreen",
            "window",
            json!({"action": "fullscreen", "on": false}),
        );
        is(
            "maximise this window",
            "window",
            json!({"action": "maximize", "on": true}),
        );
        is(
            "float this window",
            "window",
            json!({"action": "float", "on": true}),
        );
        is(
            "tile this window",
            "window",
            json!({"action": "float", "on": false}),
        );
        is(
            "minimise this window",
            "window",
            json!({"action": "minimize"}),
        );
        is(
            "bring back the minimized window",
            "window",
            json!({"action": "restore"}),
        );
        is(
            "close firefox",
            "window",
            json!({"action": "close", "app": "firefox"}),
        );
        is(
            "quit firefox",
            "window",
            json!({"action": "close", "app": "firefox"}),
        );
        is(
            "force quit firefox",
            "force_quit",
            json!({"app": "firefox", "force": true}),
        );
        is(
            "firefox is not responding",
            "force_quit",
            json!({"app": "firefox", "force": true}),
        );
        is(
            "steam won't close",
            "force_quit",
            json!({"app": "steam", "force": true}),
        );
        // the browser's tabs, Safe and the modes keep their words
        is(
            "close this tab",
            "browser_tab_do",
            json!({"action": "close"}),
        );
        is("close my safe", "safe_lock", json!({}));
        is("quit game mode", "game_mode", json!({"on": false}));
        assert_eq!(m("kill -9 1234"), None);
        assert_eq!(m("the computer is frozen"), None);
        for t in [
            "close firefox",
            "move firefox to workspace 2",
            "kill steam",
            "go to workspace 3",
        ] {
            let c = match_text(t).unwrap_or_else(|| panic!("{t:?}"));
            assert_eq!(crate::commands::validate(&c), Ok(()), "{t:?}");
        }
    }

    #[test]
    fn remembering_and_forgetting() {
        is(
            "remember: my locker is 42",
            "remember",
            json!({"text": "my locker is 42"}),
        );
        is(
            "remember I like dark mode",
            "remember",
            json!({"text": "i like dark mode"}),
        );
        is(
            "remember to call mum",
            "remember",
            json!({"text": "to call mum"}),
        );
        // with a time it is a reminder; an unclear time is the model's
        is(
            "remember to call mum at 5pm",
            "timer",
            json!({"at": "17:00", "text": "call mum", "reminder": true}),
        );
        is(
            "don't forget to stretch in 20 minutes",
            "timer",
            json!({"seconds": 1200, "text": "stretch", "reminder": true}),
        );
        assert_eq!(m("remember to call mum at 5"), None);
        // "remind me" stays a timer
        is(
            "remind me in 10 minutes to call mum",
            "timer",
            json!({"seconds": 600, "text": "call mum", "reminder": true}),
        );
        is("what do you know about me", "memories", json!({}));
        is(
            "what do you remember about my dentist",
            "memories",
            json!({"query": "my dentist"}),
        );
        is(
            "forget about my dentist",
            "forget",
            json!({"words": "my dentist"}),
        );
        is(
            "forget that my dentist is on friday",
            "forget",
            json!({"words": "my dentist is on friday"}),
        );
        is("forget everything", "forget", json!({"all": true}));
        assert_eq!(m("forget it"), None);
    }

    #[test]
    fn typos_get_a_second_try() {
        is("brigthness up", "brightness_change", json!({"delta": 10}));
        is("swich to worksapce 3", "workspace", json!({"number": 3}));
        is(
            "set a timer for 5 minuets",
            "timer",
            json!({"seconds": 300}),
        );
        // ambiguous, or a word it should not guess at: still a miss
        assert_eq!(m("mote"), None);
        assert_eq!(m("lick"), None);
        // an app's name is never "corrected"
        is("opne fierfox", "launch_app", json!({"app": "fierfox"}));
        is("make it louder", "volume_change", json!({"delta": 10}));
        is("it's too loud", "volume_change", json!({"delta": -10}));
    }

    #[test]
    fn questions_are_not_commands() {
        for q in [
            "what is the volume?",
            "tell me a joke",
            "",
            "explain game mode",
            "how do I free up space",
            "the disk is full",
            "what do you know about hyprland",
            "when is my dentist appointment",
            "how long to boil an egg",
            "what time zone am I in",
            "my laptop is frozen",
            "the desktop is frozen",
            "forget it",
            "where is the night light",
            "how do I find a file",
            "what is my github password",
        ] {
            assert_eq!(m(q), None, "{q:?}");
        }
    }
}
