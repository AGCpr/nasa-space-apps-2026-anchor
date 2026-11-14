//! WebAssembly boundary (ADR-0009 §1-3). Five calls: `load`, `submit`, `transmittable`, `demo`,
//! `preflight`. Strings in (log field grammar), JSON out. All logic lives in `anchor-core`; the
//! `unsafe` here is only pointer handling at the linear-memory edge.

use anchor_core::engine::{Engine, Output};
use anchor_core::log::Value;
use anchor_core::session::{Event, Session};
use anchor_core::synth;
use std::cell::RefCell;
use std::fmt::Write;

// ---------- JSON (hand-written, exhaustive: a new Output variant fails to compile here) ----------

fn q(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => write!(o, "\\u{:04x}", c as u32).expect("string write"),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn num(v: f64) -> String {
    if v.is_finite() { format!("{v}") } else { "null".into() }
}

fn opt(v: Option<f64>) -> String {
    v.map_or("null".into(), num)
}

pub fn output_json(eng: &Engine, o: &Output) -> String {
    let ind = |i: &str| format!("\"indicator\":{}", q(i));
    let show = |i: &str, y: f64| opt(eng.display(i, y));
    match o {
        Output::Safety { day, indicator, value, hits } => {
            let h: Vec<String> = hits.iter().map(|h| format!("{{\"id\":{},\"share\":{}}}", h.id, h.share)).collect();
            format!("{{\"t\":\"Safety\",\"day\":{day},{},\"value\":{value},\"hits\":[{}]}}", ind(indicator), h.join(","))
        }
        Output::SafetyCheckFailed { day, indicator, value } => {
            format!("{{\"t\":\"SafetyCheckFailed\",\"day\":{day},{},\"value\":{value}}}", ind(indicator))
        }
        Output::KernelUnavailable { day, indicator, value } => {
            format!("{{\"t\":\"KernelUnavailable\",\"day\":{day},{},\"value\":{value}}}", ind(indicator))
        }
        Output::OfferExtraSample { day, indicator } => format!("{{\"t\":\"OfferExtraSample\",\"day\":{day},{}}}", ind(indicator)),
        Output::DayStat { day, indicator, z, statistic, y, lo, hi, anchor } => format!(
            "{{\"t\":\"DayStat\",\"day\":{day},{},\"z\":{},\"statistic\":{},\"y\":{},\"lo\":{},\"hi\":{},\"anchor\":{}}}",
            ind(indicator),
            num(*z),
            num(*statistic),
            show(indicator, *y),
            show(indicator, *lo),
            show(indicator, *hi),
            show(indicator, *anchor)
        ),
        Output::Alarm { day, indicator, statistic, h } => {
            format!("{{\"t\":\"Alarm\",\"day\":{day},{},\"statistic\":{},\"h\":{}}}", ind(indicator), num(*statistic), num(*h))
        }
        Output::AlarmCleared { day, indicator } => format!("{{\"t\":\"AlarmCleared\",\"day\":{day},{}}}", ind(indicator)),
        Output::MissingnessAlarm { day } => format!("{{\"t\":\"MissingnessAlarm\",\"day\":{day}}}"),
        Output::MissingnessCleared { day } => format!("{{\"t\":\"MissingnessCleared\",\"day\":{day}}}"),
        Output::DriftPrompt { day, indicator, drift } => format!(
            "{{\"t\":\"DriftPrompt\",\"day\":{day},{},\"n\":{},\"sen_slope\":{},\"z\":{}}}",
            ind(indicator),
            drift.n,
            num(drift.sen_slope),
            num(drift.z)
        ),
        Output::AnchorRunPrompt { day, indicator } => format!("{{\"t\":\"AnchorRunPrompt\",\"day\":{day},{}}}", ind(indicator)),
        Output::Calibrating { day } => format!("{{\"t\":\"Calibrating\",\"day\":{day}}}"),
        Output::Recalibrated { day, bank, missingness } => format!(
            "{{\"t\":\"Recalibrated\",\"day\":{day},\"h\":{},\"arl0\":{},\"seed\":{},\"miss_h\":{}}}",
            num(bank.h),
            num(bank.arl0),
            q(&bank.seed.to_string()),
            opt(missingness.map(|m| m.h))
        ),
        Output::Rejected { seq, reason } => format!("{{\"t\":\"Rejected\",\"seq\":{seq},\"reason\":{}}}", q(reason)),
        Output::CompanionError { day, reason } => format!("{{\"t\":\"CompanionError\",\"day\":{day},\"reason\":{}}}", q(reason)),
        Output::PrivateSupport { day, items } => {
            let i: Vec<String> = items.iter().map(|s| q(s)).collect();
            format!("{{\"t\":\"PrivateSupport\",\"day\":{day},\"items\":[{}]}}", i.join(","))
        }
        Output::ConsentChanged { day, line, share, table, digest } => format!(
            "{{\"t\":\"ConsentChanged\",\"day\":{day},\"line\":{line},\"share\":{share},\"table\":{},\"digest\":{}}}",
            q(table),
            q(digest)
        ),
        Output::Reanchored { day, indicator, from, to } => format!(
            "{{\"t\":\"Reanchored\",\"day\":{day},{},\"from\":{},\"to\":{}}}",
            ind(indicator),
            show(indicator, *from),
            show(indicator, *to)
        ),
        Output::ReanchorReverted { day, indicator } => format!("{{\"t\":\"ReanchorReverted\",\"day\":{day},{}}}", ind(indicator)),
        Output::ShareQueued { share, class, to, payload } => format!(
            "{{\"t\":\"ShareQueued\",\"share\":{share},\"class\":{},\"to\":{},\"payload\":{}}}",
            q(class),
            to.as_deref().map_or("null".into(), q),
            q(payload)
        ),
        Output::ShareSent { share, to } => format!("{{\"t\":\"ShareSent\",\"share\":{share},\"to\":{}}}", q(to)),
        Output::ShareRerouted { share, to } => format!("{{\"t\":\"ShareRerouted\",\"share\":{share},\"to\":{}}}", q(to)),
        Output::ShareUndeliverable { share, class } => {
            format!("{{\"t\":\"ShareUndeliverable\",\"share\":{share},\"class\":{}}}", q(class))
        }
        Output::ShareCancelled { share } => format!("{{\"t\":\"ShareCancelled\",\"share\":{share}}}"),
        Output::ExclusionChanged { recipient, excluded } => {
            format!("{{\"t\":\"ExclusionChanged\",\"recipient\":{},\"excluded\":{excluded}}}", q(recipient))
        }
        Output::RouteEmpty { class } => format!("{{\"t\":\"RouteEmpty\",\"class\":{}}}", q(class)),
    }
}

fn event_json(eng: &Engine, (e, outs): &Event) -> String {
    let r = &e.record;
    let fields: Vec<String> = r
        .fields
        .iter()
        .map(|(k, v)| match v {
            Value::Int(i) => format!("{}:{i}", q(k)),
            Value::Text(t) => format!("{}:{}", q(k), q(t)),
        })
        .collect();
    let outs: Vec<String> = outs.iter().map(|o| output_json(eng, o)).collect();
    format!(
        "{{\"seq\":{},\"met\":{},\"kind\":{},\"f\":{{{}}},\"line\":{},\"out\":[{}]}}",
        r.seq,
        r.met_ms,
        q(&r.kind),
        fields.join(","),
        q(&e.line()),
        outs.join(",")
    )
}

fn events_json(eng: &Engine, events: &[Event]) -> String {
    let ev: Vec<String> = events.iter().map(|e| event_json(eng, e)).collect();
    format!("{{\"ok\":true,\"events\":[{}]}}", ev.join(","))
}

fn error_json(e: &str) -> String {
    format!("{{\"ok\":false,\"error\":{}}}", q(e))
}

// ---------- Safe operations (host-testable) ----------

thread_local! {
    static SESSION: RefCell<Session> = RefCell::new(Session::new());
    static OUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

fn lines(s: &str) -> Vec<String> {
    s.lines().filter(|l| !l.is_empty()).map(str::to_string).collect()
}

/// Two stored copies separated by a NUL byte (ADR-0009 §4).
pub fn op_load(input: &str) -> String {
    let (a, b) = input.split_once('\0').unwrap_or((input, input));
    match Session::load(&lines(a), &lines(b)) {
        Ok((s, events)) => {
            let out = events_json(&s.engine, &events);
            SESSION.with(|x| *x.borrow_mut() = s);
            out
        }
        Err(e) => error_json(&e),
    }
}

pub fn op_submit(wire: &str) -> String {
    SESSION.with(|x| {
        let mut s = x.borrow_mut();
        match s.submit_wire(wire) {
            Ok(ev) => events_json(&s.engine, &[ev]),
            Err(e) => error_json(&e),
        }
    })
}

pub fn op_transmittable() -> String {
    SESSION.with(|x| {
        let s = x.borrow();
        let items: Vec<String> = s
            .engine
            .transmittable()
            .iter()
            .map(|(id, to, p)| format!("{{\"share\":{id},\"to\":{},\"payload\":{}}}", q(to), q(p)))
            .collect();
        format!("{{\"ok\":true,\"items\":[{}]}}", items.join(","))
    })
}

fn install(r: Result<(Session, Vec<Event>), String>) -> String {
    match r {
        Ok((s, events)) => {
            let out = events_json(&s.engine, &events);
            SESSION.with(|x| *x.borrow_mut() = s);
            out
        }
        Err(e) => error_json(&e),
    }
}

pub fn op_demo() -> String {
    install(synth::mission(synth::DEMO_SEED))
}

pub fn op_preflight() -> String {
    install(synth::preflight(synth::DEMO_SEED))
}

// ---------- Linear-memory edge (the only unsafe code in the project) ----------

fn reply(s: String) -> usize {
    OUT.with(|o| {
        *o.borrow_mut() = s.into_bytes();
        o.borrow().len()
    })
}

/// Take ownership of a buffer the host filled after `alloc`.
fn take(ptr: *mut u8, len: usize) -> Result<String, String> {
    // SAFETY: `ptr` came from `alloc(len)` and the host wrote exactly `len` bytes into it.
    let bytes = unsafe { Vec::from_raw_parts(ptr, len, len) };
    String::from_utf8(bytes).map_err(|_| "input is not UTF-8".to_string())
}

#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut v = Vec::<u8>::with_capacity(len.max(1));
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p
}

#[unsafe(no_mangle)]
pub extern "C" fn out_ptr() -> *const u8 {
    OUT.with(|o| o.borrow().as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn load(ptr: *mut u8, len: usize) -> usize {
    reply(take(ptr, len).map_or_else(|e| error_json(&e), |s| op_load(&s)))
}

#[unsafe(no_mangle)]
pub extern "C" fn submit(ptr: *mut u8, len: usize) -> usize {
    reply(take(ptr, len).map_or_else(|e| error_json(&e), |s| op_submit(&s)))
}

#[unsafe(no_mangle)]
pub extern "C" fn transmittable() -> usize {
    reply(op_transmittable())
}

#[unsafe(no_mangle)]
pub extern "C" fn demo() -> usize {
    reply(op_demo())
}

#[unsafe(no_mangle)]
pub extern "C" fn preflight() -> usize {
    reply(op_preflight())
}

#[cfg(test)]
mod tests;
