use super::*;
use crate::outbox::{Outbox, ShareClass};
use crate::log::{Log, repair};
use crate::rng::Rng;

const DAY_MS: i64 = 86_400_000;
const TABLE: &str = "t1|1,spo2,below,900,phys,1;2,sleep_min,below,120,phys,1";

fn rec(log: &Log, met: i64, kind: &str) -> Record {
    Record::new(log.next_seq(), met, kind).unwrap()
}

fn indicator(log: &mut Log, key: &str, dir: i64, transform: i64, decimals: i64, pop: [i64; 4]) {
    let r = rec(log, 0, "indicator")
        .text("key_txt", key).unwrap()
        .int("dir", dir).unwrap()
        .int("transform", transform).unwrap()
        .int("decimals", decimals).unwrap()
        .int("mu_e6", pop[0]).unwrap()
        .int("tau2_e6", pop[1]).unwrap()
        .int("sigma2_e6", pop[2]).unwrap()
        .int("nu_p_e6", pop[3]).unwrap();
    log.append(r).unwrap();
}

fn configured_log() -> Log {
    configured_log_with(TABLE)
}

fn configured_log_with(table: &str) -> Log {
    let mut log = Log::new();
    log.append(rec(&log, 0, "setup").int("budget_days", 30).unwrap()).unwrap();
    indicator(&mut log, "mood", -1, 0, 0, [400_000, 300_000, 250_000, 5_000_000]);
    indicator(&mut log, "sleep_min", -1, 1, 0, [380_000_000, 900_000_000, 1_600_000_000, 5_000_000]);
    indicator(&mut log, "spo2", -1, 1, 1, [97_500_000, 250_000, 160_000, 5_000_000]);
    let t = rec(&log, 0, "safety_table")
        .text("table_txt", table).unwrap()
        .text("digest_txt", &sha256_hex(table.as_bytes())).unwrap();
    log.append(t).unwrap();
    for (key, crew) in [("surgeon", 0), ("bhp", 0), ("cmo", 1), ("crewmate", 1)] {
        log.append(rec(&log, 0, "recipient").text("key_txt", key).unwrap().int("crew", crew).unwrap()).unwrap();
    }
    for (class, to) in ROUTES {
        log.append(rec(&log, 0, "route").text("class_txt", class).unwrap().text("to_txt", to).unwrap()).unwrap();
    }
    log.append(rec(&log, 0, "link").int("up", 1).unwrap()).unwrap();
    log
}

const ROUTES: [(&str, &str); 7] = [
    ("safety", "surgeon"),
    ("safety", "cmo"),
    ("summary", "surgeon"),
    ("ppc", "bhp"),
    ("ppc", "surgeon"),
    ("crew", "crewmate"),
    ("crew", "cmo"),
];

fn slider(y: f64) -> i64 {
    (1000.0 / (1.0 + (-y).exp())).round().clamp(0.0, 1000.0) as i64
}

/// 150-day synthetic mission with known events (ADR-0006 test plan):
/// mood drop of ~1.5 SD from day 100; spo2 safety crossing on day 120; no check-ins on days
/// 130-140; a workload-tagged day 60 without check-in.
fn mission() -> Log {
    let mut log = configured_log();
    let mut rng = Rng::new(2026);
    let (mut m, mut s) = (0.0, 0.0);
    for _ in 0..30 {
        let r = rec(&log, 0, "preflight")
            .int("mood", slider(0.6 + 0.5 * rng.normal())).unwrap()
            .int("sleep_min", (390.0 + 40.0 * rng.normal()).round() as i64).unwrap()
            .int("spo2", (975.0 + 4.0 * rng.normal()).round() as i64).unwrap();
        log.append(r).unwrap();
    }
    for day in 1..=150i64 {
        let met = day * DAY_MS;
        m = 0.3 * m + (1.0f64 - 0.09).sqrt() * rng.normal();
        s = 0.3 * s + (1.0f64 - 0.09).sqrt() * rng.normal();
        let shift = if day >= 100 { -0.75 } else { 0.0 };
        let spo2 = if day == 120 { 880 } else { (975.0 + 4.0 * rng.normal()).round() as i64 };
        if day == 60 {
            log.append(rec(&log, met, "tag").int("day", day).unwrap().int("workload", 1).unwrap()).unwrap();
        } else if !(130..=140).contains(&day) {
            let r = rec(&log, met, "checkin")
                .int("day", day).unwrap()
                .int("mood", slider(0.6 + shift + 0.5 * m)).unwrap()
                .int("sleep_min", (390.0 + 40.0 * s).round() as i64).unwrap()
                .int("spo2", spo2).unwrap();
            log.append(r).unwrap();
        }
        log.append(rec(&log, met + DAY_MS / 2, "day_close").int("day", day).unwrap()).unwrap();
    }
    log
}

fn days_of<'a>(outs: &'a [(u64, Output)], f: impl Fn(&Output) -> Option<i64> + 'a) -> Vec<i64> {
    outs.iter().filter_map(|(_, o)| f(o)).collect()
}

