# ADR-0009: Interface and WebAssembly boundary

**Status:** accepted · 14 Nov 2026

## Context

The engine (ADR-0006 to ADR-0008) is a deterministic fold over a hash-chained log. It has no user. This ADR decides how that engine reaches an astronaut on a touch tablet without weakening any of its properties. The visual direction and the 19 interface requirements were settled in docs/UI_RESEARCH.md: direction A "survey sheet", with the evening rendering and the mission-day strip.

These constraints are binding:

- **I5:** every output is reproducible by replaying the log. No interface state may exist that the log does not support.
- **I6:** a sharing-log entry is visible before anything is transmitted.
- **I3 and I4:** every sentence contains no disease name or banned term, and names an action and an owner.
- **I9:** no "all good" screen; coverage and confidence are shown with every claim.
- **N1:** fully offline. **N3:** no data loss.
- **PRODUCT.md, Stack:** static files, vanilla TypeScript, no npm runtime dependencies, no bundler, no wasm-bindgen. The user confirmed this on 11 Nov 2026.
- **Safety architecture (ADR-0003):** kernel B is an independent N-version of kernel A, and any disagreement must be treated as a failed check.

## Decisions

### 1. One engine, compiled to WebAssembly; `unsafe` confined to a separate crate

- **Decision.** `core/` compiles unchanged to `wasm32-unknown-unknown`. A new crate, `wasm/`, holds the whole boundary. It is about 200 lines and is the only code in the project that uses `unsafe`: raw pointers into linear memory. `core` keeps `#![forbid(unsafe_code)]`.
- **Alternatives rejected:**
  - **Re-implement the engine in TypeScript.** This creates two folds whose outputs must agree bit for bit, including Monte-Carlo calibration. Every future change would need a cross-language proof, and I5 would hold only as far as the two implementations agree. N-version diversity belongs where it buys safety (the kernel), not in the statistical companion.
  - **wasm-bindgen.** It is the conventional route, but:
    - its CLI is version-locked to the library;
    - it generates JavaScript glue that no one in the project reviews;
    - it adds a build-time toolchain the stack decision excludes.

    The boundary needs exactly five functions. A hand-written ABI of five functions can be read in full.
  - **The WebAssembly component model (WIT).** Browsers do not run components natively. A transpiler (jco) would bring back the npm toolchain.
  - **A server-side engine.** This violates N1. Mars transit has a 22-minute one-way light time and two-week conjunctions.

### 2. Wire format: the log's own grammar in, hand-serialised JSON out

- **In.** The interface submits an unsealed record as `met_ms|kind|k=v;...`. This is exactly the field grammar of ADR-0002: keys sorted, integers canonical, text hex-encoded.
  - The boundary adds `v1|seq|` and the previous hash, computes the hash, and passes the line to the **existing** `parse_line`.
  - So the trust boundary uses the one parser that already has oracle vectors. No second parser is written for untrusted input.
- **Out.** JSON is produced by one exhaustive `match` over `Output`. Adding an `Output` variant without serialising it is therefore a compile error. The browser parses the JSON natively (`JSON.parse`).
- **Alternatives rejected:**
  - **serde / serde_json.** `core` has one dependency (sha2) by design (ADR-0001). The output enum is small enough to serialise by hand. That is about 60 lines, against a derive tree we would not audit.
  - **JSON in.** It would need a JSON parser in Rust at the trust boundary: either a dependency or new code without oracle vectors.
  - **A binary format** (FlatBuffers, CBOR). It needs code generation or a library on both sides, and the interface would lose the ability to log a readable trace.

### 3. Rust owns the log; the interface submits and persists

- **Decision.** The WebAssembly instance holds the `Log` and the `Engine`. `submit(record)` does four things, in order:
  1. it seals the record (sequence number, previous hash, SHA-256);
  2. it validates the record with `Engine::validate`, the ADR-0006 §7 trust boundary;
  3. it appends the record;
  4. it applies the record.

  It returns the sealed line and the outputs. A rejected record is never appended. The interface only stores lines and renders outputs.
- **Alternative rejected: the interface holds the log, and the engine is a pure function.** The interface would then compute hashes and sequence numbers. That is a second implementation of sealing, and a rejected record could reach storage before the engine saw it.

