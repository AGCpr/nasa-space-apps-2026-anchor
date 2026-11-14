# Astronaut Daily Companion: Architecture and Working Document

> **Superseded for design:** `DESIGN_PAPER.md` is the authoritative design. This file is kept for the evidence review (§1–2), roles and timeline (§7).

**Challenge:** NASA Space Apps 2026, *Create Health Monitoring Software for Astronauts on Space Missions*
**Submission:** 15 Nov 2026, 23:59 local time · **Document date:** 10 Nov 2026 · **Status:** draft v1

---

## 0. Positioning: why this project, and why it is different

### 0.1 What already exists in the field

| System | What it does | What is missing |
|---|---|---|
| Reaction Self-Test (RST/PVT-B), on the ISS since 2009 | Each morning: sleep, stress and workload self-report, then a 3-minute vigilance test | A data-collection tool. It gives the astronaut nothing back |
| WinSCAT, Cognition Battery | Compares cognitive performance against a pre-flight personal baseline | Monthly or infrequent. A research instrument |
| Astroskin / Bio-Monitor (CSA) | Continuous physiological measurement (ECG, SpO₂, respiration, temperature) | Sensor layer only. No interpretation or support |
| CIMON-2 (DLR/IBM/Airbus) | AI assistant with emotion recognition | Results "mixed". Empathy fell short; criticised as a "floating Alexa" |
| NASA HRR: *Individualized Behavioral Health Monitoring Tool*, *Virtual Space Station* | Autonomous, personalised behavioural health tool | **Still on the roadmap. NASA has not finished it** |

### 0.2 Competing teams this year (GitHub, as of 10 Nov)

| Project | Strengths | Gap |
|---|---|---|
| **AURORA** (strongest competitor) | Personal baseline, Isolation Forest, PVT-B, radiation data (OSDR/NOAA), encrypted local vault, per-category sharing, assistant that refuses to diagnose | **No psychological support.** PVT-B measures alertness, not mood |
| COSMOCare, ASTROCARE, Project-Astra, AstroHealth | Anomaly detection, dashboard, chatbot | Classic "vitals dashboard + alerts". Personalisation and language design are shallow |

**Conclusion:** Physiological monitoring, personal baselines and privacy are now table stakes. The real gap, and our differentiator:

> **A psychological companion the astronaut opens every day by choice, that gets to know them over time and supports them without fear.** Physiological monitoring is a sub-layer of this companion, not the product itself.

---

## 1. Testing the proposals: evidence, counter-thesis, decision

For each proposal: supporting research → strongest counter-thesis → evaluation → resulting design decision.

### P1. Used daily, must not feel like a chore

- **Evidence:** Self-Determination Theory meta-analyses show that supporting autonomy, competence and relatedness increases intrinsic motivation (Gillison 2019: autonomy support g=0.84). Most reasons people abandon mHealth apps relate to these three needs. None of 194 depression apps targeted all three.
- **Counter-thesis 1, law of attrition:** Median 30-day retention in mental health apps is 3.3% (Baumel 2019); Eysenbach's "law of attrition" describes the same pattern. "Opened every day" is optimistic by the literature.
- **Counter-thesis 2, astronaut time:** The ISS day is scheduled to the minute. Every extra minute is taken from another task.
- **Evaluation:** An astronaut is not an ordinary app user; they already do the RST routinely. The real risk is not abandonment but turning the app into **mechanical box-ticking**, which destroys data quality.
- **Decision:**
  - Morning check-in at most 90 seconds, attached to an existing routine rather than creating a new one (*habit stacking*).
  - **Every entry gets an immediate return** (an insight or a short suggestion). No data entry without a return.
  - **No streaks, points or badges.** Rationale in P5.
  - Tracker-type apps retain significantly better than breathing apps (Baumel). Our value is tied to measurement, which works in our favour.

### P2. Must provide psychological support

- **Evidence:** In Stuster's Journals study, all participants said the journal was an outlet for frustrations that, if expressed directly, could damage relationships. Contact with family substantially lifts morale. There is also a known **third-quarter dip**: positive entries drop in the third quarter of the mission (also seen in Mars-500).
- **Counter-thesis 1, AI cannot do empathy:** CIMON-2's results were mixed. Artificial empathy can feel fake or hollow.
- **Counter-thesis 2, replacing professional therapy:** The app may make a real problem feel "handled" and delay going to the ground psychologist.
- **Evaluation:** CIMON's mistake was trying to **imitate emotion**. We do not imitate emotion; we provide **structure**: journaling, reflection, evidence-based small interventions, connection to Earth.
- **Decision:**
  - The app never says "I understand you". It presents the observation and the options.
  - **Mission-phase aware:** support content increases proactively as the third quarter approaches.
  - **Novelty injection against monotony:** in Mars-500, new tasks such as the landing phase raised morale. The app suggests small novelties: a new Earth-observation target, a mini project.
  - From every support screen, the path to the ground psychologist is **one tap and non-judgemental**.