#[test]
fn synthetic_mission_end_to_end() {
    let log = mission();
    let (_, outs) = Engine::replay(log.entries());
    let rejected: Vec<_> = outs.iter().filter(|(_, o)| matches!(o, Output::Rejected { .. } | Output::CompanionError { .. })).collect();
    assert!(rejected.is_empty(), "{rejected:?}");

    let safety = days_of(&outs, |o| match o {
        Output::Safety { day, hits, .. } => {
            assert_eq!(hits, &vec![Hit { id: 1, share: true }]);
            Some(*day)
        }
        _ => None,
    });
    assert_eq!(safety, vec![120], "kernel must fire exactly on the injected crossing");

    let mood_alarms = days_of(&outs, |o| match o {
        Output::Alarm { day, indicator, .. } if indicator == "mood" => Some(*day),
        _ => None,
    });
    let first_after = mood_alarms.iter().find(|d| **d >= 100).copied();
    assert!(first_after.is_some_and(|d| d <= 135), "mood shift not detected in time: {mood_alarms:?}");

    let miss = days_of(&outs, |o| match o {
        Output::MissingnessAlarm { day } => Some(*day),
        _ => None,
    });
    assert!(miss.iter().any(|d| (130..=141).contains(d)), "missed check-ins not detected: {miss:?}");
    assert!(!miss.contains(&60), "workload-tagged day must not count as missed");

    let recal = outs.iter().filter(|(_, o)| matches!(o, Output::Recalibrated { .. })).count();
    assert_eq!(recal, 1 + 150 / 7);

    let all_alarms = days_of(&outs, |o| match o {
        Output::Alarm { day, .. } => Some(*day),
        _ => None,
    });
    eprintln!(
        "mission: mood shift detected on day {:?} (onset 100); missingness alarm days {:?}; \
         false alarms before day 100: {}; total alarms {}",
        first_after,
        miss,
        all_alarms.iter().filter(|d| **d < 100).count(),
        all_alarms.len()
    );
}

/// N4 end to end: healthy crew members, no events. Counts alarm episodes per person-day across
/// many seeds, including every source of error the engine really has (estimated baselines,
/// guard band, recalibration). Heavy: run with `cargo test -- --ignored budget`.
#[test]
#[ignore]
fn false_alert_budget_holds_end_to_end_on_healthy_missions() {
    let seeds: u64 = std::env::var("ANCHOR_SEEDS").ok().and_then(|v| v.parse().ok()).unwrap_or(24);
    let days = 300i64;
    let (mut episodes, mut person_days) = (0usize, 0i64);
    for seed in 0..seeds {
        let mut log = configured_log();
        let mut rng = Rng::new(10_000 + seed);
        let mean = 0.6 + 0.4 * rng.normal();
        for _ in 0..30 {
            let r = rec(&log, 0, "preflight")
                .int("mood", slider(mean + 0.5 * rng.normal())).unwrap()
                .int("sleep_min", (390.0 + 40.0 * rng.normal()).round() as i64).unwrap()
                .int("spo2", (975.0 + 4.0 * rng.normal()).round() as i64).unwrap();
            log.append(r).unwrap();
        }
        let (mut m, mut s) = (0.0, 0.0);
        for day in 1..=days {
            m = 0.3 * m + (1.0f64 - 0.09).sqrt() * rng.normal();
            s = 0.3 * s + (1.0f64 - 0.09).sqrt() * rng.normal();
            let r = rec(&log, day * DAY_MS, "checkin")
                .int("day", day).unwrap()
                .int("mood", slider(mean + 0.5 * m)).unwrap()
                .int("sleep_min", (390.0 + 40.0 * s).round() as i64).unwrap()
                .int("spo2", (975.0 + 4.0 * rng.normal()).round() as i64).unwrap();
            log.append(r).unwrap();
            log.append(rec(&log, day * DAY_MS + 1, "day_close").int("day", day).unwrap()).unwrap();
        }
        let outs = Engine::replay(log.entries()).1;
        if std::env::var("ANCHOR_DIAG").is_ok() {
            for ind in ["mood", "sleep_min", "spo2"] {
                for (lo, hi) in [(1, 90), (91, days)] {
                    let z: Vec<f64> = outs.iter().filter_map(|(_, o)| match o {
                        Output::DayStat { day, indicator, z, .. } if indicator == ind && (lo..=hi).contains(day) => Some(*z),
                        _ => None,
                    }).collect();
                    let n = z.len() as f64;
                    let mu = z.iter().sum::<f64>() / n;
                    let var = z.iter().map(|v| (v - mu).powi(2)).sum::<f64>() / n;
                    let r1 = z.windows(2).map(|w| (w[0] - mu) * (w[1] - mu)).sum::<f64>() / (n * var);
                    let al = outs.iter().filter(|(_, o)| matches!(o, Output::Alarm { day, indicator, .. } if indicator == ind && (lo..=hi).contains(day))).count();
                    eprintln!("DIAG seed {seed} {ind:9} days {lo:3}-{hi:3}: mean {mu:+.2} var {var:.2} r1 {r1:+.2} alarms {al}");
                }
            }
        }
        episodes += outs.iter().filter(|(_, o)| matches!(o, Output::Alarm { .. } | Output::MissingnessAlarm { .. })).count();
        person_days += days;
    }
    let days_per_alert = person_days as f64 / episodes.max(1) as f64;
    eprintln!("healthy missions: {episodes} alarm episodes in {person_days} person-days = one per {days_per_alert:.1} days (budget 30)");
    assert!(days_per_alert >= 25.0, "false-alert budget exceeded beyond Monte-Carlo tolerance");
}

