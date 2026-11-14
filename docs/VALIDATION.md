# Validation on real people: LifeSnaps

**Status:** run on 14 Nov 2026 · reproducible with the commands at the end.

ANCHOR's detectors were calibrated and tested on synthetic missions (ADR-0005, ADR-0006). This report asks what happens when the unmodified engine meets **real human data**. It has the gaps, noise and day-to-day structure that a simulator only approximates.

## Data

- **LifeSnaps.** 71 consenting adults wore a Fitbit Sense and answered ecological momentary assessments (EMA) for about 4 months, in daily life.
  - Source: Yfantidou et al., *Scientific Data* 2022; Zenodo record 7229547; licence CC BY 4.0.
  - File used: `daily_fitbit_sema_df_unprocessed.csv` (7,410 person-days).
  - The data is not in this repository. Only the scripts and this summary are.
- **This is not astronaut data**, and it is not a spaceflight analogue. It tests whether the *statistics* hold on real human time series. It says nothing about the effect of spaceflight.

## Protocol (analysis/lifesnaps/prepare.py, core/examples/replay_people.rs)

- **Indicators**, each with a deliberately weak, generic adult prior (ν = 5):

  | Indicator | Source column | Kind | Harmful direction |
  |---|---|---|---|
  | `sleep_min` | Fitbit `minutesAsleep` | sample | lower |
  | `rhr_d1` | Fitbit `resting_hr`, one decimal | sample | higher |
  | `mood` | EMA valence proxy: 500 + 200 × (HAPPY + RESTED/RELAXED − SAD − TENSE/ANXIOUS), on the 0–1000 slider | check-in | lower |

- **Pre-flight.** Each person's first 21 calendar days are their "pre-flight" anchor. Every later day is a mission day, closed whether or not data arrived.
- **Engine.** The production engine, through the same `Session::submit_wire` path the browser uses. No parameter was tuned on this data.
- **Comparator.** A naive single-day rule: a warning on any day whose value lies 2 SD or more beyond the person's own range in the harmful direction. Runs of consecutive warning days count once, to be fair to the comparator.
- **Inclusion.** An indicator counts for a person once it has at least 28 scored days.

## Results

| | Mood (31 people, 1,418 days) | Sleep (48 people, 2,506 days) | Resting HR (52 people, 3,049 days) |
|---|---|---|---|
| Days inside the 80 % personal range: pooled | **0.793** | **0.784** | 0.579 |
| Per-person median (IQR) | 0.794 (0.739–0.875) | 0.782 (0.733–0.844) | 0.613 (0.508–0.717) |
| ANCHOR sustained-change notes | 15 = one per **94.5** days | 20 = one per **125.3** days | 48 = one per **63.5** days |
| Naive single-day rule | 31 runs = one per 45.7 days | 114 runs = one per 22.0 days | 159 runs = one per 19.2 days |

- **Robustness.** 13,376 real records produced 0 rejections and 0 companion errors, including every real gap.
- **Speed.** 6.7 ms per record on a desktop CPU (native build), including the weekly Monte-Carlo recalibration. This is far inside the N2 budget of 100 ms per check-in.

## What this shows

1. **The personal range is calibrated on real people where its assumption holds.** For mood and sleep, the 80 % band contained 79 % and 78 % of real days, pooled and per-person median alike. The band shown on Course is therefore honest about uncertainty: "about 8 in 10 of your days fall inside it" is true for these signals. That sentence is the interface's claim (ADR-0009 §6, Principle 4).
2. **Calm by construction holds on real data.** ANCHOR raised a sustained-change note once per 95–125 days per indicator for mood and sleep. The naive single-day rule warned 2 to 6 times as often.
   - This does not show that ANCHOR's notes are *correct*. LifeSnaps has no ground truth for "a real change happened". It shows the alert *load* a person would carry.
   - The design target is at most one unwanted note per 30 days across the whole system (N4). Real life contains real changes, so free-living rates are an upper bound on false notes, not an estimate of them.