### P3. Must track values and warn about risks

- **Evidence:** The RST was designed as an early warning for fatigue countermeasures when alertness drops. The RIDGE framework ties each risk to a hazard class.
- **Counter-thesis, alarm fatigue and false alarms:** AURORA's first version produced false alarms on healthy crew; they ended up with a "2 of 4 indicators must deviate" rule. False alarms kill both trust and usage.
- **Decision:**
  - Alerts are based on **sustained deviation, not single readings** (EWMA/CUSUM or an "M of N days" rule).
  - **False alert rate on healthy synthetic crew** is reported as an evaluation metric (see §5).
  - Every alert comes with a "why?" explanation and its source.

### P4. Must not make a definitive diagnosis

- **Evidence and regulation:** Software that diagnoses falls into the medical-device category. Not diagnosing is both ethically and legally correct. AURORA explicitly does the same.
- **Counter-thesis, uncertainty also harms:** "Something is off but I can't say what" can raise anxiety more than clear information. On Mars the ground team is 20+ minutes away; without a diagnosis the astronaut may not know what to do.
- **Evaluation:** Diagnosis and **action** are different things. The app does not diagnose, but always says clearly **what to do**.
- **Decision:** Every output has a fixed structure: **observation → possible context (more than one) → concrete next step → who should be involved**. No disease names, but action is never vague.

### P5. No harsh language, no climate of fear

- **Evidence 1, nocebo:** How risk information is framed changes how bodily sensations are interpreted and leads benign symptoms to be attributed to the "problem". Positive framing ("7 in 10 people won't experience nausea that bothers them") neutralises the warning effect.
- **Evidence 2, orthosomnia:** 30.9% of sleep trackers screen positive for anxiety about their tracking data (Goel, *Sleep* 2026). As anxiety rises, sleep duration falls more steeply in wearable users (JMIR 2025). **Measuring can damage what it measures.**
- **Evidence 3, streaks/gamification:** Broken streaks produce guilt and the "what-the-hell" effect (abstinence violation). A qualitative Duolingo study reports anxiety and self-recrimination.
- **Counter-thesis, overly soft language trivialises real risk:** Calm language may lead the astronaut to underrate a real problem. Astronauts are professionals who expect technical, precise information; over-softened language can feel **childish or manipulative** to them.
- **Evaluation:** The counter-thesis is strong. The answer is not "soft" language but **calm, professional** language: short, factual and panic-free, like pilot–tower communication.
- **Decision:**
  - Language rules: fact + number + context. No exclamation marks, no "danger/critical/urgent". Personalised reference ("your normal").
  - **Two channels:** everyday insights in a calm tone. When pre-defined **safety lines** are crossed, the language becomes direct and brief but still panic-free. This mirrors the aviation "caution / warning" distinction.
  - **Orthosomnia guard:** nightly score hidden by default, weekly trend shown instead. Option to hide numbers.
  - No guilt messages for missed days. The app simply says "welcome back".

### P6. Must get to know the person and feel increasingly personal

- **Evidence:** WinSCAT uses a pre-flight personal baseline. Oura and WHOOP use 14-day and long-term personal baselines. In Mars-500, 55% of self-report variance came from stable between-person differences, so population thresholds are meaningless. The JITAI framework (Nahum-Shani 2018) defines "the right support, at the right time, only when needed" in 6 components.
- **Counter-thesis 1, shifting-baseline trap:** A self-updating baseline learns a slow decline as the "new normal". Mood falling step by step across a mission never triggers an alert.
- **Counter-thesis 2, transitions:** Oura's baseline struggles with short transitions such as illness or travel (arXiv 2503.19132).
- **Counter-thesis 3, self-report bias:** The Mars-500 crew followed unwritten display rules favouring positive emotion. Self-report can look better than reality.
- **Decision:**
  - **Dual baseline:** a short window (14 days) plus a **fixed pre-flight anchor**. Drift from the anchor is tracked separately even when the short window looks normal. This closes the shifting-baseline trap.
  - **Context tags:** EVA day, sleep shift, solar particle event, illness. The baseline excludes or separately handles these days.
  - **Self-report + objective triangulation:** if reported mood contradicts PVT and sleep data, the app does not judge; it asks gently.
  - **Visible personalisation:** the app states its confidence by days of data ("I've known you for 6 days") and shows the patterns it has learned.
  - **Interaction budget:** at most 1 proactive suggestion per day. Which suggestions work for the person is recorded and future suggestions are personalised accordingly (JITAI receptivity).

