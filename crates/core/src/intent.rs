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
    let t = text.trim().to_lowercase();
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
            r(r"(?:lock|close) (?:my |the )?safe",
              |_| Some(Call::new("safe_lock", json!({})))),
            r(r"unlock (?:my |the )?safe",
              |_| Some(Call::new("safe_unlock", json!({})))),
            r(r"is (?:my |the )?safe (?:synced|in sync|up to date|locked|unlocked|open)|(?:my |the )?safe(?:'s)? status|how many (?:passwords|logins|items|accounts) (?:do i have|(?:are )?(?:there )?in (?:my |the )?safe|does (?:my |the )?safe (?:have|hold))(?: in (?:my |the )?safe)?",
              |_| Some(Call::new("safe_status", json!({})))),
            r(r"lock(?: (?:the )?(?:screen|computer|pc|laptop|session))?",
              |_| Some(Call::new("lock", json!({})))),
            // ── open an ewe app ──
            r(r"(?:open|launch|show)(?: the)? (settings|komble|store|app store|software|sync|ewe-sync|ewe sync|terminal|kitty|files|file manager)",
              |c| {
                  let what = match &c[1] {
                      "settings" => "settings",
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
                      "user" | "account" => "user",
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
    rules()
        .iter()
        .find_map(|r| r.re.captures(&t).and_then(|c| (r.make)(&c)))
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
        assert_eq!(m("go to workspace 3"), None);
        assert_eq!(m("switch to the next workspace"), None);
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
    fn questions_are_not_commands() {
        for q in [
            "what is the volume?",
            "how much disk space do I have",
            "why is my laptop slow",
            "tell me a joke",
            "",
            "what time is it in tokyo",
            "explain game mode",
            "where is the night light",
            "how do I find a file",
            "what is my github password",
        ] {
            assert_eq!(m(q), None, "{q:?}");
        }
    }
}
