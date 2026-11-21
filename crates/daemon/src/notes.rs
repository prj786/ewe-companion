//! The user's own notes: a folder (`~/Documents/Shaun/`, or
//! `$EWE_COMPANION_NOTES`) of Markdown and plain-text files that Shaun
//! searches with the same index as the built-in pack, so "where is the
//! printer" finds the note that says so, offline, and a model gets it with
//! the question like any topic.
//!
//! - **Optional and cheap.** No folder, no cost: one `stat` per question.
//!   With one, the pack is rebuilt only when a file changed (the folder's
//!   and the files' times and sizes, checked at most every two seconds).
//! - **Bounded.** `.md`, `.markdown` and `.txt` files, two folders deep, 200
//!   files, 64 KiB each (the rest is left out); hidden files skipped.
//! - **No secrets to a model.** A paragraph that looks like a password, a
//!   PIN, a key or a card number (`memory::looks_secret`) is left out of
//!   the index, and "learn this" refuses to write one.
//!
//! "Shaun, learn this: …" ([`learn`]) writes a note there; Undo deletes it
//! while it still holds what was written.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime};

use companion_core::knowledge::{self, Pack, Topic};
use serde_json::{Value, json};
use tracing::info;

use crate::commands::{Outcome, done, failed};
use crate::hub::Undo;
use crate::{memory, places};

const MAX_FILES: usize = 200;
const MAX_BYTES: usize = 64 * 1024;
const RECHECK: Duration = Duration::from_secs(2);

/// Bumped when Shaun writes or deletes a note itself, so the next question
/// sees it at once (an edit by hand waits for the next check).
static WROTE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn changed() {
    WROTE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

/// The notes folder (it may not exist).
pub fn dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("EWE_COMPANION_NOTES") {
        return Some(PathBuf::from(d));
    }
    Some(places::user_dir("DOCUMENTS", "Documents")?.join("Shaun"))
}

fn is_note(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| matches!(e.to_ascii_lowercase().as_str(), "md" | "markdown" | "txt"))
}

/// Every note file (two folders deep, hidden ones skipped), with its time
/// and size: what decides whether the index is still right.
fn files(dir: &Path) -> Vec<(PathBuf, SystemTime, u64)> {
    let mut out = Vec::new();
    let mut todo = vec![(dir.to_path_buf(), 0usize)];
    while let Some((d, depth)) = todo.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            if out.len() >= MAX_FILES {
                break;
            }
            if e.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            let p = e.path();
            let Ok(m) = e.metadata() else { continue };
            if m.is_dir() {
                if depth < 1 {
                    todo.push((p, depth + 1));
                }
            } else if m.is_file() && is_note(&p) {
                out.push((p, m.modified().unwrap_or(SystemTime::UNIX_EPOCH), m.len()));
            }
        }
    }
    out.sort();
    out
}

/// A note's text without the paragraphs that look like secrets.
fn safe_text(text: &str) -> String {
    text.split("\n\n")
        .filter(|para| !memory::looks_secret(para))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// The topics of the notes in `dir`.
pub fn topics(dir: &Path) -> Vec<Topic> {
    let mut out = Vec::new();
    for (p, _, _) in files(dir) {
        let Ok(bytes) = std::fs::read(&p) else {
            continue;
        };
        let cut = &bytes[..bytes.len().min(MAX_BYTES)];
        let text = String::from_utf8_lossy(cut);
        let rel = p
            .strip_prefix(dir)
            .unwrap_or(&p)
            .to_string_lossy()
            .to_string();
        out.extend(knowledge::parse_note(&rel, &safe_text(&text)));
    }
    out
}

type Signature = (Option<SystemTime>, Vec<(PathBuf, SystemTime, u64)>);

struct Cache {
    wrote: u64,
    sig: Signature,
    pack: Arc<Pack>,
    checked: Instant,
}

/// The pack to search: the built-in one (or `$EWE_COMPANION_KNOWLEDGE`'s)
/// with the user's notes in the same index. Rebuilt only when a note
/// changed.
pub fn pack() -> Arc<Pack> {
    static C: OnceLock<Mutex<Option<Cache>>> = OnceLock::new();
    let base = crate::chat::pack();
    let Some(dir) = dir() else {
        return Arc::new(base.clone());
    };
    let mut g = C
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let wrote = WROTE.load(std::sync::atomic::Ordering::Relaxed);
    if let Some(c) = g.as_ref()
        && c.checked.elapsed() < RECHECK
        && c.wrote == wrote
    {
        return c.pack.clone();
    }
    let modified = std::fs::metadata(&dir).and_then(|m| m.modified()).ok();
    let sig: Signature = (
        modified,
        if modified.is_some() {
            files(&dir)
        } else {
            vec![]
        },
    );
    if let Some(c) = g.as_mut()
        && c.sig == sig
    {
        c.checked = Instant::now();
        c.wrote = wrote;
        return c.pack.clone();
    }
    let notes = if sig.0.is_some() {
        topics(&dir)
    } else {
        vec![]
    };
    if !notes.is_empty() {
        info!(notes = notes.len(), "notes indexed");
    }
    let pack = Arc::new(if notes.is_empty() {
        base.clone()
    } else {
        base.with(notes)
    });
    *g = Some(Cache {
        wrote,
        sig,
        pack: pack.clone(),
        checked: Instant::now(),
    });
    pack
}

/// A file name from a note's first words: "the printer is on the second
/// floor" → `the printer is on the second floor.md` (letters, digits and
/// spaces only, 60 characters at most).
fn file_name(text: &str) -> String {
    let first = text.lines().next().unwrap_or("").trim();
    let first = first
        .split(['.', '!', '?', ':', ',', ';'])
        .next()
        .unwrap_or(first);
    let mut name: String = first
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '\'' {
                c
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if name.chars().count() > 60 {
        name = name.chars().take(60).collect::<String>();
        if let Some(i) = name.rfind(' ') {
            name.truncate(i);
        }
    }
    if name.is_empty() {
        name = "note".into();
    }
    name
}

/// `learn {text}`: a new note in the notes folder; Undo deletes it.
pub fn learn(a: &Value) -> Outcome {
    let text = a["text"].as_str().unwrap_or("").trim().to_string();
    if text.is_empty() {
        return failed("Learn what? Say \"learn this: …\" with what to keep.");
    }
    if memory::looks_secret(&text) {
        info!("note refused (looked like a secret)");
        return failed(
            "That looks like a password, a PIN, a key or a card number: those belong in ewe Safe, not in notes.",
        )
        .private("I refused to write something that looked like a secret.");
    }
    let Some(dir) = dir() else {
        return failed("I can't find your home folder.");
    };
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return failed(format!("I could not make {}: {e}", places::show(&dir)));
    }
    let base = file_name(&text);
    let mut path = dir.join(format!("{base}.md"));
    let mut n = 2;
    while path.exists() {
        path = dir.join(format!("{base} ({n}).md"));
        n += 1;
    }
    let title = {
        let mut c = base.chars();
        match c.next() {
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            None => base.clone(),
        }
    };
    let body = format!("# {title}\n\n{text}\n");
    if let Err(e) = std::fs::write(&path, &body) {
        return failed(format!("I could not write the note: {e}"));
    }
    info!("note written");
    changed();
    Outcome {
        undo_with: Some(Undo::File(path.clone(), body)),
        rows: vec![row(&path)],
        ..done(
            format!("Learned. It is a note in {} now.", places::show(&dir)),
            None,
        )
    }
    .private("Learned it: a new note in the notes folder.")
}

fn row(p: &Path) -> Value {
    let path = p.to_string_lossy().to_string();
    let name = p
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.clone());
    json!({"title": name, "detail": places::show(p.parent().unwrap_or(p)), "icon": "note",
           "actions": [{"label": "Open", "command": "open_file", "args": {"path": path}},
                       {"label": "Show in folder", "command": "show_file", "args": {"path": path}}]})
}

