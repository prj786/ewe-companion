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

fn rules() -> &'static [Rule] {
    static RULES: OnceLock<Vec<Rule>> = OnceLock::new();
    RULES.get_or_init(|| {
        let r = |p: &str, make: fn(&regex::Captures) -> Option<Call>| Rule {
            re: Regex::new(&format!("^(?:{p})$")).expect("intent pattern"),
            make,
        };
        vec![
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
            // a bare site: "open github.com" (a known ending, so "open
            // notes.md" stays a file's business)
            r(r"(?:open|go to|visit|browse to) ((?:www\.)?[a-z0-9-]+(?:\.[a-z0-9-]+)*\.(?:com|org|net|io|dev|app|ge|de|uk|co|ai|edu|gov|me|tv|fm|ly|gg|xyz|info|eu|us|ca|au|jp|fr|it|es|nl|ru|pl|ch|se|no|fi|dk|be|at|br|tr|ua|am|az|news|wiki|blog|tech|site|online|store|to|cc|is|social)(?:/\S*)?|www\.\S+)",
              |c| Some(Call::new("open_url", json!({"url": format!("https://{}", &c[1])})))),
            // the browser's history: "find tax in my history", "that site
            // about sheep", "show my history"
            r(r"(?:find|search(?: for)?|look up|look for|show me) (.+?) in (?:my |the )?(?:browser |browsing |web )?history|(?:what was |find |open |show me |go to )?(?:that|the) (?:web ?site|site|page|article) (?:about|with|on) (.+)|(?:show|open|list)(?: me)?(?: my| the)? (?:browser |browsing |web )?history",
              |c| {
                  let q = c.get(1).or(c.get(2)).map_or("", |m| m.as_str()).trim();
                  Some(Call::new("browser_history", json!({"query": q})))
              }),
            r(r"(?:show|list|what are)(?: me)?(?: all)?(?: my| the)?(?: open)? (?:browser )?tabs(?: (?:are )?open)?|(?:which|what) tabs are open",
              |_| Some(Call::new("browser_tabs", json!({})))),
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
