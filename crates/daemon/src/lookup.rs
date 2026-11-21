//! Knowledge beyond the pack, on this computer: the ArchWiki
//! (`arch-wiki-lite`), manual pages and `--help`, and ewe's own release
//! notes ("what's new").
//!
//! Each is a *look* (`persona::LOOKS`): a model may ask for it, gets what
//! it found back wrapped as untrusted (`chat::look_up`), and answers from
//! it. With no model, the excerpt is the answer, with a button to read the
//! whole thing. What these return is never the user's own: a wiki page, a
//! manual, ewe's notes. Bounded in size, read with a time limit, argv only.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use companion_core::commands::Call;
use companion_core::knowledge::{self, Topic};
use companion_core::wiki;
use serde_json::{Value, json};
use tokio::process::Command;
use tracing::info;

use crate::commands::{Outcome, done, failed, missed};
use crate::context;
use crate::hub::Hub;

/// How much of a page the island shows, and a model gets.
const SHOWN: usize = 700;
const FOR_MODEL: usize = 1800;

/// Where arch-wiki-docs keeps the pages as HTML (optional, big): opened
/// when there, else the page on wiki.archlinux.org.
const WIKI_HTML: &str = "/usr/share/doc/arch-wiki/html/en";

/// Run a reading tool: argv only, no stdin, a time limit, its output (and
/// its errors, which some tools print their help on) as text.
async fn read_tool(
    bin: &Path,
    args: &[&str],
    env: &[(&str, &str)],
    budget: Duration,
    both: bool,
) -> Option<String> {
    let mut c = Command::new(bin);
    c.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(if both { Stdio::piped() } else { Stdio::null() })
        .current_dir(std::env::temp_dir())
        .kill_on_drop(true);
    for (k, v) in env {
        c.env(k, v);
    }
    let out = tokio::time::timeout(budget, c.output()).await.ok()?.ok()?;
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    if both && text.trim().is_empty() {
        text = String::from_utf8_lossy(&out.stderr).to_string();
    }
    // never more than a page or two of anything
    if text.len() > 512 * 1024 {
        let mut cut = 512 * 1024;
        while !text.is_char_boundary(cut) {
            cut -= 1;
        }
        text.truncate(cut);
    }
    Some(text)
}

// ── the ArchWiki ────────────────────────────────────────────────────────────

/// `wiki {query}`: the best page for these words, and the part of it that
/// fits them.
pub async fn wiki(query: &str) -> Outcome {
    wiki_page(query, false).await.0
}

/// With no model, a question the pack does not answer: the wiki's page,
/// when its title says every word of the question that matters
/// ("how do I set up pipewire" → PipeWire).
pub async fn wiki_confident(query: &str) -> Option<Outcome> {
    context::tool("wiki-search")?;
    let (out, found) = wiki_page(query, true).await;
    found.then_some(out)
}

/// The page and its part, and whether one was found (with `sure`: whether
/// its title says the question).
async fn wiki_page(query: &str, sure: bool) -> (Outcome, bool) {
    let q = query.trim();
    let Some(bin) = context::tool("wiki-search") else {
        return (
            Outcome {
                action: Some((
                    "Find arch-wiki-lite in Komble".into(),
                    Call::new(
                        "open",
                        json!({"what": "komble", "page": "search=arch-wiki-lite"}),
                    ),
                )),
                ..failed(
                    "The ArchWiki is not on this computer: it comes with the arch-wiki-lite package.",
                )
            },
            false,
        );
    };
    // wiki_width set: no `dialog` menu, plain rows; PAGER cat: no less
    let env = [("wiki_width", "200"), ("PAGER", "cat"), ("TERM", "dumb")];
    // the words that matter: the tool matches any of them, line by line
    let mut words = wiki::query_words(q);
    if words.is_empty() && !sure {
        words = q.split_whitespace().map(str::to_lowercase).collect();
    }
    words.truncate(8);
    let words: Vec<&str> = words.iter().map(String::as_str).collect();
    if words.is_empty() {
        return (missed("Say what to look up in the wiki."), false);
    }
    let Some(list) = read_tool(&bin, &words, &env, Duration::from_secs(10), false).await else {
        return (failed("The ArchWiki search did not answer in time."), false);
    };
    let hits = wiki::rank(q, wiki::parse_hits(&list));
    let Some(best) = hits.first().cloned() else {
        return (
            missed(format!("The ArchWiki on this computer has nothing on {q}.")),
            false,
        );
    };
    if sure && !wiki::title_says_all(q, &best.title) {
        return (missed("Not sure enough."), false);
    }
    let Some(text) = read_tool(&bin, &[&best.id], &env, Duration::from_secs(10), false).await
    else {
        return (failed("The ArchWiki page did not open in time."), false);
    };
    let page = wiki::parse_page(&text);
    let (section, part) = wiki::excerpt(&page, &best.title, q, FOR_MODEL);
    if part.is_empty() {
        return (
            missed(format!("The ArchWiki page {} is empty here.", best.title)),
            false,
        );
    }
    info!(chars = part.len(), "wiki");
    let place = match &section {
        Some(s) => format!("{} → {s}", best.title),
        None => best.title.clone(),
    };
    let shown = wiki::bounded(&part, SHOWN);
    let local = PathBuf::from(WIKI_HTML).join(format!("{}.html", best.title.replace(' ', "_")));
    let action = if local.is_file() {
        Some((
            "Open the page".to_string(),
            Call::new("open_file", json!({"path": local.to_string_lossy()})),
        ))
    } else {
        page.url.clone().map(|u| {
            let u = match &section {
                Some(s) => format!("{u}#{}", s.replace(' ', "_")),
                None => u,
            };
            (
                "Open the page".to_string(),
                Call::new("open_url", json!({"url": u})),
            )
        })
    };
    let others: Vec<&str> = hits
        .iter()
        .skip(1)
        .take(3)
        .map(|h| h.title.as_str())
        .collect();
    let mut facts = format!("ArchWiki, page \"{place}\":\n{part}");
    if !others.is_empty() {
        facts.push_str(&format!("\n(Other pages: {})", others.join(", ")));
    }
    (
        Outcome {
            action,
        rows: hits
            .iter()
            .skip(1)
            .take(3)
            .map(|h| {
                json!({"title": h.title, "detail": "ArchWiki", "icon": "book",
                       "actions": [{"label": "Read", "command": "wiki", "args": {"query": h.title}}]})
            })
            .collect(),
        ..done(format!("From the ArchWiki, {place}:\n{shown}"), None)
    }
    .facts(facts),
        true,
    )
}

