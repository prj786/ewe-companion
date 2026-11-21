//! Dates and time zones, offline (C5): "how many days until christmas",
//! "what day is 25 december", "what's the date in 30 days", "what time is
//! it in tokyo".
//!
//! Pure date arithmetic on day numbers (days since 1970-01-01, the proleptic
//! Gregorian calendar; Howard Hinnant's civil-from-days), with today passed
//! in by the daemon (its local date), so every answer here is unit-tested.
//! The time in a place is the daemon's (it runs `date` with `TZ` in the
//! child's environment); which zone a place means is decided here.

pub const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

pub const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// Days since 1970-01-01 of a calendar date.
pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let (m, d) = (m as i64, d as i64);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The calendar date of a day number: (year, month 1–12, day 1–31).
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// 0 = Monday … 6 = Sunday (1970-01-01 was a Thursday).
pub fn weekday(days: i64) -> usize {
    (days + 3).rem_euclid(7) as usize
}

pub fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

pub fn days_in_month(y: i64, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(y) => 29,
        2 => 28,
        _ => 0,
    }
}

pub fn valid(y: i64, m: u32, d: u32) -> bool {
    (1..=12).contains(&m) && d >= 1 && d <= days_in_month(y, m) && (1..=9999).contains(&y)
}

/// Easter Sunday (the anonymous Gregorian algorithm): (month, day).
pub fn easter(y: i64) -> (u32, u32) {
    let a = y % 19;
    let b = y / 100;
    let c = y % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = (h + l - 7 * m + 114) % 31 + 1;
    (month as u32, day as u32)
}

/// "Friday 25 December 2026".
pub fn long(days: i64) -> String {
    let (y, m, d) = civil_from_days(days);
    format!(
        "{} {d} {} {y}",
        WEEKDAYS[weekday(days)],
        MONTHS[(m - 1) as usize]
    )
}

/// A day the words name.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Day {
    /// a date with its year
    On(i64, u32, u32),
    /// a date every year (the next one, or the last one)
    Yearly(u32, u32),
    Easter,
    /// a day of the week: the next one after today, or the last before
    Weekday(usize),
    /// today plus some days
    Offset(i64),
}

/// The days that have a name: (words, the day, how a reply says it).
const NAMED: &[(&[&str], Day, &str)] = &[
    (
        &["christmas", "christmas day", "xmas", "xmas day"],
        Day::Yearly(12, 25),
        "Christmas",
    ),
    (
        &["christmas eve", "xmas eve"],
        Day::Yearly(12, 24),
        "Christmas Eve",
    ),
    (&["boxing day"], Day::Yearly(12, 26), "Boxing Day"),
    (
        &[
            "new year",
            "new years",
            "new year's",
            "new year's day",
            "new years day",
            "the new year",
        ],
        Day::Yearly(1, 1),
        "New Year's Day",
    ),
    (
        &["new year's eve", "new years eve", "nye"],
        Day::Yearly(12, 31),
        "New Year's Eve",
    ),
    (&["halloween"], Day::Yearly(10, 31), "Halloween"),
    (
        &[
            "valentine's day",
            "valentines day",
            "valentine's",
            "valentines",
            "valentine day",
        ],
        Day::Yearly(2, 14),
        "Valentine's Day",
    ),
    (
        &[
            "st patrick's day",
            "saint patrick's day",
            "st patricks day",
            "st. patrick's day",
        ],
        Day::Yearly(3, 17),
        "St Patrick's Day",
    ),
    (
        &["april fools day", "april fools' day", "april fool's day"],
        Day::Yearly(4, 1),
        "April Fools' Day",
    ),
    (
        &["easter", "easter sunday", "easter day"],
        Day::Easter,
        "Easter",
    ),
    (&["today"], Day::Offset(0), "Today"),
    (&["tomorrow"], Day::Offset(1), "Tomorrow"),
    (&["yesterday"], Day::Offset(-1), "Yesterday"),
    (
        &["the day after tomorrow", "day after tomorrow"],
        Day::Offset(2),
        "The day after tomorrow",
    ),
    (
        &["the day before yesterday", "day before yesterday"],
        Day::Offset(-2),
        "The day before yesterday",
    ),
];

