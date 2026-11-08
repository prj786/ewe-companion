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
//! `run` arm in the daemon. Every command is a read (tier T0: it only shows
//! something, such as found files, the browser's history or a sum) or a
//! desktop action (tier T1: it runs on its own and says what it did, with
//! Undo where one exists; decision 6). Anything that changes settings,
//! files or the system (T2/T3: dark or light mode, suspend, restart, shut
//! down, log out, emptying the trash) is in the list only as a *proposal*
//! ([`PROPOSALS`]): its run arm never runs it, it shows the exact command
//! with Run, and only that click runs it (P4's ConfirmCard will take over).
//!
//! Safe's commands (`safe_*`) never carry a value: Safe copies a password
//! itself, and a model that asks for one gets back "done" or a count.

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
        summary: "game mode: the desktop's (ewe-game, Ultra or Balanced: quiet banners, no effects, CPU boost; Ultra also the Performance power profile and ewe's background apps closed)",
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
        summary: "open an ewe app or panel. ewe Settings pages: appearance wallpaper animations accessibility layout \
                  windowrules displays startup shortcuts network bluetooth input power saver defaults time user system \
                  (ewe Settings opens on Appearance whatever the page, so say which page to pick). Komble pages: updates \
                  plugins settings search=<words> options=<plugin id>. Quick settings pages: home wifi bt audio cal notifs \
                  remote power, and the plugins' vpn ssh mobile mail cast",
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
        args: r#"{"query": "men's shoes", "site": "zara" (optional: a site's name or domain)}"#,
        summary: "search the web in ewe Browser (its own search engine); with site, that site's own search",
    },
    CommandInfo {
        name: "play_music",
        args: r#"{"query": "…"}"#,
        summary: "play a song or artist: from the music library in ewe Browser, else YouTube Music",
    },
    CommandInfo {
        name: "open_url",
        args: r#"{"url": "https://…", "workspace": "Work" (optional: ewe Browser's workspace)}"#,
        summary: "open a web page in ewe Browser, in one of its workspaces if named",
    },
    CommandInfo {
        name: "browser_history",
        args: r#"{"query": "tax"}"#,
        summary: "find pages in ewe Browser's history. The pages show on the island; you get only how many",
    },
    CommandInfo {
        name: "browser_tabs",
        args: "{}",
        summary: "list ewe Browser's open tabs on the island; you get only how many",
    },
    CommandInfo {
        name: "browser_tab",
        args: r#"{"words": "youtube"}"#,
        summary: "switch to the open tab in ewe Browser whose title or address has these words",
    },
    CommandInfo {
        name: "find_files",
        args: r#"{"query": "pdfs about tax", "kind": "any"|"file"|"folder" (optional)}"#,
        summary: "find files and folders in the home folder by their name; type words work (pdfs, photos, \
                  videos, spreadsheets). The results show on the island; you get only how many",
    },
    CommandInfo {
        name: "open_file",
        args: r#"{"path": "~/Documents/cv.pdf"}"#,
        summary: "open a file with its default app",
    },
    CommandInfo {
        name: "safe_copy",
        args: r#"{"query": "github", "field": "password"|"username"|"otp"}"#,
        summary: "ewe Safe (the password manager) copies a login's password, username or one-time code to the \
                  clipboard and clears it later. You never see the value or the login's name: you get back \
                  done or a count. Never ask the user for a password",
    },
    CommandInfo {
        name: "safe_login",
        args: r#"{"query": "github"}"#,
        summary: "open the website of a login in ewe Safe in ewe Browser; the user fills it there with a click",
    },
    CommandInfo {
        name: "safe_lock",
        args: "{}",
        summary: "lock ewe Safe",
    },
    CommandInfo {
        name: "safe_unlock",
        args: "{}",
        summary: "open ewe Safe's unlock prompt (the user types the master password there, never in the chat)",
    },
    CommandInfo {
        name: "safe_status",
        args: "{}",
        summary: "ewe Safe's state: locked or not, how many items, whether it is in sync",
    },
    // ── ewe browser beyond search and open ──
    CommandInfo {
        name: "browser_bookmark",
        args: "{}",
        summary: "bookmark the page open in ewe Browser (Undo removes it)",
    },
    CommandInfo {
        name: "browser_bookmarks",
        args: r#"{"query": "recipes" (optional)}"#,
        summary: "find ewe Browser's bookmarks; they show on the island, you get only how many",
    },
    CommandInfo {
        name: "browser_nav",
        args: r#"{"action": "back"|"forward"|"reload"|"reopen"}"#,
        summary: "in ewe Browser: go back or forward a page, reload it, or reopen the tab closed last",
    },
    CommandInfo {
        name: "browser_tab_do",
        args: r#"{"action": "mute"|"unmute"|"pin"|"unpin"|"close", "words": "youtube" (optional: else the tab in front)}"#,
        summary: "mute, unmute, pin or unpin a tab in ewe Browser; close asks first",
    },
    CommandInfo {
        name: "browser_workspace",
        args: r#"{"action": "list"|"switch"|"next"|"prev"|"new", "name": "Work" (switch: a name or number; new: optional)}"#,
        summary: "ewe Browser's own workspaces (the bottom bar): list them, switch, the next or previous one, a new one",
    },
    CommandInfo {
        name: "browser_downloads",
        args: "{}",
        summary: "ewe Browser's downloads, with Open and Show in folder, on the island; you get only how many",
    },
    CommandInfo {
        name: "browser_download",
        args: r#"{"id": "<download id>", "action": "open"|"show", "path": "<its file>"}"#,
        summary: "open a download, or show it in its folder (the ids come from browser_downloads)",
    },
    CommandInfo {
        name: "browser_page",
        args: r#"{"page": "home"|"history"|"settings"|"files"|"music"}"#,
        summary: "open one of ewe Browser's own pages",
    },
    // ── the desktop ──
    CommandInfo {
        name: "wifi",
        args: r#"{"on": true|false}"#,
        summary: "turn Wi-Fi on or off",
    },
    CommandInfo {
        name: "bluetooth",
        args: r#"{"on": true|false}"#,
        summary: "turn Bluetooth on or off",
    },
    CommandInfo {
        name: "bluetooth_device",
        args: r#"{"name": "airpods", "connect": true|false}"#,
        summary: "connect or disconnect a paired Bluetooth device by its name",
    },
    CommandInfo {
        name: "airplane_mode",
        args: r#"{"on": true|false}"#,
        summary: "airplane mode: Wi-Fi and Bluetooth off (off puts them back as they were)",
    },
    CommandInfo {
        name: "keep_awake",
        args: r#"{"on": true|false}"#,
        summary: "keep the screen awake (no dimming, lock or sleep) until turned off",
    },
    CommandInfo {
        name: "calc",
        args: r#"{"expr": "15% of 80"}"#,
        summary: "work out plain arithmetic and percentages exactly (+ - * / ^, %, of, off)",
    },
    CommandInfo {
        name: "timer",
        args: r#"{"seconds": 300, "text": "tea" (optional)}  or  {"at": "17:30", "text": "call mum"}; "reminder": true for "remind me" with no words"#,
        summary: "a timer or a reminder: a notification when it is due (cancellable, kept across restarts)",
    },
    CommandInfo {
        name: "timers",
        args: "{}",
        summary: "the timers and reminders waiting, with Cancel",
    },
    CommandInfo {
        name: "timer_cancel",
        args: r#"{"id": "<timer id>" (optional: the only one)}"#,
        summary: "cancel a timer or reminder",
    },
    CommandInfo {
        name: "clock",
        args: r#"{"place": "tokyo" (optional: a city, a country or a zone like Asia/Tokyo)}"#,
        summary: "the time and date now, here or in a place (offline)",
    },
    CommandInfo {
        name: "date_math",
        args: r#"{"op": "until"|"since"|"weekday", "date": "christmas"|"25 december"|"2026-12-25"|"next friday"}  or  {"op": "offset", "days": 30 (negative: ago)}"#,
        summary: "days until or since a date, the weekday a date falls on, or the date some days from today (offline)",
    },
    CommandInfo {
        name: "convert",
        args: r#"{"value": 10, "from": "km", "to": "mi"}"#,
        summary: "convert units (offline): length, weight, temperature (degC, degF, K), volume (US cups, pints, gallons), \
                  speed, data (GB = 1000 MB as disks count; GiB = 1024 MiB), time, area. Not currencies",
    },
    CommandInfo {
        name: "system_info",
        args: r#"{"what": "battery"|"memory"|"cpu"|"uptime"|"network"|"disk"|"overview", "apps": true (optional, memory or cpu: which apps use the most)}"#,
        summary: "this computer now: the battery, memory, the CPU and the apps using it, how long it has been on, the \
                  network (online or not, local addresses; never the public IP, that would ask a website), free disk \
                  space, or all of it at once (overview: for \"why is my laptop slow\")",
    },
    CommandInfo {
        name: "disk_usage",
        args: "{}",
        summary: "what takes the disk's space: the package cache, ~/.cache, the trash, Downloads, the system journal, crash dumps",
    },
    CommandInfo {
        name: "workspace",
        args: r#"{"number": 1-10}  or  {"step": "next"|"prev"}"#,
        summary: "switch to a workspace of the display you are on (Undo goes back)",
    },
    CommandInfo {
        name: "window",
        args: r#"{"action": "move", "to": 1-10} | {"action": "fullscreen"|"maximize"|"float", "on": true|false} | {"action": "minimize"|"restore"|"close"}; "app": "firefox" (optional: else the window in front)"#,
        summary: "the window in front, or an app's: move it to a workspace, fullscreen, maximise, float or tile it, \
                  minimise it, or bring back the last one minimised. close only offers a Close button for the user \
                  to click (the app may ask to save)",
    },
    CommandInfo {
        name: "force_quit",
        args: r#"{"app": "firefox", "force": true}"#,
        summary: "end an app that is frozen or not responding (asks first: shows the command with Run)",
    },
    CommandInfo {
        name: "remember",
        args: r#"{"text": "my dentist is on friday"}"#,
        summary: "keep something the user asks you to remember, in their words (Undo forgets it). Never a password, \
                  PIN, code, key or card number: those belong in ewe Safe, and are refused",
    },
    CommandInfo {
        name: "memories",
        args: r#"{"query": "dentist" (optional)}"#,
        summary: "what the user asked you to remember, with Forget on each",
    },
    CommandInfo {
        name: "forget",
        args: r#"{"words": "dentist"}  or  {"id": "<memory id>"}  or  {"all": true} (all only offers a button)"#,
        summary: "forget something you were asked to remember (Undo puts it back)",
    },
    CommandInfo {
        name: "theme_mode",
        args: r#"{"mode": "dark"|"light"}"#,
        summary: "switch the whole desktop to dark or light mode (asks first: shows the command with Run)",
    },
    CommandInfo {
        name: "power",
        args: r#"{"action": "suspend"|"restart"|"shutdown"|"logout"}"#,
        summary: "suspend, restart, shut down or log out (asks first: shows the command with Run)",
    },
    CommandInfo {
        name: "empty_trash",
        args: "{}",
        summary: "empty the trash for good (asks first: shows the command with Run)",
    },
];

