//! Detection layer (ADR-0005): CUSUM bank on baseline z, Bernoulli CUSUM for missed check-ins,
//! same-day combination for adaptive sampling, Monte-Carlo threshold calibration to a system
//! false-alert budget, and the anchor drift test (review prompts only).

use crate::numerics::{NumError, ndtri, t_to_z};
use crate::rng::Rng;

/// CUSUM reference value, tuned to a 1-SD sustained shift.
pub const K_REF: f64 = 0.5;
/// Prior lag-1 autocorrelation for calibration; non-zero on purpose (independence => false alarms).
pub const PHI_PRIOR: f64 = 0.3;
pub const PHI_MAX: f64 = 0.95;
/// Pseudo-days of the shrinkage prior: lambda = SHRINK_N / (SHRINK_N + n).
pub const SHRINK_N: f64 = 20.0;
pub const BOOTSTRAP_MIN_DAYS: usize = 90;
pub const BOOT_MEAN_BLOCK: f64 = 7.0;
pub const N_PATHS: usize = 2000;
pub const CENSOR_MAX: f64 = 0.05;
pub const RHO_W_PRIOR: f64 = 0.5;
pub const RHO_W_PRIOR_PAIRS: f64 = 10.0;
pub const GAIN_MIN: f64 = 1.10;
pub const DRIFT_WINDOW: usize = 28;
pub const DRIFT_MIN: usize = 14;
pub const DRIFT_ALPHA: f64 = 0.01;
pub const ANCHOR_RUN_DAYS: usize = 14;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

