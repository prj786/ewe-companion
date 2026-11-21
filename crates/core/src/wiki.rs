//! Reading the ArchWiki on this computer (`arch-wiki-lite`: `wiki-search`
//! lists pages, `wiki-search <id>` prints one as text) and manual pages.
//! Pure: the daemon runs the tools, this reads what they print.
//!
//! - [`parse_hits`]: `wiki-search`'s list (`0  Bluetooth headset  00000441`);
//!   [`rank`] puts the pages whose titles say the question first (the tool
//!   ranks by how many lines match, which favours long pages).
//! - [`parse_page`]: a page as text, with the site's chrome dropped (the
//!   contents list, "Jump to content", the categories), cut into its
//!   sections by the headings the contents list names.
//! - [`excerpt`]: the section that fits the question, at most `max` bytes.
//! - [`man_sections`] and [`man_excerpt`]: the same for `man` output.

/// One page `wiki-search` found.
#[derive(Debug, Clone, PartialEq)]
pub struct WikiHit {
    pub title: String,
    /// the page's number in the wiki's text (eight digits)
    pub id: String,
}

/// `wiki-search <words>` as printed with no terminal: `N  Title  ID` rows
/// (and a "Now choose a page …" line, or "No matches found.").
pub fn parse_hits(out: &str) -> Vec<WikiHit> {
    out.lines()
        .filter_map(|l| {
            let l = l.trim();
            let mut parts = l.split_whitespace();
            let n = parts.next()?;
            if !n.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
            let id = l.split_whitespace().last()?;
            if id.len() != 8 || !id.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
            let title = l[n.len()..l.len() - id.len()].trim().to_string();
            (!title.is_empty()).then(|| WikiHit {
                title,
                id: id.to_string(),
            })
        })
        .collect()
}

fn low_words(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 1)
        .map(|w| {
            // "headsets" is "headset"
            if w.len() > 4 && w.ends_with('s') && !w.ends_with("ss") {
                w[..w.len() - 1].to_string()
            } else {
                w.to_string()
            }
        })
        .collect()
}

/// Words a wiki question carries no meaning in.
const FILLER: &[&str] = &[
    "the", "a", "an", "how", "do", "to", "in", "on", "of", "for", "my", "is", "what", "with",
    "and", "or", "i", "can", "set", "up", "use", "about", "arch", "linux", "wiki", "page",
];

/// The words of a question that matter to the wiki (no filler).
pub fn query_words(q: &str) -> Vec<String> {
    low_words(q)
        .into_iter()
        .filter(|w| !FILLER.contains(&w.as_str()))
        .collect()
}

/// The hits, the ones whose titles say most of the question first (a
/// title that is exactly the question first of all); the tool's order
/// breaks ties.
pub fn rank(query: &str, hits: Vec<WikiHit>) -> Vec<WikiHit> {
    let q = query_words(query);
    let mut scored: Vec<(f32, usize, WikiHit)> = hits
        .into_iter()
        .enumerate()
        .map(|(i, h)| {
            let t = low_words(&h.title.replace('_', " "));
            let said = q.iter().filter(|w| t.contains(w)).count() as f32;
            let extra = t.iter().filter(|w| !q.contains(w)).count() as f32;
            let exact = if !q.is_empty() && said as usize == q.len() && extra == 0.0 {
                2.0
            } else {
                0.0
            };
            (said * 2.0 - extra * 0.3 + exact, i, h)
        })
        .collect();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    scored.into_iter().map(|(_, _, h)| h).collect()
}

