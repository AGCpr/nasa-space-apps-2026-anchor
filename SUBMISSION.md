# Space Apps 2026 submission: text for each form field

Copy each block into the matching field of the Project page (My Team → Project → Edit). Everything is in English, as the rules require.

---

## Project Name

ANCHOR: a personal baseline for astronaut wellbeing

## High-Level Project Summary

ANCHOR is a daily wellbeing companion for long-duration crews.

- It learns each astronaut's own normal from pre-flight days, then notices *sustained* change calmly. It does not raise alarms on single readings.
- It never diagnoses, and it shows the evidence and confidence behind every note.
- Disclosure is the astronaut's decision. Everything is private by default, and every share is listed before it is sent. Pre-agreed safety lines, such as low blood oxygen, still always reach the flight surgeon, or the CMO on board during loss of signal.

This addresses NASA HRP's risks in four areas: behavioural health under isolation, sleep and irregular schedules, mild hypoxia in exploration cabins, and Earth-independent operations.

It matters because honest self-reporting is the first casualty when monitoring feels like surveillance, and Mars-distance crews cannot rely on real-time ground support.

On 71 real people's wearable and mood data, the personal range held 79 % of mood days and 78 % of sleep days (target 80 %). ANCHOR raised 2–5× fewer notes than a naive single-day rule.

## Link to Project Demo

https://agcpr.github.io/nasa-space-apps-2026-anchor/demo/anchor-demo.mp4 (30 seconds, 4K, narrated, English subtitles, real app footage)

## Link to Final Project

https://github.com/AGCpr/nasa-space-apps-2026-anchor (source, documentation, validation). The live app is at https://agcpr.github.io/nasa-space-apps-2026-anchor/app/ ; it opens on a labelled synthetic 150-day mission with a short guide to the three moments worth seeing.

## Detailed Project Description

**What it does**

- **Check-in.** A 90-second evening check-in with three bipolar sliders (mood, energy, calm). The astronaut rates *before* seeing any data, so the number never overwrites how they feel. Sleep and SpO2 are imported from a wearable.
- **Personal range.** Each value is compared with the astronaut's own range: a Bayesian baseline anchored on pre-flight days. It is drawn as a corridor, with the pre-flight level as a dashed benchmark.
- **Sustained change.** A CUSUM detector notices sustained change. Its threshold is calibrated so a stable person sees at most one unwanted note per 30 days; the end-to-end test gives one per 39 days.
- **Notes.** Notes use calm, audited language and always name a next step and who takes it. Every note has a "Why this note?" panel: the data, the rule, what would close it, and the NASA research area it relates to.
- **New normal.** "This is my new normal": after a sustained change the astronaut may accept a new level. It is reversible, limited to once per 30 days and to within 2 SD of pre-flight, and drawn as a hinge in the range.
- **Disclosure contract.**
  - Nothing is shared by default.
  - Weekly summaries are shared only per category, with consent.
  - Every share appears in a sharing log before it is sent.
  - Any recipient can be excluded, including the CMO.
  - Safety notices re-route to the CMO on board during loss of signal.
- **Safety lines.** Pre-agreed lines (for example SpO2 below 90 %) are checked by two independently written safety kernels, in Rust and TypeScript, with triple redundancy. If they ever disagree, the value is recorded as unconfirmed and shared as such, never shown as "fine".
- **Support.** A private conference or a talk with a trusted crewmate is one tap away. An optional screener shows, before each question, exactly where a "yes" would go.
- **Evening rendering.** The interface follows the ISS pre-sleep lighting: no colour bluer than 2700 K. It meets NASA-STD-3001 Appendix F display rules: reserved red, yellow and blue meanings, at least 6:1 contrast, and touch targets of 15–20 mm for microgravity.

**How it works**

- **Log.** Every input is a record in a hash-chained, append-only log, stored in two copies so that it is repairable.
- **Engine.** The engine is a deterministic fold over that log, so every output can be reproduced by replay. It is written in Rust and compiled to WebAssembly.
- **Interface.** Vanilla TypeScript and SVG, a pure projection of the engine's outputs, with no frameworks or runtime dependencies. It runs fully offline.
- **Decisions.** Every design decision is argued, with rejected alternatives, in nine Architecture Decision Records.

**Evidence**

