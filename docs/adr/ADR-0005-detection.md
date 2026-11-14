# ADR-0005: Detection — CUSUM bank, autocorrelation, calibration, missingness, adaptive sampling, drift

**Status:** accepted · 10 Nov 2026
**Evidence:** `analysis/adr0005_debates.py` (results quoted below).

## Context

The baseline (ADR-0004) delivers, per indicator and day, a standardised deviation $z_t$ that is N(0,1) when nothing has changed. DESIGN_PAPER §4.3–4.4 requires:
- sustained-shift detection under an explicit system false-alert budget (N4: ≤ 1 per 30 days);
- missingness treated as a signal;
- adaptive sampling;
- a drift test against the anchor that produces review prompts, never alarms.

---

## D1. Detector: one-sided CUSUM per indicator

**Alternatives considered**

| Detector | Why not chosen |
|---|---|
| EWMA (HELIOS uses λ = 0.2) | Comparable ARL to CUSUM when tuned, but its run-length distribution has no simple optimality property, and its state is a weighted history, which is harder to explain than "accumulated evidence above your normal" |
| Shiryaev–Roberts | Optimal under a different (Bayesian) criterion; its statistic is less interpretable |
| BOCPD | Gives a change-point posterior, valuable for *explanations* ("likely started around day 112"), but its false-alarm rate has no closed form. Deferred to an explanation-layer ADR |

**Decision**
- $S_t = \max(0, S_{t-1} + s\,z_t - k)$ with $k = 0.5$, which is tuned for a 1-SD shift. $s = \pm1$ is the harmful direction of the indicator.
- **Why:** CUSUM is minimax-optimal for a step change (Lorden 1971; Moustakides 1986). It needs O(1) state and is deterministic. The statistic reads directly as "accumulated evidence beyond your normal".

**State rules**
- A day without data, or a context-tagged day, does **not** update $S$. Absence is handled by D4.
- $S$ resets to 0 after an alarm is raised.

---

## D2. Autocorrelation: CUSUM on $z$ with an autocorrelation-aware threshold. Pre-whitening rejected.

**The design paper planned** AR(1) pre-whitening (CUSUM on residuals).

**On debate:** for a sustained shift δ, the residual mean is only $\delta\sqrt{(1-\phi)/(1+\phi)}$, so pre-whitening throws away signal exactly when daily data are autocorrelated.

**Simulation** (both methods calibrated to ARL₀ = 240 days):

| φ | Pre-whitened: 1σ / 0.5σ delay | Raw $z$, calibrated $h$: 1σ / 0.5σ delay |
|---|---|---|
| 0.0 | 7.7 / 23.6 d | 7.7 / 23.7 d |
| 0.3 | 12.3 / 37.5 d | **11.4 / 32.6 d** |
| 0.6 | 22.5 / 61.7 d | **18.1 / 47.6 d** |

**Decision:** CUSUM on $z$. Autocorrelation enters only through the threshold, via calibration (D3). This revises DESIGN_PAPER v3 §4.3.

**Honest consequence:** with realistic daily autocorrelation (φ ≈ 0.3–0.6), the expected 1σ detection delay is 11–18 days, not the 6–8 days quoted earlier under an independence assumption. The design paper is corrected accordingly.

---

## D3. Threshold calibration to the system budget

**Target:** a single threshold $h$ shared by the $K$ z-indicators, chosen so that the expected time to the first false alarm from any of them equals that bank's share of the budget (D4 fixes the shares).

**In-control model: two options, selected by how much data exist**

| Data | Model | Why |
|---|---|---|
| < 90 in-control days | **Parametric:** Gaussian vector AR(1) with per-indicator $\phi_i$ and a cross-correlation matrix $R$. Both are shrunk: $\phi_i$ toward 0.3 and $R$ toward $I$, with weight $\lambda = 20/(20+n)$ | A short history cannot support a nonparametric resample. Shrinkage keeps estimates stable. 0.3 is a deliberately non-zero prior, because assuming independence produces false alarms |
| ≥ 90 days | **Stationary bootstrap** (Politis–Romano, mean block length 7) of the person's own historical $z$ vectors | Preserves non-Gaussian tails, cross-correlation and serial dependence without assuming a model |

