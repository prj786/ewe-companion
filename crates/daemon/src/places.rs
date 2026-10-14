//! Folders by the words people use, and the apps behind "the browser" or
//! "a text editor": what makes "open kitty in documents" and "open
//! documents in nemo" mean something on this machine.
//!
//! - **Folders.** XDG names (documents, downloads, pictures, music, videos,
//!   desktop, templates, public, projects: `~/.config/user-dirs.dirs`, else
//!   the usual names), home, screenshots, Nextcloud; `~/…` and `/…` paths;
//!   a path under home, segment by segment and case-blind ("projects/ewe");
//!   else a folder of that name up to three levels under home, shallowest
//!   first ("ewe-companion"). Only what exists.
//! - **Default apps.** "browser", "file manager", "text editor" and the
//!   like resolve through `xdg-mime`, the same answer Settings → Default
//!   apps writes. "terminal" is kitty, ewe's terminal.
//!
//! Nothing here runs anything it finds: the island opens it, the way the
//! launcher does (`Service.qml` `launch`, `folder`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use tokio::process::Command;

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// `~/.config/user-dirs.dirs`: `XDG_DOCUMENTS_DIR="$HOME/Documents"` → DOCUMENTS.
fn user_dirs(home: &Path) -> HashMap<String, PathBuf> {
    let cfg = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"));
    user_dirs_in(&cfg, home)
}

fn user_dirs_in(cfg: &Path, home: &Path) -> HashMap<String, PathBuf> {
    let mut out = HashMap::new();
    let Ok(text) = std::fs::read_to_string(cfg.join("user-dirs.dirs")) else {
        return out;
    };
    for line in text.lines() {
        let line = line.trim();
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let Some(name) = k.strip_prefix("XDG_").and_then(|k| k.strip_suffix("_DIR")) else {
            continue;
        };
        let v = v.trim().trim_matches('"');
        let p = match v.strip_prefix("$HOME") {
            Some(rest) => home.join(rest.trim_start_matches('/')),
            None => PathBuf::from(v),
        };
        out.insert(name.to_string(), p);
    }
    out
}

/// A folder named by a word: "documents", "photos", "home".
fn named(word: &str, home: &Path) -> Option<PathBuf> {
    named_in(word, home, &user_dirs(home))
}

fn named_in(word: &str, home: &Path, dirs: &HashMap<String, PathBuf>) -> Option<PathBuf> {
    let xdg = |key: &str, fallback: &str| {
        dirs.get(key)
            .cloned()
            .unwrap_or_else(|| home.join(fallback))
    };
    let p = match word {
        "~" | "home" | "home folder" | "my home" => home.to_path_buf(),
        "documents" | "document" | "docs" => xdg("DOCUMENTS", "Documents"),
        "downloads" | "download" => xdg("DOWNLOAD", "Downloads"),
        "pictures" | "picture" | "photos" | "images" | "pics" => xdg("PICTURES", "Pictures"),
        "music" => xdg("MUSIC", "Music"),
        "videos" | "video" | "movies" => xdg("VIDEOS", "Videos"),
        "desktop" => xdg("DESKTOP", "Desktop"),
        "templates" => xdg("TEMPLATES", "Templates"),
        "public" => xdg("PUBLICSHARE", "Public"),
        "projects" | "project" => xdg("PROJECTS", "Projects"),
        "screenshots" | "screenshot" => xdg("PICTURES", "Pictures").join("Screenshots"),
        "nextcloud" => home.join("Nextcloud"),
        _ => return None,
    };
    p.exists().then_some(p)
}

/// The child of `dir` called `seg`: exactly, else ignoring case (the
/// offline matcher lower-cases what it hears).
fn child(dir: &Path, seg: &str) -> Option<PathBuf> {
    let exact = dir.join(seg);
    if exact.exists() {
        return Some(exact);
    }
    let want = seg.to_lowercase();
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .find(|e| e.file_name().to_string_lossy().to_lowercase() == want)
        .map(|e| e.path())
}

fn walk(base: &Path, rel: &str) -> Option<PathBuf> {
    let mut p = base.to_path_buf();
    for seg in rel.split('/').filter(|s| !s.is_empty() && *s != ".") {
        p = if seg == ".." {
            p.parent()?.to_path_buf()
        } else {
            child(&p, seg)?
        };
    }
    p.exists().then_some(p)
}

/// "ewe companion" and "ewe-companion" are the same folder.
fn squash(s: &str) -> String {
    s.chars()
        .filter(|c| !matches!(c, '-' | '_' | ' ' | '.'))
        .flat_map(char::to_lowercase)
        .collect()
}

const SKIP: &[&str] = &[
    "node_modules",
    "target",
    "build",
    "dist",
    "venv",
    "__pycache__",
];

/// A folder called `name` up to `depth` levels under `root`, shallowest
/// first, alphabetical within a level. Hidden folders and build trees are
/// skipped, and the walk stops after a few thousand folders.
fn find_named(root: &Path, name: &str, depth: usize) -> Option<PathBuf> {
    let want = squash(name);
    if want.is_empty() {
        return None;
    }
    let mut level = vec![root.to_path_buf()];
    let mut seen = 0usize;
    for _ in 0..depth {
        let mut next = Vec::new();
        for dir in &level {
            let Ok(rd) = std::fs::read_dir(dir) else {
                continue;
            };
            let mut kids: Vec<PathBuf> = rd
                .flatten()
                .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
                .filter(|e| {
                    let n = e.file_name().to_string_lossy().to_string();
                    !n.starts_with('.') && !SKIP.contains(&n.as_str())
                })
                .map(|e| e.path())
                .collect();
            kids.sort();
            for k in kids {
                seen += 1;
                if k.file_name()
                    .is_some_and(|n| squash(&n.to_string_lossy()) == want)
                {
                    return Some(k);
                }
                next.push(k);
            }
            if seen > 4000 {
                return None;
            }
        }
        level = next;
    }
    None
}