/// The commands that never run from a call: their run arm shows the exact
/// command with Run (T2/T3, decision 6), and only that click runs it.
pub const PROPOSALS: &[&str] = &[
    "run_shell",
    "theme_mode",
    "power",
    "empty_trash",
    "force_quit",
];

/// ewe browser's own pages (`browser_page`) and the address each has.
pub const BROWSER_PAGES: &[(&str, &str)] = &[
    ("home", "app://home/"),
    ("history", "app://history/"),
    ("settings", "app://settings/"),
    ("files", "app://files/"),
    ("music", "app://music/"),
];

/// What Safe copies: `safe_copy`'s `field`.
pub const SAFE_FIELDS: &[&str] = &["password", "username", "otp"];

/// The commands that answer from Safe. Their results (a login's title, its
/// username) are shown on the island only: never in a model's context, never
/// stored (TRIO.md, "names stay local").
pub fn is_safe(command: &str) -> bool {
    command.starts_with("safe_")
}

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

/// The Settings app's pages: its pane ids and their labels, in nav order
/// (ewe-settings 0.22 `src/lib/pages.js`: Look, Desktop, Devices, System).
/// ewe Settings (0.18 to 0.22) opens on Appearance whatever page is asked
/// for, so a reply names the page to pick.
pub const SETTINGS_PAGES: &[(&str, &str)] = &[
    ("appearance", "Appearance"),
    ("wallpaper", "Wallpaper"),
    ("animations", "Animations"),
    ("accessibility", "Accessibility"),
    ("layout", "Layout"),
    ("windowrules", "Window rules"),
    ("displays", "Displays"),
    ("startup", "Startup apps"),
    ("shortcuts", "Keyboard shortcuts"),
    ("network", "Network"),
    ("bluetooth", "Bluetooth"),
    ("input", "Keyboard and mouse"),
    ("power", "Power"),
    ("saver", "Screensaver"),
    ("defaults", "Default apps"),
    ("time", "Time and place"),
    ("user", "User"),
    ("system", "About this computer"),
];