- **Real data.** On LifeSnaps (71 adults, 4 months, CC BY 4.0), the unmodified engine processed 13,376 records with 0 errors at 6.7 ms per record.
  - The personal range was calibrated for mood and sleep: 79 % and 78 % of days inside the 80 % band.
  - ANCHOR raised one note per 95–104 days per indicator, against one per 21–46 days for a naive rule.
  - We also report a negative finding. The band is too narrow for smoothed device signals such as Fitbit resting heart rate (lag-1 autocorrelation 0.87), and this is now a configuration rule.
- **Checks.** About 70 automated tests, Python reference oracles, mutation checks, and a machine audit of every sentence the astronaut can read.

**Benefits and goals**

- Earlier, gentler awareness of sustained change, which the astronaut owns.
- More honest reporting, because sharing is their choice.
- Safety lines that always work.
- An auditable record for the flight surgeon.
- Next step: a trial in an analog mission (for example HERA), with real crews and a clinician-validated screener.

**Tools**

Rust (engine, safety kernel A, WebAssembly), TypeScript (interface, safety kernel B), Python with NumPy and SciPy (reference oracles, simulation, validation), SVG, ffmpeg, puppeteer-core, Pillow and Kokoro TTS (demo video, rendered at 3840x2160), GitHub Pages.

## Use of Super Intelligence (SI)

Yes. We used an AI coding assistant (Claude, by Anthropic) during the software phase.

- **The team decided:** the challenge, the product concept and its requirements, the visual direction, and every design decision. Each decision is recorded with alternatives in the ADRs.
- **The assistant helped with:** literature and standards research, drafting design documents, writing code and tests, running the validation, and building the demo video from frame-by-frame recordings of the real app.
- **How we checked it:** two independent safety-kernel implementations, Python reference oracles, about 70 automated tests including mutation checks, and an independent design review.
- **Disclosure in the project itself:** the repository's AI_ASSISTANCE.md, `Co-Authored-By` commit metadata, and a caption in the demo video.
- **Not AI-generated:** no images or video frames. Every app shot is a recording of the running app.
- **Narration:** synthetic speech (Kokoro, open-weights neural text-to-speech), disclosed on the video's end card and in its file metadata.

## NASA Data

We used the NASA Human Research Program resources listed for this challenge: *Risks of Human Spaceflight*, *5 Hazards of Human Spaceflight* and the HRP risk pages. Every indicator and feature of ANCHOR traces to one of these risks (mapping with quotes: docs/HRP_MAPPING.md):

- **Risk of Behavioral Changes and Psychiatric Disorders.** Mood, energy and calm; calm detection without diagnosis; privacy by design, which the page calls a top priority.
- **Risk from Inadequate Sleep and Irregular Schedules.** The sleep indicator, the exclusion of workload days, and the 2700 K evening rendering modelled on the ISS LED lighting schedule.
- **Risk of Hypoxia.** The SpO2 safety line, motivated by partially depressurised exploration cabins.
- **Risk of Inadequate Teamwork.** The trusted-crewmate route and recipient exclusions.
- **Risk of Inadequate Systems Operations Support.** Offline operation, the "Why this note?" explanations, and deterministic, auditable outputs.

We also used NASA-STD-3001 Vol. 2, Appendix F (display colour, contrast, characters and touch targets) to design the interface.

## Space Agency Partner Data and Other References

- **CSA, "Monitoring Radiation for Astronaut Health".** Reviewed. Personal radiation tracking is out of scope, because crew dosimetry already covers it; stated in our design paper.
- **LifeSnaps dataset.** Yfantidou et al., *Scientific Data* 9, 663 (2022), Zenodo 7229547, CC BY 4.0. Used for validation and not redistributed.
- **Atkinson Hyperlegible font.** Braille Institute, SIL Open Font License 1.1.
- **Design references.** Methods literature cited in DESIGN_PAPER.md and docs/UI_RESEARCH.md: CUSUM and Siegmund approximations, the conjugate Normal-Inverse-Gamma baseline, Theil–Sen trend, C-SSRS screener structure, Stuster's astronaut journals, the ISS SSLA lighting studies, and touch-target studies in microgravity.
- **Software.** Rust (sha2 crate), TypeScript, Python, NumPy, SciPy, ffmpeg. No images, audio or video from third parties are used.

---

## Before you press "Submit for Judging"

- **Deadline:** Sunday 15 November, 23:59, in your Local Event's time zone.
- **Team:** every team member is a confirmed participant of the same Local Event.
- **Links:** open each one in a private window (no sign-in): the demo video, the GitHub repo and the live app.
- **Form:** tick "Terms and Conditions" and "Confirm All Work is Original", then press **Submit for Judging**. A confirmation banner appears. You can edit and resubmit until the deadline.
