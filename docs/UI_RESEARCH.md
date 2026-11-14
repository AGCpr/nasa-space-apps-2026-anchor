# UI research: what crew interfaces and wellbeing apps already do, and what ANCHOR's interface must be

**Status:** research for ADR-0009 (UI + WASM) · 14 Nov 2026
**Scope:** the astronaut-facing tablet interface. Ground and clinician views are out of scope.

## 1. Sources examined

| # | Source | What it is | Strength |
|---|---|---|---|
| S1 | NASA-STD-3001 Vol 2, **Appendix F, Display Standard** | NASA's display requirements; explicitly covers "portable computers, including laptops and tablets" (F.2) | Normative |
| S2 | NASA-STD-3001 Vol 2 Rev F, compliance matrix; OCHMO-TB-052 *Crew Controls* (Oct 2026) | Crew-interface requirements and rationale | Normative |
| S3 | Orion Display Format Standard (MPCV 72242), as described in the NASA Commercial LEO HSI workshop (2024) and an NTRS Orion HF paper (2011) | The standard chain Orion → Gateway → HLS → Artemis; checklist verification per display | Programme practice; the standard text itself is not public |
| S4 | STS-70 EDOMP touchscreen test (NTRS 20040201530) | The first test of touchscreen use in microgravity | Flight data, n = 5 |
| S5 | ISS touchscreen pointing study (arXiv 2604.17322, 2026) | 420 pointing tasks on the ISS with 5, 10 and 15 mm targets, plus WHO-5 and perceived-stress scales | Flight data, n = 2 astronauts |
| S6 | Aviation touchscreen-under-vibration studies (EFB, fighter and entry-task studies) | Target-size recommendations | Ground, vibration |
| S7 | Crew Dragon touchscreen accounts (Space.com, TechCrunch, arXiv 2408.00085) | Astronaut-in-the-loop refinement of touch registration and target size | Practice, qualitative |
| S8 | Reaction Self Test (RST) on ISS (NTRS 20090029851; Pulsar case study) | Astronaut self-feedback relative to norms; "simple enough to use without assistance"; prototyped with NEEMO crews | Flight practice since 2009 |
| S9 | Virtual Space Station (VSS) (HRR task 1176; Anderson 2016, HI-SEAS) | Autonomous behavioural-health self-help, designed with 13 veteran astronauts; **privacy was the main design driver** | Analog evaluation |
| S10 | Playbook self-scheduling (NASA Ames: HERA C3, C6, C7; NEEMO; SpaceCHI 2023 and 2025) | Tablet crew tool; aids help in one campaign and **hurt** in another; learning effects | Analog, n = 32 |
| S11 | ISS crew-autonomy demos (arXiv 1910.03014) | Crews over-trust software recommendations; information on demand | Flight demos |
| S12 | ISS Solid-State Lighting (NTRS 20160005080; Brainard; PMC11316501) | 4500 K general, **2700 K pre-sleep** (blue removed), 6500 K phase-shift; dim, blue-depleted light before sleep | Flight system plus analog RCT |
| S13 | EMA design literature (Interacting with Computers 2020; JMIR 2024 factorial, PMC11347889; PMC11079761) | Burden, item count, slider versus Likert, feedback | Peer-reviewed |
| S14 | Orthosomnia (Brain Sciences 2024, PMC11592250) and Oura/WHOOP critiques | Score-driven anxiety | One peer-reviewed study plus journalism |
| S15 | Apple Health *State of Mind* (WWDC24 session 10109) | A valence slider from −1 to 1 with 7 classes, then optional labels and associations | Shipping practice |
| S16 | Competitor hackathon UIs (LANDSCAPE.md: HELIOS, AURORA, RAKIA, …) | Mostly vitals dashboards, cards and gauges | Field context |

## 2. Findings that become requirements (the floor every option must meet)

### Colour and alerting (S1, S2)
- **R1. Colour carries meaning only, never decoration** (F.5.10).
  - **Red** = emergency or warning (F.6.4.1–2). **Yellow** = caution (F.6.4.3). **Blue** = advisory.
  - ANCHOR maps these to the paper's registers (§4.10):
    - **safety line → warning (red);**
    - **sustained-change alarm → caution (yellow, worded calmly);**
    - **prompts and insights → advisory.**
  - No other hue may imitate them. In particular, **blue must not be used as a brand or "sharing" colour**, because blue means advisory.
