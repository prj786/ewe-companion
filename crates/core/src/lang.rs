//! Languages: which keyboard layout a language's name means, which
//! languages are typed with an input method (`ewe-ime`), and whether a
//! question is written in English at all.
//!
//! Pure, so the matcher can tell "switch to georgian keyboard" (a layout
//! the user may have) from "bluetooth keyboard" (not a language), and the
//! chat can tell a model to answer in the question's language (the offline
//! pack is English).

/// Languages people name → the xkb layout code ewe Settings adds for them
/// (Settings → Keyboard and mouse → Keyboard layouts). The code itself is
/// accepted too ("switch to the ru layout").
pub const XKB: &[(&[&str], &str)] = &[
    (
        &[
            "english",
            "us",
            "american",
            "american english",
            "english us",
        ],
        "us",
    ),
    (&["british", "uk", "british english", "english uk"], "gb"),
    (&["georgian", "ქართული"], "ge"),
    (&["russian", "русский"], "ru"),
    (&["ukrainian", "українська"], "ua"),
    (&["belarusian"], "by"),
    (&["german", "deutsch"], "de"),
    (&["french", "français", "francais"], "fr"),
    (&["spanish", "español", "espanol"], "es"),
    (&["latin american", "latam"], "latam"),
    (&["italian", "italiano"], "it"),
    (&["portuguese", "português", "portugues"], "pt"),
    (&["brazilian", "brazilian portuguese"], "br"),
    (&["polish", "polski"], "pl"),
    (&["turkish", "türkçe", "turkce"], "tr"),
    (&["greek", "ελληνικά"], "gr"),
    (&["hebrew", "עברית"], "il"),
    (&["arabic", "العربية"], "ara"),
    (&["persian", "farsi", "فارسی"], "ir"),
    (&["czech", "čeština"], "cz"),
    (&["slovak"], "sk"),
    (&["hungarian", "magyar"], "hu"),
    (&["romanian", "română"], "ro"),
    (&["bulgarian", "български"], "bg"),
    (&["serbian", "српски"], "rs"),
    (&["croatian", "hrvatski"], "hr"),
    (&["slovenian"], "si"),
    (&["swedish", "svenska"], "se"),
    (&["norwegian", "norsk"], "no"),
    (&["danish", "dansk"], "dk"),
    (&["finnish", "suomi"], "fi"),
    (&["icelandic"], "is"),
    (&["dutch", "nederlands"], "nl"),
    (&["belgian"], "be"),
    (&["swiss"], "ch"),
    (&["estonian"], "ee"),
    (&["latvian"], "lv"),
    (&["lithuanian"], "lt"),
    (&["armenian", "հայերեն"], "am"),
    (&["azerbaijani", "azeri"], "az"),
    (&["kazakh"], "kz"),
    (&["uzbek"], "uz"),
    (&["mongolian"], "mn"),
    (&["japanese", "日本語"], "jp"),
    (&["korean", "한국어"], "kr"),
    (&["chinese", "中文"], "cn"),
    (&["thai"], "th"),
    (&["vietnamese"], "vn"),
    (&["hindi", "indian"], "in"),
    (&["irish"], "ie"),
];

/// The xkb code a language's name (or a code) stands for.
pub fn xkb_code(word: &str) -> Option<&'static str> {
    let w = word.trim().to_lowercase();
    let w = w.strip_suffix(" language").unwrap_or(&w).trim().to_string();
    XKB.iter()
        .find(|(names, code)| names.contains(&w.as_str()) || *code == w)
        .map(|(_, code)| *code)
}

/// The language a layout code is, for a reply ("Georgian").
pub fn xkb_name(code: &str) -> Option<String> {
    let (names, _) = XKB.iter().find(|(_, c)| *c == code)?;
    let n = names.first()?;
    let mut c = n.chars();
    Some(match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    })
}

/// Languages typed with an input method (`ewe-ime`'s catalog ids, ewe
/// 0.46): a layout alone does not type them.
pub const IME: &[(&[&str], &str)] = &[
    (&["chinese", "mandarin", "pinyin", "中文"], "pinyin"),
    (&["japanese", "日本語", "mozc"], "mozc"),
    (&["korean", "한국어", "hangul"], "hangul"),
    (&["vietnamese", "tiếng việt", "unikey", "telex"], "unikey"),
    (&["hindi", "हिन्दी"], "m17n_hi_inscript2"),
    (&["bengali", "bangla", "বাংলা"], "m17n_bn_inscript2"),
    (&["tamil", "தமிழ்"], "m17n_ta_tamil99"),
    (&["telugu"], "m17n_te_inscript2"),
    (&["marathi"], "m17n_mr_inscript2"),
    (&["gujarati"], "m17n_gu_inscript2"),
    (&["punjabi"], "m17n_pa_inscript2-guru"),
    (&["kannada"], "m17n_kn_inscript2"),
    (&["malayalam"], "m17n_ml_inscript2"),
    (&["odia", "oriya"], "m17n_or_inscript2"),
    (&["nepali"], "m17n_ne_rom"),
    (&["sinhala"], "m17n_si_wijesekara"),
    (&["thai"], "m17n_th_kesmanee"),
    (&["amharic"], "m17n_am_sera"),
    (&["tibetan"], "m17n_bo_ewts"),
    (&["khmer"], "m17n_km_yannis"),
    (&["urdu"], "m17n_ur_phonetic"),
];