### 4. Persistence: two synchronous copies in `localStorage`, repaired on load

- **Decision.**
  - Each sealed line is appended to two `localStorage` keys, A and B, **synchronously**. This happens before any output is rendered and before the transport may act.
  - On load, ADR-0002's `repair(a, b)` reconciles the copies, and then the engine replays. The app also asks the browser for persistent storage (`navigator.storage.persist()`).
- **Capacity.** Measured on the 150-day demo: about 0.62 kB of log per mission day (93 kB for 150 days). The roughly 5 MB per-origin quota therefore holds about 20 years.
- **Alternatives rejected:**
  - **IndexedDB.** Its transactions are durable but asynchronous. The rendering and transport code would have to await every commit, or a share could be shown and transmitted before its authorisation is on disk. That reverses I6's order after a crash.
  - **OPFS.** Synchronous access handles exist only in workers. A worker adds a message protocol in front of the boundary for no gain at this data volume.
  - **One copy.** ADR-0002's repair needs two independent copies to recover from a torn write.
- **Limit (stated).** Browser storage is best effort, and a user can clear it. A flight build would put the same two-copy log on the device file system behind a native shell. That is out of scope here.

### 5. The interface is a projection of the output stream; scrubbing is a prefix

- **Decision.** Every screen is computed from the list of `(record, outputs)` pairs. Nothing else is read: there is no query API into engine state.
  - The engine is a fold, so the outputs up to sequence *s* do not depend on any later entry (I5).
  - The demo's day scrubber therefore just **filters the stream** at the `day_close` of the chosen day, with no recomputation. A 150-day replay, including 22 Monte-Carlo recalibrations, is computed once.
  - Acting on a past day ("continue from here") replays that prefix once into a fresh engine. Later entries are discarded, and the app says so.
- **Alternatives rejected:**
  - **A `view()` query on the engine.** It returns only the current state. A scrubber would need it at every past day, which means snapshots, a second path for state that replay does not exercise.
  - **Re-replaying on every scrub.** This is correct but slow: seconds per move on a tablet.

### 6. `DayStat` carries the personal range, so the corridor is drawn from the log

- **Decision.** `DayStat` gains four values on the model scale:
  - `y`: the day mean;
  - `lo` and `hi`: the central 80% predictive interval of the rolling baseline that standardised that day;
  - `anchor`: the mean of the anchor in force.

  `Engine::display(indicator, x)` inverts the indicator's transform for presentation: logistic for sliders, divided by 10^d for scaled values. It sits next to the forward transform and has a round-trip test.
- **Why this is the signature move and not decoration.**
  - The corridor *is* the rolling baseline's uncertainty. A wide corridor is a less certain model (R16).
  - The dashed benchmark is the anchor. A hinge is the day `anchor` changes, and it coincides with `Reanchored` or `ReanchorReverted`.
  - Nothing on the Course screen is drawn from anywhere but the log.
- **Why 80%.** It is a range most days fall inside. "About 8 in 10 of your days fall in this band" is a sentence a person can check against their own experience. A 95% band would make almost every day look "normal" and hide the gradual change the companion exists to show.
- **Alternatives rejected:**
  - **Draw a band from `z` alone.** That is impossible: `z` has no location or scale.
  - **Show σ or z** (DESIGN_PAPER §4.10 forbids it). The interface shows personal percentiles ("lower than 9 of your last 10 weeks") and the band.

### 7. N-version safety comparison at the interface, recorded in the log

- **Decision.**
  - After every value-carrying record, the interface runs **kernel B** (TypeScript) on each value. It uses the table currently in force: the `safety_table` record, then the newest `ConsentChanged.table`. Kernel B keeps its three copies in separate strings, digest-checked by WebCrypto.
  - The interface compares B's outcome with kernel A's outcome in that record's outputs (`Safety`, `SafetyCheckFailed`, or no hit), using `compareKernels`.
  - **On disagreement,** the interface appends `kernel_disagree{day, ref, key_txt}`. The engine validates that `ref` is a value record of the open day that carried `key_txt`. It then treats the record exactly as a kernel fault: it emits `SafetyCheckFailed`, and it queues a `safety` fault share unless the value came from a screen (ADR-0008 §2).
