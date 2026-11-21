//! What the companion knows about ewe (the owner, 2026-10-14: "give this
//! companion all power to know everything about ewe").
//!
//! The knowledge pack is the Markdown in `knowledge/` at the repo root,
//! built into the binary so every install has it:
//!
//! - `primer.md` goes into every question a model gets;
//! - every other file is a list of topics, found by plain word matching
//!   ([`Pack::search`]), so the few that fit a question go along with it.
//!   With no model at all, a confident match is the answer itself.
//!
//! A topic:
//!
//! ```text
//! ## Change the wallpaper
//! keys: wallpaper, background, video wallpaper
//! open: settings wallpaper
//! Settings → Wallpaper: pick a picture from the grid or press "Choose file…".
//! ```
//!
//! `keys` are the words people use for it. `open` (optional) is an `open`
//! command, `<what> [page]`, offered as a button under the answer. `run`
//! (optional) is a shell command that does it, offered with **Run** (and
//! only run on that click).
//! `<!-- … -->` comments are for whoever keeps the pack, never for a model.

use serde_json::json;

use crate::commands::{Call, page_label, validate};

/// The pack as built: (file stem, text).
const BUILT_IN: &[(&str, &str)] = &[
    ("primer", include_str!("../../../knowledge/primer.md")),
    ("apps", include_str!("../../../knowledge/apps.md")),
    ("howto", include_str!("../../../knowledge/howto.md")),
    ("keys", include_str!("../../../knowledge/keys.md")),
    ("fix", include_str!("../../../knowledge/fix.md")),
    ("safe", include_str!("../../../knowledge/safe.md")),
    ("browser", include_str!("../../../knowledge/browser.md")),
    ("cli", include_str!("../../../knowledge/cli.md")),
    ("media", include_str!("../../../knowledge/media.md")),
    ("languages", include_str!("../../../knowledge/languages.md")),
    ("news", include_str!("../../../knowledge/news.md")),
];

/// A match this strong is an answer on its own: one of the topic's own
/// key phrases, said in the question.
pub const CONFIDENT: f32 = 3.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Topic {
    pub file: String,
    pub title: String,
    pub keys: Vec<String>,
    pub open: Option<Call>,
    /// a shell command that does it, offered with a Run button (it never
    /// runs without the click; `shell::assess` checks it like any other)
    pub run: Option<String>,
    pub body: String,
}

impl Topic {
    /// The first paragraph.
    pub fn short(&self) -> &str {
        self.body.split("\n\n").next().unwrap_or("").trim()
    }

    /// The paragraph that answers this question, for the chat when no
    /// model answers: the one naming a combo the question names, else the
    /// one sharing most of its words, else the first.
    pub fn answer(&self, question: &str) -> &str {
        let asked = combos(question);
        let q = content(words(question));
        let mut best = (0.0f32, self.short());
        for (i, para) in self
            .body
            .split("\n\n")
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .enumerate()
        {
            // the first paragraph is the topic's summary: it wins a tie
            let mut score = if i == 0 { 1.0 } else { 0.0 };
            if asked.iter().any(|c| names_combo(para, c)) {
                score += 10.0;
            }
            let pw = words(para);
            score += q.iter().filter(|w| pw.contains(w)).count() as f32;
            if score > best.0 {
                best = (score, para);
            }
        }
        best.1
    }

    /// The button under an answer, e.g. "Open Updates".
    pub fn open_label(&self) -> Option<String> {
        self.open.as_ref().map(open_label)
    }
}

/// What a button that runs this `open` call says.
pub fn open_label(call: &Call) -> String {
    let what = call.args["what"].as_str().unwrap_or("");
    let page = call.args["page"].as_str().unwrap_or("");
    match (what, page) {
        ("settings", _) => "Open ewe Settings".into(),
        ("komble", "") => "Open Komble".into(),
        ("komble", p) if p.starts_with("options=") => "Open its options".into(),
        ("komble", p) => format!("Open {}", page_label("komble", p).unwrap_or("Komble")),
        ("quicksettings", "" | "home") => "Open Quick settings".into(),
        ("quicksettings", p) => format!(
            "Open {}",
            page_label("quicksettings", p).unwrap_or("Quick settings")
        ),
        ("overview", _) => "Open the Overview".into(),
        ("sync", _) => "Open ewe Sync".into(),
        ("terminal", _) => "Open a terminal".into(),
        _ => "Open Files".into(),
    }
}

#[derive(Debug, Clone)]
pub struct Pack {
    pub primer: String,
    pub topics: Vec<Topic>,
    /// the search index over `topics` (built with them; see [`Pack::search`])
    index: Index,
}

#[derive(Debug)]
pub struct Hit<'a> {
    pub score: f32,
    pub topic: &'a Topic,
}

fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("<!--") {
        out.push_str(&rest[..at]);
        rest = match rest[at..].find("-->") {
            Some(end) => &rest[at + end + 3..],
            None => "",
        };
    }
    out.push_str(rest);
    out
}

/// One file's topics. A `# ` title line is the file's own name and is
/// dropped; text before the first `## ` is too.
pub fn parse(file: &str, text: &str) -> Vec<Topic> {
    let text = strip_comments(text);
    let mut topics = Vec::new();
    for chunk in text.split("\n## ").skip(1) {
        let mut lines = chunk.lines();
        let title = lines.next().unwrap_or("").trim().to_string();
        let mut keys = Vec::new();
        let mut open = None;
        let mut run = None;
        let mut body = Vec::new();
        for line in lines {
            if body.is_empty() {
                if let Some(k) = line.strip_prefix("keys:") {
                    keys = k
                        .split(',')
                        .map(|s| s.trim().to_lowercase())
                        .filter(|s| !s.is_empty())
                        .collect();
                    continue;
                }
                if let Some(r) = line.strip_prefix("run:") {
                    let r = r.trim();
                    run = (!r.is_empty()).then(|| r.to_string());
                    continue;
                }
                if let Some(o) = line.strip_prefix("open:") {
                    let mut w = o.split_whitespace();
                    let what = w.next().unwrap_or("");
                    let page = w.next().unwrap_or("");
                    let call = Call::new("open", json!({"what": what, "page": page}));
                    open = validate(&call).is_ok().then_some(call);
                    continue;
                }
            }
            body.push(line);
        }
        let body = body.join("\n").trim().to_string();
        if !title.is_empty() && !body.is_empty() {
            topics.push(Topic {
                file: file.to_string(),
                title,
                keys,
                open,
                run,
                body,
            });
        }
    }
    topics
}

