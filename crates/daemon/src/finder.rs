//! The finder: files and folders under home by their name ("find my
//! resume", "where is the ewe-safe folder", "find pdfs about tax").
//!
//! It walks `$HOME` with the `ignore` crate's parallel walker. Hidden files
//! and folders, build trees (`node_modules`, `target` …) and caches are
//! skipped, links are not followed, and the walk stops after
//! [`BUDGET`] (a partial answer says so). What the words ask for and how
//! well a name answers are `companion_core::find`'s rules; here the best
//! come first, then the newest.
//!
//! What it finds shows on the island as rows (Open, Show in folder, Copy
//! path, and Open terminal here for a folder); a model gets how many. Paths
//! are never logged.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use companion_core::find::{self, Kind, Query};
use ignore::{WalkBuilder, WalkState};
use serde_json::{Value, json};

use crate::commands::{Outcome, done, missed};
use crate::places;

/// The walk stops here, with what it has.
pub const BUDGET: Duration = Duration::from_millis(1500);
/// Rows on the island, at most.
const ROWS: usize = 8;
/// Matches kept while walking, before the ranking picks the best.
const KEEP: usize = 5000;

/// Folders the finder never enters: build trees and caches.
const SKIP: &[&str] = &[
    "node_modules",
    "target",
    "__pycache__",
    "venv",
    "site-packages",
    "Cache",
    "cache",
    "caches",
    "CachedData",
    "Code Cache",
    "GPUCache",
    "DawnCache",
    "ShaderCache",
];

pub struct Hit {
    pub path: PathBuf,
    pub dir: bool,
    pub rank: u8,
    pub modified: SystemTime,
}

impl Hit {
    /// Seconds since 1970, for JSON.
    pub fn modified_s(&self) -> u64 {
        self.modified
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

/// Walk `root` for `q` within `budget`: the best `limit` hits, and whether
/// the time ran out first.
pub fn walk(root: &Path, q: &Query, limit: usize, budget: Duration) -> (Vec<Hit>, bool) {
    let deadline = Instant::now() + budget;
    let hits: Mutex<Vec<Hit>> = Mutex::new(Vec::new());
    let partial = AtomicBool::new(false);
    let threads = std::thread::available_parallelism().map_or(2, |n| n.get().min(4));
    WalkBuilder::new(root)
        .standard_filters(false)
        .hidden(true)
        .follow_links(false)
        .max_depth(Some(12))
        .threads(threads)
        .filter_entry(|e| {
            !(e.file_type().is_some_and(|t| t.is_dir())
                && SKIP.contains(&e.file_name().to_string_lossy().as_ref()))
        })
        .build_parallel()
        .run(|| {
            Box::new(|entry| {
                if Instant::now() > deadline {
                    partial.store(true, Ordering::Relaxed);
                    return WalkState::Quit;
                }
                let Ok(e) = entry else {
                    return WalkState::Continue;
                };
                if e.depth() == 0 {
                    return WalkState::Continue;
                }
                let dir = e.file_type().is_some_and(|t| t.is_dir());
                let name = e.file_name().to_string_lossy();
                if let Some(rank) = find::rank(&name, dir, q) {
                    let modified = e
                        .metadata()
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .unwrap_or(UNIX_EPOCH);
                    let mut h = hits.lock().unwrap_or_else(|p| p.into_inner());
                    if h.len() < KEEP {
                        h.push(Hit {
                            path: e.path().to_path_buf(),
                            dir,
                            rank,
                            modified,
                        });
                    }
                }
                WalkState::Continue
            })
        });
    let mut hits = hits.into_inner().unwrap_or_else(|p| p.into_inner());
    hits.sort_by(|a, b| {
        b.rank
            .cmp(&a.rank)
            .then(b.modified.cmp(&a.modified))
            .then(a.path.cmp(&b.path))
    });
    hits.truncate(limit);
    (hits, partial.load(Ordering::Relaxed))
}

/// The finder under home, off the async threads.
pub async fn find(q: Query, limit: usize) -> (Vec<Hit>, bool) {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return (vec![], false);
    };
    tokio::task::spawn_blocking(move || walk(&home, &q, limit, BUDGET))
        .await
        .unwrap_or_default()
}

/// The best folder these words name anywhere under home, for "open the
/// ewe-safe folder" when `places` found none close by.
pub async fn best_folder(words: &str) -> Option<PathBuf> {
    let mut q = find::parse(words);
    q.kind = Kind::Folder;
    if q.words.is_empty() {
        return None;
    }
    find(q, 1).await.0.into_iter().next().map(|h| h.path)
}

fn row(h: &Hit) -> Value {
    let path = h.path.to_string_lossy().to_string();
    let name = h
        .path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.clone());
    let parent = h.path.parent().unwrap_or(&h.path);
    let mut actions = vec![
        if h.dir {
            json!({"label": "Open", "command": "open_folder", "args": {"path": path}})
        } else {
            json!({"label": "Open", "command": "open_file", "args": {"path": path}})
        },
        json!({"label": "Show in folder", "command": "open_folder",
               "args": {"path": parent.to_string_lossy()}}),
        json!({"label": "Copy path", "copy": path}),
    ];
    if h.dir {
        actions.push(
            json!({"label": "Open terminal here", "command": "launch_app",
                            "args": {"app": "terminal", "path": path}}),
        );
    }
    json!({"title": name, "detail": places::show(parent), "icon": if h.dir { "folder" } else { "file" },
           "actions": actions})
}