/// Mood settles `shift` logit lower from day 20 and stays there (-0.75 = 1.5 pre-flight SD;
/// adaptation or decline: only the person can say). `extra(log, day)` may append records on
/// the open day before its check-in.
fn shifted_mission(days: i64, shift_by: f64, mut extra: impl FnMut(&mut Log, i64)) -> Log {
    let mut log = configured_log();
    let mut rng = Rng::new(7);
    for _ in 0..30 {
        let r = rec(&log, 0, "preflight")
            .int("mood", slider(0.6 + 0.5 * rng.normal())).unwrap()
            .int("sleep_min", (390.0 + 40.0 * rng.normal()).round() as i64).unwrap()
            .int("spo2", (975.0 + 4.0 * rng.normal()).round() as i64).unwrap();
        log.append(r).unwrap();
    }
    let mut m = 0.0;
    for day in 1..=days {
        let met = day * DAY_MS;
        m = 0.3 * m + (1.0f64 - 0.09).sqrt() * rng.normal();
        let shift = if day >= 20 { shift_by } else { 0.0 };
        extra(&mut log, day);
        let r = rec(&log, met, "checkin")
            .int("day", day).unwrap()
            .int("mood", slider(0.6 + shift + 0.5 * m)).unwrap()
            .int("sleep_min", (390.0 + 40.0 * rng.normal()).round() as i64).unwrap()
            .int("spo2", (975.0 + 4.0 * rng.normal()).round() as i64).unwrap();
        log.append(r).unwrap();
        log.append(rec(&log, met + DAY_MS / 2, "day_close").int("day", day).unwrap()).unwrap();
    }
    log
}

fn mood_events(outs: &[(u64, Output)]) -> Vec<String> {
    outs.iter()
        .filter_map(|(_, o)| match o {
            Output::Alarm { day, indicator, .. } if indicator == "mood" => Some(format!("alarm@{day}")),
            Output::AlarmCleared { day, indicator } if indicator == "mood" => Some(format!("cleared@{day}")),
            Output::AnchorRunPrompt { day, indicator } if indicator == "mood" => Some(format!("run@{day}")),
            Output::DriftPrompt { day, indicator, .. } if indicator == "mood" => Some(format!("drift@{day}")),
            Output::Reanchored { day, .. } => Some(format!("reanchored@{day}")),
            Output::ReanchorReverted { day, .. } => Some(format!("reverted@{day}")),
            Output::Rejected { reason, .. } => Some(format!("rejected:{reason}")),
            _ => None,
        })
        .collect()
}

fn reanchor(log: &mut Log, day: i64, kind: &str, key: &str) {
    let r = rec(log, day * DAY_MS, kind).int("day", day).unwrap().text("key_txt", key).unwrap();
    log.append(r).unwrap();
}

/// ADR-0007 §5-6: an unresolved sustained shift gives one alarm and one prompt, never flaps,
/// and does not inflate the threshold (before the fix: clear/re-alarm at 77/83, h 4 -> 20).
#[test]
fn sustained_shift_latches_once_and_keeps_threshold() {
    let outs = Engine::replay(shifted_mission(120, -0.75, |_, _| {}).entries()).1;
    let ev = mood_events(&outs);
    assert_eq!(ev.iter().filter(|e| e.starts_with("alarm")).count(), 1, "{ev:?}");
    assert!(!ev.iter().any(|e| e.starts_with("cleared")), "{ev:?}");
    assert_eq!(ev.iter().filter(|e| e.starts_with("run")).count(), 1, "{ev:?}");
    let max_h = outs.iter().filter_map(|(_, o)| match o {
        Output::Recalibrated { bank, .. } => Some(bank.h),
        _ => None,
    });
    assert!(max_h.fold(0.0, f64::max) < 6.0, "episode days leaked into calibration");
}