/// The input method a language's name stands for (`ewe-ime add <id>`).
pub fn ime_id(word: &str) -> Option<&'static str> {
    let w = word.trim().to_lowercase();
    let w = w.strip_suffix(" language").unwrap_or(&w).trim();
    IME.iter()
        .find(|(names, id)| names.contains(&w) || *id == w)
        .map(|(_, id)| *id)
}

/// Common short words of English. A question in Latin letters with none of
/// these and some of another language's is most likely not English.
const ENGLISH: &[&str] = &[
    "the", "a", "an", "is", "are", "was", "to", "of", "in", "on", "for", "and", "or", "my", "i",
    "you", "it", "how", "what", "why", "where", "when", "can", "do", "does", "with", "this",
    "that", "me", "please", "open", "show", "turn", "set", "play", "find", "make", "get", "not",
    "won't", "don't", "can't", "there", "help", "add", "switch", "change", "use",
];

/// Short words of other languages written in Latin letters (Spanish,
/// French, German, Italian, Portuguese, Dutch, Polish, Turkish, Czech,
/// Swedish …), none of them English words.
const OTHER: &[&str] = &[
    // es / pt / it / fr
    "el",
    "los",
    "una",
    "uno",
    "es",
    "por",
    "para",
    "como",
    "qué",
    "que",
    "cómo",
    "dónde",
    "puedo",
    "mi",
    "mis",
    "del",
    "con",
    "sin",
    "pero",
    "está",
    "esta",
    "hola",
    "gracias",
    "não",
    "nao",
    "você",
    "voce",
    "meu",
    "minha",
    "obrigado",
    "il",
    "lo",
    "gli",
    "della",
    "sono",
    "perché",
    "grazie",
    "le",
    "les",
    "des",
    "est",
    "une",
    "et",
    "je",
    "tu",
    "vous",
    "nous",
    "mon",
    "ma",
    "avec",
    "comment",
    "pourquoi",
    "où",
    "merci",
    "bonjour",
    "c'est",
    "qu'est-ce",
    // de / nl
    "der",
    "das",
    "ist",
    "und",
    "ich",
    "nicht",
    "mein",
    "meine",
    "wie",
    "warum",
    "wo",
    "kann",
    "mit",
    "ein",
    "eine",
    "bitte",
    "danke",
    "het",
    "een",
    "ik",
    "niet",
    "hoe",
    "waarom",
    "kan",
    "mijn",
    "dank",
    // pl / cs / tr / sv / ro
    "jak",
    "nie",
    "jest",
    "się",
    "mój",
    "moje",
    "dlaczego",
    "proszę",
    "dziękuję",
    "jsem",
    "není",
    "proč",
    "děkuji",
    "nasıl",
    "neden",
    "bir",
    "ve",
    "bu",
    "için",
    "teşekkürler",
    "och",
    "inte",
    "hur",
    "varför",
    "jag",
    "și",
    "este",
    "cum",
    "mulțumesc",
];

/// Whether a question is written in another language than English: any
/// letters of another script (Cyrillic, Georgian, Greek, Arabic, Hebrew,
/// CJK …), or Latin words that are another language's and not English's.
/// The offline pack is English; a model is told to answer in the
/// question's own language anyway, so this only decides what to say when
/// there is no model.
pub fn foreign(text: &str) -> bool {
    let letters: Vec<char> = text.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.is_empty() {
        return false;
    }
    let other_script = letters
        .iter()
        .filter(|c| !(c.is_ascii_alphabetic() || ('\u{00C0}'..='\u{024F}').contains(c)))
        .count();
    if other_script * 3 >= letters.len() {
        return true;
    }
    let low = text.to_lowercase();
    let words: Vec<&str> = low
        .split(|c: char| !(c.is_alphabetic() || c == '\'' || c == '-'))
        .filter(|w| !w.is_empty())
        .collect();
    let english = words.iter().filter(|w| ENGLISH.contains(w)).count();
    let other = words.iter().filter(|w| OTHER.contains(w)).count();
    other >= 1 && other > english
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn languages_name_their_layouts() {
        assert_eq!(xkb_code("Georgian"), Some("ge"));
        assert_eq!(xkb_code("russian language"), Some("ru"));
        assert_eq!(xkb_code("ru"), Some("ru"));
        assert_eq!(xkb_code("ქართული"), Some("ge"));
        assert_eq!(xkb_code("bluetooth"), None);
        assert_eq!(xkb_name("ge").as_deref(), Some("Georgian"));
        assert_eq!(xkb_name("zz"), None);
    }

    #[test]
    fn some_languages_need_an_input_method() {
        assert_eq!(ime_id("Japanese"), Some("mozc"));
        assert_eq!(ime_id("chinese"), Some("pinyin"));
        assert_eq!(ime_id("hindi"), Some("m17n_hi_inscript2"));
        assert_eq!(ime_id("georgian"), None, "a layout types it");
    }

    #[test]
    fn foreign_questions_are_told_apart() {
        for q in [
            "как сменить обои",
            "როგორ შევცვალო ფონი",
            "壁紙を変えるには",
            "¿cómo cambio el fondo de pantalla?",
            "comment changer le fond d'écran",
            "wie ändere ich das hintergrundbild",
            "jak zmienić tapetę",
        ] {
            assert!(foreign(q), "{q}");
        }
        for q in [
            "how do I change the wallpaper",
            "volume 40",
            "what is the time in tokyo",
            "play despacito",
            "open zed",
            "",
            "12 * 7",
        ] {
            assert!(!foreign(q), "{q}");
        }
    }
}
