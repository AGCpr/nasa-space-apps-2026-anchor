# ADR-0008: Transmission layer — outbox, routing, exclusions, sharing log

**Status:** accepted · 11 Nov 2026

## Context

ADR-0007 decides **whether** something may be shared: per-line consent and kernel `share` flags. This ADR decides **what** leaves the device, **to whom**, and **when**. It must satisfy:

- **I2:** journal text and interaction metadata never leave the device.
- **I6:** every transmission is preceded by a visible sharing-log entry.
- **I8:** a high-stakes item is auto-shared only with per-item consent (already enforced by the kernel).
- **I10:** no message is ever routed to a contact the astronaut has excluded.
- **F5:** escalation is delay-aware. On Mars the one-way light time reaches ~22 min, and solar conjunction removes ground contact for ~2 weeks.
- **F6:** per-category sharing (tier 2 of the disclosure contract, DESIGN_PAPER §4.9).
- **N3 / I5:** nothing is lost in a crash, and everything is reproducible from the log.

## Decisions

### 1. The engine authorises; the transport only executes (outbox pattern)

- **The engine never performs I/O.** Every share is first an **outbox item**, created deterministically while the log is replayed. It produces `ShareQueued{share, class, to, payload}`, and **this output is the sharing-log entry (I6)**. The UI shows it.
- **The transport** (radio, network) may transmit only an item that is in the outbox. After transmitting, it appends `sent{share}`.
  - The engine validates that record: the item exists, is unsent, has a recipient, and that recipient is not excluded and is reachable.
  - So the order "log entry before transmission" is a property of the log itself. It can be checked on any log: a `sent` without a prior authorisation is impossible to append, and if one is found in an old log it is reported as `Rejected`.
- **Alternatives rejected**
  - **The engine calls the transport directly.** I/O inside the fold breaks I5: replaying would re-send. It also makes I6 a matter of call order in code instead of a checkable property of the log.
  - **The transport keeps its own queue.** A second source of truth. After a crash the two queues can disagree (sent twice, or never sent). Because the outbox is rebuilt from the log, a crash between transmission and `sent` can at worst cause **one** duplicate transmission. A duplicate is idempotent for the recipient, since the share id is in the payload. Losing a safety notice would not be acceptable; a duplicate is.
- **Share ids** are a counter in replay order. They are therefore deterministic, and identical on every replay.

### 2. Payloads come from a closed schema (I2 by construction)

- A payload is built **only** by the engine and **only** from the fields of a fixed per-class allowlist. Every value is an integer, except `ind`, which is a configured indicator key. A property test checks every queued payload against the allowlist.

  | Class | Payload keys | Source |
  |---|---|---|
  | `safety` | `share, day, lines, value` (physiological), `share, day, lines` (high-stakes), `share, day, fault` (kernel fault) | Kernel |
  | `summary` | `share, ind, day, days, mean_z_e3, alarm` | Tier-2 consent per indicator |
  | `ppc` | `share, day, ppc` | Astronaut request |
  | `crew` | `share, day, crew` | Astronaut request |

- **No free text exists in any payload.** The journal and interaction metadata have no path into an outbox item, because no payload constructor reads them.
- **High-stakes payloads carry only the line id.** The id names the item; the answer itself is implied. The astronaut's other answers are never included.
- **Kernel faults.** A `SafetyCheckFailed` on a check-in value queues a fault notice **without value or indicator**: the system's integrity is safety-relevant, the person's value is not shareable by default.
  - A fault while evaluating a **screen** queues nothing. Even "a screen was taken" is interaction metadata (I2).

### 3. Recipients and routes are fixed before flight

- **Records.**
  - `recipient{key_txt, crew}`: `crew = 1` means on board, `0` means ground.
  - `route{class_txt, to_txt}`: appended in priority order.
  - Both are allowed only before the first `day_close`. This is the same argument as for the safety table (ADR-0007 §1): the safety route is a two-party agreement that the device cannot authenticate in flight.
  - The crew is fixed for the mission, so every possible recipient is known before flight. In flight the astronaut changes **who is excluded**, not who exists.
- **Compatibility, checked at validation**

  | Class | Allowed recipients | Why |
  |---|---|---|
  | `ppc` | Ground only | A Private Psychological Conference is held with the ground |
  | `crew` | On board only | It is the human route to a trusted crewmember |
  | `safety`, `summary` | Either | The CMO on board is a legitimate safety recipient. Summaries to a crewmate are the astronaut's choice |

- **The safety route needs ≥ 2 recipients at the first `day_close`.** A single exclusion, for example of the CMO when the CMO is the person concerned (M12), can then never silently empty the safety route.

### 4. Resolution: the first recipient that is not excluded and, for safety, reachable (I10, F5)

- **Record.** `link{up}`, written by the transport whenever ground contact is gained or lost. On board is always reachable.
- **Rule.** An item's recipient is the **first** recipient in its class route that is:
  - not excluded;
  - for `safety` only, also currently reachable.

  Other classes wait for their preferred recipient instead of skipping to the next one.
