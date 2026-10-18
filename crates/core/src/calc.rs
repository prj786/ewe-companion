//! The calculator: plain arithmetic and percentages, offline (C5).
//!
//! "what is 12*7", "15% of 80", "80 + 15%", "20% off 50", "(3 + 4) ^ 2",
//! "12 times 7", "100 divided by 8". Numbers and operators only: anything
//! else (a word it does not know, a unit, a function) is not a sum, and
//! the words go on to the rest of the chat.
//!
//! Percentages read the way a pocket calculator reads them: `a + b%` is
//! `a` plus b percent of `a`, `a × b%` is `a × b/100`, `b% of a` is
//! b percent of `a`, `b% off a` takes b percent away from `a`.

/// One piece of a sum.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Tok {
    Num(f64),
    Op(char),
    Open,
    Close,
    Pct,
    Of,
    Off,
}

/// The words people say for an operator.
const WORDS: &[(&str, Tok)] = &[
    ("multiplied by", Tok::Op('*')),
    ("divided by", Tok::Op('/')),
    ("to the power of", Tok::Op('^')),
    ("percent", Tok::Pct),
    ("plus", Tok::Op('+')),
    ("minus", Tok::Op('-')),
    ("times", Tok::Op('*')),
    ("over", Tok::Op('/')),
    ("of", Tok::Of),
    ("off", Tok::Off),
    ("x", Tok::Op('*')),
];

fn lex(s: &str) -> Option<Vec<Tok>> {
    let s = s.trim().to_lowercase();
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    'outer: while i < b.len() {
        let c = b[i] as char;
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c.is_ascii_digit() || (c == '.' && b.get(i + 1).is_some_and(u8::is_ascii_digit)) {
            let start = i;
            while i < b.len() {
                let d = b[i] as char;
                // "1,000": a comma followed by exactly three digits
                let thousands = d == ','
                    && b.len() >= i + 4
                    && b[i + 1..i + 4].iter().all(u8::is_ascii_digit)
                    && b.get(i + 4).is_none_or(|n| !n.is_ascii_digit());
                if d.is_ascii_digit() || d == '.' || thousands {
                    i += 1;
                } else {
                    break;
                }
            }
            let n: f64 = s[start..i].replace(',', "").parse().ok()?;
            out.push(Tok::Num(n));
            continue;
        }
        let t = match c {
            '+' | '-' | '*' | '/' | '^' => Some(Tok::Op(c)),
            '(' => Some(Tok::Open),
            ')' => Some(Tok::Close),
            '%' => Some(Tok::Pct),
            _ => None,
        };
        if let Some(t) = t {
            out.push(t);
            i += 1;
            continue;
        }
        // ×, ÷ and the operator words
        let rest = &s[i..];
        for (sym, t) in [
            ("×", Tok::Op('*')),
            ("÷", Tok::Op('/')),
            ("−", Tok::Op('-')),
        ] {
            if let Some(r) = rest.strip_prefix(sym) {
                out.push(t);
                i = s.len() - r.len();
                continue 'outer;
            }
        }
        for (w, t) in WORDS {
            if let Some(r) = rest.strip_prefix(w)
                && r.chars().next().is_none_or(|n| !n.is_alphabetic())
            {
                out.push(*t);
                i = s.len() - r.len();
                continue 'outer;
            }
        }
        return None;
    }
    Some(out)
}

/// A value, and whether it is a bare percentage (`15%`), which `+` and
/// `-` read relative to the left side.
#[derive(Clone, Copy)]
struct Val {
    v: f64,
    pct: bool,
}

struct Parser {
    toks: Vec<Tok>,
    at: usize,
    /// how deep the recursion is: parentheses, signs and powers each go a
    /// level down, and a sum nested past [`DEPTH`] is not one
    depth: usize,
}

