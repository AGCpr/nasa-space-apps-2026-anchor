# ANCHOR: An Anchored, Privacy-Contracted Behavioural Health Companion for Long-Duration Spaceflight

*Design paper v4 — 10 Nov 2026. Working title "ANCHOR" (product name open).*
*v4 is grounded in a systematic landscape analysis of 243 relevant 2026 entries plus flight, analog, research and consumer systems (`LANDSCAPE.md`).*
*NASA Space Apps 2026 — "Create Health Monitoring Software for Astronauts on Space Missions" (2,407 registered teams on this challenge as of 10 Nov).*
*Companion files:*
- *`CRITICAL_REVIEW.md` — independent red-team review; every finding (C1–C4, M1–M19, m1–m6) is resolved in this version.*
- *`analysis/detection_limits.py`, `analysis/review_checks.py` — numerical checks.*
- *`ARCHITECTURE.md` — roles and timeline.*

*v3 is scoped as a full system design, not a hackathon prototype; time constraints are deliberately ignored.*

---

## Abstract

### Thesis (v4)

A systematic analysis of 243 public 2026 entries (LANDSCAPE.md) shows the field converging on one paradigm: *detect deviations in measured data and alert*. The best entries (HELIOS, VITASTRA) now do this well, with personal baselines, autocorrelation-aware EWMA/CUSUM, drift tests and reported false-alarm rates. ANCHOR adopts that paradigm as its floor and argues that it is not where astronaut self-care fails.

Self-care fails in three gaps that no monitoring accuracy can close:

| Gap | Problem | Evidence |
|---|---|---|
| 1. **Disclosure** | What the astronaut is willing to report | IOM; 42.5% of pilots withhold information |
| 2. **Metacognition** | What the astronaut can perceive about themselves | Under 14 days of 6-h sleep, performance fell to the level of 2 nights without sleep while subjective sleepiness did not track it (Van Dongen 2003); astronauts average ~6 h |
| 3. **Action** | Whether knowing becomes doing, and whether the astronaut can tell what actually worked | Pre/post "what worked" checks are biased by regression to the mean; in simulation a useless countermeasure appears to help by +1.71 SD |

**ANCHOR is organised as three closed loops, one per gap, around a verified safety kernel:**

1. The **Disclosure loop** (disclosure contract, graduated high-stakes protocol).
2. The **Calibration loop**: the astronaut predicts their own performance, the app measures it, and the astronaut's self-assessment accuracy is tracked and trained.
3. The **Action loop**: expected-utility alerting that interrupts only when acting now is worth more than the interruption; personal performance forecasting; randomised N-of-1 self-experiments for unbiased "what works for me".

None of the three loops appears in any of the 243 examined entries. The design goal changes accordingly: **not a better watcher of the astronaut, but a better instrument in the astronaut's hands.**

### Summary of v2–v3 (retained)

Exploration crews must increasingly detect and manage their own health: Mars one-way light time reaches ~22 min and solar conjunction removes ground contact for ~2 weeks. Existing in-flight tools (RST/PVT-B, WinSCAT, Cognition, Astroskin) feed researchers and flight surgeons, not the astronaut. The 2026 field of hackathon solutions has converged on a common pattern: vitals dashboard, personal baseline, Isolation-Forest anomaly flags, offline storage, optional chatbot. ANCHOR departs from this pattern on six axes, each grounded in published evidence and each a deliberate response to a documented failure mode:

1. **Anchored dual baseline:** a Bayesian personal baseline that adapts *and* is anchored to the pre-flight self, closing both the "frozen baseline" failure (competitors that fix the baseline at mission start) and the "shifting baseline" failure (competitors that use a rolling mean).
2. **False-alert budget as a design parameter:** detection thresholds derived from a system-level false-alert budget, with stated detection delays and their theoretical limits.
3. **Disclosure contract:** a privacy model designed against the documented underreporting incentive, with safety lines signed before flight.
4. **Verified safety kernel:** a runtime-assurance (Simplex / ASTM F3269-style) kernel with software triple-modular redundancy, independent of everything else.
5. **Psychological depth:** circadian companion, mission-phase engine, resilience dynamics, personal intervention learning, and a calibrated, machine-audited language layer.
6. **Real-data validation:** detectors evaluated on open longitudinal human datasets (LifeSnaps, GLOBEM) as well as synthetic missions with injected events.

v3 adds the results of an adversarial review:
- **Adaptive sampling intensity:** asks for more information only when evidence accumulates (1σ delay 7.8 → 6.0 days, at the cost of extra check-ins on 10% of stable days).
- **Per-person bootstrap-calibrated thresholds:** an exact false-alert budget under real correlation (7.7 → 6.5 days).
- **Graduated disclosure protocol** for high-stakes items.
- **Coverage panel** against automation complacency.
- **N-version, exhaustively verified safety kernel.**
- **Full mission lifecycle** (training → flight → return → family bridge).
- **Personal N-of-1 self-experiments.**
- **Standards-based clinician hand-off.**
- **Staged validation roadmap** from open data to an ISS technology demonstration.

For every component we state the state of the art, what competitors do, what we do differently and why, the costs, and what we deliberately leave out. We also state what cannot be solved, with the governing conditions.

---

## 1. Problem statement

| ID | Problem | Evidence |
|---|---|---|
| P1 | Self-reliance under delay | Mars one-way delay up to ~22 min, ~2-week conjunction blackout; real-time support "not effective" (NASA Mars Medical ConOps) |
| P2 | Underreporting | IOM *Safe Passage*: "institutional pressure to underreport"; astronaut journals record withholding from flight surgeons; 42.5% of military pilots withheld information (Mil Med 2023) |
| P3 | Individual variance | Mars-500: ~55% of self-report variance is stable between-person difference |
| P4 | Slow, phase-dependent decline | Third-quarter dip (Stuster; Mars-500); monotony and communication loss were the hardest periods (Mars-500 narratives) |
| P5 | Chronic sleep and circadian disruption | ~6 h actual sleep vs 8.5 h scheduled; 75–78% hypnotic use in flight (Barger 2014); ~20% of ISS sleep episodes at adverse circadian phase, costing ~1 h/night |
| P6 | Measurement can harm | Orthosomnia (30.9% risk among trackers); nocebo effects of risk framing; streak guilt |
| P7 | Engagement decay | Mental-health app 30-day retention median 3.3% (Baumel 2019) |
| P8 | Operating environment | COTS laptops/tablets on ISS (HP ZBook, ThinkPad), radiation-induced single-event upsets, no network, no repair, scheduled crew time |
| P9 | Competitive context | 2,407 teams; commodity features are table stakes and earn no creativity credit |

---

## 2. Competitive landscape (2026 entries examined in depth)

| Capability | AURORA | RAKIA | AstroHealth | AstroVitals | ASTROCARE / others | **ANCHOR** |
|---|---|---|---|---|---|---|
| Personal baseline | Fixed, first days of mission | Own baseline + mission context | Rolling 14-day mean, % deviation | Fixed, first 21 days, z-score | Not specified | **Bayesian, pre-flight prior, rolling + fixed anchor** |
| Detection method | Isolation Forest, "2 of 4 readings" | Not published | % threshold, consecutive-day rules | z ≥ 1/2 smoothed 3 d + Isolation Forest | Isolation Forest | **CUSUM + anchor drift test under a false-alert budget** |
| Stated false-alert theory | Empirical (0% healthy days on held-out sim) | — | — | — | — | **Budget → thresholds → predicted delays** |
| Psychological layer | Check-in + PVT-B (alertness only) | Mood stays on device | Mood/stress/loneliness logged, no rules | Mood/stress as z-scores | "Mental wellness module" | **Circadian, phase engine, resilience dynamics, interventions, journal** |
| Language design | Reason + action + source | "What changed, how much, since when, why, what next" | Protocol phrasing | Observed + action | — | **Template grammar, nocebo-aware framing, machine audit, aviation alert taxonomy** |
| Privacy | AES-GCM vault, per-category consent, signed batches | Mood on device | JWT roles, Mission Control sees all | — | — | **Disclosure contract: pre-signed safety lines + device-only journal + visible share log** |
| Safety architecture | System checks, audit chain | Deterministic C core, hash ledger | Range filter | — | — | **Separate verified kernel + software TMR + watchdog degraded mode** |
| Validation | 318 unit tests, sim held-out | Not published | 3 suites | 180-day sim, 5 injected events, no metrics | — | **Sim + open real-world datasets + invariant tests** |

**v4: systematic corpus.** The table above lists the entries read in v2. v4 extends the scan to **243 relevant repositories** (502 candidates), coded on 33 features and with nearest neighbours read in full. Method, prevalence tables and limits are in LANDSCAPE.md.

**v4 novelty reassessment, stated honestly.**

| ANCHOR component | Status against the 243-entry corpus |
|---|---|
| Personal baseline, offline operation, encryption, FHIR, private journal, non-diagnostic framing | **Commodity or convergent.** Adopted as the floor |
| Statistical change detection (CUSUM/EWMA), drift test, reported false-alarm rate, coverage gating | **Convergent with HELIOS / VITASTRA.** ANCHOR's increments: per-person bootstrap calibration to an explicit budget, adaptive sampling, consented re-anchoring |
| Disclosure contract with pre-agreed safety lines; graduated high-stakes protocol | **Unique** (0/243) |
| Self-assessment calibration loop | **Unique** (0/243) |
| Expected-utility alerting | **Unique** (0/243) |
| RTM-robust evaluation via randomised N-of-1 | **Unique** (0/243). HELIOS's pre/post method is RTM-biased |
| Runtime-assurance kernel (N-version + TMR) | **Unique** in corpus |
| Personal performance forecasting with Bayesian individualisation | **Near-unique.** One entry uses a non-individualised two-process model |
| Nocebo-aware calibrated language | **Near-unique** (2/243 mention tone) |
| Refusal of composite score, gamification and ground dashboard | **Contrarian, evidence-based.** 28 entries use composite scores |

**Scan protocol and scope of claims (addresses C1).**
- **Method:** GitHub and web search on 10 Nov 2026 for "Space Apps 2026" + astronaut health/monitoring/mental/mood/wellbeing. Inclusion: public repository or README naming this challenge.
- **Examined:** 11 entries (7 with substantive READMEs).
- **Scope of claims:** "absent from the field" in this paper means **absent from the examined entries**. Novelty is argued primarily against the published literature and flight practice (§2.1), which do not move during the hackathon.
- **Re-run:** the scan is repeated after the 15 Nov submission deadline.

**Reading of the table (v2 sample; superseded by the v4 corpus).** The strongest v2-sample entries (AURORA, RAKIA) are excellent physiological-monitoring and software-hygiene projects.

The v4 corpus corrects two v2 statements:
- HELIOS and VITASTRA **do** use statistical change detection with reported false-alarm rates.
- HELIOS **does** validate alert load on real human time series (PMData).

What still holds across all 243 entries: no entry designs against the underreporting incentive with a pre-agreed contract, and none closes the metacognitive or action gaps (see v4 novelty reassessment above). We adopt their hygiene where it is good (encryption at rest, hash-chained logs, input range checks, cited thresholds) and compete on the six axes above.

