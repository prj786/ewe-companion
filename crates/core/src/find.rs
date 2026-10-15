//! The finder's rules: what "find pdfs about tax" asks for, and how well a
//! name answers it. Pure, so every rule is tested here; the daemon's
//! `finder` walks the home folder with them.
//!
//! - **Every word must be in the name** ("ewe safe" finds `ewe-safe`:
//!   dashes, dots, spaces and case do not count).
//! - **Type words** become an extension filter: "pdfs" → `.pdf`, "photos"
//!   → `.jpg .png …`. "folder" or "directory" asks for folders only, and
//!   then a type word is just a word ("my pictures folder").
//! - **Rank:** the whole name (without its extension) is the words > the
//!   name starts with them > it contains them. Newer first within a rank
//!   (the daemon sorts by time).

/// Folders, files or both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Any,
    File,
    Folder,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    /// lower case, each must be in the name
    pub words: Vec<String>,
    /// lower case, without the dot; empty = any
    pub exts: Vec<&'static str>,
    pub kind: Kind,
}

impl Query {
    /// Nothing to look for ("find my files").
    pub fn is_empty(&self) -> bool {
        self.words.is_empty() && self.exts.is_empty()
    }
}

/// Words that carry no name: "find *my* resume", "pdfs *about* tax".
const FILLER: &[&str] = &[
    "my", "the", "a", "an", "all", "some", "any", "about", "for", "with", "called", "named",
    "that", "which", "me", "please", "i", "of", "on", "to", "is", "are", "where", "from", "this",
    "these", "those", "there",
];

/// Type words → extensions.
const TYPES: &[(&[&str], &[&str])] = &[
    (&["pdf", "pdfs"], &["pdf"]),
    (
        &["doc", "docx", "word", "odt"],
        &["doc", "docx", "odt", "rtf"],
    ),
    (
        &[
            "spreadsheet",
            "spreadsheets",
            "excel",
            "xlsx",
            "csv",
            "csvs",
        ],
        &["xlsx", "xls", "ods", "csv"],
    ),
    (
        &[
            "presentation",
            "presentations",
            "slides",
            "pptx",
            "powerpoint",
        ],
        &["pptx", "ppt", "odp", "key"],
    ),
    (
        &[
            "photo", "photos", "picture", "pictures", "image", "images", "pic", "pics", "jpg",
            "jpgs", "jpeg", "png", "pngs",
        ],
        &["jpg", "jpeg", "png", "webp", "heic", "gif", "avif"],
    ),
    (
        &[
            "video", "videos", "movie", "movies", "clip", "clips", "mp4", "mkv",
        ],
        &["mp4", "mkv", "webm", "mov", "avi"],
    ),
    (
        &["song", "songs", "audio", "mp3", "mp3s", "flac"],
        &["mp3", "flac", "ogg", "opus", "m4a", "wav"],
    ),
    (&["txt", "markdown"], &["txt", "md"]),
    (
        &["zip", "zips", "archive", "archives", "tarball"],
        &["zip", "tar", "gz", "xz", "zst", "7z", "rar"],
    ),
    (&["iso", "isos"], &["iso"]),
    (&["ebook", "ebooks", "epub", "epubs"], &["epub", "mobi"]),
];

const FOLDER_WORDS: &[&str] = &[
    "folder",
    "folders",
    "directory",
    "directories",
    "dir",
    "dirs",
];
const FILE_WORDS: &[&str] = &["file", "files"];

/// "ewe-safe", "Ewe Safe" and "ewe_safe" are the same name.
pub fn squash(s: &str) -> String {
    s.chars()
        .filter(|c| !matches!(c, '-' | '_' | ' ' | '.'))
        .flat_map(char::to_lowercase)
        .collect()
}