const STOP: &[&str] = &[
    "a", "an", "the", "to", "of", "in", "on", "at", "by", "for", "with", "from", "and", "or", "my",
    "me", "i", "im", "is", "are", "am", "be", "it", "its", "this", "that", "these", "there", "do",
    "doe", "did", "can", "could", "would", "should", "will", "how", "what", "whats", "where",
    "why", "when", "which", "who", "you", "your", "please", "want", "need", "get", "some", "any",
    "way", "just", "now", "here", "so", "not", "no", "dont", "cant", "about", "tell", "know",
    "show", "help", "change", "use", "using", "make", "set", "find", "ewe", "computer", "pc",
    "laptop", "wont", "doesnt", "isnt", "didnt", "arent", "wasnt", "werent", "hasnt", "havent",
];

fn stem(w: &str) -> String {
    let n = w.chars().count();
    if n > 5 && w.ends_with("ing") {
        return w[..w.len() - 3].to_string();
    }
    if n > 4 && w.ends_with("ies") {
        return format!("{}y", &w[..w.len() - 3]);
    }
    // stashes, switches, boxes, passes → stash, switch, box, pass
    if n > 4
        && ["shes", "ches", "xes", "sses", "zes"]
            .iter()
            .any(|e| w.ends_with(e))
    {
        return w[..w.len() - 2].to_string();
    }
    if n > 3 && w.ends_with('s') && !w.ends_with("ss") {
        return w[..w.len() - 1].to_string();
    }
    w.to_string()
}

/// Lower-case word stems, apostrophes dropped ("don't" → "dont").
fn words(s: &str) -> Vec<String> {
    s.to_lowercase()
        .replace(['\'', '’'], "")
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(stem)
        .collect()
}

fn content(ws: Vec<String>) -> Vec<String> {
    ws.into_iter()
        .filter(|w| !STOP.contains(&w.as_str()))
        .collect()
}

/// Key combos in the text, as `super+g`: lower case, no spaces round `+`.
fn combos(s: &str) -> Vec<String> {
    let mut t = s.to_lowercase();
    while t.contains(" +") || t.contains("+ ") {
        t = t.replace(" +", "+").replace("+ ", "+");
    }
    t.split(|c: char| c.is_whitespace() || ",;:()\"'?!".contains(c))
        .map(|w| w.trim_end_matches('.'))
        .filter(|w| {
            w.contains('+')
                && ["super+", "ctrl+", "alt+", "shift+"]
                    .iter()
                    .any(|m| w.starts_with(m))
        })
        .map(str::to_string)
        .collect()
}

/// Does the text name this combo exactly ("super+g", not "super+gr…")?
fn names_combo(text: &str, combo: &str) -> bool {
    let t = text.to_lowercase();
    t.match_indices(combo).any(|(at, _)| {
        let before = t[..at].chars().next_back();
        let after = t[at + combo.len()..].chars().next();
        !before.is_some_and(|c| c.is_alphanumeric() || c == '+')
            && !after.is_some_and(|c| c.is_alphanumeric() || c == '+')
    })
}

#[cfg(test)]
fn contains_run(hay: &[String], needle: &[String]) -> bool {
    !needle.is_empty() && hay.windows(needle.len()).any(|w| w == needle)
}

/// For each word of [`words`], whether a hyphen joins it to the next one
/// ("ewe-bt": `[true, false]`), so a phrase never ends or starts in the
/// middle of such a name.
fn hyphen_joined(s: &str) -> Vec<bool> {
    let t = s.to_lowercase().replace(['\'', '’'], "");
    let chars: Vec<char> = t.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_alphanumeric() {
            while i < chars.len() && chars[i].is_alphanumeric() {
                i += 1;
            }
            let joined = i + 1 < chars.len() && chars[i] == '-' && chars[i + 1].is_alphanumeric();
            out.push(joined);
        } else {
            i += 1;
        }
    }
    out
}

/// Where a key phrase is said in the question as whole words, as (start,
/// end) word positions: never the start of a hyphenated name ("what is
/// ewe" is not said in "what is ewe-bt").
fn phrase_at(hay: &[String], joined: &[bool], needle: &[String]) -> Option<(usize, usize)> {
    if needle.is_empty() || needle.len() > hay.len() {
        return None;
    }
    (0..=hay.len() - needle.len())
        .find(|&i| {
            hay[i..i + needle.len()] == *needle
                && !joined.get(i + needle.len() - 1).copied().unwrap_or(false)
                && !(i > 0 && joined.get(i - 1).copied().unwrap_or(false))
        })
        .map(|i| (i, i + needle.len()))
}

#[cfg(test)]
fn says_phrase(hay: &[String], joined: &[bool], needle: &[String]) -> bool {
    phrase_at(hay, joined, needle).is_some()
}