### P7. Private-by-default data model (option A)

- **Evidence:** The IOM *Safe Passage* report (2001) has a section titled "Institutional Pressure to Underreport". Astronauts fear that the smallest deviation will jeopardise future flights. In Stuster's journals, astronauts wrote that they did not tell the flight surgeon about every problem. 42.5% of military pilots report withholding health information for fear of losing flight status (Mil Med 2023).
- **Counter-thesis, operational safety:** Crew health is mission safety. A hidden problem can put the whole crew at risk, and the flight surgeon has obligations.
- **Evaluation:** Both sides are right. This tension should not be hidden; it belongs **at the centre of the design**.
- **Decision:**
  - The raw journal and free text are **never shared automatically**.
  - Numeric indicators stay on device. The astronaut enables sharing **per category**.
  - **Safety lines** are a short list signed jointly by astronaut and flight surgeon before flight. If crossed, they are reported automatically and the astronaut **knows this in advance**. No surprises.
  - What was shared, when, and with whom is always visible (sharing log).

---

## 2. Additional proposals derived from the research

| # | Proposal | Basis |
|---|---|---|
| A1 | **Mission-phase engine:** anticipate known hard periods (third quarter, mission end, communication blackouts) and increase support in advance | Stuster, Mars-500 |
| A2 | **Communication-delay-aware autonomy:** depth of self-management guidance changes with ground reachability for ISS, Moon and Mars | HRP "Distance from Earth"; AURORA also stresses this |
| A3 | **Private journal + optional on-device affect analysis:** text never leaves the device; the user sees only their own trend | Stuster; HI-SEAS text-analysis software |
| A4 | **Earth connection module:** family notes, the moment the orbit passes over home, Earth-observation targets | Stuster: family contact lifts morale |
| A5 | **Evidence-based micro-intervention library:** sleep hygiene (CBT-I components), light routine, breathing, short exercise. Each with a source | HRP countermeasures; JITAI intervention options |
| A6 | **Data-quality indicator:** with missing or little data, insight confidence drops and this is stated explicitly | Oura transition problem, P6 |
| A7 | **Language audit test:** all generated text passes a banned-word and tone check, implemented as an automated test | Nocebo literature, P5 |
| A8 | **Body figure** (the challenge's suggestion): clicking a body region shows its RIDGE effects and the person's own data | Challenge text |

---

## 3. System architecture

```
┌──────────────────────────────────────────────────────────────────┐
│ 7. UI   Today card · Body figure · Weekly trend · Journal ·      │
│         Earth connection · Sharing centre                        │
├──────────────────────────────────────────────────────────────────┤
│ 6. COMMUNICATION LAYER  Language templates · tone rules ·        │
│    language audit (A7) · "observation → context → step → who"    │
├───────────────────────────────┬──────────────────────────────────┤
│ 5. SUPPORT ENGINE             │ 4. ESCALATION                    │
│  JITAI decision rules         │  Level 0: info only              │
│  Micro-intervention library   │  Level 1: self-manage            │
│  Mission-phase engine (A1)    │  Level 2: crew / CMO             │
│  Interaction budget (1/day)   │  Level 3: ground (delay A2)      │
│  Learns what works            │  Safety lines → automatic        │
├───────────────────────────────┴──────────────────────────────────┤
│ 3. INTERPRETATION  RIDGE mapping · sourced thresholds ·          │
│    sustained deviation (EWMA/CUSUM, M-of-N) · multi-indicator    │
├──────────────────────────────────────────────────────────────────┤
│ 2. PERSONAL MODEL  Dual baseline (14d rolling + pre-flight       │
│    anchor) · robust z (median/MAD) · context tags · confidence · │
│    self-report ↔ objective triangulation                         │
├──────────────────────────────────────────────────────────────────┤
│ 1. INPUT  Check-in (sleep, mood VAS, stress, workload) ·         │
│    PVT (3 min) · vitals (manual / simulated Astroskin) ·         │
│    environment (radiation OSDR/NOAA, CO₂) · journal text         │
├──────────────────────────────────────────────────────────────────┤
│ 0. PRIVACY CORE  Encrypted on-device store · category permissions│
│    · sharing log · safety-line agreement                         │
└──────────────────────────────────────────────────────────────────┘
```

### 3.1 Daily flow

1. **Morning, ≤ 90 s:** sleep hours (confirm only if auto-filled), 3 sliders for mood, stress and energy → **one immediate insight**.
2. **Optional:** 3-minute PVT. Suggested 2–3 times a week, not mandatory daily.
3. **During the day:** one suggestion, only if the interaction budget allows and a JITAI rule fires.
4. **Evening, optional:** one-line journal or free writing.
5. **Weekly:** trend summary and learned patterns. The astronaut decides whether to share it.

### 3.2 Output template (every message follows this structure)

> **Observation:** Over the last 4 nights your sleep was on average 48 min shorter than your 14-day normal.
> **Context:** This week had 2 EVA preparations. This is common in busy periods.
> **Step:** Dimming screen light 30 min earlier tonight is one option.
> **Who:** This is something you can manage yourself right now. If it continues for 3 more nights, I suggest talking to the CMO.

---

## 4. Technology choice (decision pending, see §8)

| Option | Pros | Cons |
|---|---|---|
| **A) Single-page browser app** (React/Vite or plain TS) + Python for synthetic data | Works offline, fits the privacy story, easy demo | Requires front-end skill |
| **B) Streamlit** (all Python) | Fastest prototype, strong on data | Weak "professional product" look, weak offline story |