/// Plain words → what to look for.
pub fn parse(text: &str) -> Query {
    let low = text.to_lowercase();
    let raw: Vec<&str> = low
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| matches!(c, '?' | '!' | ',' | '"' | '\'' | ';')))
        .filter(|w| !w.is_empty())
        .collect();
    let folder = raw.iter().any(|w| FOLDER_WORDS.contains(w));
    let mut q = Query {
        words: Vec::new(),
        exts: Vec::new(),
        kind: if folder { Kind::Folder } else { Kind::Any },
    };
    for w in raw {
        if FILLER.contains(&w) || FOLDER_WORDS.contains(&w) {
            continue;
        }
        if FILE_WORDS.contains(&w) {
            q.kind = Kind::File;
            continue;
        }
        // ".pdf" names the extension itself
        let bare = w.strip_prefix('.').unwrap_or(w);
        if !folder && let Some((_, exts)) = TYPES.iter().find(|(ws, _)| ws.contains(&bare)) {
            for e in exts.iter() {
                if !q.exts.contains(e) {
                    q.exts.push(e);
                }
            }
            continue;
        }
        q.words.push(w.to_string());
    }
    if !q.exts.is_empty() {
        q.kind = Kind::File;
    }
    q
}

/// How well `name` answers `q`: 3 the whole name, 2 it starts with the
/// words, 1 it contains them; `None` when it does not answer.
pub fn rank(name: &str, is_dir: bool, q: &Query) -> Option<u8> {
    match q.kind {
        Kind::Folder if !is_dir => return None,
        Kind::File if is_dir => return None,
        _ => {}
    }
    let low = name.to_lowercase();
    let (stem, ext) = match low.rsplit_once('.') {
        Some((s, e)) if !is_dir && !s.is_empty() => (s, e),
        _ => (low.as_str(), ""),
    };
    if !q.exts.is_empty() && !q.exts.contains(&ext) {
        return None;
    }
    let flat = squash(&low);
    if !q.words.iter().all(|w| flat.contains(&squash(w))) {
        return None;
    }
    if q.words.is_empty() {
        return Some(1);
    }
    let joined: String = q.words.iter().map(|w| squash(w)).collect();
    let stem = squash(stem);
    Some(if stem == joined || flat == joined {
        3
    } else if stem.starts_with(&joined) {
        2
    } else {
        1
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_words_become_a_query() {
        let q = parse("my resume");
        assert_eq!(q.words, ["resume"]);
        assert_eq!(q.kind, Kind::Any);
        let q = parse("pdfs about tax");
        assert_eq!(
            (q.words.clone(), q.exts.clone()),
            (vec!["tax".to_string()], vec!["pdf"])
        );
        assert_eq!(q.kind, Kind::File);
        let q = parse("the ewe-safe folder");
        assert_eq!(
            (q.words.clone(), q.kind),
            (vec!["ewe-safe".to_string()], Kind::Folder)
        );
        // a folder of that name is not a type of file
        let q = parse("my pictures folder");
        assert_eq!(
            (q.words.clone(), q.exts.len()),
            (vec!["pictures".to_string()], 0)
        );
        let q = parse("photos from Paris");
        assert_eq!(q.words, ["paris"]);
        assert!(q.exts.contains(&"jpg"));
        assert_eq!(parse("the .iso").exts, ["iso"]);
        assert_eq!(parse("files called notes").kind, Kind::File);
        assert!(parse("my files").is_empty());
    }

    #[test]
    fn every_word_must_be_in_the_name() {
        let q = parse("ewe safe");
        assert_eq!(rank("ewe-safe", true, &q), Some(3));
        assert_eq!(rank("ewe-safe-notes.txt", false, &q), Some(2));
        assert_eq!(rank("old-ewe-safe", true, &q), Some(1));
        assert_eq!(rank("ewe-browser", true, &q), None);
        let q = parse("resume");
        assert_eq!(
            rank("Resume.pdf", false, &q),
            Some(3),
            "case and extension do not count"
        );
        assert_eq!(rank("resume-2026.odt", false, &q), Some(2));
        assert_eq!(rank("my-resume.docx", false, &q), Some(1));
    }

    #[test]
    fn kinds_and_types_filter() {
        let q = parse("pdfs about tax");
        assert_eq!(rank("tax-2025.pdf", false, &q), Some(2));
        assert_eq!(rank("tax-2025.odt", false, &q), None, "not a pdf");
        assert_eq!(rank("tax", true, &q), None, "a folder is not a pdf");
        let q = parse("the ewe-safe folder");
        assert_eq!(rank("ewe-safe", false, &q), None, "a file is not a folder");
        assert_eq!(rank("ewe-safe", true, &q), Some(3));
        // a type alone: every file of that type
        assert_eq!(rank("IMG_0001.JPG", false, &parse("my photos")), Some(1));
        assert_eq!(rank(".bashrc", false, &parse("bashrc")), Some(3));
    }
}