#[test]
fn reanchoring_accepts_a_level_and_revert_restores_the_reference() {
    let log = shifted_mission(130, -0.75, |log, day| match day {
        30 | 41 | 60 => reanchor(log, day, "reanchor", "mood"),
        40 => {
            reanchor(log, day, "reanchor", "spo2");
            reanchor(log, day, "reanchor", "mood");
        }
        100 => reanchor(log, day, "reanchor_revert", "mood"),
        _ => {}
    });
    let outs = Engine::replay(log.entries()).1;
    let ev = mood_events(&outs);
    let at = |s: &str| ev.iter().position(|e| e == s).unwrap_or_else(|| panic!("{s} missing in {ev:?}"));
    let (alarm, re, rev) = (at("alarm@23"), at("reanchored@40"), at("reverted@100"));
    assert!(alarm < at("rejected:alarm episode younger than 14 days"));
    assert!(at("rejected:re-anchoring forbidden for safety-line indicators") < re);
    assert_eq!(ev[re + 1], "cleared@40", "accepting the level ends the episode");
    assert_eq!(ev.iter().filter(|e| *e == "rejected:re-anchoring limited to once per 30 days").count(), 2);
    assert!(!ev[re..rev].iter().any(|e| e.starts_with("run")), "{ev:?}");
    assert!(ev[rev..].contains(&"run@105".to_string()), "the pre-flight reference is back: {ev:?}");
    // The accepted level is the new reference: mean z ~ 0 (it was ~ -1.5 before), and the
    // deviation is visible again after the revert. Single alarms from AR noise stay possible.
    let mean_z = |lo: i64, hi: i64| {
        let z: Vec<f64> = outs.iter().filter_map(|(_, o)| match o {
            Output::DayStat { day, indicator, z, .. } if indicator == "mood" && (lo..=hi).contains(day) => Some(*z),
            _ => None,
        }).collect();
        z.iter().sum::<f64>() / z.len() as f64
    };
    assert!(mean_z(41, 99).abs() < 0.3, "accepted period mean z {}", mean_z(41, 99));
    assert!(mean_z(101, 115) < -0.8, "after revert mean z {}", mean_z(101, 115));
}

#[test]
fn reanchoring_refuses_a_level_too_far_from_preflight() {
    // -1.25 logit = 2.5 pre-flight SD in the harmful direction.
    let log = shifted_mission(50, -1.25, |log, day| if day == 45 { reanchor(log, day, "reanchor", "mood") });
    let ev = mood_events(&Engine::replay(log.entries()).1);
    assert!(ev.contains(&"rejected:new level too far from pre-flight to accept alone".to_string()), "{ev:?}");
    assert!(!ev.iter().any(|e| e.starts_with("reanchored")));
}

const HS_TABLE: &str =
    "t1|1,spo2,below,900,phys,1;2,sleep_min,below,120,phys,1;3,cssrs_plan,above,0,high,0;4,cssrs_intent,above,0,high,1";

fn screen(log: &Log, day: i64, plan: i64, intent: i64) -> Record {
    rec(log, day * DAY_MS, "screen").int("day", day).unwrap().int("cssrs_plan", plan).unwrap().int("cssrs_intent", intent).unwrap()
}

#[test]
fn screener_is_kernel_only_and_always_offers_private_support() {
    let log = configured_log_with(HS_TABLE);
    let mut eng = Engine::replay(log.entries()).0;
    let entry = |r: Record| Entry { record: r, prev_hash: String::new(), hash: "0".repeat(64) };
    // Companion state = everything except the outbox, which a consented share legitimately fills.
    // Values awaiting the kernel-B comparison are not companion state either (ADR-0009 §7).
    let companion = |e: &Engine| format!("{:?}", Engine { outbox: Outbox::default(), checkable: vec![], ..e.clone() });
    let before = companion(&eng);
    let out = eng.apply(&entry(screen(&log, 1, 1, 0)));
    assert_eq!(out, vec![
        Output::Safety { day: 1, indicator: "cssrs_plan".into(), value: 1, hits: vec![Hit { id: 3, share: false }] },
        Output::PrivateSupport { day: 1, items: vec!["cssrs_plan".into()] },
    ]);
    let out = eng.apply(&entry(screen(&log, 1, 0, 1)));
    assert!(matches!(&out[0], Output::Safety { hits, .. } if hits == &vec![Hit { id: 4, share: true }]));
    // Consented imminent-risk item: the line id only, never the answers (ADR-0008 §2).
    assert_eq!(out[1], Output::ShareQueued { share: 0, class: "safety", to: Some("surgeon".into()), payload: "share=0;day=1;lines=4".into() });
    assert!(eng.apply(&entry(screen(&log, 1, 0, 0))).is_empty());
    assert_eq!(companion(&eng), before, "a screen must not touch companion state");

    // Private support does not depend on the kernel: two damaged copies still get it.
    let t = eng.table.as_mut().unwrap();
    t.copies[0].push('x');
    t.copies[1].push('x');
    let out = eng.apply(&entry(screen(&log, 1, 1, 0)));
    assert!(matches!(out[0], Output::SafetyCheckFailed { .. }) && matches!(out[2], Output::PrivateSupport { .. }), "{out:?}");
    assert!(!out.iter().any(|o| matches!(o, Output::ShareQueued { .. })), "a screen never shares a fault: {out:?}");

    let eng = Engine::replay(log.entries()).0;
    for bad in [
        rec(&log, 0, "screen").int("day", 1).unwrap().int("mood", 1).unwrap(),
        rec(&log, 0, "screen").int("day", 1).unwrap().int("spo2", 1).unwrap(),
        rec(&log, 0, "screen").int("day", 1).unwrap().int("cssrs_plan", 2).unwrap(),
        rec(&log, 0, "screen").int("day", 1).unwrap(),
        screen(&log, 2, 1, 0),
    ] {
        assert!(eng.validate(&bad).is_err(), "accepted {bad:?}");
    }
}