/// `notes {query?}`: the notes, or the ones these words fit, with Open.
pub fn list(a: &Value) -> Outcome {
    let Some(dir) = dir() else {
        return failed("I can't find your home folder.");
    };
    let all = files(&dir);
    if all.is_empty() {
        return done(
            format!(
                "You have no notes yet. Say \"learn this: …\", or put Markdown or text files in {}.",
                places::show(&dir)
            ),
            None,
        );
    }
    let q = a
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_lowercase();
    let shown: Vec<&PathBuf> = if q.is_empty() {
        all.iter().map(|(p, _, _)| p).collect()
    } else {
        let pack = Pack::new(String::new(), topics(&dir));
        let mut hits: Vec<PathBuf> = Vec::new();
        for h in pack.search(&q, 10) {
            if let Some(rel) = h.topic.note() {
                let p = dir.join(rel);
                if !hits.contains(&p) {
                    hits.push(p);
                }
            }
        }
        let keep: Vec<&PathBuf> = all
            .iter()
            .map(|(p, _, _)| p)
            .filter(|p| hits.contains(p))
            .collect();
        if keep.is_empty() {
            return done(format!("None of your notes is about {q}."), None);
        }
        keep
    };
    let n = shown.len();
    Outcome {
        rows: shown.iter().take(20).map(|p| row(p)).collect(),
        ..done(
            if n == 1 {
                "One note:".to_string()
            } else {
                format!("{n} notes:")
            },
            None,
        )
    }
    .private(format!("{n} notes; they show on the island."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_names_come_from_the_first_words() {
        assert_eq!(
            file_name("The printer is on the 2nd floor."),
            "The printer is on the 2nd floor"
        );
        assert_eq!(file_name("wifi: Home5G"), "wifi");
        assert_eq!(file_name("../../etc/hosts"), "note");
        assert_eq!(file_name("a/b c"), "a b c");
        assert_eq!(file_name("!!!"), "note");
        assert!(file_name(&"word ".repeat(40)).chars().count() <= 60);
    }

    #[test]
    fn secret_paragraphs_stay_out() {
        let t = safe_text("The router is in the hall.\n\nThe wifi password is hunter22");
        assert_eq!(t, "The router is in the hall.");
    }

    #[test]
    fn notes_in_a_folder_are_topics() {
        let d = std::env::temp_dir().join(format!("shaun-notes-{}", std::process::id()));
        std::fs::create_dir_all(d.join("home")).unwrap();
        std::fs::write(
            d.join("printer.md"),
            "# Printer\n\nSecond floor, by the kitchen.\n",
        )
        .unwrap();
        std::fs::write(d.join("home/bins.txt"), "Bins go out on Tuesday night.").unwrap();
        std::fs::write(d.join(".hidden.md"), "# Hidden\n\nnever").unwrap();
        std::fs::write(d.join("photo.jpg"), "not a note").unwrap();
        let t = topics(&d);
        let mut names: Vec<&str> = t.iter().filter_map(|t| t.note()).collect();
        names.sort();
        assert_eq!(names, ["home/bins.txt", "printer.md"]);
        std::fs::remove_dir_all(&d).ok();
    }
}
