//! Typo tolerance for the offline matcher: "opne firefox", "volme 50",
//! "brigthness up".
//!
//! When the matcher misses, [`correct`] fixes the command words once and
//! the matcher tries again. A word is replaced only when it is not one of
//! the matcher's own words, has at least four letters, and is within one
//! edit (four to six letters) or two edits (seven and more) of exactly one
//! of them; a swap of two neighbours counts as one edit. Ambiguous words
//! ("mote": mute or mode?) and short ones stay as they are, and so does
//! everything after a word that starts free text (an app's name after
//! "open", a search after "search", a memory after "remember"): those are
//! the user's own words, never corrected.

/// The matcher's own words: what [`correct`] may correct to.
const VOCABULARY: &[&str] = &[
    // apps and places
    "open",
    "launch",
    "start",
    "close",
    "quit",
    "show",
    "settings",
    "terminal",
    "files",
    "folder",
    "overview",
    "install",
    "updates",
    "plugins",
    // sound and screen
    "volume",
    "sound",
    "louder",
    "quieter",
    "softer",
    "mute",
    "unmute",
    "brightness",
    "brighter",
    "dimmer",
    "darker",
    "screen",
    "screenshot",
    "increase",
    "decrease",
    "raise",
    "lower",
    "higher",
    // media
    "play",
    "pause",
    "resume",
    "next",
    "previous",
    "track",
    "song",
    "music",
    "skip",
    // switches
    "turn",
    "switch",
    "enable",
    "disable",
    "wifi",
    "bluetooth",
    "airplane",
    "flight",
    "mode",
    "game",
    "dark",
    "light",
    "disturb",
    "notifications",
    "keep",
    "awake",
    "connect",
    "disconnect",
    // timers
    "timer",
    "timers",
    "remind",
    "reminder",
    "reminders",
    "minute",
    "minutes",
    "second",
    "seconds",
    "hour",
    "hours",
    "cancel",
    // windows and workspaces
    "workspace",
    "window",
    "windows",
    "fullscreen",
    "maximise",
    "maximize",
    "minimise",
    "minimize",
    "float",
    "floating",
    "move",
    "kill",
    "force",
    "frozen",
    "responding",
    // this computer
    "battery",
    "memory",
    "uptime",
    "disk",
    "space",
    "network",
    "internet",
    "connected",
    "address",
    "using",
    "computer",
    "laptop",
    "slow",
    // dates, units, sums
    "what",
    "time",
    "date",
    "today",
    "tomorrow",
    "until",
    "christmas",
    "convert",
    "calculate",
    "square",
    "root",
    "squared",
    "cubed",
    // memory
    "remember",
    "forget",
    "everything",
    // the rest
    // (not "lock": "lick" or "luck" must never lock the screen)
    "search",
    "find",
    "bookmark",
    "bookmarks",
    "history",
    "tabs",
    "reload",
    "refresh",
    "safe",
    "password",
    "suspend",
    "restart",
    "empty",
    "trash",
    "make",
    "more",
    "less",
    "down",
    "full",
    "list",
];

/// After these words the rest is the user's own (an app, a query, a
/// memory, a name): nothing after them is corrected.
const FREE_TEXT: &[&str] = &[
    "open",
    "launch",
    "start",
    "run",
    "close",
    "quit",
    "kill",
    "search",
    "google",
    "find",
    "locate",
    "play",
    "install",
    "remember",
    "forget",
    "copy",
    "connect",
    "disconnect",
    "called",
    "named",
    "about",
    "convert",
    "remind",
];

/// Edit distance with a swap of neighbours as one edit (optimal string
/// alignment), stopping early past `max`.
fn distance(a: &str, b: &str, max: usize) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > max {
        return max + 1;
    }
    let (n, m) = (a.len(), b.len());
    let mut d = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut v = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                v = v.min(d[i - 2][j - 2] + 1);
            }
            d[i][j] = v;
        }
    }
    d[n][m]
}

