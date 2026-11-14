//! Deterministic PRNG for calibration and bootstrap (ADR-0005). xoshiro256** seeded through
//! SplitMix64 (Blackman & Vigna). Seeds are logged so every simulation replays exactly (I5).
//! Cross-checked against oracle/rng_ref.py.

#[derive(Clone, Debug)]
pub struct Rng {
    s: [u64; 4],
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut sm = seed;
        Rng { s: [splitmix64(&mut sm), splitmix64(&mut sm), splitmix64(&mut sm), splitmix64(&mut sm)] }
    }

    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    /// Uniform in [0, 1) with 53 random bits.
    pub fn uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// Standard normal by Box-Muller; one value per two uniforms keeps consumption fixed.
    pub fn normal(&mut self) -> f64 {
        let u1 = 1.0 - self.uniform(); // (0, 1]: ln is finite
        let u2 = self.uniform();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }

    /// Uniform integer in [0, n) without modulo bias (Lemire's rejection on the high bits).
    pub fn below(&mut self, n: u64) -> u64 {
        assert!(n > 0, "range must be non-empty");
        let threshold = n.wrapping_neg() % n;
        loop {
            let m = (self.next_u64() as u128) * (n as u128);
            if (m as u64) >= threshold {
                return (m >> 64) as u64;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VECTORS: &str = include_str!("../../oracle/vectors/rng_vectors.txt");

    #[test]
    fn matches_python_reference_stream() {
        let mut checked = 0;
        for line in VECTORS.lines().filter(|l| !l.is_empty() && !l.starts_with('#')) {
            let f: Vec<&str> = line.split('\t').collect();
            let mut r = Rng::new(f[1].parse().unwrap());
            match f[0] {
                "u64" => {
                    for want in f[2].split(',') {
                        assert_eq!(r.next_u64(), want.parse::<u64>().unwrap(), "{line}");
                    }
                }
                "normal" => {
                    for want in f[2].split(',') {
                        let w: f64 = want.parse().unwrap();
                        assert!((r.normal() - w).abs() <= 1e-15 * w.abs().max(1.0), "{line}");
                    }
                }
                "below" => {
                    let n: u64 = f[2].parse().unwrap();
                    for want in f[3].split(',') {
                        assert_eq!(r.below(n), want.parse::<u64>().unwrap(), "{line}");
                    }
                }
                t => panic!("unknown tag {t}"),
            }
            checked += 1;
        }
        assert!(checked >= 6);
    }

    #[test]
    fn normal_moments_are_right() {
        let mut r = Rng::new(9);
        let xs: Vec<f64> = (0..400_000).map(|_| r.normal()).collect();
        let m = xs.iter().sum::<f64>() / xs.len() as f64;
        let v = xs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / xs.len() as f64;
        assert!(m.abs() < 0.006 && (v - 1.0).abs() < 0.01, "mean {m} var {v}");
    }
}
