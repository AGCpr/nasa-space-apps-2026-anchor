//! Personal Bayesian baseline (ADR-0004): population prior -> pre-flight anchor -> rolling posterior,
//! and standardisation through the Student-t posterior predictive.

use crate::numerics::{NumError, t_to_z};

/// Half a slider point in 0.1-point units: keeps logit finite at 0 and 1000, barely bends the interior.
pub const LOGIT_EPS: f64 = 5.0;
pub const SLIDER_MAX: i64 = 1000;
/// Pseudo-observations the anchor may contribute to the rolling posterior.
pub const ROLLING_CAP: f64 = 7.0;
/// Untagged days in the rolling window.
pub const WINDOW: usize = 14;
/// Winsorisation limit in predictive SDs before a value updates the baseline.
pub const WINSOR_SD: f64 = 4.0;
/// Reported z is clipped here: acute extremes are the safety kernel's job, not the CUSUM's.
pub const Z_CAP: f64 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BaselineError {
    InvalidPopulation,
    OutOfRange,
    NonFinite,
    Numeric(NumError),
}

impl From<NumError> for BaselineError {
    fn from(e: NumError) -> Self {
        BaselineError::Numeric(e)
    }
}

/// Logit transform for 0..=1000 sliders.
pub fn logit_slider(x: i64) -> Result<f64, BaselineError> {
    if !(0..=SLIDER_MAX).contains(&x) {
        return Err(BaselineError::OutOfRange);
    }
    let x = x as f64;
    Ok(((x + LOGIT_EPS) / (SLIDER_MAX as f64 - x + LOGIT_EPS)).ln())
}

/// Inverse of `logit_slider` on the continuous slider scale, for presentation only.
pub fn slider_from_logit(y: f64) -> f64 {
    let e = y.exp();
    ((e * (SLIDER_MAX as f64 + LOGIT_EPS) - LOGIT_EPS) / (1.0 + e)).clamp(0.0, SLIDER_MAX as f64)
}

/// Population reference for one indicator (on the transformed scale).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Population {
    pub mu: f64,
    /// Between-person variance of personal means.
    pub tau2: f64,
    /// Typical within-person day-to-day variance.
    pub sigma2: f64,
    /// Pseudo-observations the population variance contributes.
    pub nu_p: f64,
}

impl Population {
    fn validate(&self) -> Result<(), BaselineError> {
        let ok = self.mu.is_finite() && self.tau2 > 0.0 && self.sigma2 > 0.0 && self.nu_p >= 2.0;
        if ok && self.tau2.is_finite() && self.sigma2.is_finite() && self.nu_p.is_finite() {
            Ok(())
        } else {
            Err(BaselineError::InvalidPopulation)
        }
    }
}

/// Normal-Inverse-Gamma state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Nig {
    pub m: f64,
    pub kappa: f64,
    pub alpha: f64,
    pub beta: f64,
}

impl Nig {
    /// Posterior predictive: Student-t with (location, scale, df).
    pub fn predictive(&self) -> (f64, f64, f64) {
        let scale = (self.beta * (self.kappa + 1.0) / (self.alpha * self.kappa)).sqrt();
        (self.m, scale, 2.0 * self.alpha)
    }

    /// Conjugate single-observation update (sequential == batch).
    pub fn update(&self, y: f64) -> Nig {
        let k1 = self.kappa + 1.0;
        Nig {
            m: (self.kappa * self.m + y) / k1,
            kappa: k1,
            alpha: self.alpha + 0.5,
            beta: self.beta + self.kappa * (y - self.m).powi(2) / (2.0 * k1),
        }
    }

    /// Standardised deviation of y under this state, ~N(0,1) when nothing has changed; clipped to ±Z_CAP.
    pub fn z(&self, y: f64) -> Result<f64, BaselineError> {
        if !y.is_finite() {
            return Err(BaselineError::NonFinite);
        }
        let (loc, scale, df) = self.predictive();
        Ok(t_to_z((y - loc) / scale, df)?.clamp(-Z_CAP, Z_CAP))
    }