/// Komble's pages (`komble --<page>`); `search=<words>` and
/// `options=<plugin id>` take a value.
pub const KOMBLE_PAGES: &[(&str, &str)] = &[
    ("updates", "Updates"),
    ("plugins", "Plugins"),
    ("settings", "Komble's settings"),
];

/// Quick settings' pages (`Shell.openQuickSettings`; ewe 0.39 has no rail,
/// a page opens in place of home). Plugin pages (vpn, ssh, mobile, mail,
/// cast) exist only while their plugin is on, labelled as their manifests'
/// `quickPage.label`; an unknown page opens home.
pub const QS_TABS: &[(&str, &str)] = &[
    ("home", "Quick settings"),
    ("wifi", "Wi-Fi"),
    ("bt", "Bluetooth"),
    ("audio", "Sound"),
    ("cal", "Calendar"),
    ("notifs", "Notifications"),
    ("remote", "Sharing"),
    ("power", "Power"),
    ("vpn", "VPN"),
    ("ssh", "SSH"),
    ("mobile", "Mobile"),
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

/// Safe's words go to its command line as one argument: never one it
/// could read as a flag.
fn safe_words(a: &Value) -> Result<(), String> {
    match a["query"].as_str() {
        Some(q) if q.trim_start().starts_with('-') || q.len() > 200 => {
            Err("ewe Safe: that is not a name to look for".into())
        }
        _ => Ok(()),
    }
}

/// An id another app gave us (a Safe item's UUID, a browser tab's id):
/// letters, digits and dashes, nothing a command line could read as more.
fn token_ok(id: &str) -> bool {
    (4..=64).contains(&id.len())
        && !id.starts_with('-')
        && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// A name the user gave something (a site, a workspace, a device): one
/// argument on another app's command line, never one it could read as a
/// flag.
fn name_ok(s: &str, max: usize) -> bool {
    let t = s.trim();
    !t.is_empty() && t.len() <= max && !t.starts_with('-') && !t.chars().any(char::is_control)
}

/// A Bluetooth address: `AA:BB:CC:DD:EE:FF`.
pub fn is_mac(s: &str) -> bool {
    let parts: Vec<&str> = s.split(':').collect();
    parts.len() == 6
        && parts
            .iter()
            .all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_hexdigit()))
}