impl Direction {
    pub fn sign(self) -> f64 {
        match self {
            Direction::Up => 1.0,
            Direction::Down => -1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DetectError {
    BadInput(&'static str),
    BudgetUnreachable,
    Numeric(NumError),
}

impl From<NumError> for DetectError {
    fn from(e: NumError) -> Self {
        DetectError::Numeric(e)
    }
}

// ---------------------------------------------------------------- D1: CUSUM

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cusum {
    pub s: f64,
}

impl Cusum {
    /// One day's update in the harmful direction; returns true on alarm (and resets).
    pub fn step(&mut self, z: f64, dir: Direction, h: f64) -> bool {
        debug_assert!(z.is_finite() && h > 0.0);
        self.s = (self.s + dir.sign() * z - K_REF).max(0.0);
        if self.s > h {
            self.s = 0.0;
            return true;
        }
        false
    }
}

// ---------------------------------------------------------------- D4: missingness

/// Posterior-mean miss probability under a Beta(1, 9) prior.
pub fn miss_rate(misses: usize, days: usize) -> f64 {
    debug_assert!(misses <= days);
    (1.0 + misses as f64) / (10.0 + days as f64)
}

/// Alternative miss probability: "three times as likely to be missed".
pub fn miss_alternative(p0: f64) -> f64 {
    (1.0 - (1.0 - p0).powi(3)).min(0.9)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BernoulliCusum {
    pub s: f64,
    inc_miss: f64,
    inc_done: f64,
}

impl BernoulliCusum {
    pub fn new(p0: f64) -> Result<Self, DetectError> {
        let p1 = miss_alternative(p0);
        if p0.is_nan() || p0 <= 0.0 || p0 >= p1 || p1 >= 1.0 {
            return Err(DetectError::BadInput("p0 must be in (0, ~0.54)"));
        }
        Ok(BernoulliCusum { s: 0.0, inc_miss: (p1 / p0).ln(), inc_done: ((1.0 - p1) / (1.0 - p0)).ln() })
    }

    /// Same detector with accumulated evidence carried over (used when p0 is re-estimated).
    pub fn with_state(mut self, s: f64) -> Self {
        self.s = s.max(0.0);
        self
    }

    pub fn increments(&self) -> (f64, f64) {
        (self.inc_miss, self.inc_done)
    }

    pub fn step(&mut self, missed: bool, h: f64) -> bool {
        self.s = (self.s + if missed { self.inc_miss } else { self.inc_done }).max(0.0);
        if self.s > h {
            self.s = 0.0;
            return true;
        }
        false
    }
}

// ---------------------------------------------------------------- D5: adaptive sampling

/// Standardised day value from m same-day z samples with within-day correlation rho_w.
pub fn day_value(zs: &[f64], rho_w: f64) -> Option<f64> {
    if zs.is_empty() || !(0.0..1.0).contains(&rho_w) || zs.iter().any(|z| !z.is_finite()) {
        return None;
    }
    let m = zs.len() as f64;
    let mean = zs.iter().sum::<f64>() / m;
    Some(mean / ((1.0 + (m - 1.0) * rho_w) / m).sqrt())
}

/// Within-day correlation from paired same-day samples, Fisher-z shrunk toward RHO_W_PRIOR.
pub fn estimate_rho_w(pairs: &[(f64, f64)]) -> f64 {
    let n = pairs.len();
    if n < 4 {
        return RHO_W_PRIOR;
    }
    let (ma, mb) = pairs.iter().fold((0.0, 0.0), |(a, b), p| (a + p.0, b + p.1));
    let (ma, mb) = (ma / n as f64, mb / n as f64);
    let (mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0);
    for (a, b) in pairs {
        sab += (a - ma) * (b - mb);
        saa += (a - ma).powi(2);
        sbb += (b - mb).powi(2);
    }
    let r = if saa > 0.0 && sbb > 0.0 { (sab / (saa * sbb).sqrt()).clamp(-0.999, 0.999) } else { RHO_W_PRIOR };
    let w = n as f64 - 3.0;
    let zf = (RHO_W_PRIOR_PAIRS * RHO_W_PRIOR.atanh() + w * r.atanh()) / (RHO_W_PRIOR_PAIRS + w);
    zf.tanh().clamp(0.0, 0.99)
}

pub fn adaptive_gain(rho_w: f64) -> f64 {
    (2.0 / (1.0 + rho_w)).sqrt()
}

/// Offer one extra same-day check-in only when evidence is building and it is worth the burden.
pub fn offer_extra_sample(s: f64, h: f64, rho_w: f64) -> bool {
    s > h / 2.0 && adaptive_gain(rho_w) >= GAIN_MIN
}

// ---------------------------------------------------------------- D3: calibration

/// In-control model for the parametric calibration path.
#[derive(Clone, Debug, PartialEq)]
pub struct InControl {
    pub phi: Vec<f64>,
    /// Lower Cholesky factor of the (shrunk) innovation correlation.
    pub chol: Vec<Vec<f64>>,
}

fn cholesky(a: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, DetectError> {
    let k = a.len();
    let mut l = vec![vec![0.0; k]; k];
    for i in 0..k {
        for j in 0..=i {
            let s: f64 = (0..j).map(|p| l[i][p] * l[j][p]).sum();
            if i == j {
                let d = a[i][i] - s;
                if d <= 0.0 {
                    return Err(DetectError::BadInput("correlation not positive definite"));
                }
                l[i][j] = d.sqrt();
            } else {
                l[i][j] = (a[i][j] - s) / l[j][j];
            }
        }
    }
    Ok(l)
}

fn check_history(history: &[Vec<f64>], k: usize) -> Result<(), DetectError> {
    if k == 0 || history.iter().any(|r| r.len() != k || r.iter().any(|v| !v.is_finite())) {
        return Err(DetectError::BadInput("history rows must be complete and finite"));
    }
    Ok(())
}

/// Shrunk per-indicator AR(1) and cross-correlation from in-control history (rows = days).
pub fn estimate_in_control(history: &[Vec<f64>], k: usize) -> Result<InControl, DetectError> {
    check_history(history, k)?;
    let n = history.len();
    let lambda = SHRINK_N / (SHRINK_N + n as f64);
    let mean: Vec<f64> = (0..k).map(|j| history.iter().map(|r| r[j]).sum::<f64>() / n.max(1) as f64).collect();
    let centred = |i: usize, j: usize| history[i][j] - mean[j];
    let var: Vec<f64> = (0..k).map(|j| (0..n).map(|i| centred(i, j).powi(2)).sum::<f64>()).collect();
    let phi = (0..k)
        .map(|j| {
            let r1 = if n >= 3 && var[j] > 0.0 {
                (1..n).map(|i| centred(i, j) * centred(i - 1, j)).sum::<f64>() / var[j]
            } else {
                PHI_PRIOR
            };
            lambda * PHI_PRIOR + (1.0 - lambda) * r1.clamp(0.0, PHI_MAX)
        })
        .collect();
    let mut corr = vec![vec![0.0; k]; k];
    for a in 0..k {
        for b in 0..k {
            let r = if a == b {
                1.0
            } else if n >= 3 && var[a] > 0.0 && var[b] > 0.0 {
                (0..n).map(|i| centred(i, a) * centred(i, b)).sum::<f64>() / (var[a] * var[b]).sqrt()
            } else {
                0.0
            };
            corr[a][b] = if a == b { 1.0 } else { (1.0 - lambda) * r };
        }
    }
    Ok(InControl { phi, chol: cholesky(&corr)? })
}

pub enum Model<'a> {
    Parametric(InControl),
    Bootstrap(&'a [Vec<f64>]),
}

/// Parametric until BOOTSTRAP_MIN_DAYS of history exist, stationary bootstrap afterwards.
pub fn choose_model(history: &[Vec<f64>], k: usize) -> Result<Model<'_>, DetectError> {
    if history.len() >= BOOTSTRAP_MIN_DAYS {
        check_history(history, k)?;
        Ok(Model::Bootstrap(history))
    } else {
        Ok(Model::Parametric(estimate_in_control(history, k)?))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Calibration {
    pub h: f64,
    /// Monte-Carlo estimate of the in-control ARL at h (censored paths count as the horizon).
    pub arl0: f64,
    pub censored: f64,
    pub horizon: usize,
    pub seed: u64,
}

/// Running maximum of the bank statistic along one simulated in-control path.
fn path_record(model: &Model, dirs: &[Direction], horizon: usize, rng: &mut Rng) -> Vec<f64> {
    let k = dirs.len();
    let mut s = vec![0.0; k];
    let mut x = vec![0.0; k];
    let mut rec = Vec::with_capacity(horizon);
    let mut best: f64 = 0.0;
    let mut boot_idx = 0usize;
    for t in 0..horizon {
        match model {
            Model::Parametric(ic) => {
                let u: Vec<f64> = (0..k).map(|_| rng.normal()).collect();
                // x_t = phi x_{t-1} + sqrt(1 - phi^2) L u_t keeps unit marginal variance.
                for (i, (xi, (row, phi))) in x.iter_mut().zip(ic.chol.iter().zip(&ic.phi)).enumerate() {
                    let e: f64 = row[..=i].iter().zip(&u).map(|(l, uj)| l * uj).sum();
                    *xi = if t == 0 { e } else { phi * *xi + (1.0 - phi * phi).sqrt() * e };
                }
            }
            Model::Bootstrap(h) => {
                let n = h.len() as u64;
                boot_idx = if t == 0 || rng.uniform() < 1.0 / BOOT_MEAN_BLOCK {
                    rng.below(n) as usize
                } else {
                    (boot_idx + 1) % h.len()
                };
                x.copy_from_slice(&h[boot_idx]);
            }
        }
        for i in 0..k {
            s[i] = (s[i] + dirs[i].sign() * x[i] - K_REF).max(0.0);
            best = best.max(s[i]);
        }
        rec.push(best);
    }
    rec
}

/// Mean first-passage time over records (non-decreasing); censored paths count as their length.
fn arl_at(records: &[Vec<f64>], h: f64) -> (f64, f64) {
    let mut total = 0.0;
    let mut censored = 0usize;
    for r in records {
        let t = r.partition_point(|&v| v <= h);
        if t == r.len() {
            censored += 1;
        }
        total += (t + 1).min(r.len()) as f64;
    }
    (total / records.len() as f64, censored as f64 / records.len() as f64)
}

fn bisect(records: &[Vec<f64>], target: f64) -> Result<f64, DetectError> {
    let (mut lo, mut hi) = (0.0, 1.0);
    while arl_at(records, hi).0 < target {
        hi *= 2.0;
        if hi > 1e4 {
            return Err(DetectError::BudgetUnreachable);
        }
    }
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if arl_at(records, mid).0 < target { lo = mid } else { hi = mid }
    }
    Ok(hi)
}

fn calibrate_records(
    target: f64,
    seed: u64,
    mut make: impl FnMut(usize, &mut Rng) -> Vec<f64>,
) -> Result<Calibration, DetectError> {
    if target.is_nan() || target < 1.0 {
        return Err(DetectError::BadInput("budget must be at least one day"));
    }
    let mut horizon = (10.0 * target).ceil() as usize;
    for _ in 0..4 {
        let mut rng = Rng::new(seed);
        let records: Vec<Vec<f64>> = (0..N_PATHS).map(|_| make(horizon, &mut rng)).collect();
        let h = bisect(&records, target)?;
        let (arl0, censored) = arl_at(&records, h);
        if censored <= CENSOR_MAX {
            return Ok(Calibration { h, arl0, censored, horizon, seed });
        }
        horizon *= 2;
    }
    Err(DetectError::BudgetUnreachable)
}

/// Common threshold for the z-indicator bank so that its in-control ARL equals `budget_days`.
pub fn calibrate_bank(model: &Model, dirs: &[Direction], budget_days: f64, seed: u64) -> Result<Calibration, DetectError> {
    let k = dirs.len();
    match model {
        Model::Parametric(ic) if ic.phi.len() != k => return Err(DetectError::BadInput("model/direction size mismatch")),
        Model::Bootstrap(h) if h.first().is_some_and(|r| r.len() != k) || h.is_empty() => {
            return Err(DetectError::BadInput("model/direction size mismatch"));
        }
        _ => {}
    }
    calibrate_records(budget_days, seed, |horizon, rng| path_record(model, dirs, horizon, rng))
}

/// Threshold for the missed-check-in CUSUM at in-control miss probability p0.
pub fn calibrate_bernoulli(p0: f64, budget_days: f64, seed: u64) -> Result<Calibration, DetectError> {
    let proto = BernoulliCusum::new(p0)?;
    calibrate_records(budget_days, seed, |horizon, rng| {
        let mut c = proto;
        let mut best: f64 = 0.0;
        (0..horizon)
            .map(|_| {
                c.s = (c.s + if rng.uniform() < p0 { c.inc_miss } else { c.inc_done }).max(0.0);
                best = best.max(c.s);
                best
            })
            .collect()
    })
}

/// Split the system budget (days between false alerts) between K z-indicators and missingness.
pub fn split_budget(system_days: f64, k: usize) -> (f64, f64) {
    let k = k as f64;
    ((k + 1.0) * system_days / k, (k + 1.0) * system_days)
}

// ---------------------------------------------------------------- D6: drift

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Drift {
    pub n: usize,
    /// OLS slope per day in anchor-SD units (signed, raw direction).
    pub slope: f64,
    /// Theil-Sen slope, shown to the astronaut.
    pub sen_slope: f64,
    pub phi: f64,
    /// Normal-equivalent statistic in the harmful direction.
    pub z: f64,
    pub prompt: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DriftOutcome {
    NotEnoughData(usize),
    Tested(Drift),
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if !n.is_multiple_of(2) { v[n / 2] } else { 0.5 * (v[n / 2 - 1] + v[n / 2]) }
}

/// One-sided trend test on the last DRIFT_WINDOW anchor-standardised values (oldest first).
pub fn drift_test(values: &[f64], dir: Direction) -> Result<DriftOutcome, DetectError> {
    if values.iter().any(|v| !v.is_finite()) {
        return Err(DetectError::BadInput("non-finite value"));
    }
    let v = &values[values.len().saturating_sub(DRIFT_WINDOW)..];
    let n = v.len();
    if n < DRIFT_MIN {
        return Ok(DriftOutcome::NotEnoughData(n));
    }
    let nf = n as f64;
    let xbar = (nf - 1.0) / 2.0;
    let ybar = v.iter().sum::<f64>() / nf;
    let sxx: f64 = (0..n).map(|i| (i as f64 - xbar).powi(2)).sum();
    let sxy: f64 = v.iter().enumerate().map(|(i, y)| (i as f64 - xbar) * (y - ybar)).sum();
    let slope = sxy / sxx;
    let resid: Vec<f64> = v.iter().enumerate().map(|(i, y)| y - ybar - slope * (i as f64 - xbar)).collect();
    let ssr: f64 = resid.iter().map(|r| r * r).sum();
    let phi = if ssr > 0.0 {
        ((1..n).map(|i| resid[i] * resid[i - 1]).sum::<f64>() / ssr).clamp(0.0, 0.9)
    } else {
        0.0
    };
    let se = (ssr / (nf - 2.0) / sxx * (1.0 + phi) / (1.0 - phi)).sqrt();
    let signed = dir.sign() * slope;
    let t = if se > 0.0 {
        signed / se
    } else if signed > 0.0 {
        f64::INFINITY
    } else if signed < 0.0 {
        f64::NEG_INFINITY
    } else {
        0.0
    };
    let z = t_to_z(t, nf - 2.0)?;
    let mut pair_slopes: Vec<f64> =
        (0..n).flat_map(|i| ((i + 1)..n).map(move |j| (v[j] - v[i]) / (j - i) as f64)).collect();
    let sen_slope = median(&mut pair_slopes);
    let prompt = z > ndtri(1.0 - DRIFT_ALPHA)?;
    Ok(DriftOutcome::Tested(Drift { n, slope, sen_slope, phi, z, prompt }))
}

/// Prompt when the mean anchor distance of the last ANCHOR_RUN_DAYS untagged days is beyond
/// 1 SD in the harmful direction ($|m_W - m_0|/s_0 > 1$, DESIGN_PAPER §4.4). A run of 14
/// single days all beyond 1 SD fired for only 1.8% of 1.5-SD shifts (ADR-0007 §6).
pub fn anchor_run_prompt(distances: &[f64], dir: Direction) -> bool {
    let Some(last) = distances.len().checked_sub(ANCHOR_RUN_DAYS).map(|s| &distances[s..]) else {
        return false;
    };
    dir.sign() * last.iter().sum::<f64>() / ANCHOR_RUN_DAYS as f64 > 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn iid_model(k: usize) -> Model<'static> {
        Model::Parametric(InControl {
            phi: vec![0.0; k],
            chol: (0..k).map(|i| (0..k).map(|j| if i == j { 1.0 } else { 0.0 }).collect()).collect(),
        })
    }

    #[test]
    fn calibration_matches_siegmund_for_one_iid_indicator() {
        // Siegmund: k = 0.5, ARL0 = 240 <=> h = 3.67 (analysis/detection_limits.py).
        let c = calibrate_bank(&iid_model(1), &[Direction::Up], 240.0, 1).unwrap();
        assert!((c.h - 3.67).abs() < 0.2, "h = {}", c.h);
        assert!(c.censored <= CENSOR_MAX);
    }

    #[test]
    fn calibration_reproduces_autocorrelated_debate_result() {
        // adr0005_debates.py: phi = 0.6, ARL0 = 240 => h = 8.42 for CUSUM on raw z.
        let one = |phi| Model::Parametric(InControl { phi: vec![phi], chol: vec![vec![1.0]] });
        let c = calibrate_bank(&one(0.6), &[Direction::Up], 240.0, 2).unwrap();
        assert!((c.h - 8.42).abs() < 0.45, "h = {}", c.h);
    }

    #[test]
    fn calibrated_bank_meets_budget_on_fresh_paths() {
        let dirs = [Direction::Up, Direction::Down, Direction::Up, Direction::Down];
        let c = calibrate_bank(&iid_model(4), &dirs, 30.0, 3).unwrap();
        // validate with an independent seed: the realised ARL0 must be near the target
        let mut rng = Rng::new(999);
        let recs: Vec<Vec<f64>> = (0..4000).map(|_| path_record(&iid_model(4), &dirs, 600, &mut rng)).collect();
        let (arl, _) = arl_at(&recs, c.h);
        assert!((arl - 30.0).abs() < 3.0, "validated ARL0 {arl} at h {}", c.h);
    }

    #[test]
    fn calibration_is_deterministic_given_seed() {
        let a = calibrate_bank(&iid_model(2), &[Direction::Up; 2], 30.0, 77).unwrap();
        let b = calibrate_bank(&iid_model(2), &[Direction::Up; 2], 30.0, 77).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn bootstrap_on_iid_history_agrees_with_parametric() {
        let mut rng = Rng::new(5);
        let hist: Vec<Vec<f64>> = (0..3000).map(|_| vec![rng.normal(), rng.normal()]).collect();
        let m = choose_model(&hist, 2).unwrap();
        assert!(matches!(m, Model::Bootstrap(_)));
        let b = calibrate_bank(&m, &[Direction::Up; 2], 60.0, 6).unwrap();
        let p = calibrate_bank(&iid_model(2), &[Direction::Up; 2], 60.0, 6).unwrap();
        assert!((b.h - p.h).abs() < 0.35, "bootstrap {} vs parametric {}", b.h, p.h);
    }

    #[test]
    fn short_history_uses_shrunk_parametric_model() {
        let hist = vec![vec![0.1, 0.2], vec![-0.3, 0.0], vec![0.5, 0.4]];
        let Model::Parametric(ic) = choose_model(&hist, 2).unwrap() else { panic!() };
        for p in &ic.phi {
            assert!((*p - PHI_PRIOR).abs() < 0.3);
        }
        assert!(estimate_in_control(&[vec![1.0]], 2).is_err());
    }

    #[test]
    fn bernoulli_cusum_increments_and_calibration() {
        let b = BernoulliCusum::new(0.1).unwrap();
        let (m, d) = b.increments();
        assert!(m > 0.0 && d < 0.0);
        let c = calibrate_bernoulli(0.1, 270.0, 8).unwrap();
        assert!(c.h > m, "threshold must need more than one miss");
        assert!(BernoulliCusum::new(0.0).is_err());
    }

    #[test]
    fn day_value_is_standard_normal_in_control() {
        let mut rng = Rng::new(10);
        let rho: f64 = 0.5;
        let vals: Vec<f64> = (0..200_000)
            .map(|_| {
                let c = rng.normal();
                let z1 = rho.sqrt() * c + (1.0 - rho).sqrt() * rng.normal();
                let z2 = rho.sqrt() * c + (1.0 - rho).sqrt() * rng.normal();
                day_value(&[z1, z2], rho).unwrap()
            })
            .collect();
        let m = vals.iter().sum::<f64>() / vals.len() as f64;
        let v = vals.iter().map(|x| (x - m).powi(2)).sum::<f64>() / vals.len() as f64;
        assert!(m.abs() < 0.01 && (v - 1.0).abs() < 0.015, "mean {m} var {v}");
        assert_eq!(day_value(&[1.3], 0.5), Some(1.3));
    }

    #[test]
    fn rho_w_estimation_and_offer_rule() {
        assert_eq!(estimate_rho_w(&[(0.0, 0.0); 3]), RHO_W_PRIOR);
        let mut rng = Rng::new(11);
        let pairs: Vec<(f64, f64)> = (0..400)
            .map(|_| {
                let c = rng.normal();
                (0.3f64.sqrt() * c + 0.7f64.sqrt() * rng.normal(), 0.3f64.sqrt() * c + 0.7f64.sqrt() * rng.normal())
            })
            .collect();
        assert!((estimate_rho_w(&pairs) - 0.3).abs() < 0.08);
        assert!(offer_extra_sample(2.0, 3.0, 0.3));
        assert!(!offer_extra_sample(2.0, 3.0, 0.8)); // gain too small to justify the burden
        assert!(!offer_extra_sample(1.0, 3.0, 0.3)); // evidence not building
    }

    #[test]
    fn cusum_accumulates_and_resets() {
        let mut c = Cusum::default();
        assert!(!c.step(1.5, Direction::Up, 2.0));
        assert!(!c.step(1.5, Direction::Up, 2.0));
        assert!(c.step(1.5, Direction::Up, 2.0));
        assert_eq!(c.s, 0.0);
        let mut d = Cusum::default();
        assert!(!d.step(3.0, Direction::Down, 2.0) && d.s == 0.0);
    }

    #[test]
    fn anchor_run_uses_the_window_mean() {
        let mut d = vec![1.5; 14];
        assert!(anchor_run_prompt(&d, Direction::Up));
        d[13] = 0.5; // one quiet day no longer cancels a sustained shift
        assert!(anchor_run_prompt(&d, Direction::Up));
        assert!(!anchor_run_prompt(&d, Direction::Down));
        assert!(!anchor_run_prompt(&[1.5; 13], Direction::Up), "needs 14 days");
        let mut e = vec![9.0; 30];
        e.extend([0.9; 14]);
        assert!(!anchor_run_prompt(&e, Direction::Up), "only the last 14 days count");
    }

    const VECTORS: &str = include_str!("../../oracle/vectors/detect_vectors.txt");

    #[test]
    fn deterministic_parts_agree_with_python_reference() {
        let mut n = 0;
        for line in VECTORS.lines().filter(|l| !l.is_empty() && !l.starts_with('#')) {
            let f: Vec<&str> = line.split('\t').collect();
            let nums = |s: &str| -> Vec<f64> { s.split(',').map(|x| x.parse().unwrap()).collect() };
            let close = |a: f64, b: f64| (a - b).abs() <= 1e-10 * b.abs().max(1.0) || (a.is_infinite() && a == b);
            let dir = if f[1] == "up" { Direction::Up } else { Direction::Down };
            match f[0] {
                "drift" => {
                    let want = nums(f[3]);
                    match drift_test(&nums(f[2]), dir).unwrap() {
                        DriftOutcome::NotEnoughData(k) => assert_eq!(want, vec![-1.0, k as f64], "{line}"),
                        DriftOutcome::Tested(d) => {
                            let got = [d.slope, d.sen_slope, d.phi, d.z, d.prompt as u8 as f64];
                            for (g, w) in got.iter().zip(&want) {
                                assert!(close(*g, *w), "{line}\n got {got:?}");
                            }
                        }
                    }
                }
                "cusum" => {
                    let h: f64 = f[2].parse().unwrap();
                    let want = nums(f[4]);
                    let mut c = Cusum::default();
                    for (z, w) in nums(f[3]).iter().zip(&want) {
                        let alarm = c.step(*z, dir, h);
                        let got = if alarm { -1.0 } else { c.s };
                        assert!(close(got, *w), "{line}");
                    }
                }
                "dayvalue" => {
                    let got = day_value(&nums(f[3]), f[2].parse().unwrap()).unwrap();
                    assert!(close(got, f[4].parse().unwrap()), "{line}");
                }
                t => panic!("unknown tag {t}"),
            }
            n += 1;
        }
        assert!(n >= 100, "too few vectors: {n}");
    }
}
