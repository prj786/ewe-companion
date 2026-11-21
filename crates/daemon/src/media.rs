//! Media with ewe Player (ewe-apps): play from the user's own music library,
//! play a link (YouTube, Twitch, the BBC and many other sites go through
//! yt-dlp, ewe 0.46; Vimeo serves DRM or wants a sign-in, so it is said
//! plainly instead of tried), say
//! what is playing, and why a file or a link will not play. Also the two
//! file acts the finder's rows use: show a file in ewe Files, open it with
//! another app.
//!
//! ewe's apps are started by the island (`do app`: the bin's name must be
//! ewe-player, ewe-files-app or ewe-viewer, the args are argv), so they
//! belong to the desktop, not to this daemon's unit. An older island
//! without `app` gets the launcher's `launch` with one argument.
//!
//! Names stay local: a track's or a file's name is shown on the island
//! only (`Outcome::private`); a model hears a count or the codecs.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use companion_core::commands::Call;
use companion_core::media::{self, Probe};
use serde_json::{Value, json};
use tokio::process::Command;
use tracing::info;

use crate::commands::{Outcome, done, failed, missed};
use crate::hub::Hub;
use crate::{context, places};

/// ewe's own apps the island may start with arguments (`do app`).
pub const APPS: &[(&str, &str)] = &[
    ("ewe-player", "ewe Player"),
    ("ewe-files-app", "ewe Files"),
    ("ewe-viewer", "ewe Viewer"),
];

fn app_name(bin: &str) -> &'static str {
    APPS.iter()
        .find(|(b, _)| *b == bin)
        .map_or("the app", |(_, n)| *n)
}

/// Start one of ewe's apps with these arguments, through the island.
pub async fn start(hub: &Arc<Hub>, bin: &str, args: Vec<String>) -> Result<(), String> {
    let Some(path) = context::tool(bin) else {
        return Err(format!(
            "{} is not installed: it comes with ewe's apps (ewe-apps) in Komble.",
            app_name(bin)
        ));
    };
    let path = path.to_string_lossy().to_string();
    match hub
        .shell_do("app", json!({"bin": path, "args": args}))
        .await
    {
        Ok(_) => Ok(()),
        // an island from before 0.7: the launcher's way, one argument
        Err(e) if e.starts_with("The island cannot do") && args.len() <= 1 => {
            let home = std::env::var("HOME").unwrap_or_default();
            hub.shell_do(
                "launch",
                json!({"app": bin, "path": args.first(), "dir": home}),
            )
            .await
            .map(|_| ())
        }
        Err(e) if e.starts_with("The island cannot do") => Err(
            "The Shaun plugin is older than this Shaun: update ewe (Komble → Updates) to play several files at once."
                .into(),
        ),
        Err(e) => Err(e),
    }
}

/// What the user copied, as text (`wl-paste`, on the island's display).
pub async fn clipboard(hub: &Arc<Hub>) -> Option<String> {
    let bin = context::tool("wl-paste")?;
    let mut c = Command::new(bin);
    c.args(["--no-newline", "--type", "text"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let wd = hub.env().wayland;
    if !wd.is_empty() {
        c.env("WAYLAND_DISPLAY", wd);
    }
    let out = tokio::time::timeout(Duration::from_secs(2), c.output())
        .await
        .ok()?
        .ok()?;
    let mut t = String::from_utf8_lossy(&out.stdout).to_string();
    if t.len() > 64 * 1024 {
        let mut cut = 64 * 1024;
        while !t.is_char_boundary(cut) {
            cut -= 1;
        }
        t.truncate(cut);
    }
    let t = t.trim().to_string();
    (!t.is_empty()).then_some(t)
}

async fn output(bin: &str, args: &[&str], budget: Duration) -> Option<String> {
    let path = context::tool(bin)?;
    let mut c = Command::new(path);
    c.args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let out = tokio::time::timeout(budget, c.output()).await.ok()?.ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).to_string())
}

// ── the music library ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Track {
    path: String,
    title: String,
    artist: String,
    album: String,
    genre: String,
}

const AUDIO: &[&str] = &[
    "mp3", "flac", "ogg", "opus", "m4a", "aac", "wav", "wma", "alac", "aiff", "ape", "mka", "oga",
];