/// A Hyprland window's address: `0x55d0c1a2b3c0`.
pub fn is_address(s: &str) -> bool {
    s.strip_prefix("0x")
        .is_some_and(|h| (1..=16).contains(&h.len()) && h.chars().all(|c| c.is_ascii_hexdigit()))
}

/// A time of day, `17:30` or `9:05`, as (hour, minute).
pub fn clock(s: &str) -> Option<(u32, u32)> {
    let (h, m) = s.trim().split_once(':')?;
    let (h, m): (u32, u32) = (h.parse().ok()?, m.parse().ok()?);
    (h < 24 && m < 60 && s.trim().len() <= 5).then_some((h, m))
}

/// The longest timer: a week.
pub const TIMER_MAX_S: i64 = 7 * 86_400;

/// Programs that are the desktop itself: Quit is never offered for them,
/// and "kill hyprland" is refused (it would end the session or the shell).
pub const SESSION: &[&str] = &[
    "hyprland",
    "quickshell",
    "qs",
    "ewe-companiond",
    "systemd",
    "dbus-daemon",
    "dbus-broker",
    "pipewire",
    "pipewire-pulse",
    "wireplumber",
    "xwayland",
    "greetd",
    "networkmanager",
    "polkitd",
    "xdg-desktop-portal",
    "xdg-desktop-portal-hyprland",
];