    /// Values whose z is ±`z`: the central predictive interval the interface draws as the
    /// personal range (ADR-0009 §6). Bisection on the monotone t -> z map.
    pub fn interval(&self, z: f64) -> Result<(f64, f64), BaselineError> {
        Ok((self.value_at(-z)?, self.value_at(z)?))
    }

    /// The value whose standardised deviation is `z` (inverse of `z` before clipping), by
    /// bisection on the monotone t -> z map; used for conformal band edges (ADR-0010).
    pub fn value_at(&self, z: f64) -> Result<f64, BaselineError> {
        let (loc, scale, df) = self.predictive();
        let (mut lo, mut hi) = (-64.0, 64.0);
        for _ in 0..70 {
            let mid = 0.5 * (lo + hi);
            if t_to_z(mid, df)? < z { lo = mid } else { hi = mid }
        }
        Ok(loc + 0.5 * (lo + hi) * scale)
    }

    fn winsorise(&self, y: f64) -> f64 {
        let (loc, scale, _) = self.predictive();
        y.clamp(loc - WINSOR_SD * scale, loc + WINSOR_SD * scale)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Anchor {
    pub prior: Nig,
    pub n0: usize,
    /// Weight on the population mean (1 = population only).
    pub shrinkage: f64,
    nu_p: f64,
}

/// Pre-flight anchor shrunk toward the population (ADR-0004 "Anchor").
pub fn anchor(pop: &Population, preflight: &[f64]) -> Result<Anchor, BaselineError> {
    pop.validate()?;
    if preflight.iter().any(|v| !v.is_finite()) {
        return Err(BaselineError::NonFinite);
    }
    let n0 = preflight.len();
    let pseudo = pop.sigma2 / pop.tau2;
    let (mean, ss) = if n0 == 0 {
        (pop.mu, 0.0)
    } else {
        let mean = preflight.iter().sum::<f64>() / n0 as f64;
        (mean, preflight.iter().map(|v| (v - mean).powi(2)).sum::<f64>())
    };
    let kappa = n0 as f64 + pseudo;
    let shrinkage = pseudo / kappa;
    let dv = n0.saturating_sub(1) as f64;
    let prior = Nig {
        m: shrinkage * pop.mu + (1.0 - shrinkage) * mean,
        kappa,
        alpha: (pop.nu_p + dv) / 2.0,
        beta: (pop.nu_p * pop.sigma2 + ss) / 2.0,
    };
    Ok(Anchor { prior, n0, shrinkage, nu_p: pop.nu_p })
}

impl Anchor {
    /// The anchor discounted to at most ROLLING_CAP pseudo-observations, never more certain than itself.
    pub fn rolling_prior(&self) -> Nig {
        let c = self.prior.kappa.min(ROLLING_CAP);
        let alpha = self.prior.alpha.min((self.nu_p + c - 1.0) / 2.0);
        Nig { m: self.prior.m, kappa: c, alpha, beta: alpha * self.prior.beta / self.prior.alpha }
    }

    /// Rolling posterior from the last WINDOW untagged values (oldest first), winsorised sequentially.
    pub fn rolling(&self, window: &[f64]) -> Result<Nig, BaselineError> {
        if window.iter().any(|v| !v.is_finite()) {
            return Err(BaselineError::NonFinite);
        }
        let start = window.len().saturating_sub(WINDOW);
        Ok(window[start..].iter().fold(self.rolling_prior(), |s, &y| s.update(s.winsorise(y))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::numerics::ndtri;

    const VECTORS: &str = include_str!("../../oracle/vectors/baseline_vectors.txt");

    fn floats(s: &str) -> Vec<f64> {
        if s.is_empty() { vec![] } else { s.split(',').map(|v| v.parse().unwrap()).collect() }
    }

    fn rel_eq(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-10 * b.abs().max(1.0)
    }

    /// Rust baseline must match the independent scipy-based Python implementation.
    #[test]
    fn agrees_with_python_reference() {
        let mut n = 0;
        for line in VECTORS.lines().filter(|l| !l.is_empty() && !l.starts_with('#')) {
            let f: Vec<&str> = line.split('\t').collect();
            let p = floats(f[0]);
            let pop = Population { mu: p[0], tau2: p[1], sigma2: p[2], nu_p: p[3] };
            let a = anchor(&pop, &floats(f[1])).unwrap();
            let r = a.rolling(&floats(f[2])).unwrap();
            let y: f64 = f[3].parse().unwrap();
            let want = floats(f[4]);
            let got = [a.prior.m, a.prior.kappa, a.prior.alpha, a.prior.beta, r.m, r.kappa, r.alpha, r.beta, r.z(y).unwrap()];
            for (g, w) in got.iter().zip(&want) {
                assert!(rel_eq(*g, *w), "{line}\n got {got:?}");
            }
            n += 1;
        }
        assert!(n >= 200, "too few vectors: {n}");
    }

    // --- calibration: under the model, z must be standard normal ---
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> f64 {
            // SplitMix64 -> uniform in (0,1)
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64 + 1e-300
        }
        fn normal(&mut self) -> f64 {
            (-2.0 * self.next().ln()).sqrt() * (2.0 * std::f64::consts::PI * self.next()).cos()
        }
        /// Marsaglia-Tsang gamma(shape, 1), shape >= 1.
        fn gamma(&mut self, k: f64) -> f64 {
            let d = k - 1.0 / 3.0;
            let c = 1.0 / (9.0 * d).sqrt();
            loop {
                let x = self.normal();
                let v = (1.0 + c * x).powi(3);
                if v > 0.0 && self.next().ln() < 0.5 * x * x + d - d * v + d * v.ln() {
                    return d * v;
                }
            }
        }
    }

    #[test]
    fn z_is_standard_normal_under_the_model() {
        let s = Nig { m: 0.3, kappa: 4.5, alpha: 3.0, beta: 2.2 };
        let mut rng = Rng(42);
        let mut zs: Vec<f64> = (0..200_000)
            .map(|_| {
                let sigma2 = s.beta / rng.gamma(s.alpha);
                let mu = s.m + (sigma2 / s.kappa).sqrt() * rng.normal();
                s.z(mu + sigma2.sqrt() * rng.normal()).unwrap()
            })
            .collect();
        zs.sort_by(f64::total_cmp);
        for p in [0.01, 0.05, 0.1, 0.25, 0.5, 0.75, 0.9, 0.95, 0.99] {
            let emp = zs[(p * zs.len() as f64) as usize];
            assert!((emp - ndtri(p).unwrap()).abs() < 0.03, "quantile {p}: {emp}");
        }
    }

    #[test]
    fn shrinkage_and_cap_behave_as_specified() {
        let pop = Population { mu: 0.0, tau2: 1.0, sigma2: 1.0, nu_p: 5.0 };
        assert_eq!(anchor(&pop, &[]).unwrap().shrinkage, 1.0);
        let many = anchor(&pop, &vec![2.0; 99]).unwrap();
        assert!((many.prior.m - 2.0 * 99.0 / 100.0).abs() < 1e-12);
        let rp = many.rolling_prior();
        assert_eq!(rp.kappa, ROLLING_CAP);
        assert!(rp.alpha <= many.prior.alpha);
        assert!((rp.beta / rp.alpha - many.prior.beta / many.prior.alpha).abs() < 1e-12);
    }

    #[test]
    fn one_outlier_cannot_drag_the_baseline() {
        let pop = Population { mu: 0.0, tau2: 0.5, sigma2: 1.0, nu_p: 5.0 };
        let a = anchor(&pop, &[0.1, -0.2, 0.0, 0.3, -0.1]).unwrap();
        let calm = a.rolling(&[0.0; 13]).unwrap();
        let spiked = a.rolling(&[&[0.0; 12][..], &[1e6]].concat()).unwrap();
        let (_, scale, _) = calm.predictive();
        assert!((spiked.m - calm.m).abs() < scale, "outlier moved mean by {}", spiked.m - calm.m);
    }

    #[test]
    fn logit_is_finite_monotone_and_validated() {
        assert!(logit_slider(0).unwrap().is_finite() && logit_slider(1000).unwrap().is_finite());
        assert!(logit_slider(500).unwrap().abs() < 1e-12);
        assert!(logit_slider(999).unwrap() > logit_slider(998).unwrap());
        assert_eq!(logit_slider(1001), Err(BaselineError::OutOfRange));
    }
}