/// The music root ewe Player scans: `$EWE_MUSIC_DIR`, else the xdg Music
/// folder.
fn music_root() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("EWE_MUSIC_DIR").filter(|d| !d.is_empty()) {
        return Some(PathBuf::from(d));
    }
    places::user_dir("MUSIC", "Music")
}

/// ewe Player's own index (`$XDG_CACHE_HOME/ewe-player/music-index.json`:
/// tags read once by the app), else the music folder's files by name.
fn library() -> Vec<Track> {
    let cache = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")));
    if let Some(c) = cache
        && let Ok(text) = std::fs::read(c.join("ewe-player/music-index.json"))
        && let Ok(v) = serde_json::from_slice::<Value>(&text)
    {
        let s = |t: &Value, k: &str| t[k].as_str().unwrap_or("").to_string();
        let tracks: Vec<Track> = v["tracks"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|t| {
                let path = t["path"].as_str()?.to_string();
                Some(Track {
                    title: s(t, "title"),
                    artist: s(t, "artist"),
                    album: s(t, "album"),
                    genre: s(t, "genre"),
                    path,
                })
            })
            .filter(|t| Path::new(&t.path).is_file())
            .collect();
        if !tracks.is_empty() {
            return tracks;
        }
    }
    let Some(root) = music_root() else {
        return vec![];
    };
    let mut out = Vec::new();
    let mut todo = vec![(root, 0)];
    while let Some((d, depth)) = todo.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        let mut entries: Vec<_> = rd.flatten().collect();
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let p = e.path();
            if e.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            if p.is_dir() && depth < 8 {
                todo.push((p, depth + 1));
            } else if p
                .extension()
                .and_then(|x| x.to_str())
                .is_some_and(|x| AUDIO.contains(&x.to_ascii_lowercase().as_str()))
            {
                let title = p
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let album = p
                    .parent()
                    .and_then(|p| p.file_name())
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                out.push(Track {
                    path: p.to_string_lossy().to_string(),
                    title,
                    artist: String::new(),
                    album,
                    genre: String::new(),
                });
            }
            if out.len() >= 50_000 {
                return out;
            }
        }
    }
    out
}

/// The tracks every word of the query fits (title, artist, album, genre or
/// the file's path), a whole artist, album or genre first.
fn pick(tracks: &[Track], query: &str) -> Vec<Track> {
    let q = query.to_lowercase();
    let words: Vec<&str> = q
        .split_whitespace()
        .filter(|w| {
            ![
                "some", "songs", "song", "music", "tracks", "by", "the", "of", "a",
            ]
            .contains(w)
        })
        .collect();
    if words.is_empty() {
        return tracks.to_vec();
    }
    let phrase = words.join(" ");
    let mut found: Vec<(u8, &Track)> = tracks
        .iter()
        .filter_map(|t| {
            let all = format!(
                "{} {} {} {} {}",
                t.title, t.artist, t.album, t.genre, t.path
            )
            .to_lowercase();
            if !words.iter().all(|w| all.contains(w)) {
                return None;
            }
            let rank = if [&t.artist, &t.album, &t.genre]
                .iter()
                .any(|f| f.to_lowercase() == phrase)
            {
                2
            } else if t.title.to_lowercase() == phrase {
                1
            } else {
                0
            };
            Some((rank, t))
        })
        .collect();
    // a stable sort keeps the library's order (album, then track) in a rank
    found.sort_by_key(|a| std::cmp::Reverse(a.0));
    found.into_iter().map(|(_, t)| t.clone()).collect()
}

/// Most tracks one ask plays.
const PLAY_MAX: usize = 200;