#[test]
fn consent_changes_one_flag_immediately_and_the_table_is_otherwise_fixed() {
    let mut log = configured_log_with(HS_TABLE);
    let consent = |log: &Log, line: i64, share: i64| rec(log, DAY_MS, "consent").int("day", 1).unwrap().int("line", line).unwrap().int("share", share).unwrap();
    log.append(consent(&log, 3, 1)).unwrap(); // grant plan
    log.append(consent(&log, 4, 0)).unwrap(); // withdraw intent
    log.append(screen(&log, 1, 1, 1)).unwrap();
    let (eng, outs) = Engine::replay(log.entries());
    let expected = HS_TABLE.replace("plan,above,0,high,0", "plan,above,0,high,1").replace("intent,above,0,high,1", "intent,above,0,high,0");
    let changes: Vec<&Output> = outs.iter().map(|(_, o)| o).filter(|o| matches!(o, Output::ConsentChanged { .. })).collect();
    assert!(matches!(changes[0], Output::ConsentChanged { line: 3, share: true, .. }));
    let Output::ConsentChanged { line: 4, share: false, table, digest, .. } = changes[1] else { panic!("{outs:?}") };
    assert_eq!((table.as_str(), digest.clone()), (expected.as_str(), sha256_hex(expected.as_bytes())));
    let hits: Vec<Vec<Hit>> = outs.iter().filter_map(|(_, o)| match o {
        Output::Safety { hits, .. } => Some(hits.clone()),
        _ => None,
    }).collect();
    // Fields are evaluated in key order: cssrs_intent before cssrs_plan.
    assert_eq!(hits, vec![vec![Hit { id: 4, share: false }], vec![Hit { id: 3, share: true }]]);
    for (bad, why) in [
        (consent(&log, 1, 0), "physiological lines always share"),
        (consent(&log, 3, 1), "consent unchanged"),
        (consent(&log, 99, 1), "no such line"),
        (consent(&log, 3, 2), "share must be 0 or 1"),
    ] {
        assert_eq!(eng.validate(&bad), Err(why));
    }

    let mut log = configured_log_with(HS_TABLE);
    log.append(rec(&log, DAY_MS, "day_close").int("day", 1).unwrap()).unwrap();
    let eng = Engine::replay(log.entries()).0;
    let t = rec(&log, DAY_MS, "safety_table").text("table_txt", "t1|").unwrap().text("digest_txt", &sha256_hex(b"t1|")).unwrap();
    assert_eq!(eng.validate(&t), Err("safety table is fixed after first day_close"));
    let mut bare = Log::new();
    bare.append(rec(&bare, 0, "setup").int("budget_days", 30).unwrap()).unwrap();
    let close = rec(&bare, DAY_MS, "day_close").int("day", 1).unwrap();
    assert_eq!(Engine::replay(bare.entries()).0.validate(&close), Err("no safety table before first day_close"));
}

#[test]
fn replay_is_deterministic_and_matches_incremental_application() {
    let log = mission();
    let (_, a) = Engine::replay(log.entries());
    let (_, b) = Engine::replay(log.entries());
    assert_eq!(a, b);
    let mut eng = Engine::new();
    let incremental: Vec<(u64, Output)> =
        log.entries().iter().flat_map(|e| eng.apply(e).into_iter().map(move |o| (e.record.seq, o))).collect();
    assert_eq!(a, incremental);
}

#[test]
fn replay_after_storage_repair_gives_identical_outputs() {
    let log = mission();
    let good = log.lines();
    let (mut ca, mut cb) = (good.clone(), good.clone());
    let flip = |copy: &mut Vec<String>, idx: usize| {
        let mut bytes = copy[idx].clone().into_bytes();
        let p = bytes.len() - 10;
        bytes[p] ^= 0x04;
        copy[idx] = String::from_utf8(bytes).unwrap();
    };
    flip(&mut ca, 50);
    flip(&mut cb, 200);
    flip(&mut ca, 300);
    let fixed = repair(&ca, &cb).unwrap();
    let restored = Log::load(&fixed.lines).unwrap();
    assert_eq!(Engine::replay(restored.entries()).1, Engine::replay(log.entries()).1);
}

#[test]
fn kernel_output_survives_companion_failure() {
    let mut log = configured_log();
    log.append(rec(&log, DAY_MS, "checkin").int("day", 1).unwrap().int("spo2", 850).unwrap()).unwrap();
    let mut eng = Engine::new();
    for e in &log.entries()[..log.entries().len() - 1] {
        eng.apply(e);
    }
    eng.inject_companion_fault = true;
    let out = eng.apply(log.entries().last().unwrap());
    assert!(matches!(&out[0], Output::Safety { value: 850, .. }), "{out:?}");
    assert!(matches!(out.last(), Some(Output::CompanionError { .. })));
}

