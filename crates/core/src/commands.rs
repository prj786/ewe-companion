//! The command catalog: everything the companion can do on this machine.
//!
//! One list, three users:
//!
//! - the offline matcher ([`crate::intent`]) turns plain words into a [`Call`]
//!   with no AI at all (C5);
//! - the persona ([`crate::persona`]) shows the list to a model, which asks
//!   for a command with an `ACTION` line;
//! - the daemon's `commands` module runs a [`Call`].
//!
//! Adding a command is three steps: a row here, its words in `intent`, its
//! `run` arm in the daemon. Every command is a desktop action (tier T1:
//! it runs on its own and says what it did, with Undo where one exists;
//! decision 6). Anything that changes settings, files or the system (T2/T3)
//! needs a confirm card first and does not belong in this list until P4's
//! ConfirmCard exists.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Call {
    pub command: String,
    #[serde(default)]
    pub args: Value,
}

impl Call {
    pub fn new(command: &str, args: Value) -> Call {
        Call {
            command: command.into(),
            args,
        }
    }
}

pub struct CommandInfo {
    pub name: &'static str,
    /// the arguments, as a model reads them
    pub args: &'static str,
    pub summary: &'static str,
}

pub const CATALOG: &[CommandInfo] = &[
    CommandInfo {
        name: "volume_set",
        args: r#"{"percent": 0-150}"#,
        summary: "set the speaker volume",
    },
    CommandInfo {
        name: "volume_change",
        args: r#"{"delta": -100..100}"#,
        summary: "turn the volume up or down by some percent",
    },
    CommandInfo {
        name: "mute",
        args: r#"{"on": true|false}  (omit "on" to toggle)"#,
        summary: "mute or unmute the speakers",
    },
    CommandInfo {
        name: "brightness_set",
        args: r#"{"percent": 1-100}"#,
        summary: "set the screen brightness",
    },
    CommandInfo {
        name: "brightness_change",
        args: r#"{"delta": -100..100}"#,
        summary: "make the screen brighter or dimmer by some percent",
    },
    CommandInfo {
        name: "media",
        args: r#"{"action": "play_pause"|"next"|"previous"}"#,
        summary: "control the music or video that is playing",
    },
    CommandInfo {
        name: "dnd",
        args: r#"{"on": true|false}"#,
        summary: "turn Do not disturb on or off",
    },
    CommandInfo {
        name: "game_mode",
        args: r#"{"on": true|false}"#,
        summary: "game mode: Do not disturb, no animations or blur, CPU boost (gamemode)",
    },
    CommandInfo {
        name: "screenshot",
        args: r#"{"mode": "region"|"full"|"window"}"#,
        summary: "take a screenshot",
    },
    CommandInfo {
        name: "lock",
        args: "{}",
        summary: "lock the screen",
    },
    CommandInfo {
        name: "open",
        args: r#"{"what": "settings"|"komble"|"quicksettings"|"overview"|"sync"|"terminal"|"files", "page": "…" (optional)}"#,
        summary: "open an ewe app or panel. Settings pages: appearance accessibility animations layout windowrules \
                  displays network bluetooth wallpaper input saver power defaults startup shortcuts time user system \
                  (Settings opens on Appearance whatever the page, so say which page to pick). Komble pages: updates \
                  plugins settings search=<words> options=<plugin id>. Quick settings pages: home wifi bt audio cal notifs",
    },
    CommandInfo {
        name: "launch_app",
        args: r#"{"app": "nemo"|"kitty"|"zed"|"browser"|…, "path": "documents"|"~/Projects/ewe"|… (optional)}"#,
        summary: "open any installed app by its name. With a path, the app opens that folder or file \
                  (a terminal starts in it). Paths may be a folder's name (documents, downloads, pictures, \
                  music, videos, desktop, projects, screenshots, home), ~/…, /…, or a folder under home",
    },
    CommandInfo {
        name: "open_folder",
        args: r#"{"path": "documents"|"~/Projects/ewe"|…}"#,
        summary: "show a folder in the file manager",
    },
    CommandInfo {
        name: "run_shell",
        args: r#"{"command": "ewe-theme scheme apply ewe-light", "about": "Switch to light mode", "terminal": false}"#,
        summary: "propose a shell command for anything no other command covers: ewe's own tools (ewe-theme, \
                  ewe-conf, ewe-plugin…) or any command-line tool. The user sees the exact command and runs it \
                  with a click; nothing runs without that. `about` says what it does in a few words. Set \
                  terminal true when it needs a password (sudo), asks questions, or keeps running. One line; \
                  never a partial upgrade (pacman -Sy without u), never write ewe.conf or the generated files \
                  by hand",
    },
    CommandInfo {
        name: "web_search",
        args: r#"{"query": "…"}"#,
        summary: "search the web in the browser",
    },
    CommandInfo {
        name: "play_music",
        args: r#"{"query": "…"}"#,
        summary: "find a song or artist and play it in the browser",
    },
    CommandInfo {
        name: "open_url",
        args: r#"{"url": "https://…"}"#,
        summary: "open a web page in the browser",
    },
];