/// `play_local {query | path}`: these songs from the user's own music, in
/// ewe Player.
pub async fn play_local(hub: &Arc<Hub>, a: &Value) -> Outcome {
    if let Some(p) = a.get("path").and_then(Value::as_str) {
        let Some(path) = places::resolve(p) else {
            return failed("That file is not there any more.");
        };
        return match start(hub, "ewe-player", vec![path.to_string_lossy().to_string()]).await {
            Ok(()) => done(
                format!(
                    "Playing {} in ewe Player",
                    path.file_name()
                        .map(|n| n.to_string_lossy())
                        .unwrap_or_default()
                ),
                None,
            )
            .private("Playing a song from the music library in ewe Player."),
            Err(e) => failed(e),
        };
    }
    let q = a.get("query").and_then(Value::as_str).unwrap_or("").trim();
    let tracks = tokio::task::spawn_blocking(library)
        .await
        .unwrap_or_default();
    if tracks.is_empty() {
        let root = music_root()
            .map(|r| places::show(&r))
            .unwrap_or_else(|| "~/Music".into());
        return missed(format!("There is no music in {root} yet."));
    }
    let found = pick(&tracks, q);
    if found.is_empty() {
        return missed(format!("Nothing in your music library fits {q}."));
    }
    let n = found.len().min(PLAY_MAX);
    let args: Vec<String> = found
        .iter()
        .take(PLAY_MAX)
        .map(|t| t.path.clone())
        .collect();
    if let Err(e) = start(hub, "ewe-player", args).await {
        return failed(e);
    }
    info!(tracks = n, "playing from the library");
    let what = if q.is_empty() {
        "your music".to_string()
    } else {
        q.to_string()
    };
    let line = match n {
        1 => format!("Playing {} in ewe Player", found[0].title),
        n => format!("Playing {n} songs for {what} in ewe Player:"),
    };
    let rows = found
        .iter()
        .take(6)
        .map(|t| {
            let detail = [t.artist.as_str(), t.album.as_str()]
                .iter()
                .filter(|s| !s.is_empty())
                .copied()
                .collect::<Vec<_>>()
                .join(" · ");
            json!({"title": if t.title.is_empty() { &t.path } else { &t.title },
                   "detail": detail, "icon": "music",
                   "actions": [{"label": "Play", "command": "play_local", "args": {"path": t.path}},
                               {"label": "Show in folder", "command": "show_file", "args": {"path": t.path}}]})
        })
        .collect();
    Outcome {
        rows,
        ..done(line, None)
    }
    .private(format!(
        "Playing {n} songs from the music library in ewe Player."
    ))
}

// ── a link ──────────────────────────────────────────────────────────────────

/// `play_link {url?}`: a video or song link in ewe Player (the copied one
/// when none is given).
pub async fn play_link(hub: &Arc<Hub>, a: &Value) -> Outcome {
    let url = match a.get("url").and_then(Value::as_str) {
        Some(u) => u.to_string(),
        None => match clipboard(hub).await {
            Some(t) if companion_core::commands::link_ok(t.trim()) => t.trim().to_string(),
            Some(_) => {
                return failed(
                    "What you copied is not a link. Copy the video's address, then ask again.",
                );
            }
            None => {
                return failed(
                    "Copy the video's link first (in the browser: right-click → Copy link), then ask again.",
                );
            }
        },
    };
    let site = site_of(&url);
    // ewe Player answers "This video is locked" for these: say it here
    if locked_site(&site) {
        return Outcome {
            action: Some((
                "Open it in the browser".into(),
                Call::new("open_url", json!({"url": url})),
            )),
            ..done(locked_line(&site), None)
        };
    }
    match start(hub, "ewe-player", vec![url.clone()]).await {
        Ok(()) => done(format!("Playing the {site} link in ewe Player"), None)
            .private(format!("Playing a {site} link in ewe Player.")),
        Err(e) if context::tool("ewe-player").is_none() => Outcome {
            action: Some((
                "Open it in the browser".into(),
                Call::new("open_url", json!({"url": url})),
            )),
            ..failed(e)
        },
        Err(e) => failed(e),
    }
}

/// Sites whose videos ewe Player cannot get (ewe-apps' links map,
/// 2026-11-21: Vimeo's anonymous player serves FairPlay DRM, vimeo.com
/// wants a sign-in).
fn locked_site(site: &str) -> bool {
    matches!(site, "Vimeo" | "Netflix")
}

fn locked_line(site: &str) -> String {
    format!(
        "This {site} video is locked: {site} protects its videos (DRM) or wants a sign-in, so it plays only on the site itself. Open it in the browser."
    )
}