3. **Finding: the band is too narrow for smoothed device signals.** Resting HR kept only 58 % of days in its 80 % band. The cause is in the data, not in the prior:
   - Fitbit's resting HR is a multi-day moving estimate, and its lag-1 autocorrelation has a median of **0.87** across 55 people. Sleep's is **−0.03**.
   - The next day deviates from the 14-day mean by 1.33 times the spread inside that window.
   - The band assumes days are close to independent, so it understates the uncertainty of a smoothed signal.
   - **Detection is not affected in the same way.** The CUSUM threshold is calibrated on an AR(1) model or a stationary bootstrap that includes this autocorrelation (ADR-0005). That is why resting-HR notes stay at one per 64 days, where the naive rule gives one per 19 days.
   - **Consequence (now a rule).** An indicator whose estimated day-to-day autocorrelation exceeds 0.5 must not show the independent-days band. It must be displayed with a widened band or as a trend only, until an autocorrelation-aware predictive band is implemented. ANCHOR's configured indicators (mood, energy, calm, sleep, SpO2 spot checks) are all low-autocorrelation measures. Resting HR is not configured in the demo for this reason.
   - **Two fixes were tried and rejected** (ADR-0010):
     - A conformal band calibrated on the person's own recent days: resting HR stays at 0.584, and mood and sleep lose about 2 points.
     - Removing the 14-day guard band: resting HR rises to 0.650, but sustained-change notes for mood fall by 40 %, because slow change is absorbed.
     - The band is therefore defined as a settled *reference*, not a forecast.
4. **The "fewer check-ins" note is an artefact here.** One note per 40 days, because LifeSnaps prompted EMA on a schedule rather than daily. In the study design a missing mood entry is often not a missed check-in, so this rate says nothing about ANCHOR's missingness detector.

## Injected change on real data: does ANCHOR notice?

LifeSnaps has no labelled changes, so a known change was injected into the real series. From mission day 40 on, each person's real sleep is shortened by k times their own pre-flight SD (`prepare.py ... k`). Everything else stays real: gaps, noise and day-to-day structure.

- **People:** 23 had enough sleep data after day 40.
- **"Noticed":** a sleep note within 28 days of the change.
- **k = 0:** the same protocol with nothing injected. It measures how often a note appears by chance in that window.
- **Intervals:** Wilson 95 % (`inject_report.py`).

| Injected change | ANCHOR noticed | median delay | Naive single-day rule noticed | median delay |
|---|---|---|---|---|
| none (chance) | 9 % [2–27] | - | 39 % [22–59] | - |
| 0.5 SD shorter sleep | 39 % [22–59] | 8 d | 48 % [29–67] | 8 d |
| 1.0 SD | 52 % [33–71] | 6 d | 65 % [45–81] | 6 d |
| 1.5 SD | 61 % [41–78] | 5 d | 65 % [45–81] | 4 d |
| Notes before the change | one per **224** days | | one per **45** days | |

- **Specificity.** Most of the naive rule's apparent sensitivity is chance: it "notices" 39 % of people when nothing changed. Above chance, ANCHOR gains 30, 43 and 52 points at 0.5, 1.0 and 1.5 SD. The naive rule gains 9, 26 and 26 points, while raising five times as many notes beforehand. In practice, a note from ANCHOR is far more likely to mean something.
- **Sensitivity is limited by data gaps.** Fitbit sleep was present on only about half of all days in this study, and a note needs evidence from days that exist. The median delays (5–8 days) are among the people who were noticed within 28 days, so they are not an expected delay. ADR-0005 gives the expected delay for a 1 SD change on complete data with realistic autocorrelation as 11–18 days. With n = 23 the intervals are wide, and the differences between the two rules overlap. This is a direction, not proof.

## Limits

- Free-living adults on Earth, not astronauts. No ground-truth change labels.
- The mood measure is a proxy built from categorical EMA flags, not ANCHOR's own continuous valence slider.
- The first 21 days stand in for a pre-flight period that would normally be supervised and longer.
- No safety-line crossings occurred. The sleep line was set at 180 min for this run, and the safety kernel is verified separately against oracle vectors (ADR-0003).

## Reproduce

```
# 1. Download rais_anonymized.zip from https://zenodo.org/records/7229547 (615 MB) and extract
#    rais_anonymized/csv_rais_anonymized/daily_fitbit_sema_df_unprocessed.csv
python -I analysis/lifesnaps/prepare.py daily_fitbit_sema_df_unprocessed.csv people.txt
(cd core && cargo run --release --example replay_people -- ../people.txt > ../results.jsonl)
python -I analysis/lifesnaps/report.py results.jsonl
# injected change (k = 0, -0.5, -1.0, -1.5)
python -I analysis/lifesnaps/prepare.py daily_fitbit_sema_df_unprocessed.csv people_k.txt -1.0
(cd core && cargo run --release --example replay_people -- ../people_k.txt > ../res_k.jsonl)
python -I analysis/lifesnaps/inject_report.py res_k.jsonl "shift -1.0 SD"
```

## Reference

Yfantidou, S., Karagianni, C., Efstathiou, S. et al. LifeSnaps, a 4-month multi-modal dataset capturing unobtrusive snapshots of our lives in the wild. *Sci Data* **9**, 663 (2022). https://doi.org/10.1038/s41597-022-01764-x