impl Pack {
    pub fn from_files<'a>(files: impl IntoIterator<Item = (&'a str, &'a str)>) -> Pack {
        let mut primer = String::new();
        let mut topics = Vec::new();
        for (name, text) in files {
            if name == "primer" {
                primer = strip_comments(text)
                    .lines()
                    .filter(|l| !l.starts_with("# "))
                    .collect::<Vec<_>>()
                    .join("\n")
                    .trim()
                    .to_string();
            } else {
                topics.extend(parse(name, text));
            }
        }
        Pack::new(primer, topics)
    }

    /// A pack of these topics, indexed.
    pub fn new(primer: String, topics: Vec<Topic>) -> Pack {
        let index = Index::build(&topics);
        Pack {
            primer,
            topics,
            index,
        }
    }

    /// This pack with more topics (the user's own notes): one index over
    /// all of them, so a note and a topic compete on the same terms.
    pub fn with(&self, more: Vec<Topic>) -> Pack {
        let mut topics = self.topics.clone();
        topics.extend(more);
        Pack::new(self.primer.clone(), topics)
    }

    /// The pack built into this binary.
    pub fn built_in() -> Pack {
        Pack::from_files(BUILT_IN.iter().copied())
    }

    /// Score every topic against a question; the best `n` that match at all.
    ///
    /// Two parts. A key phrase said whole in the question is a *confident*
    /// match on its own (at least [`CONFIDENT`], more for longer phrases):
    /// that is what lets the pack answer with no model. Then BM25 over the
    /// topic's keys, title and text (keys count most, then the title, a
    /// long text counts less per word), so the topics rank by how much of
    /// the question they share, rare words weighing more than common ones.
    /// A question word the pack does not know is tried as a synonym
    /// ([`SYNONYMS`]: "film" is a video) and then as a typo of a word it
    /// does know ("bluetoth"), each for a little less.
    pub fn search(&self, question: &str, n: usize) -> Vec<Hit<'_>> {
        let all = words(question);
        let joined = hyphen_joined(question);
        // the question with its typos fixed, for the phrase match too
        let fixed: Vec<String> = all
            .iter()
            .map(|w| {
                if STOP.contains(&w.as_str()) || self.index.knows(w) {
                    w.clone()
                } else {
                    self.index.typo_of(w).unwrap_or_else(|| w.clone())
                }
            })
            .collect();
        let mut slots: Vec<Vec<(String, f32)>> = Vec::new();
        let mut seen: Vec<&str> = Vec::new();
        for (w, f) in all.iter().zip(&fixed) {
            if STOP.contains(&w.as_str()) || seen.contains(&w.as_str()) {
                continue;
            }
            seen.push(w);
            slots.push(self.index.alternatives(w, f));
        }
        let asked_combos = combos(question);
        // the key phrases said in the question, per topic, with where. A
        // phrase is matched on the whole question, so "do not disturb"
        // still counts though "do" and "not" are stop words
        let said: Vec<Vec<(usize, usize, usize)>> = self
            .index
            .docs
            .iter()
            .map(|doc| {
                doc.keys
                    .iter()
                    .filter_map(|kw| {
                        let (a, b) = phrase_at(&all, &joined, kw)
                            .or_else(|| phrase_at(&fixed, &joined, kw))?;
                        Some((a, b, kw.len()))
                    })
                    .collect()
            })
            .collect();
        // a phrase whose last words start another topic's longer name says
        // less: "what is ewe" in "what is ewe sync" is about ewe Sync
        let cut_short = |i: usize, (a, b): (usize, usize)| {
            said.iter()
                .enumerate()
                .any(|(j, m)| j != i && m.iter().any(|&(c, d, _)| c > a && c < b && d > b))
        };
        let mut hits: Vec<Hit> = self
            .topics
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                let mut score = 0.0f32;
                for &(a, b, len) in &said[i] {
                    if !cut_short(i, (a, b)) {
                        score += CONFIDENT + 2.0 * (len as f32 - 1.0);
                    }
                }
                score += BM25_SCALE * self.index.bm25(i, &slots);
                // the question names the topic's title, every word of it
                let title = content(words(&t.title));
                if !title.is_empty() && title.iter().all(|w| fixed.contains(w)) {
                    score += TITLE_BONUS;
                }
                // "what does Super+G do": the topic that names that combo
                if asked_combos.iter().any(|c| names_combo(&t.body, c)) {
                    score += 4.0;
                }
                (score > 0.0).then_some(Hit { score, topic: t })
            })
            .collect();
        hits.sort_by(|a, b| b.score.total_cmp(&a.score));
        hits.truncate(n);
        hits
    }
}

/// How much BM25 weighs next to a key phrase ([`CONFIDENT`]): one strong
/// word shared with a topic's keys, title and text scores about 2, so a
/// single word is never confident alone; two strong words are.
const BM25_SCALE: f32 = 1.4;
/// A question that says every word of a topic's title is about it.
const TITLE_BONUS: f32 = 1.0;
/// BM25's term-frequency saturation and length normalisation.
const K1: f32 = 1.2;
/// The fields: keys count most, then the title, then the text; a field's
/// length counts this much (`b`) against it.
const FIELDS: [(f32, f32); 3] = [(3.0, 0.2), (2.0, 0.3), (1.0, 0.75)];
/// How much a synonym or a typo's fix counts against the word itself.
const SYNONYM_WEIGHT: f32 = 0.8;
const TYPO_WEIGHT: f32 = 0.7;

