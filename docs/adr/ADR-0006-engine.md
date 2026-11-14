# ADR-0006: Engine — from log to outputs

**Status:** accepted · 10 Nov 2026 · §8 history rule and §9 episode semantics amended by ADR-0007 §5 · `DayStat` extended with the personal range and a `kernel_disagree` record added by ADR-0009 §6–7

## Context

ADR-0002 to ADR-0005 delivered isolated components. The engine wires them together. It must satisfy:
- **I5:** derived state is a pure function of the log; replaying the log reproduces every output exactly.
- **I1:** the safety kernel always runs, and runs first, whatever happens in the companion layer.
- **N3:** state survives crash and corruption, because it can be rebuilt from the (repaired) log.

## Decisions

### 1. The engine is a deterministic fold over log entries
- **Shape:** `Engine::apply(&mut self, &Entry) -> Vec<Output>`; replay is `log.entries().fold(...)`.
- **Alternatives considered**
  - **A stateful service with snapshots.** Faster restart, but snapshots are a second source of truth that can disagree with the log. At ~10 records/day (≈ 11,000 over 3 years) a full replay is cheap, so a snapshot buys nothing worth that risk.
  - **Pure `replay(log) -> outputs` only.** Simplest, but the UI needs outputs as each entry arrives. The fold gives both modes from one code path, and a test asserts that incremental and full replay agree.

### 2. All configuration lives in the log
- Indicator definitions, population priors, the safety table, budget and directions are **log records**.
- **Alternative rejected:** an external config file. Replay would then depend on a second, mutable input, and I5 could not be checked.
- **Consequence:** every configuration change is itself auditable (who changed what, when).
- **Floats are forbidden in the log (ADR-0002).** Population parameters are therefore stored as integers in millionths: `mu_e6`, `tau2_e6`, `sigma2_e6`, `nu_p_e6`.

### 3. Time enters only through explicit `day_close` records
- **Alternative rejected:** reading a clock to decide when a day ends. That would make output depend on when replay happens.
- **Decision:** a `day_close` record (written by the UI at the end of a crew day) triggers all day-level processing:
  - standardisation and day-value combination;
  - CUSUM steps;
  - the missingness step;
  - the weekly drift test and recalibration.
- Mission days are integers in records. Mission elapsed time in milliseconds (`met_ms`) is used only for ordering.

### 4. Kernel first, companion second, companion failures contained
- For every value in a `checkin` or `sample` record:
  1. The TMR kernel is evaluated **before** anything else.
  2. Its outputs are emitted.
  3. Only then does companion processing run, and it returns `Result`. An error becomes `Output::CompanionError` and **never** suppresses or reorders kernel outputs. This is I1 by construction, and a test injects companion failures.
- The engine keeps three independently parsed in-memory copies of the safety table, compared against the logged digest, so the TMR protects against memory corruption as well as storage corruption.
- **N-version comparison with kernel B** happens at the UI boundary (TypeScript), where kernel B runs. The engine outputs kernel A's result together with the raw input, so the UI can compare the two.

### 5. Deterministic randomness for recalibration
- Calibration needs a seed.
- **Decision:** seed = the first 16 hex digits of the hash of the `day_close` entry that triggers the recalibration.
- **Why:** reproducible from the log alone, different every week, and not chosen by anyone. That rules out a "lucky seed" being picked to get a convenient threshold.

### 6. Day processing order (per `day_close`)
1. A day tagged with a context tag is excluded from baseline, CUSUM and drift. It counts for missingness only if it is not tagged `workload`.
2. For each indicator with samples that day:
   - $z$ of each sample against the rolling posterior of **previous** days;
   - the day value $z_\text{day}$ (ADR-0005 D5);
   - the CUSUM step;
   - an `OfferExtraSample` output if the rule holds and only one sample exists.
