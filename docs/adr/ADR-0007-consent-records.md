# ADR-0007: Consent records — re-anchoring, in-flight consent, high-stakes screener

**Status:** accepted · 10 Nov 2026

## Context

- ADR-0006 left two mechanisms of the design paper outside the engine:
  - re-anchoring by consent (DESIGN_PAPER §4.4);
  - the graduated high-stakes protocol (§4.9, invariant I8).
- Reading the engine against the paper exposed two further gaps:
  - **G1: the safety table could be replaced at any time** by a single `safety_table` record, including in flight. A physiological line agreed with the flight surgeon could silently disappear. This also contradicts "pre-signed before flight".
  - **G2: high-stakes screener items had no correct path.** Every value had to belong to an indicator, so a C-SSRS answer would have entered the CUSUM, the calibration history and the rolling baseline. That breaks the separation between the companion's statistics and the kernel's protocol, and mixes the most sensitive answers into ordinary trend data.

All new records obey ADR-0006: they are log records, validated at the trust boundary, and replay reproduces their effect exactly (I5).

## Decisions

### 1. The safety table is fixed at configuration; in flight only consent flags change

- **Rule.** A `safety_table` record is valid **only before the first `day_close`**. The first `day_close` is rejected unless a table exists. An empty table (`t1|`) is allowed, but it must be an explicit choice. The engine never runs a mission "without a decision".
- **Why.** The physiological lines are an agreement between two parties (astronaut and flight surgeon). The engine cannot authenticate who wrote a record, so it cannot let one party change a two-party agreement in flight. Changing those lines is a ground procedure: a new mission configuration, which is outside this ADR.
- **Alternatives rejected**
  - **Free replacement (status quo).** This is G1.
  - **"Only stricter" in-flight replacement (lines may be added or tightened, never removed or loosened).** Plausible, but "stricter" has no total order across lines: is moving a limit up while adding a new line stricter? It needs a table-diff semantics that must itself be verified N-version. That is a large new attack surface for a rare need.
  - **Signatures by both parties.** Correct in principle, but the system has no key infrastructure, and a single device holds both "parties".

### 2. In-flight consent change: `consent{day, line, share}`

- **Scope.** Only the auto-share flag of a **high-stakes** line can change. Physiological lines always share, and the record is rejected for them. A change that does not change anything is rejected too, so the log contains only real decisions.
- **Effect.** The engine:
  1. takes the TMR-verified copy;
  2. re-serialises the table canonically with the one flag changed (`SafetyTable::with_consent`, `canonical`);
  3. replaces all three copies and the digest.

  If no two copies agree, the change is refused ("safety table copies disagree"). A consent change is never applied on top of an unverified table.
- **Output.** `ConsentChanged{day, line, share, table, digest}`. The new canonical table text is included so that kernel B at the UI boundary evaluates exactly the same table. It never re-derives the table.
- **Withdrawal takes effect immediately. This was debated.**
  - **Thesis (delay withdrawal, e.g. 72 h).** An astronaut in crisis may withdraw auto-share exactly when it matters.
  - **Antithesis.**
    - Consent that cannot be withdrawn at will is not valid consent (the research-ethics norm of withdrawal at any time).
    - A delay gains no protection, because the screener is voluntary (§3): anyone who wants to avoid sharing can simply not answer.
    - What a delay *does* change is the astronaut's belief that the system can act against their current wish. That is precisely the underreporting incentive the disclosure contract exists to remove (§4.9).
  - **Decision.** Immediate, logged and visible. The residual risk is the one already stated as limit L6b: human routes.
- **Granting** also takes effect immediately.

### 3. High-stakes screener: `screen{day, item=0|1, ...}`, a kernel-only path

- **Validation**
  - Each item key must be referenced by the safety table **only through high-stakes lines**.
  - An item key must not be a companion indicator.
  - Answers are 0 or 1, because C-SSRS screener items are yes/no.
  - The table must exist.
- **Processing**
  1. **Kernel first (I1).** Each answer is evaluated by the TMR kernel exactly like any value: `Safety` with per-line share flags, or `SafetyCheckFailed`. Sharing is decided **only** by the kernel and the per-item consent (I8).
  2. **Any "yes" yields `PrivateSupport{day, items}`.** This is the protocol's private response: the safety plan, the one-tap astronaut-initiated request for a Private Psychological Conference, and the chosen trusted crewmember. It is derived from the answers themselves, **not** from the kernel's result, so a failed kernel check never withholds the private support.
  3. **Nothing else.** A screen record leaves every other piece of engine state unchanged:
     - no samples, baseline, CUSUM or calibration history;
     - no check-in credit for the missingness detector.

     A test asserts that the entire engine state is identical before and after applying a screen record. The one exception is the outbox, where a consented share is queued (ADR-0008).
