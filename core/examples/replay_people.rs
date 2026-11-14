//! Real-data validation (docs/VALIDATION.md): replay per-person wire logs through the unmodified
//! engine and print one JSON line of metrics per person.
//! Usage: cargo run --release --example replay_people -- <people.txt>   (written by analysis/lifesnaps/prepare.py)

use anchor_core::engine::Output;
use anchor_core::session::Session;
use std::collections::BTreeMap;
use std::time::Instant;

/// Harmful direction per indicator, as configured in prepare.py.
fn harm(key: &str) -> f64 {
    if key == "rhr_d1" { 1.0 } else { -1.0 }
}

#[derive(Default)]
struct Ind {
    days: u32,
    in_band: u32,
    alarms: u32,
    naive_days: u32,
    naive_runs: u32,
    last_naive: i64,
}

fn main() {
    let path = std::env::args().nth(1).expect("usage: replay_people <people.txt>");
    let text = std::fs::read_to_string(path).expect("read input");
    for block in text.split("\n\n").filter(|b| b.trim_start().starts_with('#')) {
        let mut lines = block.lines();
        let id = lines.next().expect("header").trim_start_matches("# ").to_string();
        let mut s = Session::new();
        let mut inds: BTreeMap<String, Ind> = BTreeMap::new();
        let (mut rejected, mut errors, mut miss_alarms, mut days, mut submits) = (0u32, 0u32, 0u32, 0i64, 0u32);
        let t0 = Instant::now();
        for wire in lines.filter(|l| !l.is_empty()) {
            submits += 1;
            let Ok((_, outs)) = s.submit_wire(wire) else {
                rejected += 1;
                continue;
            };
            for o in outs {
                match o {
                    Output::DayStat { day, indicator, z, y, lo, hi, .. } => {
                        days = days.max(day);
                        let i = inds.entry(indicator.clone()).or_default();
                        i.days += 1;
                        i.in_band += (lo <= y && y <= hi) as u32;
                        // Naive comparator: one day beyond 2 SD in the harmful direction is an alert.
                        if harm(&indicator) * z >= 2.0 {
                            i.naive_days += 1;
                            i.naive_runs += (i.last_naive != day - 1) as u32;
                            i.last_naive = day;
                        }
                    }
                    Output::Alarm { indicator, .. } => inds.entry(indicator).or_default().alarms += 1,
                    Output::MissingnessAlarm { .. } => miss_alarms += 1,
                    Output::CompanionError { .. } | Output::KernelUnavailable { .. } => errors += 1,
                    _ => {}
                }
            }
        }
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        let ind_json: Vec<String> = inds
            .iter()
            .map(|(k, i)| {
                format!(
                    "\"{k}\":{{\"days\":{},\"in_band\":{},\"alarms\":{},\"naive_days\":{},\"naive_runs\":{}}}",
                    i.days, i.in_band, i.alarms, i.naive_days, i.naive_runs
                )
            })
            .collect();
        println!(
            "{{\"id\":\"{id}\",\"mission_days\":{days},\"records\":{submits},\"rejected\":{rejected},\"errors\":{errors},\"miss_alarms\":{miss_alarms},\"ms\":{ms:.1},\"ind\":{{{}}}}}",
            ind_json.join(",")
        );
    }
}
