//! Units: "convert 10 km to miles", "20 c in f", "5 gb in mb", "how many
//! cm in 3 feet", offline (C5).
//!
//! Length, mass, temperature, volume (US cups, pints and gallons), speed,
//! data, time and area. Data counts the way disks and file managers do:
//! 1 GB = 1000 MB; GiB, MiB and KiB count in 1024s. A year is 365 days.
//! Currencies are not units here: their rates need the network, so "100
//! usd in eur" is not a conversion, and the words go on to the rest of the
//! chat.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dim {
    Length,
    Mass,
    Temperature,
    Volume,
    Speed,
    Data,
    Time,
    Area,
}

/// One unit: `base = value * factor + offset` in its dimension's base unit
/// (metre, kilogram, degree Celsius, litre, metre per second, byte, second,
/// square metre).
#[derive(Debug)]
pub struct Unit {
    /// what a call carries (`"km"`, `"degF"`, `"gib"`)
    pub id: &'static str,
    pub dim: Dim,
    pub factor: f64,
    pub offset: f64,
    /// how a reply writes it after a number: "km", "°F", "cups"
    pub short: &'static str,
    /// the words people use for it (lower case)
    pub words: &'static [&'static str],
}

const fn u(
    id: &'static str,
    dim: Dim,
    factor: f64,
    short: &'static str,
    words: &'static [&'static str],
) -> Unit {
    Unit {
        id,
        dim,
        factor,
        offset: 0.0,
        short,
        words,
    }
}

use Dim::*;