/// Words people use for what the pack calls something else, as stems
/// (after [`words`]): the question's word on the left, the pack's on the
/// right. Only tried when they help: the word itself counts first.
pub const SYNONYMS: &[(&str, &[&str])] = &[
    ("film", &["video", "movie"]),
    ("movie", &["video"]),
    ("clip", &["video"]),
    ("song", &["music", "track"]),
    ("tune", &["music"]),
    ("track", &["music", "song"]),
    ("photo", &["picture", "image"]),
    ("pic", &["picture", "image"]),
    ("image", &["picture", "photo"]),
    ("picture", &["image", "photo"]),
    ("monitor", &["display", "screen"]),
    ("screen", &["display"]),
    ("display", &["screen", "monitor"]),
    ("wireless", &["wifi"]),
    ("internet", &["wifi", "network"]),
    ("uninstall", &["remove"]),
    ("delete", &["remove", "trash"]),
    ("bin", &["trash"]),
    ("speaker", &["sound", "audio"]),
    ("audio", &["sound"]),
    ("sound", &["audio"]),
    ("hotkey", &["shortcut"]),
    ("keybind", &["shortcut"]),
    ("keybinding", &["shortcut"]),
    ("console", &["terminal"]),
    ("store", &["komble"]),
    ("appstore", &["komble"]),
    ("upgrade", &["update"]),
    ("subtitle", &["subtitles", "caption"]),
    ("caption", &["subtitle"]),
    ("sub", &["subtitle"]),
    ("dictionary", &["spell", "spelling"]),
    ("spellcheck", &["spell", "spelling"]),
    ("ime", &["typing", "input"]),
    ("chinese", &["pinyin"]),
    ("japanese", &["mozc"]),
    ("korean", &["hangul"]),
    ("youtube", &["link", "video"]),
    ("heic", &["iphone", "photo"]),
    ("iphone", &["heic"]),
    ("expose", &["overview"]),
    ("tray", &["fold", "chevron"]),
    ("password", &["safe"]),
    ("taskbar", &["dock"]),
];

/// One topic as the index sees it: its key phrases as words, and each
/// field's word counts and length.
#[derive(Debug, Clone, Default)]
struct Doc {
    keys: Vec<Vec<String>>,
    tf: [std::collections::HashMap<String, f32>; 3],
    len: [f32; 3],
}

#[derive(Debug, Clone, Default)]
struct Index {
    docs: Vec<Doc>,
    /// in how many topics each word appears (any field)
    df: std::collections::HashMap<String, f32>,
    avg: [f32; 3],
}

impl Index {
    fn build(topics: &[Topic]) -> Index {
        let mut docs = Vec::with_capacity(topics.len());
        let mut df: std::collections::HashMap<String, f32> = Default::default();
        let mut sum = [0.0f32; 3];
        for t in topics {
            // one phrase counts once: "monitor" and "monitors", or
            // "ewe-browser" and "ewe browser", are the same words
            let mut keys: Vec<Vec<String>> = Vec::new();
            for k in &t.keys {
                let kw = words(k);
                if !kw.is_empty() && !keys.contains(&kw) {
                    keys.push(kw);
                }
            }
            let fields = [
                content(keys.iter().flatten().cloned().collect()),
                content(words(&t.title)),
                content(words(&t.body)),
            ];
            let mut d = Doc {
                keys,
                ..Default::default()
            };
            let mut seen: std::collections::HashSet<&str> = Default::default();
            for (f, ws) in fields.iter().enumerate() {
                d.len[f] = ws.len() as f32;
                sum[f] += ws.len() as f32;
                for w in ws {
                    *d.tf[f].entry(w.clone()).or_default() += 1.0;
                    if seen.insert(w.as_str()) {
                        *df.entry(w.clone()).or_default() += 1.0;
                    }
                }
            }
            docs.push(d);
        }
        let n = topics.len().max(1) as f32;
        Index {
            docs,
            df,
            avg: sum.map(|s| (s / n).max(1.0)),
        }
    }

    fn knows(&self, w: &str) -> bool {
        self.df.contains_key(w)
    }

    /// The one word of the pack this unknown word is a typo of: within one
    /// edit up to six letters, two from seven (as the matcher's typo pass),
    /// when no other word is as close. Words shorter than six letters are
    /// left alone: "rsync" is a program, not a typo of "sync".
    fn typo_of(&self, w: &str) -> Option<String> {
        let len = w.chars().count();
        if len < 6 || !w.chars().all(|c| c.is_ascii_lowercase()) {
            return None;
        }
        let max = if len >= 7 { 2 } else { 1 };
        let mut best: Vec<&str> = Vec::new();
        let mut best_d = max + 1;
        for v in self.df.keys().filter(|v| v.len() >= 3) {
            let d = crate::typo::distance(w, v, max);
            if d < best_d {
                best_d = d;
                best = vec![v.as_str()];
            } else if d == best_d && d <= max {
                best.push(v.as_str());
            }
        }
        match best[..] {
            [one] => Some(one.to_string()),
            _ => None,
        }
    }

    /// What a question word may match, with how much it counts: itself,
    /// its synonyms, or (when the pack does not know it) its typo's fix.
    fn alternatives(&self, w: &str, fixed: &str) -> Vec<(String, f32)> {
        let mut alts = vec![(w.to_string(), 1.0)];
        if let Some((_, syn)) = SYNONYMS.iter().find(|(k, _)| *k == w) {
            alts.extend(syn.iter().map(|s| (stem(s), SYNONYM_WEIGHT)));
        }
        if fixed != w {
            alts.push((fixed.to_string(), TYPO_WEIGHT));
        }
        alts
    }

    fn idf(&self, w: &str) -> f32 {
        let n = self.docs.len() as f32;
        let df = self.df.get(w).copied().unwrap_or(0.0);
        (1.0 + (n - df + 0.5) / (df + 0.5)).ln()
    }

    /// BM25F of one topic: per question word, the best of its
    /// alternatives, each field's count weighted and length-normalised,
    /// then saturated; idf scaled to 0..1 by the rarest possible word.
    fn bm25(&self, i: usize, slots: &[Vec<(String, f32)>]) -> f32 {
        let d = &self.docs[i];
        let top = self.idf("\u{0}").max(f32::EPSILON);
        let mut total = 0.0;
        for alts in slots {
            let mut best = 0.0f32;
            for (w, weight) in alts {
                let mut tf = 0.0;
                for (f, (wf, b)) in FIELDS.iter().enumerate() {
                    let c = d.tf[f].get(w).copied().unwrap_or(0.0);
                    if c > 0.0 {
                        tf += wf * c / (1.0 - b + b * d.len[f] / self.avg[f]);
                    }
                }
                if tf > 0.0 {
                    let s = weight * (self.idf(w) / top) * tf * (K1 + 1.0) / (tf + K1);
                    best = best.max(s);
                }
            }
            total += best;
        }
        total
    }
}