### 2.1 Novelty against literature and practice (stable reference)

| Contribution | Closest prior art | What is new |
|---|---|---|
| Anchored hierarchical baseline with consented re-anchoring | WinSCAT pre-flight baseline; Oura/WHOOP rolling baselines | Combines both and adds a consent mechanism to separate adaptation from decline |
| Budget-derived, bootstrap-calibrated thresholds | SPC/CUSUM in industry; EWMA for EMA (preprint) | Exact per-person false-alert budget under correlated, autocorrelated, bounded data |
| Adaptive sampling intensity | JITAI (intervention timing); sequential sampling in SPC | Applies variable sampling to self-report burden in spaceflight |
| Disclosure contract + graduated high-stakes protocol | NASA-STD-3001 privacy and PPC; IOM recommendations | Turns the documented underreporting incentive into an explicit design object |
| Runtime-assurance kernel, N-version + TMR | ASTM F3269; Simplex; SWIFT-R | Applies RTA to a behavioural-health companion |
| Circadian companion fused with mission timeline and habitat lighting | Entrain; ISS SSLA; analog lighting studies | Personal phase, with uncertainty, plus the mission schedule as a single daily suggestion |
| Resilience dynamics as non-alerting self-insight | van de Leemput 2014; prospective EWS tests | Uses EWS within their evidential limits |
| Personal N-of-1 self-experiments | Bayesian N-of-1 trials | Gives the astronaut a scientific tool for their own countermeasures |

---

## 3. Requirements and invariants

**Functional:** F1 check-in ≤ 90 s · F2 per-person baseline from pre-flight data · F3 sustained-deviation and drift detection, no single-reading alerts except safety lines · F4 one insight per check-in, ≤ 1 proactive suggestion/day · F5 delay-aware escalation · F6 per-category sharing + pre-agreed safety lines · F7 every output explained, sourced, with confidence.

**Non-functional:** N1 fully offline · N2 < 100 ms per check-in on COTS hardware · N3 no data loss; all state reconstructible · N4 ≤ 1 false alert / astronaut / 30 days in stable conditions · N5 no diagnosis, no alarming vocabulary (machine-checked).

**Invariants (each is an automated test, §12):**
- **I1** A crossed safety line always yields a safety output, whatever the state of non-kernel components.
- **I2** Journal text and interaction metadata never leave the device.
- **I3** No output contains a disease name or banned term.
- **I4** Every output names an action and an owner.
- **I5** Derived state is a pure function of the raw log (replay ⇒ identical outputs).
- **I6** Every transmission is preceded by a visible sharing-log entry.
- **I7** No exploratory signal (resilience dynamics, interaction metadata) can by itself raise an escalation level.
- **I8** A high-stakes self-report item is auto-shared only if it is an imminent-risk item **and** the astronaut pre-consented to auto-sharing for that item.
- **I9** No screen asserts overall wellness ("all good"); every summary shows coverage and confidence.
- **I10** No message is ever routed to a contact the astronaut has excluded.
- **I11** Learning features (bandit, self-experiments) can be switched off, and their absence never disables detection or safety.

---

## 4. Component designs

Each component: **State of the art → What competitors do → ANCHOR design → Why → Costs (or why none) → Opportunities adopted / rejected.**

### 4.1 Measurement: what is asked, and how

- **State of the art.** RST asks sleep timing and VAS ratings of stress, workload, physical and behavioural state daily, plus a 3-min PVT-B. Single-item momentary affect measures are supported as valid, low-burden tools; three items instead of one reduce measurement bias, most when measurement error is high (arXiv 2502.02846). A 5-item brief EMA mood measure retained compliance without losing reliability (Bristol 2026).
- **Competitors.** Free-form mood/stress fields (AstroHealth, VITALS); RST-style check-in (AURORA).
- **ANCHOR.** Core daily set: sleep onset/offset (auto-filled if wearable present, confirm only), **valence, energy, calm** as three bipolar 0–100 sliders (the two-items-per-dimension structure of validated daily mood scales, reduced to one per dimension for burden), stress, workload. PVT-B 2–3×/week (not daily: practice effects and time cost; the Cognition battery literature corrects for practice effects). Weekly: one validated short scale (e.g. PANAS short form), which also lets us recalibrate the daily sliders.
- **Why.** Three dimensions (valence/energy/calm) distinguish states that a single mood slider conflates (tired-content vs tense-energetic), which matters for choosing interventions (§4.8).
- **Costs.** ~20 s more than a single slider. Accepted for the bias reduction.
- **v3: import before asking (M13).** ANCHOR is a personal layer over existing instruments. It imports RST/PVT-B results, Cognition scores, wearable sleep and actigraphy where crews already produce them, and asks only for what is missing. Every value carries provenance (measured / self-reported / imported / derived), following RAKIA's good practice.
- **v3: adaptive sampling intensity (M2).** When accumulated evidence on any indicator passes half the alert threshold, the app offers one extra 20-second check-in that day.
  - Computed effect: 1σ detection delay falls from 7.8 to 6.0 days at an unchanged false-alert budget.
  - Cost: extra check-ins on ~10% of stable days.
  - The extra check-in is optional; declining it is itself logged as context, never as a signal.
- **v3: quiet mode and reactivity monitoring (M4).**
  - Quiet mode runs on imported and passive data only.
  - Reactivity is monitored as a shift in level, variance or compliance in the first 3 weeks after onboarding; a detected shift is reported to the astronaut.
- **Rejected.** Voice or facial emotion analysis (AD ASTRA-style speech analysis, camera affect): privacy-invasive, weak validity in this population, microphone/camera in crew quarters conflicts with the disclosure contract (§4.9), and CIMON-2's experience shows emotion inference is not what crews value.

### 4.2 Personal baseline

- **State of the art.** WinSCAT compares in-flight results with a pre-flight baseline. Oura/WHOOP use 14-day and multi-month personal baselines but publish no formula and fail on transitions (arXiv 2503.19132). Hierarchical Bayesian partial pooling lets an individual start at population values and personalise as data accrue (mHealth online learning, arXiv 2511.08719); shrinkage factor $B=\sigma^2/(\sigma^2+\tau^2)$.
- **Competitors.** Fixed early-mission baseline (AURORA, AstroVitals) ⇒ cannot adapt to legitimate microgravity adaptation, produces persistent "deviations". Rolling mean (AstroHealth) ⇒ absorbs slow decline (shifting-baseline trap).
- **ANCHOR.** Three-level hierarchy:
  1. **Population prior** for each indicator from published astronaut/analog normative data (e.g. Cognition normative data for professional astronauts, Frontiers 2024).
  2. **Pre-flight personal anchor** from $n_0$ pre-flight sessions, shrunk toward the population prior by $B$.
  3. **Rolling in-flight posterior** (14 untagged days) on top of the anchor.
  Detection runs against level 3 (episodes, §4.3) and level 2 (drift, §4.4) simultaneously.
- **Equations.** Normal–Inverse-Gamma prior $\mathrm{NIG}(m_0,\kappa_0,\alpha_0,\beta_0)$ with $m_0 = B\,\mu_{pop} + (1-B)\,\bar x_{pre}$; conjugate update over the window; posterior predictive Student-$t_{2\alpha}$; standardised deviation via the $t$-CDF mapped to a normal quantile; winsorisation at ±4 predictive SD before updating.
- **Why.** It is the only structure that is simultaneously usable on day 1, personal by week 2, and immune to both baseline failure modes.
- **Costs.** Needs pre-flight sessions (realistic: astronauts already do pre-flight baseline data collection for Standard Measures). Without them, the population prior dominates and confidence is shown as low.
- **v3: training-built anchor (M3).** Astronauts are already sleep-deficient in the 3 months before flight (Barger 2014), so a handful of pre-flight sessions may anchor to a stressed state.
  - The anchor is built from **months of training-period use** (lifecycle mode "Training", §4.15) across varied contexts (travel, simulations, rest weeks).
  - It is stored as a **context-conditioned distribution**, not a point.
  - If the anchor itself deviates from the population prior by more than 1 SD, it is flagged for review by the astronaut and, if they choose, their psychologist before launch.
- **v3: bounded scales.** Slider values are logit-transformed, $y=\log\frac{x+\epsilon}{100-x+\epsilon}$, before modelling to remove ceiling and floor compression.

### 4.3 Episode detection under a false-alert budget

- **State of the art.** CUSUM (Page 1954) is minimax-optimal for detecting a mean shift (Lorden 1971; Moustakides 1986); Bayesian online change-point detection (Adams & MacKay 2007) gives run-length posteriors; EWMA statistical process control has been applied to EMA and passive sensing for depression onset (Research Square preprint). Aviation regulation requires alerting systems to "minimize the effects of false and nuisance alerts" (14 CFR 25.1322(d)).
- **Competitors.** Isolation Forest scores (AURORA, AstroVitals, ASTROCARE) and fixed % rules (AstroHealth). Neither gives a controllable false-alert rate; AURORA had to retrain after false alarms on healthy crew.
- **ANCHOR.** One-sided CUSUM per indicator on standardised values, in the harmful direction. System budget: $K$ indicators, ≤ 1 false alert per $B$ days ⇒ per-indicator $\mathrm{ARL}_0 \ge KB$. For $K=8$, $B=30$, $k=0.5$: $h=3.67$; simulated ARL₀ = 244 d; mean detection delay 3.1 / 7.7 / 23.2 days for 2σ / 1σ / 0.5σ sustained shifts.
- **Why CUSUM over Isolation Forest.** (i) Analytical control of false alerts (N4); (ii) O(1) state, deterministic, trivially verifiable; (iii) explainable ("your sleep has been about 1 SD below your normal for 6 days"); (iv) optimality guarantees. Isolation Forest gives none of these and needs training data we do not have for astronauts.
- **Why not BOCPD as primary.** BOCPD gives richer output (probability of a change point) but its false-alert rate depends on hazard-rate and prior choices without a closed form. **Adopted as secondary:** BOCPD run-length posterior feeds the confidence display ("a change likely started around day 112").
- **Costs.** Detection is not instantaneous (limit L1, §6). Large acute changes are therefore routed to safety lines, not to statistics.
- **v3: real data are correlated and autocorrelated (M1).**
  - The union bound assumes independent indicators. With 8 indicators at ρ = 0.5, the union-bound threshold yields a system ARL₀ of 47 days instead of 30: safe, but slow.
  - v3 pre-whitens each indicator with an AR(1) fit and runs CUSUM on the residuals.
  - It then calibrates **one system threshold per person** by block bootstrap of that person's own residual vectors, which preserves cross-indicator correlation and autocorrelation, so the realised false-alert rate equals the budget.
  - Computed for ρ = 0.5: h falls from 3.67 to 3.16 and 1σ detection delay from 7.7 to 6.5 days.
  - Combined with adaptive sampling (§4.1), the expected 1σ delay is ~5 days at the same budget. This combination is estimated, not yet simulated jointly.
  - Recalibration runs weekly and is logged.
