// Template catalogue (ADR-0009 §8; DESIGN_PAPER §4.10). Every sentence the astronaut reads comes
// from here. Each message names its register, a step and an owner (I4); the audit in
// app/test/audit.test.ts renders every template with every slot value and checks I3/I4.
import type { OutKind, ShareClass } from "./model.ts";

/** NASA-STD-3001 App. F mapping (UI_RESEARCH R1): warning red, caution yellow, advisory blue. */
export type Register = "warning" | "caution" | "advisory" | "info";
export type Msg = { register: Register; title: string; text: string; action: string; owner: string; why?: Why };
/** "Why this note?": the data, the rule, what closes it, and the NASA research area (docs/HRP_MAPPING.md). */
export type Why = { lines: string[]; area?: { name: string; url: string } };

export type Slots = {
  ind: string; // indicator key
  dir: 1 | -1; // harmful direction
  to: string | null; // recipient key
  value: string; // formatted value with unit
  n: number;
  cls: ShareClass;
  item: string; // screener item key
  share: boolean;
  screen: boolean;
};

/** Interface chrome, English. Other locales provide the same keys. */
const UI_EN = {
  today: "Today",
  course: "Course",
  sharing: "Sharing",
  support: "Support",
  checkinTitle: "Evening check‑in", // U+2011: no line break inside "check-in"
  checkinHint: "Rate first. Your own data follows once you save.",
  notAnswered: "Not rated yet",
  save: "Save check-in",
  saved: "Saved",
  staysHere: "Stays on this tablet.",
  weeklyTo: "A weekly summary of this goes to",
  syntheticReplay: "Synthetic replay",
  syntheticNote: "All data on this screen is generated for demonstration. None of it is astronaut data.",
  linkUp: "Ground link up",
  linkDown: "Loss of signal: on-board only",
  nextDay: "Close this day",
  continueHere: "Continue from this day",
  continueNote: "Later days of the replay are set aside, and what you do here stays in this window only.",
  newNormal: "This is my new normal",
  newNormalNote: "Accept your recent level as the reference for this value. It shows as a hinge, and you can undo it.",
  undoNormal: "Undo: this is not my normal",
  rangeNote: "The band is your usual range: about 8 in 10 of your days fall inside it. A wider band means the model is less certain.",
  anchorNote: "Dashed line: your pre-flight level.",
  screenerTitle: "Harder thoughts (optional)",
  screenerIntro: "Two questions, asked only when you choose. Before you answer, each one shows where a yes would go.",
  screenerPlaceholder: "Item wording is a placeholder for the licensed, validated screener text, to be set before flight.",
  planTitle: "Your plan",
  plan1: "Things that help me settle, written before flight.",
  plan2: "People I can reach: the CMO on board, my trusted crewmate, my behavioural health team.",
  plan3: "A private conference can be asked for at any time, without giving a reason.",
  ppc: "Ask for a private conference",
  crew: "Ask to talk with a crewmate",
  yes: "Yes",
  no: "No",
  sendAnswers: "Save answers",
  lightAuto: "Auto",
  lightDay: "Day",
  lightEvening: "Evening",
  replayOpen: "Open the synthetic replay",
  liveOpen: "Start with a synthetic pre-flight package",
  nothingToday: "No notes today.",
  aboutTitle: "About this measure",
  vitalsTitle: "Morning measurements",
  vitalsHint: "Measured with the station's medical kit. Each value is checked against your safety lines as soon as it is saved.",
  saveVitals: "Save measurements",
  kernelAgree: "Independent safety check: both versions agreed on every value shown.",
};

type Template = (s: Slots) => Msg;
/** Outputs not worded as a message, each with the reason (the coverage test requires one or the other). */
type Silent = { silent: string };

export type Catalogue = {
  locale: string;
  indicator: Record<string, { label: string; low: string; mid: string; high: string; unit: string }>;
  recipient: Record<string, string>;
  shareClass: Record<ShareClass, string>;
  item: Record<string, { question: string; label: string }>;
  /** NASA HRP research area per indicator, named without diagnostic vocabulary (HRP_MAPPING.md). */
  area: Record<string, { name: string; url: string }>;
  /** "About this measure": how to take it, why it changes in space, what helps (challenge: simple instructions). */
  about: Record<string, { how: string; space: string; helps: string }>;
  messages: Record<OutKind, Template | Silent>;
  reasons: Record<string, string>;
  ui: Record<keyof typeof UI_EN, string>;
  insight: {
    inside: (ind: string, below: number, of: number) => Msg;
    outside: (ind: string, dirWord: string, below: number, of: number) => Msg;
    early: (ind: string) => Msg;
  };
};

