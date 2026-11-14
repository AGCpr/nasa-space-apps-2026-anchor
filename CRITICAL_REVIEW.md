# Critical Review of DESIGN_PAPER v2 (ANCHOR)

*Reviewer stance: an independent analyst trying to break the design. Every finding lists severity, evidence, the fix, and where the fix lives in DESIGN_PAPER v3. Time constraints are deliberately ignored.*
*Numerical claims are checked in `analysis/review_checks.py`.*

**Severity scale**
- **Critical:** the design contradicts its own goals, or a credible reviewer would reject it.
- **Major:** a weakness that materially lowers validity, safety or adoption.
- **Minor:** polish, or a risk with a cheap fix.

---

## Summary verdict

v2 is a strong design with a clear thesis. A NASA reviewer, however, would find four critical problems:
1. It overclaims novelty from a small sample of competitors.
2. Its own self-harm safety line recreates the underreporting incentive it was built to remove.
3. It assumes a personal device that the ISS does not guarantee.
4. It ignores automation complacency: "no alert" will be read as "I am fine".

Beyond these, its statistics rest on assumptions (independence, unbounded normal data, one observation per day) that cost detection speed. It also covers only the in-flight phase of a mission that begins months before launch and ends months after landing. All of these are fixable, and several fixes become new contributions.

---

## Critical findings

### C1. Novelty is claimed from ~8 of 2,407 entries
- **Problem.** v2 says its contributions are "absent from the 2026 field" but examined about 8 public repositories. Most teams have no public repository yet.
- **Risk.** A judge who has seen a similar submission discounts every other claim.
- **Fix.**
  - Reword the claims to "absent from all examined entries (N, method, date)".
  - Add a reproducible scan protocol: GitHub search queries, dates, inclusion criteria, and a table of every entry examined.
  - Ground novelty primarily in the **literature and flight practice**, which is stable, rather than in a moving competitor field.
  → v3 §2.

### C2. The self-harm safety line contradicts the disclosure contract
- **Problem.** The contract exists because astronauts withhold information that may affect flight status (IOM; Mil Med 2023: 42.5% of pilots withheld). Automatically sharing a self-harm disclosure creates exactly that incentive for the most dangerous item. A rational astronaut answers "no".
- **Evidence.** Digital screening platforms do flag any positive C-SSRS answer. Self-report increased disclosure compared with nurse interviews in one hospital study (OR 6.5), which suggests privacy of mode matters. No study we found tests disclosure deterrence directly, so this is an inference, labelled as such.
- **Fix: graduated protocol.**
  - Any positive item on a C-SSRS-style screener → immediate private safety plan, plus a one-tap request for a Private Psychological Conference (PPC) initiated by the astronaut. **No automatic share.**
  - Automatic notification only for the imminent-risk items (intent or plan), and only if the astronaut agreed to this pre-flight.
  - The astronaut also names a trusted crewmember as an alternative human route.
  - This tension is irreducible and is stated as limit L6b, not hidden.
  → v3 §4.9.

### C3. Device and threat model are undefined
- **Problem.** The ISS has 100+ laptops (most of them shared ThinkPads) and a few iPads for procedures. Personal devices are tightly segmented and scheduled. The design assumes a private personal device and never defines who the adversary is.
- **Fix.**
  - Explicit threat model covering five actors: curious crewmate, ground operator, maintenance/IT, post-mission data access, and device loss or theft (training iPads were stolen in 2023).
  - Per-astronaut encrypted vault, usable on any crew device.
  - Quick lock, plus a privacy screen: no content in notifications or on the lock screen.
  - Hardware-backed key where available, with PIN-only derivation as fallback (stronger than PBKDF2-PIN alone).
  - Explicit data-retention and post-mission deletion rules.
  → v3 §5.7.

### C4. Automation complacency: silence will be read as health
- **Problem.** Trust should match capability (Lee & See 2004); over-trust leads to misuse (Parasuraman & Riley 1997). A detector with 6–8 day delays and no coverage of unmeasured states will be read as "no alert ⇒ fine".
- **Fix.**
  - Never display "all good". Show a **coverage panel** instead: what is measured, the confidence, and what the system cannot see.
  - Weekly self-review prompt independent of alerts.
  - Each insight carries a counterfactual ("this note disappears if sleep returns to your normal for 3 nights").
  - The trust-calibration message is part of day-0 onboarding.
  → v3 §4.13.

---

## Major findings

### M1. Statistical assumptions cost detection speed
- **Problem.** Sliders are bounded (0–100), with ceiling and floor effects. Daily data are autocorrelated. Indicators are correlated. A union-bound threshold over correlated indicators is conservative.
- **Evidence (computed).** With 8 indicators at ρ = 0.5, the union-bound threshold yields a system ARL₀ of 47 days instead of the 30-day target. Calibrating the threshold to the target reduces 1σ detection delay from 7.7 to 6.5 days.
- **Fix.**
  - Logit transform for bounded scales.
  - AR(1) pre-whitening, with CUSUM run on the residuals.
  - The system threshold is calibrated per person by block bootstrap of that person's own residuals (correlation and autocorrelation included), not by a union bound.
  - Optional joint MEWMA for correlated indicator groups.
  → v3 §4.3.