/// Whether yt-dlp's `-J` says the video is protected: `_has_drm`, or
/// every format with video marked `has_drm`.
fn drm_in(info: &Value) -> bool {
    if info["_has_drm"] == true {
        return true;
    }
    let video: Vec<&Value> = info["formats"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|f| f["vcodec"].as_str() != Some("none"))
        .collect();
    !video.is_empty() && video.iter().all(|f| f["has_drm"] == true)
}

/// A link's site in a word: "YouTube", "vimeo.com".
fn site_of(url: &str) -> String {
    let host = url
        .split("://")
        .nth(1)
        .unwrap_or(url)
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .trim_start_matches("www.")
        .trim_start_matches("m.")
        .to_lowercase();
    match host.as_str() {
        "youtube.com" | "youtu.be" | "music.youtube.com" => "YouTube".into(),
        "vimeo.com" | "player.vimeo.com" => "Vimeo".into(),
        "netflix.com" => "Netflix".into(),
        "bbc.co.uk" | "bbc.com" => "BBC".into(),
        "twitch.tv" => "Twitch".into(),
        "soundcloud.com" => "SoundCloud".into(),
        h => h.to_string(),
    }
}

// ── what is playing ─────────────────────────────────────────────────────────

/// A player's name as people say it: `ewe-player` → ewe Player,
/// `firefox.instance_1_23` → Firefox.
fn player_name(p: &str) -> String {
    let base = p.split('.').next().unwrap_or(p);
    match base {
        "ewe-player" => "ewe Player".into(),
        "ewe-browser" | "ewe_browser" => "ewe Browser".into(),
        "spotify" => "Spotify".into(),
        "firefox" => "Firefox".into(),
        "chromium" => "Chromium".into(),
        "chrome" | "google-chrome" => "Chrome".into(),
        "vlc" => "VLC".into(),
        "mpv" => "mpv".into(),
        "kdeconnect" => "your phone".into(),
        b => {
            let mut c = b.chars();
            c.next()
                .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                .unwrap_or_default()
        }
    }
}

/// `now_playing`: what plays, and where (MPRIS through playerctl; WebKit's
/// own players are skipped: ewe Player's is `ewe-player`).
pub async fn now_playing() -> Outcome {
    let Some(list) = output("playerctl", &["--list-all"], Duration::from_secs(3)).await else {
        return done("Nothing is playing.", None).line_is_facts();
    };
    let players: Vec<&str> = list
        .lines()
        .map(str::trim)
        .filter(|p| !p.is_empty() && !p.starts_with("org.webkit"))
        .collect();
    let mut paused = None;
    for p in &players {
        let Some(m) = output(
            "playerctl",
            &[
                "-p",
                p,
                "metadata",
                "--format",
                "{{status}}\t{{title}}\t{{artist}}\t{{album}}",
            ],
            Duration::from_secs(3),
        )
        .await
        else {
            continue;
        };
        let f: Vec<&str> = m.trim_end_matches('\n').split('\t').collect();
        let (status, title, artist) = (
            f.first().copied().unwrap_or(""),
            f.get(1).copied().unwrap_or(""),
            f.get(2).copied().unwrap_or(""),
        );
        let who = player_name(p);
        let what = match (title, artist) {
            ("", _) => "something".to_string(),
            (t, "") => t.to_string(),
            (t, a) => format!("{t} — {a}"),
        };
        if status == "Playing" {
            return Outcome {
                action: Some((
                    "Pause".into(),
                    Call::new("media", json!({"action": "play_pause"})),
                )),
                ..done(format!("Playing: {what} ({who})"), None)
            }
            .private(format!("Something is playing in {who}."))
            .facts(format!("Something is playing in {who}."));
        }
        if status == "Paused" && paused.is_none() {
            paused = Some((what, who));
        }
    }
    match paused {
        Some((what, who)) => Outcome {
            action: Some((
                "Play".into(),
                Call::new("media", json!({"action": "play_pause"})),
            )),
            ..done(format!("Paused: {what} ({who})"), None)
        }
        .private(format!("Something is paused in {who}."))
        .facts(format!("Nothing plays; something is paused in {who}.")),
        None => done("Nothing is playing.", None).line_is_facts(),
    }
}

// ── why will this not play ──────────────────────────────────────────────────