/// The longest sum, in bytes (the same cap as `validate`'s).
pub const MAX_LEN: usize = 200;
/// The deepest nesting the parser follows.
const DEPTH: usize = 64;

impl Parser {
    fn peek(&self) -> Option<Tok> {
        self.toks.get(self.at).copied()
    }
    fn next(&mut self) -> Option<Tok> {
        let t = self.peek();
        self.at += 1;
        t
    }

    // sum := term (('+'|'-') term)*
    fn sum(&mut self) -> Option<Val> {
        let mut l = self.term()?;
        while let Some(Tok::Op(op @ ('+' | '-'))) = self.peek() {
            self.next();
            let r = self.term()?;
            let rv = if r.pct { l.v * r.v } else { r.v };
            l = Val {
                v: if op == '+' { l.v + rv } else { l.v - rv },
                pct: false,
            };
        }
        Some(l)
    }

    // term := unary (('*'|'/') unary)*
    fn term(&mut self) -> Option<Val> {
        let mut l = self.unary()?;
        while let Some(Tok::Op(op @ ('*' | '/'))) = self.peek() {
            self.next();
            let r = self.unary()?;
            l = Val {
                v: if op == '*' {
                    l.v * r.v
                } else {
                    if r.v == 0.0 {
                        return None;
                    }
                    l.v / r.v
                },
                pct: false,
            };
        }
        Some(l)
    }

    // unary := '-' unary | '+' unary | power. A sign binds looser than a
    // power: -2^2 is -(2^2) = -4, as on paper
    fn unary(&mut self) -> Option<Val> {
        self.depth += 1;
        if self.depth > DEPTH {
            return None;
        }
        let v = match self.peek() {
            Some(Tok::Op('-')) => {
                self.next();
                self.unary().map(|v| Val { v: -v.v, ..v })
            }
            Some(Tok::Op('+')) => {
                self.next();
                self.unary()
            }
            _ => self.power(),
        };
        self.depth -= 1;
        v
    }

    // power := percent ('^' unary)?   (right-associative: 2^3^2 = 2^9)
    fn power(&mut self) -> Option<Val> {
        let base = self.percent()?;
        if let Some(Tok::Op('^')) = self.peek() {
            self.next();
            let e = self.unary()?;
            return Some(Val {
                v: base.v.powf(e.v),
                pct: false,
            });
        }
        Some(base)
    }

    // percent := atom ('%' (('of'|'off') unary)?)?
    fn percent(&mut self) -> Option<Val> {
        let a = self.atom()?;
        if self.peek() != Some(Tok::Pct) {
            return Some(a);
        }
        self.next();
        let p = a.v / 100.0;
        match self.peek() {
            Some(Tok::Of) => {
                self.next();
                let of = self.unary()?;
                Some(Val {
                    v: p * of.v,
                    pct: false,
                })
            }
            Some(Tok::Off) => {
                self.next();
                let of = self.unary()?;
                Some(Val {
                    v: of.v - p * of.v,
                    pct: false,
                })
            }
            _ => Some(Val { v: p, pct: true }),
        }
    }

    // atom := number | '(' sum ')'
    fn atom(&mut self) -> Option<Val> {
        match self.next()? {
            Tok::Num(n) => Some(Val { v: n, pct: false }),
            Tok::Open => {
                let v = self.sum()?;
                (self.next()? == Tok::Close).then_some(Val { pct: false, ..v })
            }
            _ => None,
        }
    }
}

/// The answer to a sum, or None when the words are not one (or it has no
/// answer: a division by zero, a number too big), or it is longer than
/// [`MAX_LEN`] or nested deeper than the parser follows: the matcher tries
/// every chat line here first, so nothing here may cost much.
pub fn eval(expr: &str) -> Option<f64> {
    if expr.len() > MAX_LEN {
        return None;
    }
    let toks = lex(expr)?;
    // a sum has an operator past its first piece, or a percent: "42",
    // "-5" or "(7)" alone are not questions for the calculator
    if !toks.iter().enumerate().any(|(i, t)| {
        matches!(t, Tok::Pct | Tok::Of | Tok::Off) || (i > 0 && matches!(t, Tok::Op(_)))
    }) {
        return None;
    }
    let mut p = Parser {
        toks,
        at: 0,
        depth: 0,
    };
    let v = p.sum()?;
    (p.at == p.toks.len() && v.v.is_finite()).then_some(v.v)
}