- **v3: correlated groups.** For indicator groups with strong expected co-movement (valence/energy/calm), an optional MEWMA statistic complements the univariate CUSUMs. It is reported only when it adds a detection that the univariate statistics miss.

- **Implementation correction (ADR-0005, supersedes the v3 bullets above where they conflict).**
  - **Pre-whitening is rejected.** For a sustained shift δ, pre-whitening leaves only δ·√((1−φ)/(1+φ)) of the signal in the residuals. CUSUM therefore runs on z itself, and autocorrelation enters only through a calibrated threshold.
  - Simulated delays at ARL₀ = 240 d:

    | φ | Pre-whitened, 1σ / 0.5σ | Raw z + calibrated threshold, 1σ / 0.5σ |
    |---|---|---|
    | 0.3 | 12.3 / 37.5 d | 11.4 / 32.6 d |
    | 0.6 | 22.5 / 61.7 d | 18.1 / 47.6 d |

  - **Realistic 1σ delays are 11–18 days**, not the 6–8 days quoted earlier under independence.
  - **Adaptive sampling gain** is √(2/(1+ρ_w)): only ×1.16 at within-day correlation ρ_w = 0.5. The extra check-in is offered only when the person's estimated gain is ≥ 1.10.
  - **Calibration** uses common random numbers and is exact by bisection. It uses a shrunk parametric AR(1) model below 90 days of history and a stationary bootstrap above.

### 4.4 Drift against the anchor

- **State of the art.** The third-quarter phenomenon and gradual adaptation are documented; no consumer or competitor system distinguishes adaptation from decline.
- **ANCHOR.** 28-day OLS slope of untagged values in anchor-SD units, AR(1)-corrected standard error. Detectable after $n^*\approx(z\sigma\sqrt{12}/\beta)^{2/3}$ days: 0.10σ/day → ~19 d; 0.05σ/day → ~30 d; 0.02σ/day → ~55 d. In addition, $|m_W-m_0|/s_0>1$ for ≥ 14 days produces a **review prompt** (never an alarm).
- **Re-anchoring by consent.** The astronaut may confirm "this is my new normal" (e.g. sleep pattern settled after adaptation). Re-anchoring is logged, reversible, and **forbidden for safety-line indicators**.
- **Why.** This is the only mechanism in the field that separates "I have adapted" from "I am slowly declining", and it does so by asking the person rather than guessing.
- **Costs.** An astronaut could re-anchor a real decline. Mitigation: re-anchoring is visible in the weekly summary, limited to once per indicator per 30 days, and does not affect safety lines.
- **Implementation (ADR-0007 §4–6).**
  - **What re-anchoring accepts.** It accepts a *level*, never a *trend*: the drift window survives it.
  - **When it is allowed.**
    - During an alarm episode only once the episode is ≥ 14 days old.
    - Only if the new level is ≤ 2 pre-flight SD from the pre-flight anchor in the harmful direction.
  - **Revert.** A revert restores the previous anchor and its rolling window.
  - **Anchor-run prompt.** It uses the 14-day window mean, as specified above. A first implementation used a run of 14 single days, which fired for only 1.8% of 1.5-SD shifts.

### 4.5 Resilience dynamics (exploratory layer)

- **State of the art.** Rising autocorrelation, variance and inter-emotion correlation precede mood transitions (van de Leemput et al., PNAS 2014); shown within a single patient (Wichers 2016); actigraphy-based variants exist (van Roon 2023). **But** prospective tests are weak: rising variance in ~11% of people before transitions vs ~12% without (no discrimination); recurrence study sensitivity ~33%, specificity ~84%; in bipolar disorder EWS raised transition probability only from ~32–36% to ~46–48% (Bos 2022).
- **Competitors.** None.
- **ANCHOR.** Compute lag-1 autocorrelation and variance of detrended valence/energy over a rolling 21-day window, trend measured by Kendall's τ. Displayed **only** as a private "resilience trend" in the weekly reflection, framed as "how quickly your mood returns to your normal after a hard day". **Never** an alert, never shared, never raises escalation (invariant I7).
- **Why include it at all.** It is the most mechanistically interesting signal in affective science, it is directly meaningful to the user as "recovery speed", and it contributes to self-awareness without claiming prediction.
- **Why not as an alert.** With sensitivity ~33% it would miss most transitions and with specificity ~84% it would produce a false alarm in roughly one of six stable periods, violating N4 and P6. This decision follows the evidence, not novelty.
- **Additional reason (computed).** A lag-1 autocorrelation from 21 daily points has a standard error of about 0.22 (95% CI ± 0.43). One window cannot distinguish 0.2 from 0.6. The display therefore shows the **trend across overlapping windows** with its uncertainty band, never a single value.

### 4.6 Circadian companion

- **State of the art.** Astronauts average ~6 h sleep against 8.5 h scheduled; 75–78% use hypnotics in flight (Barger 2014). ISS Solid State Lighting Assemblies change colour and intensity on a 24-h cycle; in a 45-day analog, dynamic lighting reduced circadian phase drift (acrophase 6.22 h vs 4.76 h, p = 0.0002) and improved some cognitive accuracy (Sleep Advances 2024). Mathematical circadian models personalise light schedules better than static schedules; Apple Watch data predicts DLMO within ~1 h (Forger lab); a level-set Kalman filter estimates phase *with uncertainty* from wearables (arXiv 2207.09406). Limitation reported: users rarely follow recommended schedules.
- **Competitors.** Sleep hours logged or thresholded (e.g. < 6 h for 3 days). No circadian modelling.
- **ANCHOR.** Estimate each astronaut's circadian phase from sleep timing (and wearable activity when available) with a published limit-cycle circadian model and report it with uncertainty. Combine with the mission timeline (sleep shifts, EVAs, vehicle arrivals) and the habitat lighting mode to produce **one** concrete daily timing suggestion (e.g. when to seek bright light, when to switch quarters to pre-sleep mode), and a forecast before scheduled sleep shifts ("start shifting 20 min/day from Thursday").
- **Why.** Sleep is the most prevalent, best-evidenced, most actionable problem in flight, and it mediates mood: in the OASIS trial (n = 3,755), improving sleep with digital CBT-I reduced insomnia (d = 1.11) and paranoia (d = 0.19), and sleep change mediated ~60% of the paranoia effect. A sleep-first strategy is therefore also a mental-health strategy.
- **Costs.** Without measured light exposure, phase estimates carry systematic error. Mitigation: uncertainty displayed; suggestions limited to low-risk timing advice; never medication advice.
- **Adherence problem.** Addressed by the budget of one suggestion/day and personal learning of which suggestions are followed (§4.8).
- **v3: light data and uncertainty gating (M8).**
  - Light inputs, in order of preference:
    1. A light-sensing actigraph, where flown.
    2. The habitat lighting-mode log (the ISS lighting schedule is known and logged).
    3. Self-reported light environment.
  - Phase is estimated with a level-set Kalman filter, which propagates uncertainty.
  - **Advice is suppressed when the 90% phase interval is wider than 3 h.** The app then says what it cannot tell and suggests the one action that is robust to the uncertainty: a regular sleep window.
- **v3: shift planning.** Before a scheduled sleep shift (vehicle arrival, EVA), the app computes a personal pre-adaptation plan. It shows the predicted alertness cost of not following it, so the choice stays with the astronaut.

### 4.7 Mission-phase engine

- **State of the art.** Third-quarter positive-affect decline (Stuster; Mars-500); novel tasks (landing simulation) restored motivation in Mars-500; communication delay and loss were the hardest periods.
- **ANCHOR.** The mission timeline marks phases (first 2 weeks, mid-mission, third quarter, final month, conjunction blackout). Support intensity and content change by phase: novelty prompts in monotony periods, increased Earth-connection content before blackout, autonomy-focused guidance during delay. The engine **anticipates** rather than reacts.
- **Costs.** Phase effects are averages; individuals differ. The engine raises *availability* of support, not alert sensitivity, so a wrong phase assumption has no false-alert cost.

### 4.8 Interventions and personal learning

- **Library (each item carries citation, effect size and evidence grade).**
  - Sleep/circadian (digital CBT-I components; light timing): strongest evidence.
  - Behavioural micro-actions (short exercise, breathing): low cost, modest evidence.
  - Expressive writing: **small effect** (d ≈ 0.15, Frattaroli 2006; non-significant for depressive symptoms, Reinhold 2018). Offered as a private outlet because astronauts themselves reported journals as a release valve (Stuster), **not** as a treatment claim.
  - Conflict and stress modules modelled on the NASA-funded Virtual Space Station / Expedition-APPP content areas (conflict, stress, depression), evaluated in Antarctic analogs and RCTs on Earth.
  - Earth connection: awe and self-transcendence from Earth viewing (overview effect, Yaden 2016; evidence is testimonial, labelled as such).
- **Personal learning.** Contextual Thompson sampling with a hierarchical prior (as deployed in HeartSteps; micro-randomised suggestions increased 30-min step count by 14%, p = .06, Klasnja 2019). Context = phase, current indicators, time of day. Reward = suggestion accepted *and* self-rated helpful next morning. Constraints: ≤ 1 proactive suggestion/day; safety-relevant content is never subject to exploration; every choice is explained ("suggested because it helped you on 4 of 5 similar days").
- **Competitors.** Static rule → action mapping (AstroVitals, AstroHealth).
- **Costs.** Learning needs weeks of data; the hierarchical prior makes early choices sensible. Exploration can occasionally offer a less useful suggestion; harm is bounded by the content filter.
- **Rejected.** Streaks, points, badges (guilt, abstinence-violation effect, Duolingo study); push-notification nudging beyond the budget.
- **v3: evidence grades (m1).** Every library item carries a grade:

  | Grade | Meaning |
  |---|---|
  | A | Meta-analysis or large RCT in comparable adults |
  | B | RCT(s) or a strong analog study |
  | C | Observational or analog evidence only |
  | D | Expert/testimonial evidence only |

  Examples: digital CBT-I = A; dynamic lighting = B; expressive writing = A for a small effect (d ≈ 0.15); overview-effect content = D. The grade is visible to the astronaut.
- **v3: responsible learning (M7).**
  - The learning feature requires informed consent.
  - Exploration happens only among options of equal evidence grade.
  - Posteriors are discounted (half-life 30 days) to follow phase changes.
  - Any policy change is first assessed by **off-policy evaluation** on the astronaut's own logged data.
  - The fallback is the fixed rule policy.
  - The astronaut can switch learning off (I11).
- **v3: personal N-of-1 self-experiments (new).**
  - The astronaut can run a structured experiment on themselves, for example "bright light at 07:00 vs 09:00" or "evening journaling vs none", in randomised blocks over 2–4 weeks.
  - The analysis is a Bayesian N-of-1 model and reports the probability that A beats B for *them*, with uncertainty.
  - **Why:** it converts the app from something that advises the astronaut into an instrument the astronaut uses. This is the strongest available lever for autonomy and competence (SDT), and it suits a population of trained experimenters.
  - **Costs:** carry-over effects. Mitigation: washout days, and only interventions with short-lived effects are eligible.
