# Use of AI (Super Intelligence) in this project

The NASA Space Apps rules ask that AI use be stated in descriptive text and in metadata. This file is the descriptive text. In the metadata, every commit made with the assistant carries a `Co-Authored-By: Claude` trailer.

## Tool

Claude (Anthropic), used as a coding and research assistant in Claude Code, in November 2026.

## What the team decided (not delegated)

- The challenge choice.
- The product concept and its requirements:
  - a daily companion the astronaut *wants* to use, not a chore;
  - psychological support;
  - tracks values and warns about risks without ever diagnosing;
  - no harsh or fear-inducing language;
  - learns the person over time;
  - private by default.
- The visual direction (chosen from four researched options).
- The decision to validate on real public data, and to publish.
- Every design choice was put to the team as a decision, with alternatives argued in the ADRs (`docs/adr/`).

## What the assistant did, under the team's direction

- **Research:**
  - literature and standards searches (NASA HRP pages, NASA-STD-3001, the papers cited in DESIGN_PAPER.md and docs/UI_RESEARCH.md);
  - a survey of 243 public repositories (LANDSCAPE.md).
- **Drafting:** design documents and ADRs, which the team reviewed.
- **Software:**
  - the Rust engine, the WebAssembly boundary, the TypeScript interface and safety kernel B;
  - the Python reference oracles;
  - the analysis scripts.
- **Testing:**
  - unit, property, mutation and end-to-end tests;
  - the language audit;
  - colour and contrast tests;
  - browser verification.
- **Validation:** the LifeSnaps real-data run and its report (docs/VALIDATION.md).

## How AI output was checked

- **Independent implementations.** The safety kernel exists twice, in two languages, plus a Python reference. Numerics and detection are checked against Python and SciPy oracles with stored vectors.
- **One command re-runs every check.** `bash verify.sh` runs oracles, about 70 tests, lints, the WebAssembly build and the interface tests. The end-to-end false-alert budget test runs with `--full`.
- **Mutation checks.** These confirmed that key tests fail when the behaviour they guard is broken: routing exclusions, transport authorisation, the language audit and the colour rules.
- **Independent design review.** The interface was reviewed by a separate reviewer pass against its written design contract, over four rounds, before release.
- **Negative results kept.** Results that went against the design are reported, not hidden. Examples are the resting-heart-rate band finding in docs/VALIDATION.md and the demo seed choice in ADR-0009 §12.

## What is not AI-generated

- No images are AI-generated. Screenshots and every app shot in the demo video are captures of the running app, recorded by `demo/record.mjs`.
- **Audio is the one exception.** The demo video's narration is synthetic speech (Kokoro, an open-weights neural text-to-speech model, voice af_heart). It is disclosed on the video's end card and in the MP4 file's metadata (`comment` field), and the script is `demo/make_video.py`.
- The font is Atkinson Hyperlegible (Braille Institute, OFL).
- No NASA branding is used.
