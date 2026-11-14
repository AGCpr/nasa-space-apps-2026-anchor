# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

- **Static files only.** Vanilla TypeScript compiled by `tsc` to native ES modules, served from local files (Python's standard `http.server` for development).
- **No npm runtime dependencies and no bundler.** This was confirmed by the user on 11 Nov 2026.
- **The Rust engine (`core/`) runs in the browser as WebAssembly** (`wasm32-unknown-unknown`), with a hand-written ABI and no wasm-bindgen. Kernel B (`kernel-b/`, TypeScript) runs beside it for the N-version safety check.

## Users

- **Primary.** An astronaut on a long-duration mission (ISS today, Mars transit as the design case), using a touch tablet (iPad class), usually at the end of the crew day in dim crew quarters. The jobs are:
  - a daily check-in of 90 s or less;
  - seeing what their own data says about them;
  - deciding what to share and with whom;
  - reaching a person when they need one.
- **Secondary.** NASA Space Apps judges evaluating the demo, through a clearly labelled synthetic mission replay.

## Product Purpose

- **What it is.** ANCHOR is a daily companion the astronaut *wants* to use, not a mission chore. It:
  - learns the person's own normal;
  - notices sustained change calmly;
  - never diagnoses;
  - keeps the astronaut in control of disclosure.
- **What success means:**
  - the check-in becomes habit;
  - honest reporting increases because sharing is the astronaut's choice;
  - pre-agreed safety lines still always work.

## Positioning

These mechanisms are unique in a 243-repository landscape (LANDSCAPE.md):
- **The disclosure contract:** pre-signed safety lines, a device-only journal and a visible share log.
- **An anchored personal baseline with consented re-anchoring** ("this is my new normal" accepts a level, never a trend).
- **A false-alert budget** of at most one alert per 30 days, held end to end.
- **A verified safety kernel** with TMR and N-version checking.

## Operating Context

- **Fully offline**, with communication delay and loss of signal (conjunction).
- **Microgravity:** a floating tablet and imprecise touch, so targets must be large.
- **Circadian-lit crew quarters in the evening.** Blue light should be low at night.
- **Crew structure:** a CMO on board, a flight surgeon and behavioural-health team on the ground, and the Private Psychological Conference (PPC).
- **Mission timeline:** days numbered from launch; context tags such as EVA and workload.

## Capabilities and Constraints

- **Engine outputs**, defined in `core/src/engine.rs`. Every output must be rendered as calm text from a template catalogue:
  - safety hits;
  - alarms and clearances;
  - drift and anchor-run prompts;
  - missingness;
  - re-anchoring;
  - consent changes;
  - private support;
  - the share queue (queued, sent, rerouted, undeliverable, cancelled);
  - route-empty notices.
- **Invariants the interface must keep visible:**
  - I3: no disease names or banned terms;
  - I4: every message names an action and an owner;
  - I6: the sharing log is shown before transmission;
  - I9: no "all good" screen, and every summary shows coverage and confidence.
- **Further rules:**
  - no σ is ever shown; personal percentiles and plain-language confidence instead;
  - the screener shows each item's current auto-share state before the astronaut answers;
  - the screener is voluntary, never forced.
- **UI copy is in English**, held in one template catalogue that is ready for more languages (the design names Russian as needed). A machine audit checks the catalogue.

## Evidence on Hand

- `core/src/engine/tests.rs` generates **synthetic missions**: a 150-day mission with a mood shift, a safety crossing and a missing-check-in gap. The demo replay uses this generator. All data is synthetic and must be labelled so.
- **There is no real astronaut data, no testimonials and no endorsements.** None may be implied.

## Product Principles

1. The person's own normal is the reference, and the person may redefine it, visibly and reversibly.
2. Calm by construction: one alert per episode, majority framing, a counterfactual on every insight, no alarm vocabulary.
3. Disclosure is the astronaut's decision, except the pre-signed safety lines they agreed to before flight.
4. Never claim more certainty than the data supports: coverage and confidence are always shown.
5. Safety first in the architecture: the kernel always runs, and a disagreement is never shown as "fine".

## Accessibility & Inclusion

- **Large touch targets** for microgravity: at least 80 CSS px (15 mm), and 104 CSS px (20 mm) for primary controls (UI_RESEARCH R7).
- **Readable under dim light.** Text contrast of at least 6:1 (NASA-STD-3001 App. F, stricter than WCAG AA), checked by `app/test/tokens.test.ts`.
- **No information carried by colour alone.**
- **Reduced motion respected.**