#[test]
fn trust_boundary_rejects_bad_records() {
    let mut log = configured_log();
    let eng = Engine::replay(log.entries()).0;
    let bad = [
        rec(&log, 0, "checkin").int("day", 1).unwrap().int("unknown", 1).unwrap(),
        rec(&log, 0, "checkin").int("day", 2).unwrap().int("mood", 500).unwrap(),
        rec(&log, 0, "checkin").int("day", 1).unwrap().int("mood", 1001).unwrap(),
        rec(&log, 0, "checkin").int("day", 1).unwrap(),
        rec(&log, 0, "safety_table").text("table_txt", TABLE).unwrap().text("digest_txt", "00").unwrap(),
        rec(&log, 0, "mystery").int("day", 1).unwrap(),
    ];
    for r in &bad {
        assert!(eng.validate(r).is_err(), "accepted {r:?}");
    }
    log.append(rec(&log, DAY_MS, "day_close").int("day", 1).unwrap()).unwrap();
    let eng = Engine::replay(log.entries()).0;
    let late = rec(&log, DAY_MS, "setup").int("budget_days", 30).unwrap();
    assert_eq!(eng.validate(&late), Err("configuration after first day_close"));
    // An invalid record already in a log is reported, never skipped silently.
    log.append(rec(&log, DAY_MS, "checkin").int("day", 9).unwrap().int("mood", 1).unwrap()).unwrap();
    let outs = Engine::replay(log.entries()).1;
    assert!(matches!(outs.last(), Some((_, Output::Rejected { .. }))));
}


// ---------------------------------------------------------------- ADR-0008: transmission

/// Mission time of the latest entry, so ad-hoc records never go back in time.
fn now(log: &Log) -> i64 {
    log.entries().last().map_or(0, |e| e.record.met_ms)
}

/// Append the record only if the engine accepts it, applying it incrementally; returns the outputs.
fn push(log: &mut Log, eng: &mut Engine, make: impl FnOnce(&Log) -> Record) -> Result<Vec<Output>, &'static str> {
    let r = make(log);
    eng.validate(&r)?;
    log.append(r).unwrap();
    Ok(eng.apply(log.entries().last().unwrap()))
}

fn int_rec(log: &Log, kind: &str, fields: &[(&str, i64)]) -> Record {
    fields.iter().fold(rec(log, now(log), kind), |r, (k, v)| r.int(k, *v).unwrap())
}

fn exclude(log: &Log, who: &str, on: i64) -> Record {
    rec(log, now(log), "exclude").text("key_txt", who).unwrap().int("on", on).unwrap()
}

#[test]
fn routing_honours_exclusions_and_reachability() {
    let mut log = configured_log();
    let mut eng = Engine::replay(log.entries()).0;
    let out = push(&mut log, &mut eng, |log| int_rec(log, "checkin", &[("day", 1), ("spo2", 850)])).unwrap();
    assert_eq!(out[1], Output::ShareQueued {
        share: 0, class: "safety", to: Some("surgeon".into()), payload: "share=0;day=1;lines=1;value=850".into(),
    });
    // Conjunction: the safety notice goes to the CMO on board instead of waiting (F5).
    assert_eq!(push(&mut log, &mut eng, |log| int_rec(log, "link", &[("up", 0)])).unwrap(),
        vec![Output::ShareRerouted { share: 0, to: "cmo".into() }]);
    let out = push(&mut log, &mut eng, |log| exclude(log, "cmo", 1)).unwrap();
    assert_eq!(out[1], Output::ShareUndeliverable { share: 0, class: "safety" });
    let out = push(&mut log, &mut eng, |log| exclude(log, "surgeon", 1)).unwrap();
    assert!(out.contains(&Output::RouteEmpty { class: "safety" }), "{out:?}");
    assert!(eng.transmittable().is_empty());
    assert_eq!(push(&mut log, &mut eng, |log| int_rec(log, "sent", &[("share", 0)])), Err("share has no eligible recipient"));
    let out = push(&mut log, &mut eng, |log| exclude(log, "cmo", 0)).unwrap();
    assert_eq!(out[1], Output::ShareRerouted { share: 0, to: "cmo".into() });
    assert_eq!(eng.transmittable(), vec![(0, "cmo", "share=0;day=1;lines=1;value=850")]);
    assert_eq!(push(&mut log, &mut eng, |log| int_rec(log, "sent", &[("share", 0)])).unwrap(),
        vec![Output::ShareSent { share: 0, to: "cmo".into() }]);
    assert_eq!(push(&mut log, &mut eng, |log| int_rec(log, "sent", &[("share", 0)])), Err("share already sent"));

    // A PPC request waits for its ground recipient instead of being re-routed to the crew.
    let out = push(&mut log, &mut eng, |log| int_rec(log, "ppc_request", &[("day", 1)])).unwrap();
    assert_eq!(out, vec![Output::ShareQueued { share: 1, class: "ppc", to: Some("bhp".into()), payload: "share=1;day=1;ppc=1".into() }]);
    assert_eq!(push(&mut log, &mut eng, |log| int_rec(log, "sent", &[("share", 1)])), Err("recipient excluded or unreachable"));
    assert!(push(&mut log, &mut eng, |log| int_rec(log, "link", &[("up", 1)])).unwrap().is_empty());
    assert!(push(&mut log, &mut eng, |log| int_rec(log, "sent", &[("share", 1)])).is_ok());
    let out = push(&mut log, &mut eng, |log| int_rec(log, "crew_request", &[("day", 1)])).unwrap();
    assert!(matches!(&out[0], Output::ShareQueued { to: Some(t), .. } if t == "crewmate"));
    assert_eq!(push(&mut log, &mut eng, |log| exclude(log, "cmo", 0)), Err("exclusion unchanged"));
    // Incremental application and full replay agree on the whole transmission history (I5).
    let incremental: Vec<Output> = {
        let mut e = Engine::new();
        log.entries().iter().flat_map(|x| e.apply(x)).collect()
    };
    assert_eq!(incremental, Engine::replay(log.entries()).1.into_iter().map(|(_, o)| o).collect::<Vec<_>>());

    // Routing is fixed in flight.
    push(&mut log, &mut eng, |log| rec(log, DAY_MS, "day_close").int("day", 1).unwrap()).unwrap();
    let route = |log: &Log, c: &str, t: &str| rec(log, 0, "route").text("class_txt", c).unwrap().text("to_txt", t).unwrap();
    assert_eq!(eng.validate(&route(&log, "summary", "bhp")), Err("routing is fixed after first day_close"));
}