- **Why in the log.**
  - A disagreement is safety-relevant. It must reach the ground (Principle 5: a disagreement is never shown as "fine").
  - It must also survive replay. A notice shown only in the interface would vanish on reload, and the ground would never learn of it.
- **Alternatives rejected:**
  - **Run kernel B inside the engine.** Diversity of language, runtime and integer model is the point (ADR-0003). Kernel B compiled into the same WebAssembly module would share the runtime it is meant to be independent of.
  - **Compare in the interface and only display.** It is not auditable, and it is not shared.
- **Limit (stated).** Kernel B is asynchronous, because WebCrypto is. The interface blocks further input for that day until the comparison has run, so a `day_close` cannot overtake it.

### 8. A template catalogue with a machine audit

- **Decision.**
  - Every sentence the astronaut reads comes from `app/src/catalogue.ts`. Each template declares:
    - its **register**: advisory, caution or warning (DESIGN_PAPER §4.10, mapped to NASA-STD-3001 colours in UI_RESEARCH R1);
    - its **action**;
    - its **owner** ("you", "your flight surgeon", "the CMO on board").
  - The catalogue is keyed by locale. Only `en` ships; the audit runs per locale, so a Russian catalogue inherits the checks.
- **The audit (`app/test/audit.test.ts`)** renders every template with every slot value: every indicator label, every recipient, extreme and boundary numbers. It fails on:
  - a banned term: disease and diagnostic names, alarm vocabulary, blame;
  - an exclamation mark;
  - a missing action or owner;
  - more than 3 sentences in a warning;
  - a sentence over 25 words.
- **Copy outside the catalogue.** Interface chrome in `views.ts`, `main.ts` and `ui.ts` (labels, status words, payload descriptions) is scanned lexically: every string literal of two or more words gets the banned-term and exclamation checks. This is weaker than the catalogue audit, because it has no action or owner check. Moving that copy into the catalogue is required before a second locale ships.
- **Mutation check (14 Nov 2026).** The audit was confirmed to fail on a banned term plus an exclamation mark, and on an empty action.
- **Coverage test.** The same test instantiates the WebAssembly engine under Node, plays the demo mission, and fails if any emitted output kind has no template. The TypeScript union of output kinds makes the mapping total at compile time as well.
- **Alternatives rejected:**
  - **ICU MessageFormat or Fluent libraries.** These are npm runtime dependencies. `Intl.PluralRules` is built in and covers plurals.
  - **Generated text (LLM).** Already rejected in DESIGN_PAPER §4.10: non-deterministic and unauditable.

### 9. Evening rendering follows the lighting schedule

- **Decision.**
  - Two token sets, *day* and *evening*, are CSS custom properties switched by `data-light`.
  - In *auto* mode, the switch follows the ISS crew-day schedule in GMT: pre-sleep from 19:30, about two hours before sleep, which is when the SSLA switches to pre-sleep mode. In the replay it follows the replayed time.
  - The astronaut can fix the mode manually.
- **Checked mechanically** (`app/test/tokens.test.ts`):
  - every text and background pair has a contrast ratio of at least 6:1 in both modes (R3);
  - every evening token has a blue channel of at most 35% of its largest channel (R10: no short-wavelength accents);
  - day-mode advisory blue is the only blue hue in either set (R1);
  - in day mode, every saturated decorative hue keeps at least 15° from caution yellow and warning red (R1). Ochre contours are the closest, at 16°. They are thin lines only, never a fill or a label.
- **Found by the test.** The first day-mode warning red reached only 5.5:1 on the panel colour; it was lightened to pass 6:1.
- **Limit (stated).** In evening rendering everything is warm, so caution yellow and warning red are close in hue to the amber text. There they are told apart by shape (triangle, diamond) and label only, which R2 already requires of every state.
- **Alternative rejected: follow `prefers-color-scheme`.** That follows the tablet's setting, not the station's lighting, and the countermeasure is tied to the schedule.

### 10. Rendering: vanilla DOM and SVG, re-rendered on committed events