/// What `open` opens.
pub const OPEN_WHAT: &[&str] = &[
    "settings",
    "komble",
    "quicksettings",
    "overview",
    "sync",
    "terminal",
    "files",
];

/// The Settings app's pages: its sidebar ids (ewe-settings `App.svelte`)
/// and their labels. Settings 0.18 opens on Appearance whatever page is
/// asked for, so a reply names the page to pick.
pub const SETTINGS_PAGES: &[(&str, &str)] = &[
    ("appearance", "Appearance"),
    ("accessibility", "Accessibility"),
    ("animations", "Animations"),
    ("layout", "Layout"),
    ("windowrules", "Window rules"),
    ("displays", "Displays"),
    ("network", "Networking"),
    ("bluetooth", "Bluetooth"),
    ("wallpaper", "Wallpaper"),
    ("input", "Keyboard and mouse"),
    ("saver", "Screensaver"),
    ("power", "Power"),
    ("defaults", "Default apps"),
    ("startup", "Startup"),
    ("shortcuts", "Shortcuts"),
    ("time", "Time and place"),
    ("user", "User"),
    ("system", "System"),
];

/// Komble's pages (`komble --<page>`); `search=<words>` and
/// `options=<plugin id>` take a value.
pub const KOMBLE_PAGES: &[(&str, &str)] = &[
    ("updates", "Updates"),
    ("plugins", "Plugins"),
    ("settings", "Komble's settings"),
];

/// Quick settings' tabs (`Shell.openQuickSettings`). Plugin tabs (vpn, ssh,
/// mobile, mail, cast) exist only while their plugin is on; an unknown tab
/// opens Home.
pub const QS_TABS: &[(&str, &str)] = &[
    ("home", "Quick settings"),
    ("wifi", "Wi-Fi"),
    ("bt", "Bluetooth"),
    ("audio", "Sound"),
    ("cal", "Calendar"),
    ("notifs", "Notifications"),
    ("vpn", "VPN"),
    ("ssh", "SSH"),
    ("mobile", "Phone"),
    ("mail", "Mail"),
    ("cast", "Cast"),
];