### M2. One observation per day is information-starved
- **Problem.** Detection delay is driven by information per day. Asking more often raises burden (P7).
- **Evidence (computed).** An adaptive rule asks for a second short check-in on days when accumulated evidence exceeds half the threshold. This cuts 1σ detection delay from 7.8 to 6.0 days, at the cost of extra check-ins on about 10% of stable days.
- **Fix.**
  - **Adaptive sampling intensity**: more information when evidence accumulates, none otherwise. This is a new contribution.
  - Also **import existing flight measures** (RST, Standard Measures, wearable sleep) instead of re-asking.
  → v3 §4.1, §4.3.

### M3. The pre-flight anchor may itself be abnormal
- **Problem.** Astronauts are already sleep-deficient during the 3 months of pre-flight training (Barger 2014: < 6.5 h). A few pre-flight sessions may anchor to a stressed state.
- **Fix.**
  - Build the anchor from **months of training-period use** across varied contexts (the lifecycle, M11).
  - Store it as a distribution, not a point.
  - Flag the anchor when it deviates from the population prior, and let the astronaut and psychologist review it.
  → v3 §4.2.

### M4. Measurement reactivity
- **Problem.** Daily mood monitoring may itself change mood. The evidence is thin; the MoodMonitor trial was designed precisely because reactivity "has received little attention".
- **Fix.**
  - Monitor for reactivity: changes in compliance, variance and level after onboarding.
  - Offer a **quiet mode** (passive and imported data only).
  - Treat reactivity as an analog-study hypothesis.
  → v3 §4.1, §12.

### M5. Display rules and social desirability
- **Problem.** The Mars-500 crew followed unwritten positive-display rules. Self-report level is biased upward.
- **Fix.**
  - Detection uses **within-person change**, which is insensitive to a constant upward bias.
  - Triangulation with harder-to-fake measures.
  - Wording never connects inputs to evaluation or selection.
  - The contract removes the motive.
  - The residual bias that varies over time is stated as a limitation.
  → v3 §4.12, §13.

### M6. External validity of validation data
- **Problem.** LifeSnaps and GLOBEM participants are students and community adults, not astronauts. GLOBEM labels are weekly PHQ-4 scores.
- **Fix.**
  - Staged validation roadmap: open datasets → HERA / CHAPEA-type analogs (CHAPEA used a 22-min one-way delay) → Antarctic station → ISS technology demonstration.
  - Pre-registered hypotheses, with null results reported.
  - Each stage has explicit go/no-go criteria.
  → v3 §14.

### M7. Learning by randomisation in an operational setting
- **Problem.** Thompson sampling randomises which suggestions are shown. Rewards are sparse (≤ 1 per day), self-reported, and non-stationary across mission phases.
- **Fix.**
  - Informed consent for the learning feature.
  - Exploration limited to equally evidence-graded options.
  - Discounted posteriors for non-stationarity.
  - **Off-policy evaluation** on logged data before any policy change.
  - Fallback to a fixed rule policy.
  - The astronaut can switch learning off.
  → v3 §4.8.

### M8. Circadian estimates without light data
- **Problem.** Phase models are driven by light. Without a light sensor the estimate carries systematic error, which the Entrain real-world study reported.
- **Fix.**
  - Use habitat lighting-mode logs (the ISS lighting schedule is known) and the optional light-sensing actigraph used in flight research.
  - Report the phase **with uncertainty** (level-set Kalman filter).
  - Advice is restricted to low-risk timing suggestions, and advice is suppressed when uncertainty exceeds a set limit.
  → v3 §4.6.

### M9. Software TMR does not protect against software bugs
- **Problem.** Three runs of the same code on the same CPU only mask transient data faults. A logic bug is reproduced three times.
- **Fix.**
  - **Design diversity:** two independently written kernel implementations (N-version), whose outputs are compared.
  - Since the input space is small (finite safety table × discretised inputs), **exhaustive verification** of the kernel against a formal specification.
  - State plainly that TMR covers single-event upsets and diversity covers logic errors.
  → v3 §5.2.

### M10. The interpersonal hazard is under-served
- **Problem.** Group interaction is among the top categories in astronaut journals (Stuster), and isolation and confinement is largely interpersonal. v2 is purely individual.
- **Fix.**
  - Private interpersonal tools that never aggregate crew data: conflict-preparation and debrief modules modelled on the VSS/Expedition-APPP conflict content, and a "prepare for a difficult conversation" worksheet.
  - Optional bilateral sharing of a *single* agreed signal between two consenting crewmates, which does not violate the no-aggregation rule.
  → v3 §4.14.