pub const UNITS: &[Unit] = &[
    // length (metre)
    u(
        "mm",
        Length,
        0.001,
        "mm",
        &[
            "mm",
            "millimeter",
            "millimeters",
            "millimetre",
            "millimetres",
        ],
    ),
    u(
        "cm",
        Length,
        0.01,
        "cm",
        &[
            "cm",
            "centimeter",
            "centimeters",
            "centimetre",
            "centimetres",
        ],
    ),
    u(
        "m",
        Length,
        1.0,
        "m",
        &["m", "meter", "meters", "metre", "metres"],
    ),
    u(
        "km",
        Length,
        1000.0,
        "km",
        &[
            "km",
            "kms",
            "kilometer",
            "kilometers",
            "kilometre",
            "kilometres",
        ],
    ),
    u("in", Length, 0.0254, "in", &["inch", "inches", "\""]),
    u("ft", Length, 0.3048, "ft", &["ft", "foot", "feet", "'"]),
    u("yd", Length, 0.9144, "yd", &["yd", "yds", "yard", "yards"]),
    u("mi", Length, 1609.344, "miles", &["mi", "mile", "miles"]),
    u(
        "nmi",
        Length,
        1852.0,
        "nautical miles",
        &["nmi", "nautical mile", "nautical miles"],
    ),
    // mass (kilogram)
    u("mg", Mass, 1e-6, "mg", &["mg", "milligram", "milligrams"]),
    u(
        "g",
        Mass,
        0.001,
        "g",
        &["g", "gram", "grams", "gramme", "grammes"],
    ),
    u(
        "kg",
        Mass,
        1.0,
        "kg",
        &[
            "kg",
            "kgs",
            "kilo",
            "kilos",
            "kilogram",
            "kilograms",
            "kilogramme",
            "kilogrammes",
        ],
    ),
    u(
        "t",
        Mass,
        1000.0,
        "tonnes",
        &["tonne", "tonnes", "metric ton", "metric tons"],
    ),
    u(
        "oz",
        Mass,
        0.028_349_523_125,
        "oz",
        &["oz", "ounce", "ounces"],
    ),
    u(
        "lb",
        Mass,
        0.453_592_37,
        "lb",
        &["lb", "lbs", "pound", "pounds"],
    ),
    u(
        "st",
        Mass,
        6.350_293_18,
        "stone",
        &["st", "stone", "stones"],
    ),
    // temperature (degree Celsius)
    Unit {
        id: "degC",
        dim: Temperature,
        factor: 1.0,
        offset: 0.0,
        short: "°C",
        words: &[
            "c",
            "°c",
            "ºc",
            "celsius",
            "centigrade",
            "degrees c",
            "degree c",
            "deg c",
            "degrees celsius",
            "degree celsius",
        ],
    },
    Unit {
        id: "degF",
        dim: Temperature,
        factor: 5.0 / 9.0,
        offset: -160.0 / 9.0,
        short: "°F",
        words: &[
            "f",
            "°f",
            "ºf",
            "fahrenheit",
            "degrees f",
            "degree f",
            "deg f",
            "degrees fahrenheit",
            "degree fahrenheit",
        ],
    },
    Unit {
        id: "K",
        dim: Temperature,
        factor: 1.0,
        offset: -273.15,
        short: "K",
        words: &["kelvin", "kelvins", "degrees kelvin"],
    },
    // volume (litre); cups, pints, quarts and gallons are the US ones
    u(
        "ml",
        Volume,
        0.001,
        "ml",
        &[
            "ml",
            "mls",
            "milliliter",
            "milliliters",
            "millilitre",
            "millilitres",
        ],
    ),
    u(
        "cl",
        Volume,
        0.01,
        "cl",
        &[
            "cl",
            "centiliter",
            "centiliters",
            "centilitre",
            "centilitres",
        ],
    ),
    u(
        "dl",
        Volume,
        0.1,
        "dl",
        &["dl", "deciliter", "deciliters", "decilitre", "decilitres"],
    ),
    u(
        "l",
        Volume,
        1.0,
        "l",
        &["l", "ltr", "liter", "liters", "litre", "litres"],
    ),
    u(
        "m3",
        Volume,
        1000.0,
        "m³",
        &[
            "m3",
            "m³",
            "cubic meter",
            "cubic meters",
            "cubic metre",
            "cubic metres",
        ],
    ),
    u(
        "tsp",
        Volume,
        0.004_928_921_593_75,
        "tsp",
        &["tsp", "teaspoon", "teaspoons"],
    ),
    u(
        "tbsp",
        Volume,
        0.014_786_764_781_25,
        "tbsp",
        &["tbsp", "tablespoon", "tablespoons"],
    ),
    u(
        "floz",
        Volume,
        0.029_573_529_562_5,
        "fl oz",
        &["fl oz", "floz", "fluid ounce", "fluid ounces"],
    ),
    u("cup", Volume, 0.236_588_236_5, "cups", &["cup", "cups"]),
    u(
        "pt",
        Volume,
        0.473_176_473,
        "pints",
        &["pt", "pint", "pints"],
    ),
    u(
        "qt",
        Volume,
        0.946_352_946,
        "quarts",
        &["qt", "quart", "quarts"],
    ),
    u(
        "gal",
        Volume,
        3.785_411_784,
        "gallons",
        &["gal", "gallon", "gallons"],
    ),
    // speed (metre per second)
    u(
        "m/s",
        Speed,
        1.0,
        "m/s",
        &[
            "m/s",
            "mps",
            "meters per second",
            "metres per second",
            "meter per second",
            "metre per second",
        ],
    ),
    u(
        "km/h",
        Speed,
        1.0 / 3.6,
        "km/h",
        &[
            "km/h",
            "kmh",
            "kph",
            "kmph",
            "km per hour",
            "kilometers per hour",
            "kilometres per hour",
            "kilometer per hour",
            "kilometre per hour",
        ],
    ),
    u(
        "mph",
        Speed,
        0.447_04,
        "mph",
        &["mph", "mi/h", "miles per hour", "mile per hour"],
    ),
    u(
        "kn",
        Speed,
        1852.0 / 3600.0,
        "knots",
        &["kn", "knot", "knots"],
    ),
    // data (byte): disks count in 1000s, the i-units in 1024s
    u("b", Data, 1.0, "bytes", &["b", "byte", "bytes"]),
    u("kb", Data, 1e3, "KB", &["kb", "kilobyte", "kilobytes"]),
    u(
        "mb",
        Data,
        1e6,
        "MB",
        &["mb", "megabyte", "megabytes", "meg", "megs"],
    ),
    u(
        "gb",
        Data,
        1e9,
        "GB",
        &["gb", "gig", "gigs", "gigabyte", "gigabytes"],
    ),
    u("tb", Data, 1e12, "TB", &["tb", "terabyte", "terabytes"]),
    u("pb", Data, 1e15, "PB", &["pb", "petabyte", "petabytes"]),
    u(
        "kib",
        Data,
        1024.0,
        "KiB",
        &["kib", "kibibyte", "kibibytes"],
    ),
    u(
        "mib",
        Data,
        1_048_576.0,
        "MiB",
        &["mib", "mebibyte", "mebibytes"],
    ),
    u(
        "gib",
        Data,
        1_073_741_824.0,
        "GiB",
        &["gib", "gibibyte", "gibibytes"],
    ),
    u(
        "tib",
        Data,
        1_099_511_627_776.0,
        "TiB",
        &["tib", "tebibyte", "tebibytes"],
    ),
    u("bit", Data, 0.125, "bits", &["bit", "bits"]),
    u(
        "mbit",
        Data,
        125_000.0,
        "Mbit",
        &["mbit", "mbits", "megabit", "megabits"],
    ),
    u(
        "gbit",
        Data,
        125_000_000.0,
        "Gbit",
        &["gbit", "gbits", "gigabit", "gigabits"],
    ),
    // time (second)
    u(
        "ms",
        Time,
        0.001,
        "ms",
        &["ms", "millisecond", "milliseconds"],
    ),
    u(
        "s",
        Time,
        1.0,
        "seconds",
        &["s", "sec", "secs", "second", "seconds"],
    ),
    u(
        "min",
        Time,
        60.0,
        "minutes",
        &["min", "mins", "minute", "minutes"],
    ),
    u(
        "h",
        Time,
        3600.0,
        "hours",
        &["h", "hr", "hrs", "hour", "hours"],
    ),
    u("day", Time, 86_400.0, "days", &["day", "days"]),
    u(
        "week",
        Time,
        604_800.0,
        "weeks",
        &["wk", "wks", "week", "weeks"],
    ),
    u(
        "year",
        Time,
        31_536_000.0,
        "years",
        &["yr", "yrs", "year", "years"],
    ),
    // area (square metre)
    u(
        "cm2",
        Area,
        1e-4,
        "cm²",
        &[
            "cm2",
            "cm²",
            "sq cm",
            "square centimeter",
            "square centimeters",
            "square centimetre",
            "square centimetres",
        ],
    ),
    u(
        "m2",
        Area,
        1.0,
        "m²",
        &[
            "m2",
            "m²",
            "sq m",
            "sqm",
            "square meter",
            "square meters",
            "square metre",
            "square metres",
        ],
    ),
    u(
        "km2",
        Area,
        1e6,
        "km²",
        &[
            "km2",
            "km²",
            "sq km",
            "square kilometer",
            "square kilometers",
            "square kilometre",
            "square kilometres",
        ],
    ),
    u("ha", Area, 1e4, "hectares", &["ha", "hectare", "hectares"]),
    u("acre", Area, 4_046.856_422_4, "acres", &["acre", "acres"]),
    u(
        "sqft",
        Area,
        0.092_903_04,
        "sq ft",
        &["sqft", "sq ft", "ft2", "ft²", "square foot", "square feet"],
    ),
    u(
        "sqmi",
        Area,
        2_589_988.110_336,
        "sq mi",
        &["sqmi", "sq mi", "mi2", "mi²", "square mile", "square miles"],
    ),
];