const cap = (x: string) => `${x[0]!.toUpperCase()}${x.slice(1)}`;
const label = (s: Slots) => en.indicator[s.ind]?.label ?? s.ind.replace(/_/g, " ");
const who = (to: string | null) => (to === null ? "no one yet" : (en.recipient[to] ?? to));
const lower = (s: Slots) => (s.dir === -1 ? "lower" : "higher");

export const en: Catalogue = {
  locale: "en",
  indicator: {
    mood: { label: "mood", low: "Unpleasant", mid: "Neutral", high: "Pleasant", unit: "" },
    energy: { label: "energy", low: "Drained", mid: "Even", high: "Energised", unit: "" },
    calm: { label: "calm", low: "Tense", mid: "Steady", high: "Calm", unit: "" },
    sleep_min: { label: "sleep", low: "", mid: "", high: "", unit: "min" },
    spo2: { label: "blood oxygen (SpO2)", low: "", mid: "", high: "", unit: "%" },
    pulse_bpm: { label: "resting pulse", low: "", mid: "", high: "", unit: "bpm" },
    sbp_mmhg: { label: "blood pressure (systolic)", low: "", mid: "", high: "", unit: "mmHg" },
    temp_c: { label: "body temperature", low: "", mid: "", high: "", unit: "°C" },
    glucose_mgdl: { label: "blood glucose", low: "", mid: "", high: "", unit: "mg/dL" },
  },
  recipient: {
    surgeon: "your flight surgeon",
    bhp: "your behavioural health team",
    cmo: "the CMO on board",
    crewmate: "your trusted crewmate",
  },
  shareClass: {
    safety: "Pre-agreed safety notice",
    summary: "Weekly summary",
    ppc: "Request for a private conference",
    crew: "Request to talk with a crewmate",
  },
  about: {
    mood: {
      how: "Rate it in the evening check-in, from unpleasant to pleasant, before looking at any data.",
      space: "Isolation, confinement and distance from Earth can wear on mood over months. NASA counts behavioural health among the main concerns for long missions.",
      helps: "Regular sleep, exercise, time at a window, contact with family and talking things through. NASA names keeping a journal as one self-care option.",
    },
    energy: {
      how: "Rate it in the evening check-in, from drained to energised.",
      space: "Workload, sleep loss and shifting light and dark cycles can drain energy across a long mission.",
      helps: "Protected sleep time, scheduled breaks and the daily exercise session.",
    },
    calm: {
      how: "Rate it in the evening check-in, from tense to calm.",
      space: "Living in a small space with the same few people can let tension build up.",
      helps: "Breathing exercises, private time, and a talk with a trusted crewmate.",
    },
    sleep_min: {
      how: "Imported each morning from the wearable; nothing to enter.",
      space: "The station sees 16 sunrises a day, and lighting schedules and workload both disturb sleep.",
      helps: "The pre-sleep lighting, a regular sleep window, and dark, quiet crew quarters.",
    },
    spo2: {
      how: "Fingertip oximeter, seated and still, for 30 seconds.",
      space: "Exploration vehicles may run at reduced cabin pressure, which can lower blood oxygen a little.",
      helps: "Rest and measure again; tell the CMO if you feel short of breath.",
    },
    pulse_bpm: {
      how: "Before getting up, count your pulse for 60 seconds, or read it from the cuff.",
      space: "In weightlessness the heart and blood vessels adapt; the resting pulse helps follow that over the mission.",
      helps: "The daily exercise session and enough fluids.",
    },
    sbp_mmhg: {
      how: "Cuff on the upper arm, seated and strapped in, after five minutes of rest; note the upper number.",
      space: "Fluids shift towards the head in weightlessness and the circulation adapts. Low pressure can make standing up after landing harder.",
      helps: "Exercise, fluids, and the compression garments worn before landing.",
    },
    temp_c: {
      how: "Each morning before eating or drinking, with the oral or forehead thermometer.",
      space: "Spaceflight can change how the immune system responds, and dormant viruses can wake up. A raised temperature is worth knowing early.",
      helps: "Rest, fluids, and a word with the CMO.",
    },
    glucose_mgdl: {
      how: "Fingerstick meter every third morning, before breakfast.",
      space: "Menus are fixed for months on long missions; glucose shows how the body is handling the space diet.",
      helps: "Regular meals and the variety on the menu. Ask the flight surgeon about lasting changes.",
    },
  },
  area: {
    mood: { name: "behavioural health and performance", url: "https://www.nasa.gov/reference/risk-of-behavioral-changes-and-psychiatric-disorders/" },
    energy: { name: "behavioural health and performance", url: "https://www.nasa.gov/reference/risk-of-behavioral-changes-and-psychiatric-disorders/" },
    calm: { name: "behavioural health and performance", url: "https://www.nasa.gov/reference/risk-of-behavioral-changes-and-psychiatric-disorders/" },
    sleep_min: { name: "sleep and irregular schedules", url: "https://www.nasa.gov/reference/risk-from-inadequate-sleep-and-irregular-schedules/" },
    spo2: { name: "oxygen in reduced-pressure cabins (hypoxia)", url: "https://www.nasa.gov/reference/risk-of-hypoxia/" },
    pulse_bpm: { name: "heart and blood vessels in weightlessness", url: "https://www.nasa.gov/reference/risk-of-cardiovascular-adaptations/" },
    sbp_mmhg: { name: "heart and blood vessels in weightlessness", url: "https://www.nasa.gov/reference/risk-of-cardiovascular-adaptations/" },
    temp_c: { name: "the immune system in spaceflight", url: "https://www.nasa.gov/reference/risk-of-altered-immune-system-responses/" },
    glucose_mgdl: { name: "food and nutrition on long missions", url: "https://www.nasa.gov/reference/risk-of-inadequate-food-and-nutrition/" },
    crew: { name: "teamwork in isolation", url: "https://www.nasa.gov/reference/risk-of-inadequate-teamwork/" },
    support: { name: "independent operation far from Earth", url: "https://www.nasa.gov/reference/risk-of-inadequate-systems-operations-support/" },
  },
  item: {
    cssrs_plan: {
      label: "the question about a plan",
      question: "In the past week, have you made a plan for ending your life?",
    },
    cssrs_intent: {
      label: "the question about intention",
      question: "Do you intend to act on thoughts of ending your life?",
    },
  },
  messages: {
    Safety: (s) =>
      s.screen
        ? {
            register: "advisory",
            title: "Shared as you agreed",
            text: s.share
              ? `As you agreed before flight, ${who(s.to)} has been told that you answered yes to ${en.item[s.item]?.label ?? "one item"}. Nothing else was sent.`
              : "This answer stays on this tablet.",
            action: "use your plan or one of the ways to reach a person below",
            owner: "You",
          }
        : {
            register: "warning",
            title: `${label(s)} past your safety line`,
            text: `Tonight's ${label(s)} of ${s.value} is past the line you agreed before flight. ${who(s.to)[0]!.toUpperCase()}${who(s.to).slice(1)} has the reading.`,
            action: `measure your ${label(s)} again now`,
            owner: "You",
          },
    SafetyCheckFailed: (s) =>
      s.screen
        ? {
            register: "caution",
            title: "Safety check could not confirm an answer",
            text: "The safety check could not confirm one answer. Nothing was sent.",
            action: "if you want someone to talk to, the options on this page reach a person",
            owner: "You",
          }
        : {
            register: "caution",
            title: "Safety check could not confirm a reading",
            text: `The two independent safety checks gave different results for ${label(s)}. ${who(s.to)[0]!.toUpperCase()}${who(s.to).slice(1)} has been asked to look at it.`,
            action: `measure your ${label(s)} again`,
            owner: "You",
          },
    KernelUnavailable: () => ({
      register: "caution",
      title: "Safety check not set up",
      text: "The pre-flight safety table is not loaded, so values are stored without a safety check.",
      action: "load the pre-flight safety table",
      owner: "The CMO on board",
    }),
    OfferExtraSample: (s) => ({
      register: "advisory",
      title: "One more reading, if you like",
      text: `One more ${label(s)} reading later today would make tonight's picture clearer. It is optional.`,
      action: `add a short ${label(s)} reading later today, or let it go`,
      owner: "You",
    }),
    DayStat: { silent: "drawn as the corridor on Course" },
    Alarm: (s) => ({
      register: "caution",
      title: `${label(s)[0]!.toUpperCase()}${label(s).slice(1)} has stayed ${lower(s)} than usual`,
      text: `For several days your ${label(s)} has stayed ${lower(s)} than your usual range. Most often this follows a heavy work block or shorter sleep. This note closes after 3 days back in your range.`,
      action: "look at what changed this week; talking it through with someone you trust is one option",
      owner: "You",
    }),
    AlarmCleared: (s) => ({
      register: "info",
      title: `${label(s)[0]!.toUpperCase()}${label(s).slice(1)} is back in your range`,
      text: `Your ${label(s)} is back in your usual range, so the earlier note is closed.`,
      action: "none needed",
      owner: "You",
    }),
    // Advisory, not caution: no guilt for skipped days (DESIGN_PAPER §4.10).
    MissingnessAlarm: () => ({
      register: "advisory",
      title: "Fewer check-ins lately",
      text: "There have been fewer check-ins than usual lately. Busy weeks happen. Once a check-in is back, your picture is current again.",
      action: "a short check-in when it suits you",
      owner: "You",
    }),
    MissingnessCleared: () => ({
      register: "info",
      title: "Check-ins back to your rhythm",
      text: "Check-ins are back to your usual rhythm.",
      action: "none needed",
      owner: "You",
    }),
    DriftPrompt: (s) => ({
      register: "advisory",
      title: `A slow drift in ${label(s)}`,
      text: `Over the last ${s.n} days your ${label(s)} has drifted slowly ${lower(s)}. It is a gentle trend, not a sudden change.`,
      action: "look at it on Course and decide whether it is your new normal",
      owner: "You",
    }),
    AnchorRunPrompt: (s) => ({
      register: "advisory",
      title: `${label(s)[0]!.toUpperCase()}${label(s).slice(1)} away from your pre-flight level`,
      text: `For two weeks your ${label(s)} has sat away from your pre-flight level. If this level feels right to you, you can accept it as your new normal, and undo that later.`,
      action: "decide on Course whether this is your new normal",
      owner: "You",
    }),
    Calibrating: () => ({
      register: "info",
      title: "Learning your usual range",
      text: "ANCHOR is still learning your usual range. Notes about change start after the first week.",
      action: "keep checking in when it suits you",
      owner: "You",
    }),
    Recalibrated: { silent: "technical: threshold re-derived weekly; recorded in the log for audit" },
    Rejected: (s) => ({
      register: "info",
      title: "Not saved",
      text: `That change was not saved: ${s.value}.`,
      action: "nothing else changed; you can try again later",
      owner: "You",
    }),
    CompanionError: () => ({
      register: "caution",
      title: "Part of tonight's analysis did not run",
      text: "Part of tonight's analysis could not run, so no new note was made. Your entries are stored safely, and the safety check ran as usual.",
      action: "look at the technical record when convenient",
      owner: "The CMO on board",
    }),
    PrivateSupport: () => ({
      register: "advisory",
      title: "Thank you for answering",
      text: "These answers stay on this tablet unless you agreed otherwise before flight. Your plan and two ways to reach a person are here.",
      action: "open your plan, or ask for a private conference or a crewmate",
      owner: "You",
    }),
    ConsentChanged: (s) => ({
      register: "info",
      title: "Sharing choice updated",
      text: s.share
        ? `From now on, a yes to ${en.item[s.item]?.label ?? "this item"} goes to ${who(s.to)} as you chose.`
        : `From now on, a yes to ${en.item[s.item]?.label ?? "this item"} stays on this tablet.`,
      action: "change it again on Sharing at any time",
      owner: "You",
    }),
    Reanchored: (s) => ({
      register: "info",
      title: `New normal set for ${label(s)}`,
      text: `Your new normal for ${label(s)} is set. Course shows it as a hinge in your range.`,
      action: "undo it on Course if it does not feel right",
      owner: "You",
    }),
    ReanchorReverted: (s) => ({
      register: "info",
      title: `Earlier normal back for ${label(s)}`,
      text: `Your earlier normal for ${label(s)} is back. Days since the change will not be learned as normal.`,
      action: "none needed",
      owner: "You",
    }),
    ShareQueued: (s) => ({
      register: "info",
      title: en.shareClass[s.cls],
      text: s.to === null ? `${en.shareClass[s.cls]}: no one on your list can receive it now, so it waits on this tablet.` : `${en.shareClass[s.cls]} for ${who(s.to)}: listed here before it is sent.`,
      action: "see what it contains on Sharing",
      owner: "You",
    }),
    ShareSent: { silent: "shown as the share's status in the sharing log" },
    ShareRerouted: (s) => ({
      register: "info",
      title: "Going to someone else on your list",
      text: `This now goes to ${who(s.to)}, the next person on your list who can receive it.`,
      action: "change your list on Sharing if you want",
      owner: "You",
    }),
    ShareUndeliverable: (s) => ({
      register: "info",
      title: "Waiting on this tablet",
      text: `${en.shareClass[s.cls]}: no one on your list can receive it now, so it waits on this tablet.`,
      action: "change your list on Sharing if you want it to go now",
      owner: "You",
    }),
    ShareCancelled: () => ({
      register: "info",
      title: "Not sent",
      text: "You stopped sharing this summary before it left, so it was not sent.",
      action: "none needed",
      owner: "You",
    }),
    ExclusionChanged: (s) => ({
      register: "info",
      title: "Contact list updated",
      text: s.share
        ? `${who(s.to)[0]!.toUpperCase()}${who(s.to).slice(1)} will receive nothing from ANCHOR.`
        : `${who(s.to)[0]!.toUpperCase()}${who(s.to).slice(1)} can again receive what you share.`,
      action: "change it again on Sharing at any time",
      owner: "You",
    }),
    RouteEmpty: (s) => ({
      register: "caution",
      title: "No one left on a list",
      text: `Everyone on the list for ${en.shareClass[s.cls].toLowerCase()} is now excluded, so these will wait on this tablet.`,
      action: "add someone back on Sharing if you want them to go",
      owner: "You",
    }),
  },
  reasons: {
    "alarm episode younger than 14 days": "a new normal can be set once a change has lasted 14 days",
    "re-anchoring limited to once per 30 days": "a new normal can be set once every 30 days",
    "not enough recent days": "it needs 14 recent days to describe your level",
    "new level too far from pre-flight to accept alone": "this level is far from your pre-flight level, so it is one to agree together with your flight surgeon",
    "re-anchoring forbidden for safety-line indicators": "values with a pre-agreed safety line keep their pre-flight reference",
    "nothing to revert": "there is no earlier normal to go back to",
    "day is not the open day": "it belongs to a day that is already closed",
    "consent unchanged": "it was already set that way",
    "exclusion unchanged": "it was already set that way",
    "sharing unchanged": "it was already set that way",
    "link state unchanged": "it was already set that way",
  },
  ui: UI_EN,
  insight: {
    inside: (ind, below, of) => ({
      register: "info",
      title: `${cap(en.indicator[ind]?.label ?? ind)} tonight`,
      text: `Your ${en.indicator[ind]?.label ?? ind} tonight is inside your usual range, higher than ${below} of your last ${of} check-ins.`,
      action: "look at how it has moved over the mission on Course",
      owner: "You",
    }),
    outside: (ind, dirWord, below, of) => ({
      register: "info",
      title: `${cap(en.indicator[ind]?.label ?? ind)} tonight`,
      text: `Your ${en.indicator[ind]?.label ?? ind} tonight is ${dirWord} than your usual range, higher than ${below} of your last ${of} check-ins. One evening on its own says little.`,
      action: "look at how it has moved over the mission on Course",
      owner: "You",
    }),
    early: (ind) => ({
      register: "info",
      title: `${cap(en.indicator[ind]?.label ?? ind)} tonight`,
      text: `Saved. Your usual range for ${en.indicator[ind]?.label ?? ind} is still being learned, so tonight is not compared yet.`,
      action: "keep checking in when it suits you",
      owner: "You",
    }),
  },
};

export const catalogues: Record<string, Catalogue> = { en };

export function isSilent(t: Template | Silent): t is Silent {
  return typeof t !== "function";
}