fn month_of(w: &str) -> Option<u32> {
    let w = w.trim_end_matches('.');
    if w.len() < 3 {
        return None;
    }
    MONTHS
        .iter()
        .position(|m| {
            let m = m.to_lowercase();
            m == w || (w.len() >= 3 && m.starts_with(w) && (w.len() == 3 || w == "sept"))
        })
        .map(|i| i as u32 + 1)
}

fn weekday_of(w: &str) -> Option<usize> {
    let w = w.trim_end_matches('.');
    const SHORT: [&[&str]; 7] = [
        &["mon"],
        &["tue", "tues"],
        &["wed"],
        &["thu", "thur", "thurs"],
        &["fri"],
        &["sat"],
        &["sun"],
    ];
    WEEKDAYS
        .iter()
        .position(|d| d.to_lowercase() == w)
        .or_else(|| SHORT.iter().position(|s| s.contains(&w)))
}

/// "25", "25th", "1st", "2nd", "3rd".
fn day_number(w: &str) -> Option<u32> {
    let w = w
        .strip_suffix("st")
        .or_else(|| w.strip_suffix("nd"))
        .or_else(|| w.strip_suffix("rd"))
        .or_else(|| w.strip_suffix("th"))
        .unwrap_or(w);
    let n: u32 = w.parse().ok()?;
    (1..=31).contains(&n).then_some(n)
}

fn year_number(w: &str) -> Option<i64> {
    let n: i64 = w.parse().ok()?;
    (w.len() == 4 && (1000..=9999).contains(&n)).then_some(n)
}

/// The day these words name, and its name when it has one: "christmas",
/// "next friday", "25 december", "the 25th of december 2027", "december
/// 25", "dec 25th", "2026-12-25", "25.12.2026", "tomorrow". None for
/// anything else ("it", "my birthday", "25/12": day and month could be
/// either way round).
pub fn parse(words: &str) -> Option<(Day, Option<&'static str>)> {
    let w = words
        .trim()
        .trim_end_matches(['?', '!', '.'])
        .trim()
        .to_lowercase();
    let w = w.strip_prefix("on ").unwrap_or(&w).trim();
    if w.is_empty() || w.len() > 40 {
        return None;
    }
    let bare = w.strip_prefix("the ").unwrap_or(w);
    for (names, day, name) in NAMED {
        if names.contains(&w) || names.contains(&bare) {
            return Some((*day, Some(*name)));
        }
    }
    // a day of the week: "friday", "next friday", "this friday"
    let wd = bare
        .strip_prefix("next ")
        .or_else(|| bare.strip_prefix("this "))
        .or_else(|| bare.strip_prefix("coming "))
        .unwrap_or(bare);
    if let Some(d) = weekday_of(wd) {
        return Some((Day::Weekday(d), None));
    }
    // 2026-12-25
    if let [y, m, d] = bare.split('-').collect::<Vec<_>>()[..]
        && let (Some(y), Ok(m), Ok(d)) = (year_number(y), m.parse::<u32>(), d.parse::<u32>())
    {
        return valid(y, m, d).then_some((Day::On(y, m, d), None));
    }
    // 25.12.2026, 25.12.
    let dotted: Vec<&str> = bare.trim_end_matches('.').split('.').collect();
    if dotted.len() >= 2
        && dotted
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
    {
        let (d, m): (u32, u32) = (dotted[0].parse().ok()?, dotted[1].parse().ok()?);
        return match dotted.get(2) {
            None => valid(2000, m, d).then_some((Day::Yearly(m, d), None)),
            Some(y) => {
                let y = year_number(y)?;
                valid(y, m, d).then_some((Day::On(y, m, d), None))
            }
        };
    }
    // "25 december", "25th of december 2027", "december 25, 2027", "dec 25th"
    let toks: Vec<&str> = bare
        .split([' ', ','])
        .filter(|t| !t.is_empty() && *t != "of" && *t != "the")
        .collect();
    let (d, m, y) = match toks[..] {
        [a, b] => match (day_number(a), month_of(a)) {
            (Some(d), _) => (d, month_of(b)?, None),
            (None, Some(m)) => (day_number(b)?, m, None),
            _ => return None,
        },
        [a, b, c] => match (day_number(a), month_of(a)) {
            (Some(d), _) => (d, month_of(b)?, Some(year_number(c)?)),
            (None, Some(m)) => (day_number(b)?, m, Some(year_number(c)?)),
            _ => return None,
        },
        _ => return None,
    };
    match y {
        Some(y) => valid(y, m, d).then_some((Day::On(y, m, d), None)),
        // 29 February is a date every leap year
        None => valid(2000, m, d).then_some((Day::Yearly(m, d), None)),
    }
}

