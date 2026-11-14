# ANCHOR and NASA's Human Research Program risks

This file records which NASA resources ANCHOR uses and how. It is the project's answer to the submission form's "NASA Data" field.

The challenge's Resources tab lists three NASA resources: HRP *5 Hazards of Human Spaceflight*, *Risks of Human Spaceflight*, and *Ongoing HRP Research*. Every ANCHOR indicator and feature traces to one of the HRP risks below. Rows H6–H8 were added with the morning vitals; each indicator also carries an "About this measure" card in the app (how it is measured, why it matters in space, what helps). Each row names the HRP page, what that page says (paraphrased), and the ANCHOR mechanism it grounds.

## Mapping

| HRP risk (formal title on the NASA page) | What the NASA page says (paraphrase) | ANCHOR mechanism grounded in it |
|---|---|---|
| **Risk of Behavioral Changes and Psychiatric Disorders** ("adverse cognitive or behavioral changes and psychiatric disorders leading to in-mission health and performance and long-term health effects") [H1] | Isolation and confinement raise behavioural-health risk and can affect sleep, morale and decisions. Long Mars communication delays weaken the ground support available on the ISS. NASA encourages self-care such as a regular journal. Privacy is called a top priority for psychological well-being. | **Indicators.** Mood, energy and calm: three bipolar sliders in a daily check-in of 90 s or less (UI_RESEARCH R11, R15).<br>**Detection.** A personal baseline anchored before flight. Sustained change is detected by a CUSUM statistic under a false-alert budget of at most one note per 30 days (ADR-0005, ADR-0006), with no diagnosis and no alarm vocabulary (I3).<br>**Privacy.** The disclosure contract: private by default, consent per category, a visible sharing log, exclusions without exception (ADR-0007, ADR-0008).<br>**Offline.** Everything runs on board (N1), because ground support is delayed. |
| **Risk from Inadequate Sleep and Irregular Schedules** ("performance decrements and adverse health outcomes resulting from sleep loss, circadian desynchronization, and work overload") [H2] | Work overload, irregular light-dark cycles and disrupted sleep patterns are named stressors. The ISS uses LED lighting that simulates Earth's light-dark cycle to help sleep. | **Indicator.** `sleep_min`, imported from a wearable.<br>**Workload.** Workload-tagged days are excluded from learning and from missed-check-in counting (ADR-0006), so overload is not read as a personal decline.<br>**Evening rendering.** The interface follows the ISS pre-sleep lighting: no token bluer than 2700 K, tested in `app/test/tokens.test.ts` (ADR-0009 §9). |
| **Risk of Hypoxia** [H3] | Deep-space vehicles will keep the cabin partially depressurised to cut pre-spacewalk oxygen needs, which raises the risk of mild hypoxia. NASA is building new health-monitoring protocols to track how crews adapt. | **Indicator.** `spo2`.<br>**Safety line.** A pre-agreed physiological line (SpO2 below 90.0 %) that always shares, is checked by two independent safety kernels with TMR (ADR-0003), and is re-routed to the CMO on board during loss of signal (ADR-0008). The demo mission includes such a crossing on day 120. |
| **Risk of Inadequate Teamwork** ("in-mission performance and behavioral health decrements due to inadequate cooperation, coordination, communication, and psychosocial adaptation within a team") [H4] | Mars missions add communication delays, more isolation and smaller habitats, and crews must work through conflicts themselves. | **Crew route.** A one-tap request to talk with a trusted crewmate (`crew_request`).<br>**Exclusions.** The astronaut may exclude any recipient, including the CMO when the CMO is the person concerned (I10).<br>**No crew-level mood view.** It was rejected because with 4–6 crew any aggregate re-identifies individuals (DESIGN_PAPER §4.9). |
| **Risk of Inadequate Systems Operations Support** ("adverse mission outcomes due to Earth independent human-systems operation") [H5] | Crews far from Earth must run systems without real-time support from mission control, so systems must be easy to operate independently. | **Explainable notes.** Every note answers "Why this note?": the data, the rule, the confidence, what would close it, and the NASA research area.<br>**Determinism.** Deterministic templates rather than generated text (ADR-0009 §8).<br>**Reproducibility.** Replayable, hash-chained logs (ADR-0002).<br>**Interface standards.** NASA-STD-3001 Appendix F display rules: reserved colours, contrast of at least 6:1, large touch targets for microgravity (UI_RESEARCH R1–R9). |
| **Risk of Cardiovascular Adaptations** ("cardiovascular adaptations contributing to adverse mission performance and health outcomes") [H6] | Spaceflight changes the heart and vessels. Contributing factors listed include length of exposure, reduced physical activity, impaired sleep, psychological stress and immune dysregulation, which ties this risk to the behavioural and sleep risks above. | **Indicators.** Morning `pulse_bpm` and `sbp_mmhg` from the on-board cuff, entered with large steppers.<br>**Safety line.** Systolic pressure above 160 mmHg shares to the flight surgeon. Below the line, a sustained rise is a personal note, never a single-reading alert (ADR-0011). |
| **Risk of Altered Immune System Responses** (Roadmap: "in-mission impacts, adverse health events or long-term health impacts due to altered immune response") [H7] | Spaceflight alters immune response; the risk is under active HRP research and is linked to the food and cardiovascular risks. | **Indicator.** Morning `temp_c`.<br>**Safety line.** Above 38.0 °C shares to the flight surgeon on the first reading. The demo mission shows a one-morning fever on day 80 that reaches the surgeon and, by design, opens no "several days" note (ADR-0011). |
| **Risk of Inadequate Food and Nutrition** ("performance decrement and crew illness due to inadequate food and nutrition") [H8] | Inadequate energy and nutrient intake can dysregulate other systems, including immune and cardiovascular function. | **Indicator.** Fasting `glucose_mgdl`, every third morning (a finger-stick, so not daily).<br>**Safety line.** Below 70 mg/dL shares to the flight surgeon. A sustained fall is a personal note. |

