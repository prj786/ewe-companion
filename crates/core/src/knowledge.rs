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
        ("settings", _) => "Open Settings".into(),
        ("komble", "") => "Open Komble".into(),
        ("komble", p) if p.starts_with("options=") => "Open its options".into(),
        ("komble", p) => format!("Open {}", page_label("komble", p).unwrap_or("Komble")),
        ("quicksettings", "" | "home") => "Open Quick settings".into(),
        ("quicksettings", p) => format!(
            "Open {}",
            page_label("quicksettings", p).unwrap_or("Quick settings")
        ),
        ("overview", _) => "Open the Overview".into(),
        ("sync", _) => "Open ewe-sync".into(),
        ("terminal", _) => "Open a terminal".into(),
        _ => "Open Files".into(),
    }
}

#[derive(Debug, Clone)]
pub struct Pack {
    pub primer: String,
    pub topics: Vec<Topic>,
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
    "laptop",
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

fn contains_run(hay: &[String], needle: &[String]) -> bool {
    !needle.is_empty() && hay.windows(needle.len()).any(|w| w == needle)
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
        Pack { primer, topics }
    }

    /// The pack built into this binary.
    pub fn built_in() -> Pack {
        Pack::from_files(BUILT_IN.iter().copied())
    }

    /// Score every topic against a question; the best `n` that match at all.
    ///
    /// A key phrase said in the question counts most (more for longer
    /// phrases), then a word of the title, then a word that is part of a
    /// key, then (a little) a word of the text.
    pub fn search(&self, question: &str, n: usize) -> Vec<Hit<'_>> {
        let all = words(question);
        let q = content(all.clone());
        let asked_combos = combos(question);
        let mut hits: Vec<Hit> = self
            .topics
            .iter()
            .filter_map(|t| {
                let mut score = 0.0f32;
                let mut key_words: Vec<String> = Vec::new();
                for k in &t.keys {
                    let kw = words(k);
                    // a phrase is matched on the whole question, so "do not
                    // disturb" still counts though "do" and "not" are stop words
                    if contains_run(&all, &kw) {
                        score += 3.0 + 2.0 * (kw.len() as f32 - 1.0);
                    }
                    key_words.extend(kw);
                }
                let title = content(words(&t.title));
                let body = words(&t.body);
                let mut body_score = 0.0f32;
                let mut seen: Vec<&str> = Vec::new();
                for w in &q {
                    if seen.contains(&w.as_str()) {
                        continue;
                    }
                    seen.push(w);
                    if title.contains(w) {
                        score += 1.5;
                    }
                    if key_words.contains(w) {
                        score += 1.0;
                    }
                    if body.contains(w) {
                        body_score += 0.3;
                    }
                }
                score += body_score.min(1.5);
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
        assert_eq!(top("what can you do?"), "The companion");
        assert_eq!(top("the disk is full"), "The disk is full");
        assert_eq!(top("how do I add a dock"), "The dock");
        assert_eq!(top("what does Super + G do?"), "Shortcuts for windows");
        assert_eq!(top("what is super+shift+s"), "Workspaces and the Pen");
        assert_eq!(top("wifi password"), "Connect to Wi-Fi");
        assert_eq!(top("turn off animations"), "Animations and motion");
        assert_eq!(
            top("my screen is black after plugging a monitor"),
            "A black screen after plugging a monitor"
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
        assert_eq!(h.topic.open_label().as_deref(), Some("Open Settings"));
        assert!(h.topic.short().starts_with("Settings → Wallpaper"));
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
        let q = "how do I stash a window in the pen";
        let h = &p.search(q, 1)[0];
        assert!(
            h.topic.answer(q).starts_with("The Pen is"),
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