- **v4: why randomisation is non-negotiable (regression to the mean).**
  - Countermeasures are naturally started when an indicator is bad. Because daily indicators are autocorrelated around a personal mean, they tend to improve afterwards whether or not the countermeasure works.
  - Expected spurious improvement after $L$ days from a trigger at $x_0$ (personal mean $\mu$, AR coefficient $\phi$) is $(1-\phi^{L})(\mu - x_0)$.
  - **Simulation** (`analysis/rtm_check.py`; φ = 0.6, trigger at −1.5 SD, 5-day evaluation, 7,349 episodes): a countermeasure with **zero true effect** shows **+1.71 SD** "improvement" in a pre/post comparison; a coin-flip randomised N-of-1 design estimates **+0.05 SD**.
  - **Consequences:**
    - Every "what works for you" claim in ANCHOR comes either from randomised N-of-1 blocks or from observational estimates explicitly corrected by the RTM term above, using the person's own AR fit.
    - Uncorrected pre/post comparisons are never shown.
    - The best entry in the field (HELIOS) currently uses the uncorrected pre/post design.

### 4.9 Disclosure contract (privacy)

- **State of the art.** NASA-STD-3001 V2 7071 requires privacy for missions > 30 days; V1 6010 schedules Private Psychological Conferences. IOM documents underreporting incentives; NASA's own panel suggested long-term flight-surgeon relationships to encourage disclosure.
- **Competitors.** AURORA: per-category consent and signed batches. RAKIA: mood stays on device. AstroHealth: Mission Control sees everything (inverse of our design).
- **ANCHOR.** A three-tier contract, explained to the astronaut on day 0 and visible thereafter:
  1. **Never leaves device:** journal, interaction metadata, resilience dynamics.
  2. **Astronaut-controlled:** each indicator category, weekly summaries.
  3. **Pre-signed safety lines:** a short list agreed by astronaut and flight surgeon before flight; crossing shares automatically, and the astronaut knew in advance.
  Every share appears in the sharing log *before* transmission (I6).
- **Why this beats storage-level privacy.** Underreporting is an incentive problem. Encryption protects data from third parties; the contract protects the astronaut's *career interest*, which is what drives withholding. That is the mechanism that makes honest data possible.
- **v3: graduated protocol for high-stakes items (C2).**
  - **The contradiction in v2:** automatically sharing a self-harm disclosure recreates the very incentive the contract removes, for the most dangerous item.
  - **The v3 protocol** uses a C-SSRS-style stepwise screener, offered weekly and on demand, never forced.
    - **Any positive answer:** an immediate private safety plan, plus a one-tap **astronaut-initiated** request for a Private Psychological Conference, plus the astronaut's chosen trusted crewmember as a human route. No automatic share.
    - **Imminent-risk items (intent or plan):** automatic notification **only** if the astronaut pre-consented to this before flight, item by item (I8).
  - **Rationale:** the protocol keeps disclosure attractive at every level below imminent risk, where early help is most effective.
  - **Residual risk:** an astronaut who declined pre-consent and is at imminent risk is reached only through the human routes. This is stated, not hidden (limit L6b).
- **v3: contact routing (M12).** Before flight, the astronaut chooses a primary and an alternate crew contact and may exclude anyone, including the CMO when the CMO is the person concerned. Routing honours these choices without exception (I10).
- **v3: data governance (M19).**
  - The astronaut owns the data.
  - Default retention: mission + 1 year, then deletion unless the astronaut extends it.
  - Deletion on request.
  - A separate, opt-in **research donation** tier with its own consent, de-identified to the extent possible. Re-identification risk in astronaut cohorts is stated in the consent.
- **Implementation (ADR-0008).**
  - **Who decides.** The engine authorises every share as an outbox item whose `ShareQueued` output is the sharing-log entry. The transport may only acknowledge such items with `sent{share}`, so I6 is a property of the log.
  - **What is sent.** Payloads come from a closed integer schema (I2).
  - **Routing.**
    - Routes are fixed before flight, and the safety route has ≥ 2 recipients.
    - Exclusions are honoured without exception (I10).
    - During loss of signal, safety notices re-route to the CMO on board (F5). Summaries and PPC requests wait for their chosen recipient.
  - **Consent timing.** Tier-2 withdrawal cancels unsent summaries. Tier-3 shares follow the consent in force when the information arose.
- **Rejected: crew-level aggregated wellbeing view.** With 4–6 crew, no anonymisation technique (k-anonymity, differential privacy) can prevent re-identification; any "anonymous crew mood" is effectively individual disclosure.

### 4.10 Communication layer

- **State of the art.** Nocebo research: risk framing shapes symptom attribution; majority framing ("7 in 10 will not…") attenuates warning effects. Aviation alerting: Warning (immediate awareness and response), Caution (immediate awareness, subsequent response), Advisory (awareness, may need response); minimise nuisance alerts (14 CFR 25.1322).
- **Competitors.** Reason + action + source (AURORA); "what changed, how much, since when, why, what next" (RAKIA). Both good; neither specifies tone rules or audits them.
- **ANCHOR.**
  - **Template grammar:** Observation (number, personal reference) → Context (≥ 1 benign context when supported by data) → Step (concrete, optional) → Who (self / CMO / ground with current delay).
  - **Registers mapped to aviation taxonomy:** Advisory → calm insight; Caution → direct but calm, with a time frame; Warning → reserved for safety lines, ≤ 3 sentences, action first.
  - **Framing rules:** majority framing, personal reference, no disease names, no exclamation marks, no blame.
  - **Machine audit** of every template × slot combination (I3, I4).
  - **Orthosomnia guard:** nightly scores hidden by default; weekly trend shown.
  - **No guilt for missed days.**
- **v3: numbers people understand (M18).** No "σ" is ever shown. Deviations are expressed as personal percentiles ("lower than 9 of your last 10 weeks"), small multiples and plain-language confidence ("fairly sure", "early hint").
- **v3: counterfactuals.** Each insight states what would make it go away ("this note clears if your sleep returns to your normal for 3 nights"). This makes the system's reasoning inspectable and gives the astronaut a goal rather than a verdict.
- **v3: languages (M16).**
  - Templates, banned-term lists and audits are maintained per language, with English and Russian at minimum.
  - Scale anchors are validated per language.
  - Tone is reviewed by a panel of native-speaking clinicians in addition to the lexical audit (m2).
  - A readability limit applies per language.
- **Rejected: LLM-generated responses.** Non-deterministic, unverifiable for I3/I4, hallucination risk, and a 2026 commodity feature (multiple competitors ship chatbots). Templates are a deliberate safety choice, not a limitation of effort.
- **Implementation (ADR-0009).**
  - **Catalogue.** Every message comes from `app/src/catalogue.ts` and declares a register (mapped to the NASA-STD-3001 warning, caution and advisory colours), a step and an owner.
  - **Audit.** Every template is rendered with every slot value and checked for banned terms, exclamation marks, the step and owner, and sentence length (I3, I4). A coverage test runs the real WebAssembly engine and fails on any output that has no wording.
  - **Display.** The interface is a projection of the log, and the personal range is drawn from `DayStat`, so nothing on screen lacks support in the log (I5).

### 4.11 Interaction-as-sensor (exploratory, opt-in)

- **State of the art.** BiAffect: content-free keyboard metadata (speed, backspace rate, phone motion) predicted clinician-rated depression item scores (conditional R² = .63, pilot n small), predicted low-arousal states in suicidal ideation (npj Digit Med 2024), and sleep duration (2025).
- **ANCHOR.** With explicit opt-in, record content-free check-in metadata (completion time, slider revisions, journal typing cadence), stored only on device, used only for triangulation (§4.12), never as a standalone signal (I7).
- **Costs.** Weak validation outside clinical populations; device- and person-specific. Treated as low-weight evidence.

### 4.12 Triangulation

- **ANCHOR.** Persistent divergence between self-report and harder-to-fake measures (PVT, wearable sleep, interaction metadata) produces a gentle question ("Your reaction times have been slower this week while your mood ratings look steady. Would you like to look at it together?"). Reaction-time tests can be faked downward, not upward, which makes the asymmetry informative.
- **v3: display rules (M5).**
  - Detection works on *within-person change*, so a constant upward reporting bias (Mars-500 display rules) cancels out.
  - Wording never connects inputs to evaluation, selection or flight assignment.
  - What remains is bias that varies over time (e.g. hiding *more* as the mission goes on). This is listed as a limitation and is the main reason the triangulation exists.

### 4.13 Trust calibration and the coverage panel (C4)

- **Problem.** Trust should match capability (Lee & See 2004); over-trust produces misuse (Parasuraman & Riley 1997). A detector with multi-day delays will be read as "no alert ⇒ I am fine".
- **ANCHOR.**
  - The home screen never asserts wellness (I9). It shows a **coverage panel**: what is measured, the confidence, how old the data are, and an explicit list of what the system cannot see (e.g. "I don't see how you get on with the crew", "I can't detect sudden changes faster than a few days").
  - A weekly **self-review**, independent of any alert, asks the astronaut for their own judgement first and then shows the data. Disagreement is welcomed as information.
  - On day 0, onboarding explains the detection delays in plain language.
- **Why.** Calibrated trust is the condition under which a monitoring tool adds safety rather than replacing judgement.
- **Costs.** Slightly less reassuring. This is intentional.

### 4.14 Interpersonal support without surveillance (M10)

- **Problem.** Group interaction is among the top categories in astronaut journals, and the isolation-and-confinement hazard is largely interpersonal. Crew aggregation is excluded for re-identification reasons (§4.9).
- **ANCHOR.**
  - **Private conflict tools:** preparation and debrief worksheets for difficult conversations, modelled on the conflict content of the NASA-funded Virtual Space Station / Expedition-APPP.
  - **Dyadic opt-in signal:** two crewmates may agree to share one chosen signal with each other only (e.g. "I need a quiet evening"). This is symmetric, revocable, and never visible to anyone else or to the ground.
  - **PPC preparation:** the astronaut can assemble private notes and selected charts to bring to their Private Psychological Conference. This strengthens the existing human support channel rather than competing with it.
- **Costs.** No crew-level picture for the ground. That is a deliberate privacy choice, argued in §4.9.

### 4.15 Mission lifecycle modes (M3, M11)

| Mode | When | Purpose |
|---|---|---|
| Training | From crew assignment (months to years before launch) | Build habit and anchor; habituate to the app; detect reactivity; test N-of-1 countermeasures on Earth |
| Flight | Launch → landing | Full system |
| Return | Landing → ~6 months | Readaptation tracked against the anchor (NASA-STD-3001 V1 4013); support content for readjustment to work, family and society; gradual hand-off to ground support |
| Family bridge | Any phase, opt-in | Astronaut-controlled, non-clinical status for family ("good week", "busy week"), aligned with V1 6004 family support |

### 4.16 Clinician hand-off (M14)

- When the astronaut shares, data are exported as standard-format observations (HL7 FHIR Observation resources), each with provenance and the reasoning trail (which statistic, which threshold, which context tags).
- A separate **clinician view** shows effect sizes, confidence intervals and data completeness; the astronaut-facing language is not shown there.
- The flight surgeon receives what the astronaut chose to send, in a form they can audit.