pub fn is_session(app: &str) -> bool {
    let a = app.trim().to_lowercase();
    SESSION.contains(&a.as_str())
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
    // an optional name: a site, a workspace, a device
    let opt_name = |k: &str, max: usize| -> Result<(), String> {
        match a.get(k) {
            None | Some(Value::Null) => Ok(()),
            Some(Value::String(v)) if name_ok(v, max) => Ok(()),
            _ => Err(format!("{}: \"{k}\" must be a short name", call.command)),
        }
    };
    let opt_text = |k: &str, max: usize| -> Result<(), String> {
        match a.get(k) {
            None | Some(Value::Null) => Ok(()),
            Some(Value::String(v)) if v.len() <= max && !v.chars().any(char::is_control) => Ok(()),
            _ => Err(format!("{}: \"{k}\" must be some text", call.command)),
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
        "web_search" => {
            text("query")?;
            opt_name("site", 64)
        }
        "play_music" => text("query"),
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
            if !(u.starts_with("https://") || u.starts_with("http://")) {
                return Err("open_url: only http and https pages".into());
            }
            opt_name("workspace", 64)
        }
        "lock" | "browser_tabs" | "safe_lock" | "safe_status" | "browser_bookmark"
        | "browser_downloads" | "timers" | "empty_trash" => Ok(()),
        "browser_bookmarks" => opt_text("query", 200),
        "browser_nav" => one_of("action", &["back", "forward", "reload", "reopen"]),
        "browser_tab_do" => {
            one_of("action", &["mute", "unmute", "pin", "unpin", "close"])?;
            match a.get("id").filter(|v| !v.is_null()) {
                Some(id) if id.as_str().is_some_and(token_ok) => Ok(()),
                Some(_) => Err("browser_tab_do: \"id\" is not a tab id".into()),
                None => opt_name("words", 100),
            }
        }
        "browser_workspace" => {
            one_of("action", &["list", "switch", "next", "prev", "new"])?;
            opt_name("name", 64)?;
            if a["action"] == "switch" && a.get("name").is_none_or(Value::is_null) {
                return Err("browser_workspace: switch needs the workspace's name".into());
            }
            Ok(())
        }
        "browser_download" => {
            one_of("action", &["open", "show"])?;
            match a.get("id").and_then(Value::as_str) {
                Some(id)
                    if !id.is_empty()
                        && id.len() <= 64
                        && !id.starts_with('-')
                        && id
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c)) => {}
                _ => return Err("browser_download: \"id\" is not a download id".into()),
            }
            match a.get("path") {
                None | Some(Value::Null) => Ok(()),
                Some(Value::String(p)) => path_text(p, &call.command),
                Some(_) => Err("browser_download: \"path\" must be text".into()),
            }
        }
        "browser_page" => {
            let pages: Vec<&str> = BROWSER_PAGES.iter().map(|(p, _)| *p).collect();
            one_of("page", &pages)
        }
        "wifi" | "bluetooth" | "keep_awake" => flag("on", true),
        // the Undo of airplane mode carries how Wi-Fi and Bluetooth were
        "airplane_mode" => {
            flag("on", true)?;
            flag("wifi", false)?;
            flag("bluetooth", false)
        }
        "bluetooth_device" => {
            flag("connect", true)?;
            match a.get("mac").filter(|v| !v.is_null()) {
                Some(m) if m.as_str().is_some_and(is_mac) => Ok(()),
                Some(_) => Err("bluetooth_device: \"mac\" is not a Bluetooth address".into()),
                None => match a.get("name").and_then(Value::as_str) {
                    Some(n) if name_ok(n, 100) => Ok(()),
                    _ => Err("bluetooth_device: \"name\" must be the device's name".into()),
                },
            }
        }
        "calc" => {
            text("expr")?;
            if a["expr"].as_str().unwrap_or("").len() > 200 {
                return Err("calc: that sum is too long".into());
            }
            Ok(())
        }
        "timer" => {
            let secs = a.get("seconds").filter(|v| !v.is_null());
            let at = a.get("at").filter(|v| !v.is_null());
            match (secs, at) {
                (Some(_), None) => int("seconds", 1, TIMER_MAX_S)?,
                (None, Some(t)) if t.as_str().and_then(clock).is_some() => {}
                (None, Some(_)) => return Err("timer: \"at\" must be a time like 17:30".into()),
                _ => return Err("timer: give \"seconds\" or \"at\", one of them".into()),
            }
            flag("reminder", false)?;
            opt_text("text", 200)
        }
        "timer_cancel" => match a.get("id").filter(|v| !v.is_null()) {
            None => Ok(()),
            Some(id) if id.as_str().is_some_and(token_ok) => Ok(()),
            Some(_) => Err("timer_cancel: \"id\" is not a timer id".into()),
        },
        "clock" => opt_name("place", 64),
        "date_math" => {
            one_of("op", &["until", "since", "weekday", "offset"])?;
            if a["op"] == "offset" {
                int("days", -36_500, 36_500)
            } else {
                match a.get("date").and_then(Value::as_str) {
                    Some(d) if d.len() <= 60 && crate::dates::parse(d).is_some() => Ok(()),
                    _ => Err("date_math: \"date\" must name a day (christmas, 25 december, 2026-12-25, next friday)".into()),
                }
            }
        }
        "convert" => {
            let v = a.get("value").and_then(Value::as_f64);
            let (from, to) = (
                a["from"].as_str().unwrap_or(""),
                a["to"].as_str().unwrap_or(""),
            );
            match v {
                Some(v) if v.is_finite() && v.abs() < 1e15 => {}
                _ => return Err("convert: \"value\" must be a number".into()),
            }
            if from.len() > 40 || to.len() > 40 || crate::units::convert(1.0, from, to).is_none() {
                return Err(format!(
                    "convert: {from} to {to} is not a conversion I know (units of one kind; no currencies)"
                ));
            }
            Ok(())
        }
        "system_info" => {
            one_of(
                "what",
                &[
                    "battery", "memory", "cpu", "uptime", "network", "disk", "overview",
                ],
            )?;
            flag("apps", false)
        }
        "disk_usage" => Ok(()),
        "workspace" => {
            let has = |k: &str| a.get(k).is_some_and(|v| !v.is_null());
            match (has("number"), has("step"), has("id")) {
                (true, false, false) => int("number", 1, 10),
                (false, true, false) => one_of("step", &["next", "prev"]),
                // the Undo: back to the workspace it left, by Hyprland's own id
                (false, false, true) => int("id", 1, 1000),
                _ => Err("workspace: give \"number\" (1-10) or \"step\" (next or prev)".into()),
            }
        }
        "window" => {
            one_of(
                "action",
                &[
                    "move",
                    "fullscreen",
                    "maximize",
                    "float",
                    "minimize",
                    "restore",
                    "close",
                ],
            )?;
            flag("on", false)?;
            opt_name("app", 64)?;
            if a["action"] == "move" {
                match (
                    a.get("to").filter(|v| !v.is_null()),
                    a.get("to_id").filter(|v| !v.is_null()),
                ) {
                    (Some(_), None) => int("to", 1, 10)?,
                    (None, Some(_)) => int("to_id", 1, 1000)?,
                    _ => return Err("window: move needs \"to\", a workspace from 1 to 10".into()),
                }
            }
            if let Some(addr) = a.get("address").filter(|v| !v.is_null()) {
                match addr.as_str() {
                    Some(s) if is_address(s) => {}
                    _ => return Err("window: \"address\" is not a window's address".into()),
                }
            }
            match a.get("token").filter(|v| !v.is_null()) {
                None => Ok(()),
                Some(t) if t.as_str().is_some_and(token_ok) => Ok(()),
                Some(_) => Err("window: that is not a token".into()),
            }
        }
        "force_quit" => {
            flag("force", false)?;
            match a.get("app").and_then(Value::as_str) {
                Some(n) if is_session(n) => Err(format!(
                    "force_quit: {n} is part of the desktop itself; stopping it would end your session or the shell"
                )),
                Some(n)
                    if name_ok(n, 64)
                        && n.chars()
                            .all(|c| c.is_alphanumeric() || " ._+-".contains(c)) =>
                {
                    Ok(())
                }
                _ => Err("force_quit: \"app\" must be an app's name".into()),
            }
        }
        "remember" => {
            text("text")?;
            if a["text"].as_str().unwrap_or("").len() > 300 {
                return Err("remember: that is too long to keep; say it in a sentence".into());
            }
            if a["text"]
                .as_str()
                .unwrap_or("")
                .chars()
                .any(char::is_control)
            {
                return Err("remember: \"text\" must be one line".into());
            }
            Ok(())
        }
        "memories" => opt_text("query", 100),
        "forget" => {
            let has = |k: &str| a.get(k).is_some_and(|v| !v.is_null());
            if has("all") {
                flag("all", true)?;
                return match a.get("token").filter(|v| !v.is_null()) {
                    None => Ok(()),
                    Some(t) if t.as_str().is_some_and(token_ok) => Ok(()),
                    Some(_) => Err("forget: that is not a token".into()),
                };
            }
            match (a.get("id").filter(|v| !v.is_null()), has("words")) {
                (Some(id), _) if id.as_str().is_some_and(token_ok) => Ok(()),
                (Some(_), _) => Err("forget: \"id\" is not a memory's id".into()),
                (None, true) => opt_name("words", 100),
                (None, false) => Err("forget: say what to forget (\"words\")".into()),
            }
        }
        "theme_mode" => one_of("mode", &["dark", "light"]),
        "power" => one_of("action", &["suspend", "restart", "shutdown", "logout"]),
        "browser_history" => match a.get("query") {
            None | Some(Value::Null) => Ok(()),
            Some(Value::String(q)) if q.len() <= 200 => Ok(()),
            _ => Err("browser_history: \"query\" must be some text".into()),
        },
        "browser_tab" => {
            // the words of a tab ("youtube"), or the browser's own tab id
            // (a row's click)
            if let Some(id) = a.get("id").filter(|v| !v.is_null()) {
                match id.as_str() {
                    Some(i) if token_ok(i) => Ok(()),
                    _ => Err("browser_tab: \"id\" is not a tab id".into()),
                }
            } else {
                text("words")
            }
        }
        "find_files" => {
            text("query")?;
            if a["query"].as_str().unwrap_or("").len() > 200 {
                return Err("find_files: \"query\" is too long".into());
            }
            match a.get("kind") {
                None | Some(Value::Null) => Ok(()),
                Some(_) => one_of("kind", &["any", "file", "folder"]),
            }
        }
        "open_file" => {
            text("path")?;
            path_text(a["path"].as_str().unwrap_or(""), &call.command)
        }
        "safe_copy" => {
            match a.get("id").filter(|v| !v.is_null()) {
                Some(id) if id.as_str().is_some_and(token_ok) => {}
                Some(_) => return Err("safe_copy: \"id\" is not an item id".into()),
                None => {
                    text("query")?;
                    safe_words(a)?;
                }
            }
            match a.get("field") {
                None | Some(Value::Null) => {}
                Some(_) => one_of("field", SAFE_FIELDS)?,
            }
            // what the island shows after a row's click; never a value
            for k in ["title", "username"] {
                match a.get(k) {
                    None | Some(Value::Null) => {}
                    Some(Value::String(s)) if s.len() <= 200 => {}
                    Some(_) => return Err(format!("safe_copy: \"{k}\" must be text")),
                }
            }
            Ok(())
        }
        "safe_login" => {
            text("query")?;
            safe_words(a)
        }
        "safe_unlock" => match a.get("then").filter(|v| !v.is_null()) {
            // the command that was waiting for the unlock, run after it
            None => Ok(()),
            Some(then) => {
                let next: Call = serde_json::from_value(then.clone())
                    .map_err(|_| "safe_unlock: \"then\" must be a command".to_string())?;
                if !matches!(
                    next.command.as_str(),
                    "safe_copy" | "safe_login" | "safe_status"
                ) {
                    return Err(
                        "safe_unlock: only an ewe Safe command can wait for the unlock".into(),
                    );
                }
                validate(&next)
            }
        },
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
mod session_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_desktop_itself_is_never_quit() {
        for app in ["Hyprland", "quickshell", "qs", "pipewire", "Xwayland"] {
            let c = Call::new("force_quit", json!({"app": app, "force": false}));
            assert!(validate(&c).is_err(), "{app}");
        }
        let c = Call::new("force_quit", json!({"app": "firefox", "force": true}));
        assert!(validate(&c).is_ok());
    }
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
            ("browser_history", json!({"query": "tax"})),
            ("browser_tabs", json!({})),
            ("browser_tab", json!({"words": "youtube"})),
            ("find_files", json!({"query": "pdfs about tax"})),
            ("open_file", json!({"path": "~/Documents/cv.pdf"})),
            ("safe_copy", json!({"query": "github", "field": "password"})),
            ("safe_login", json!({"query": "github"})),
            ("safe_lock", json!({})),
            ("safe_unlock", json!({})),
            ("safe_status", json!({})),
            ("browser_bookmark", json!({})),
            ("browser_bookmarks", json!({"query": "recipes"})),
            ("browser_nav", json!({"action": "back"})),
            ("browser_tab_do", json!({"action": "mute"})),
            (
                "browser_workspace",
                json!({"action": "switch", "name": "Work"}),
            ),
            ("browser_downloads", json!({})),
            (
                "browser_download",
                json!({"id": "d-1", "action": "show", "path": "~/Downloads/a.iso"}),
            ),
            ("browser_page", json!({"page": "history"})),
            ("wifi", json!({"on": false})),
            ("bluetooth", json!({"on": true})),
            (
                "bluetooth_device",
                json!({"name": "airpods", "connect": true}),
            ),
            ("airplane_mode", json!({"on": true})),
            ("keep_awake", json!({"on": true})),
            ("calc", json!({"expr": "15% of 80"})),
            ("timer", json!({"seconds": 300, "text": "tea"})),
            ("timers", json!({})),
            ("timer_cancel", json!({})),
            ("clock", json!({"place": "tokyo"})),
            ("date_math", json!({"op": "until", "date": "christmas"})),
            ("convert", json!({"value": 10, "from": "km", "to": "mi"})),
            ("system_info", json!({"what": "memory", "apps": true})),
            ("disk_usage", json!({})),
            ("workspace", json!({"number": 3})),
            ("window", json!({"action": "move", "to": 2})),
            ("force_quit", json!({"app": "firefox", "force": true})),
            ("remember", json!({"text": "my dentist is on friday"})),
            ("memories", json!({})),
            ("forget", json!({"words": "dentist"})),
            ("theme_mode", json!({"mode": "light"})),
            ("power", json!({"action": "suspend"})),
            ("empty_trash", json!({})),
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
    fn new_powers_check_their_arguments() {
        let ok = |c: &str, a: Value| validate(&Call::new(c, a));
        // Safe: an item id from a row's click, or the words; never a value
        assert_eq!(
            ok(
                "safe_copy",
                json!({"id": "4f9c0d1e-1111-2222-3333-444455556666", "field": "otp", "title": "GitHub"})
            ),
            Ok(())
        );
        assert_eq!(ok("safe_copy", json!({"query": "github"})), Ok(()));
        assert!(ok("safe_copy", json!({"id": "--field=password"})).is_err());
        assert!(ok("safe_copy", json!({"id": "a b"})).is_err());
        assert!(ok("safe_copy", json!({"query": "github", "field": "notes"})).is_err());
        assert!(ok("safe_copy", json!({})).is_err());
        assert!(ok("safe_copy", json!({"query": "--reveal"})).is_err());
        assert!(ok("safe_login", json!({"query": "-x"})).is_err());
        // a command waiting for the unlock: only a Safe one
        assert_eq!(
            ok(
                "safe_unlock",
                json!({"then": {"command": "safe_copy", "args": {"query": "github"}}})
            ),
            Ok(())
        );
        assert!(
            ok(
                "safe_unlock",
                json!({"then": {"command": "run_shell", "args": {"command": "id"}}})
            )
            .is_err()
        );
        assert!(ok("safe_unlock", json!({"then": "lock"})).is_err());
        // the browser: words, or its own tab id
        assert_eq!(ok("browser_tab", json!({"id": "1002f85c"})), Ok(()));
        assert!(ok("browser_tab", json!({"id": "-x"})).is_err());
        assert!(ok("browser_tab", json!({})).is_err());
        assert_eq!(ok("browser_history", json!({})), Ok(()));
        assert!(ok("browser_history", json!({"query": 3})).is_err());
        // files
        assert!(ok("find_files", json!({"query": ""})).is_err());
        assert!(ok("find_files", json!({"query": "tax", "kind": "disk"})).is_err());
        assert_eq!(
            ok("find_files", json!({"query": "tax", "kind": "folder"})),
            Ok(())
        );
        assert!(ok("open_file", json!({"path": "a\nb"})).is_err());
        assert!(is_safe("safe_copy") && !is_safe("open_url"));
    }

    #[test]
    fn wave_two_commands_check_their_arguments() {
        let ok = |c: &str, a: Value| validate(&Call::new(c, a));
        // a site or a workspace is one argument, never a flag
        assert_eq!(
            ok(
                "web_search",
                json!({"query": "men's shoes", "site": "zara"})
            ),
            Ok(())
        );
        assert_eq!(
            ok("web_search", json!({"query": "kettle", "site": "h&m"})),
            Ok(())
        );
        assert!(ok("web_search", json!({"query": "x", "site": "--new-window"})).is_err());
        assert!(ok("web_search", json!({"query": "x", "site": ""})).is_err());
        assert!(
            ok(
                "open_url",
                json!({"url": "https://a.org", "workspace": "-x"})
            )
            .is_err()
        );
        assert_eq!(
            ok(
                "open_url",
                json!({"url": "https://a.org", "workspace": "Work"})
            ),
            Ok(())
        );
        // the browser
        assert!(ok("browser_nav", json!({"action": "home"})).is_err());
        assert!(ok("browser_tab_do", json!({"action": "close", "id": "-all"})).is_err());
        assert_eq!(
            ok(
                "browser_tab_do",
                json!({"action": "pin", "words": "github"})
            ),
            Ok(())
        );
        assert!(ok("browser_workspace", json!({"action": "switch"})).is_err());
        assert_eq!(ok("browser_workspace", json!({"action": "new"})), Ok(()));
        assert!(
            ok(
                "browser_workspace",
                json!({"action": "remove", "name": "x"})
            )
            .is_err()
        );
        assert!(ok("browser_download", json!({"id": "--x", "action": "open"})).is_err());
        assert!(ok("browser_download", json!({"id": "7", "action": "delete"})).is_err());
        assert!(ok("browser_page", json!({"page": "file:///etc"})).is_err());
        // the desktop
        assert!(ok("wifi", json!({})).is_err());
        assert!(ok("bluetooth_device", json!({"connect": true})).is_err());
        assert_eq!(
            ok(
                "bluetooth_device",
                json!({"mac": "AA:BB:CC:DD:EE:0F", "connect": false})
            ),
            Ok(())
        );
        assert!(
            ok(
                "bluetooth_device",
                json!({"mac": "AA:BB:CC:DD:EE", "connect": false})
            )
            .is_err()
        );
        assert!(ok("bluetooth_device", json!({"name": "-r", "connect": true})).is_err());
        assert_eq!(
            ok("timer", json!({"at": "17:30", "text": "call mum"})),
            Ok(())
        );
        assert!(ok("timer", json!({"at": "25:00"})).is_err());
        assert!(ok("timer", json!({"seconds": 0})).is_err());
        assert!(ok("timer", json!({"seconds": 60, "at": "17:30"})).is_err());
        assert!(ok("timer", json!({"seconds": 999_999_999})).is_err());
        assert!(ok("timer", json!({"seconds": 60, "text": "a\nb"})).is_err());
        assert!(ok("timer_cancel", json!({"id": "x;y"})).is_err());
        assert!(ok("theme_mode", json!({"mode": "blue"})).is_err());
        assert!(ok("power", json!({"action": "format"})).is_err());
        assert!(ok("calc", json!({"expr": ""})).is_err());
        for p in PROPOSALS {
            assert!(known(p), "{p}");
        }
        assert_eq!(clock("9:05"), Some((9, 5)));
        assert_eq!(clock("24:00"), None);
        assert!(is_mac("aa:bb:cc:dd:ee:ff") && !is_mac("aa:bb"));
    }

    #[test]
    fn the_wiser_commands_check_their_arguments() {
        let ok = |c: &str, a: Value| validate(&Call::new(c, a));
        assert_eq!(ok("clock", json!({})), Ok(()));
        assert!(ok("clock", json!({"place": "-x"})).is_err());
        assert_eq!(
            ok("date_math", json!({"op": "offset", "days": -30})),
            Ok(())
        );
        assert_eq!(
            ok("date_math", json!({"op": "weekday", "date": "2026-12-25"})),
            Ok(())
        );
        assert!(ok("date_math", json!({"op": "until", "date": "my birthday"})).is_err());
        assert!(ok("date_math", json!({"op": "offset"})).is_err());
        assert!(ok("date_math", json!({"op": "until"})).is_err());
        // units by their words too; no currencies, no mixed kinds
        assert_eq!(
            ok(
                "convert",
                json!({"value": 20.5, "from": "celsius", "to": "fahrenheit"})
            ),
            Ok(())
        );
        assert!(ok("convert", json!({"value": 100, "from": "usd", "to": "eur"})).is_err());
        assert!(ok("convert", json!({"value": 1, "from": "km", "to": "kg"})).is_err());
        assert!(ok("convert", json!({"value": "ten", "from": "km", "to": "mi"})).is_err());
        assert!(ok("system_info", json!({"what": "secrets"})).is_err());
        // a workspace: a number, a step, or (the Undo) Hyprland's own id
        assert_eq!(ok("workspace", json!({"step": "next"})), Ok(()));
        assert_eq!(ok("workspace", json!({"id": 13})), Ok(()));
        assert!(ok("workspace", json!({"number": 11})).is_err());
        assert!(ok("workspace", json!({"number": 2, "step": "next"})).is_err());
        assert!(ok("workspace", json!({})).is_err());
        assert_eq!(
            ok(
                "window",
                json!({"action": "close", "address": "0x55d0c1a2b3c0", "token": "a1b2c3d4e5f6"})
            ),
            Ok(())
        );
        assert_eq!(
            ok(
                "window",
                json!({"action": "move", "to_id": 12, "address": "0xabc"})
            ),
            Ok(())
        );
        assert!(ok("window", json!({"action": "move"})).is_err());
        assert!(ok("window", json!({"action": "move", "to": 0})).is_err());
        assert!(ok("window", json!({"action": "kill"})).is_err());
        assert!(
            ok(
                "window",
                json!({"action": "close", "address": "address:0x1; rm"})
            )
            .is_err()
        );
        assert!(ok("window", json!({"action": "close", "token": "x y"})).is_err());
        assert!(ok("force_quit", json!({"app": "fire;fox"})).is_err());
        assert!(ok("force_quit", json!({"app": "-9 1"})).is_err());
        assert!(ok("remember", json!({"text": ""})).is_err());
        assert!(ok("remember", json!({"text": "a\nb"})).is_err());
        assert_eq!(ok("forget", json!({"all": true})), Ok(()));
        assert_eq!(ok("forget", json!({"id": "m19a3f0b2e1003"})), Ok(()));
        assert!(ok("forget", json!({})).is_err());
        assert!(ok("forget", json!({"id": "../x"})).is_err());
        assert!(PROPOSALS.contains(&"force_quit"));
        assert!(is_address("0xABCdef0") && !is_address("0x") && !is_address("12"));
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
        // ewe Settings 0.22's labels (lib/pages.js), ewe 0.39's pages
        assert_eq!(page_label("settings", "network"), Some("Network"));
        assert_eq!(page_label("settings", "startup"), Some("Startup apps"));
        assert_eq!(
            page_label("settings", "shortcuts"),
            Some("Keyboard shortcuts")
        );
        assert_eq!(
            page_label("settings", "system"),
            Some("About this computer")
        );
        assert_eq!(page_label("quicksettings", "power"), Some("Power"));
        assert_eq!(ok("quicksettings", "remote"), Ok(()));
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