/// This year's day for a yearly date, when the year has it.
fn in_year(day: Day, y: i64) -> Option<i64> {
    let (m, d) = match day {
        Day::Yearly(m, d) => (m, d),
        Day::Easter => easter(y),
        _ => return None,
    };
    valid(y, m, d).then(|| days_from_civil(y, m, d))
}

/// The day the words mean, looking ahead: the next yearly date on or after
/// today, the next weekday after today, a dated day as it is.
pub fn next(day: Day, today: i64) -> i64 {
    match day {
        Day::On(y, m, d) => days_from_civil(y, m, d),
        Day::Offset(n) => today + n,
        Day::Weekday(w) => {
            let ahead = (w as i64 - weekday(today) as i64).rem_euclid(7);
            today + if ahead == 0 { 7 } else { ahead }
        }
        Day::Yearly(..) | Day::Easter => {
            let (y, ..) = civil_from_days(today);
            (y..y + 9)
                .filter_map(|y| in_year(day, y))
                .find(|t| *t >= today)
                .unwrap_or(today)
        }
    }
}

/// The day the words mean, looking back: the last yearly date on or before
/// today, the last weekday before today.
pub fn last(day: Day, today: i64) -> i64 {
    match day {
        Day::Weekday(w) => {
            let back = (weekday(today) as i64 - w as i64).rem_euclid(7);
            today - if back == 0 { 7 } else { back }
        }
        Day::Yearly(..) | Day::Easter => {
            let (y, ..) = civil_from_days(today);
            (y - 8..=y)
                .rev()
                .filter_map(|y| in_year(day, y))
                .find(|t| *t <= today)
                .unwrap_or(today)
        }
        _ => next(day, today),
    }
}

fn plural(n: i64, one: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {one}s")
    }
}

/// "47 days", and " (6 weeks 5 days)" from two weeks up.
fn days_and_weeks(n: i64) -> (String, String) {
    let n = n.abs();
    let weeks = match (n / 7, n % 7) {
        _ if n < 14 => String::new(),
        (w, 0) => format!(" ({})", plural(w, "week")),
        (w, d) => format!(" ({} {})", plural(w, "week"), plural(d, "day")),
    };
    (plural(n, "day"), weeks)
}