Recommendation: A. But the decision depends on team skills.

---

## 5. Validation: the project's scientific evidence

Demo data: **180-day synthetic ISS mission**, 4 personas, with known events injected:
- Third-quarter mood dip
- Sleep restriction period
- Solar particle event (real NOAA dates)
- Slow decline (tests the shifting-baseline trap)
- Fully healthy control persona

| Metric | Target |
|---|---|
| False alerts per month on healthy persona | ≤ 1 |
| Detection latency for injected events | Reported (days) |
| Slow decline: with vs without dual baseline | Show it is missed without and caught with (**main experiment**) |
| Language audit: banned words and tone violations | 0 |
| Check-in duration | ≤ 90 s |

**Limitation:** Synthetic data, not validated with real astronauts, not a medical device. Stated explicitly in the presentation.

---

## 6. Risks and limits

| Risk | Mitigation |
|---|---|
| Looking similar to AURORA and other teams | Narrative centres on the psychological companion, not physiology. The dual-baseline experiment is put forward as evidence |
| Scope explosion in 48 hours | "Must have / nice to have" split in §7 |
| Medical claims | "Decision-support prototype" stated on every screen |
| Pre-hackathon preparation rule | **Verify:** check the official rules on how much code may be prepared before the hackathon |

---

## 7. Working dynamic

### 7.1 Roles (team of up to 6)

| Role | Responsibility |
|---|---|
| **Product and research lead** | This document, psychology evidence, language rules, final presentation narrative |
| **Data and model** | Synthetic mission generator, personal model (§3 layers 2–3), validation experiment (§5) |
| **Front end** | UI layer, body figure, charts |
| **Content and support** | Micro-intervention library, message templates, source mapping |
| **Demo and submission** | Video, project page, submission checklist |

With fewer people: Product and Content merge, and the Product lead takes over Demo.

### 7.2 Timeline

| When | Work | Output |
|---|---|---|
| **10–11 Nov** | Finalise document v1, close open decisions, form team | This document v2, team |
| **12–13 Nov** | Language rules, message templates, micro-intervention list, synthetic data design, screen sketches *(code preparation subject to rules)* | Templates and sketches |
| **14 Nov 09:00 → 15:00** | Synthetic data, personal-model core, UI skeleton | First end-to-end flow |
| **14 Nov 15:00 → 24:00** | Escalation, support engine, body figure, privacy centre | Features complete |
| **15 Nov 00:00 → 12:00** | Validation experiment, language audit test, bug fixing | Results table |
| **15 Nov 12:00 → 20:00** | Demo video, project page, README | Submission package |
| **15 Nov 20:00 → 23:59** | **Buffer.** Submit by 22:00 at the latest | Submission |

### 7.3 Scope priority

- **Must have:** check-in → personal model (dual baseline) → message template → escalation → privacy centre → validation table
- **Nice to have:** body figure, PVT, Earth connection, mission-phase engine
- **Out of scope:** real sensor integration, LLM chatbot, crew-level analytics, mobile app

---

## 8. Open decisions (questioning order)

1. The user's skills and which roles are missing from the team
2. Technology choice (A or B)
3. Product name and one-sentence pitch
4. Contents of the safety-line list (which indicators, which thresholds)
5. Verification of the preparation rule (official rules)
6. Verification of the submission format (video length, slide count)

---

## Sources