3. The daily mean (transformed scale) is appended to the rolling window and the anchor-standardised series.
4. Missingness: a day with no check-in, and not tagged `workload`, steps the Bernoulli CUSUM as a miss; otherwise as done. The miss rate is updated.
5. Days on which every indicator has a value are appended to the calibration history.
6. Every 7th closed day: drift test and anchor-run check per indicator, then recalibration of the bank and missingness thresholds.
7. Before the first calibration there is no threshold, so no alarm can fire. The engine reports `Calibrating` instead of guessing.

### 7. Validation at the trust boundary
- `Engine::validate(&Record)` checks the record **before** it is appended:
  - known kind;
  - known indicator keys;
  - slider values in range;
  - day not decreasing;
  - configuration only before the first `day_close`.
- An invalid record that reaches replay anyway (e.g. from an old log) produces `Output::Rejected` deterministically. It is never silently skipped.

### 8. Guard band and learning gate for the rolling baseline (added after end-to-end testing)

**Finding 1: the rolling baseline erodes the CUSUM.** The CUSUM standardises against the rolling posterior. After a sustained shift, the rolling posterior learns the new level within ~2 weeks, and the evidence stops accumulating. In the first end-to-end mission run, a 1.5-SD mood drop starting on day 100 was detected only on **day 129**. This is the shifting-baseline trap again, this time inside the detector.

**Fix: guard band.** A day's mean enters the rolling window only after `GUARD_DAYS = 14` further days. This matches the typical detection delay (ADR-0005), and HELIOS converged on the same 14-day lag independently.

**Finding 2: the end-to-end budget was violated, and the cause was a suspicion gate.**
- The first gate design excluded from learning (and from the calibration history) every day with $S > h/2$.
- On healthy simulated missions (24 seeds × 300 days) this gave **one false alert per 15.3 days**, against a budget of 30.
- Diagnostics (daily $z$ moments per period) showed z drifting in the harmful direction after day 90. The gate is one-sided: it rejects only harmful-direction days, so the baseline learns only from "good" days, drifts away, and pushes $z$ further in the harmful direction. The same selection truncated the calibration history and pulled $h$ down.

**Decision**
- **Learning gate:** exclude a day only while that indicator is inside a **latched alarm episode**, so that a real abnormal state is not learned and the alarm does not clear prematurely.
- **Calibration history:** keeps **all** untagged complete days. Contamination by a real episode can only raise $h$, which is the budget-safe direction.

**Evidence (healthy missions, 24 seeds × 300 days = 7,200 person-days)**

| Variant | Days per false alert | Budget |
|---|---|---|
| Suspicion gate | 15.3 | 30 (**violated**) |
| No gate | 36.4 (12 seeds) | 30 |
| **Episode gate (chosen)** | **37.3** | 30 |

On the mission scenario, the same 1.5-SD drop is now detected on **day 104**, i.e. 4 days after onset.

The end-to-end budget test is `cargo test -- --ignored budget` (≈ 50 s). It is the authoritative check of N4, because it includes every real source of error: estimated baselines, guard band, recalibration and selection effects.

### 9. Alarm latching (alert burden)
- **Finding:** before latching, an 11-day gap in check-ins produced 7 missingness alarms, and sustained shifts re-fired after every reset.
- **Decision, following the aviation principle that an alert persists while its condition exists rather than re-annunciating:**
  - an indicator alarm fires once per episode and clears after `CLEAR_DAYS = 3` consecutive days at $S = 0$ (`AlarmCleared`);
  - the missingness alarm clears on the next day with a check-in (`MissingnessCleared`);
  - the in-control miss rate is learned only outside missingness episodes.

### 10. DayStat output
- `DayStat { z, statistic }` is emitted per indicator per processed day.
- **Why:** the UI needs it for trends and explanations. It also made the diagnosis in §8 possible, and that diagnosis would otherwise have needed private instrumentation.

## Consequences
- The UI layer only appends records and renders outputs; it holds no state that replay cannot rebuild.
- Re-anchoring by consent (DESIGN_PAPER §4.4) and the high-stakes protocol records are **not** part of this ADR. They come next, as their own record kinds.