## Other NASA sources used in the design

- **Space radiation hazard ([5 Hazards](https://www.nasa.gov/hrp/hazards/)) and NASA NTRS 20060015096 (radiation effects on commercial hardware).** Commercial tablets in orbit suffer single-event upsets. This is why the safety kernel keeps three copies of the safety table and votes (TMR), and why kernel B is an independent implementation (ADR-0003; DESIGN_PAPER §5.2).
- **NASA-STD-3001 Vol. 2, Appendix F (display design).** Colour meanings, contrast, character height, touch targets. See docs/UI_RESEARCH.md, sources S1–S2.
- **ISS Solid-State Lighting Assemblies (SSLA) schedule.** The basis of the evening rendering. See UI_RESEARCH S12.
- **NASA HRP behavioural-health research.** Standardised measures, the Reaction Self Test, and journals (Stuster), as surveyed in DESIGN_PAPER.md §2.

## Not used, and why

- **NASA astronaut medical data (LSAH, Twins Study).** It is not public, and ANCHOR claims no astronaut data. Validation instead uses a synthetic mission (labelled as such) and a public, consented, real-world wearable and mood dataset, LifeSnaps (see docs/VALIDATION.md).
- **CSA radiation monitoring** (a partner resource on the challenge page). Radiation dose is a crew-level environmental measure that the dosimetry systems already track. Personal radiation tracking is out of scope (stated in DESIGN_PAPER.md, scope table).

## Wording in the astronaut-facing interface

The interface names these as "NASA research areas" (behavioural health, sleep and schedules, hypoxia, teamwork, independent operations, heart and circulation, immune health, nutrition). It does not use the formal HRP titles. The formal titles contain diagnostic and alarm vocabulary that the language audit bans from astronaut-facing copy (I3), because it frightens without informing.

## Sources

- [H1] NASA HRP, "Risk of Behavioral Changes and Psychiatric Disorders": https://www.nasa.gov/reference/risk-of-behavioral-changes-and-psychiatric-disorders/
- [H2] NASA HRP, "Risk from Inadequate Sleep and Irregular Schedules": https://www.nasa.gov/reference/risk-from-inadequate-sleep-and-irregular-schedules/
- [H3] NASA HRP, "Risk of Hypoxia": https://www.nasa.gov/reference/risk-of-hypoxia/
- [H4] NASA HRP, "Risk of Inadequate Teamwork": https://www.nasa.gov/reference/risk-of-inadequate-teamwork/
- [H5] NASA HRP, "Risk of Inadequate Systems Operations Support": https://www.nasa.gov/reference/risk-of-inadequate-systems-operations-support/
- [H6] NASA HRP, "Risk of Cardiovascular Adaptations": https://www.nasa.gov/reference/risk-of-cardiovascular-adaptations/ (Human Research Roadmap record: https://humanresearchroadmap.nasa.gov/Risks/risk.aspx?i=179)
- [H7] NASA HRP, "Risk of Altered Immune System Responses": https://www.nasa.gov/reference/risk-of-altered-immune-system-responses/ (Human Research Roadmap record: https://humanresearchroadmap.nasa.gov/Risks/risk.aspx?i=85)
- [H8] NASA HRP, "Risk of Inadequate Food and Nutrition": https://www.nasa.gov/reference/risk-of-inadequate-food-and-nutrition/ (risk record Rev D, 2025: https://www.nasa.gov/wp-content/uploads/2025/08/food-risk-record-rev-d-web-final.pdf)
- NASA HRP, "Risks of Human Spaceflight" (index): https://www.nasa.gov/hrp/risks/
- NASA HRP, "5 Hazards of Human Spaceflight": https://www.nasa.gov/hrp/hazards/

All pages were read on 14 Nov 2026.