fn label_of(table: &[(&str, &'static str)], id: &str) -> Option<&'static str> {
    table.iter().find(|(k, _)| *k == id).map(|(_, l)| *l)
}

/// A page's label, for a reply: `("settings", "input")` → "Keyboard and mouse".
pub fn page_label(what: &str, page: &str) -> Option<&'static str> {
    match what {
        "settings" => label_of(SETTINGS_PAGES, page),
        "komble" if page.starts_with("search=") => Some("Discover"),
        "komble" if page.starts_with("options=") => Some("the plugin's options"),
        "komble" => label_of(KOMBLE_PAGES, page),
        "quicksettings" => label_of(QS_TABS, page),
        _ => None,
    }
}

/// A Komble page that takes a value: `search=<words>`, `options=<id>`.
fn komble_value_ok(page: &str) -> bool {
    if let Some(w) = page.strip_prefix("search=") {
        let w = w.trim();
        !w.is_empty()
            && w.len() <= 100
            && !w.starts_with('-')
            && w.chars()
                .all(|c| c.is_alphanumeric() || " -_.+".contains(c))
    } else if let Some(id) = page.strip_prefix("options=") {
        !id.is_empty()
            && id.len() <= 64
            && id.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
            && id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-')
    } else {
        false
    }
}

/// A path as words or a path: no control characters, a sane length. Where
/// it points is the daemon's business (it must exist); nothing here runs it.
fn path_text(p: &str, command: &str) -> Result<(), String> {
    if p.len() > 500 || p.chars().any(char::is_control) {
        Err(format!("{command}: that is not a path"))
    } else {
        Ok(())
    }
}

pub fn known(name: &str) -> bool {
    CATALOG.iter().any(|c| c.name == name)
}

/// Check a call's arguments before anything runs: a model may ask for
/// anything, and a wrong shape must fail here, not halfway through a run.
pub fn validate(call: &Call) -> Result<(), String> {
    let a = &call.args;
    let int = |k: &str, lo: i64, hi: i64| -> Result<(), String> {
        match a.get(k).and_then(Value::as_i64) {
            Some(n) if (lo..=hi).contains(&n) => Ok(()),
            _ => Err(format!(
                "{}: \"{k}\" must be a number from {lo} to {hi}",
                call.command
            )),
        }
    };
    let one_of = |k: &str, opts: &[&str]| -> Result<(), String> {
        match a.get(k).and_then(Value::as_str) {
            Some(v) if opts.contains(&v) => Ok(()),
            _ => Err(format!(
                "{}: \"{k}\" must be one of {}",
                call.command,
                opts.join(", ")
            )),
        }
    };
    let text = |k: &str| -> Result<(), String> {
        match a.get(k).and_then(Value::as_str) {
            Some(v) if !v.trim().is_empty() && v.len() <= 500 => Ok(()),
            _ => Err(format!("{}: \"{k}\" must be some text", call.command)),
        }
    };
    let flag = |k: &str, required: bool| -> Result<(), String> {
        match a.get(k) {
            Some(Value::Bool(_)) => Ok(()),
            None | Some(Value::Null) if !required => Ok(()),
            _ => Err(format!("{}: \"{k}\" must be true or false", call.command)),
        }
    };
    match call.command.as_str() {
        "volume_set" => int("percent", 0, 150),
        "volume_change" | "brightness_change" => int("delta", -100, 100),
        "brightness_set" => int("percent", 1, 100),
        "mute" => flag("on", false),
        "dnd" | "game_mode" => flag("on", true),
        "media" => one_of("action", &["play_pause", "next", "previous"]),
        "screenshot" => one_of("mode", &["region", "full", "window"]),
        "open" => {
            one_of("what", OPEN_WHAT)?;
            let page = match a.get("page") {
                None | Some(Value::Null) => return Ok(()),
                Some(Value::String(p)) if p.is_empty() => return Ok(()),
                Some(Value::String(p)) => p.as_str(),
                Some(_) => return Err("open: \"page\" must be text".into()),
            };
            let what = a["what"].as_str().unwrap_or("");
            let ok = match what {
                "komble" => label_of(KOMBLE_PAGES, page).is_some() || komble_value_ok(page),
                "settings" | "quicksettings" => page_label(what, page).is_some(),
                _ => false,
            };
            if ok {
                Ok(())
            } else {
                Err(format!("open: {what} has no page \"{page}\""))
            }
        }
        "web_search" | "play_music" => text("query"),
        "launch_app" => {
            text("app")?;
            if a["app"].as_str().unwrap_or("").len() > 100 {
                return Err("launch_app: \"app\" is too long".into());
            }
            match a.get("path") {
                None | Some(Value::Null) => Ok(()),
                Some(Value::String(p)) if p.is_empty() => Ok(()),
                Some(Value::String(p)) => path_text(p, &call.command),
                Some(_) => Err("launch_app: \"path\" must be text".into()),
            }
        }
        "run_shell" => {
            text("command")?;
            if a["command"].as_str().unwrap_or("").len() > 2000 {
                return Err("run_shell: the command is too long".into());
            }
            flag("terminal", false)?;
            match a.get("about") {
                None | Some(Value::Null) | Some(Value::String(_)) => Ok(()),
                Some(_) => Err("run_shell: \"about\" must be text".into()),
            }
        }
        "open_folder" => {
            text("path")?;
            path_text(a["path"].as_str().unwrap_or(""), &call.command)
        }
        "open_url" => {
            text("url")?;
            let u = a["url"].as_str().unwrap_or("");
            if u.starts_with("https://") || u.starts_with("http://") {
                Ok(())
            } else {
                Err("open_url: only http and https pages".into())
            }
        }
        "lock" => Ok(()),
        other => Err(format!("no command called {other}")),
    }
}

/// `%`-encode a query for a URL (RFC 3986 unreserved characters stay).
pub fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Where a web search goes (a setting later).
pub fn search_url(query: &str) -> String {
    format!("https://duckduckgo.com/?q={}", url_encode(query))
}

/// Where "play <something>" goes until ewe-browser can play by itself.
pub fn music_url(query: &str) -> String {
    format!("https://music.youtube.com/search?q={}", url_encode(query))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn every_catalog_entry_validates_a_good_call() {
        let good = [
            ("volume_set", json!({"percent": 60})),
            ("volume_change", json!({"delta": -10})),
            ("mute", json!({})),
            ("brightness_set", json!({"percent": 40})),
            ("brightness_change", json!({"delta": 10})),
            ("media", json!({"action": "next"})),
            ("dnd", json!({"on": true})),
            ("game_mode", json!({"on": false})),
            ("screenshot", json!({"mode": "region"})),
            ("lock", json!({})),
            ("open", json!({"what": "komble"})),
            ("web_search", json!({"query": "arch wiki hyprland"})),
            ("play_music", json!({"query": "massive attack"})),
            ("open_url", json!({"url": "https://archlinux.org"})),
            ("launch_app", json!({"app": "kitty", "path": "documents"})),
            ("open_folder", json!({"path": "~/Projects"})),
            (
                "run_shell",
                json!({"command": "df -h", "about": "Show free disk space"}),
            ),
        ];
        assert_eq!(good.len(), CATALOG.len());
        for (name, args) in good {
            assert!(known(name), "{name}");
            assert_eq!(validate(&Call::new(name, args)), Ok(()), "{name}");
        }
    }

    #[test]
    fn bad_calls_are_refused() {
        assert!(validate(&Call::new("volume_set", json!({"percent": 900}))).is_err());
        assert!(validate(&Call::new("volume_set", json!({"percent": "loud"}))).is_err());
        assert!(validate(&Call::new("dnd", json!({}))).is_err());
        assert!(validate(&Call::new("open_url", json!({"url": "file:///etc/shadow"}))).is_err());
        assert!(validate(&Call::new("launch_app", json!({}))).is_err());
        assert!(
            validate(&Call::new(
                "launch_app",
                json!({"app": "nemo", "path": "a\nb"})
            ))
            .is_err()
        );
        assert!(validate(&Call::new("launch_app", json!({"app": "nemo", "path": 3}))).is_err());
        assert_eq!(
            validate(&Call::new("launch_app", json!({"app": "nemo"}))),
            Ok(())
        );
        assert!(validate(&Call::new("open_folder", json!({"path": ""}))).is_err());
        assert!(validate(&Call::new("rm_rf", json!({}))).is_err());
        assert!(validate(&Call::new("open", json!({"what": "terminal; rm -rf"}))).is_err());
    }

    #[test]
    fn open_takes_a_page_where_there_are_pages() {
        let ok = |what: &str, page: &str| {
            validate(&Call::new("open", json!({"what": what, "page": page})))
        };
        assert_eq!(ok("settings", "displays"), Ok(()));
        assert_eq!(ok("komble", "updates"), Ok(()));
        assert_eq!(ok("komble", "search=video editor"), Ok(()));
        assert_eq!(ok("komble", "options=ewe.dock"), Ok(()));
        assert_eq!(ok("quicksettings", "wifi"), Ok(()));
        assert_eq!(ok("settings", ""), Ok(()));
        assert!(ok("settings", "kernel").is_err());
        assert!(ok("terminal", "wifi").is_err());
        assert!(ok("komble", "search=--devtools").is_err());
        assert!(ok("komble", "search=a;b").is_err());
        assert!(ok("komble", "options=../../etc").is_err());
        assert_eq!(page_label("settings", "input"), Some("Keyboard and mouse"));
        assert_eq!(page_label("quicksettings", "bt"), Some("Bluetooth"));
    }

    #[test]
    fn queries_are_encoded() {
        assert_eq!(url_encode("a b&c"), "a+b%26c");
        assert_eq!(
            search_url("ewe desktop"),
            "https://duckduckgo.com/?q=ewe+desktop"
        );
        assert!(music_url("ქართული").starts_with("https://music.youtube.com/search?q=%E1"));
    }
}