#[test]
fn routing_configuration_is_checked() {
    let route = |log: &Log, c: &str, t: &str| rec(log, 0, "route").text("class_txt", c).unwrap().text("to_txt", t).unwrap();
    let mut cfg = Log::new();
    for (k, c) in [("g", 0), ("c", 1)] {
        let r = rec(&cfg, 0, "recipient").text("key_txt", k).unwrap().int("crew", c).unwrap();
        cfg.append(r).unwrap();
    }
    let eng = Engine::replay(cfg.entries()).0;
    assert_eq!(eng.validate(&route(&cfg, "ppc", "c")), Err("recipient kind not allowed for this class"));
    assert_eq!(eng.validate(&route(&cfg, "crew", "g")), Err("recipient kind not allowed for this class"));
    assert_eq!(eng.validate(&route(&cfg, "gossip", "g")), Err("unknown share class"));
    assert_eq!(eng.validate(&route(&cfg, "safety", "nobody")), Err("unknown recipient"));
    let dup = rec(&cfg, 0, "recipient").text("key_txt", "g").unwrap().int("crew", 0).unwrap();
    assert_eq!(eng.validate(&dup), Err("invalid or duplicate recipient"));

    // A single-recipient safety route would let one exclusion empty it: refused at first close.
    let full = configured_log();
    let mut eng = Engine::new();
    for e in full.entries() {
        let skip = e.record.kind == "route" && text(&e.record, "class_txt") == Some("safety") && text(&e.record, "to_txt") == Some("cmo");
        if !skip {
            eng.apply(e);
        }
    }
    let close = rec(&full, DAY_MS, "day_close").int("day", 1).unwrap();
    assert_eq!(eng.validate(&close), Err("safety route needs at least 2 recipients"));
}

#[test]
fn summaries_are_private_by_default_and_withdrawal_cancels_unsent() {
    let mut log = configured_log();
    let mut eng = Engine::replay(log.entries()).0;
    let share = |log: &Log, on: i64| rec(log, now(log), "share_category").text("key_txt", "mood").unwrap().int("on", on).unwrap();
    let mut queued = vec![];
    for day in 1..=21 {
        if day == 3 {
            push(&mut log, &mut eng, |log| share(log, 1)).unwrap();
        }
        if day == 6 {
            push(&mut log, &mut eng, |log| int_rec(log, "link", &[("up", 0)])).unwrap();
        }
        if day == 9 {
            let out = push(&mut log, &mut eng, |log| share(log, 0)).unwrap();
            assert_eq!(out, vec![Output::ShareCancelled { share: 0 }], "unsent summary must be cancelled");
        }
        let r = rec(&log, day * DAY_MS, "checkin").int("day", day).unwrap().int("mood", if day == 4 { 500 } else { 650 }).unwrap();
        push(&mut log, &mut eng, |_| r).unwrap();
        let out = push(&mut log, &mut eng, |log| rec(log, day * DAY_MS, "day_close").int("day", day).unwrap()).unwrap();
        queued.extend(out.into_iter().filter_map(|o| match o {
            Output::ShareQueued { class: "summary", payload, .. } => Some(payload),
            _ => None,
        }));
    }
    // Only the week while consent was on; coverage and mean z, never raw values.
    assert_eq!(queued.len(), 1, "{queued:?}");
    assert!(queued[0].starts_with("share=0;ind=mood;day=7;days=7;mean_z_e3="), "{}", queued[0]);
    assert!(queued[0].ends_with(";alarm=0"));
}

