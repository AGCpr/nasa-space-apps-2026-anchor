//! Synthetic demonstration mission (ADR-0009 §12). Every value is generated; nothing here is, or
//! may be presented as, astronaut data. A real engine is driven while the log is written, so each
//! scripted action is valid at the moment it is appended.

use crate::log::sha256_hex;
use crate::rng::Rng;
use crate::session::{Event, Session};

pub const DAY_MS: i64 = 86_400_000;
const HOUR_MS: i64 = 3_600_000;
pub const DEMO_DAYS: i64 = 150;
/// Chosen by `find_demo_seed` (ADR-0009 §12).
pub const DEMO_SEED: u64 = 1;
pub const DEMO_TABLE: &str =
    "t1|1,spo2,below,900,phys,1;2,sleep_min,below,120,phys,1;3,cssrs_plan,above,0,high,0;4,cssrs_intent,above,0,high,1";
pub const RECIPIENTS: [(&str, i64); 4] = [("surgeon", 0), ("bhp", 0), ("cmo", 1), ("crewmate", 1)];
pub const ROUTES: [(&str, &str); 7] = [
    ("safety", "surgeon"),
    ("safety", "cmo"),
    ("summary", "surgeon"),
    ("ppc", "bhp"),
    ("ppc", "surgeon"),
    ("crew", "crewmate"),
    ("crew", "cmo"),
];
/// Check-in sliders: bipolar, harmful direction down (R15).
pub const SLIDERS: [&str; 3] = ["mood", "energy", "calm"];
pub const MOOD_SHIFT_DAY: i64 = 100;
pub const LOS_DAYS: (i64, i64) = (118, 122);
pub const SPO2_CROSSING_DAY: i64 = 120;
pub const REANCHOR_DAY: i64 = 125;
pub const GAP_DAYS: (i64, i64) = (130, 138);

fn slider(y: f64) -> i64 {
    (1000.0 / (1.0 + (-y).exp())).round().clamp(0.0, 1000.0) as i64
}

fn push(s: &mut Session, events: &mut Vec<Event>, r: crate::log::Record) -> Result<(), String> {
    let (kind, seq) = (r.kind.clone(), r.seq);
    events.push(s.submit(r).map_err(|e| format!("seq {seq} ({kind}): {e}"))?);
    Ok(())
}

