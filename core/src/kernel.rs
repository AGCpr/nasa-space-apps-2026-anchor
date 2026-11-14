//! Safety kernel A (ADR-0003). Written against the ADR specification only; kernel B
//! (TypeScript) and the Python reference are independent implementations of the same spec.
//!
//! Canonical table: `t1|<id>,<indicator>,<op>,<limit>,<class>,<consent>;...`
//! ids strictly increasing; op in {above, below} (strict); class in {phys, high};
//! consent in {0,1}, and phys lines must carry 1 (they always share).

use crate::log::{is_key, sha256_hex};

pub const MAX_LINES: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Above,
    Below,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Physiological,
    HighStakes,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub id: u16,
    pub indicator: String,
    pub op: Op,
    pub limit: i64,
    pub class: Class,
    pub auto_share_consented: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hit {
    pub id: u16,
    pub share: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SafetyTable {
    lines: Vec<Line>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KernelError {
    Grammar(&'static str),
}

fn parse_i64(s: &str) -> Result<i64, KernelError> {
    let digits = s.strip_prefix('-').unwrap_or(s);
    let ok = !digits.is_empty()
        && digits.bytes().all(|b| b.is_ascii_digit())
        && (digits == "0" || !digits.starts_with('0'))
        && s != "-0";
    if !ok {
        return Err(KernelError::Grammar("limit is not canonical decimal"));
    }
    s.parse().map_err(|_| KernelError::Grammar("limit out of i64 range"))
}

impl SafetyTable {
    pub fn parse(s: &str) -> Result<Self, KernelError> {
        let body = s.strip_prefix("t1|").ok_or(KernelError::Grammar("missing t1| header"))?;
        let mut lines: Vec<Line> = Vec::new();
        if !body.is_empty() {
            for item in body.split(';') {
                let f: Vec<&str> = item.split(',').collect();
                if f.len() != 6 {
                    return Err(KernelError::Grammar("line needs 6 fields"));
                }
                let id: u16 = {
                    let v = parse_i64(f[0])?;
                    u16::try_from(v).map_err(|_| KernelError::Grammar("id out of u16 range"))?
                };
                if lines.last().is_some_and(|p| p.id >= id) {
                    return Err(KernelError::Grammar("ids not strictly increasing"));
                }
                if !is_key(f[1]) {
                    return Err(KernelError::Grammar("invalid indicator key"));
                }
                let op = match f[2] {
                    "above" => Op::Above,
                    "below" => Op::Below,
                    _ => return Err(KernelError::Grammar("op must be above|below")),
                };
                let limit = parse_i64(f[3])?;
                let (class, consent) = match (f[4], f[5]) {
                    ("phys", "1") => (Class::Physiological, true),
                    ("high", "0") => (Class::HighStakes, false),
                    ("high", "1") => (Class::HighStakes, true),
                    _ => return Err(KernelError::Grammar("class/consent combination invalid")),
                };
                lines.push(Line { id, indicator: f[1].to_string(), op, limit, class, auto_share_consented: consent });
                if lines.len() > MAX_LINES {
                    return Err(KernelError::Grammar("more than 64 lines"));
                }
            }
        }
        Ok(SafetyTable { lines })
    }

    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    /// Canonical text; `parse(t.canonical()) == t`, and `canonical(parse(s)) == s` for valid `s`.
    pub fn canonical(&self) -> String {
        let items: Vec<String> = self
            .lines
            .iter()
            .map(|l| {
                let op = if l.op == Op::Above { "above" } else { "below" };
                let class = if l.class == Class::Physiological { "phys" } else { "high" };
                format!("{},{},{op},{},{class},{}", l.id, l.indicator, l.limit, l.auto_share_consented as u8)
            })
            .collect();
        format!("t1|{}", items.join(";"))
    }

    /// The same table with one high-stakes line's auto-share consent set (ADR-0007 §2).
    pub fn with_consent(&self, id: u16, share: bool) -> Result<SafetyTable, KernelError> {
        let mut t = self.clone();
        let l = t.lines.iter_mut().find(|l| l.id == id).ok_or(KernelError::Grammar("no such line"))?;
        if l.class != Class::HighStakes {
            return Err(KernelError::Grammar("physiological lines always share"));
        }
        l.auto_share_consented = share;
        Ok(t)
    }

    /// Every crossed line for this input, sorted by id (table order is already sorted).
    pub fn evaluate(&self, indicator: &str, value: i64) -> Vec<Hit> {
        debug_assert!(self.lines.len() <= MAX_LINES);
        let mut hits = Vec::with_capacity(4);
        for l in self.lines.iter().filter(|l| l.indicator == indicator) {
            let crossed = match l.op {
                Op::Above => value > l.limit,
                Op::Below => value < l.limit,
            };
            if crossed {
                let share = match l.class {
                    Class::Physiological => true,
                    Class::HighStakes => l.auto_share_consented,
                };
                hits.push(Hit { id: l.id, share });
            }
        }
        debug_assert!(hits.windows(2).all(|w| w[0].id < w[1].id));
        hits
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Trusted result; empty = no line crossed.
    Hits(Vec<Hit>),
    /// No two verified copies agreed. Never to be rendered as "fine".
    CheckFailed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TmrResult {
    pub outcome: Outcome,
    /// Copies whose digest or parse failed; repair them from a verified copy.
    pub damaged: Vec<usize>,
}

/// Triple-modular evaluation over three stored table copies (ADR-0003 §2).
/// A copy counts only if its SHA-256 equals the signed digest and it parses.
pub fn tmr_evaluate(copies: [&str; 3], expected_digest: &str, indicator: &str, value: i64) -> TmrResult {
    if !is_key(indicator) {
        return TmrResult { outcome: Outcome::CheckFailed, damaged: vec![] };
    }
    let mut results: Vec<Vec<Hit>> = Vec::with_capacity(3);
    let mut damaged = Vec::new();
    for (i, c) in copies.iter().enumerate() {
        match (sha256_hex(c.as_bytes()) == expected_digest, SafetyTable::parse(c)) {
            (true, Ok(t)) => results.push(t.evaluate(indicator, value)),
            _ => damaged.push(i),
        }
    }
    let outcome = results
        .iter()
        .find(|r| results.iter().filter(|o| o == r).count() >= 2)
        .map_or(Outcome::CheckFailed, |r| Outcome::Hits(r.clone()));
    TmrResult { outcome, damaged }
}

/// A verified copy to overwrite damaged ones with, if any exists.
pub fn verified_copy<'a>(copies: [&'a str; 3], expected_digest: &str) -> Option<&'a str> {
    copies
        .into_iter()
        .find(|c| sha256_hex(c.as_bytes()) == expected_digest && SafetyTable::parse(c).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VECTORS: &str = include_str!("../../oracle/vectors/kernel_vectors.txt");

    fn parse_expected(s: &str) -> Vec<Hit> {
        if s == "none" {
            return vec![];
        }
        s.split(',')
            .map(|h| {
                let (id, sh) = h.split_once(':').unwrap();
                Hit { id: id.parse().unwrap(), share: sh == "1" }
            })
            .collect()
    }

    /// Kernel A must agree with the Python reference on every equivalence-class vector.
    #[test]
    fn agrees_with_reference_on_exhaustive_vectors() {
        let mut table: Option<SafetyTable> = None;
        let mut table_text = "";
        let mut cases = 0;
        for line in VECTORS.lines().filter(|l| !l.is_empty() && !l.starts_with('#')) {
            let (tag, rest) = line.split_once('\t').unwrap();
            match tag {
                "table" => {
                    let t = SafetyTable::parse(rest).expect("reference table must parse");
                    assert_eq!(t.canonical(), rest, "canonical form must round-trip");
                    table = Some(t);
                    table_text = rest;
                }
                "digest" => assert_eq!(sha256_hex(table_text.as_bytes()), rest, "SHA-256 differs from Python hashlib"),
                "case" => {
                    let f: Vec<&str> = rest.split('\t').collect();
                    let got = table.as_ref().unwrap().evaluate(f[0], f[1].parse().unwrap());
                    assert_eq!(got, parse_expected(f[2]), "case {rest}");
                    cases += 1;
                }
                "reject" => assert!(SafetyTable::parse(rest).is_err(), "should reject: {rest}"),
                _ => panic!("unknown tag {tag}"),
            }
        }
        assert!(cases > 1000, "too few vectors: {cases}");
    }

    const T: &str = "t1|1,spo2,below,900,phys,1;2,cssrs_plan,above,0,high,0;3,cssrs_intent,above,0,high,1";

    #[test]
    fn consent_change_touches_exactly_one_high_stakes_flag() {
        let t = SafetyTable::parse(T).unwrap();
        let w = t.with_consent(2, true).unwrap();
        assert_eq!(w.canonical(), T.replace("cssrs_plan,above,0,high,0", "cssrs_plan,above,0,high,1"));
        assert_eq!(w.evaluate("cssrs_plan", 1), vec![Hit { id: 2, share: true }]);
        assert_eq!(SafetyTable::parse(&w.canonical()).unwrap(), w);
        assert!(t.with_consent(1, false).is_err(), "phys line consent must not change");
        assert!(t.with_consent(9, true).is_err());
        assert_eq!(SafetyTable::parse("t1|").unwrap().canonical(), "t1|");
    }

    #[test]
    fn tmr_masks_one_damaged_copy_and_fails_safe_on_two() {
        let d = sha256_hex(T.as_bytes());
        let bad = T.replace("900", "800");
        let r = tmr_evaluate([T, &bad, T], &d, "spo2", 850);
        assert_eq!(r.outcome, Outcome::Hits(vec![Hit { id: 1, share: true }]));
        assert_eq!(r.damaged, vec![1]);
        let r2 = tmr_evaluate([&bad, &bad, T], &d, "spo2", 850);
        assert_eq!(r2.outcome, Outcome::CheckFailed);
        assert_eq!(verified_copy([&bad, &bad, T], &d), Some(T));
    }

    #[test]
    fn high_stakes_share_follows_per_item_consent() {
        let t = SafetyTable::parse(T).unwrap();
        assert_eq!(t.evaluate("cssrs_plan", 1), vec![Hit { id: 2, share: false }]);
        assert_eq!(t.evaluate("cssrs_intent", 1), vec![Hit { id: 3, share: true }]);
        assert_eq!(t.evaluate("spo2", 900), vec![]); // strict comparison
    }

    #[test]
    fn invalid_indicator_input_fails_safe() {
        let d = sha256_hex(T.as_bytes());
        assert_eq!(tmr_evaluate([T, T, T], &d, "SpO2", 1).outcome, Outcome::CheckFailed);
    }
}
