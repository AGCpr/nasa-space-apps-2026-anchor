# Systematic Landscape Analysis — Astronaut Health Monitoring (Space Apps 2026 and prior art)

*10 Nov 2026. Reproducible: `analysis/landscape/code_corpus.py` (corpus, README cache, coding, summary).*

---

## 1. Method

### 1.1 Corpus construction
- **Source:** GitHub repository search via the GitHub API.
  - Restricted to repositories created after 1 Aug 2026 (the 2026 challenge period).
  - ~37 queries over names, descriptions and READMEs, e.g. "astronaut health monitoring", "astronaut mental health", "crew wellbeing", `"Health Monitoring Software for Astronauts" in:readme`, `"Space Apps Challenge 2026" health in:readme`.
- **Candidates:** 502 unique repositories.
- **Inclusion criteria:**
  - The README or description mentions astronaut/crew **and** health/vitals/well-being/mental/medical/physiology.
  - Off-topic challenges are excluded: human-activity recognition for experiments, farming, wildfire, SPHEREx, landing sites, etc.
- **Included:** **243 repositories**, of which **197** have a substantive README (≥ 1,500 characters).

### 1.2 Coding
- Each README is coded for 33 features with fixed, published regular expressions (`FEATURES` in the script).
- Keyword coding is a **reproducible first pass**. It over-counts generic words and misses features described without the keyword.
- Nearest neighbours (by overlap with ANCHOR's design axes) were therefore **read in full**: HELIOS, VITASTRA, SpaceFog, orbita, astra-loop, kairos-monitor, baseline, HALO Crew, AURORA, RAKIA, AstroVitals, AstroHealth.

### 1.3 Beyond GitHub
The following were examined from publications:
- **Flight and analog systems:** RST/PVT-B, WinSCAT, Cognition, Astroskin/Bio-Monitor, CIMON-2, Virtual Space Station / Expedition-APPP, ESA Concordia and MECA, ICE-Q.
- **Research platforms:** mindLAMP, Beiwe, BiAffect, 2B-Alert.
- **Consumer systems:** Oura, WHOOP, Entrain, Sleepio.

### 1.4 Limits of this analysis
- Teams without a public repository, and repositories created before 1 Aug 2026, are not covered. The registry lists 2,407 teams, so the corpus is roughly a 10% sample, biased toward teams that publish early.
- GitHub search rate limits truncated some queries.

---

## 2. Prevalence of features (243 relevant repositories)

| Feature | All | Substantive | Reading |
|---|---|---|---|
| Mood / psychological data collected | 56% | 65% | Common, mostly collection only |
| Radiation | 52% | 62% | Commodity |
| Offline-first | 40% | 49% | Commodity |
| Personal baseline | 40% | 47% | Commodity |
| Sleep | 40% | 46% | Common |
| NASA open data used | 38% | 46% | Common |
| Gamification | 30% | 38% | Common, and contra-indicated by evidence (§3) |
| Ground / Mission Control dashboard | 29% | 34% | Common, and conflicts with disclosure |
| LLM / chatbot | 26% | 31% | Commodity |
| Explainability claims | 25% | 29% | Common |
| Wearables | 21% | 25% | Common |
| Camera / face analysis | 18% | 21% | Common, privacy-invasive |
| Tests present | 20% | 24% | — |
| Non-diagnostic disclaimer | 16% | 20% | — |
| Voice | 14% | 17% | — |
| Privacy / consent / encryption | 14% | 17% | Minority |
| PVT / cognitive tests | 13% | 16% | Minority |
| ML anomaly models | 10% | 12% | Minority |
| False-alarm metrics reported | 10% | 12% | Minority |
| Psychological **intervention** | 9% | 10% | **Rare** |
| Circadian | 8% | 9% | **Rare** |
| Statistical change detection (CUSUM/EWMA/change point) | 5% | 6% | **Rare** |
| Crew interpersonal | 5% | 6% | **Rare** |
| Calm-language / nocebo-aware design | 1% | 1% | **Nearly absent** |

### White-space probes (full-text search over all READMEs)

| Concept | Repositories |
|---|---|
| Self-assessment calibration / metacognition | **0** |
| Decision-theoretic (expected-utility) alerting | **0** |
| Regression-to-the-mean awareness | **0** |
| Randomised N-of-1 self-experiments | **0** |
| Pre-flight disclosure agreement / safety lines | **0** (all 14 keyword hits were "limitations disclosure" sections) |
| Biomathematical fatigue model | 1 (Borbély two-process model in a Monte Carlo "digital twin") |
| Composite readiness/health/risk score | 28 |

---

## 3. Nearest neighbours, read in full

| Project | Strengths | Gaps relative to ANCHOR |
|---|---|---|
| **HELIOS** (strongest in corpus) | Pre-flight baseline adapting with a 30-day half-life using only readings older than 14 days (anti-drift); robust EWMA adjusted for autocorrelation; Mann–Kendall drift with Sen's slope; FA/person-month reported (2.7; 95.2% caught; median delay 12 d on 200 held-out simulated people); real-data alert-load test (PMData); Inspiration4 lab models; FHIR; SBAR consult; blackout mode; data-coverage gating; day planner; glove mode; 6 languages; honest corrections | **Composite readiness score**; "What works for you" uses pre/post checks 2–7 days after a protocol, which is **regression-to-the-mean biased** (our simulation: a useless countermeasure appears to improve the indicator by +1.71 SD); crew health map and CMO view (disclosure incentive); no calibration of self-assessment; no utility-aware alerting; no runtime-assurance kernel |
| **VITASTRA** | CUSUM; personal baselines; three privacy tiers; "psychological data never auto-shared"; circadian check-in | No pre-agreed safety-line contract; mission-control tier present; no calibration loop |
| **astra-loop** | 14-day EWMA baseline plus a second layer that overrides drifted baselines | Badges; no psychological intervention design |
| **kairos-monitor** | Frozen 7-day baseline; explainable readiness | Frozen baseline (adaptation shows up as persistent deviation) |
| **baseline** | Own 30-day range; validated false alarms on noise; real Skylab M133 sleep records | Rolling range (shifting-baseline risk) |
| **HALO Crew** | Three-tier privacy (Private → HALO → Crew-safe); family connection | Voice agents; AI copilot |
| **AURORA / RAKIA** | Security engineering, deterministic core, consent sync | Psychological layer thin |
| **SpaceFog** | Rich NASA evidence; Cognition-battery z-scores with crew task reallocation | Crew-level flagging and task reallocation (disclosure incentive) |

---

## 4. Prior art outside the hackathon

| System | Relevance |
|---|---|
| RST/PVT-B (ISS) | Daily self-report + vigilance; data flow to ground |
| WinSCAT, Cognition | Pre-flight baseline; practice-effect correction |
| Virtual Space Station / Expedition-APPP | Conflict, stress and depression self-help; analog and RCT evaluations |
| ESA Concordia, MECA | Speech and video analysis of mood; the "electronic partner" concept; ICE-Q questionnaire |
| CIMON-2 | Emotion-aware assistant; mixed results |
| 2B-Alert (US Army) | Bayesian individualisation of a sleep/caffeine performance model; accurate after ~12 PVTs |
| mindLAMP / Beiwe | Open digital-phenotyping platforms; relapse anomaly detection (89% sensitivity / 75% specificity, n = 90) |
| BiAffect | Content-free keystroke dynamics predict mood |
| Entrain | Model-based personal light schedules; low adherence |
| Horvitz et al. | Decision-theoretic alerting: expected value of transmitting an alert vs. cost of interruption |

---

## 5. Conclusions for ANCHOR

1. **Commodity in 2026:** personal baselines, offline operation, radiation, LLM chat, gamification, mood logging, composite scores and ground dashboards. None of these earns creativity credit.
2. **Converged with the best entries:** statistical change detection, false-alarm reporting, drift tests, FHIR export, coverage gating and private journals. HELIOS already does these well. ANCHOR must **not** claim them as novel; it should match or exceed them and move on.
3. **Unoccupied, evidence-backed territory** (0 of 243): self-assessment calibration; expected-utility alerting; RTM-robust evaluation via randomised N-of-1; disclosure contract with pre-agreed safety lines; runtime-assurance kernel; nocebo-aware language design.
4. **Contra-indicated but popular:** composite scores, gamification, ground dashboards and camera emotion analysis. ANCHOR's refusal of each is a reasoned position, not an omission.

---

## Sources (beyond those in DESIGN_PAPER)
- HELIOS — [github.com/WillBettleheim/Helios](https://github.com/WillBettleheim/Helios)
- VITASTRA — [github.com/nishatkh/VITASTRA](https://github.com/nishatkh/VITASTRA)
- SpaceFog — [github.com/Moh-lab-sys/SpaceFog](https://github.com/Moh-lab-sys/SpaceFog)
- astra-loop — [github.com/ripro805/astra-loop-health-twin-nasa-space-app-challenge-2026](https://github.com/ripro805/astra-loop-health-twin-nasa-space-app-challenge-2026)
- kairos-monitor — [github.com/xeon797/kairos-monitor](https://github.com/xeon797/kairos-monitor)
- baseline — [github.com/obsidianprimeofficial-alt/baseline](https://github.com/obsidianprimeofficial-alt/baseline)
- HALO Crew — [github.com/fahin99/Halo-Crew](https://github.com/fahin99/Halo-Crew)
- ESA Concordia — [ESA](https://www.esa.int/Science_Exploration/Human_and_Robotic_Exploration/Concordia/Experiments_2012-2013), [ESA blog 2025](https://blogs.esa.int/concordia/2025/12/16/two-decades-of-research-at-concordia/)
- mindLAMP/Beiwe — [Transl Psychiatry 2020](https://www.nature.com/articles/s41398-020-01123-7), [JOSS](https://www.theoj.org/joss-papers/joss.03417/10.21105.joss.03417.pdf)
- 2B-Alert — [J Sleep Res](https://bhsai.org/pubs/Reifman_2019_2B_Alert_App.pdf), [Sleep 2023](https://academic.oup.com/sleep/article/46/7/zsad080/7092866)
- Horvitz, Jacobs & Hovel, Attention-Sensitive Alerting — [arXiv 1301.6707](https://arxiv.org/pdf/1301.6707)
- Van Dongen et al., Sleep 2003 — [PDF](https://www.med.upenn.edu/uep/assets/user-content/documents/VanDongen2003CumulativeCost.pdf)
- N-of-1 / personal science — [arXiv 1901.03423](https://arxiv.org/pdf/1901.03423), [Frontiers 2020](https://www.frontiersin.org/journals/computer-science/articles/10.3389/fcomp.2020.00021/full)