### 4.17 Microgravity ergonomics and accessibility (M17, m5)

- Large touch targets, one-hand operation, layouts suited to a tethered tablet.
- No precision dragging: sliders use tap-to-set with confirmation.
- Full keyboard and voice-free operation; WCAG 2.2 AA.
- Usability validation in an analog habitat and, when possible, in parabolic flight.

### 4.18 The Calibration loop: training self-assessment (v4, headline contribution)

- **Problem.** The challenge asks for software that lets astronauts "evaluate and act on the status of their health". Self-evaluation is only as good as the evaluator.
  - Under 14 days of 6-h sleep, cognitive deficits accumulated to the level of 2 nights of total deprivation (Van Dongen et al. 2003). Subjective sleepiness rose only modestly and did not track the decline; this point is to be confirmed against the full text before publication.
  - Astronauts average ~6 h of sleep in flight (Barger 2014). The operating condition of spaceflight is therefore exactly the one in which people misjudge their own impairment.
- **State of the art.** 2B-Alert individualises a performance model from PVT results and becomes accurate after ~12 tests. RST measures alertness. **No system measures or trains the astronaut's own judgement.** In the corpus: 0 of 243.
- **ANCHOR design.**
  1. **Predict first.** Before each PVT (2–3×/week), the astronaut gives a 2-second prediction: "How sharp are you right now?" (0–100), and optionally predicts their lapse count.
  2. **Measure.** The PVT result.
  3. **Score the judgement:**
     - *Bias* is the mean of predicted − actual percentile, both expressed on the person's own scale.
     - *Resolution* is the within-person correlation between predictions and outcomes.
     - Both are tracked over time with Bayesian shrinkage.
  4. **Feed back without judgement:** "On short-sleep days you tend to rate yourself sharper than your test shows." This is shown as a personal pattern, never as a grade.
  5. **Use it.**
     - Calibrated subjective ratings carry more weight in the personal model.
     - Systematic over-confidence under sleep debt raises the expected value of acting in the Action loop (§4.19), because the astronaut is least able to notice that need themselves.
     - Before high-stakes tasks (EVA, robotic arm operations), the app offers a quick "check your sense against a test" moment.
- **Why this is the right lever.** It directly serves the challenge's core verb (self-evaluate). It strengthens the astronaut instead of replacing their judgement, the opposite of automation complacency (§4.13). It also improves data quality for every other loop.
- **Costs.**
  - About 2 s per PVT session.
  - Possible reactivity: people may "game" predictions. Mitigation: predictions are never shared and never scored as good or bad.
  - Calibration is task-specific: PVT calibration may not transfer to complex tasks, so claims are limited to vigilance.
- **Validation hypothesis (pre-registered, S2/S3).** Calibration bias under sleep restriction decreases over 8 weeks of feedback compared with a prediction-only control.

### 4.19 Expected-utility alerting (v4)

- **Problem.** Threshold alerts fire whenever evidence crosses a line, whether or not anything useful can be done now. That generates interruptions, nocebo burden and alarm fatigue (14 CFR 25.1322 requires minimising nuisance alerts). HELIOS reduced false alarms by raising thresholds, which cost 4 days of median delay. This is a pure detection trade-off with no reference to action.
- **State of the art.** Decision-theoretic alerting (Horvitz et al., UAI 1999; CACM 2003) computes the expected value of transmitting an alert now against the expected cost of interruption and the value of the person seeing it later. It has not been applied to astronaut health (0/243).
- **ANCHOR design.** For each candidate insight, compute

  $$\mathrm{EVTA} = p\,\big[B(a,\text{now}) - B(a,\text{next touchpoint})\big] - C_{\text{int}}(\text{context}) - C_{\text{noc}}(p)$$

  where:
  - $p$ is the posterior probability of a real change (BOCPD/CUSUM).
  - $B(a,\cdot)$ is the expected benefit of the best available action $a$ if taken now or at the next natural touchpoint (normally the next morning check-in). It is derived from the action's evidence-graded effect size (§4.8), the predicted cost of delay (§4.20), and the astronaut's calibration bias (§4.18).
  - $C_{\text{int}}$ is the interruption cost, high during scheduled operations and sleep, from the mission timeline.
  - $C_{\text{noc}}(p)$ is a nocebo/anxiety cost, larger when $p$ is low.
- **Policy.**
  - **Interrupt only if EVTA > 0.**
  - Otherwise defer the insight to the next check-in (batching). Nothing is lost; it is only delivered at a calmer moment.
  - **If no effective action exists, do not interrupt.** The insight goes to the weekly review ("actionability gating").
  - Safety lines bypass this entirely (kernel).
- **Why.** It separates detection (how sure) from interruption (is it worth it now). Alert burden can then fall **without** raising detection thresholds, because detection still happens on time and only the delivery moment changes.
- **Costs.**
  - The utilities are estimates. Mitigation: conservative defaults, the astronaut can tune the interruption cost per context, and a sensitivity analysis is reported.
  - Mis-specified benefit could defer something urgent. Mitigation: anything that is urgent belongs on the safety-line table, not in the utility model.
- **Metric.** Interruptions per person-month and time in alert, against action benefit retained, compared with threshold policies on the same benchmark (§12).

### 4.20 Personal performance forecasting (v4)

- **State of the art.**
  - The Unified Model of Performance with Bayesian individualisation (2B-Alert) predicts PVT performance from sleep and caffeine history and personalises after ~12 PVTs. It is validated prospectively in a 62-h sleep-deprivation study (n = 21) and in caffeine-optimisation studies.
  - In the corpus, one entry uses a non-individualised two-process model.
- **ANCHOR design.**
  - Individualised performance model from sleep timing, circadian phase (§4.6) and PVT history, with predictive intervals.
  - Uses:
    1. **Forecast** the mission timeline ahead: "on EVA day, predicted lapses are in the top 10% of your range unless sleep improves by Wednesday".
    2. Supply $B(a,\cdot)$ to the EVTA calculation.
    3. Provide the "actual" reference in the Calibration loop between tests.
    4. Plan sleep shifts.
- **Costs.** The model is validated for total and partial sleep deprivation on Earth, and microgravity effects on its parameters are unknown. Mitigation: individualisation absorbs some of this, and forecast intervals widen when recent prediction errors grow (an online coverage check: if the 80% intervals miss more than 20% of tests, the intervals are inflated and the astronaut is told).
- **Rejected.** Caffeine dosing recommendations. They are a medication-adjacent recommendation, and caffeine sensitivity is unmodelled (a stated limitation of 2B-Alert 2.0).

### 4.21 No composite score (v4)

- **Field practice.** 28 of 243 entries reduce health to a single readiness, health or risk score; HELIOS uses "100 minus penalties".
- **ANCHOR position.** No composite score.
  - A single number invites score-chasing and anxiety, the orthosomnia pattern in which people optimise the metric rather than their state.
  - It hides which lever matters, which conflicts with actionability gating.
  - It implies a clinical validity that no entry has. HELIOS's own README says its readiness "is not a clinically validated score".
  - ANCHOR shows per-domain status, with coverage and confidence, and one recommended action.
- **Cost.** Less glanceable. Mitigation: the home screen leads with *the single most valuable action today*, which is what a glance should deliver.

---

## 5. Space-grade system architecture

### 5.1 Classification
Under NPR 7150.2 the companion is Class C/D mission-support software, not Class A. Functions traceable to a hazard analysis (safety lines) are safety-critical under NASA-STD-8739.8. **Only the kernel** carries that burden; this is why it is isolated.

### 5.2 Runtime assurance (Simplex / ASTM F3269 pattern)

```
input ─► validate ─► append-only log (2 copies, CRC + hash chain)
                          │
          ┌───────────────┴───────────────────────────┐
          ▼                                           ▼
  SAFETY KERNEL  (verified, runs first)       COMPANION (complex, unverified)
  ─ safety table (3 CRC-checked copies)       baseline → CUSUM/drift → BOCPD
  ─ evaluate 3× and majority-vote (TMR)       circadian → phase engine
  ─ disagreement ⇒ fail-safe output           bandit → language → UI
  ─ writes safety output + share event        exploratory: resilience, interaction
          │                                           │
          └──────────────► UI ◄───────────────────────┘
  watchdog: companion exception/timeout ⇒ degraded mode (kernel + data entry)
```