/// A bare-integer subtraction with no spaces ("555-1234", "2026-10-18",
/// "1-2") reads as a phone number or a date, not a sum: the matcher takes
/// it only after "what is" or "calculate", or written "12 - 3".
pub fn looks_like_a_number(expr: &str) -> bool {
    let b = expr.as_bytes();
    b.windows(3)
        .any(|w| w[0].is_ascii_digit() && w[1] == b'-' && w[2].is_ascii_digit())
}

/// A result as people write it: `84`, `12.5`, `0.3333333333`, `1.5e+21`.
pub fn show(v: f64) -> String {
    if v == 0.0 {
        return "0".into();
    }
    if v.abs() >= 1e15 || v.abs() < 1e-9 {
        return format!("{v:e}");
    }
    let s = format!("{v:.10}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is(e: &str, want: &str) {
        assert_eq!(eval(e).map(show).as_deref(), Some(want), "{e:?}");
    }

    #[test]
    fn arithmetic() {
        is("2+2", "4");
        is("12*7", "84");
        is("12 x 7", "84");
        is("12 × 7", "84");
        is("100 / 8", "12.5");
        is("100 divided by 8", "12.5");
        is("12 times 7 plus 1", "85");
        is("(3 + 4) ^ 2", "49");
        is("2^3^2", "512");
        is("-5 + 3", "-2");
        is("1,000 * 3", "3000");
        is("1/3", "0.3333333333");
        is("0.1 + 0.2", "0.3");
        // a sign binds looser than a power, as on paper
        is("-2^2", "-4");
        is("(-2)^2", "4");
        is("2^-1", "0.5");
        is("3 * -2", "-6");
    }

    #[test]
    fn no_sum_costs_much() {
        // nesting deeper than the parser follows, and long lines: None,
        // never a stack overflow (the daemon runs with panic = abort)
        let deep = format!("{}1{}", "(".repeat(5000), ")".repeat(5000));
        assert_eq!(eval(&deep), None);
        assert_eq!(eval(&"-".repeat(5000)), None);
        assert_eq!(eval(&format!("{}1+1", "-".repeat(150))), None);
        let long = "1+".repeat(200) + "1";
        assert!(long.len() > MAX_LEN);
        assert_eq!(eval(&long), None);
        // within the caps, still fine
        let ok = format!("{}1+1{}", "(".repeat(30), ")".repeat(30));
        assert_eq!(eval(&ok), Some(2.0));
    }

    #[test]
    fn dashed_numbers_look_like_numbers() {
        assert!(looks_like_a_number("555-1234"));
        assert!(looks_like_a_number("2026-10-18"));
        assert!(!looks_like_a_number("12 - 3"));
        assert!(!looks_like_a_number("-5 + 3"));
    }

    #[test]
    fn percentages() {
        is("15% of 80", "12");
        is("15 percent of 80", "12");
        is("80 + 15%", "92");
        is("80 - 15%", "68");
        is("20% off 50", "40");
        is("200 * 5%", "10");
        is("50%", "0.5");
    }

    #[test]
    fn what_is_not_a_sum() {
        for e in [
            "",
            "42",
            "ewe",
            "2 apples + 3",
            "1/0",
            "5 +",
            "(2",
            "sqrt 4",
            "the volume",
            "10 minutes",
            "1,00 + 2",
            "-5",
            "(7)",
        ] {
            assert_eq!(eval(e), None, "{e:?}");
        }
    }
}