/// Where a note's topics come from: `notes:<path in the notes folder>`.
pub const NOTE_PREFIX: &str = "notes:";

impl Topic {
    /// The note this topic is from (its path in the notes folder), when it
    /// is one of the user's own notes rather than the built-in pack.
    pub fn note(&self) -> Option<&str> {
        self.file.strip_prefix(NOTE_PREFIX)
    }
}

/// One of the user's own notes (`~/Documents/Shaun/printer.md`) as topics.
///
/// A note written like the pack (`## Title` with `keys:`) is its topics;
/// anything else is one topic: its title is the `# ` heading or the first
/// line, its keys the file's name and that title, its text the rest. `rel`
/// is its path in the notes folder.
pub fn parse_note(rel: &str, text: &str) -> Vec<Topic> {
    let file = format!("{NOTE_PREFIX}{rel}");
    let text = strip_comments(text);
    if text.contains("\n## ") && text.contains("\nkeys:") {
        let t = parse(&file, &format!("\n{text}"));
        if !t.is_empty() {
            return t;
        }
    }
    let stem = rel
        .rsplit('/')
        .next()
        .unwrap_or(rel)
        .rsplit_once('.')
        .map_or(rel, |(s, _)| s)
        .replace(['_', '-'], " ")
        .trim()
        .to_string();
    let mut lines = text.lines().skip_while(|l| l.trim().is_empty());
    let first = lines.next().unwrap_or("").trim();
    let (title, rest): (String, String) = match first.strip_prefix("# ") {
        Some(h) => (h.trim().to_string(), lines.collect::<Vec<_>>().join("\n")),
        None => (
            if stem.is_empty() {
                first.chars().take(80).collect()
            } else {
                stem.clone()
            },
            text.clone(),
        ),
    };
    let body = rest.trim().to_string();
    if title.is_empty() || body.is_empty() {
        return vec![];
    }
    let mut keys = Vec::new();
    // the name, the title, and the title's own longer words: a note about
    // the printer answers "where is the printer"
    let mut names = vec![stem.to_lowercase(), title.to_lowercase()];
    names.extend(
        content(words(&title))
            .into_iter()
            .filter(|w| w.chars().count() >= 4 && w.chars().all(char::is_alphabetic)),
    );
    for k in names {
        let k = k.trim().to_string();
        // a key that is a stop word alone would match nearly any question
        let kw = words(&k);
        if !k.is_empty()
            && !keys.contains(&k)
            && !(kw.len() == 1 && STOP.contains(&kw[0].as_str()))
            && !kw.is_empty()
        {
            keys.push(k);
        }
    }
    vec![Topic {
        file,
        title,
        keys,
        open: None,
        run: None,
        body,
    }]
}

/// "What's new in ewe 0.45" → (0, 45): the release a news topic is about.
pub fn news_version(title: &str) -> Option<(u32, u32)> {
    let v = title.strip_prefix("What's new in ewe ")?;
    minor_of(v)
}

/// The release a version belongs to: "0.45.0-beta" → (0, 45).
pub fn minor_of(version: &str) -> Option<(u32, u32)> {
    let mut parts = version.trim().trim_start_matches('v').split(['.', '-']);
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    Some((major, minor))
}

impl Pack {
    /// The news topics (`news.md`), newest release first.
    pub fn news(&self) -> Vec<((u32, u32), &Topic)> {
        let mut n: Vec<_> = self
            .topics
            .iter()
            .filter(|t| t.file == "news")
            .filter_map(|t| Some((news_version(&t.title)?, t)))
            .collect();
        n.sort_by_key(|a| std::cmp::Reverse(a.0));
        n
    }
}

