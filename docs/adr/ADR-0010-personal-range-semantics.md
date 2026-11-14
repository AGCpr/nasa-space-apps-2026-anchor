# ADR-0010: The personal range is a reference, not a forecast

**Status:** accepted · 14 Nov 2026 · follows the real-data finding in docs/VALIDATION.md

## Context

On LifeSnaps (71 adults, real wearable and mood data), the 80 % personal range held:
- **79 %** of mood days;
- **78 %** of sleep days;
- only **58 %** of Fitbit resting-heart-rate days.

Resting HR is smoothed by the device and drifts slowly. Its lag-1 autocorrelation has a median of 0.87; sleep's is −0.03.

The interface tells the astronaut that "about 8 in 10 of your days fall inside" the band (ADR-0009 §6). For a signal like resting HR that sentence is false. Two fixes were tried on the same 71 people before deciding.

## Experiments (same data, same engine, one change each)

| Variant | Mood | Sleep | Resting HR | Mood notes (one per … days) |
|---|---|---|---|---|
| Current: model band, 14-day guard band | 0.793 | 0.782 | 0.579 | 94.5 |
| **A. Conformal band:** edges at the 10th and 90th percentiles of the person's own last 56 learnable days | 0.776 | 0.763 | 0.584 | 94.5 |
| **B. No guard band:** new days enter the baseline at once (diagnostic only) | 0.791 | 0.786 | 0.650 | 157.6 |

The table reports pooled coverage. B's per-person median for resting HR was 0.73, against 0.61 for the current design.

- **A fails.** Calibrating on the person's recent deviations assumes those deviations are exchangeable. A drifting signal breaks that assumption, so the band did not improve where it mattered and lost about 2 points elsewhere.
  - A synthetic AR(1) test with φ = 0.9 did not reproduce the problem: the model band held 0.80 there.
  - What fails on real data is therefore long-memory drift, not short-term autocorrelation.
- **B explains part of the gap and costs the product's purpose.**
  - Without the guard band the baseline follows the person closely, so coverage rises.
  - But slow change is then learned as normal before anyone notices: sustained-change notes for mood fell by 40 %.
  - The guard band exists so a slow decline is *noticed*, not absorbed (ADR-0006 §8).
- **What remains is window variance.** In a drifting, autocorrelated signal, the spread within 14 days underestimates the spread across weeks.

## Decision

1. **Keep the design.** The band shows the person's *settled reference*: their level as learned at least 14 days ago, with its uncertainty. It is not a forecast of tonight.
   - A night outside the band means "away from your settled normal".
   - That is exactly what the detector then weighs, with a threshold calibrated for autocorrelation (ADR-0005).
2. **Configuration rule.** An indicator's day-to-day autocorrelation is estimated from pre-flight data. If it exceeds 0.5, the indicator is shown as a trend against its pre-flight level, without the "8 in 10 days" band and without that sentence.
   - All of ANCHOR's configured indicators are below this limit: mood, energy and calm sliders, sleep duration, SpO2 spot checks.
   - Device-smoothed signals such as resting HR or HRV are above it. They stay out of the band display until an autocorrelation-aware predictive band exists, for example a state-space (local-level) model.
3. **Wording.** The sentence "about 8 in 10 of your days fall inside it" is kept, because it was verified on real data for every configured indicator (79 % mood, 78 % sleep).

## Consequences

- The interface claim is true for what ANCHOR shows, and it is backed by real data.
- The limitation is stated, not hidden, and the experiments that rule out the obvious fixes are recorded with numbers.
- **Future work:** a local-level (random-walk-plus-noise) model would give a calibrated band for smoothed physiological signals without giving up the guard band. It needs its own validation on long series.

## Reproduce

`analysis/lifesnaps/` with the engine constants changed one at a time:
- Variant A: the patch is described in this ADR; it was reverted, not committed.
- Variant B: set `GUARD_DAYS = 0`.

The commands are the ones in docs/VALIDATION.md.