/// The folder or file these words name, if it exists.
pub fn resolve(words: &str) -> Option<PathBuf> {
    let home = home()?;
    let mut w = words.trim().trim_matches(['"', '\'']).trim().to_string();
    for pre in ["the ", "my "] {
        if w.get(..pre.len())
            .is_some_and(|h| h.eq_ignore_ascii_case(pre))
        {
            w = w[pre.len()..].trim().to_string();
        }
    }
    for suf in [" folder", " directory", " dir"] {
        let at = w.len().saturating_sub(suf.len());
        if w.get(at..).is_some_and(|t| t.eq_ignore_ascii_case(suf)) {
            w.truncate(at);
            w = w.trim().to_string();
        }
    }
    if w.is_empty() {
        return None;
    }
    if w == "~" {
        return Some(home);
    }
    if let Some(rest) = w.strip_prefix("~/") {
        return walk(&home, rest);
    }
    if let Some(rest) = w.strip_prefix('/') {
        return walk(Path::new("/"), rest);
    }
    let low = w.to_lowercase();
    if let Some(p) = named(&low, &home) {
        return Some(p);
    }
    // "documents/work": a named folder, then the rest under it
    if let Some((first, rest)) = w.split_once('/')
        && let Some(base) = named(&first.to_lowercase(), &home)
    {
        return walk(&base, rest);
    }
    walk(&home, &w).or_else(|| find_named(&home, &w, 3))
}

/// A path as a person reads it: `~/Documents`.
pub fn show(p: &Path) -> String {
    match home() {
        Some(h) if p == h => "~".into(),
        Some(h) => match p.strip_prefix(&h) {
            Ok(rest) => format!("~/{}", rest.display()),
            Err(_) => p.display().to_string(),
        },
        None => p.display().to_string(),
    }
}

/// The desktop id behind a kind of app: "browser" → `zen` (from
/// `zen.desktop`), "terminal" → `kitty`. None for an app's own name.
pub async fn default_app(name: &str) -> Option<String> {
    let n = name.trim().to_lowercase();
    let n = n
        .strip_prefix("the ")
        .or_else(|| n.strip_prefix("a "))
        .unwrap_or(&n);
    let mime = match n {
        "terminal" | "a terminal" | "terminal emulator" | "console" | "shell" => {
            return Some("kitty".into());
        }
        "browser" | "web browser" | "internet" => "x-scheme-handler/https",
        "file manager" | "files" | "file browser" | "file explorer" | "explorer" => {
            "inode/directory"
        }
        "text editor" | "editor" | "notepad" => "text/plain",
        "image viewer" | "photo viewer" | "picture viewer" => "image/png",
        "video player" | "movie player" => "video/mp4",
        "music player" | "audio player" => "audio/mpeg",
        "pdf viewer" | "pdf reader" | "document viewer" => "application/pdf",
        "mail" | "email" | "e-mail" | "mail app" | "email app" | "mail client" | "email client" => {
            "x-scheme-handler/mailto"
        }
        _ => return None,
    };
    let mut c = Command::new("xdg-mime");
    c.args(["query", "default", mime])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let out = tokio::time::timeout(Duration::from_secs(2), c.output())
        .await
        .ok()?
        .ok()?;
    let id = String::from_utf8(out.stdout).ok()?.trim().to_string();
    let id = id.strip_suffix(".desktop").unwrap_or(&id).to_string();
    (!id.is_empty()).then_some(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> PathBuf {
        let root = std::env::temp_dir().join(format!("ewe-places-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for d in [
            "Documents/Work",
            "Projects/ewe/ewe-companion",
            "Projects/ewe/node_modules/x",
            ".hidden/ewe-companion",
        ] {
            std::fs::create_dir_all(root.join(d)).unwrap();
        }
        std::fs::write(root.join("Documents/notes.txt"), "hi").unwrap();
        root
    }

    #[test]
    fn folders_resolve_by_their_words() {
        let root = tree();
        let h = &root;
        let none = HashMap::new();
        assert_eq!(named_in("documents", h, &none), Some(h.join("Documents")));
        assert_eq!(named_in("docs", h, &none), Some(h.join("Documents")));
        assert_eq!(named_in("music", h, &none), None, "only what exists");
        assert_eq!(
            walk(h, "projects/ewe"),
            Some(h.join("Projects/ewe")),
            "case-blind"
        );
        assert_eq!(
            walk(h, "Documents/notes.txt"),
            Some(h.join("Documents/notes.txt"))
        );
        assert_eq!(walk(h, "nope/here"), None);
        assert_eq!(
            find_named(h, "ewe companion", 3),
            Some(h.join("Projects/ewe/ewe-companion")),
            "spaces and dashes are the same; hidden folders are skipped"
        );
        assert_eq!(find_named(h, "x", 4), None, "build trees are skipped");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn user_dirs_are_read() {
        let root = std::env::temp_dir().join(format!("ewe-ud-{}", std::process::id()));
        std::fs::create_dir_all(root.join(".config")).unwrap();
        std::fs::write(
            root.join(".config/user-dirs.dirs"),
            "# comment\nXDG_DOCUMENTS_DIR=\"$HOME/Docs\"\nXDG_PROJECTS_DIR=\"/srv/p\"\n",
        )
        .unwrap();
        let d = user_dirs_in(&root.join(".config"), &root);
        assert_eq!(d.get("DOCUMENTS"), Some(&root.join("Docs")));
        assert_eq!(d.get("PROJECTS"), Some(&PathBuf::from("/srv/p")));
        let _ = std::fs::remove_dir_all(&root);
    }
}