/// What date_math answers, given today (a day number in local time):
///
/// - `until`: "Christmas is in 47 days (6 weeks 5 days): Friday 25 December
///   2026";
/// - `since`: "New Year's Day was 311 days ago: Thursday 1 January 2026";
/// - `weekday`: "Christmas 2026 is on a Friday (25 December 2026)";
/// - `offset`: "In 30 days it's Tuesday 8 December 2026".
///
/// None when the words name no day, or the op is not one of these.
pub fn answer(op: &str, date: Option<&str>, days: Option<i64>, today: i64) -> Option<String> {
    if op == "offset" {
        let n = days?;
        let t = today + n;
        return Some(match n {
            0 => format!("Today is {}", long(t)),
            1 => format!("Tomorrow is {}", long(t)),
            -1 => format!("Yesterday was {}", long(t)),
            n if n > 0 && n % 7 == 0 => format!("In {} it's {}", plural(n / 7, "week"), long(t)),
            n if n > 0 => format!("In {} it's {}", plural(n, "day"), long(t)),
            n if n % 7 == 0 => format!("{} ago it was {}", plural(-n / 7, "week"), long(t)),
            n => format!("{} ago it was {}", plural(-n, "day"), long(t)),
        });
    }
    let (day, name) = parse(date?)?;
    let t = match op {
        "since" => last(day, today),
        "until" | "weekday" => next(day, today),
        _ => return None,
    };
    let n = t - today;
    let label = name.map(str::to_string).unwrap_or_else(|| long(t));
    let named = name.is_some();
    Some(match op {
        "weekday" => {
            let (y, m, d) = civil_from_days(t);
            let date = format!("{d} {} {y}", MONTHS[(m - 1) as usize]);
            let wd = WEEKDAYS[weekday(t)];
            let verb = if n < 0 { "was" } else { "is" };
            match name {
                Some("Today" | "Tomorrow" | "Yesterday") => format!("{label} {verb} {wd}, {date}"),
                Some(nm) => format!("{nm} {y} {verb} on a {wd} ({date})"),
                None => format!("{date} {verb} a {wd}"),
            }
        }
        _ => {
            let (days, weeks) = days_and_weeks(n);
            let when = if named {
                format!(": {}", long(t))
            } else {
                String::new()
            };
            match n {
                0 => format!("{label} is today{when}"),
                1 => format!("{label} is tomorrow{when}"),
                -1 => format!("{label} was yesterday{when}"),
                n if n > 0 => format!("{label} is in {days}{weeks}{when}"),
                _ => format!("{label} was {days} ago{weeks}{when}"),
            }
        }
    })
}

// ── time zones ──────────────────────────────────────────────────────────────

/// Places people name that are not a zone's city: countries, regions,
/// other cities, abbreviations.
const ZONE_ALIASES: &[(&[&str], &str)] = &[
    (
        &[
            "utc",
            "gmt",
            "zulu",
            "coordinated universal time",
            "greenwich",
        ],
        "UTC",
    ),
    (
        &["beijing", "china", "shenzhen", "guangzhou"],
        "Asia/Shanghai",
    ),
    (
        &[
            "delhi",
            "new delhi",
            "mumbai",
            "bombay",
            "india",
            "bangalore",
            "bengaluru",
            "calcutta",
            "chennai",
            "hyderabad",
        ],
        "Asia/Kolkata",
    ),
    (
        &[
            "san francisco",
            "sf",
            "la",
            "los angeles",
            "seattle",
            "california",
            "silicon valley",
            "las vegas",
            "portland",
            "pacific",
            "pacific time",
            "pst",
            "pdt",
        ],
        "America/Los_Angeles",
    ),
    (
        &[
            "new york",
            "new york city",
            "nyc",
            "ny",
            "boston",
            "washington",
            "washington dc",
            "dc",
            "miami",
            "atlanta",
            "philadelphia",
            "east coast",
            "eastern",
            "eastern time",
            "est",
            "edt",
        ],
        "America/New_York",
    ),
    (
        &[
            "texas",
            "dallas",
            "houston",
            "austin",
            "central time",
            "cst",
            "cdt",
        ],
        "America/Chicago",
    ),
    (
        &["mountain time", "mst", "mdt", "colorado"],
        "America/Denver",
    ),
    (
        &[
            "uk",
            "england",
            "britain",
            "great britain",
            "united kingdom",
            "scotland",
            "wales",
            "edinburgh",
            "manchester",
            "bst",
        ],
        "Europe/London",
    ),
    (&["japan", "osaka", "kyoto"], "Asia/Tokyo"),
    (&["korea", "south korea"], "Asia/Seoul"),
    (
        &["germany", "munich", "hamburg", "frankfurt", "cologne"],
        "Europe/Berlin",
    ),
    (&["france", "cet", "cest"], "Europe/Paris"),
    (&["spain", "barcelona"], "Europe/Madrid"),
    (&["italy", "milan", "florence", "venice"], "Europe/Rome"),
    (&["netherlands", "holland"], "Europe/Amsterdam"),
    (&["switzerland", "geneva", "basel"], "Europe/Zurich"),
    (&["poland", "krakow", "cracow"], "Europe/Warsaw"),
    (&["ireland"], "Europe/Dublin"),
    (&["portugal"], "Europe/Lisbon"),
    (&["greece"], "Europe/Athens"),
    (&["turkey", "türkiye", "ankara"], "Europe/Istanbul"),
    (&["ukraine", "kyiv", "kiev"], "Europe/Kyiv"),
    (
        &["russia", "st petersburg", "saint petersburg"],
        "Europe/Moscow",
    ),
    (&["armenia"], "Asia/Yerevan"),
    (&["azerbaijan"], "Asia/Baku"),
    (&["israel", "tel aviv"], "Asia/Jerusalem"),
    (&["egypt"], "Africa/Cairo"),
    (&["uae", "emirates", "abu dhabi"], "Asia/Dubai"),
    (&["thailand"], "Asia/Bangkok"),
    (
        &["vietnam", "hanoi", "saigon", "ho chi minh city"],
        "Asia/Ho_Chi_Minh",
    ),
    (&["indonesia"], "Asia/Jakarta"),
    (&["philippines"], "Asia/Manila"),
    (&["pakistan"], "Asia/Karachi"),
    (
        &["canada", "ontario", "ottawa", "montreal"],
        "America/Toronto",
    ),
    (&["mexico"], "America/Mexico_City"),
    (
        &["brazil", "rio", "rio de janeiro", "são paulo"],
        "America/Sao_Paulo",
    ),
    (&["argentina"], "America/Argentina/Buenos_Aires"),
    (&["australia", "canberra"], "Australia/Sydney"),
    (&["new zealand", "nz"], "Pacific/Auckland"),
    (&["hawaii"], "Pacific/Honolulu"),
    (&["alaska"], "America/Anchorage"),
];