- RST/PVT-B: [Sleep 2022](https://academic.oup.com/sleep/article/45/3/zsac006/6505235) · [NASA Spinoff 2013](https://spinoff.nasa.gov/Spinoff2013/hm_1.html)
- [NASA OCHMO-TB-016 Behavioral Health](https://www.nasa.gov/wp-content/uploads/2023/12/ochmo-tb-016-behavioral-health.pdf) · [HRR Virtual Space Station](https://humanresearchroadmap.nasa.gov/Tasks/task.aspx?i=1176)
- [UPenn Cognition](https://www.med.upenn.edu/uep/cognition.html) · [Frontiers Physiol 2024 normative data](https://www.frontiersin.org/journals/physiology/articles/10.3389/fphys.2024.1451269/full) · [NASA/TM-20210021987](https://ntrs.nasa.gov/api/citations/20210021987/downloads/NASA_TM20210021987.pdf)
- [Astroskin](https://hexoskin.com/pages/astroskin-vital-signs-monitoring-platform-for-advanced-research) · [CSA Bio-Monitor](https://www.canada.ca/en/space-agency/news/2016/10/new-advances-innovative-health-technology-tested-space.html)
- [NASA Five Hazards (RIDGE)](https://www.nasa.gov/stem-content/five-hazards-of-human-spaceflight/)
- Stuster Journals: [NASA](https://www.nasa.gov/mission_pages/station/research/news/Stuster_Journals.html) · [HRP Evidence Report BMed](https://humanresearchroadmap.nasa.gov/evidence/reports/BMed.pdf) · [PopSci](https://www.popsci.com/astronaut-diaries-mission-Mars/)
- Mars-500: [Basner, PMC3968121](https://pmc.ncbi.nlm.nih.gov/articles/PMC3968121/) · [Wang, PLoS ONE 2014](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0087087)
- Underreporting: [IOM Safe Passage, ch. 6](https://www.nationalacademies.org/read/10218/chapter/8) · [Mil Med 2023 pilots](https://academic.oup.com/milmed/article/188/3-4/e446/6761438) · [CSMonitor 2007](https://www.csmonitor.com/2007/0730/p03s01-ussc.html)
- SDT: [Gillison 2019 meta-analysis](https://www.tandfonline.com/doi/full/10.1080/17437199.2020.1718529) · [mHealth engagement overview](https://pmc.ncbi.nlm.nih.gov/articles/PMC13398183/) · [SDT in behaviour change technologies](https://arxiv.org/pdf/2402.00121)
- Attrition: [Eysenbach 2005](https://www.jmir.org/2005/1/e11/PDF) · [Baumel 2019](https://www.jmir.org/2019/9/e14567/PDF)
- JITAI: [Nahum-Shani 2018](https://www.researchgate.net/publication/308535769_Just-in-Time_Adaptive_Interventions_JITAIs_in_Mobile_Health_Key_Components_and_Design_Principles_for_Ongoing_Health_Behavior_Support) · [JITAI current review](https://pmc.ncbi.nlm.nih.gov/articles/PMC13217361/)
- Orthosomnia: [PMC11592250](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11592250/) · [Sleep 2026 abstract](https://academic.oup.com/sleep/article/49/Supplement_1/A246/8674102) · [JMIR 2025](https://www.jmir.org/2025/1/e68816/XML)
- Gamification harms: [Hadi Mogavi 2022](https://arxiv.org/pdf/2203.16175)
- Nocebo and framing: [Scoping review 2026](https://pmc.ncbi.nlm.nih.gov/articles/PMC13205298/) · [Cybersickness, Ann Behav Med 2021](https://pmc.ncbi.nlm.nih.gov/articles/PMC8517218/)
- Personal baselines: [Readiness comparison](https://sahha.ai/blog/how-readiness-scores-are-calculated/) · [arXiv 2503.19132](https://arxiv.org/pdf/2503.19132)
- CIMON: [MIT Tech Review](https://www.technologyreview.com/2020/01/14/64990/an-emotionally-intelligent-ai-could-support-astronauts-on-a-trip-to-mars/) · [Space.com](https://www.space.com/astronauts-artificial-intelligence-companions-deep-space-missions)
- Competitors: [AURORA](https://github.com/fabriciojunio/aurora-astronaut-health) · [COSMOCare](https://github.com/omkargurav0216-code/COSMOCare) · [ASTROCARE](https://github.com/sachuabhijith3485-cpu/ASTROCARE) · [Project-Astra](https://github.com/md-istiak/Project-Astra)

> Note: some findings from secondary sources (Castro & Valença 2025, Silverman & Barasch 2023, the n=26 and n=113 sleep trials) were not used because they could not be verified against the primary papers.