/// Pre-flight package: configuration and 30 pre-flight days (all synthetic).
pub fn preflight(seed: u64) -> Result<(Session, Vec<Event>), String> {
    let (mut s, mut ev) = (Session::new(), Vec::new());
    let mut rng = Rng::new(seed);
    let r = s.record(0, "setup").int("budget_days", 30).map_err(|e| e.to_string())?;
    push(&mut s, &mut ev, r)?;
    // key, transform, decimals, population (mu, tau2, sigma2, nu_p) x 1e6 on the model scale
    let inds: [(&str, i64, i64, [i64; 4]); 5] = [
        ("mood", 0, 0, [400_000, 300_000, 250_000, 5_000_000]),
        ("energy", 0, 0, [300_000, 300_000, 250_000, 5_000_000]),
        ("calm", 0, 0, [500_000, 300_000, 250_000, 5_000_000]),
        ("sleep_min", 1, 0, [380_000_000, 900_000_000, 1_600_000_000, 5_000_000]),
        ("spo2", 1, 1, [97_500_000, 250_000, 160_000, 5_000_000]),
    ];
    for (key, tr, dec, p) in inds {
        let r = s.record(0, "indicator").text("key_txt", key).and_then(|r| r.int("dir", -1)).and_then(|r| r.int("transform", tr))
            .and_then(|r| r.int("decimals", dec)).and_then(|r| r.int("mu_e6", p[0])).and_then(|r| r.int("tau2_e6", p[1]))
            .and_then(|r| r.int("sigma2_e6", p[2])).and_then(|r| r.int("nu_p_e6", p[3])).map_err(|e| e.to_string())?;
        push(&mut s, &mut ev, r)?;
    }
    let r = s.record(0, "safety_table").text("table_txt", DEMO_TABLE).and_then(|r| r.text("digest_txt", &sha256_hex(DEMO_TABLE.as_bytes())));
    push(&mut s, &mut ev, r.map_err(|e| e.to_string())?)?;
    for (key, crew) in RECIPIENTS {
        let r = s.record(0, "recipient").text("key_txt", key).and_then(|r| r.int("crew", crew)).map_err(|e| e.to_string())?;
        push(&mut s, &mut ev, r)?;
    }
    for (class, to) in ROUTES {
        let r = s.record(0, "route").text("class_txt", class).and_then(|r| r.text("to_txt", to)).map_err(|e| e.to_string())?;
        push(&mut s, &mut ev, r)?;
    }
    let r = s.record(0, "link").int("up", 1).map_err(|e| e.to_string())?;
    push(&mut s, &mut ev, r)?;
    for _ in 0..30 {
        let r = s.record(0, "preflight")
            .int("mood", slider(0.6 + 0.5 * rng.normal()))
            .and_then(|r| r.int("energy", slider(0.4 + 0.5 * rng.normal())))
            .and_then(|r| r.int("calm", slider(0.7 + 0.5 * rng.normal())))
            .and_then(|r| r.int("sleep_min", (390.0 + 40.0 * rng.normal()).round() as i64))
            .and_then(|r| r.int("spo2", (975.0 + 4.0 * rng.normal()).round() as i64))
            .map_err(|e| e.to_string())?;
        push(&mut s, &mut ev, r)?;
    }
    Ok((s, ev))
}