// ── manual pages and --help ─────────────────────────────────────────────────

/// Programs whose `--help` is never run (some ignore it and act, or end
/// the session).
const NO_HELP: &[&str] = &[
    "shutdown",
    "reboot",
    "poweroff",
    "halt",
    "init",
    "telinit",
    "rm",
    "dd",
    "shred",
    "wipefs",
    "kill",
    "killall",
    "pkill",
    "mkfs",
    "fdisk",
    "sfdisk",
    "parted",
    "mkswap",
    "swapoff",
    "su",
    "sudo",
    "doas",
    "pkexec",
    "login",
    "passwd",
    "chsh",
    "chfn",
    "yes",
    "cat",
    "tee",
    "sh",
    "bash",
    "zsh",
    "fish",
    "python",
    "python3",
    "node",
    "perl",
    "ruby",
    "lua",
    "hyprctl",
    "loginctl",
    "systemctl",
    "ewe-companiond",
];

/// Whether a program says it knows `--help`: its file holds those bytes.
fn mentions_help(bin: &Path) -> bool {
    use std::io::Read;
    let Ok(f) = std::fs::File::open(bin) else {
        return false;
    };
    let mut buf = Vec::new();
    if f.take(48 * 1024 * 1024).read_to_end(&mut buf).is_err() {
        return false;
    }
    buf.windows(6).any(|w| w == b"--help")
}

/// `man_page {name, query?}`: what the manual (or `--help`) says that fits.
pub async fn man_page(name: &str, query: &str) -> Outcome {
    let name = name.trim();
    let mut text = None;
    let mut source = "manual";
    if let Some(man) = context::tool("man") {
        let env = [
            ("MANPAGER", "cat"),
            ("PAGER", "cat"),
            ("MANWIDTH", "90"),
            ("GROFF_NO_SGR", "1"),
            ("LC_ALL", "C.UTF-8"),
        ];
        text = read_tool(&man, &[name], &env, Duration::from_secs(6), false)
            .await
            .filter(|t| t.lines().count() > 3);
    }
    if text.is_none() {
        let base = name.rsplit('/').next().unwrap_or(name);
        if let Some(bin) = context::tool(name).filter(|_| !NO_HELP.contains(&base))
            && mentions_help(&bin)
        {
            text = read_tool(
                &bin,
                &["--help"],
                &[("LC_ALL", "C.UTF-8")],
                Duration::from_secs(3),
                true,
            )
            .await
            .filter(|t| t.lines().count() > 1);
            source = "--help";
        }
    }
    let Some(text) = text else {
        return missed(format!("There is no manual for {name} on this computer."));
    };
    let part = if source == "manual" {
        wiki::man_excerpt(&wiki::man_sections(&text), query, FOR_MODEL)
    } else {
        help_excerpt(&text, query, FOR_MODEL)
    };
    info!(source, chars = part.len(), "manual");
    let shown = wiki::bounded(&part, SHOWN);
    let all = if source == "manual" {
        format!("man {name}")
    } else {
        format!("{name} --help | less")
    };
    Outcome {
        propose: Some((all, format!("Read all of {name}'s {source}"))),
        ..done(format!("{name}, from its {source}:\n{shown}"), None)
    }
    .facts(format!("{name}'s {source}:\n{part}"))
}

