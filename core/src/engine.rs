//! Engine (ADR-0006): a deterministic fold over log entries. Kernel first, companion second;
//! configuration and time come only from the log, so replay reproduces every output (I5).

use crate::baseline::{Anchor, Population, WINDOW, anchor, logit_slider};
use crate::detect::{
    BernoulliCusum, Calibration, Cusum, DRIFT_WINDOW, Direction, Drift, DriftOutcome, anchor_run_prompt,
    calibrate_bank, calibrate_bernoulli, choose_model, day_value, drift_test, estimate_rho_w, increment, miss_rate,
    offer_extra_sample, split_budget,
};
use crate::kernel::{Class, Hit, Outcome, SafetyTable, tmr_evaluate, verified_copy};
use crate::log::{Entry, Record, Value, is_key, sha256_hex};
use crate::outbox::{Outbox, ShareClass};
use std::collections::{BTreeMap, VecDeque};

pub const DEFAULT_BUDGET_DAYS: f64 = 30.0;
pub const MIN_BUDGET_DAYS: i64 = 7;
pub const RECALIBRATE_EVERY: u64 = 7;
/// Days a value waits before it may enter the rolling baseline (ADR-0006 §8).
pub const GUARD_DAYS: usize = 14;
/// Consecutive quiet days (statistic at zero) that clear a latched alarm.
pub const CLEAR_DAYS: u32 = 3;
/// Minimum days between two re-anchorings of one indicator (DESIGN_PAPER §4.4, ADR-0007 §4).
pub const REANCHOR_EVERY: i64 = 30;
/// Largest harmful-direction distance (pre-flight SD) a re-anchored level may have from the
/// pre-flight anchor; beyond it the new level is not the astronaut's alone to accept (ADR-0007 §4).
pub const REANCHOR_MAX_SD: f64 = 2.0;
/// Age (days) at which an alarm episode counts as sustained: it may then be re-anchored, and its
/// days leave the calibration history (ADR-0007 §4-5).
pub const SUSTAINED_DAYS: i64 = 14;
/// z of the personal-range edges: the central 80% of the rolling predictive (ADR-0009 §6).
pub const RANGE_Z: f64 = 1.281_551_565_544_600_5;
const E6: f64 = 1e6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transform {
    /// 0..=1000 slider (0.1-point units), logit-transformed.
    Slider,
    /// Integer value divided by 10^decimals.
    Scaled(u32),
}