/// The zone a place means, from the zones this computer has
/// (`/usr/share/zoneinfo`'s names, e.g. "Asia/Tokyo"): a name people use
/// (`ZONE_ALIASES`), else a zone whose city is the place ("tokyo", "new
/// york", "los angeles"), else the zone itself ("europe/berlin").
pub fn zone_for(place: &str, zones: &[String]) -> Option<String> {
    let p = place
        .trim()
        .trim_end_matches(['?', '.', '!'])
        .trim()
        .to_lowercase();
    let p = p.strip_prefix("the ").unwrap_or(&p).trim();
    if p.is_empty() || p.len() > 64 {
        return None;
    }
    let has = |z: &str| zones.iter().any(|x| x == z);
    for (names, zone) in ZONE_ALIASES {
        if names.contains(&p) {
            // an older tzdata calls Kyiv Kiev
            if has(zone) {
                return Some(zone.to_string());
            }
            if *zone == "Europe/Kyiv" && has("Europe/Kiev") {
                return Some("Europe/Kiev".into());
            }
        }
    }
    let city = p.replace(' ', "_");
    let mut best: Option<&String> = None;
    for z in zones {
        let lz = z.to_lowercase();
        let last = lz.rsplit('/').next().unwrap_or(&lz);
        if (last == city && z.contains('/')) || lz == city {
            // the shortest name: America/Indianapolis over America/Indiana/Indianapolis
            if best.is_none_or(|b| z.len() < b.len()) {
                best = Some(z);
            }
        }
    }
    best.cloned()
}

/// How a zone's city reads: "Asia/Ho_Chi_Minh" → "Ho Chi Minh".
pub fn zone_city(zone: &str) -> String {
    zone.rsplit('/').next().unwrap_or(zone).replace('_', " ")
}