/// `--help` text: its first lines, then the lines that say the question's
/// words.
fn help_excerpt(text: &str, query: &str, max: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut out: Vec<&str> = lines.iter().take(4).copied().collect();
    let q: Vec<String> = query
        .to_lowercase()
        .split_whitespace()
        .filter(|w| w.len() > 2)
        .map(str::to_string)
        .collect();
    if !q.is_empty() {
        for l in lines.iter().skip(4) {
            let low = l.to_lowercase();
            if q.iter().any(|w| low.contains(w.as_str())) {
                out.push(l);
            }
        }
    }
    wiki::bounded(&out.join("\n"), max)
}

// ── what's new in ewe ───────────────────────────────────────────────────────

/// The ewe installed here: `$EWE_COMPANION_EWE_VERSION` (tests), else
/// `/usr/share/ewe/VERSION`.
pub fn ewe_version() -> Option<String> {
    if let Ok(v) = std::env::var("EWE_COMPANION_EWE_VERSION") {
        return Some(v.trim().to_string()).filter(|v| !v.is_empty());
    }
    let v = std::fs::read_to_string("/usr/share/ewe/VERSION").ok()?;
    Some(v.trim().to_string()).filter(|v| !v.is_empty())
}

fn short(t: &Topic) -> &str {
    t.body.trim()
}

/// `whats_new {version?}`: the release notes of the ewe here (or the one
/// asked about), in plain words, with the releases before it as rows.
pub fn whats_new(_hub: &Arc<Hub>, a: &Value) -> Outcome {
    let pack = crate::chat::pack();
    let news = pack.news();
    let Some(newest) = news.first() else {
        return failed("I have no release notes in this version of me.");
    };
    let asked = a.get("version").and_then(Value::as_str).map(str::to_string);
    let here = ewe_version();
    let want = asked
        .as_deref()
        .or(here.as_deref())
        .and_then(knowledge::minor_of);
    let (topic, note) = match want {
        Some(v) => match news.iter().find(|(n, _)| *n == v) {
            Some((_, t)) => (*t, None),
            None if v > newest.0 => (
                newest.1,
                Some(format!(
                    "I have no notes for ewe {}.{} yet (my notes end at {}.{}); here is the newest I know.",
                    v.0, v.1, newest.0.0, newest.0.1
                )),
            ),
            None => {
                return failed(format!(
                    "I have no notes for ewe {}.{}: mine go back to {}.{}.",
                    v.0,
                    v.1,
                    news.last().map_or(0, |n| n.0.0),
                    news.last().map_or(0, |n| n.0.1)
                ));
            }
        },
        None => (newest.1, None),
    };
    let mut line = String::new();
    if let (None, Some(v)) = (&asked, &here) {
        line.push_str(&format!("You have ewe {v}. "));
    }
    if let Some(n) = note {
        line.push_str(&n);
        line.push(' ');
    }
    line.push_str(&format!("{}:\n{}", topic.title, short(topic)));
    let rows = news
        .iter()
        .filter(|(_, t)| !std::ptr::eq(*t, topic))
        .filter(|(v, _)| want.is_none_or(|w| *v < w))
        .take(3)
        .map(|(v, t)| {
            json!({"title": t.title, "detail": "Earlier", "icon": "news",
                   "actions": [{"label": "Read", "command": "whats_new",
                                "args": {"version": format!("{}.{}", v.0, v.1)}}]})
        })
        .collect();
    Outcome {
        rows,
        ..done(line, None)
    }
    .line_is_facts()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_text_keeps_its_head_and_the_lines_that_fit() {
        let t = "Usage: tool [OPTION]\nA tool.\n\nOptions:\n  -v  verbose\n  -x, --exclude=PAT  skip files\n  -q  quiet\n";
        let e = help_excerpt(t, "exclude files", 500);
        assert!(e.starts_with("Usage: tool"));
        assert!(e.contains("--exclude"));
        assert!(!e.contains("quiet"));
    }

    #[test]
    fn a_program_that_mentions_help() {
        let d = std::env::temp_dir().join(format!("shaun-help-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("a"), b"\x7fELF ... --help ...").unwrap();
        std::fs::write(d.join("b"), b"\x7fELF ... nothing").unwrap();
        assert!(mentions_help(&d.join("a")));
        assert!(!mentions_help(&d.join("b")));
        std::fs::remove_dir_all(&d).ok();
    }
}