impl Transform {
    fn apply(self, v: i64) -> Result<f64, &'static str> {
        match self {
            Transform::Slider => logit_slider(v).map_err(|_| "slider value outside 0..=1000"),
            Transform::Scaled(d) => Ok(v as f64 / 10f64.powi(d as i32)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Output {
    Safety { day: i64, indicator: String, value: i64, hits: Vec<Hit> },
    SafetyCheckFailed { day: i64, indicator: String, value: i64 },
    KernelUnavailable { day: i64, indicator: String, value: i64 },
    OfferExtraSample { day: i64, indicator: String },
    /// Daily standardised value and CUSUM statistic per indicator (charts, explanations, audit),
    /// with the day mean, the central 80% predictive range that standardised it and the anchor
    /// mean, all on the model scale: the interface draws the corridor from these (ADR-0009 §6).
    DayStat { day: i64, indicator: String, z: f64, statistic: f64, y: f64, lo: f64, hi: f64, anchor: f64 },
    Alarm { day: i64, indicator: String, statistic: f64, h: f64 },
    AlarmCleared { day: i64, indicator: String },
    MissingnessAlarm { day: i64 },
    MissingnessCleared { day: i64 },
    DriftPrompt { day: i64, indicator: String, drift: Drift },
    AnchorRunPrompt { day: i64, indicator: String },
    Calibrating { day: i64 },
    Recalibrated { day: i64, bank: Calibration, missingness: Option<Calibration> },
    Rejected { seq: u64, reason: &'static str },
    CompanionError { day: i64, reason: String },
    /// Any "yes" on the high-stakes screener: private safety plan, PPC request, trusted crewmember.
    PrivateSupport { day: i64, items: Vec<String> },
    /// In-flight consent change; carries the new canonical table so kernel B evaluates the same one.
    ConsentChanged { day: i64, line: u16, share: bool, table: String, digest: String },
    /// Anchor means (model scale) before and after a consented re-anchoring.
    Reanchored { day: i64, indicator: String, from: f64, to: f64 },
    ReanchorReverted { day: i64, indicator: String },
    /// The sharing-log entry (I6): shown before anything is transmitted (ADR-0008).
    ShareQueued { share: u64, class: &'static str, to: Option<String>, payload: String },
    ShareSent { share: u64, to: String },
    ShareRerouted { share: u64, to: String },
    ShareUndeliverable { share: u64, class: &'static str },
    ShareCancelled { share: u64 },
    ExclusionChanged { recipient: String, excluded: bool },
    RouteEmpty { class: &'static str },
}

#[derive(Clone, Debug)]
struct Indicator {
    dir: Direction,
    transform: Transform,
    pop: Population,
    preflight: Vec<f64>,
    anchor: Option<Anchor>,
    /// Earlier anchors with their rolling windows, oldest (pre-flight) first; a revert pops one.
    superseded: Vec<(Anchor, VecDeque<f64>)>,
    last_reanchor: Option<i64>,
    window: VecDeque<f64>,
    /// Daily means waiting out the guard band, with whether they may be learned.
    pending: VecDeque<(f64, bool)>,
    alarmed: bool,
    alarm_day: i64,
    run_prompted: bool,
    /// Tier-2 consent: share this indicator's weekly summary (ADR-0008 §5).
    shared: bool,
    /// Daily z values of the current week, for the summary.
    week: Vec<f64>,
    quiet_days: u32,
    /// Raw daily means for the drift tests, standardised against the current anchor when tested.
    recent: VecDeque<f64>,
    cusum: Cusum,
    pairs: Vec<(f64, f64)>,
}

#[derive(Clone, Debug, Default)]
struct Day {
    samples: BTreeMap<String, Vec<f64>>,
    has_checkin: bool,
    tagged: bool,
    workload: bool,
}

#[derive(Clone, Debug)]
struct Table {
    copies: [String; 3],
    digest: String,
}

#[derive(Clone, Debug)]
pub struct Engine {
    budget_days: f64,
    indicators: BTreeMap<String, Indicator>,
    table: Option<Table>,
    last_closed: i64,
    day: Day,
    bank_h: Option<f64>,
    history: Vec<Vec<f64>>,
    misses: usize,
    miss_days: usize,
    miss: Option<(BernoulliCusum, f64)>,
    miss_alarmed: bool,
    outbox: Outbox,
    /// Today's values `(seq, indicator, value, from_screen, compared)`, awaiting the kernel-B
    /// comparison at the interface (ADR-0009 §7).
    checkable: Vec<(u64, String, i64, bool, bool)>,
    #[cfg(test)]
    inject_companion_fault: bool,
}

impl Default for Engine {
    fn default() -> Self {
        Engine {
            budget_days: DEFAULT_BUDGET_DAYS,
            indicators: BTreeMap::new(),
            table: None,
            last_closed: 0,
            day: Day::default(),
            bank_h: None,
            history: Vec::new(),
            misses: 0,
            miss_days: 0,
            miss: None,
            miss_alarmed: false,
            outbox: Outbox::default(),
            checkable: Vec::new(),
            #[cfg(test)]
            inject_companion_fault: false,
        }
    }
}

fn text<'a>(r: &'a Record, k: &str) -> Option<&'a str> {
    match r.fields.get(k) {
        Some(Value::Text(t)) => Some(t),
        _ => None,
    }
}

impl Engine {
    pub fn new() -> Self {
        Self::default()
    }

    fn configuring(&self) -> bool {
        self.last_closed == 0
    }

    fn open_day(&self) -> i64 {
        self.last_closed + 1
    }

    /// Indicator fields of a value-carrying record (every key except `day`).
    fn values<'a>(&self, r: &'a Record) -> impl Iterator<Item = (&'a String, &'a Value)> {
        r.fields.iter().filter(|(k, _)| k.as_str() != "day")
    }

    /// Trust-boundary check, to be called before a record is appended (ADR-0006 §7).
    pub fn validate(&self, r: &Record) -> Result<(), &'static str> {
        let int = |k: &str| r.get_int(k).ok_or("missing integer field");
        let check_values = |need_day: bool| -> Result<(), &'static str> {
            if need_day && int("day")? != self.open_day() {
                return Err("day is not the open day");
            }
            let mut any = false;
            for (k, v) in self.values(r) {
                let ind = self.indicators.get(k).ok_or("unknown indicator")?;
                let Value::Int(v) = v else { return Err("indicator value must be an integer") };
                ind.transform.apply(*v)?;
                any = true;
            }
            if any { Ok(()) } else { Err("no indicator values") }
        };
        match r.kind.as_str() {
            "setup" => {
                if !self.configuring() {
                    return Err("configuration after first day_close");
                }
                if int("budget_days")? < MIN_BUDGET_DAYS {
                    return Err("budget below minimum");
                }
                Ok(())
            }
            "indicator" => {
                if !self.configuring() {
                    return Err("configuration after first day_close");
                }
                let key = text(r, "key_txt").ok_or("missing key_txt")?;
                if !is_key(key) || key == "day" || self.indicators.contains_key(key) {
                    return Err("invalid or duplicate indicator key");
                }
                if !matches!(int("dir")?, 1 | -1) {
                    return Err("dir must be 1 or -1");
                }
                match (int("transform")?, int("decimals")?) {
                    (0, 0) | (1, 0..=9) => {}
                    _ => return Err("transform/decimals invalid"),
                }
                if int("tau2_e6")? <= 0 || int("sigma2_e6")? <= 0 || int("nu_p_e6")? < 2_000_000 {
                    return Err("population parameters invalid");
                }
                int("mu_e6").map(|_| ())
            }
            "safety_table" => {
                if !self.configuring() {
                    return Err("safety table is fixed after first day_close");
                }
                let t = text(r, "table_txt").ok_or("missing table_txt")?;
                let d = text(r, "digest_txt").ok_or("missing digest_txt")?;
                if sha256_hex(t.as_bytes()) != d {
                    return Err("table digest mismatch");
                }
                SafetyTable::parse(t).map(|_| ()).map_err(|_| "table does not parse")
            }
            "preflight" => {
                if !self.configuring() {
                    return Err("preflight after first day_close");
                }
                check_values(false)
            }
            "checkin" | "sample" => check_values(true),
            "tag" => {
                if int("day")? != self.open_day() {
                    return Err("day is not the open day");
                }
                if !matches!(int("workload")?, 0 | 1) { Err("workload must be 0 or 1") } else { Ok(()) }
            }
            "day_close" => {
                if int("day")? != self.open_day() {
                    return Err("day is not the open day");
                }
                if self.configuring() && self.table.is_none() {
                    return Err("no safety table before first day_close");
                }
                if self.configuring() {
                    self.outbox.ready()?;
                }
                Ok(())
            }
            // ADR-0008: transmission layer.
            "recipient" | "route" if !self.configuring() => Err("routing is fixed after first day_close"),
            "recipient" => self.outbox.validate_recipient(text(r, "key_txt").ok_or("missing key_txt")?, int("crew")?),
            "route" => self.outbox.validate_route(
                text(r, "class_txt").ok_or("missing class_txt")?,
                text(r, "to_txt").ok_or("missing to_txt")?,
            ),
            "exclude" => {
                let key = text(r, "key_txt").ok_or("missing key_txt")?;
                if !self.outbox.is_recipient(key) {
                    return Err("unknown recipient");
                }
                match int("on")? {
                    0 | 1 if (int("on")? == 1) == self.outbox.is_excluded(key) => Err("exclusion unchanged"),
                    0 | 1 => Ok(()),
                    _ => Err("on must be 0 or 1"),
                }
            }
            "link" => match int("up")? {
                0 | 1 if (int("up")? == 1) == self.outbox.link_up() => Err("link state unchanged"),
                0 | 1 => Ok(()),
                _ => Err("up must be 0 or 1"),
            },
            "share_category" => {
                let ind = self.indicators.get(text(r, "key_txt").ok_or("missing key_txt")?).ok_or("unknown indicator")?;
                match int("on")? {
                    0 | 1 if (int("on")? == 1) == ind.shared => Err("sharing unchanged"),
                    0 | 1 => Ok(()),
                    _ => Err("on must be 0 or 1"),
                }
            }
            "ppc_request" | "crew_request" => {
                if int("day")? != self.open_day() { Err("day is not the open day") } else { Ok(()) }
            }
            "sent" => self.outbox.validate_sent(u64::try_from(int("share")?).map_err(|_| "no such share")?),
            "screen" => {
                if int("day")? != self.open_day() {
                    return Err("day is not the open day");
                }
                let table = self.verified_table().ok_or("safety table unavailable")?;
                let mut any = false;
                for (k, v) in self.values(r) {
                    let mut lines = table.lines().iter().filter(|l| &l.indicator == k).peekable();
                    if lines.peek().is_none()
                        || lines.any(|l| l.class != Class::HighStakes)
                        || self.indicators.contains_key(k)
                    {
                        return Err("screen item must be a high-stakes-only table key");
                    }
                    if !matches!(v, Value::Int(0 | 1)) {
                        return Err("screen answer must be 0 or 1");
                    }
                    any = true;
                }
                if any { Ok(()) } else { Err("no screen items") }
            }
            "consent" => {
                if int("day")? != self.open_day() {
                    return Err("day is not the open day");
                }
                let table = self.verified_table().ok_or("safety table copies disagree")?;
                let id = u16::try_from(int("line")?).map_err(|_| "no such line")?;
                let share = match int("share")? {
                    0 => false,
                    1 => true,
                    _ => return Err("share must be 0 or 1"),
                };
                let line = table.lines().iter().find(|l| l.id == id).ok_or("no such line")?;
                if line.class != Class::HighStakes {
                    return Err("physiological lines always share");
                }
                if line.auto_share_consented == share { Err("consent unchanged") } else { Ok(()) }
            }
            "reanchor" | "reanchor_revert" => {
                if self.configuring() || int("day")? != self.open_day() {
                    return Err("day is not the open day");
                }
                let key = text(r, "key_txt").ok_or("missing key_txt")?;
                let ind = self.indicators.get(key).ok_or("unknown indicator")?;
                if r.kind == "reanchor_revert" {
                    return if ind.superseded.is_empty() { Err("nothing to revert") } else { Ok(()) };
                }
                let table = self.verified_table().ok_or("safety table unavailable")?;
                if table.lines().iter().any(|l| l.indicator == key) {
                    return Err("re-anchoring forbidden for safety-line indicators");
                }
                if ind.alarmed && self.open_day() - ind.alarm_day < SUSTAINED_DAYS {
                    return Err("alarm episode younger than 14 days");
                }
                if ind.last_reanchor.is_some_and(|d| self.open_day() - d < REANCHOR_EVERY) {
                    return Err("re-anchoring limited to once per 30 days");
                }
                if ind.recent.len() < WINDOW {
                    return Err("not enough recent days");
                }
                let pre = ind.superseded.first().map_or(ind.anchor.ok_or("no anchor")?, |s| s.0).prior;
                let new = anchor(&ind.pop, &ind.recent.iter().copied().collect::<Vec<_>>()).map_err(|_| "re-anchoring failed")?;
                let harm = ind.dir.sign() * (new.prior.m - pre.m) / (pre.beta / pre.alpha).sqrt();
                if harm > REANCHOR_MAX_SD { Err("new level too far from pre-flight to accept alone") } else { Ok(()) }
            }
            "kernel_disagree" => {
                if int("day")? != self.open_day() {
                    return Err("day is not the open day");
                }
                let (seq, key) = (u64::try_from(int("ref")?).map_err(|_| "no such value")?, text(r, "key_txt").ok_or("missing key_txt")?);
                match self.checkable.iter().find(|c| c.0 == seq && c.1 == key) {
                    None => Err("no such value today"),
                    Some(c) if c.4 => Err("disagreement already recorded"),
                    Some(_) => Ok(()),
                }
            }
            _ => Err("unknown record kind"),
        }
    }

    /// Model-scale value of `indicator` in its record units (slider 0..=1000, or value/10^d),
    /// for presentation only (ADR-0009 §6).
    pub fn display(&self, indicator: &str, y: f64) -> Option<f64> {
        Some(match self.indicators.get(indicator)?.transform {
            Transform::Slider => crate::baseline::slider_from_logit(y),
            Transform::Scaled(_) => y,
        })
    }

    /// Decimal places of an indicator's record units (0 for sliders).
    pub fn decimals(&self, indicator: &str) -> Option<u32> {
        Some(match self.indicators.get(indicator)?.transform {
            Transform::Slider => 0,
            Transform::Scaled(d) => d,
        })
    }

    /// Apply one log entry. Never panics on bad input: invalid records yield `Rejected`.
    pub fn apply(&mut self, e: &Entry) -> Vec<Output> {
        let r = &e.record;
        if let Err(reason) = self.validate(r) {
            return vec![Output::Rejected { seq: r.seq, reason }];
        }
        let int = |k: &str| r.get_int(k).expect("validated");
        match r.kind.as_str() {
            "setup" => {
                self.budget_days = int("budget_days") as f64;
                vec![]
            }
            "indicator" => {
                let key = text(r, "key_txt").expect("validated").to_string();
                let transform = if int("transform") == 0 { Transform::Slider } else { Transform::Scaled(int("decimals") as u32) };
                let pop = Population {
                    mu: int("mu_e6") as f64 / E6,
                    tau2: int("tau2_e6") as f64 / E6,
                    sigma2: int("sigma2_e6") as f64 / E6,
                    nu_p: int("nu_p_e6") as f64 / E6,
                };
                let dir = if int("dir") == 1 { Direction::Up } else { Direction::Down };
                self.indicators.insert(
                    key,
                    Indicator {
                        dir,
                        transform,
                        pop,
                        preflight: vec![],
                        anchor: None,
                        superseded: vec![],
                        last_reanchor: None,
                        window: VecDeque::new(),
                        pending: VecDeque::new(),
                        alarmed: false,
                        alarm_day: 0,
                        run_prompted: false,
                        shared: false,
                        week: vec![],
                        quiet_days: 0,
                        recent: VecDeque::new(),
                        cusum: Cusum::default(),
                        pairs: vec![],
                    },
                );
                vec![]
            }
            "safety_table" => {
                let t = text(r, "table_txt").expect("validated");
                // Three independent copies so a memory fault in one is outvoted (ADR-0003 §2).
                self.table = Some(Table {
                    copies: [t.to_string(), t.to_string(), t.to_string()],
                    digest: text(r, "digest_txt").expect("validated").to_string(),
                });
                vec![]
            }
            "preflight" => {
                for (k, v) in r.fields.iter() {
                    let Value::Int(v) = v else { continue };
                    let ind = self.indicators.get_mut(k).expect("validated");
                    ind.preflight.push(ind.transform.apply(*v).expect("validated"));
                }
                vec![]
            }
            "checkin" | "sample" => self.on_values(r),
            "tag" => {
                self.day.tagged = true;
                self.day.workload |= int("workload") == 1;
                vec![]
            }
            "day_close" => self.on_day_close(e),
            "screen" => self.on_screen(r),
            "consent" => {
                let (id, share) = (int("line") as u16, int("share") == 1);
                let t = self.verified_table().expect("validated").with_consent(id, share).expect("validated");
                let table = t.canonical();
                let digest = sha256_hex(table.as_bytes());
                self.table = Some(Table { copies: [table.clone(), table.clone(), table.clone()], digest: digest.clone() });
                vec![Output::ConsentChanged { day: int("day"), line: id, share, table, digest }]
            }
            "reanchor" => self.on_reanchor(int("day"), text(r, "key_txt").expect("validated")),
            "reanchor_revert" => {
                let key = text(r, "key_txt").expect("validated");
                let ind = self.indicators.get_mut(key).expect("validated");
                let (a, w) = ind.superseded.pop().expect("validated");
                (ind.anchor, ind.window) = (Some(a), w);
                // "Not my normal": days waiting out the guard band must not be learned either.
                ind.pending.iter_mut().for_each(|p| p.1 = false);
                vec![Output::ReanchorReverted { day: int("day"), indicator: key.to_string() }]
            }
            "recipient" => {
                self.outbox.add_recipient(text(r, "key_txt").expect("validated"), int("crew") == 1);
                vec![]
            }
            "route" => {
                self.outbox.add_route(text(r, "class_txt").expect("validated"), text(r, "to_txt").expect("validated"));
                vec![]
            }
            "exclude" => self.outbox.set_excluded(text(r, "key_txt").expect("validated"), int("on") == 1),
            "link" => self.outbox.set_link(int("up") == 1),
            "share_category" => {
                let key = text(r, "key_txt").expect("validated");
                let on = int("on") == 1;
                self.indicators.get_mut(key).expect("validated").shared = on;
                if on { vec![] } else { self.outbox.cancel_summaries(key) }
            }
            "ppc_request" => self.outbox.queue(ShareClass::Ppc, &[("day", int("day").to_string()), ("ppc", "1".into())], None),
            "crew_request" => self.outbox.queue(ShareClass::Crew, &[("day", int("day").to_string()), ("crew", "1".into())], None),
            "sent" => vec![self.outbox.mark_sent(int("share") as u64)],
            "kernel_disagree" => {
                // An N-version disagreement is a kernel fault (ADR-0003, ADR-0009 §7).
                let (seq, key) = (int("ref") as u64, text(r, "key_txt").expect("validated"));
                let c = self.checkable.iter_mut().find(|c| c.0 == seq && c.1 == key).expect("validated");
                c.4 = true;
                let (day, value, screen) = (int("day"), c.2, c.3);
                let fault = vec![Output::SafetyCheckFailed { day, indicator: key.to_string(), value }];
                let mut out = self.queue_safety(day, &fault, screen);
                out.insert(0, fault.into_iter().next().expect("one"));
                out
            }
            _ => unreachable!("validated"),
        }
    }

    fn int_values(&self, r: &Record) -> Vec<(String, i64)> {
        self.values(r)
            .map(|(k, v)| match v {
                Value::Int(v) => (k.clone(), *v),
                Value::Text(_) => unreachable!("validated"),
            })
            .collect()
    }

    fn verified_table(&self) -> Option<SafetyTable> {
        let t = self.table.as_ref()?;
        let copy = verified_copy([t.copies[0].as_str(), t.copies[1].as_str(), t.copies[2].as_str()], &t.digest)?;
        SafetyTable::parse(copy).ok()
    }

    /// TMR kernel for every value (I1); independent of all companion state.
    fn kernel(&self, day: i64, raw: &[(String, i64)]) -> Vec<Output> {
        let mut out = Vec::new();
        for (k, v) in raw {
            out.push(match &self.table {
                None => Output::KernelUnavailable { day, indicator: k.clone(), value: *v },
                Some(t) => {
                    let copies = [t.copies[0].as_str(), t.copies[1].as_str(), t.copies[2].as_str()];
                    match tmr_evaluate(copies, &t.digest, k, *v).outcome {
                        Outcome::Hits(h) if h.is_empty() => continue,
                        Outcome::Hits(hits) => Output::Safety { day, indicator: k.clone(), value: *v, hits },
                        Outcome::CheckFailed => Output::SafetyCheckFailed { day, indicator: k.clone(), value: *v },
                    }
                }
            });
        }
        out
    }

    /// Queue tier-3 shares for the kernel's outputs (ADR-0008 §2). Values travel only when every
    /// shared line is physiological; a screen never shares a value, nor even a kernel fault.
    fn queue_safety(&mut self, day: i64, kernel_out: &[Output], screen: bool) -> Vec<Output> {
        let table = self.verified_table();
        let mut out = Vec::new();
        for o in kernel_out {
            match o {
                Output::Safety { value, hits, .. } => {
                    let shared: Vec<&Hit> = hits.iter().filter(|h| h.share).collect();
                    if shared.is_empty() {
                        continue;
                    }
                    let lines = shared.iter().map(|h| h.id.to_string()).collect::<Vec<_>>().join("+");
                    let mut f = vec![("day", day.to_string()), ("lines", lines)];
                    let phys = |id: u16| {
                        table.as_ref().is_some_and(|t| t.lines().iter().any(|l| l.id == id && l.class == Class::Physiological))
                    };
                    if !screen && shared.iter().all(|h| phys(h.id)) {
                        f.push(("value", value.to_string()));
                    }
                    out.extend(self.outbox.queue(ShareClass::Safety, &f, None));
                }
                Output::SafetyCheckFailed { .. } if !screen => {
                    let f = [("day", day.to_string()), ("fault", "1".to_string())];
                    out.extend(self.outbox.queue(ShareClass::Safety, &f, None));
                }
                _ => {}
            }
        }
        out
    }

    /// High-stakes screener (ADR-0007 §3): the kernel decides sharing; any "yes" gets private
    /// support derived from the answers themselves; no other state changes.
    fn on_screen(&mut self, r: &Record) -> Vec<Output> {
        let day = r.get_int("day").expect("validated");
        let raw = self.int_values(r);
        self.checkable.extend(raw.iter().map(|(k, v)| (r.seq, k.clone(), *v, true, false)));
        let mut out = self.kernel(day, &raw);
        let mut shares = self.queue_safety(day, &out, true);
        out.append(&mut shares);
        let items: Vec<String> = raw.into_iter().filter(|(_, v)| *v == 1).map(|(k, _)| k).collect();
        if !items.is_empty() {
            out.push(Output::PrivateSupport { day, items });
        }
        out
    }

    /// Consented re-anchoring (ADR-0007 §4): the pre-flight estimator applied to recent days.
    fn on_reanchor(&mut self, day: i64, key: &str) -> Vec<Output> {
        let ind = self.indicators.get_mut(key).expect("validated");
        let recent: Vec<f64> = ind.recent.iter().copied().collect();
        let (Some(old), Ok(new)) = (ind.anchor, anchor(&ind.pop, &recent)) else {
            return vec![Output::CompanionError { day, reason: "re-anchoring failed".into() }];
        };
        // The accepted level becomes the reference everywhere: anchor, rolling window, evidence.
        let window: VecDeque<f64> = recent[recent.len().saturating_sub(WINDOW)..].iter().copied().collect();
        ind.superseded.push((old, std::mem::replace(&mut ind.window, window)));
        ind.pending.clear(); // those days are now in the window
        ind.anchor = Some(new);
        ind.last_reanchor = Some(day);
        ind.cusum.s = 0.0;
        let mut out = vec![Output::Reanchored { day, indicator: key.to_string(), from: old.prior.m, to: new.prior.m }];
        if std::mem::take(&mut ind.alarmed) {
            out.push(Output::AlarmCleared { day, indicator: key.to_string() });
        }
        out
    }

    fn on_values(&mut self, r: &Record) -> Vec<Output> {
        let day = r.get_int("day").expect("validated");
        let raw = self.int_values(r);
        // 1. Kernel first, for every value, independent of everything below (I1); its shares are
        //    queued before any companion processing.
        self.checkable.extend(raw.iter().map(|(k, v)| (r.seq, k.clone(), *v, false, false)));
        let mut out = self.kernel(day, &raw);
        let mut shares = self.queue_safety(day, &out, false);
        out.append(&mut shares);
        // 2. Companion: record samples, offer an extra sample if evidence is building.
        if r.kind == "checkin" {
            self.day.has_checkin = true;
        }
        match self.companion_values(day, &raw) {
            Ok(mut o) => out.append(&mut o),
            Err(reason) => out.push(Output::CompanionError { day, reason }),
        }
        out
    }

    fn companion_values(&mut self, day: i64, raw: &[(String, i64)]) -> Result<Vec<Output>, String> {
        #[cfg(test)]
        if self.inject_companion_fault {
            return Err("injected fault".into());
        }
        let mut out = Vec::new();
        for (k, v) in raw {
            let ind = self.indicators.get(k).expect("validated");
            let y = ind.transform.apply(*v).map_err(str::to_string)?;
            let samples = self.day.samples.entry(k.clone()).or_default();
            samples.push(y);
            if samples.len() != 1 || self.day.tagged {
                continue;
            }
            let (Some(h), Some(a)) = (self.bank_h, ind.anchor.as_ref()) else { continue };
            let z = a.rolling(&ind.window.iter().copied().collect::<Vec<_>>()).and_then(|s| s.z(y)).map_err(|e| format!("{e:?}"))?;
            let provisional = (ind.cusum.s + increment(z, ind.dir)).max(0.0);
            if offer_extra_sample(provisional, h, estimate_rho_w(&ind.pairs)) {
                out.push(Output::OfferExtraSample { day, indicator: k.clone() });
            }
        }
        Ok(out)
    }

    fn on_day_close(&mut self, e: &Entry) -> Vec<Output> {
        let day = e.record.get_int("day").expect("validated");
        let out = match self.companion_close(day, e) {
            Ok(o) => o,
            Err(reason) => vec![Output::CompanionError { day, reason }],
        };
        self.last_closed = day;
        self.day = Day::default();
        self.checkable.clear();
        out
    }

    fn companion_close(&mut self, day: i64, e: &Entry) -> Result<Vec<Output>, String> {
        #[cfg(test)]
        if self.inject_companion_fault {
            return Err("injected fault".into());
        }
        let mut out = Vec::new();
        if self.configuring() {
            for ind in self.indicators.values_mut() {
                ind.anchor = Some(anchor(&ind.pop, &ind.preflight).map_err(|e| format!("anchor: {e:?}"))?);
            }
        }
        let day_state = std::mem::take(&mut self.day);
        let mut day_z: Vec<Option<f64>> = Vec::with_capacity(self.indicators.len());
        let mut in_episode = false;
        for (key, ind) in self.indicators.iter_mut() {
            let samples = day_state.samples.get(key);
            let (Some(samples), false) = (samples, day_state.tagged) else {
                day_z.push(None);
                continue;
            };
            let a = ind.anchor.expect("anchors set at first close");
            let state = a.rolling(&ind.window.iter().copied().collect::<Vec<_>>()).map_err(|e| format!("{e:?}"))?;
            let zs: Vec<f64> = samples.iter().map(|y| state.z(*y)).collect::<Result<_, _>>().map_err(|e| format!("{e:?}"))?;
            let rho = estimate_rho_w(&ind.pairs);
            let zd = day_value(&zs, rho).ok_or("day value undefined")?;
            if zs.len() >= 2 {
                ind.pairs.push((zs[0], zs[1]));
            }
            let mut learnable = true; // before the first calibration nothing is suspected
            if let Some(h) = self.bank_h {
                let before = (ind.cusum.s + increment(zd, ind.dir)).max(0.0);
                let fired = ind.cusum.step(zd, ind.dir, h);
                if fired {
                    // Hold the evidence at h instead of resetting it, so an episode ends only when
                    // the evidence has really decayed; a reset let noise clear it (ADR-0007 §5).
                    ind.cusum.s = h;
                }
                // Latch: one alarm per episode; cleared after CLEAR_DAYS quiet days (ADR-0006 §9).
                if fired && !ind.alarmed {
                    ind.alarmed = true;
                    ind.alarm_day = day;
                    ind.quiet_days = 0;
                    out.push(Output::Alarm { day, indicator: key.clone(), statistic: before, h });
                } else if ind.alarmed {
                    ind.quiet_days = if !fired && ind.cusum.s == 0.0 { ind.quiet_days + 1 } else { 0 };
                    if ind.quiet_days >= CLEAR_DAYS {
                        ind.alarmed = false;
                        out.push(Output::AlarmCleared { day, indicator: key.clone() });
                    }
                }
                // Learning gate: do not learn days inside an alarmed episode. A gate on mere
                // suspicion (S > h/2) is one-sided and biased the baseline (ADR-0006 §8).
                learnable = !ind.alarmed && !fired;
                in_episode |= ind.alarmed && day - ind.alarm_day >= SUSTAINED_DAYS;
            }
            let mean = samples.iter().sum::<f64>() / samples.len() as f64;
            let (lo, hi) = state.interval(RANGE_Z).map_err(|e| format!("{e:?}"))?;
            out.push(Output::DayStat { day, indicator: key.clone(), z: zd, statistic: ind.cusum.s, y: mean, lo, hi, anchor: a.prior.m });
            ind.week.push(zd);
            ind.pending.push_back((mean, learnable));
            while ind.pending.len() > GUARD_DAYS {
                let (m, ok) = ind.pending.pop_front().expect("non-empty");
                if ok {
                    push_capped(&mut ind.window, m, WINDOW);
                }
            }
            push_capped(&mut ind.recent, mean, DRIFT_WINDOW);
            day_z.push(Some(zd));
        }
        if self.bank_h.is_none() {
            out.push(Output::Calibrating { day });
        }
        // Days of a sustained episode are not in-control data: kept, a 1.5-SD shift raised h from
        // 4 to 20 under the bootstrap. Short episodes stay in, because dropping them is a one-sided
        // selection that lowered h (one false alert per 29.1 vs 39.6 days, ADR-0007 §5).
        if !in_episode && day_z.iter().all(Option::is_some) && !day_z.is_empty() {
            self.history.push(day_z.into_iter().flatten().collect());
        }
        if !day_state.workload {
            let missed = !day_state.has_checkin;
            // The in-control miss rate is learned only outside an alarmed episode.
            if !self.miss_alarmed {
                self.miss_days += 1;
                self.misses += missed as usize;
            }
            if let Some((c, h)) = self.miss.as_mut() {
                let fired = c.step(missed, *h);
                if fired && !self.miss_alarmed {
                    self.miss_alarmed = true;
                    out.push(Output::MissingnessAlarm { day });
                } else if self.miss_alarmed && !missed {
                    self.miss_alarmed = false;
                    out.push(Output::MissingnessCleared { day });
                }
            }
        }
        let closed = (day as u64).max(1);
        if closed == 1 || closed.is_multiple_of(RECALIBRATE_EVERY) {
            if closed > 1 {
                for (key, ind) in self.indicators.iter_mut() {
                    let a = ind.anchor.expect("anchors set at first close");
                    let s0 = (a.prior.beta / a.prior.alpha).sqrt();
                    let series: Vec<f64> = ind.recent.iter().map(|m| (m - a.prior.m) / s0).collect();
                    if let DriftOutcome::Tested(d) = drift_test(&series, ind.dir).map_err(|e| format!("{e:?}"))?
                        && d.prompt
                    {
                        out.push(Output::DriftPrompt { day, indicator: key.clone(), drift: d });
                    }
                    // Latched like alarms: one prompt per run, re-armed once the run has ended.
                    let run = anchor_run_prompt(&series, ind.dir);
                    if run && !ind.run_prompted {
                        out.push(Output::AnchorRunPrompt { day, indicator: key.clone() });
                    }
                    ind.run_prompted = run;
                }
            }
            out.push(self.recalibrate(day, e)?);
            if closed > 1 {
                out.extend(self.weekly_summaries(day));
            }
        }
        Ok(out)
    }

    /// Tier-2 weekly summaries (ADR-0008 §5): coverage, mean z and alarm state, nothing raw.
    fn weekly_summaries(&mut self, day: i64) -> Vec<Output> {
        let mut out = Vec::new();
        for (key, ind) in self.indicators.iter_mut() {
            let week = std::mem::take(&mut ind.week);
            if !ind.shared {
                continue;
            }
            let mean = if week.is_empty() { 0.0 } else { week.iter().sum::<f64>() / week.len() as f64 };
            let f = [
                ("ind", key.clone()),
                ("day", day.to_string()),
                ("days", week.len().to_string()),
                ("mean_z_e3", ((mean * 1000.0).round() as i64).to_string()),
                ("alarm", (ind.alarmed as u8).to_string()),
            ];
            out.extend(self.outbox.queue(ShareClass::Summary, &f, Some(key)));
        }
        out
    }

    fn recalibrate(&mut self, day: i64, e: &Entry) -> Result<Output, String> {
        let seed = u64::from_str_radix(&e.hash[..16], 16).expect("hash is hex");
        let k = self.indicators.len();
        if k == 0 {
            return Err("no indicators configured".into());
        }
        let dirs: Vec<Direction> = self.indicators.values().map(|i| i.dir).collect();
        let (bank_budget, miss_budget) = split_budget(self.budget_days, k);
        let model = choose_model(&self.history, k).map_err(|e| format!("{e:?}"))?;
        let bank = calibrate_bank(&model, &dirs, bank_budget, seed).map_err(|e| format!("{e:?}"))?;
        self.bank_h = Some(bank.h);
        let p0 = miss_rate(self.misses, self.miss_days);
        let missingness = match BernoulliCusum::new(p0) {
            Ok(proto) => {
                let c = calibrate_bernoulli(p0, miss_budget, seed ^ 0x5DEE_CE66_D1CE_4E5B).map_err(|e| format!("{e:?}"))?;
                let s = self.miss.map_or(0.0, |(m, _)| m.s);
                self.miss = Some((proto.with_state(s), c.h));
                Some(c)
            }
            // A miss rate this high is itself the finding; keep the previous detector.
            Err(_) => None,
        };
        Ok(Output::Recalibrated { day, bank, missingness })
    }

    /// Transport interface (ADR-0008 §1): `(share, recipient, payload)` it may send now, each to
    /// be acknowledged by appending `sent{share}`.
    pub fn transmittable(&self) -> Vec<(u64, &str, &str)> {
        self.outbox.transmittable()
    }

    /// Full replay: the only way state is ever rebuilt (I5).
    pub fn replay(entries: &[Entry]) -> (Engine, Vec<(u64, Output)>) {
        let mut eng = Engine::new();
        let mut outs = Vec::new();
        for e in entries {
            outs.extend(eng.apply(e).into_iter().map(|o| (e.record.seq, o)));
        }
        (eng, outs)
    }
}

fn push_capped(q: &mut VecDeque<f64>, v: f64, cap: usize) {
    q.push_back(v);
    while q.len() > cap {
        q.pop_front();
    }
}

#[cfg(test)]
mod tests;