- **Why the classes differ.**
  - A safety notice is time-critical. During conjunction it must reach the CMO on board instead of waiting two weeks. The CMO is in the pre-agreed route, so this stays inside the contract.
  - A summary or a PPC request is not time-critical. Re-routing it, for example to a crewmate, would override the astronaut's choice of who sees what.
- **No dynamic delay threshold** (e.g. "prefer the CMO if the light time is over 5 min"). The right threshold depends on the line, and choosing it is a clinical decision. The route **order** agreed before flight already expresses it: a Mars crew can put the CMO first.
- **Re-resolution.** Every change of exclusion or link state re-resolves all unsent items.
  - Each item whose recipient changes yields `ShareRerouted{share, to}`.
  - An item with no eligible recipient yields `ShareUndeliverable{share, class}` and stays in the outbox; it becomes deliverable when a recipient becomes eligible again. Nothing is ever dropped silently.
  - An exclusion that leaves a class with no non-excluded recipient yields `RouteEmpty{class}` at the moment of the change, so the consequence is visible **before** anything needs sending.
- **Exclusion is never refused (I10, "without exception").**
  - **Counter-thesis:** excluding every safety recipient defeats the pre-signed safety lines.
  - **Answer:** I10 is stated without exception in the design. The ≥ 2 rule makes this a deliberate multi-step act, never an accident, and it is surfaced. The residual risk is stated: the safety lines are then shown on the device only. This joins L6b (human routes) as a limit.
- **Records.** `exclude{key_txt, on}` takes effect immediately in both directions, the same argument as consent withdrawal in ADR-0007 §2. A record that changes nothing is rejected.

### 5. Tier 2: weekly summaries per indicator, private by default

- **Record.** `share_category{key_txt, on}` switches sharing of one indicator's weekly summary. The default is off (privacy option A).
- **Content.** Every 7th closed day, each consented indicator queues one summary:
  - days with data in the week (coverage, I9);
  - mean daily z × 1000 (integer);
  - the alarm state.

  It contains no raw values and no daily series: it is the minimum that lets the ground see a trend with its coverage.
- **Withdrawal cancels unsent summaries of that indicator** (`ShareCancelled`).
- **Safety items are never cancelled by a later consent withdrawal.**
  - **Thesis:** withdrawal should apply to everything not yet sent.
  - **Antithesis:** a tier-3 share is a protective act authorised by a contract that was in force when the disclosure was made. A tier-2 summary is a discretionary disclosure.
  - **Decision:** consent is evaluated at the moment the information arises.
  - **UI requirement that follows:** the screener shows each item's current auto-share state **before** the astronaut answers, so no answer is ever given without knowing its consequence.

### 6. Human routes: `ppc_request{day}` and `crew_request{day}`

- **The one-tap routes of the high-stakes protocol** (ADR-0007 §3). They are astronaut-initiated by definition: the engine never creates one.
- **Payload.** Only "a request exists" and the day. Anything further is said between people, outside the system.

## Out of scope
- **Transport framing and encryption, acknowledgements from the ground, and the FHIR export (§4.16).** These live below the `sent` boundary.
- **Dyadic crew signals and the family bridge (§4.14–4.15).** These are separate opt-in features with their own consent.
- **Expected-utility timing of alerts (EVTA, §4.19).** It will decide *when to interrupt the astronaut*, not what is transmitted.

## Verification
- **Scenario tests** (`engine::tests`):
  - **Conjunction:** losing the link re-routes a safety notice to the CMO.
  - **Exclusions:** excluding both safety recipients yields `RouteEmpty` and an undeliverable item; including one again re-routes it.
  - **PPC request:** it waits for the ground instead of being re-routed to the crew.
  - **Bad acknowledgements:** a duplicate or unauthorised `sent` is refused.
  - **Tier 2:** only the consented week is summarised, and withdrawal cancels the unsent summary.
  - **Configuration:** a single-recipient safety route is refused at the first close.
- **Property test** (12 seeds × 35 days of random exclusions, link changes, requests, screens, crossings and transmissions, including a transport that acknowledges arbitrary shares). It checks:
  - I6: no `ShareSent` without an earlier `ShareQueued`;
  - I10: no queue, re-route or send to an excluded recipient;
  - no send to an unreachable ground recipient;
  - `ppc` only to the ground and `crew` only on board;
  - I2: every payload key is in the allowlist, and every value except `ind` is an integer.
- **Mutation check** (manual, 11 Nov 2026):
  - re-routing that ignores exclusions → caught (`I10 at queue`);
  - removing the `sent` eligibility check → caught.

## Consequences
- **Every transmission is explainable from the log alone:** which item, which consent, which route, which exclusions, and the link state at the time.
- **Residual limits**
  - A crash between transmission and `sent` can cause one idempotent duplicate.
  - An astronaut who excludes every safety recipient keeps the safety lines on the device only.