- **R2. Colour is never the only cue** (V2 10045, F.5.10). Every state also has a shape, a label or a position.
- **R3. Text contrast ≥ 6:1, 10:1 preferred** (F.5.10). This is stricter than WCAG AA (4.5:1). We adopt it.
- **R4. Type.**
  - Sans serif, with clear l/1 and O/0 distinction (F.5.1.2–3).
  - Fixed-width (tabular) figures for numbers and tables (F.5.1.4).
  - Mixed case (F.5.1.8).
  - Character height ≥ 0.25°, 0.4° preferred (F.5.1.7). At a ~50 cm tablet distance, 0.4° ≈ 3.5 mm cap height, roughly 19 CSS px of body text on an iPad.
- **R5. Alerts are acknowledgeable, with visible state, and no flashing** except for events needing immediate attention (F.6.4.6, F.6.4.9, F.5.11).
  - ANCHOR has **no** event needing immediate attention outside the safety lines, so the companion layer **never flashes**.
  - This matches the paper's calm register and ADR-0006 latching: one alert per episode, cleared state shown.
- **R6. No horizontal scrolling. Vertical scrolling only for lists** (F, display layout).

### Touch in microgravity (S4–S7)
- **R7. Touch target ≥ 15 mm, 20 mm for primary or safety-relevant controls.**
  - STS-70 gives 10 mm as the *minimum*. The ISS 2026 study finds pointing *speed* equal to the ground, but movement error and variability **significantly higher**.
  - The vibration and EFB studies recommend 15 mm generally and ~20 mm for safety-critical controls.
  - On an iPad (≈ 0.19 mm per CSS px), that is **≥ 80 CSS px (15 mm), and ≥ 104 CSS px (20 mm) for the primary action**.
  - This is roughly twice Apple's 44 pt guideline, and that is deliberate.
- **R8. Finger, not stylus.** Finger pointing was significantly faster than stylus in microgravity (S5).
- **R9. Prefer controls that tolerate an imprecise landing:**
  - **sliders accept a drag anywhere along the track**, and the value is set on release;
  - discrete choices are large segmented buttons;
  - no small handles, no hover, no long-press.
  - Dragon's refinement work focused exactly on how a touch is *registered* (S7).

### Light and time of day (S12)
- **R10. The interface follows the ISS lighting schedule.**
  - Most check-ins happen at the end of the crew day, which is exactly when the SSLA switches to **pre-sleep 2700 K, blue removed**. A bright, blue-rich screen at that moment undoes a flight countermeasure.
  - ANCHOR therefore has an **evening rendering**: a dim, low-luminance, warm palette with no short-wavelength accents, switched by the mission timeline (or manually).
  - Conflict with R1: advisory blue cannot appear in evening rendering. Resolution: in the evening, advisories rely on their **shape and label** (R2 already guarantees these), rendered in the warm neutral. Warning red and caution amber survive, because they are long-wavelength.

### Daily use and burden (S8, S13–S15)
- **R11. Check-in ≤ 2 min, target ≤ 90 s** (F1). Sliders and Likert scales give equal compliance (JMIR 2024). Fewer items trend towards higher compliance. **Open-ended prompts add burden**, so the journal stays optional.
- **R12. Feedback after the check-in raises compliance.** Showing people their own data raised compliance by 23% (Hsieh 2008, in S13). RST was built on the same idea: feedback relative to *one's own* norms (S8).
- **R13. Rate first, then see the data.**
  - Sleep clinicians advise writing down how you feel *before* opening the tracker (S14). Score-first design drives orthosomnia and lets the number overwrite felt state.
  - ANCHOR therefore never shows the person's model before that day's sliders are submitted.
  - This protects the self-report, and it is also exactly what the Calibration loop needs: the prediction must come before the measurement.