### M11. Only the in-flight phase is designed
- **Problem.** NASA-STD-3001 covers pre-, in- and post-mission behavioural health (V1 4011, 4013) and family support (V1 6004).
- **Fix: four lifecycle modes.**
  - **Training:** months of use to build the anchor and the habit.
  - **Flight.**
  - **Return:** readaptation tracking against the anchor, with a gradual hand-off to ground support.
  - **Family bridge:** astronaut-controlled sharing of non-clinical status ("good week") with family.
  → v3 §4.15.

### M12. The CMO may be the patient or a party to the conflict
- **Problem.** Escalation level 2 assumes a neutral Crew Medical Officer.
- **Fix.** The astronaut pre-selects primary and alternate crew contacts. The system never routes a concern to a person the astronaut has excluded.
  → v3 §4.9.

### M13. Duplicating existing flight measures adds burden
- **Fix.** ANCHOR is a **personal layer over existing instruments**. It imports RST/PVT-B results, Cognition scores and wearable sleep, and asks only for what is missing.
  → v3 §4.1.

### M14. Clinical hand-off is unspecified
- **Fix.**
  - When the astronaut shares, export standard-format observations (HL7 FHIR Observation resources) with provenance (measured / self-reported / derived) and the reasoning trail.
  - A separate clinician view shows effect sizes and confidence, not the astronaut-facing wording.
  → v3 §4.16.

### M15. Regulatory framing missing
- **Fix.** Frame the product as software that *informs* management of a *non-serious* situation. This is the lowest-risk category in the IMDRF SaMD framework. Any path to "drive" or "diagnose" is explicitly excluded.
  → v3 §10.

### M16. International crews
- **Fix.**
  - Templates and audits per language (English/Russian at minimum).
  - Scale anchors validated per language.
  - Banned-term lists reviewed by native-speaking clinicians.
  → v3 §4.10.

### M17. Microgravity ergonomics
- **Fix.**
  - Large touch targets, one-hand operation and a tethered-tablet layout.
  - No precision dragging: tap-to-set sliders with confirmation.
  - Usability tests in parabolic flight or analog.
  → v3 §4.17.

### M18. Statistical outputs are not intuitive
- **Fix.** Never show "σ". Show personal percentiles ("lower than 9 of your last 10 weeks"), small multiples, and plain-language confidence.
  → v3 §4.10.

### M19. Data governance after the mission
- **Fix.** Ownership by the astronaut, a retention period, deletion on request, and an **opt-in research donation** tier with separate consent.
  → v3 §4.9.

---

## Minor findings

| ID | Finding | Fix |
|---|---|---|
| m1 | "Evidence grade" is undefined | GRADE-style A–D scale defined in §4.8 |
| m2 | The lexical audit cannot judge tone | Add a human review panel and readability limits |
| m3 | A 21-day autocorrelation estimate is very noisy (SE ≈ 0.22, 95% CI ± 0.43) | Reinforces the exploratory-only status of §4.5; shown numerically |
| m4 | BOCPD hazard rate unspecified | Set from expected change frequency (one per ~60 days), with a sensitivity analysis |
| m5 | Accessibility not stated | WCAG 2.2 AA |
| m6 | Working name may collide | Trademark check before release |

---

## New contributions arising from this review
1. **Adaptive sampling intensity:** information on demand (M2).
2. **Per-person bootstrap-calibrated thresholds:** an exact false-alert budget under real correlation (M1).
3. **Graduated disclosure protocol for high-stakes items** (C2).
4. **Coverage panel and counterfactual explanations** against complacency (C4).
5. **N-version, exhaustively verified kernel** (M9).
6. **Mission lifecycle modes** with a training-built anchor (M3, M11).
7. **Personal N-of-1 self-experiments**: the astronaut can test, for example, light-timing A vs B over two weeks with a proper Bayesian analysis. This strengthens autonomy (SDT) and draws on Bayesian N-of-1 methods.
8. **Clinician hand-off in a standard format** (M14).

---

## Sources added in this review
- Lee & See, *Trust in automation*, Human Factors 46(1), 2004 — [Academia](https://www.academia.edu/515392/Trust_In_Automation_Designing_for_Appropriate_Reliance)
- MoodMonitor reactivity RCT protocol — [PMC5075187](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC5075187/)
- ISS laptops and personal access — [NASA](https://www.nasa.gov/news-release/nasa-extends-the-world-wide-web-out-into-space-2/), [AIAA Aerospace America](https://aerospaceamerica.aiaa.org/features/protecting-iss/), [NTRS 20130000791](https://ntrs.nasa.gov/api/citations/20130000791/downloads/20130000791.pdf), [FedScoop 2023](https://fedscoop.com/nasa-investigating-2023-theft-of-astronaut-training-devices/)
- CHAPEA — [NASA](https://www.nasa.gov/humans-in-space/chapea/about-chapea/), [KBR](https://www.kbr.com/en/insights-news/stories/analyzing-psychological-impact-first-mars-habitat-simulation)
- C-SSRS — [Screener](https://cssrs.columbia.edu/wp-content/uploads/C-SSRS1-14-09-BaselineScreening.pdf), [self-report vs interview 2025](https://onlinelibrary.wiley.com/doi/10.1111/inm.70029), [ED validation](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC9811343/)