- **Why not reuse indicators.** See G2. A yes/no item has no meaningful Gaussian baseline either, and putting it into the bank would let the most sensitive answers influence ordinary alarms (I7 spirit).
- **Why a screen is not a check-in.** The screener is voluntary and infrequent. Counting it as engagement would let one weekly screen mask disengagement from the daily check-in, which is what the missingness detector measures.
- **Out of scope here:**
  - **Scheduling** (offering the screener weekly and on demand). This is UI behaviour and needs no state.
  - **Transmission.** The sharing log (I6) and contact routing (I10) belong to the transmission layer and are the subject of a later ADR. The engine's `share` flags are the input to that layer.

### 4. Re-anchoring by consent: `reanchor{day, key_txt}` and `reanchor_revert{day, key_txt}`

- **What it does.** The anchor of the indicator is recomputed with the **same estimator** as the pre-flight anchor (`baseline::anchor`, population prior plus data). Instead of the pre-flight data, it uses the last ≤ 28 untagged daily means (`DRIFT_WINDOW`).
  - Because the estimator is the same, the numerics are already covered by the oracle vectors.
  - The previous anchor is kept on a stack, so every re-anchor is reversible, back to the pre-flight anchor.
- **Alternatives considered**
  - **Shift only the anchor mean, keep the pre-flight variance.** Rejected: adaptation can change dispersion too (e.g. sleep becoming regular), and keeping a pre-flight variance would mis-scale every later drift test. The full re-estimate is shrunk towards the population prior, as the original was.
  - **Pool pre-flight and recent data.** Rejected: the premise of a re-anchor is that the level has changed. Pooling two levels inflates the variance and biases the mean.
  - **Clear the drift window on re-anchor.** Rejected, and this is the key safeguard. The drift test keeps the days *before and after* the re-anchor in one window, expressed in the new anchor's units. A re-anchor **cannot hide an ongoing slope**: a slow decline that was re-anchored keeps producing drift prompts. What a re-anchor accepts is a **level**, not a trend.
- **Conditions (validation)**
  - **Not for any indicator referenced by the safety table** (§4.4). Its trend is safety-relevant and is not the astronaut's alone to redefine. Because the table is fixed in flight (§1), a later table change cannot create a re-anchored safety indicator.
  - **An alarm episode must be at least `SUSTAINED_DAYS = 14` days old.**
    - *First draft, rejected: "never during an alarm episode."* Simulation showed that a sustained level shift keeps its alarm latched indefinitely. The ADR-0006 learning gate never learns episode days, so re-anchoring is the **only** exit, and the first draft would have closed it.
    - The 14-day age keeps the original purpose: an *acute* episode cannot be silenced with "this is my new normal". Only a state the person has lived with for two weeks can be accepted.
  - **The accepted level may be at most `REANCHOR_MAX_SD = 2` pre-flight SD from the pre-flight anchor in the harmful direction.**
    - The distance is measured against the **pre-flight** anchor, not the current one, so repeated re-anchors cannot ratchet the reference downwards.
    - Beyond 2 SD the new level is not the astronaut's alone to accept. The alarm stays latched as one calm, persistent advisory, never a repeated one, and the human routes (the Private Psychological Conference) apply.
    - 2 SD is a policy value, not a statistical one. It is twice the distance at which the anchor-run prompt (§6) invites a review.
  - **At most once per indicator per `REANCHOR_EVERY = 30` days.** This is the rate limit from §4.4.
  - **At least `WINDOW = 14` daily means available.** The same minimum the rolling baseline uses. Fewer days would define a "new normal" from noise.
- **What it changes.** The accepted level becomes the reference everywhere:
  - the anchor;
  - the rolling window, which becomes the last 14 daily means;
  - the guard-band queue is cleared, because those days are now in the window;
  - the CUSUM statistic resets to 0;
  - a latched alarm clears, with an `AlarmCleared` output.

  Changing only the anchor would not work: the rolling window still holds the old level, so z would stay shifted.
- **What it keeps.** The drift window (the slope safeguard above) and the calibration history (z units).
- **Revert.**
  - A revert restores the previous anchor **and its rolling window**.
  - Days still waiting out the guard band are marked not learnable, because "this was not my normal" means they must not be learned either.
  - A revert is allowed at any time while a previous anchor exists. It moves towards the pre-flight reference, which is the conservative direction, so it is not rate-limited and does not reset the rate limit.
