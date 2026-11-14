//! Append-only, hash-chained event log (ADR-0002).
//!
//! One record per line:
//! `v1|<seq>|<met_ms>|<kind>|<k>=<v>;...|<prev_hash>|<hash>`
//! with `hash = SHA-256(body + "|" + prev_hash)` where body is the first five fields.
//! Integer fields are signed decimal; keys ending in `_txt` carry lowercase hex of UTF-8.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt;

pub const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";
pub const MAX_FIELDS: usize = 64;
pub const MAX_KEY_LEN: usize = 32;
pub const MAX_LINE_LEN: usize = 65_536;
const TEXT_SUFFIX: &str = "_txt";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Text(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub seq: u64,
    pub met_ms: i64,
    pub kind: String,
    pub fields: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub record: Record,
    pub prev_hash: String,
    pub hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogError {
    Grammar(&'static str),
    BadHash,
    BrokenChain,
    SeqGap { expected: u64, found: u64 },
    TimeReversed,
}

impl fmt::Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

pub fn is_key(s: &str) -> bool {
    !s.is_empty() && s.len() <= MAX_KEY_LEN && s.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

fn is_kind(s: &str) -> bool {
    !s.is_empty() && s.len() <= MAX_KEY_LEN && s.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
}

fn is_hash(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub fn sha256_hex(data: &[u8]) -> String {
    let d = Sha256::digest(data);
    let mut s = String::with_capacity(64);
    for b in d {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn hex_encode(text: &str) -> String {
    text.bytes().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode(h: &str) -> Result<String, LogError> {
    if !h.len().is_multiple_of(2) || !h.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
        return Err(LogError::Grammar("text value is not lowercase hex"));
    }
    let bytes: Vec<u8> = (0..h.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).expect("validated hex"))
        .collect();
    String::from_utf8(bytes).map_err(|_| LogError::Grammar("text value is not UTF-8"))
}

/// Canonical decimal: no leading '+', no leading zeros, no "-0".
fn parse_int(s: &str) -> Result<i64, LogError> {
    let digits = s.strip_prefix('-').unwrap_or(s);
    let canonical = !digits.is_empty()
        && digits.bytes().all(|b| b.is_ascii_digit())
        && (digits == "0" || !digits.starts_with('0'))
        && s != "-0";
    if !canonical {
        return Err(LogError::Grammar("integer is not canonical decimal"));
    }
    s.parse::<i64>().map_err(|_| LogError::Grammar("integer out of range"))
}

fn parse_u64(s: &str) -> Result<u64, LogError> {
    let v = parse_int(s)?;
    u64::try_from(v).map_err(|_| LogError::Grammar("negative sequence number"))
}

impl Record {
    pub fn new(seq: u64, met_ms: i64, kind: &str) -> Result<Self, LogError> {
        if !is_kind(kind) {
            return Err(LogError::Grammar("invalid kind"));
        }
        Ok(Record { seq, met_ms, kind: kind.to_string(), fields: BTreeMap::new() })
    }

    pub fn int(self, key: &str, v: i64) -> Result<Self, LogError> {
        if !is_key(key) || key.ends_with(TEXT_SUFFIX) {
            return Err(LogError::Grammar("invalid integer key"));
        }
        self.insert(key, Value::Int(v))
    }

    pub fn text(self, key: &str, v: &str) -> Result<Self, LogError> {
        if !is_key(key) || !key.ends_with(TEXT_SUFFIX) {
            return Err(LogError::Grammar("text keys must end in _txt"));
        }
        self.insert(key, Value::Text(v.to_string()))
    }

    fn insert(mut self, key: &str, v: Value) -> Result<Self, LogError> {
        if self.fields.len() >= MAX_FIELDS || self.fields.contains_key(key) {
            return Err(LogError::Grammar("too many or duplicate fields"));
        }
        self.fields.insert(key.to_string(), v);
        Ok(self)
    }

    /// The five hashed fields. BTreeMap iteration gives sorted keys, hence one encoding per record.
    pub fn body(&self) -> String {
        let fields: Vec<String> = self
            .fields
            .iter()
            .map(|(k, v)| match v {
                Value::Int(i) => format!("{k}={i}"),
                Value::Text(t) => format!("{k}={}", hex_encode(t)),
            })
            .collect();
        format!("v1|{}|{}|{}|{}", self.seq, self.met_ms, self.kind, fields.join(";"))
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        match self.fields.get(key) {
            Some(Value::Int(v)) => Some(*v),
            _ => None,
        }
    }
}

pub fn compute_hash(body: &str, prev_hash: &str) -> String {
    sha256_hex(format!("{body}|{prev_hash}").as_bytes())
}

impl Entry {
    pub fn line(&self) -> String {
        format!("{}|{}|{}", self.record.body(), self.prev_hash, self.hash)
    }
}

/// Parse one line and check its own hash. Chain position is checked by the caller.
pub fn parse_line(line: &str) -> Result<Entry, LogError> {
    if line.len() > MAX_LINE_LEN || !line.is_ascii() {
        return Err(LogError::Grammar("line too long or not ASCII"));
    }
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() != 7 || parts[0] != "v1" {
        return Err(LogError::Grammar("expected 7 fields starting with v1"));
    }
    let seq = parse_u64(parts[1])?;
    let met_ms = parse_int(parts[2])?;
    let mut rec = Record::new(seq, met_ms, parts[3])?;
    if !parts[4].is_empty() {
        let mut last_key: Option<&str> = None;
        for kv in parts[4].split(';') {
            let (k, v) = kv.split_once('=').ok_or(LogError::Grammar("field without '='"))?;
            if last_key.is_some_and(|p| p >= k) {
                return Err(LogError::Grammar("keys not strictly sorted"));
            }
            last_key = Some(k);
            rec = if k.ends_with(TEXT_SUFFIX) { rec.text(k, &hex_decode(v)?)? } else { rec.int(k, parse_int(v)?)? };
        }
    }
    let (prev_hash, hash) = (parts[5], parts[6]);
    if !is_hash(prev_hash) || !is_hash(hash) {
        return Err(LogError::Grammar("hash fields must be 64 lowercase hex"));
    }
    if compute_hash(&rec.body(), prev_hash) != hash {
        return Err(LogError::BadHash);
    }
    Ok(Entry { record: rec, prev_hash: prev_hash.to_string(), hash: hash.to_string() })
}

/// Validate `line` as the successor of `prev` (None = genesis position).
fn accept(line: &str, prev: Option<&Entry>) -> Result<Entry, LogError> {
    let e = parse_line(line)?;
    let (want_prev, want_seq) = match prev {
        Some(p) => (p.hash.as_str(), p.record.seq + 1),
        None => (GENESIS, 0),
    };
    if e.prev_hash != want_prev {
        return Err(LogError::BrokenChain);
    }
    if e.record.seq != want_seq {
        return Err(LogError::SeqGap { expected: want_seq, found: e.record.seq });
    }
    if prev.is_some_and(|p| e.record.met_ms < p.record.met_ms) {
        return Err(LogError::TimeReversed);
    }
    Ok(e)
}

#[derive(Clone, Debug, Default)]
pub struct Log {
    entries: Vec<Entry>,
}

impl Log {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn next_seq(&self) -> u64 {
        self.entries.last().map_or(0, |e| e.record.seq + 1)
    }

    /// Seal and append a record. The record is validated as if read back, so a bad record never enters.
    pub fn append(&mut self, record: Record) -> Result<&Entry, LogError> {
        let prev = self.entries.last();
        let prev_hash = prev.map_or(GENESIS.to_string(), |p| p.hash.clone());
        let hash = compute_hash(&record.body(), &prev_hash);
        let line = Entry { record, prev_hash, hash }.line();
        let e = accept(&line, prev)?;
        self.entries.push(e);
        Ok(self.entries.last().expect("just pushed"))
    }

    pub fn lines(&self) -> Vec<String> {
        self.entries.iter().map(Entry::line).collect()
    }

    /// Load a full copy; fails at the first invalid line.
    pub fn load(lines: &[String]) -> Result<Self, (usize, LogError)> {
        let mut log = Log::new();
        for (i, l) in lines.iter().enumerate() {
            let e = accept(l, log.entries.last()).map_err(|err| (i, err))?;
            log.entries.push(e);
        }
        Ok(log)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepairError {
    /// Neither copy has a valid line at this index although lines remain.
    Unrecoverable(usize),
    /// Both copies are valid but different here: two histories, not a bit flip. Never guess.
    Divergent(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repair {
    pub lines: Vec<String>,
    /// Indices where copy A was replaced from copy B, and vice versa.
    pub fixed_in_a: Vec<usize>,
    pub fixed_in_b: Vec<usize>,
}

/// Rebuild the log from two copies (ADR-0002 §5). A bit flip breaks only its own line's hash;
/// later lines still chain to the true hash, so the other copy's line restores the chain.
pub fn repair(a: &[String], b: &[String]) -> Result<Repair, RepairError> {
    let mut out: Vec<Entry> = Vec::new();
    let (mut fa, mut fb) = (Vec::new(), Vec::new());
    for i in 0..a.len().max(b.len()) {
        let prev = out.last();
        let ea = a.get(i).and_then(|l| accept(l, prev).ok());
        let eb = b.get(i).and_then(|l| accept(l, prev).ok());
        let chosen = match (ea, eb) {
            (Some(x), Some(y)) if x != y => return Err(RepairError::Divergent(i)),
            (Some(x), Some(_)) => x,
            (Some(x), None) => {
                if i < b.len() {
                    fb.push(i);
                }
                x
            }
            (None, Some(y)) => {
                if i < a.len() {
                    fa.push(i);
                }
                y
            }
            (None, None) => return Err(RepairError::Unrecoverable(i)),
        };
        out.push(chosen);
    }
    Ok(Repair { lines: out.iter().map(Entry::line).collect(), fixed_in_a: fa, fixed_in_b: fb })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(n: u64) -> Log {
        let mut log = Log::new();
        for i in 0..n {
            let r = Record::new(log.next_seq(), 1_000 * i as i64, "checkin")
                .unwrap()
                .int("valence", 500 + i as i64)
                .unwrap()
                .int("sleep_min", -3)
                .unwrap()
                .text("note_txt", "uyku iyi — ok ✓")
                .unwrap();
            log.append(r).unwrap();
        }
        log
    }

    #[test]
    fn round_trip_and_unicode_text() {
        let log = sample(5);
        let back = Log::load(&log.lines()).unwrap();
        assert_eq!(back.entries(), log.entries());
        let Value::Text(t) = &back.entries()[0].record.fields["note_txt"] else { panic!() };
        assert_eq!(t, "uyku iyi — ok ✓");
    }

    #[test]
    fn every_single_byte_flip_is_detected() {
        let lines = sample(3).lines();
        for (li, line) in lines.iter().enumerate() {
            for pos in 0..line.len() {
                let mut bytes = line.clone().into_bytes();
                bytes[pos] ^= 0x01;
                let mut copy = lines.clone();
                copy[li] = String::from_utf8_lossy(&bytes).into_owned();
                assert!(Log::load(&copy).is_err(), "flip at line {li} byte {pos} undetected");
            }
        }
    }

    #[test]
    fn rejects_noncanonical_and_disordered_input() {
        let line = sample(1).lines()[0].clone();
        for bad in [
            line.replacen("v1|0|", "v1|00|", 1),
            line.replacen("v1|0|", "v1|+0|", 1),
            line.replacen("sleep_min=-3;valence", "valence=500;sleep_min", 1),
            line.replacen("checkin", "Checkin", 1),
            format!("{line}|extra"),
        ] {
            assert!(parse_line(&bad).is_err(), "accepted: {bad}");
        }
    }

    #[test]
    fn append_enforces_sequence_and_time() {
        let mut log = sample(2);
        assert_eq!(log.append(Record::new(5, 9_999, "x").unwrap()).unwrap_err(), LogError::SeqGap { expected: 2, found: 5 });
        assert_eq!(log.append(Record::new(2, 0, "x").unwrap()).unwrap_err(), LogError::TimeReversed);
        assert!(Record::new(0, 0, "x").unwrap().int("bad_txt", 1).is_err());
    }

    #[test]
    fn truncated_last_line_is_reported_at_its_index() {
        let mut lines = sample(4).lines();
        let l = lines[3].clone();
        lines[3] = l[..l.len() / 2].to_string();
        assert_eq!(Log::load(&lines).unwrap_err().0, 3);
    }

    fn flip(lines: &mut [String], i: usize) {
        let mut b = lines[i].clone().into_bytes();
        let p = b.len() / 2;
        b[p] ^= 0x02;
        lines[i] = String::from_utf8_lossy(&b).into_owned();
    }

    #[test]
    fn repair_from_complementary_corruption() {
        let good = sample(6).lines();
        let (mut a, mut b) = (good.clone(), good.clone());
        flip(&mut a, 1);
        flip(&mut a, 4);
        flip(&mut b, 2);
        let r = repair(&a, &b).unwrap();
        assert_eq!(r.lines, good);
        assert_eq!((r.fixed_in_a, r.fixed_in_b), (vec![1, 4], vec![2]));
    }

    #[test]
    fn repair_extends_from_longer_copy_and_refuses_to_guess() {
        let good = sample(5).lines();
        assert_eq!(repair(&good[..3], &good).unwrap().lines, good);

        let (mut a, mut b) = (good.clone(), good.clone());
        flip(&mut a, 2);
        flip(&mut b, 2);
        assert_eq!(repair(&a, &b).unwrap_err(), RepairError::Unrecoverable(2));

        let mut other = Log::load(&good[..2]).unwrap();
        other.append(Record::new(2, 99_999, "checkin").unwrap()).unwrap();
        assert_eq!(repair(&good, &other.lines()).unwrap_err(), RepairError::Divergent(2));
    }
}