/// How sure a hit is: every word of the question is in its title.
pub fn title_says_all(query: &str, title: &str) -> bool {
    let q = query_words(query);
    let t = low_words(&title.replace('_', " "));
    !q.is_empty() && q.iter().all(|w| t.contains(w))
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Page {
    /// the page's address on wiki.archlinux.org (from "Retrieved from")
    pub url: Option<String>,
    /// the text before the first section
    pub intro: String,
    /// (heading, text) in order
    pub sections: Vec<(String, String)>,
}

fn is_number(l: &str) -> bool {
    !l.is_empty()
        && l.split('.')
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

/// A page as `wiki-search <id>` prints it (markers already dropped by the
/// tool when there is no terminal; dropped here too).
pub fn parse_page(text: &str) -> Page {
    let lines: Vec<String> = text
        .lines()
        .map(|l| l.replace("@@b", "").replace("@@w", "").trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    let start = lines.iter().position(|l| l == "From ArchWiki");
    // the contents list: a number line, then the heading
    let toc: Vec<String> = match start {
        Some(s) => lines[..s]
            .windows(2)
            .filter(|w| is_number(&w[0]) && !is_number(&w[1]))
            .map(|w| w[1].clone())
            .collect(),
        None => vec![],
    };
    let end = lines
        .iter()
        .position(|l| l.starts_with("Categories:") || l.starts_with("Retrieved from"))
        .unwrap_or(lines.len());
    let url = lines
        .iter()
        .find_map(|l| l.strip_prefix("Retrieved from \""))
        .and_then(|u| u.split("title=").nth(1))
        .map(|t| t.split(['&', '"']).next().unwrap_or(t))
        .map(|t| format!("https://wiki.archlinux.org/title/{t}"));
    let body = match start {
        Some(s) if s < end => &lines[s + 1..end],
        _ => &lines[..end],
    };
    // "Related articles" and its list of titles come first
    let mut i = 0;
    if body.first().is_some_and(|l| l == "Related articles") {
        i = 1;
        // short titles ("/Examples", "Sound system"); a page's own section
        // may share a name with one ("WirePlumber"), so only the length and
        // the full stop tell them from the text
        while i < body.len() && body[i].len() < 48 && !body[i].ends_with(['.', ':']) {
            i += 1;
        }
    }
    let mut page = Page {
        url,
        ..Default::default()
    };
    let mut next = 0;
    let mut current: Option<(String, Vec<String>)> = None;
    let mut intro = Vec::new();
    for l in &body[i..] {
        if next < toc.len() && *l == toc[next] {
            if let Some((h, t)) = current.take() {
                page.sections.push((h, t.join("\n")));
            }
            current = Some((l.clone(), Vec::new()));
            next += 1;
            continue;
        }
        if l.starts_with("Toggle ") && l.ends_with(" subsection") {
            continue;
        }
        match current.as_mut() {
            Some((_, t)) => t.push(l.clone()),
            None => intro.push(l.clone()),
        }
    }
    if let Some((h, t)) = current {
        page.sections.push((h, t.join("\n")));
    }
    page.intro = intro.join("\n");
    page
}

/// Text cut to `max` bytes at a line (or word) end.
pub fn bounded(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.trim().to_string();
    }
    let mut cut = max;
    while !text.is_char_boundary(cut) {
        cut -= 1;
    }
    let head = &text[..cut];
    let at = head
        .rfind('\n')
        .filter(|&i| i > max / 2)
        .or_else(|| head.rfind(' '));
    format!("{} …", head[..at.unwrap_or(cut)].trim_end())
}

fn hits_in(text: &str, q: &[String]) -> usize {
    let w = low_words(text);
    q.iter()
        .map(|x| w.iter().filter(|y| *y == x).count().min(4))
        .sum()
}

/// The part of a page that fits the question: the section that says most
/// of its words (then most often; a heading counts twice), else the
/// introduction; `max` bytes at most. Words of the page's `title` do not
/// count: "bluetooth headset" on the Bluetooth headset page is its
/// introduction, "bluetooth headset is silent" its Troubleshooting.
pub fn excerpt(page: &Page, title: &str, query: &str, max: usize) -> (Option<String>, String) {
    // the words that only name the page say nothing about which part
    let named = low_words(&title.replace('_', " "));
    let q: Vec<String> = query_words(query)
        .into_iter()
        .filter(|w| !named.contains(w))
        .collect();
    let best = page
        .sections
        .iter()
        .map(|(h, t)| {
            // the question's words it says at all count most, then how often
            let said = q
                .iter()
                .filter(|w| {
                    hits_in(h, std::slice::from_ref(w)) + hits_in(t, std::slice::from_ref(w)) > 0
                })
                .count();
            (said * 3 + hits_in(h, &q) * 2 + hits_in(t, &q), h, t)
        })
        .filter(|(s, _, t)| *s > 0 && !t.trim().is_empty())
        .max_by(|a, b| a.0.cmp(&b.0));
    let intro_score = hits_in(&page.intro, &q);
    match best {
        Some((s, h, t)) if s > intro_score || page.intro.trim().is_empty() => {
            (Some(h.clone()), bounded(t, max))
        }
        _ if !page.intro.trim().is_empty() => (None, bounded(&page.intro, max)),
        _ => match page.sections.first() {
            Some((h, t)) => (Some(h.clone()), bounded(t, max)),
            None => (None, String::new()),
        },
    }
}

/// `man` output as (heading, text) sections: a heading is a line in capital
/// letters at the start of the line (NAME, SYNOPSIS, OPTIONS …). Backspace
/// overstrikes (bold, underline) are dropped first.
pub fn man_sections(text: &str) -> Vec<(String, String)> {
    let mut clean = String::with_capacity(text.len());
    for c in text.chars() {
        if c == '\u{8}' {
            clean.pop();
        } else {
            clean.push(c);
        }
    }
    let mut out: Vec<(String, Vec<String>)> = Vec::new();
    for l in clean.lines() {
        let is_head = !l.starts_with(' ')
            && !l.is_empty()
            && l.chars().any(|c| c.is_alphabetic())
            && l.chars().all(|c| !c.is_lowercase())
            && l.len() < 40;
        if is_head {
            out.push((l.trim().to_string(), Vec::new()));
        } else if let Some((_, t)) = out.last_mut() {
            t.push(l.trim_end().to_string());
        }
    }
    out.into_iter()
        .map(|(h, t)| (h, t.join("\n").trim_matches('\n').to_string()))
        .filter(|(h, _)| !h.is_empty())
        .collect()
}

/// What a manual says that fits the question: its NAME line, its SYNOPSIS,
/// and the paragraph (an option, an example) that says most of the
/// question's words, `max` bytes in all.
pub fn man_excerpt(sections: &[(String, String)], query: &str, max: usize) -> String {
    let get = |name: &str| {
        sections
            .iter()
            .find(|(h, _)| h == name)
            .map(|(_, t)| t.as_str())
            .unwrap_or("")
    };
    let squash = |t: &str| t.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out = String::new();
    let name = squash(get("NAME"));
    if !name.is_empty() {
        out.push_str(&name);
        out.push('\n');
    }
    let syn: Vec<String> = get("SYNOPSIS")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .take(3)
        .map(str::to_string)
        .collect();
    if !syn.is_empty() {
        out.push_str(&format!("Usage: {}\n", syn.join(" | ")));
    }
    let q = query_words(query);
    if !q.is_empty() {
        // paragraphs: blocks between blank lines, in any section but NAME
        let best = sections
            .iter()
            .filter(|(h, _)| h != "NAME" && h != "SYNOPSIS")
            .flat_map(|(_, t)| t.split("\n\n"))
            .map(|p| (hits_in(p, &q), p))
            .filter(|(s, _)| *s > 0)
            .max_by(|a, b| a.0.cmp(&b.0).then(b.1.len().cmp(&a.1.len())));
        if let Some((_, p)) = best {
            let p: Vec<&str> = p.lines().map(str::trim).collect();
            out.push_str(&p.join("\n"));
        }
    } else if let Some((_, d)) = sections.iter().find(|(h, _)| h == "DESCRIPTION") {
        out.push_str(&squash(d.split("\n\n").next().unwrap_or("")));
    }
    bounded(&out, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: &str = "Now choose a page with wiki-search [number]\n\n0  Bluetooth                 00000440\n1  Bluetooth headset         00000441\n2  PipeWire                  00000877\n";

    const PAGE: &str = "Jump to content\nContents\nmove to sidebar\nhide\nBeginning\n1\nHeadset via PipeWire\nToggle Headset via PipeWire subsection\n1.1\nTroubleshooting\n2\nPairing\nFrom ArchWiki\nRelated articles\nBluetooth\n/Troubleshooting\nPairing\nThis article covers the configuration of Bluetooth headsets.\nHeadset via PipeWire\nPipeWire acts as a drop-in replacement for PulseAudio and supports Bluetooth.\nTroubleshooting\nIf the headset is silent, restart the wireplumber service.\nPairing\nPut the headset in pairing mode and run bluetoothctl.\nCategories: \nSound\nRetrieved from \"https://wiki.archlinux.org/index.php?title=Bluetooth_headset&oldid=887463\"\n";

    #[test]
    fn the_list_and_the_best_title() {
        let hits = parse_hits(LIST);
        assert_eq!(hits.len(), 3);
        assert_eq!(hits[1].title, "Bluetooth headset");
        assert_eq!(hits[1].id, "00000441");
        assert!(parse_hits("No matches found.").is_empty());
        let r = rank("bluetooth headset not working", hits.clone());
        assert_eq!(r[0].title, "Bluetooth headset");
        let r = rank("pipewire", hits);
        assert_eq!(r[0].title, "PipeWire");
        assert!(title_says_all("bluetooth headset", "Bluetooth headset"));
        assert!(!title_says_all("bluetooth headset", "Bluetooth"));
    }

    #[test]
    fn a_page_in_sections() {
        let p = parse_page(PAGE);
        assert_eq!(
            p.url.as_deref(),
            Some("https://wiki.archlinux.org/title/Bluetooth_headset")
        );
        assert_eq!(
            p.intro,
            "This article covers the configuration of Bluetooth headsets."
        );
        let heads: Vec<&str> = p.sections.iter().map(|(h, _)| h.as_str()).collect();
        assert_eq!(
            heads,
            ["Headset via PipeWire", "Troubleshooting", "Pairing"]
        );
        let (h, t) = excerpt(&p, "Bluetooth headset", "headset is silent", 500);
        assert_eq!(h.as_deref(), Some("Troubleshooting"));
        assert!(t.contains("wireplumber"));
        let (h, _) = excerpt(&p, "Bluetooth headset", "bluetooth headset", 500);
        assert_eq!(h, None, "the introduction when nothing stands out");
        assert!(bounded(&"word ".repeat(100), 50).len() <= 54);
    }

    #[test]
    fn a_manual_page() {
        let man = "RSYNC(1)\n\nNAME\n       rsync - a fast, versatile, remote (and local) file-copying tool\n\nSYNOPSIS\n       rsync [OPTION...] SRC... [DEST]\n\nDESCRIPTION\n       Rsync is a fast and extraordinarily versatile file copying tool.\n\nOPTIONS\n       --exclude=PATTERN\n              exclude files matching PATTERN\n\n       --delete\n              delete extraneous files from dest dirs\n";
        let s = man_sections(man);
        assert!(s.iter().any(|(h, _)| h == "OPTIONS"));
        let e = man_excerpt(&s, "exclude a folder", 800);
        assert!(e.starts_with("rsync - a fast"), "{e}");
        assert!(e.contains("Usage: rsync [OPTION...]"));
        assert!(e.contains("--exclude=PATTERN"), "{e}");
        let e = man_excerpt(&s, "", 800);
        assert!(e.contains("extraordinarily versatile"));
        // bold by overstrike
        let s = man_sections("N\u{8}NA\u{8}AM\u{8}ME\u{8}E\n  ls - list\n");
        assert_eq!(s[0].0, "NAME");
    }
}