/// The one vocabulary word this word is a typo of: the closest within the
/// limit, when no other is as close. A word and its plural tied ("minuts":
/// minute or minutes?) go by the typo's last letter.
fn fix(word: &str) -> Option<&'static str> {
    let len = word.chars().count();
    if len < 4 || !word.chars().all(|c| c.is_ascii_lowercase()) || VOCABULARY.contains(&word) {
        return None;
    }
    let max = if len >= 7 { 2 } else { 1 };
    let mut best: Vec<&'static str> = Vec::new();
    let mut best_d = max + 1;
    for v in VOCABULARY.iter().filter(|v| v.len() >= 4) {
        let d = distance(word, v, max);
        if d < best_d {
            best_d = d;
            best = vec![v];
        } else if d == best_d && d <= max {
            best.push(v);
        }
    }
    match best[..] {
        [one] => Some(one),
        [a, b] if b == format!("{a}s") || a == format!("{b}s") => {
            let plural = if a.len() > b.len() { a } else { b };
            let single = if a.len() > b.len() { b } else { a };
            Some(if word.ends_with('s') { plural } else { single })
        }
        _ => None,
    }
}

/// The words with their typos fixed, or None when nothing changed. `text`
/// is the matcher's cleaned text (lower case, single spaces).
pub fn correct(text: &str) -> Option<String> {
    let mut changed = false;
    let mut free = false;
    let words: Vec<String> = text
        .split(' ')
        .map(|w| {
            if free {
                return w.to_string();
            }
            let w2 = match fix(w) {
                Some(f) => {
                    changed = true;
                    f.to_string()
                }
                None => w.to_string(),
            };
            if FREE_TEXT.contains(&w2.as_str()) {
                free = true;
            }
            w2
        })
        .collect();
    changed.then(|| words.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swaps_and_slips_are_one_edit() {
        assert_eq!(distance("opne", "open", 2), 1);
        assert_eq!(distance("volme", "volume", 2), 1);
        assert_eq!(distance("brigthness", "brightness", 2), 1);
        assert_eq!(distance("mote", "mute", 2), 1);
        assert!(distance("abc", "xyzw", 1) > 1);
    }

    #[test]
    fn command_words_are_corrected() {
        assert_eq!(correct("opne firefox").as_deref(), Some("open firefox"));
        assert_eq!(correct("volme 50").as_deref(), Some("volume 50"));
        assert_eq!(correct("brigthness up").as_deref(), Some("brightness up"));
        assert_eq!(
            correct("set a timer for 5 minuets").as_deref(),
            Some("set a timer for 5 minutes")
        );
        assert_eq!(
            correct("timer for 3 minuts").as_deref(),
            Some("timer for 3 minutes")
        );
        assert_eq!(
            correct("swich to worksapce 3").as_deref(),
            Some("switch to workspace 3")
        );
        assert_eq!(
            correct("turn of blutooth").as_deref(),
            Some("turn of bluetooth")
        );
    }

    #[test]
    fn what_is_left_alone() {
        // mute or mode, timer or time: ambiguous
        assert_eq!(correct("mote"), None);
        assert_eq!(correct("mate"), None);
        assert_eq!(correct("timr 5 minutes"), None);
        // never the lock screen by a slip
        assert_eq!(correct("lick"), None);
        // short words, free text, words it knows
        assert_eq!(correct("opne fierfox").as_deref(), Some("open fierfox"));
        assert_eq!(correct("search for volme settings").as_deref(), None);
        assert_eq!(correct("remember that my dentst is on friday"), None);
        assert_eq!(correct("volume 50"), None);
        assert_eq!(correct("tell me a joke"), None);
        // a reminder's words are the user's: "take" must not become "make"
        assert_eq!(correct("remind me to take a break in 20 minuts"), None);
        assert_eq!(correct("why is the sky blue"), None);
    }
}