/// What "this" is when the words name nothing: the link or path copied,
/// else what ewe Player has open (its MPRIS url).
async fn what_to_check(hub: &Arc<Hub>) -> Option<String> {
    if let Some(t) = clipboard(hub).await {
        let t = t.lines().next().unwrap_or("").trim().to_string();
        let t = t.strip_prefix("file://").map(str::to_string).unwrap_or(t);
        if companion_core::commands::link_ok(&t) || places::resolve(&t).is_some_and(|p| p.is_file())
        {
            return Some(t);
        }
    }
    let url = output(
        "playerctl",
        &["-p", "ewe-player", "metadata", "xesam:url"],
        Duration::from_secs(2),
    )
    .await?;
    let url = url.trim();
    let url = url.strip_prefix("file://").unwrap_or(url);
    (!url.is_empty()).then(|| url.to_string())
}

/// Whether GStreamer has this element (`gst-inspect-1.0 --exists`).
async fn has_element(name: &str) -> bool {
    let Some(bin) = context::tool("gst-inspect-1.0") else {
        return false;
    };
    let mut c = Command::new(bin);
    c.args(["--exists", name])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    matches!(
        tokio::time::timeout(Duration::from_secs(5), c.status()).await,
        Ok(Ok(s)) if s.success()
    )
}

/// `ffprobe` of a file or a link: its streams, or why it could not tell.
async fn probe(target: &str) -> Result<Probe, String> {
    let Some(bin) = context::tool("ffprobe") else {
        return Err("ffprobe (ffmpeg) is not installed, so I cannot look inside it.".into());
    };
    let mut c = Command::new(bin);
    c.args([
        "-v",
        "error",
        "-print_format",
        "json",
        "-show_streams",
        "-show_format",
        "--",
        target,
    ])
    .stdin(Stdio::null())
    .stderr(Stdio::null())
    .kill_on_drop(true);
    let out = match tokio::time::timeout(Duration::from_secs(8), c.output()).await {
        Ok(Ok(o)) => o,
        _ => return Err("It did not answer in time.".into()),
    };
    let v: Value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
    if !out.status.success() || v["format"].is_null() {
        return Err(
            "It is not a video or song I can read: it may be damaged, or only partly downloaded."
                .into(),
        );
    }
    Ok(media::parse_ffprobe(&v))
}

/// `why_no_play {path | url}`: the codecs in words, what this computer can
/// decode, and the fix.
pub async fn why_no_play(hub: &Arc<Hub>, a: &Value) -> Outcome {
    let target = match (
        a.get("path").and_then(Value::as_str),
        a.get("url").and_then(Value::as_str),
    ) {
        (Some(p), _) => p.to_string(),
        (None, Some(u)) => u.to_string(),
        (None, None) => match what_to_check(hub).await {
            Some(t) => t,
            None => {
                return failed(
                    "Which one? Copy the file or its link and ask again, or say \"why won't ~/Videos/film.mkv play\".",
                );
            }
        },
    };
    if companion_core::commands::link_ok(&target) {
        return why_link(&target).await;
    }
    let Some(path) = places::resolve(&target).filter(|p| p.is_file()) else {
        return failed(format!("I can't find {target}."));
    };
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let p = match probe(&path.to_string_lossy()).await {
        Ok(p) => p,
        Err(e) => return failed(e).private("I could not read that file."),
    };
    let mut out = explain(&p).await;
    let path_s = path.to_string_lossy().to_string();
    out.rows = vec![
        json!({"title": name, "detail": places::show(path.parent().unwrap_or(&path)),
        "icon": "file", "actions": [
            {"label": "Open with another app", "command": "open_with", "args": {"path": path_s}},
            {"label": "Show in folder", "command": "show_file", "args": {"path": path_s}}]}),
    ];
    out
}