/// A unit by its id or by a word people use for it ("miles", "°c", "km/h").
pub fn find(word: &str) -> Option<&'static Unit> {
    let w = word.trim().trim_end_matches('.').to_lowercase();
    UNITS
        .iter()
        .find(|u| u.id.eq_ignore_ascii_case(&w) || u.words.contains(&w.as_str()))
}

/// `value` in `from`, as `to`. None when either is not a unit, or they
/// measure different things (km in kg).
pub fn convert(value: f64, from: &str, to: &str) -> Option<f64> {
    let (a, b) = (find(from)?, find(to)?);
    if a.dim != b.dim || !value.is_finite() {
        return None;
    }
    let base = value * a.factor + a.offset;
    let v = (base - b.offset) / b.factor;
    v.is_finite().then_some(v)
}

/// A number as a reply writes it: at most 1 decimal from 100 up, 2 from 1
/// up, 4 significant digits below; no trailing zeros.
pub fn show(v: f64) -> String {
    if v == 0.0 {
        return "0".into();
    }
    let a = v.abs();
    if !(1e-6..1e15).contains(&a) {
        return format!("{v:.3e}");
    }
    let decimals = if a >= 100.0 {
        1
    } else if a >= 1.0 {
        2
    } else {
        // 4 significant digits
        (3 - a.log10().floor() as i32).clamp(0, 9) as usize
    };
    let s = format!("{v:.decimals$}");
    let s = if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    };
    if s == "-0" { "0".into() } else { s }
}