- **Decision.**
  - Each view is a function from the projected state to DOM nodes, built with a 20-line `h()` helper.
  - The active view is re-rendered after each committed record. It is never re-rendered during a drag, so a slider keeps its pointer capture.
  - Charts are SVG, each with a text alternative: a summary sentence plus the underlying table.
- **Why SVG over Canvas.**
  - SVG is accessible: real text and titles.
  - It is crisp at any zoom.
  - The data is at most a few hundred points per indicator.
- **Alternative rejected: a virtual-DOM library.** That is a runtime dependency (excluded), solving a performance problem that does not exist at this scale.

### 11. Touch: drag anywhere, commit on release

- **Decision.**
  - A slider is the whole track (at least 104 CSS px tall, R7). A press anywhere moves the value there; the value follows the finger and commits on release (R9).
  - Every other control is at least 80 CSS px in its smaller dimension. Primary actions are at least 104 px.
  - There is no hover, no long-press and no swipe-only action.
  - Each slider has words at both ends and a neutral middle (R15). Its untouched state is visibly "not answered", not 50.

### 12. The demo is a labelled synthetic mission generated by the engine itself

- **Decision.**
  - `core/src/synth.rs` generates a 150-day mission with scripted events, driving a real `Engine` while it writes the log, so every scripted action is valid when it is appended. The events are:
    - a mood shift on day 100;
    - an astronaut re-anchoring on day 125;
    - a loss of signal on days 118–122;
    - an SpO2 safety crossing during that loss of signal (re-routed to the CMO on board);
    - a check-in gap on days 130–138;
    - a weekly sleep summary shared and then withdrawn;
    - a screener and a consent change;
    - a simulated transport acknowledging every transmittable share.
  - **Seed choice (stated).** The generator's first seed (2026) produced a false mood alarm on day 87. It stayed latched into the scripted shift, so the demo could not show the shift being noticed. The ignored test `find_demo_seed` picks the first seed whose mood episode starts after the shift: seed 1, alarm on day 103. False alarms on other indicators fall as they fall and are shown: on days 55, 62 and 110, among others. The false-alert rate is established separately by the end-to-end budget test (ADR-0006, N4), not by this demo.
  - The app shows "Synthetic replay" in the top line whenever this log is loaded.
  - The live log and the demo log never share storage.
- **Why in Rust.** The scenario then has a Rust test that asserts each scripted event produces its expected outputs, so the demo cannot silently stop demonstrating what it claims.

## Out of scope

- A pre-flight configuration editor. The demo and live modes start from a generated pre-flight package, labelled synthetic.
- Wearable import. Sleep and SpO2 arrive as `sample` records only in the demo.
- A Russian catalogue: the structure and audit are ready.
- BOCPD explanations and EVTA timing.

## Verification

- **Rust:**
  - `wasm` crate tests run on the host: the sealing round-trip, rejection of a bad record without an append, and JSON for every `Output` variant;
  - `synth` scenario test;
  - `kernel_disagree` validation and fault-share test;
  - `display` round-trip.
- **Node (`app/test`):**
  - the catalogue audit and coverage test, run against the real `.wasm`;
  - the token contrast and blue-channel test;
  - a projection test: the outputs at a scrub prefix equal a fresh replay of that prefix.
- **Browser (14 Nov 2026, built-in browser, 1180 × 820).** Exercised:
  - all four views in day and evening rendering;
  - a check-in;
  - a screener answer shared under consent;
  - a refused re-anchoring;
  - live mode persisting across a reload;
  - repair of a damaged copy A from copy B, with the repaired log written back;
  - `?inject-kernel-b-fault`: kernel B silently missing every line produced a recorded `kernel_disagree` and a "could not confirm" note.
- `bash verify.sh` builds the WebAssembly module and runs all of the above.

## Consequences

- The interface cannot show anything the log does not support. Any screen can be audited by replaying the log.
- The project now ships one `unsafe` crate. It is small, has no logic of its own, and is covered by host-side tests.
- A disagreement between kernels is now a first-class, shared, replayable event rather than a log line in a console.
- The colour and contrast requirements are tests, not intentions.