- **R14. No composite "readiness" score.** Composite scores are unvalidated (S14) and would violate I9.
- **R15. Valence as a bipolar slider with words at both ends and a neutral middle** (Apple's State of Mind, S15). ANCHOR's valence/energy/calm sliders follow this. No smiley faces: they are a known framing bias and childish in this context.

### Trust, autonomy, privacy (S9–S11)
- **R16. Show confidence and coverage next to every claim.** Crews over-trust software recommendations (S11), so the interface must make uncertainty visible (I9, §4.13).
- **R17. Detail on demand, not by default** (S11). The first view is one sentence, with the "why" one tap away.
- **R18. Aids can hurt.** Playbook's C7 "no-go zones" *reduced* performance where C6 aids had helped (S10). Every assistive overlay is therefore optional, and the core flow must work without it.
- **R19. Privacy is the adoption driver** (S9: the VSS was designed around fear of grounding).
  - Where data goes must be visible at the place where data is entered: the share state is shown on the check-in and screener screens themselves (ADR-0008 §5), not buried in settings.

### What the field ships, and why we diverge (S16, S14)
- **Hackathon competitors:** vitals dashboards with gauges, rings and red/green status cards.
  - Gauges and rings invite the score reading R14 forbids.
  - Red/green status for psychological data turns R1's warning red into a mood label.
- **Consumer wearables:** daily scores with a gentle tone. The tone is good. The score-first order is the problem (R13).

## 3. Information architecture (common to every option)

The mode is **Operate**: the layout is standard and only the visual world varies.
- **A left rail with four destinations** (a large-target rail, R7):
  1. **Today:** the check-in, then one insight with its counterfactual (§4.10).
  2. **Course:** each indicator's history against the personal range and anchor, plus re-anchor and revert.
  3. **Sharing:** the contract tiers, the live sharing log (I6), routes, exclusions and per-indicator consent.
  4. **Support:** the screener, with each item's auto-share state shown *before* answering, the private safety plan, and the PPC and crewmate requests.
- **A persistent top line:** mission day, link state (ground reachable or LOS), and the mode. "Synthetic replay" is shown in the top line whenever the demo is running.
- **The demo replay** is the same app over a synthetic log, with a day scrubber. It is never mixed with real data.

## 4. Visual directions

Every direction must meet R1–R19. A direction lends only **palette, type, density and one signature move** (Operate rules).

### A. Survey sheet: the person's range drawn as terrain
- **World:** topographic survey sheets in night mode. Ground: deep pine-ink. Contour lines: ochre. The **personal range** is a moss-green corridor.
- **Signature move:** each indicator is drawn as its own corridor.
  - The pre-flight anchor is a dashed "benchmark" line.
  - A re-anchor is a marked hinge where the corridor shifts. A revert shows the hinge undone.
  - This makes ANCHOR's unique mechanism (anchored baseline plus consented re-anchoring, unique 0/243) *visible* as the interface's form.
- **Grounding:** R12 and R13 (own-norm feedback after rating), R16 (the corridor width *is* the confidence), R14 (no score: a position in a range).
- **Colour fit with R1:** green, ochre and pine are none of red, yellow or blue. Caution amber and warning red stay distinct. Evening rendering is natural (dark, warm).
- **Risk:** cartographic ornament can turn toy-like. Contours are used only where they draw data.

### B. Mission-day strip: everything on one time axis
- **World:** crew timeline traditions. Graphite ground, muted bands.
- **Signature move:** a horizontal strip of mission days at the bottom of every screen. Each day's column holds check-in, tags, episodes and shares. Today is the expanded last column. The sharing log lives *on the same axis*, so "what left the device, when" is visible in time (strong for R19 and I6).
- **Risk:** it is drawn from a tool the audience operates (timeline software, Playbook), so it can read as a costume of a work tool. It feels like a duty roster, which works against "a companion you want to use" (the user's first requirement). Kept as a strong alternate. Its strip could also be adopted *as a component* by A.

### C. Earthlight: the screen as part of the lighting schedule
- **World:** night-side Earth seen from the Cupola, in navy-black, sodium-amber and aurora-green.
- **Signature move:** the interface's palette *is* the SSLA schedule. Daytime rendering is crisp. **Pre-sleep rendering slides warm and dim as the crew day ends**, taking R10 from a constraint to the form. A thin terminator line marks the circadian phase.
- **Risk:**
  - "Dark ground plus glowing accent" is the most common AI-generated look.
  - Emotion can lead the task.
  - Aurora-green and sodium-amber sit close to caution amber, so R1 separation needs care.
- Its R10 behaviour is adoptable by any direction.

### D. Category standard (the standing alternative)
- A familiar health-app language: light cards, clear charts, one accent, executed at the level of Apple Health. It is legible and expected.
- **Risk:**
  - It earns no creativity credit among 2,407 teams.
  - Light, blue-white cards conflict with R10 at the end-of-day check-in.
  - Rings and score cards must be avoided (R14) even though the category leans on them.

### Considered and declined
- **Field journal** (Stuster's astronaut journals, paper notebook). It is resonant and fits the private tier. Declined because it renders as cream paper with serif type, the most common AI default, and it frames data as diary, which weakens the "track and notice" job. Kept from it: the journal entry screen may borrow the notebook's quietness.
- **Cockpit, caution-and-warning panel.** This is a costume of a tool the crew operates, and it maps psychological data onto vehicle-fault semantics. That is exactly the stigma the disclosure contract exists to remove.

## 5. Recommendation

**A (Survey sheet)**, with two named raises:
- **From C:** the SSLA-linked evening rendering (R10) as a first-class mode.
- **From B:** the mission-day strip as the shared time axis on the Course and Sharing screens.

**Why:**
- A's signature move is the only one that draws ANCHOR's *unique* mechanism rather than a generic metaphor.
- It satisfies the colour semantics without fighting them.
- It is naturally calm and dark for the evening check-in.

**Decision (14 Nov 2026):** the user chose **A**, with both raises.

## Sources
- [NASA-STD-3001 Vol 2, Appendix F, Display Standard](https://www.nasa.gov/reference/appendix-f-vol-2/)
- [NASA-STD-3001 Vol 2 Rev F, Appendix D compliance matrix](https://www.nasa.gov/wp-content/uploads/2025/07/nasa-std-3001-vol-2-app-d-requirements-compliance-matrix.pdf)
- [OCHMO-TB-052 Crew Controls](https://www.nasa.gov/wp-content/uploads/2026/10/ochmo-tb-052-crew-controls.pdf)
- [CLD HSI Workshop: Crew Interfaces (2024)](https://ntrs.nasa.gov/api/citations/20240003566/downloads/CLD%20HSI%20Workshop%20Crew%20Interfaces.pdf)
- [Orion human factors (NTRS 20110004034)](https://ntrs.nasa.gov/api/citations/20110004034/downloads/20110004034.pdf)
- [STS-70 EDOMP human factors assessment (NTRS 20040201530)](https://ntrs.nasa.gov/api/citations/20040201530/downloads/20040201530.pdf)
- [Analysing Human Interaction with Electronic Displays in Microgravity (arXiv 2604.17322)](https://arxiv.org/pdf/2604.17322)
- [Target Size Guidelines for Interactive Displays on the Flight Deck](https://www.researchgate.net/publication/283056109_Target_Size_Guidelines_for_Interactive_Displays_on_the_Flight_Deck)
- [Touchscreen operations under vibration](https://www.researchgate.net/publication/364952304_Assessing_touchscreen_operations_under_vibration_effects_of_vibrational_environments_button_size_and_button_spacing_on_digit-_and_letter-entry_task_performance)
- [Crew Dragon touchscreen, astronaut accounts (TechCrunch)](https://techcrunch.com/2020/05/04/this-is-certainly-different-astronauts-on-controlling-the-dragon-spacecraft-via-touchscreen/)
- [Crew Dragon touchscreen (Space.com)](https://www.space.com/spacex-crew-dragon-touchscreen-astronaut-thoughts.html)
- [Designing Beyond Current Conceptualizations of Spaceflight Experiences (arXiv 2408.00085)](https://arxiv.org/pdf/2408.00085)
- [Psychomotor Vigilance Self Test on ISS (NTRS 20090029851)](https://ntrs.nasa.gov/api/citations/20090029851/downloads/20090029851.pdf)
- [Pulsar RST case study](https://pulsarinformatics.com/sites/default/files/resources/downloads/Pulsar_CaseStudy_02.pdf)
- [Virtual Space Station, HRR task](https://humanresearchroadmap.nasa.gov/tasks/task.aspx?i=1176)
- [VSS evaluation at HI-SEAS (PubMed 27779949)](https://pubmed.ncbi.nlm.nih.gov/27779949/)
- [Spontaneous self-scheduling, HERA C6](https://ntrs.nasa.gov/api/citations/20250001447/downloads/IEEE%20Aero.pdf)
- [Self-scheduling HERA C7 (Karasinski & Marquez 2025)](https://doi.org/10.1177/10711813251370737)
- [Multidimensional Usability Assessment in Spaceflight Analogs (SpaceCHI 2025)](https://ntrs.nasa.gov/citations/20250006351)
- [AI for human exploration, ISS autonomy demos (arXiv 1910.03014)](https://arxiv.org/pdf/1910.03014)
- [ISS Solid-State Lighting (NTRS 20160005080)](https://ntrs.nasa.gov/api/citations/20160005080/downloads/20160005080.pdf)
- [Dynamic lighting in a 45-day analog (PMC11316501)](https://pmc.ncbi.nlm.nih.gov/articles/PMC11316501/)
- [The Design of EMA Technologies (Interacting with Computers)](https://academic.oup.com/iwc/article/32/3/257/5897245)
- [EMA best practices factorial experiment (JMIR 2024)](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11347889/)
- [EMA burden and adherence (PMC11079761)](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11079761/)
- [Prevalence of orthosomnia (PMC11592250)](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11592250/)
- [Explore wellbeing APIs in HealthKit (WWDC24)](https://developer.apple.com/videos/play/wwdc2024/10109/)