fn with_unit(v: f64, u: &Unit) -> String {
    let n = show(v);
    match u.short {
        s if s.starts_with('°') => format!("{n} {s}"),
        // "1 cup", "1 hour": the singular for exactly one
        s if (v - 1.0).abs() < f64::EPSILON && s.ends_with('s') && s.len() > 3 => {
            format!("{n} {}", &s[..s.len() - 1])
        }
        s => format!("{n} {s}"),
    }
}

/// The reply's line: "10 km = 6.21 miles", with a note when data sizes
/// could be read either way.
pub fn line(value: f64, from: &str, to: &str) -> Option<String> {
    let (a, b) = (find(from)?, find(to)?);
    let v = convert(value, from, to)?;
    let mut s = format!("{} = {}", with_unit(value, a), with_unit(v, b));
    if a.dim == Dim::Data
        && a.id != b.id
        && !(a.id.ends_with("ib")
            || b.id.ends_with("ib")
            || a.id.contains("bit")
            || b.id.contains("bit"))
        && a.id != "b"
        && b.id != "b"
    {
        s.push_str(" (1 GB = 1000 MB, the way disks count; say GiB or MiB for 1024s)");
    }
    Some(s)
}

/// "10", "1.5", "-40", "1,000", "a", "an", "one", "half a".
fn number(w: &str) -> Option<f64> {
    match w {
        "a" | "an" | "one" => Some(1.0),
        "half a" | "half an" => Some(0.5),
        _ => {
            let n: f64 = w.replace(',', "").parse().ok()?;
            n.is_finite().then_some(n)
        }
    }
}

/// A number and the unit right after it: "10 km", "10km", "20°c", "-40 f",
/// "a mile". The unit is the rest of the words.
fn amount(s: &str) -> Option<(f64, &'static Unit)> {
    let s = s.trim();
    for pre in ["half a ", "half an ", "a ", "an ", "one "] {
        if let Some(rest) = s.strip_prefix(pre) {
            return Some((number(pre.trim())?, find(rest)?));
        }
    }
    // the number's own characters, then the unit (with or without a space)
    let end = s
        .char_indices()
        .find(|(i, c)| {
            !(c.is_ascii_digit() || *c == '.' || *c == ',' || (*i == 0 && (*c == '-' || *c == '+')))
        })
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    if end == 0 {
        return None;
    }
    let n = number(&s[..end])?;
    Some((n, find(&s[end..])?))
}