/// "9 hours ahead of you", "3 hours 30 minutes behind you", "the same time
/// as here", from both UTC offsets in minutes.
pub fn offset_words(there_min: i64, here_min: i64) -> String {
    let diff = there_min - here_min;
    if diff == 0 {
        return "the same time as here".into();
    }
    let a = diff.abs();
    let (h, m) = (a / 60, a % 60);
    let span = match (h, m) {
        (0, m) => plural(m, "minute"),
        (h, 0) => plural(h, "hour"),
        (h, m) => format!("{} {}", plural(h, "hour"), plural(m, "minute")),
    };
    format!(
        "{span} {} you",
        if diff > 0 { "ahead of" } else { "behind" }
    )
}

/// A `date +%z` offset ("+0900", "-0330") in minutes.
pub fn utc_offset_min(z: &str) -> Option<i64> {
    let z = z.trim();
    let (sign, rest) = match z.as_bytes().first()? {
        b'+' => (1, &z[1..]),
        b'-' => (-1, &z[1..]),
        _ => return None,
    };
    if rest.len() != 4 || !rest.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let h: i64 = rest[..2].parse().ok()?;
    let m: i64 = rest[2..].parse().ok()?;
    Some(sign * (h * 60 + m))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Sunday 8 November 2026
    fn today() -> i64 {
        days_from_civil(2026, 11, 8)
    }

    #[test]
    fn civil_dates_round_trip() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(weekday(0), 3, "a Thursday");
        assert_eq!(long(today()), "Sunday 8 November 2026");
        for z in [-800_000, -1, 59, 10_957, 20_765, 2_932_896] {
            let (y, m, d) = civil_from_days(z);
            assert_eq!(days_from_civil(y, m, d), z);
        }
        assert_eq!(
            long(days_from_civil(2028, 2, 29)),
            "Tuesday 29 February 2028"
        );
        assert_eq!(easter(2026), (4, 5));
        assert_eq!(easter(2027), (3, 28));
        assert!(!valid(2026, 2, 29) && valid(2028, 2, 29));
    }

    #[test]
    fn words_name_days() {
        assert_eq!(
            parse("christmas"),
            Some((Day::Yearly(12, 25), Some("Christmas")))
        );
        assert_eq!(
            parse("New Year"),
            Some((Day::Yearly(1, 1), Some("New Year's Day")))
        );
        assert_eq!(parse("next friday"), Some((Day::Weekday(4), None)));
        assert_eq!(parse("25 december"), Some((Day::Yearly(12, 25), None)));
        assert_eq!(
            parse("the 25th of december"),
            Some((Day::Yearly(12, 25), None))
        );
        assert_eq!(
            parse("december 25th, 2027"),
            Some((Day::On(2027, 12, 25), None))
        );
        assert_eq!(parse("dec 25"), Some((Day::Yearly(12, 25), None)));
        assert_eq!(parse("2026-12-25"), Some((Day::On(2026, 12, 25), None)));
        assert_eq!(parse("25.12.2026"), Some((Day::On(2026, 12, 25), None)));
        assert_eq!(parse("29 february"), Some((Day::Yearly(2, 29), None)));
        assert_eq!(parse("tomorrow"), Some((Day::Offset(1), Some("Tomorrow"))));
        for w in [
            "it",
            "my birthday",
            "25/12",
            "31 february",
            "2026-02-30",
            "the weekend",
            "",
            "march",
        ] {
            assert_eq!(parse(w), None, "{w:?}");
        }
    }

    #[test]
    fn date_math_answers() {
        let a = |op: &str, d: &str| answer(op, Some(d), None, today()).unwrap();
        assert_eq!(
            a("until", "christmas"),
            "Christmas is in 47 days (6 weeks 5 days): Friday 25 December 2026"
        );
        assert_eq!(
            a("until", "new year"),
            "New Year's Day is in 54 days (7 weeks 5 days): Friday 1 January 2027"
        );
        assert_eq!(
            a("until", "25 december"),
            "Friday 25 December 2026 is in 47 days (6 weeks 5 days)"
        );
        assert_eq!(a("until", "friday"), "Friday 13 November 2026 is in 5 days");
        assert_eq!(a("until", "sunday"), "Sunday 15 November 2026 is in 7 days");
        assert_eq!(
            a("until", "halloween"),
            "Halloween is in 357 days (51 weeks): Sunday 31 October 2027"
        );
        assert_eq!(
            a("until", "2026-11-01"),
            "Sunday 1 November 2026 was 7 days ago"
        );
        assert_eq!(
            a("since", "new year"),
            "New Year's Day was 311 days ago (44 weeks 3 days): Thursday 1 January 2026"
        );
        assert_eq!(a("weekday", "25 december"), "25 December 2026 is a Friday");
        assert_eq!(
            a("weekday", "christmas"),
            "Christmas 2026 is on a Friday (25 December 2026)"
        );
        assert_eq!(
            a("weekday", "2025-12-25"),
            "25 December 2025 was a Thursday"
        );
        assert_eq!(
            a("weekday", "tomorrow"),
            "Tomorrow is Monday, 9 November 2026"
        );
        let o = |n: i64| answer("offset", None, Some(n), today()).unwrap();
        assert_eq!(o(30), "In 30 days it's Tuesday 8 December 2026");
        assert_eq!(o(14), "In 2 weeks it's Sunday 22 November 2026");
        assert_eq!(o(1), "Tomorrow is Monday 9 November 2026");
        assert_eq!(o(-10), "10 days ago it was Thursday 29 October 2026");
        assert_eq!(answer("until", Some("my birthday"), None, today()), None);
        assert_eq!(answer("explode", Some("christmas"), None, today()), None);
        // christmas day itself
        let xmas = days_from_civil(2026, 12, 25);
        assert_eq!(
            answer("until", Some("christmas"), None, xmas).unwrap(),
            "Christmas is today: Friday 25 December 2026"
        );
    }

    #[test]
    fn places_find_their_zone() {
        let zones: Vec<String> = [
            "Asia/Tokyo",
            "Asia/Shanghai",
            "Asia/Kolkata",
            "America/New_York",
            "America/Los_Angeles",
            "America/Indiana/Indianapolis",
            "America/Indianapolis",
            "Europe/London",
            "Europe/Kiev",
            "Asia/Tbilisi",
            "Asia/Ho_Chi_Minh",
            "UTC",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let z = |p: &str| zone_for(p, &zones);
        assert_eq!(z("tokyo").as_deref(), Some("Asia/Tokyo"));
        assert_eq!(z("New York").as_deref(), Some("America/New_York"));
        assert_eq!(z("beijing").as_deref(), Some("Asia/Shanghai"));
        assert_eq!(z("mumbai").as_deref(), Some("Asia/Kolkata"));
        assert_eq!(z("san francisco").as_deref(), Some("America/Los_Angeles"));
        assert_eq!(z("uk").as_deref(), Some("Europe/London"));
        assert_eq!(z("utc").as_deref(), Some("UTC"));
        assert_eq!(z("indianapolis").as_deref(), Some("America/Indianapolis"));
        assert_eq!(z("kyiv").as_deref(), Some("Europe/Kiev"), "an older tzdata");
        assert_eq!(z("ho chi minh").as_deref(), Some("Asia/Ho_Chi_Minh"));
        assert_eq!(z("asia/tbilisi").as_deref(), Some("Asia/Tbilisi"));
        assert_eq!(z("narnia"), None);
        assert_eq!(z(""), None);
        assert_eq!(zone_city("Asia/Ho_Chi_Minh"), "Ho Chi Minh");
        assert_eq!(offset_words(540, 240), "5 hours ahead of you");
        assert_eq!(offset_words(-300, 240), "9 hours behind you");
        assert_eq!(offset_words(330, 240), "1 hour 30 minutes ahead of you");
        assert_eq!(offset_words(60, 60), "the same time as here");
        assert_eq!(utc_offset_min("+0900"), Some(540));
        assert_eq!(utc_offset_min("-0330"), Some(-210));
        assert_eq!(utc_offset_min("JST"), None);
    }
}