/// Property test for I2, I6, I10 and class/recipient compatibility under random operations:
/// exclusions, link loss, requests, screens, safety crossings and transmissions in any order.
#[test]
fn transmission_invariants_hold_under_random_operations() {
    let recipients: BTreeMap<&str, bool> = [("surgeon", false), ("bhp", false), ("cmo", true), ("crewmate", true)].into();
    for seed in 0..12u64 {
        let mut rng = Rng::new(900 + seed);
        let mut log = configured_log_with(HS_TABLE);
        let mut eng = Engine::replay(log.entries()).0;
        if seed % 2 == 0 {
            push(&mut log, &mut eng, |log| rec(log, 0, "share_category").text("key_txt", "mood").unwrap().int("on", 1).unwrap()).unwrap();
        }
        for day in 1..=35i64 {
            let r = rec(&log, day * DAY_MS, "checkin").int("day", day).unwrap()
                .int("spo2", 860 + rng.below(140) as i64).unwrap().int("mood", 300 + rng.below(500) as i64).unwrap();
            push(&mut log, &mut eng, |_| r).unwrap();
            for _ in 0..rng.below(5) {
                let who = *recipients.keys().nth(rng.below(4) as usize).unwrap();
                let r = match rng.below(7) {
                    0 | 1 => exclude(&log, who, rng.below(2) as i64),
                    2 => int_rec(&log, "link", &[("up", rng.below(2) as i64)]),
                    3 => int_rec(&log, "ppc_request", &[("day", day)]),
                    4 => int_rec(&log, "crew_request", &[("day", day)]),
                    5 => screen(&log, day, rng.below(2) as i64, rng.below(2) as i64),
                    _ => match eng.transmittable().first() {
                        Some((id, _, _)) => int_rec(&log, "sent", &[("share", *id as i64)]),
                        None => continue,
                    },
                };
                let _ = push(&mut log, &mut eng, |_| r); // a refused record is simply not appended
            }
            // A buggy transport acknowledging an arbitrary share must never get through.
            let _ = push(&mut log, &mut eng, |log| int_rec(log, "sent", &[("share", rng.below(40) as i64)]));
            push(&mut log, &mut eng, |log| rec(log, day * DAY_MS + 1, "day_close").int("day", day).unwrap()).unwrap();
        }
        let (_, outs) = Engine::replay(log.entries());
        // Link changes often produce no output, so the link state is read from the records.
        let links: Vec<(u64, bool)> = log.entries().iter()
            .filter(|e| e.record.kind == "link")
            .map(|e| (e.record.seq, e.record.get_int("up") == Some(1)))
            .collect();
        let link_at = |seq: u64| links.iter().take_while(|(s, _)| *s <= seq).last().is_some_and(|l| l.1);
        let mut excluded = std::collections::BTreeSet::new();
        let mut class_of: BTreeMap<u64, &str> = BTreeMap::new();
        let mut sent = 0;
        for (seq, o) in &outs {
            assert!(!matches!(o, Output::Rejected { .. } | Output::CompanionError { .. }), "{o:?}");
            let link_up = link_at(*seq);
            match o {
                Output::ExclusionChanged { recipient, excluded: on } => {
                    if *on { excluded.insert(recipient.clone()) } else { excluded.remove(recipient) };
                }
                Output::ShareQueued { share, class, to, payload } => {
                    class_of.insert(*share, class);
                    let allowed = ShareClass::parse(class).unwrap().allowed_keys();
                    for kv in payload.split(';') {
                        let (k, v) = kv.split_once('=').unwrap();
                        assert!(allowed.contains(&k), "I2: key {k} in {payload}");
                        assert!(k == "ind" || v.split('+').all(|n| n.parse::<i64>().is_ok()), "I2: {payload}");
                    }
                    if let Some(t) = to {
                        assert!(!excluded.contains(t), "I10 at queue: {o:?}");
                    }
                }
                Output::ShareRerouted { to, .. } => assert!(!excluded.contains(to), "I10 at reroute: {o:?}"),
                Output::ShareSent { share, to } => {
                    let class = class_of.get(share).unwrap_or_else(|| panic!("I6: sent before queued: {o:?}"));
                    assert!(!excluded.contains(to), "I10 at send: {o:?}");
                    assert!(recipients[to.as_str()] || link_up, "sent to unreachable ground: {o:?}");
                    match *class {
                        "ppc" => assert!(!recipients[to.as_str()]),
                        "crew" => assert!(recipients[to.as_str()]),
                        _ => {}
                    }
                    sent += 1;
                }
                _ => {}
            }
        }
        assert!(sent > 5, "seed {seed}: too few transmissions exercised ({sent})");
    }
}