/// The words of a unit question, as (value, from id, to id):
///
/// - "convert 10 km to miles", "10 km in miles", "20°c to fahrenheit",
///   "what is 5 gb in mb", "how much is 70 kg in pounds";
/// - "how many cm in 3 feet", "how many seconds are in an hour", "how many
///   ml is 2 cups".
///
/// None for anything else, including two units that measure different
/// things and a currency ("100 usd in eur").
pub fn parse_question(text: &str) -> Option<(f64, &'static str, &'static str)> {
    let t = text.trim().trim_end_matches(['?', '.', '!']).trim();
    if t.len() > 120 {
        return None;
    }
    // "how many cm (are) in 3 feet"
    if let Some(rest) = t.strip_prefix("how many ") {
        for sep in [
            " are there in ",
            " are in ",
            " is in ",
            " in ",
            " is ",
            " are ",
            " make ",
            " makes ",
            " to ",
        ] {
            if let Some((to, from)) = rest.split_once(sep)
                && let Some(to) = find(to)
                && let Some((n, from)) = amount(from)
                && from.dim == to.dim
            {
                return Some((n, from.id, to.id));
            }
        }
        return None;
    }
    let mut t = t;
    for pre in [
        "convert ",
        "what is ",
        "what's ",
        "whats ",
        "how much is ",
        "how much are ",
    ] {
        if let Some(r) = t.strip_prefix(pre) {
            t = r.trim();
            break;
        }
    }
    // the first split where both sides read: "1 hour in seconds"
    for sep in [
        " in ",
        " to ",
        " into ",
        " as ",
        " = ",
        " is how many ",
        " is what in ",
        " equals how many ",
    ] {
        let mut from = 0;
        while let Some(i) = t[from..].find(sep) {
            let at = from + i;
            if let (Some((n, a)), Some(b)) = (amount(&t[..at]), find(&t[at + sep.len()..]))
                && a.dim == b.dim
            {
                return Some((n, a.id, b.id));
            }
            from = at + sep.len();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conv(t: &str) -> Option<String> {
        let (v, a, b) = parse_question(t)?;
        line(v, a, b)
    }

    #[test]
    fn everyday_conversions() {
        assert_eq!(
            conv("convert 10 km to miles").as_deref(),
            Some("10 km = 6.21 miles")
        );
        assert_eq!(
            conv("10 km in miles").as_deref(),
            Some("10 km = 6.21 miles")
        );
        assert_eq!(conv("20 c in f").as_deref(), Some("20 °C = 68 °F"));
        assert_eq!(conv("20°c to fahrenheit").as_deref(), Some("20 °C = 68 °F"));
        assert_eq!(conv("-40 f in c").as_deref(), Some("-40 °F = -40 °C"));
        assert_eq!(conv("0 c in kelvin").as_deref(), Some("0 °C = 273.1 K"));
        assert_eq!(conv("3 feet in cm").as_deref(), Some("3 ft = 91.44 cm"));
        assert_eq!(
            conv("100 km/h in mph").as_deref(),
            Some("100 km/h = 62.14 mph")
        );
        assert_eq!(
            conv("100kmh to mph").as_deref(),
            Some("100 km/h = 62.14 mph")
        );
        assert_eq!(conv("70 kg in pounds").as_deref(), Some("70 kg = 154.3 lb"));
        assert_eq!(conv("2 cups in ml").as_deref(), Some("2 cups = 473.2 ml"));
        assert_eq!(
            conv("1 hour in seconds").as_deref(),
            Some("1 hour = 3600 seconds")
        );
        assert_eq!(
            conv("an hour in minutes").as_deref(),
            Some("1 hour = 60 minutes")
        );
        assert_eq!(
            conv("what is 1.5 l in cups").as_deref(),
            Some("1.5 l = 6.34 cups")
        );
        assert_eq!(
            conv("how many cm in 3 feet").as_deref(),
            Some("3 ft = 91.44 cm")
        );
        assert_eq!(
            conv("how many seconds are in an hour").as_deref(),
            Some("1 hour = 3600 seconds")
        );
        assert_eq!(
            conv("how many ml is 2 cups").as_deref(),
            Some("2 cups = 473.2 ml")
        );
        assert_eq!(conv("1 mile in km").as_deref(), Some("1 mile = 1.61 km"));
        assert_eq!(conv("1 acre in m2").as_deref(), Some("1 acre = 4046.9 m²"));
        assert_eq!(conv("10 inches in cm").as_deref(), Some("10 in = 25.4 cm"));
    }

    #[test]
    fn data_counts_like_disks_unless_it_says_gib() {
        assert_eq!(
            conv("5 gb in mb").as_deref(),
            Some("5 GB = 5000 MB (1 GB = 1000 MB, the way disks count; say GiB or MiB for 1024s)")
        );
        assert_eq!(conv("5 gib in mib").as_deref(), Some("5 GiB = 5120 MiB"));
        assert_eq!(conv("1 tb in gib").as_deref(), Some("1 TB = 931.3 GiB"));
        assert_eq!(
            conv("100 megabits in mb").as_deref(),
            Some("100 Mbit = 12.5 MB")
        );
    }

    #[test]
    fn what_is_not_a_conversion() {
        for t in [
            "100 usd in eur",
            "100 pounds in euros",
            "convert 10 km to kg",
            "10 in 20",
            "set a timer for 5 minutes",
            "remind me in 5 minutes to stretch",
            "move this window to workspace 2",
            "what is 12*7",
            "open github.com in workspace 2",
            "what is the time in tokyo",
            "km to miles",
            "",
        ] {
            assert_eq!(parse_question(t), None, "{t:?}");
        }
    }

    #[test]
    fn numbers_read_cleanly() {
        assert_eq!(show(6.213_711_922), "6.21");
        assert_eq!(show(154.323_583_6), "154.3");
        assert_eq!(show(0.000_621_371), "0.0006214");
        assert_eq!(show(5000.0), "5000");
        assert_eq!(show(-40.0), "-40");
        assert_eq!(convert(1.0, "km", "kg"), None);
        assert_eq!(convert(1.0, "zz", "km"), None);
        assert!(find("Miles").is_some() && find("degrees celsius").is_some());
    }
}