/// The 150-day demonstration mission (scripted events: see ADR-0009 §12 and the test below).
pub fn mission(seed: u64) -> Result<(Session, Vec<Event>), String> {
    let (mut s, mut ev) = preflight(seed)?;
    let mut rng = Rng::new(seed ^ 0xA5A5);
    let mut ar = [0.0f64; 4]; // AR(1) states: mood, energy, calm, sleep
    for day in 1..=DEMO_DAYS {
        let base = day * DAY_MS;
        for a in ar.iter_mut() {
            *a = 0.3 * *a + (1.0f64 - 0.09).sqrt() * rng.normal();
        }
        let e = |r: Result<crate::log::Record, crate::log::LogError>| r.map_err(|e| e.to_string());
        let morning = base + 7 * HOUR_MS;
        // Scripted context records.
        let script: Vec<(i64, crate::log::Record)> = match day {
            1 => vec![(morning, e(s.record(morning, "share_category").text("key_txt", "sleep_min").and_then(|r| r.int("on", 1)))?)],
            d if d == LOS_DAYS.0 => vec![(morning, e(s.record(morning, "link").int("up", 0))?)],
            d if d == LOS_DAYS.1 + 1 => vec![(morning, e(s.record(morning, "link").int("up", 1))?)],
            70 => vec![(morning, e(s.record(morning, "share_category").text("key_txt", "sleep_min").and_then(|r| r.int("on", 0)))?)],
            _ => vec![],
        };
        for (_, r) in script {
            push(&mut s, &mut ev, r)?;
        }
        // Morning: imported wearable samples (continue through the check-in gap).
        let spo2 = if day == SPO2_CROSSING_DAY { 884 } else { (975.0 + 4.0 * rng.normal()).round() as i64 };
        let r = e(s.record(morning, "sample").int("day", day).and_then(|r| r.int("sleep_min", (390.0 + 40.0 * ar[3]).round() as i64)).and_then(|r| r.int("spo2", spo2)))?;
        push(&mut s, &mut ev, r)?;
        ev.extend(s.transmit(morning)?);
        // Evening check-in (pre-sleep lighting): three bipolar sliders.
        let evening = base + 20 * HOUR_MS + 30 * 60_000;
        if !(GAP_DAYS.0..=GAP_DAYS.1).contains(&day) {
            let shift = if day >= MOOD_SHIFT_DAY { -0.75 } else { 0.0 };
            let r = e(s.record(evening, "checkin").int("day", day)
                .and_then(|r| r.int("mood", slider(0.6 + shift + 0.5 * ar[0])))
                .and_then(|r| r.int("energy", slider(0.4 + 0.4 * shift + 0.5 * ar[1])))
                .and_then(|r| r.int("calm", slider(0.7 + 0.5 * ar[2]))))?;
            push(&mut s, &mut ev, r)?;
        }
        let r = match day {
            45 => Some(e(s.record(evening, "screen").int("day", day).and_then(|r| r.int("cssrs_plan", 0)).and_then(|r| r.int("cssrs_intent", 0)))?),
            50 => Some(e(s.record(evening, "consent").int("day", day).and_then(|r| r.int("line", 3)).and_then(|r| r.int("share", 1)))?),
            REANCHOR_DAY => Some(e(s.record(evening, "reanchor").int("day", day).and_then(|r| r.text("key_txt", "mood")))?),
            141 => Some(e(s.record(evening, "ppc_request").int("day", day))?),
            _ => None,
        };
        if let Some(r) = r {
            push(&mut s, &mut ev, r)?;
        }
        ev.extend(s.transmit(evening)?);
        let close = base + 23 * HOUR_MS;
        let r = e(s.record(close, "day_close").int("day", day))?;
        push(&mut s, &mut ev, r)?;
        ev.extend(s.transmit(close)?);
    }
    Ok((s, ev))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Output;

    /// The demo must keep demonstrating what ADR-0009 §12 says it demonstrates.
    #[test]
    fn demo_mission_shows_every_scripted_event() {
        let (s, ev) = mission(DEMO_SEED).unwrap();
        let outs: Vec<&Output> = ev.iter().flat_map(|(_, o)| o).collect();
        let any = |f: &dyn Fn(&Output) -> bool| outs.iter().any(|o| f(o));
        assert!(!any(&|o| matches!(o, Output::Rejected { .. } | Output::CompanionError { .. })));
        let mood_alarm = outs.iter().find_map(|o| match o {
            Output::Alarm { day, indicator, .. } if indicator == "mood" && *day >= MOOD_SHIFT_DAY => Some(*day),
            _ => None,
        });
        assert!(mood_alarm.is_some_and(|d| d < REANCHOR_DAY - 14), "mood alarm {mood_alarm:?}");
        assert!(any(&|o| matches!(o, Output::Reanchored { day: REANCHOR_DAY, indicator, .. } if indicator == "mood")));
        assert!(any(&|o| matches!(o, Output::Safety { day: SPO2_CROSSING_DAY, indicator, .. } if indicator == "spo2")));
        // Safety notice during loss of signal goes to the CMO on board, and is sent.
        let safety = outs.iter().find_map(|o| match o {
            Output::ShareQueued { share, class: "safety", to, .. } => Some((*share, to.clone())),
            _ => None,
        });
        assert_eq!(safety.as_ref().map(|s| s.1.as_deref()), Some(Some("cmo")));
        assert!(any(&|o| matches!(o, Output::ShareSent { share, to } if Some(*share) == safety.as_ref().map(|s| s.0) && to == "cmo")));
        assert!(any(&|o| matches!(o, Output::MissingnessAlarm { day } if (GAP_DAYS.0..=GAP_DAYS.1 + 1).contains(day))));
        assert!(any(&|o| matches!(o, Output::ConsentChanged { line: 3, share: true, .. })));
        assert!(any(&|o| matches!(o, Output::ShareQueued { class: "summary", .. })));
        assert!(any(&|o| matches!(o, Output::ShareQueued { class: "ppc", .. })));
        // Everything authorised was transmitted by the end.
        assert!(s.engine.transmittable().is_empty());
        // DayStat carries a proper range around its location on every day.
        for o in &outs {
            if let Output::DayStat { lo, hi, y, .. } = o {
                assert!(lo < hi && y.is_finite(), "{o:?}");
            }
        }
    }

    /// Seed search for the demo (run with --ignored --nocapture): the first seed whose mood
    /// episode starts after the scripted shift, so the shift is not masked by a coincident
    /// false alarm. Other indicators' false alarms are left as they fall (ADR-0009 §12).
    #[test]
    #[ignore]
    fn find_demo_seed() {
        for seed in 1..200u64 {
            let (_, ev) = mission(seed).unwrap();
            let mood: Vec<i64> = ev.iter().flat_map(|(_, o)| o).filter_map(|o| match o {
                Output::Alarm { day, indicator, .. } if indicator == "mood" => Some(*day),
                _ => None,
            }).collect();
            let first_after = mood.iter().find(|d| **d >= 80).copied();
            if first_after.is_some_and(|d| (MOOD_SHIFT_DAY..MOOD_SHIFT_DAY + 12).contains(&d)) {
                eprintln!("seed {seed}: mood alarms {mood:?}");
                return;
            }
        }
    }

    #[test]
    fn display_inverts_the_slider_transform_and_range_is_80_percent() {
        let (s, _) = preflight(1).unwrap();
        for v in [0, 1, 250, 500, 999, 1000] {
            let y = crate::baseline::logit_slider(v).unwrap();
            assert!((s.engine.display("mood", y).unwrap() - v as f64).abs() < 1e-9);
        }
        assert_eq!(s.engine.display("spo2", 97.5), Some(97.5));
        let nig = crate::baseline::Nig { m: 1.0, kappa: 5.0, alpha: 4.0, beta: 2.0 };
        let (lo, hi) = nig.interval(crate::engine::RANGE_Z).unwrap();
        assert!((nig.z(lo).unwrap() + crate::engine::RANGE_Z).abs() < 1e-9);
        assert!((nig.z(hi).unwrap() - crate::engine::RANGE_Z).abs() < 1e-9);
    }

    /// ADR-0009 §7: a kernel disagreement is a recorded, shared kernel fault; a screen's is not shared.
    #[test]
    fn kernel_disagreement_is_a_shared_fault() {
        let (mut s, _) = preflight(3).unwrap();
        let met = DAY_MS;
        let (e, _) = s.submit(s.record(met, "sample").int("day", 1).unwrap().int("spo2", 970).unwrap()).unwrap();
        let disagree = |s: &Session, seq: u64, key: &str| s.record(met, "kernel_disagree").int("day", 1).unwrap().int("ref", seq as i64).unwrap().text("key_txt", key).unwrap();
        assert_eq!(s.submit(disagree(&s, e.record.seq, "mood")).unwrap_err(), "no such value today");
        let (_, out) = s.submit(disagree(&s, e.record.seq, "spo2")).unwrap();
        assert_eq!(out[0], Output::SafetyCheckFailed { day: 1, indicator: "spo2".into(), value: 970 });
        assert!(matches!(&out[1], Output::ShareQueued { class: "safety", payload, .. } if payload.ends_with("fault=1")));
        assert_eq!(s.submit(disagree(&s, e.record.seq, "spo2")).unwrap_err(), "disagreement already recorded");
        let (e, _) = s.submit(s.record(met, "screen").int("day", 1).unwrap().int("cssrs_plan", 0).unwrap()).unwrap();
        let (_, out) = s.submit(disagree(&s, e.record.seq, "cssrs_plan")).unwrap();
        assert_eq!(out.len(), 1, "a screen never shares, not even a fault: {out:?}");
        s.submit(s.record(met, "day_close").int("day", 1).unwrap()).unwrap();
        assert_eq!(s.submit(disagree(&s, e.record.seq, "cssrs_plan")).unwrap_err(), "day is not the open day");
    }
}