/// A link: a web page (yt-dlp plays it in ewe Player), or a media file
/// (probed like one).
async fn why_link(url: &str) -> Outcome {
    let site = site_of(url);
    if media::looks_like_page(url) {
        let Some(ytdlp) = context::tool("yt-dlp") else {
            return Outcome {
                action: Some((
                    "Find yt-dlp in Komble".into(),
                    Call::new("open", json!({"what": "komble", "page": "search=yt-dlp"})),
                )),
                ..done(
                    format!(
                        "This is a {site} web page, not a video file. ewe Player plays such links through yt-dlp, which is not installed."
                    ),
                    None,
                )
            }
            .line_is_facts();
        };
        let mut c = Command::new(ytdlp);
        c.args([
            "-J",
            "--no-playlist",
            "--skip-download",
            "--no-warnings",
            "--",
            url,
        ])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
        let info: Option<Value> =
            match tokio::time::timeout(Duration::from_secs(20), c.output()).await {
                Ok(Ok(o)) if o.status.success() => serde_json::from_slice(&o.stdout).ok(),
                _ => None,
            };
        // a protected video, or a site that only serves those
        if locked_site(&site) || info.as_ref().is_some_and(drm_in) {
            return Outcome {
                action: Some((
                    "Open in the browser".into(),
                    Call::new("open_url", json!({"url": url})),
                )),
                ..done(locked_line(&site), None)
            }
            .line_is_facts();
        }
        return if info.is_some() {
            Outcome {
                action: Some((
                    "Play in ewe Player".into(),
                    Call::new("play_link", json!({"url": url})),
                )),
                ..done(
                    format!(
                        "This is a {site} web page, not a video file, so a plain player cannot open it. ewe Player plays it through yt-dlp."
                    ),
                    None,
                )
            }
            .line_is_facts()
        } else {
            Outcome {
                action: Some((
                    "Open in the browser".into(),
                    Call::new("open_url", json!({"url": url})),
                )),
                ..done(
                    format!(
                        "This is a {site} web page, and yt-dlp cannot read a video from it: it may need a sign-in, be private, or the site is not one yt-dlp knows. Watch it in the browser."
                    ),
                    None,
                )
            }
            .line_is_facts()
        };
    }
    match probe(url).await {
        Ok(p) => explain(&p).await,
        Err(_) => done(
            "I could not reach it: the link may be broken, or the server wants a sign-in.",
            None,
        )
        .line_is_facts(),
    }
}

/// The codecs, what decodes them here, and what to do.
async fn explain(p: &Probe) -> Outcome {
    let what = format!(
        "This is {} with {}.",
        media::container_words(&p.container),
        media::describe(p)
    );
    if p.streams.is_empty() {
        return done(format!("{what} There is nothing in it to play."), None).line_is_facts();
    }
    let libav = has_element("avdec_h264").await;
    let mut missing: Vec<(&str, bool)> = Vec::new(); // (words, gst-libav would decode it)
    for s in &p.streams {
        // plain PCM needs no decoder; a codec this table does not know is
        // not blamed
        if s.decoders.is_empty() {
            continue;
        }
        let mut ok = false;
        for d in &s.decoders {
            if has_element(d).await {
                ok = true;
                break;
            }
        }
        if !ok {
            let in_libav = s.decoders.iter().any(|d| d.starts_with("avdec_"));
            if !missing.iter().any(|(w, _)| *w == s.words) {
                missing.push((&s.words, in_libav));
            }
        }
    }
    if missing.is_empty() {
        return done(
            format!(
                "{what} This computer can decode all of it, so ewe Player should play it. If it still does not, the file may be damaged or only partly downloaded: try another app."
            ),
            None,
        )
        .line_is_facts();
    }
    let names: Vec<&str> = missing.iter().map(|(w, _)| *w).collect();
    let fixable = !libav && missing.iter().all(|(_, l)| *l);
    if fixable {
        return Outcome {
            action: Some((
                "Get gst-libav in Komble".into(),
                Call::new("open", json!({"what": "komble", "page": "search=gst-libav"})),
            )),
            ..done(
                format!(
                    "{what} This computer has no decoder for {}: it comes with gst-libav (FFmpeg's decoders for GStreamer), which is not installed. Updating ewe installs it, or get it in Komble.",
                    names.join(" or ")
                ),
                None,
            )
        }
        .line_is_facts();
    }
    done(
        format!(
            "{what} ewe Player has no decoder for {} on this computer. Open it with another app: mpv plays nearly everything.",
            names.join(" or ")
        ),
        None,
    )
    .line_is_facts()
}

// ── files: show one, open it with another app ───────────────────────────────