- **Kernel properties.** About 300 lines; no allocation after start; bounded loops; no recursion; ≥ 2 assertions per function (Holzmann's Power of Ten); no dependency on the companion.
- **Software TMR.** The kernel evaluates each input three times against three independently stored, CRC-verified copies of the safety table and majority-votes (SWIFT-R principle: software-only triple redundancy with voting). Cost is negligible at microseconds per evaluation, so we take full TMR rather than detection-only duplication.
- **Fail-safe direction.** If the votes disagree, the output is "safety check could not complete; please re-measure / contact CMO". The system never silently passes.
- **v3: what TMR does not cover (M9).** Three executions of the same code on the same processor mask transient data faults (single-event upsets) but reproduce a logic bug three times. v3 adds two further layers:
  - **Design diversity (N-version):** two kernel implementations written independently from the same formal specification. Their outputs are compared, and disagreement takes the fail-safe path.
  - **Exhaustive verification:** the kernel's input space is finite (safety table × discretised input values × consent flags), so the specification is checked against every combination, not a sample.
  - Together: TMR covers radiation, diversity covers common logic errors, and exhaustive checking covers specification compliance.
  - **Residual risk:** an error in the specification itself. Mitigation: the specification is reviewed by the flight surgeon who signs the safety table.

### 5.3 Event-sourced, integrity-checked storage
- **Single source of truth:** an append-only log of raw records. Each record carries a CRC32 and a hash link to its predecessor; writes are atomic (write-ahead then commit).
- **Redundancy:** two copies on separate storage, with periodic scrubbing; repair from the copy whose chain verifies.
- **Derived state** (baselines, CUSUM, posteriors, bandit) is recomputed by replay: ~10 records/day × 3 years ≈ 11,000 records, replayed in well under a second.
- **Encryption at rest:** AES-GCM with a key derived from the astronaut's PIN, adopting AURORA's well-chosen hygiene.

### 5.4 Determinism and time
- The core consists of pure functions; random draws in Thompson sampling come from a seeded PRNG stored in the log, so replay is exact (I5).
- No wall-clock reads in the core. Each record carries mission elapsed time and GMT (the ISS reference); non-monotonic timestamps are rejected.

### 5.5 Performance
Every update is O(1): conjugate, CUSUM and BOCPD with a truncated run length, plus EWS on a 21-day window. Per check-in compute is in microseconds to low milliseconds; the N2 budget (< 100 ms) is dominated by rendering. No network, no model inference server, no LLM.

### 5.6 FMEA (abridged)

| Failure | Detection | Mitigation |
|---|---|---|
| Crash mid-write | CRC fails | Write-ahead/commit; re-enter last entry |
| Bit flip in a log record | CRC/hash mismatch | Repair from second copy |
| Bit flip in the safety table | CRC on each of 3 copies; TMR vote | Vote masks a single fault; double fault ⇒ fail-safe output |
| Bit flip in derived state | Rebuilt at start | Replay |
| Companion exception/timeout | Watchdog | Degraded mode: kernel + data entry |
| Clock error | Monotonicity check | Reject; mission elapsed time is primary |
| Implausible input | Range check | Reject with explanation |
| Weak prior / data gaps | Posterior width, missingness CUSUM | Low-confidence label; "welcome back" |
| Storage full | Free-space check | Log is tiny; early notice |

### 5.7 Threat model and device assumptions (C3)

ISS crews use shared laptops (100+ on board, mostly ThinkPads) and a few iPads for procedures. Personal devices are segmented and tightly scheduled. ANCHOR therefore assumes **no private hardware**.

| Actor | Threat | Control |
|---|---|---|
| Curious crewmate | Reads the screen or device | Per-astronaut encrypted vault usable on any crew device; quick lock; auto-lock; no content in notifications or on the lock screen |
| Ground operator / IT | Accesses stored data during maintenance or sync | Data never synced by default; vault encrypted with a key the ground does not hold |
| Post-mission access | Data used for selection or assignment | Contract, retention limits, deletion; research use only via separate opt-in |
| Device loss or theft | Training devices were stolen in 2023 | Encryption at rest; hardware-backed key where available, PIN-derived key with rate limiting as fallback |
| Tampering | Altered records or safety table | Hash chain, CRC, N-version kernel, signed safety table |

Out of scope: a fully compromised operating system. This is stated, not assumed away.

### 5.8 Relation to flight software practice
The kernel/companion split mirrors cFS practice, where a Health & Safety application monitors applications and services watchdogs independently of mission applications. The prototype runs as an offline web app (TypeScript) for demonstration. The kernel is written so it can be ported to C under the same rules.

---

## 6. Fundamental limits

| ID | Limit | Governing condition | Design response |
|---|---|---|---|
| L1 | No detector is both instant and silent | Delay scales as $\log(\mathrm{ARL}_0)/I$ with $I=\delta^2/2$ (Lorden, asymptotic) | Published delays (§4.3); acute changes go to safety lines; small changes go to drift review |
| L2 | Slow decline ≡ adaptation from in-flight data alone | Requires an external reference | Pre-flight anchor + consented re-anchoring (§4.4) |
| L3 | Deliberate misreporting is undetectable from self-report alone | Needs a channel that is unfakeable in the favourable direction | Triangulation (§4.12); remove the motive (§4.9) |
| L4 | No astronaut ground truth | Validity must come from proxies | Synthetic missions + real human datasets (§12); claims bounded accordingly |
| L5 | Software cannot be proven error-free by testing (Dijkstra; Rice) | Guarantees are possible only for small, simple components | Guarantees stated for the kernel only; everything else fails safe |
| L6 | Privacy vs. operational safety is irreducible | Must be settled before flight by the people who bear it | Disclosure contract (§4.9) |
| L7 | Cold start | Posterior SD ∝ $1/\sqrt{\kappa_0+n}$ | Hierarchical prior; confidence display |
| L8 | Early-warning signals are weak predictors | Sensitivity ~33% / specificity ~84% in prospective studies | Exploratory display only (I7) |
| L9 | Behaviour change needs adherence | Users rarely follow schedules (Entrain study) | One suggestion/day; personal learning of what is followed |
| L6b | High-stakes disclosure vs. automatic protection | Any automatic share reduces disclosure; any non-share leaves imminent risk to human routes | Graduated protocol with per-item pre-consent (§4.9, I8) |
| L10 | Monitoring may change what it measures (reactivity) | Effect size unknown for this population | Reactivity monitoring, quiet mode, analog testing (§4.1) |
| L11 | Time-varying concealment cannot be separated from true improvement by self-report | Requires an independent channel | Triangulation (§4.12); contract removes the motive (§4.9) |

---

## 7. End-to-end flow

```
DAY 0      disclosure contract shown · safety lines confirmed · pre-flight anchor loaded
TRAINING   months of use ─► habit + context-conditioned anchor + reactivity check + N-of-1 on Earth
MORNING    import (RST/PVT, wearable) + ≤90 s check-in ─► validate ─► log ─► KERNEL (N-version + TMR) ─(line)─► safety output + visible share
           (if evidence > h/2: offer one extra 20 s check-in later that day)
PVT DAYS   predict own sharpness (2 s) ─► PVT ─► calibration update ─► performance-model update ─► forecast
ANY TIME   candidate insight ─► EVTA > 0 ? interrupt now : defer to next check-in / weekly review
                                      └► companion:
                                         baseline update (untagged) ─► CUSUM ─┐
                                         anchor drift ────────────────────────┤
                                         missingness ─────────────────────────┼─► rule table ─► escalation (delay-aware)
                                         triangulation ───────────────────────┘
                                         circadian phase ─► timing suggestion
                                         phase engine + bandit ─► ≤1 suggestion/day
                                         template ─► audit ─► 1 insight (with confidence)
DAY        at most 1 proactive suggestion (budget, receptivity)
EVENING    optional private journal
WEEKLY     self-review first · coverage panel · trends · learned patterns · resilience trend (private)
           · re-anchor prompts · threshold recalibration · share decision · optional PPC preparation
RETURN     readaptation vs anchor · hand-off to ground support · retention / deletion choice
```

---

## 8. Decision dependency map

```
 population prior ─┐
 pre-flight data ──┴─► 4.2 BASELINE ──┬─► 4.3 CUSUM (budget) ──┐
                                      ├─► 4.4 ANCHOR DRIFT ────┤
 mission timeline ─► context tags ────┘                        ├─► escalation ─► 4.10 LANGUAGE ─► UI
                    4.7 PHASE ENGINE ───────────► 4.8 BANDIT ──┤
 sleep/wearable ──► 4.6 CIRCADIAN ──────────────► 4.8 ─────────┤
 4.11 interaction ─► 4.12 TRIANGULATION ──────────────────────┘
 4.5 RESILIENCE ─► weekly reflection only (I7)
 4.9 CONTRACT ─► governs every outbound edge · KERNEL ─► independent of all of the above
```

---

## 9. Decision register (what is in, what is out, and why)

| Item | Decision | Reason |
|---|---|---|
| Bayesian anchored baseline | **In** | Only structure robust to both baseline failure modes |
| CUSUM with false-alert budget | **In** | Controllable, optimal, explainable, verifiable |
| BOCPD | **In (secondary)** | Change-point timing for explanations; no closed-form false-alert rate |
| Isolation Forest | **Out** | No false-alert control, opaque, needs training data we lack; already a commodity |
| Resilience dynamics (EWS) | **In (exploratory only)** | Meaningful as "recovery speed"; too weak to alert (L8) |
| Circadian companion | **In** | Largest evidenced problem; nobody else addresses it |
| Mission-phase engine | **In** | Documented third-quarter and blackout effects |
| Thompson-sampling personalisation | **In (constrained)** | Deployed in HeartSteps; explainable; bounded exploration |
| Expressive journal | **In (as outlet, not treatment)** | Astronaut-reported value; small clinical effect stated honestly |
| Interaction-as-sensor | **In (opt-in, exploratory)** | Promising (BiAffect), weakly validated |
| Template language + audit | **In** | Determinism, invariants I3/I4 |
| LLM chatbot | **Out** | Non-deterministic, unverifiable, hallucination risk, commodity |
| Voice/face emotion analysis | **Out** | Privacy, weak validity, contradicts the contract |
| Streaks/badges/points | **Out** | Guilt and abstinence-violation evidence |
| Crew aggregate mood view | **Out** | Re-identification unavoidable at n ≤ 6 |
| Mission Control "see all" dashboard | **Out** | Recreates the underreporting incentive |
| Radiation dose tracking as a feature | **Out (context only)** | Well covered by flight dosimetry and by AURORA; solar particle events enter only as context tags |
| Real sensor integration | **In (via import, v3)** | Imports from flight-certified sources (actigraphy, Bio-Monitor exports, RST) rather than direct consumer BLE pairing, which would add an uncertified radio path |
| Encryption at rest, hash chain, range checks | **In (hygiene)** | Adopted from field best practice; not claimed as novelty |
| Adaptive sampling intensity | **In (v3)** | Faster detection; burden only when evidence accumulates |
| Bootstrap-calibrated system threshold | **In (v3)** | Exact false-alert budget under real correlation |
| Automatic share of any self-harm answer | **Out (v3)** | Recreates the underreporting incentive; replaced by the graduated protocol |
| Coverage panel, never "all good" | **In (v3)** | Calibrated trust; counters complacency |
| N-version kernel + exhaustive verification | **In (v3)** | TMR alone does not cover logic errors |
| Lifecycle modes (training, return, family bridge) | **In (v3)** | Better anchor; covers NASA-STD-3001 pre/post requirements |
| Dyadic opt-in signal | **In (v3)** | Interpersonal support without aggregation |
| N-of-1 self-experiments | **In (v3)** | Autonomy and competence; suits trained experimenters |
| FHIR clinician export | **In (v3)** | Auditable hand-off in a standard format |
| Duplicate measurement of RST/Cognition | **Out (v3)** | Import instead of re-asking |
| Self-assessment calibration loop | **In (v4)** | Targets the metacognitive gap; 0/243 |
| Expected-utility alerting with actionability gating | **In (v4)** | Lowers interruptions without delaying detection; 0/243 |
| Individualised performance forecasting | **In (v4)** | Forecasts the mission timeline ahead; feeds EVTA and calibration |
| Uncorrected pre/post "what worked" | **Out (v4)** | RTM bias: +1.71 SD spurious effect in simulation |
| Composite readiness score | **Out (v4)** | Score-chasing, hides levers, false clinical validity |
| Caffeine dosing advice | **Out (v4)** | Medication-adjacent; sensitivity unmodelled |

---

## 10. Novel contributions

1. Anchored hierarchical baseline with consented re-anchoring, solving both the frozen-baseline and the shifting-baseline failures.
2. False-alert budget → thresholds → predicted detection delays, stated and verified numerically.
3. Disclosure contract as an incentive-compatible privacy mechanism against documented underreporting.
4. Runtime-assurance safety kernel with software TMR, independent of the companion.
5. Circadian companion fused with the mission timeline and habitat lighting modes.
6. Resilience dynamics as a private, non-alerting self-insight, with the evidence-based reason it does not alert.
7. Evidence ledger: every rule, threshold and intervention carries its citation, effect size and evidence grade, so weak evidence is labelled weak.
8. Validation on open real-world longitudinal human data in addition to synthetic missions.
9. Adaptive sampling intensity (v3).
10. Per-person bootstrap-calibrated false-alert budget (v3).
11. Graduated, pre-consented protocol for high-stakes disclosure (v3).
12. Coverage panel and counterfactual explanations for calibrated trust (v3).
13. N-version, exhaustively verified safety kernel (v3).
14. Full mission lifecycle with a training-built anchor (v3).
15. Personal N-of-1 self-experiments (v3), justified in v4 by quantified RTM bias.
16. **Calibration loop**: measuring and training the astronaut's self-assessment (v4).
17. **Expected-utility alerting** with actionability gating (v4).
18. **Individualised performance forecasting** integrated with alerting and calibration (v4).
19. **ANCHOR-Bench**: an open benchmark released for the field (v4, §12).

*Status against the 243-entry corpus: items 3, 4, 11, 13, 15, 16, 17 and 19 are absent from every examined entry. Items 2, 9, 10 and 14 are increments on convergent practice (see §2).*

**Regulatory position (M15).** ANCHOR *informs* the management of *non-serious* situations and never diagnoses or drives treatment. This is the lowest-risk category of the IMDRF Software-as-a-Medical-Device framework. Any feature that would move it to "drive" or "diagnose" is excluded by design (template grammar, I3). The safety kernel's outputs route to humans; they do not prescribe.

---

## 11. Alignment with judging criteria and data requirement

| Criterion | How ANCHOR addresses it |
|---|---|
| Impact | Targets the most prevalent in-flight problems (sleep, circadian, behavioural decline) and the disclosure problem that undermines all monitoring |
| Creativity | Eight contributions (§10) absent from the 2026 field |
| Validity | Equations, simulated verification, real-data validation, explicit limits (§6) |
| Relevance | Covers every objective in the challenge text: indicators, self-assessment, guidance, when to involve crew or flight surgeons, UI with body figure and colour coding |
| Presentation | One story: "a companion an astronaut trusts enough to tell the truth" |

NASA/space-agency data used: OSDR RadLab and NOAA SWPC solar particle event dates (context tags), NASA HRP evidence (thresholds, phase effects), ISS lighting schedule (circadian module). The challenge Resources tab opens on **13 Nov** and must be reviewed for additional datasets.

---

## 12. Verification and validation

**Verification (invariants)**
- I1: exhaustive safety table × boundary values × injected companion failures × injected table bit flips.
- I2, I6, I7: property-based tests over random configurations.
- I3, I4: audit of every template × random slot values.
- I5: replay determinism, including after corruption and repair.
- Storage: fault injection (bit flips, truncated writes).

**Validation**

| Dataset | Use | Metric |
|---|---|---|
| Synthetic 180-day missions (≥ 4 personas; third-quarter dip, sleep restriction, real-date solar particle event, slow drift 0.02–0.1σ/day, healthy control) | Known-truth detection | False alerts per 30 d on control; detection delay vs theory; **main experiment: drift missed without anchor, caught with anchor** |
| **LifeSnaps** (n = 71, ~4 months, Fitbit + 3×/day mood EMA + weekly PANAS/STAI; open, Zenodo) | Real human daily time series | Empirical false-alert rate under the budget on real noise; baseline stability; missingness behaviour |
| **GLOBEM** (497 participants, 4 years, weekly PHQ-4, end-of-term BDI-II; open sample n = 40) | Real labels of worsening | Does sustained-deviation/drift precede PHQ-4 worsening? Report sensitivity, specificity and lead time honestly, including null results |
| Language audit | Every template | 0 violations |

**v4: ANCHOR-Bench (an open benchmark for the field).** Entries in the corpus each report results on their own private simulators, so their numbers cannot be compared. ANCHOR-Bench releases:

- **A versioned generator.** It produces synthetic crews with between-person spread from real data, AR(1) noise, bounded scales, missingness that is not at random, context events, slow drift, third-quarter dips, and **countermeasures with known true effects** (including zero).
- **Scoring:**
  - Detection rate.
  - Median delay.
  - False alerts per person-month.
  - Time in alert.
  - **Interruptions per person-month.**
  - **Action benefit retained.**
  - **Bias of "what worked" estimates against the known true effects.**
  - **Calibration-feedback effect.**
- **Reference policies:** threshold, EWMA (HELIOS-style settings), CUSUM with per-person calibration, and EVTA.

Releasing the benchmark turns ANCHOR's claims into ones anyone can check, and gives the field a common yardstick.
| Usability walkthrough | Check-in | ≤ 90 s |

---

## 13. Limitations
- No astronaut data. Real-world validation uses terrestrial students and adults whose stressors differ.
- Safety-line thresholds are illustrative and require clinical ownership.
- The circadian estimate lacks measured light exposure.
- The independence assumptions in §4.3–4.4 are corrected only to first order (AR(1)).
- Intervention effects are taken from terrestrial evidence.
- Expressive writing and Earth-view benefits are weakly evidenced and are labelled as such.
- Time-varying concealment cannot be fully separated from improvement (L11).
- Measurement reactivity in this population is unknown (L10).
- The high-stakes protocol leaves non-consenting astronauts at imminent risk to human routes only (L6b).
- A fully compromised operating system is outside the threat model.
- The combined effect of bootstrap calibration and adaptive sampling (~5 days for 1σ) is estimated, not yet jointly simulated.

---

## 14. Staged validation and deployment roadmap (M6)

Each stage has pre-registered hypotheses, explicit go/no-go criteria, and reports null results.

| Stage | Setting | Hypotheses | Go / no-go |
|---|---|---|---|
| S0 | Synthetic missions | Detector meets the false-alert budget; the anchor catches drift that the rolling baseline misses; all invariants hold | All invariants pass; realised budget within ±10% |
| S1 | Open data: LifeSnaps, GLOBEM | Budget holds on real human noise; deviations precede PHQ-4 worsening above chance | Realised false-alert rate ≤ budget; lead time reported with CI |
| S2 | Terrestrial pilot (shift workers, polar or offshore crews) | Check-in ≤ 90 s; retention ≥ 70% at 8 weeks; no measurable negative reactivity; language audit accepted by users | Retention and SUS ≥ 70; no harm signal |
| S3 | Space analog (HERA, CHAPEA-type, 22-min delay) | Delay-aware escalation works; the disclosure contract increases honest reporting vs. the standard condition | Analog PI and BHP review |
| S4 | Antarctic winter-over | Long-duration phase effects; circadian module under extreme photoperiod | Same as S3 plus circadian accuracy vs. actigraphy |
| S5 | ISS technology demonstration | Operational acceptability; kernel behaviour on flight hardware | NASA software classification and assurance review |

**Design-change control.** Every statistical parameter (budget, window, thresholds) is versioned, and every change is evaluated by replaying the stored logs before release (I5 makes this exact).

---

## References

**Space medicine and operations**
- NASA Mars Medical System ConOps — [NTRS 20200001715](https://ntrs.nasa.gov/api/citations/20200001715/downloads/20200001715.pdf)
- EIMO ConOps — [NTRS 20250000009](https://ntrs.nasa.gov/citations/20250000009)
- NASA-STD-3001 Vol 1 Rev C — [PDF](https://www.nasa.gov/wp-content/uploads/2023/11/nasa-std-3001-vol-1-rev-c-with-signature.pdf)
- OCHMO Mission Duration — [PDF](https://www.nasa.gov/wp-content/uploads/2023/12/ochmo-tb-007-mission-duration.pdf)
- OCHMO-TB-016 Behavioral Health — [PDF](https://www.nasa.gov/wp-content/uploads/2023/12/ochmo-tb-016-behavioral-health.pdf)
- RST/PVT-B — [Sleep 2022](https://academic.oup.com/sleep/article/45/3/zsac006/6505235), [NASA Spinoff 2013](https://spinoff.nasa.gov/Spinoff2013/hm_1.html)
- Cognition — [UPenn](https://www.med.upenn.edu/uep/cognition.html), [Frontiers Physiol 2024](https://www.frontiersin.org/journals/physiology/articles/10.3389/fphys.2024.1451269/full)
- [NASA/TM-20210021987 Standard Measures](https://ntrs.nasa.gov/api/citations/20210021987/downloads/NASA_TM20210021987.pdf)
- [Astroskin](https://hexoskin.com/pages/astroskin-vital-signs-monitoring-platform-for-advanced-research)
- [NASA Five Hazards](https://www.nasa.gov/stem-content/five-hazards-of-human-spaceflight/)
- Stuster Journals — [NASA](https://www.nasa.gov/mission_pages/station/research/news/Stuster_Journals.html), [HRP BMed Evidence Report](https://humanresearchroadmap.nasa.gov/evidence/reports/BMed.pdf)
- Mars-500 — [Basner PMC3968121](https://pmc.ncbi.nlm.nih.gov/articles/PMC3968121/), [Wang PLoS ONE 2014](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0087087)
- Underreporting — [IOM Safe Passage ch. 6](https://www.nationalacademies.org/read/10218/chapter/8), [Mil Med 2023](https://academic.oup.com/milmed/article/188/3-4/e446/6761438), [CSMonitor 2007](https://www.csmonitor.com/2007/0730/p03s01-ussc.html)
- Virtual Space Station — [HRR task](https://humanresearchroadmap.nasa.gov/tasks/task.aspx?i=1176), [Carter et al.](https://www.researchgate.net/publication/7799628_An_interactive_media_program_for_managing_psychosocial_problems_on_long-duration_spaceflights), [Dartmouth 2008](https://geiselmed.dartmouth.edu/news/2008/09/26_buckey.shtml)
- CIMON — [MIT Tech Review](https://www.technologyreview.com/2020/01/14/64990/an-emotionally-intelligent-ai-could-support-astronauts-on-a-trip-to-mars/), [Space.com](https://www.space.com/astronauts-artificial-intelligence-companions-deep-space-missions)

**Sleep and circadian**
- Barger et al., Lancet Neurol 2014 — [abstract](https://www.thelancet.com/journals/laneur/article/PIIS1474-4422(14)70122-X/abstract)
- Brainard et al., SSLA, Acta Astronautica 2013 — [ADS](https://ui.adsabs.harvard.edu/abs/2013AcAau..92...21B/abstract)
- Dynamic lighting 45-day analog — [PMC11316501](https://pmc.ncbi.nlm.nih.gov/articles/PMC11316501/), [Sleep Advances 2024](https://academic.oup.com/sleepadvances/article/5/1/zpae032/7685068)
- [HRP Sleep Evidence Report](https://humanresearchroadmap.nasa.gov/Evidence/reports/Sleep.pdf)
- Optimal adjustment of the human circadian clock — [PLoS Comput Biol](https://journals.plos.org/ploscompbiol/article?id=10.1371%2Fjournal.pcbi.1008445&rev=1)
- Level-set Kalman filter for circadian phase — [arXiv 2207.09406](https://arxiv.org/pdf/2207.09406)
- OASIS trial, Freeman 2017 — [PubMed 28888927](https://pubmed.ncbi.nlm.nih.gov/28888927/)
- Orthosomnia — [PMC11592250](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11592250/), [Sleep 2026](https://academic.oup.com/sleep/article/49/Supplement_1/A246/8674102), [JMIR 2025](https://www.jmir.org/2025/1/e68816/XML)

**Affective dynamics and measurement**
- van de Leemput et al., PNAS 2014 — [PDF](https://www.pnas.org/doi/pdf/10.1073/pnas.1312114110); Bos & De Jonge critique — [PNAS](https://www.pnas.org/doi/10.1073/pnas.1323672111)
- Wichers et al. 2016 — [Karger](https://karger.com/pps/article/85/2/114/294376/Critical-Slowing-Down-as-a-Personalized-Early)
- Prospective EWS tests — [Smit et al.](https://www.researchgate.net/publication/355175280_Detecting_impending_symptom_transitions_using_early_warning_signals_in_individuals_receiving_treatment_for_depression), [Bos 2022 bipolar](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC9471093/), [Critical slowing down in momentary affect](https://www.researchgate.net/publication/388752577_Critical_Slowing_Down_in_Momentary_Affect_as_Early_Warning_Signal_of_Impending_Transitions_in_Depression)
- EWMA for EMA depression onset — [Research Square](https://assets-eu.researchsquare.com/files/rs-8960944/v1_covered_34ebc2ea-11a1-462a-ac15-5d28d38e3d42.pdf?c=1773231324)
- VAS and single items in EMA — [ResearchGate](https://www.researchgate.net/publication/388765970_Accuracy_and_Consistency_of_Visual_Analog_Scales_in_Ecological_Momentary_Assessment_and_Digital_Studies), [arXiv 2502.02846](https://arxiv.org/pdf/2502.02846), [Bristol brief EMA](https://research-information.bris.ac.uk/en/publications/ecological-momentary-assessment-of-mood-reliability-and-validity-/)
- BiAffect — [JMIR 2018](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC6076371/), [npj Digit Med 2024](https://www.nature.com/articles/s41746-024-01048-1), [JMIR 2024 backspace](https://www.jmir.org/2024/1/e51269), [npj 2025 rhythms](https://www.nature.com/articles/s41746-025-02254-1)

**Statistics and personalisation**
- Page 1954; Lorden, Ann Math Stat 1971; Moustakides, Ann Stat 1986; Siegmund, *Sequential Analysis*, 1985
- Adams & MacKay, BOCPD — [arXiv 0710.3742](https://arxiv.org/pdf/0710.3742)
- Hierarchical online learning in mHealth — [arXiv 2511.08719](https://arxiv.org/pdf/2511.08719)
- Bayesian adaptive N-of-1 — [arXiv 1911.00878](https://arxiv.org/pdf/1911.00878)
- HeartSteps MRT, Klasnja 2019 — [Ann Behav Med](https://academic.oup.com/abm/article/53/6/573/5091257); HeartSteps algorithm evaluation — [arXiv 2501.02137](https://arxiv.org/pdf/2501.02137)
- JITAI, Nahum-Shani 2018 — [ResearchGate](https://www.researchgate.net/publication/308535769_Just-in-Time_Adaptive_Interventions_JITAIs_in_Mobile_Health_Key_Components_and_Design_Principles_for_Ongoing_Health_Behavior_Support)

**Engagement, framing, interventions**
- SDT — [Gillison 2019](https://www.tandfonline.com/doi/full/10.1080/17437199.2020.1718529), [mHealth engagement overview](https://pmc.ncbi.nlm.nih.gov/articles/PMC13398183/)
- Attrition — [Eysenbach 2005](https://www.jmir.org/2005/1/e11/PDF), [Baumel 2019](https://www.jmir.org/2019/9/e14567/PDF)
- Gamification harms — [Hadi Mogavi 2022](https://arxiv.org/pdf/2203.16175)
- Nocebo framing — [Scoping review 2026](https://pmc.ncbi.nlm.nih.gov/articles/PMC13205298/), [Ann Behav Med 2021](https://pmc.ncbi.nlm.nih.gov/articles/PMC8517218/)
- Expressive writing — [Frattaroli 2006 summary](https://steps.org/sources/frattaroli-2006-experimental-disclosure-meta-analysis/), [Reinhold 2018](https://paulbuerkner.com/publications/pdf/2018__Reinhold_et_al__Clinical_Psychology_Science_and_Practice.pdf)
- Overview effect, Yaden 2016 — [PDF](https://static1.squarespace.com/static/53d29678e4b04e06965e9423/t/59397a7b20099e3818f062c3/1496939132423/the+overview+effect.pdf)
- Flightcrew alerting — [14 CFR 25.1322](https://www.ecfr.gov/current/title-14/chapter-I/subchapter-C/part-25/subpart-F/subject-group-ECFR3043e8882f16d6e/section-25.1322), [AC 25.1322-1](https://www.faa.gov/documentLibrary/media/Advisory_Circular/AC_25.1322-1.pdf)

**Software assurance and reliability**
- NPR 7150.2D App. D — [NODIS](https://nodis3.gsfc.nasa.gov/displayDir.cfm?Internal_ID=N_PR_7150_002D_&page_name=AppendixD); [SWEHB 7.02](https://swehb.nasa.gov/spaces/SWEHBVB/pages/32604319/7.02+-+Classification+and+Safety-Criticality); [NASA-STD-8739.8](https://snebulos.mit.edu/projects/reference/nasa-generic/nasa-std-8739-8.pdf)
- Holzmann, Power of Ten — [ACM](https://dl.acm.org/doi/10.1109/MC.2006.212)
- Runtime assurance — [arXiv 2110.03506](https://arxiv.org/pdf/2110.03506), [ASTM F3269](https://www.researchgate.net/publication/348242885_ASTM_F3269_-_An_Industry_Standard_on_Run_Time_Assurance_for_Aircraft_Systems), [Verified monitors, arXiv 2110.11755](https://arxiv.org/pdf/2110.11755); Sha, IEEE Software 2001
- SWIFT, Reis 2005 — [ResearchGate](https://www.researchgate.net/publication/4129281_SWIFT_software_implemented_fault_tolerance); soft-error protection survey — [MDPI](https://www.mdpi.com/2079-9292/11/3/456)
- COTS radiation — [NTRS 20060015096](https://ntrs.nasa.gov/api/citations/20060015096/downloads/20060015096.pdf); [HP ZBook on ISS](https://h20195.www2.hp.com/v2/getpdf.aspx/c09260838.pdf)
- cFS Health & Safety — [github.com/nasa/HS](https://github.com/nasa/HS/blob/main/README.md)
- Dijkstra 1970; Rice, Trans AMS 1953

**Datasets**
- LifeSnaps — [Sci Data 2022](https://www.nature.com/articles/s41597-022-01764-x), [Zenodo](https://zenodo.org/records/6832242)
- GLOBEM — [arXiv 2211.02733](https://arxiv.org/pdf/2211.02733), [GitHub](https://github.com/UW-EXP/GLOBEM)
- NASA OSDR — [PMC11701653](https://pmc.ncbi.nlm.nih.gov/articles/PMC11701653/); HERA Standard Measures — [NLSP](https://nlsp.nasa.gov/explore/home/lsda_sm_hera)

**Competitor projects (2026)**
- [AURORA](https://github.com/fabriciojunio/aurora-astronaut-health)
- [RAKIA](https://github.com/jeswinbenedict/RAKIA)
- [AstroHealth](https://github.com/Ridwanulkarim/nasa-challenge-Create-Health-Monitoring-Software-for-Astronauts-on-Space-Missions)
- [AstroVitals](https://github.com/Qasim14Khan/AstroVitals)
- [ASTROCARE](https://github.com/sachuabhijith3485-cpu/ASTROCARE)
- [VITALS](https://github.com/noor-hossain13/vitals-astronaut-health-monitor)
- [Orbital Field Medical Journal](https://github.com/Rohini-S-286/orbital-field-medical-journal)

**Added in v4**
- Van Dongen, Maislin, Mullington & Dinges, Sleep 26(2):117–126, 2003 — [PDF](https://www.med.upenn.edu/uep/assets/user-content/documents/VanDongen2003CumulativeCost.pdf)
- Reifman et al., 2B-Alert App, J Sleep Res — [PDF](https://bhsai.org/pubs/Reifman_2019_2B_Alert_App.pdf)
- Vital-Lopez et al., 2B-Alert App 2.0, Sleep 2023 — [OUP](https://academic.oup.com/sleep/article/46/7/zsad080/7092866)
- Ramakrishnan et al., Unified Model of Performance, Sleep — doi:10.5665/sleep.6164
- Horvitz, Jacobs & Hovel, Attention-Sensitive Alerting, UAI 1999 — [arXiv 1301.6707](https://arxiv.org/pdf/1301.6707)
- Horvitz et al., Models of Attention, CACM 2003 — [PDF](http://erichorvitz.com/cacm-attention.pdf)
- Daza, counterfactual framework for N-of-1 — [ResearchGate](https://www.researchgate.net/publication/319164989_Causal_Analysis_of_Self-tracked_Time_Series_Data_Using_a_Counterfactual_Framework_for_N-of-1_Trials)
- Person as Population — [arXiv 1901.03423](https://arxiv.org/pdf/1901.03423)
- ESA Concordia / MECA — [ESA](https://www.esa.int/Science_Exploration/Human_and_Robotic_Exploration/Concordia/Experiments_2012-2013)
- mindLAMP/Beiwe relapse anomaly detection — [Transl Psychiatry 2020](https://www.nature.com/articles/s41398-020-01123-7)
- HELIOS — [GitHub](https://github.com/WillBettleheim/Helios)
- VITASTRA — [GitHub](https://github.com/nishatkh/VITASTRA)
- Full corpus: `LANDSCAPE.md`, `analysis/landscape/coded.csv`

**Added in v3 (from CRITICAL_REVIEW.md)**
- Trust in automation:
  - Lee & See, Human Factors 46(1), 2004 — [Academia](https://www.academia.edu/515392/Trust_In_Automation_Designing_for_Appropriate_Reliance)
  - Parasuraman & Riley, Human Factors 39(2), 1997
- Measurement reactivity — MoodMonitor protocol, [PMC5075187](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC5075187/)
- ISS devices:
  - [NASA Crew Support LAN](https://www.nasa.gov/news-release/nasa-extends-the-world-wide-web-out-into-space-2/)
  - [AIAA Aerospace America](https://aerospaceamerica.aiaa.org/features/protecting-iss/)
  - [NTRS 20130000791](https://ntrs.nasa.gov/api/citations/20130000791/downloads/20130000791.pdf)
  - [FedScoop 2023](https://fedscoop.com/nasa-investigating-2023-theft-of-astronaut-training-devices/)
- CHAPEA — [NASA](https://www.nasa.gov/humans-in-space/chapea/about-chapea/)
- C-SSRS:
  - [Screener](https://cssrs.columbia.edu/wp-content/uploads/C-SSRS1-14-09-BaselineScreening.pdf)
  - [Self-report vs interview, 2025](https://onlinelibrary.wiley.com/doi/10.1111/inm.70029)
- IMDRF, *Software as a Medical Device: Possible Framework for Risk Categorization*, 2014
- HL7 FHIR R4, Observation resource

**Judging criteria**
- [NASA Open blog: Space Apps judging and awards](https://open.nasa.gov/blog/spaceapps-judging-and-awards/)

> **Verification note.** Figures quoted from secondary summaries and not yet checked against the primary paper: the OASIS hallucination effect, pooled digital CBT-I effect sizes, the Smit et al. sensitivity/specificity values, and AURORA/RAKIA internals beyond their READMEs. Check these before final submission.
