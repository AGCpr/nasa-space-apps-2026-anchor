use super::*;
use anchor_core::detect::{Calibration, Drift};
use anchor_core::kernel::Hit;

/// Minimal strict JSON validator (RFC 8259 subset the serialiser can emit), so the test needs
/// no dependency. Returns the rest of the input after one value.
fn value(s: &str) -> Result<&str, String> {
    let s = s.trim_start();
    let c = s.chars().next().ok_or("eof")?;
    match c {
        '{' | '[' => {
            let close = if c == '{' { '}' } else { ']' };
            let mut r = s[1..].trim_start();
            if let Some(rest) = r.strip_prefix(close) {
                return Ok(rest);
            }
            loop {
                if c == '{' {
                    r = string(r.trim_start())?.trim_start().strip_prefix(':').ok_or("missing ':'")?;
                }
                r = value(r)?.trim_start();
                if let Some(rest) = r.strip_prefix(',') {
                    r = rest;
                } else {
                    return r.strip_prefix(close).ok_or(format!("expected {close} at {r:.20}"));
                }
            }
        }
        '"' => string(s),
        't' => s.strip_prefix("true").ok_or("bad literal".into()),
        'f' => s.strip_prefix("false").ok_or("bad literal".into()),
        'n' => s.strip_prefix("null").ok_or("bad literal".into()),
        _ => {
            let end = s.find(|c: char| !(c.is_ascii_digit() || "-+.eE".contains(c))).unwrap_or(s.len());
            s[..end].parse::<f64>().map_err(|_| format!("bad number {:.20}", s))?;
            Ok(&s[end..])
        }
    }
}

fn string(s: &str) -> Result<&str, String> {
    let mut it = s.strip_prefix('"').ok_or("expected string")?.char_indices();
    while let Some((i, c)) = it.next() {
        match c {
            '"' => return Ok(&s[i + 2..]),
            '\\' => {
                it.next();
            }
            c if (c as u32) < 0x20 => return Err("raw control character".into()),
            _ => {}
        }
    }
    Err("unterminated string".into())
}

fn valid(s: &str) {
    assert_eq!(value(s).map(str::trim), Ok(""), "{s}");
}

#[test]
fn every_output_variant_serialises_to_valid_json() {
    let (s, _) = synth::preflight(1).unwrap();
    let cal = Calibration { h: 4.5, arl0: 36.0, censored: 0.0, horizon: 360, seed: u64::MAX };
    let all = vec![
        Output::Safety { day: 1, indicator: "spo2".into(), value: 884, hits: vec![Hit { id: 1, share: true }] },
        Output::SafetyCheckFailed { day: 1, indicator: "spo2".into(), value: 1 },
        Output::KernelUnavailable { day: 1, indicator: "spo2".into(), value: 1 },
        Output::OfferExtraSample { day: 1, indicator: "mood".into() },
        Output::DayStat { day: 1, indicator: "mood".into(), z: f64::NAN, statistic: 0.0, y: 0.1, lo: -1.0, hi: 1.0, anchor: 0.4 },
        Output::Alarm { day: 1, indicator: "mood".into(), statistic: 5.0, h: 4.5 },
        Output::AlarmCleared { day: 1, indicator: "mood".into() },
        Output::MissingnessAlarm { day: 1 },
        Output::MissingnessCleared { day: 1 },
        Output::DriftPrompt { day: 1, indicator: "mood".into(), drift: Drift { n: 28, slope: -0.1, sen_slope: -0.1, phi: 0.3, z: 3.0, prompt: true } },
        Output::AnchorRunPrompt { day: 1, indicator: "mood".into() },
        Output::Calibrating { day: 1 },
        Output::Recalibrated { day: 1, bank: cal, missingness: None },
        Output::Rejected { seq: 3, reason: "a \"quoted\" reason\n" },
        Output::CompanionError { day: 1, reason: "x\\y\u{1}".into() },
        Output::PrivateSupport { day: 1, items: vec!["cssrs_plan".into()] },
        Output::ConsentChanged { day: 1, line: 3, share: true, table: "t1|".into(), digest: "ab".into() },
        Output::Reanchored { day: 1, indicator: "mood".into(), from: 0.6, to: -0.1 },
        Output::ReanchorReverted { day: 1, indicator: "mood".into() },
        Output::ShareQueued { share: 0, class: "safety", to: None, payload: "share=0;day=1".into() },
        Output::ShareSent { share: 0, to: "cmo".into() },
        Output::ShareRerouted { share: 0, to: "cmo".into() },
        Output::ShareUndeliverable { share: 0, class: "ppc" },
        Output::ShareCancelled { share: 0 },
        Output::ExclusionChanged { recipient: "cmo".into(), excluded: true },
        Output::RouteEmpty { class: "crew" },
    ];
    for o in &all {
        let j = output_json(&s.engine, o);
        valid(&j);
        assert!(j.starts_with("{\"t\":\""), "{j}");
    }
    // Seeds above 2^53 travel as strings; non-finite numbers as null; slider values in record units.
    assert!(output_json(&s.engine, &all[12]).contains("\"seed\":\"18446744073709551615\""));
    assert!(output_json(&s.engine, &all[4]).contains("\"z\":null"));
}

#[test]
fn boundary_round_trip_and_rejection_without_append() {
    let out = op_preflight();
    valid(&out);
    let first_line = |j: &str| j.matches("\"line\":").count();
    let n = first_line(&out);
    assert!(n > 40, "preflight events: {n}");
    // A rejected record is reported and not appended.
    let bad = op_submit(&format!("{}|checkin|day=2;mood=500", synth::DAY_MS));
    assert_eq!(bad, "{\"ok\":false,\"error\":\"day is not the open day\"}");
    let ok = op_submit(&format!("{}|checkin|day=1;mood=500", synth::DAY_MS));
    valid(&ok);
    assert!(ok.starts_with("{\"ok\":true"), "{ok}");
    // Reload from two copies, one with a flipped bit: repaired and identical.
    let lines: Vec<String> = SESSION.with(|s| s.borrow().log.lines());
    let mut damaged = lines.clone();
    damaged[5] = damaged[5].replacen("dir=-1", "dir=-2", 1);
    assert_ne!(damaged[5], lines[5]);
    let reloaded = op_load(&format!("{}\0{}", damaged.join("\n"), lines.join("\n")));
    assert!(reloaded.starts_with("{\"ok\":true"), "{reloaded:.200}");
    assert_eq!(SESSION.with(|s| s.borrow().log.lines()), lines);
    valid(&op_transmittable());
}

#[test]
fn demo_serialises() {
    let out = op_demo();
    valid(&out);
    assert!(out.contains("\"t\":\"Reanchored\""));
}