/// `show_file {path}`: the file selected in ewe Files, else its folder in
/// the file manager.
pub async fn show_file(hub: &Arc<Hub>, path: &str) -> Outcome {
    let Some(p) = places::resolve(path) else {
        return failed("That file is not there any more.");
    };
    if context::tool("ewe-files-app").is_some() {
        let arg = format!("--select={}", p.to_string_lossy());
        if start(hub, "ewe-files-app", vec![arg]).await.is_ok() {
            let dir = p.parent().unwrap_or(&p);
            return done(format!("Opened {} in ewe Files", places::show(dir)), None);
        }
    }
    crate::commands::show_folder(hub, &p).await
}

/// `open_with {path}`: ewe Files' "Open with" list for the file, else mpv
/// for a video or song.
pub async fn open_with(hub: &Arc<Hub>, path: &str) -> Outcome {
    let Some(p) = places::resolve(path).filter(|p| p.is_file()) else {
        return failed("That file is not there any more.");
    };
    let path_s = p.to_string_lossy().to_string();
    if context::tool("ewe-files-app").is_some()
        && start(hub, "ewe-files-app", vec![format!("--open-with={path_s}")])
            .await
            .is_ok()
    {
        return done("Pick an app in ewe Files' Open with list", None);
    }
    let dir = p.parent().unwrap_or(&p).to_string_lossy().to_string();
    match hub
        .shell_do("launch", json!({"app": "mpv", "path": path_s, "dir": dir}))
        .await
    {
        Ok(_) => done("Opened it in mpv", None),
        Err(e) => failed(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(title: &str, artist: &str, album: &str, genre: &str) -> Track {
        Track {
            path: format!("/m/{artist}/{album}/{title}.flac"),
            title: title.into(),
            artist: artist.into(),
            album: album.into(),
            genre: genre.into(),
        }
    }

    #[test]
    fn the_library_is_searched_by_every_word() {
        let lib = vec![
            t("So What", "Miles Davis", "Kind of Blue", "Jazz"),
            t("Teardrop", "Massive Attack", "Mezzanine", "Trip-Hop"),
            t("Blue in Green", "Miles Davis", "Kind of Blue", "Jazz"),
            t("Jazz Thing", "Gang Starr", "Mo' Better Blues", "Hip-Hop"),
        ];
        let jazz = pick(&lib, "jazz");
        assert_eq!(jazz.len(), 3);
        assert_eq!(jazz[0].title, "So What", "a whole genre first");
        assert_eq!(jazz[2].title, "Jazz Thing");
        assert_eq!(pick(&lib, "massive attack").len(), 1);
        assert_eq!(pick(&lib, "songs by miles davis").len(), 2);
        assert_eq!(pick(&lib, "").len(), 4, "all of it");
        assert!(pick(&lib, "polka").is_empty());
    }

    #[test]
    fn protected_videos_are_said_plainly() {
        use serde_json::json;
        assert!(drm_in(&json!({"_has_drm": true})));
        assert!(drm_in(&json!({"formats": [
            {"vcodec": "avc1", "has_drm": true}, {"vcodec": "none", "acodec": "mp4a"}]})));
        assert!(!drm_in(&json!({"formats": [
            {"vcodec": "avc1", "has_drm": true}, {"vcodec": "vp9"}]})));
        assert!(!drm_in(&json!({"formats": []})));
        assert!(locked_site(&site_of("https://vimeo.com/76979871")));
        assert!(locked_site(&site_of("https://player.vimeo.com/video/1")));
        assert!(!locked_site(&site_of("https://www.twitch.tv/videos/1")));
        assert_eq!(site_of("https://www.bbc.co.uk/news/av/x"), "BBC");
    }

    #[test]
    fn sites_and_players_in_words() {
        assert_eq!(site_of("https://www.youtube.com/watch?v=x"), "YouTube");
        assert_eq!(site_of("https://youtu.be/x"), "YouTube");
        assert_eq!(site_of("https://example.org/a.mp4"), "example.org");
        assert_eq!(player_name("ewe-player"), "ewe Player");
        assert_eq!(player_name("firefox.instance_1_23"), "Firefox");
        assert_eq!(player_name("rhythmbox"), "Rhythmbox");
    }
}