- **Outputs.** `Reanchored{day, indicator, from, to}` gives the anchor means on the model scale. `ReanchorReverted{day, indicator}`. Both feed the weekly summary's visibility requirement (§4.4).
- **Evidence** (`engine::tests`, mood settles 1.5 SD lower from day 20):

  | Event | Result |
  |---|---|
  | Day 30: re-anchor during a 7-day-old episode | Rejected |
  | Day 40: re-anchor `spo2` | Rejected (safety-line indicator) |
  | Day 40: re-anchor `mood` | Accepted; the alarm clears |
  | Days 41 and 60: further re-anchors | Rejected (30-day limit) |
  | Days 41–99 | Mean mood z = **+0.06** (−1.5 without re-anchoring), variance 0.85; no anchor-run prompt |
  | Day 100: revert | An alarm fires on day 101, the run prompt on day 105, and mean z for days 101–115 is < −0.8 |
  | A 2.5 SD shift | Re-anchor rejected (distance cap) |
- **Implementation consequence.** The drift series is stored as **raw** daily means and standardised against the current anchor when tested. Before this ADR it was stored pre-standardised, which would have frozen it to the old anchor.

### 5. Episode semantics, corrected (found while testing §4; amends ADR-0006 §8–9)

The re-anchoring test ran a sustained 1.5-SD shift **without** re-anchoring first. With the ADR-0006 engine, this showed two defects.

**D1: the latched alarm flapped.**
- `Cusum::step` resets the statistic to 0 after each crossing. Inside a latched episode, a few noisy days after a reset satisfied "3 days at S = 0".
- Result: the alarm cleared on day 77 and fired again on day 83. That is the re-annunciation latching was meant to stop.
- **Fix:** on a crossing the engine **holds S at h** instead of resetting. An episode clears only when the evidence has decayed to 0 and stayed there for 3 days.

**D2: sustained episodes poisoned calibration.**
- ADR-0006 kept every day in the calibration history, arguing that contamination "can only raise h, the budget-safe direction".
- **The size was never measured.** When the stationary bootstrap took over at 90 days, ~70 shifted days moved h from **4 to 20**. The bootstrap does not centre, so a persistent mean counts as in-control.
- An h of 20 makes the bank effectively blind for the rest of the mission. It is "budget-safe" only in the sense that a disabled detector is.

**Decision for D2, by measurement** (healthy missions, 24 seeds × 300 days = 7,200 person-days, with D1 fixed):

| History rule | Days per false alert | Sustained 1.5-SD shift: max h |
|---|---|---|
| ADR-0006: keep all days | 40.2 | ≈ 20 (blind) |
| Drop all episode days | **29.1** (one-sided selection lowers h; the ADR-0006 §8 warning confirmed) | ≈ 4 |
| **Drop days of episodes ≥ 14 days old (chosen)** | **39.6** | **≈ 4** |

- **Why the chosen rule works.** False episodes in healthy data are short, so they keep their days and the history is not truncated. Real sustained shifts stop contaminating after two weeks.
- **Same constant.** The 14-day age is the same `SUSTAINED_DAYS` that makes an episode eligible for re-anchoring: one definition of "sustained" for the whole engine.
- **Before this ADR:** 37.3 days per false alert (ADR-0006). Fixing D1 alone improved this to 40.2, because it removed the re-fires after spurious clears.
- **Mission scenario:** the 1.5-SD drop starting on day 100 is detected on day 105 (104 before).

### 6. Anchor-run prompt: window mean, not a run of days (amends ADR-0005 D6)

- **The defect.** The implementation prompted only when **each** of the last 14 daily distances exceeded 1 SD. With daily noise of about 1 pre-flight SD, that almost never happens.
- **The fix.** The design paper specifies $|m_W - m_0|/s_0 > 1$, a window mean, and the implementation now follows it.

Power per weekly check, measured on AR(1) data with φ = 0.3 (`analysis/anchor_run_check.py`):

| Shift (SD) | Run of 14 days > 1 | 14-day mean > 1 |
|---|---|---|
| 0 (false prompt) | 0.0000 | 0.0025 |
| 1.0 | 0.0008 | 0.50 |
| 1.5 | **0.018** | **0.92** |
| 2.0 | 0.136 | 0.998 |

- **Latching.** Like alarms, the prompt is now latched: one prompt per run, re-armed once the window mean returns inside 1 SD. Without latching, the fixed prompt repeated every week for an unresolved shift.

## Consequences
- Every consent-bearing decision of the paper that changes engine behaviour is now a log record with a deterministic effect, and is replayable and auditable.
- **Remaining for the transmission ADR:**
  - the sharing log entry before transmission (I6);
  - contact routing and exclusions (I10);
  - per-category sharing consent (tier 2);
  - the PPC request record.
- **Kernel B** needs no serialiser: it receives the canonical table text in `ConsentChanged`.
- **Residual limit.** Re-anchoring can still accept a *level* drop as normal if the astronaut so chooses. That is the design intent (autonomy), it is made visible, and it is bounded by the rate limit and by the slope test that survives it.