/// `find_files`: the rows, or a miss (the chat then treats the words as a
/// question: "find a good pdf reader").
pub async fn run(a: &Value) -> Outcome {
    let words = a["query"].as_str().unwrap_or("").trim();
    let mut q = find::parse(words);
    match a.get("kind").and_then(Value::as_str) {
        Some("folder") => q.kind = Kind::Folder,
        Some("file") => q.kind = Kind::File,
        _ => {}
    }
    if q.is_empty() {
        return missed("Say what to look for: a word of its name, or a kind like pdfs.");
    }
    let (hits, partial) = find(q, ROWS).await;
    let n = hits.len();
    let what = words
        .strip_prefix("my ")
        .or_else(|| words.strip_prefix("the "))
        .unwrap_or(words);
    if n == 0 {
        return missed(if partial {
            format!("I found nothing called {what} in the time I had to look.")
        } else {
            format!("I found nothing called {what} in your home folder.")
        });
    }
    let line = match (n, partial) {
        (1, false) => "Found it:".to_string(),
        (_, false) => format!("Found {n}, best first:"),
        (_, true) => format!("Found {n} before I had to stop looking:"),
    };
    Outcome {
        rows: hits.iter().map(row).collect(),
        ..done(line, None)
    }
    .private(format!(
        "Found {n} files or folders; they show on the island."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("ewe-finder-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for d in [
            "Documents/Taxes",
            "Projects/ewe/ewe-safe/src",
            "Projects/ewe/node_modules/ewe-safe",
            ".hidden/ewe-safe",
            ".cache/x",
        ] {
            std::fs::create_dir_all(root.join(d)).unwrap();
        }
        for f in [
            "Documents/Taxes/tax-2025.pdf",
            "Documents/Taxes/tax-notes.txt",
            "Documents/resume.odt",
            "Projects/ewe/ewe-safe/README.md",
        ] {
            std::fs::write(root.join(f), "x").unwrap();
        }
        root
    }

    #[test]
    fn it_finds_by_name_and_skips_what_it_should() {
        let root = tree("names");
        let names = |q: &str| -> Vec<String> {
            walk(&root, &find::parse(q), 10, Duration::from_secs(5))
                .0
                .iter()
                .map(|h| h.path.strip_prefix(&root).unwrap().display().to_string())
                .collect()
        };
        assert_eq!(
            names("where is the ewe-safe folder"),
            ["Projects/ewe/ewe-safe"]
        );
        assert_eq!(names("pdfs about tax"), ["Documents/Taxes/tax-2025.pdf"]);
        assert_eq!(names("my resume"), ["Documents/resume.odt"]);
        // the folder named exactly first, then what starts with the words
        assert_eq!(
            names("taxes"),
            ["Documents/Taxes"],
            "every word must be in the name"
        );
        assert_eq!(
            names("tax").len(),
            3,
            "a folder and two files start with tax"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_budget_stops_the_walk() {
        let root = tree("budget");
        let (_, partial) = walk(&root, &find::parse("tax"), 10, Duration::ZERO);
        assert!(partial);
        let _ = std::fs::remove_dir_all(&root);
    }
}