**Procedure**
- Simulate $N = 2000$ paths of length $H = 10B$ with a seeded generator.
- For each path, compute the running maximum $R_t$ of the bank's CUSUM statistics **once**. The first passage for any $h$ is then the first $t$ with $R_t > h$, because the CUSUM paths themselves do not depend on $h$ before the first alarm.
- Bisection on $h$ with these **common random numbers** is therefore exact, monotone and cheap.
- Paths with no alarm by $H$ are counted as $H$ (conservative, because it under-estimates ARL₀, which lowers $h$ and means *more* alerts, never fewer). The censored fraction is reported. If it exceeds 5%, $H$ is doubled.
- **Recalibration:** weekly, logged with seed and inputs, so that replay (I5) reproduces it exactly.

---

## D4. Missingness: Bernoulli CUSUM

**Alternative rejected:** a normal approximation of the check-in rate. It is poor for rates near 0 or 1, which is exactly the regime of a diligent astronaut.

**Decision:** a log-likelihood-ratio CUSUM for "missed check-in" (Reynolds & Stoumbos).
- In-control miss probability $p_0$: personal, from a Beta(1, 9) prior plus history.
- Alternative: $p_1 = \min(0.9, 1 - (1-p_0)^3)$, i.e. "missing three times as often in odds-like terms".
- Increments:
  - miss: $\ln(p_1/p_0)$;
  - done: $\ln\big((1-p_1)/(1-p_0)\big)$.
- Scheduled-workload days are tagged and skipped.

**Budget split:** the allowed system alarm rate $1/B$ is divided in proportion to the number of statistics:
- the missingness CUSUM gets $1/(K+1)$ of it, i.e. $B_m = (K+1)B$;
- the z-bank gets the rest, i.e. $B_\text{bank} = (K+1)B/K$.

Each is calibrated separately by Monte Carlo. Treating the two as independent can only overstate the combined false-alarm rate, so the real rate stays within the budget.

---

## D5. Adaptive sampling with within-day correlation

- With $m$ same-day samples and within-day correlation $\rho_w$, the day value is

  $$z_\text{day} = \bar z \Big/ \sqrt{\big(1+(m-1)\rho_w\big)/m}$$

  This is N(0,1) in control, so **calibration is unaffected by adaptive sampling** (when $\rho_w$ is right).
- The gain on a shift is $\sqrt{2/(1+\rho_w)}$:

  | $\rho_w$ | Gain |
  |---|---|
  | 0 | ×1.41 |
  | 0.3 | ×1.24 |
  | 0.5 | ×1.16 |
  | 0.7 | ×1.09 |

  `review_checks.py` assumed $\rho_w = 0$, so the earlier benefit claim was optimistic.
- **Estimating $\rho_w$:** from paired same-day samples, Fisher-z shrunk toward a conservative prior $\rho_w = 0.5$ (equivalent to 10 pairs).
  - Under-estimating $\rho_w$ inflates the in-control variance and causes extra false alarms. Starting high is therefore the safe side.
- **Offer rule:** offer an extra 20 s check-in only when $S > h/2$ **and** the estimated gain is ≥ 1.10 (i.e. $\rho_w \le 0.65$). Otherwise the burden buys almost nothing.

---

## D6. Drift against the anchor (review prompt, never an alarm)

**Alternatives considered**

| Method | Assessment |
|---|---|
| Mann–Kendall + Sen (HELIOS) | Robust, but assumes independence; it needs the same autocorrelation correction (Hamed–Rao), and its p-value is less transparent |
| OLS with naive SE | Over-confident under autocorrelation |

**Decision**
- **Test:** OLS slope of the last 28 untagged anchor-standardised values, with its variance inflated by $(1+\hat\phi)/(1-\hat\phi)$ (AR(1) effective sample size). One-sided in the harmful direction at α = 0.01.
- **Displayed estimate:** Sen's slope, which is robust, so one bad day cannot exaggerate it.
- A prompt also fires when the mean anchor distance of the last 14 untagged days exceeds 1 SD (corrected by ADR-0007 §6: the original "every one of 14 days" rule fired for only 1.8% of 1.5-SD shifts).
- Evaluated weekly. Expected false prompts are ≈ 0.01 × K per week (≈ one per 12 weeks for K = 8). This is acceptable because a prompt only invites the astronaut's own review.
- Minimum 14 values; below that, no test is run and the state is "not enough data".

## Consequences

- New core modules: `rng` (xoshiro256\*\* with SplitMix64 seeding, cross-checked against a Python implementation) and `detect`.
- DESIGN_PAPER §4.1, §4.3 and abstract figures are revised to the autocorrelated delays and the ρ_w-dependent adaptive-sampling gain.