/// The topics that fit a question, as a model reads them.
pub fn prompt_block(hits: &[&Hit]) -> String {
    let mut s = String::new();
    for h in hits {
        s.push_str(&format!("## {}\n{}\n\n", h.topic.title, h.topic.body));
    }
    s.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack() -> Pack {
        Pack::built_in()
    }

    #[test]
    fn the_built_in_pack_parses() {
        let p = pack();
        assert!(p.primer.contains("ewe.conf"), "primer");
        assert!(!p.primer.contains("<!--"), "comments never reach a model");
        assert!(p.topics.len() >= 50, "{} topics", p.topics.len());
        for t in &p.topics {
            assert!(!t.keys.is_empty(), "{}: no keys", t.title);
            assert!(!t.body.contains("<!--"), "{}", t.title);
            assert!(
                !t.body.starts_with("keys:") && !t.body.starts_with("open:"),
                "{}",
                t.title
            );
            // a one-word key that is a stop word would match nearly any question
            for k in &t.keys {
                let kw = words(k);
                assert!(
                    !(kw.len() == 1 && STOP.contains(&kw[0].as_str())),
                    "{}: key {k:?}",
                    t.title
                );
            }
        }
    }

    #[test]
    fn every_run_line_is_one_the_guard_lets_through() {
        for t in pack().topics {
            if let Some(r) = &t.run {
                let a = crate::shell::assess(r);
                assert!(
                    !matches!(a.verdict, crate::shell::Verdict::Refused(_)),
                    "{}: {r}: {a:?}",
                    t.title
                );
            }
        }
        let p = pack();
        let light = p.search("switch to light mode", 1);
        assert_eq!(
            light[0].topic.run.as_deref(),
            Some("ewe-theme scheme apply ewe-light")
        );
    }

    #[test]
    fn every_open_line_is_a_valid_command() {
        // a typo in an `open:` line would silently drop the button
        for (name, text) in BUILT_IN {
            for t in parse(name, text) {
                let raw = strip_comments(text);
                let declared = raw
                    .split("\n## ")
                    .find(|c| c.lines().next().map(str::trim) == Some(t.title.as_str()))
                    .is_some_and(|c| c.lines().any(|l| l.starts_with("open:")));
                assert_eq!(
                    declared,
                    t.open.is_some(),
                    "{}: open line does not validate",
                    t.title
                );
            }
        }
    }

    fn top(q: &str) -> String {
        pack()
            .search(q, 1)
            .first()
            .map(|h| h.topic.title.clone())
            .unwrap_or_default()
    }

    #[test]
    fn questions_find_their_topic() {
        assert_eq!(top("how do I change my wallpaper?"), "Change the wallpaper");
        assert_eq!(top("How do I connect to wifi"), "Connect to Wi-Fi");
        assert_eq!(top("pair my headphones"), "Pair a Bluetooth device");
        assert_eq!(top("where are the updates"), "Update the system");
        assert_eq!(top("is there a night light?"), "Night light");
        assert_eq!(
            top("my bluetooth keeps disconnecting"),
            "A Bluetooth device will not pair"
        );
        assert_eq!(
            top("what's the shortcut to close a window"),
            "Shortcuts for windows"
        );
        assert_eq!(top("how do I make text bigger"), "Make text bigger");
        assert_eq!(
            top("how do i turn on do not disturb"),
            "Do not disturb and notifications"
        );
        assert_eq!(top("what can you do?"), "Shaun");
        assert_eq!(top("the disk is full"), "The disk is full");
        assert_eq!(top("how do I add a dock"), "The dock");
        assert_eq!(top("what does Super + G do?"), "Shortcuts for windows");
        assert_eq!(
            top("what does super+z do"),
            "Workspaces and minimizing windows"
        );
        assert_eq!(
            top("how do I minimize a window"),
            "Workspaces and minimizing windows"
        );
        assert_eq!(top("wifi password"), "Connect to Wi-Fi");
        assert_eq!(top("turn off animations"), "Animations and motion");
        assert_eq!(
            top("my screen is black after plugging a monitor"),
            "A black screen after plugging a monitor"
        );
        assert_eq!(
            top("how do I import from 1password?"),
            "Import passwords from 1Password"
        );
        assert_eq!(top("can safe keep my ssh keys"), "SSH keys in ewe Safe");
        assert_eq!(top("does it support passkeys"), "Passkeys");
        assert_eq!(
            top("how do I get my passwords on my phone"),
            "ewe Safe on your phone"
        );
        assert_eq!(
            top("what is my password for github"),
            "Copy a password, a username or a code"
        );
        assert_eq!(
            top("how do I clear my browsing history"),
            "History in ewe Browser"
        );
        assert_eq!(
            top("does the browser sync my tabs from other computers"),
            "Sync in ewe Browser"
        );
        assert_eq!(
            top("how do I fill a password in the browser"),
            "ewe Safe in ewe Browser"
        );
        assert_eq!(top("wifi password"), "Connect to Wi-Fi", "still Wi-Fi's");
    }

    #[test]
    fn wave_two_questions_find_their_topic() {
        // cli.md: every tool by its name
        for (q, t) in [
            (
                "what does ewe-conf do",
                "ewe-conf, the settings file's tool",
            ),
            (
                "how do I use ewe-theme to import a base24 scheme",
                "ewe-theme, schemes from the terminal",
            ),
            (
                "how do I install a plugin with ewe-plugin",
                "ewe-plugin, plugins from the terminal",
            ),
            ("what is ewe-bt", "ewe-bt, Bluetooth from the terminal"),
            (
                "how do I mount nextcloud with ewe-files",
                "ewe-cloud, ewe-files and ewe-drive: your account and its folders",
            ),
            (
                "what does ewe-caldav print",
                "ewe-mail and ewe-caldav: mail and the calendar",
            ),
            (
                "what is ewe-auth for",
                "ewe-auth, the optional Google sign-in",
            ),
            (
                "ewe-globalshortcuts list",
                "ewe-globalshortcuts, apps' global shortcuts",
            ),
            (
                "what does ewe-sync --check do",
                "ewe-sync from the terminal",
            ),
            (
                "when do I run ewe-setup",
                "ewe-setup, install.sh and the installer",
            ),
            (
                "what can the ewe-browser cli do",
                "ewe-browser, the browser's command line",
            ),
            ("komble --search from a script", "komble from the terminal"),
            (
                "how do I use ewe-companion ask",
                "ewe-companion, me from the terminal",
            ),
            (
                "which qs ipc targets are there",
                "The shell's own commands (qs ipc)",
            ),
            ("where is ewe-pass", "ewe-pass, the Passwords plugin's tool"),
            (
                "what command line tools does ewe have",
                "The ewe command-line tools",
            ),
            // browser.md: the bottom bar, this wave's features
            (
                "how do browser workspaces work",
                "The bottom bar and workspaces in ewe Browser",
            ),
            (
                "can I search zara from the browser",
                "Search a site from ewe Browser",
            ),
            ("what are the browser shortcuts", "Shortcuts in ewe Browser"),
            (
                "how do I find text on a page",
                "Find on a page in ewe Browser",
            ),
            (
                "can I swipe back in the browser with two fingers",
                "Gestures and mouse buttons in ewe Browser",
            ),
            (
                "how do I turn on secure dns",
                "Privacy and security in ewe Browser",
            ),
            (
                "a site wants my camera permission",
                "Privacy and security in ewe Browser",
            ),
            (
                "does the bitwarden extension work in ewe Browser",
                "ewe Safe in ewe Browser",
            ),
            (
                "how do I open link in new workspace",
                "Right-click menus in ewe Browser",
            ),
            // safe.md
            (
                "how do I use ewe-safe run with an env file",
                "Use ewe Safe from scripts",
            ),
            ("what does quick access do", "Quick Access"),
            ("can safe be my ssh agent", "SSH keys in ewe Safe"),
            // howto.md: what Shaun does now
            ("how do I turn on airplane mode", "Airplane mode"),
            ("can you set a timer", "Timers and reminders"),
            ("remind me later", "Timers and reminders"),
            ("is there a calculator", "Do a sum"),
            ("how do I empty the trash", "Empty the trash"),
            ("my screen keeps turning off", "Keep the computer awake"),
        ] {
            assert_eq!(top(q), t, "{q:?}");
        }
        let p = pack();
        assert!(p.primer.contains("ewe-conf (settings)"), "the CLI index");
        assert!(
            p.primer.len() < 4_000,
            "the primer goes with every question: keep it small"
        );
    }

    #[test]
    fn the_ui_revamp_questions_find_their_topic() {
        // ewe 0.39, ewe Settings 0.22, Komble 0.21, ewe Sync 0.16,
        // ewe Browser 0.8, ewe Safe 0.4 (2026-11-01)
        for (q, t) in [
            (
                "how do I connect my github with an ssh key",
                "Connect to GitHub, GitLab or Azure with ewe Safe",
            ),
            (
                "can safe sign my git commits",
                "Connect to GitHub, GitLab or Azure with ewe Safe",
            ),
            ("how do I move ssh from 1password", "SSH keys in ewe Safe"),
            (
                "where is the power button in quick settings",
                "Quick settings",
            ),
            (
                "where did the system monitor go",
                "The System monitor: CPU, memory and internet speed",
            ),
            (
                "how do I see my internet speed",
                "The System monitor: CPU, memory and internet speed",
            ),
            (
                "sync says two computers changed my settings",
                "Sync will not upload or says another computer is newer",
            ),
            (
                "where are my plugin settings",
                "Add, remove or set up a plugin",
            ),
            ("back up this computer", "Back up and restore this computer"),
        ] {
            assert_eq!(top(q), t, "{q:?}");
        }
    }

    #[test]
    fn no_answer_quotes_a_retired_label() {
        // labels the 0.39 wave removed (the rail, the Options dialog,
        // Settings' old pages, ewe Sync's old words, the browser's old
        // save prompt, the Pen's keys); comments may still name them
        for (name, text) in BUILT_IN {
            let said = strip_comments(text);
            for gone in [
                "→ Options",
                "Options →",
                "Networking",
                "→ Startup →",
                "→ Shortcuts lists",
                "Settings → System",
                "Restart shell\"",
                "This machine",
                "Machines:",
                "→ Push anyway",
                "Save to ewe Safe?",
                "Settings → ewe ",
                "Super+S (or",
                "Notifications tab",
            ] {
                assert!(!said.contains(gone), "{name}.md still says {gone:?}");
            }
        }
    }

    #[test]
    fn typos_synonyms_and_names_still_find_their_topic() {
        // a typo of a word the pack knows, fixed for a little less
        assert_eq!(top("bluetoth headphones"), "Pair a Bluetooth device");
        assert_eq!(top("how do I chnage the walpaper"), "Change the wallpaper");
        // a synonym: the pack says "display", people say "monitor"
        assert_eq!(
            top("set up my second monitor"),
            "Change displays and scaling"
        );
        // "what is ewe" is not said in "what is ewe-bt"
        assert_eq!(top("what is ewe-bt"), "ewe-bt, Bluetooth from the terminal");
        assert_eq!(top("what is ewe"), "What is ewe");
        // "what is ewe" is cut short by a longer name that follows
        assert_eq!(top("what is ewe sync"), "ewe Sync, your account");
        assert_eq!(top("what is ewe bt"), "ewe-bt, Bluetooth from the terminal");
        // the question names the title: it wins a tie of phrases
        assert_eq!(
            top("where is the power button in quick settings"),
            "Quick settings"
        );
        // a short word's typo fix never makes a confident answer
        let p = pack();
        let rsync = p.search("how do I use rsync", 1);
        assert!(
            rsync.first().is_none_or(|h| h.score < CONFIDENT),
            "{:?}",
            rsync.first().map(|h| &h.topic.title)
        );
        // contractions are not words: "won't" matched every "won't pair"
        let p = pack();
        let best = p.search("it won't", 1).first().map(|h| h.score);
        assert!(best.is_none_or(|s| s < 1.0), "{best:?}");
        // one strong word alone is never confident; a key phrase is
        assert!(p.search("hyprland", 1)[0].score >= CONFIDENT);
        let one = p
            .search("glitter", 1)
            .first()
            .map(|h| h.score)
            .unwrap_or(0.0);
        assert!(one < CONFIDENT);
    }

    #[test]
    fn a_note_is_a_topic_and_finds_itself() {
        let n = parse_note(
            "printer.md",
            "# Printer\n\nThe printer is on the second floor, by the kitchen.\n",
        );
        assert_eq!(n.len(), 1);
        assert_eq!(n[0].title, "Printer");
        assert_eq!(n[0].note(), Some("printer.md"));
        assert_eq!(n[0].keys, ["printer"]);
        let long = parse_note(
            "x.md",
            "# The printer is on the second floor\n\nby the kitchen",
        );
        assert!(
            long[0].keys.contains(&"printer".to_string()),
            "{:?}",
            long[0].keys
        );
        assert!(long[0].keys.contains(&"floor".to_string()));
        let plain = parse_note("wifi at home.txt", "Home5G, the router is in the hall.");
        assert_eq!(plain[0].title, "wifi at home");
        assert!(plain[0].body.starts_with("Home5G"));
        // a note written like the pack keeps its topics
        let pack_like = parse_note(
            "garden.md",
            "## Water the tomatoes\nkeys: tomatoes, watering\nEvery other evening.\n\n## Compost\nkeys: compost\nTurn it on Sundays.\n",
        );
        assert_eq!(pack_like.len(), 2);
        assert!(parse_note("empty.md", "   \n").is_empty());
        // in one index with the pack, a note answers its own words
        let p = Pack::built_in().with(n);
        let h = &p.search("where is the printer", 1)[0];
        assert_eq!(h.topic.note(), Some("printer.md"));
        assert!(h.score >= CONFIDENT);
    }

    #[test]
    fn news_topics_name_their_release() {
        assert_eq!(news_version("What's new in ewe 0.45"), Some((0, 45)));
        assert_eq!(news_version("Change the wallpaper"), None);
        assert_eq!(minor_of("0.45.0-beta"), Some((0, 45)));
        assert_eq!(minor_of("v1.2"), Some((1, 2)));
        assert_eq!(minor_of("beta"), None);
        let p = pack();
        let news = p.news();
        assert!(news.len() >= 5, "news.md holds the last releases");
        assert!(news.windows(2).all(|w| w[0].0 > w[1].0), "newest first");
    }

    #[test]
    fn hyphens_and_phrases() {
        assert_eq!(hyphen_joined("what is ewe-bt"), [false, false, true, false]);
        assert_eq!(hyphen_joined("don't - stop"), [false, false]);
        let w = |s: &str| words(s);
        assert!(says_phrase(
            &w("what is ewe"),
            &hyphen_joined("what is ewe"),
            &w("what is ewe")
        ));
        assert!(!says_phrase(
            &w("what is ewe-bt"),
            &hyphen_joined("what is ewe-bt"),
            &w("what is ewe")
        ));
        assert!(says_phrase(
            &w("use ewe-bt now"),
            &hyphen_joined("use ewe-bt now"),
            &w("ewe bt")
        ));
    }

    /// Everything ewe ships (knowledge/coverage.json, written by
    /// scripts/coverage.py from the repos and the vault) has a topic whose
    /// title or keys name it: a new Settings page, ewe-* tool, app or
    /// built-in plugin without one fails the build.
    #[test]
    fn every_shipped_feature_has_a_topic() {
        let list: serde_json::Value =
            serde_json::from_str(include_str!("../../../knowledge/coverage.json"))
                .expect("coverage.json is JSON");
        let items = list["items"].as_array().expect("items");
        assert!(items.len() >= 60, "{} items", items.len());
        let p = pack();
        let names: Vec<Vec<Vec<String>>> = p
            .topics
            .iter()
            .map(|t| {
                std::iter::once(words(&t.title))
                    .chain(t.keys.iter().map(|k| words(k)))
                    .collect()
            })
            .collect();
        let mut missing = Vec::new();
        for it in items {
            let terms: Vec<Vec<String>> = it["terms"]
                .as_array()
                .expect("terms")
                .iter()
                .filter_map(|t| t.as_str())
                .map(words)
                .collect();
            let covered = names.iter().any(|fields| {
                fields
                    .iter()
                    .any(|f| terms.iter().any(|t| contains_run(f, t)))
            });
            if !covered {
                missing.push(format!("{} ({})", it["name"], it["kind"]));
            }
        }
        assert!(
            missing.is_empty(),
            "no topic names these (knowledge/*.md, docs/EXTENDING.md recipe 4): {missing:#?}"
        );
    }

    #[test]
    fn small_talk_finds_nothing_confident() {
        let p = pack();
        for q in [
            "hi",
            "thanks",
            "what's the weather in Tbilisi",
            "tell me a joke",
            "is it safe to do that",
        ] {
            let best = p.search(q, 1).first().map(|h| h.score).unwrap_or(0.0);
            assert!(best < CONFIDENT, "{q:?} scored {best}");
        }
    }

    #[test]
    fn a_wallpaper_question_is_confident_and_offers_settings() {
        let p = pack();
        let h = &p.search("change the wallpaper", 1)[0];
        assert!(h.score >= CONFIDENT);
        assert_eq!(h.topic.open_label().as_deref(), Some("Open ewe Settings"));
        assert!(h.topic.short().starts_with("ewe Settings → Wallpaper"));
    }

    #[test]
    fn the_answer_is_the_paragraph_that_fits() {
        let p = pack();
        let q = "what does super+g do";
        let h = &p.search(q, 1)[0];
        assert!(
            h.topic.answer(q).contains("Super+G groups"),
            "{}",
            h.topic.answer(q)
        );
        // the Pen went in ewe 0.34: asking for it gets minimizing
        let q = "how do I stash a window in the pen";
        let h = &p.search(q, 1)[0];
        assert!(
            h.topic.answer(q).starts_with("Super+Z minimizes"),
            "{}",
            h.topic.answer(q)
        );
        let q = "can you open apps?";
        let h = &p.search(q, 1)[0];
        assert_eq!(h.topic.answer(q), h.topic.short(), "{}", h.topic.title);
        let q = "change the wallpaper";
        assert_eq!(
            p.search(q, 1)[0].topic.answer(q),
            p.search(q, 1)[0].topic.short()
        );
    }

    #[test]
    fn labels() {
        let l = |what: &str, page: &str| {
            open_label(&Call::new("open", json!({"what": what, "page": page})))
        };
        assert_eq!(l("komble", "updates"), "Open Updates");
        assert_eq!(l("quicksettings", "wifi"), "Open Wi-Fi");
        assert_eq!(l("komble", "options=ewe.dock"), "Open its options");
        assert_eq!(l("overview", ""), "Open the Overview");
    }

    #[test]
    fn stems_and_stop_words() {
        assert_eq!(
            words("Updates, pairing; batteries!"),
            ["update", "pair", "battery"]
        );
        assert_eq!(
            words("stashes switches boxes glass"),
            ["stash", "switch", "box", "glass"]
        );
        assert_eq!(content(words("how do I do it")), Vec::<String>::new());
    }
}

#[cfg(test)]
mod calibrate {
    use super::*;
    #[test]
    #[ignore]
    fn print_scores() {
        let p = Pack::built_in();
        let qs = std::env::var("QS").unwrap_or_default();
        for q in qs.split('|') {
            let hits = p.search(q, 3);
            println!("Q {q:?}");
            for h in hits {
                println!("   {:6.2} {}", h.score, h.topic.title);
            }
        }
    }
}
