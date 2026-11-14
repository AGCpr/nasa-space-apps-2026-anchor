// The four destinations (UI_RESEARCH §3): Today, Course, Sharing, Support. Each is a function of
// the projected state; the signature move (the personal range as a terrain corridor with its
// pre-flight benchmark and re-anchor hinges) lives on Course and in Today's range bars.
import type { Catalogue, Msg, Slots, Why } from "./catalogue.ts";
import { isSilent } from "./catalogue.ts";
import type { Indicator, Note, Share, State } from "./model.ts";
import { shown } from "./model.ts";
import { bipolar, card, h, s, popUp, segmented, stepper } from "./ui.ts";

export type View = "today" | "course" | "sharing" | "support";

export type Ctx = {
  st: State;
  cat: Catalogue;
  mode: "replay" | "fork" | "live";
  canAct: boolean;
  draft: Record<string, number>;
  courseInd: string;
  screen: Record<string, 0 | 1>;
  /** Morning measurements being entered, in record units. */
  vitals: Record<string, number>;
  kernel: { compared: number; disagreements: number; injected: boolean };
  /** Inner width of the chart sheet in CSS px. */
  sheetWidth: number;
  flash: Msg[];
  act: {
    setDraft(k: string, v: number): void;
    saveCheckin(): void;
    setCourse(k: string): void;
    reanchor(k: string, revert: boolean): void;
    shareCategory(k: string, on: boolean): void;
    consent(line: number, share: boolean): void;
    exclude(k: string, on: boolean): void;
    link(up: boolean): void;
    request(kind: "ppc_request" | "crew_request"): void;
    setScreen(k: string, v: 0 | 1): void;
    setVital(k: string, v: number): void;
    saveVitals(): void;
    saveScreen(): void;
    go(v: View): void;
  };
};

const SLIDERS = ["mood", "energy", "calm"];

const RANK: Record<Msg["register"], number> = { warning: 0, caution: 1, advisory: 2, info: 3 };

/** Display value of a DayStat field (sliders arrive 0..1000, scaled values in their units). */
const statShown = (i: Indicator, v: number) => (i.slider ? v / 10 : v);

export function fmt(c: Catalogue, i: Indicator, raw: number): string {
  const unit = c.indicator[i.key]?.unit ?? "";
  const v = shown(i, raw);
  return `${v.toFixed(i.slider ? 0 : i.decimals)}${unit ? ` ${unit}` : ""}`;
}

const label = (c: Catalogue, k: string) => c.indicator[k]?.label ?? k.replace(/_/g, " ");
const cap = (x: string) => `${x[0]!.toUpperCase()}${x.slice(1)}`;
const who = (c: Catalogue, k: string | null) => (k === null ? "no one yet" : (c.recipient[k] ?? k));

/** Word a note through the catalogue; silent outputs give null. */
export function message(c: Catalogue, st: State, n: Note): Msg | null {
  const t = c.messages[n.out.t];
  if (isSilent(t)) return null;
  const o = n.out;
  // A screener item is a safety-table key that is not an indicator (also for a kernel_disagree on it).
  const screen = n.kind === "screen" || ("indicator" in o && !st.indicators.has(o.indicator));
  const slots: Slots = { ind: "", dir: -1, to: null, value: "", n: 0, cls: "safety", item: "", share: false, screen };
  if ("indicator" in o) {
    slots.ind = o.indicator;
    slots.item = o.indicator;
    slots.dir = st.indicators.get(o.indicator)?.dir ?? -1;
  }
  if (o.t === "Safety" || o.t === "SafetyCheckFailed") {
    const i = st.indicators.get(o.indicator);
    slots.value = i ? fmt(c, i, o.value) : String(o.value);
    const q = st.notes.find((x) => x.seq === n.seq && x.out.t === "ShareQueued");
    slots.to = q && q.out.t === "ShareQueued" ? q.out.to : null;
    slots.share = o.t === "Safety" && o.hits.some((x) => x.share);
  }
  if (o.t === "DriftPrompt") slots.n = o.n;
  if ("class" in o) slots.cls = o.class;
  if (o.t === "ShareQueued" || o.t === "ShareRerouted" || o.t === "ShareSent") slots.to = o.to;
  if (o.t === "ShareUndeliverable" || o.t === "ShareCancelled") slots.cls = st.shares.get(o.share)?.cls ?? "safety";
  if (o.t === "ExclusionChanged") {
    slots.to = o.recipient;
    slots.share = o.excluded;
  }
  if (o.t === "ConsentChanged") {
    slots.item = st.table?.lines.find((l) => l.id === o.line)?.indicator ?? "";
    slots.share = o.share;
    slots.to = st.routes.get("safety")?.[0] ?? null;
  }
  if (o.t === "Rejected") slots.value = c.reasons[o.reason] ?? "it did not pass a check";
  const m = t(slots);
  const w = why(c, st, n, slots);
  return w ? { ...m, why: w } : m;
}

/** "Why this note?" (HRP_MAPPING.md, Systems Operations Support): data, rule, confidence and what closes it. */
function why(c: Catalogue, st: State, n: Note, s: Slots): Why | undefined {
  const o = n.out;
  const area = (k: string) => c.area[k] ?? (s.screen ? c.area.mood : undefined);
  const lab = label(c, s.ind);
  const base = (lines: string[], k = s.ind): Why => {
    const a = area(k);
    return a ? { lines, area: a } : { lines };
  };
  switch (o.t) {
    case "Safety": {
      if (s.screen) return base(["Rule: a yes always offers your plan and two ways to reach a person.", "Only items you agreed to share before flight are shared, and only the fact of a yes, never the answer text."]);
      const l = st.table?.lines.find((x) => o.hits.some((hh) => hh.id === x.id));
      const i = st.indicators.get(o.indicator);
      return base([
        `Data: your ${lab} reading of ${s.value} on day ${o.day}.`,
        l && i ? `Rule: a line you agreed before flight (${l.op} ${fmt(c, i, l.limit)}). It does not depend on your usual range.` : "Rule: a line you agreed before flight.",
        "Check: two separately written safety programs test every value. If they ever disagree, the value is marked unconfirmed.",
      ]);
    }
    case "SafetyCheckFailed":
      return base(["Rule: two separately written safety programs test every value.", "When they disagree, the value is marked unconfirmed and, outside the optional questions, shared as such."], "support");
    case "Alarm": {
      const i = st.indicators.get(o.indicator);
      const days = i ? i.stats.filter((d) => d.day > o.day - 14 && d.day <= o.day).length : 0;
      return base([
        `Data: your daily ${lab} on ${days} of the 14 days up to day ${o.day}, each compared with your usual range at the time.`,
        `Rule: each day's gap from your usual range adds to an evidence score. It reached ${o.statistic.toFixed(1)} on day ${o.day}, past the level of ${o.h.toFixed(1)} set for you.`,
        "That level is tuned so that, while nothing changes, a note like this comes about once a month or less.",
        "Closes: after 3 days back in your usual range, or when you set a new normal.",
      ]);
    }
    case "AlarmCleared":
      return base(["Rule: the evidence score fell back to zero and stayed there for 3 days."]);
    case "DriftPrompt":
      return base([`Data: your ${lab} over the last ${o.n} days.`, "Rule: the trend is steady enough that chance alone would produce it less than once in 100 times, even allowing for each day resembling the one before.", "A drift is a question for you, not a finding."]);
    case "AnchorRunPrompt":
      return base(["Rule: your average over the last 14 days is further from your pre-flight level than your usual day-to-day variation before flight.", "Setting a new normal is your choice and can be undone."]);
    case "MissingnessAlarm":
      return base(["Rule: the share of days without a check-in rose above your own usual rate.", "Days you tagged as heavy workload are not counted."], "support");
    case "OfferExtraSample":
      return base(["Rule: your evidence score passed half of your level today, and for you a second reading adds enough to be worth asking."]);
    case "Reanchored":
      return base(["Rule: a new normal can be set once a change has lasted 14 days, at most once every 30 days, and only within twice your usual pre-flight variation.", "It is recorded in your log and can be undone."]);
    case "PrivateSupport":
      return base(["Rule: any yes offers your plan and two ways to reach a person.", "Your answers stay on this tablet unless you agreed otherwise before flight."], "crew");
    default:
      return undefined;
  }
}

// ---------- Today ----------

function grouped(c: Catalogue, alarms: { i: Indicator; m: Msg; h: number; stat: number }[]): Msg {
  const m = c.group(alarms.map((a) => ({ ind: a.i.key, since: a.i.alarmDay!, dir: a.i.dir })));
  // Every NASA research area the grouped measures belong to, each once.
  const areas = [...new Map(alarms.map((a) => c.area[a.i.key]).filter((a) => a !== undefined).map((a) => [a.url, a])).values()];
  const lines = [
    "Rule: each day's gap from your usual range adds to an evidence score for that measure.",
    ...alarms.map((a) => `${cap(label(c, a.i.key))}: the score reached ${a.stat.toFixed(1)} on day ${a.i.alarmDay}, past the level of ${a.h.toFixed(1)} set for you.`),
    "Each level is tuned so that, while nothing changes, a note like this comes about once a month or less.",
    "Closes: each one after 3 days back in its usual range, or when you set a new normal.",
  ];
  const [area, ...also] = areas;
  return { ...m, why: area ? { lines, area, ...(also.length ? { also } : {}) } : { lines } };
}

function rangeBar(c: Catalogue, i: Indicator, tonight: number | null): HTMLElement | null {
  const last = i.stats.at(-1);
  if (!last) return null;
  const lo = statShown(i, last.lo), hi = statShown(i, last.hi), a = statShown(i, last.anchor);
  const x = (v: number) => Math.min(100, Math.max(0, v));
  return h(
    "div",
    { class: "rbar" },
    h("span", { class: "marg" }, label(c, i.key)),
    h(
      "div",
      { class: "rbar__track", role: "img", "aria-label": `Usual range ${lo.toFixed(0)} to ${hi.toFixed(0)} of 100${tonight === null ? "" : `, tonight ${(tonight / 10).toFixed(0)}`}` },
      h("div", { class: "rbar__range", style: `left:${x(lo)}%;width:${x(hi) - x(lo)}%` }),
      h("div", { class: "rbar__anchor", style: `left:${x(a)}%` }),
      tonight === null ? null : h("div", { class: "rbar__tonight", style: `left:${x(tonight / 10)}%` }),
    ),
  );
}

function insight(c: Catalogue, st: State, values: Record<string, number>): Msg | null {
  let best: { msg: Msg; score: number } | null = null;
  for (const k of SLIDERS) {
    const i = st.indicators.get(k);
    const v = values[k];
    if (!i || v === undefined) continue;
    const last = i.stats.at(-1);
    const past = [...i.values.entries()].filter(([d]) => d < st.openDay || d <= st.lastClosed).slice(-11, -1).map(([, x]) => x[0]!);
    if (!last || past.length < 5 || !st.calibrated) {
      best ??= { msg: c.insight.early(k), score: 99 };
      continue;
    }
    const below = past.filter((p) => p < v).length;
    const y = v / 10;
    const lo = statShown(i, last.lo), hi = statShown(i, last.hi);
    const outside = y < lo || y > hi;
    // Most notable = furthest toward the harmful side; outside the range only counts on that side.
    const harmful = i.dir === -1 ? y < lo : y > hi;
    const score = (i.dir === -1 ? below : past.length - below) / past.length - (harmful ? 1 : 0);
    const m0 = outside ? c.insight.outside(k, y < lo ? "lower" : "higher", below, past.length) : c.insight.inside(k, below, past.length);
    const a = c.area[k];
    const lines = [`Data: tonight's ${label(c, k)} against your last ${past.length} check-ins and your current usual range.`, `Usual range: ${lo.toFixed(0)} to ${hi.toFixed(0)} of 100, the band where about 8 in 10 of your days fall.`, "No single evening changes any note; only a sustained shift does."];
    const msg: Msg = { ...m0, why: a ? { lines, area: a } : { lines } };
    if (!best || score < best.score) best = { msg, score };
  }
  return best?.msg ?? null;
}

function shareState(c: Catalogue, st: State, k: string): string {
  const i = st.indicators.get(k);
  const lines = st.table?.lines.filter((l) => l.indicator === k) ?? [];
  const parts: string[] = [];
  if (i?.shared) parts.push(`${c.ui.weeklyTo} ${who(c, st.routes.get("summary")?.[0] ?? null)}.`);
  for (const l of lines) parts.push(`Pre-agreed safety line: a value ${l.op} ${fmt(c, i!, l.limit)} goes to ${who(c, st.routes.get("safety")?.[0] ?? null)}.`);
  return parts.length ? parts.join(" ") : c.ui.staysHere;
}

export function today(x: Ctx): HTMLElement {
  const { st, cat: c } = x;
  const done = st.checkinDays.has(st.openDay) || (x.mode === "replay" && st.checkinDays.has(st.lastClosed));
  const shownDay = x.mode === "replay" ? st.lastClosed : st.openDay;
  const values: Record<string, number> = {};
  for (const k of SLIDERS) {
    const v = st.indicators.get(k)?.values.get(shownDay)?.[0];
    if (v !== undefined) values[k] = v;
  }
  const head = h("header", { class: "view__head" }, h("h1", {}, done ? `Tonight, day ${shownDay}` : `${c.ui.checkinTitle}, day ${shownDay}`));
  const body: (HTMLElement | null)[] = [];
  if (!done && x.canAct) {
    const ready = SLIDERS.every((k) => x.draft[k] !== undefined);
    const sameDest = new Set(SLIDERS.map((k) => shareState(c, st, k))).size === 1;
    body.push(
      h("p", { class: "lede" }, c.ui.checkinHint),
      ...SLIDERS.map((k) => {
        const l = c.indicator[k]!;
        return h(
          "div",
          { class: "item" },
          bipolar({ id: `sl-${k}`, label: cap(l.label), low: l.low, mid: l.mid, high: l.high, value: x.draft[k] ?? null, notRated: c.ui.notAnswered, disabled: false, onCommit: (v) => x.act.setDraft(k, v) }),
          sameDest ? null : h("p", { class: "share-state" }, h("span", { class: "marg" }, "Where it goes"), ` ${shareState(c, st, k)}`),
        );
      }),
      sameDest ? h("p", { class: "share-state" }, h("span", { class: "marg" }, "Where it goes"), ` ${shareState(c, st, SLIDERS[0]!)}`) : null,
      h("button", { type: "button", class: "primary", disabled: !ready, onclick: () => x.act.saveCheckin() }, c.ui.save),
    );
  } else if (!done) {
    body.push(h("p", { class: "lede" }, "No check-in on this day."));
  } else {
    const m = insight(c, st, values);
    const bars = SLIDERS.map((k) => {
      const i = st.indicators.get(k);
      return i ? rangeBar(c, i, values[k] ?? null) : null;
    }).filter((b) => b !== null);
    if (bars.length) body.push(h("section", { class: "ranges" }, h("h2", { class: "marg" }, "Tonight in your usual range"), m ? h("p", { class: "ranges__lede" }, m.text) : null, ...bars, h("p", { class: "fine" }, `${c.ui.rangeNote} Dashed mark: your pre-flight level.`)));
  }
  // Open states first (one note per episode), then this day's notes.
  const open: Msg[] = [];
  const alarms: { i: Indicator; m: Msg; h: number; stat: number }[] = [];
  for (const i of st.indicators.values()) {
    if (i.alarmDay === null) continue;
    const n = [...st.notes].reverse().find((z) => z.out.t === "Alarm" && "indicator" in z.out && z.out.indicator === i.key);
    const m = n && message(c, st, n);
    if (m && n.out.t === "Alarm") alarms.push({ i, m, h: n.out.h, stat: n.out.statistic });
  }
  if (alarms.length === 1) open.push({ ...alarms[0]!.m, title: `${alarms[0]!.m.title} (since day ${alarms[0]!.i.alarmDay})` });
  else if (alarms.length > 1) open.push(grouped(c, alarms));
  if (st.missingness) {
    const n = [...st.notes].reverse().find((z) => z.out.t === "MissingnessAlarm");
    const m = n && message(c, st, n);
    if (m) open.push(m);
  }
  const dayNotes = st.notes
    .filter((n) => n.day === shownDay && n.out.t !== "Alarm" && n.out.t !== "MissingnessAlarm" && n.out.t !== "ShareQueued" && n.out.t !== "ShareRerouted")
    // F4: at most one proactive suggestion a day; the first offer stands for the others.
    .filter((n, _, all) => n.out.t !== "OfferExtraSample" || all.find((z) => z.out.t === "OfferExtraSample") === n)
    .map((n) => message(c, st, n))
    .filter((m): m is Msg => m !== null);
  const all = [...open, ...dayNotes].sort((a, b) => RANK[a.register] - RANK[b.register]);
  const rating = !done && x.canAct;
  const notes = rating ? all.filter((m) => m.register === "warning") : all;
  const held = all.length - notes.length;
  const recent = [...st.checkinDays].filter((d) => d > shownDay - 14 && d <= shownDay).length;
  const queued = [...st.shares.values()].filter((sh) => sh.day === shownDay).length;
  return h(
    "section",
    { class: "view" },
    head,
    h(
      "div",
      { class: "cols" },
      h("div", { class: "col" }, ...body, vitalsCard(x)),
      h(
        "section",
        { class: "col notes" },
        h("h2", { class: "marg" }, "Notes"),
        ...notes.map((m) => card(m)),
        held ? h("p", { class: "held" }, `${held} note${held > 1 ? "s" : ""} about your own data ${held > 1 ? "appear" : "appears"} once you save tonight's check-in.`) : notes.length ? null : h("p", {}, c.ui.nothingToday),
        h("p", { class: "fine" }, `Coverage: ${recent} check-ins in the last 14 days.${queued ? ` ${queued} item${queued > 1 ? "s" : ""} listed on Sharing today.` : ""}`),
      ),
    ),
  );
}

// ---------- Morning measurements (challenge: prompt measurements as part of the routine) ----------

export const VITALS = ["pulse_bpm", "sbp_mmhg", "temp_c", "glucose_mgdl"];
const STEP: Record<string, number> = { pulse_bpm: 1, sbp_mmhg: 2, temp_c: 1, glucose_mgdl: 1 };

function vitalsCard(x: Ctx): HTMLElement | null {
  const { st, cat: c } = x;
  const keys = VITALS.filter((k) => st.indicators.has(k));
  if (!keys.length || !x.canAct) return null;
  const today = st.openDay;
  if (keys.some((k) => st.indicators.get(k)!.values.has(today))) return null;
  return h("section", { class: "vitals" },
    h("h2", { class: "marg" }, c.ui.vitalsTitle),
    h("p", { class: "fine" }, c.ui.vitalsHint),
    ...keys.map((k) => {
      const i = st.indicators.get(k)!;
      const touched = x.vitals[k] !== undefined;
      const v = x.vitals[k] ?? [...i.values.values()].at(-1)?.[0] ?? 0;
      return stepper({ id: `v-${k}`, label: cap(label(c, k).replace(/ \(.*\)/, "")), value: shown(i, v), step: STEP[k]! / 10 ** i.decimals, decimals: i.decimals, unit: c.indicator[k]?.unit ?? "", disabled: false,
        hint: touched ? undefined : c.ui.lastReading,
        onChange: (n) => x.act.setVital(k, Math.round(n * 10 ** i.decimals)) });
    }),
    h("p", { class: "share-state" }, h("span", { class: "marg" }, "Where it goes"), ` ${c.ui.staysHere} Values past a pre-agreed safety line go to ${who(c, st.routes.get("safety")?.[0] ?? null)}.`),
    h("button", { type: "button", class: "secondary", disabled: Object.keys(x.vitals).length === 0, onclick: () => x.act.saveVitals() }, c.ui.saveVitals));
}

/** "About this measure": how to take it, why it changes in space, what helps, its safety line. */
function about(c: Catalogue, st: State, k: string): HTMLElement | null {
  const a = c.about[k];
  if (!a) return null;
  const i = st.indicators.get(k);
  const line = st.table?.lines.find((l) => l.indicator === k && !l.high);
  const area = c.area[k];
  return h("section", { class: "about" },
    h("h2", { class: "marg" }, c.ui.aboutTitle),
    h("dl", {},
      h("dt", {}, "How to measure"), h("dd", {}, a.how),
      h("dt", {}, "Why it changes in space"), h("dd", {}, a.space),
      h("dt", {}, "What helps"), h("dd", {}, a.helps),
      line && i ? h("dt", {}, "Safety line") : null,
      line && i ? h("dd", {}, `A value ${line.op} ${fmt(c, i, line.limit)} goes to ${who(c, st.routes.get("safety")?.[0] ?? null)}, as agreed before flight.`) : null),
    area ? h("p", { class: "fine" }, "NASA research area: ", h("a", { href: area.url, target: "_blank", rel: "noopener" }, area.name), ".") : null);
}

// ---------- Course: the survey sheet ----------

function ticks(lo: number, hi: number): number[] {
  const span = hi - lo;
  const raw = span / 5;
  const mag = 10 ** Math.floor(Math.log10(raw));
  const step = [1, 2, 2.5, 5, 10].map((m) => m * mag).find((st) => st >= raw)!;
  const out: number[] = [];
  for (let v = Math.ceil(lo / step) * step; v <= hi; v += step) out.push(+v.toFixed(6));
  return out;
}

type Geo = { x: (d: number) => number; W: number; padL: number; padR: number; upto: number };

/** A label that stays on the sheet: right-aligned when it would run off the right edge. */
function tag(g: Geo, x: number, y: number, text: string, cls: string): SVGElement {
  const right = x > g.W - g.padR - text.length * 9;
  return s("text", { x: right ? x - 12 : x, y, class: cls, "text-anchor": right ? "end" : "start" }, text);
}

/** One SVG unit is one CSS pixel, so chart lettering keeps the 16 px floor (R4). */
function geo(upto: number, W: number): Geo {
  const padL = 72, padR = 64; // right margin holds the current-level stub of a fresh hinge
  return { W, padL, padR, upto, x: (d) => padL + ((d - 1) / Math.max(1, upto - 1)) * (W - padL - padR) };
}

function runs<T extends { day: number }>(xs: T[]): T[][] {
  const out: T[][] = [];
  for (const v of xs) {
    const last = out.at(-1);
    if (last && v.day - last.at(-1)!.day === 1) last.push(v);
    else out.push([v]);
  }
  return out;
}

export function episodes(st: State, key: string, upto: number): { from: number; to: number | null }[] {
  const ep: { from: number; to: number | null }[] = [];
  for (const n of st.notes) {
    if (!("indicator" in n.out) || n.out.indicator !== key) continue;
    if (n.out.t === "Alarm") ep.push({ from: n.out.day, to: null });
    if (n.out.t === "AlarmCleared" && ep.at(-1)?.to === null) ep.at(-1)!.to = n.out.day;
  }
  return ep.filter((e) => e.from <= upto);
}

function corridorChart(c: Catalogue, st: State, i: Indicator, g: Geo): SVGElement {
  // Top band: episode boxes (y 6-32), two label lanes (y 54, 76), the hinge-label row (y 88-114), then the plot.
  // The label band is reserved only when there is something to label; otherwise the plot takes the space.
  const labelled = episodes(st, i.key, g.upto).length > 0 || i.hinges.some((hg) => hg.day <= g.upto);
  const H = 440, top = labelled ? 122 : 48, bottom = H - 30;
  const pts = i.stats.map((d) => ({ day: d.day, y: statShown(i, d.y), lo: statShown(i, d.lo), hi: statShown(i, d.hi), a: statShown(i, d.anchor) }));
  const line = st.table?.lines.find((l) => l.indicator === i.key && !l.high);
  const limit = line ? shown(i, line.limit) : null;
  const safety = i.safetyDays.map((d) => ({ day: d, v: shown(i, i.values.get(d)?.find((v) => line && (line.op === "below" ? v < line.limit : v > line.limit)) ?? 0) }));
  const all = pts.flatMap((p) => [p.y, p.lo, p.hi, p.a]).concat(limit === null ? [] : [limit], safety.map((p) => p.v));
  let lo = Math.min(...all), hi = Math.max(...all);
  if (i.slider) [lo, hi] = [Math.max(0, lo), Math.min(100, hi)];
  const pad = (hi - lo) * 0.08 || 1;
  lo -= pad;
  hi += pad;
  const y = (v: number) => bottom - ((v - lo) / (hi - lo)) * (bottom - top);
  const svg = s("svg", { viewBox: `0 0 ${g.W} ${H}`, class: "chart", role: "img", "aria-labelledby": "chart-sum" });
  // Contours: value lines with labels, the only ornament, and each one reads a value.
  for (const t of ticks(lo, hi)) {
    svg.append(s("line", { x1: g.padL, x2: g.W - g.padR, y1: y(t), y2: y(t), class: "contour" }));
    svg.append(s("text", { x: g.padL - 10, y: y(t) + 6, class: "axis", "text-anchor": "end" }, `${+t.toFixed(2)}`));
  }
  // The axis says what its numbers are (HIG charts.md: describe units where the chart can show them).
  const unit = i.slider ? "0–100" : (c.indicator[i.key]?.unit ?? "");
  if (unit) svg.append(s("text", { x: g.padL - 10, y: top - 14, class: "axis axis--unit", "text-anchor": "end" }, unit));
  // Sustained-change episodes: a caution bar over the days they cover, and one label per episode,
  // starting at its triangle (shape + words, R2). Two label lanes keep neighbours apart.
  const laneEnd = [-Infinity, -Infinity];
  for (const e of episodes(st, i.key, g.upto)) {
    const x0 = g.x(e.from), x1 = g.x(Math.min(e.to ?? g.upto, g.upto));
    const text = `Change noticed · day ${e.from}${e.to ? `–${e.to}` : " on"}`;
    const w = text.length * 8.5 + 26;
    const lx = Math.max(g.padL, Math.min(x0, g.W - 4 - w));
    const lane = Math.max(0, laneEnd.findIndex((end) => lx > end + 16));
    laneEnd[lane] = lx + w;
    const ly = 24 + 30 * lane;
    svg.append(s("rect", { x: x0, y: 74, width: Math.max(6, x1 - x0), height: 6, rx: 3, class: "episode" }));
    svg.append(s("path", { d: `M${x0} 74 V${ly + 6}`, class: "episode__leader" }));
    svg.append(s("path", { d: `M${lx} ${ly + 2} l8 -16 l8 16z`, class: "episode__glyph" }));
    svg.append(s("text", { x: lx + 24, y: ly, class: "episode__label" }, text));
  }
  // The corridor: the personal range, as a band per run of consecutive days.
  for (const r of runs(pts)) {
    const d = `M${r.map((p) => `${g.x(p.day)} ${y(p.hi)}`).join(" L")} L${[...r].reverse().map((p) => `${g.x(p.day)} ${y(p.lo)}`).join(" L")}Z`;
    svg.append(s("path", { d, class: "corridor" }));
  }
  // Benchmark: the anchor in force, dashed; a hinge where the person moved it.
  const anchors = runs(pts).flatMap((r) => {
    const segs: (typeof pts)[] = [];
    for (const p of r) {
      const last = segs.at(-1);
      if (last && Math.abs(last.at(-1)!.a - p.a) < 1e-9) last.push(p);
      else segs.push([p]);
    }
    return segs;
  });
  for (const seg of anchors) {
    const d = seg.length === 1 ? `M${g.x(seg[0]!.day) - 3} ${y(seg[0]!.a)} h6` : `M${seg.map((p) => `${g.x(p.day)} ${y(p.a)}`).join(" L")}`;
    svg.append(s("path", { d, class: "benchmark" }));
  }
  for (const hg of i.hinges) {
    if (hg.day > g.upto) continue;
    const xh = g.x(hg.day);
    const undone = hg.reverted !== null && hg.reverted <= g.upto;
    const yFrom = y(statShown(i, hg.from)), yTo = y(statShown(i, hg.to));
    // The benchmark visibly steps: old level up to the hinge, a vertical step, then the new level
    // (to the next recorded day, or into the right margin when the hinge is today).
    const next = i.stats.find((d) => d.day > hg.day);
    const xEnd = next ? g.x(next.day) : xh + 44;
    svg.append(s("path", { d: `M${xh} ${yFrom} V${yTo} H${xEnd}`, class: `hinge${undone ? " hinge--undone" : ""}` }));
    svg.append(s("circle", { cx: xh, cy: yTo, r: 8, class: `hinge__pin${undone ? " hinge--undone" : ""}` }));
    // Label on a ground-coloured plate in the top band's second lane, tied to the pin by a leader.
    const text = undone ? `New normal undone · day ${hg.reverted}` : `New normal · day ${hg.day}`;
    const w = text.length * 9 + 16;
    const lx = Math.max(g.padL, Math.min(xh - w / 2, g.W - 4 - w));
    svg.append(s("path", { d: `M${xh} ${yTo - 8} V${top - 8}`, class: "hinge__leader" }));
    svg.append(s("rect", { x: lx, y: top - 34, width: w, height: 26, rx: 4, class: "hinge__plate" }));
    svg.append(s("text", { x: lx + 8, y: top - 16, class: "hinge__label" }, text));
  }
  if (limit !== null) {
    svg.append(s("line", { x1: g.padL, x2: g.W - g.padR, y1: y(limit), y2: y(limit), class: "limit" }));
    svg.append(s("text", { x: g.W - g.padR, y: y(limit) - 8, class: "limit__label", "text-anchor": "end" }, `Safety line ${fmt(c, i, line!.limit)}`));
  }
  // The person's own values.
  for (const r of runs(pts)) {
    if (r.length > 1) svg.append(s("path", { d: `M${r.map((p) => `${g.x(p.day)} ${y(p.y)}`).join(" L")}`, class: "trace" }));
    for (const p of r) svg.append(s("circle", { cx: g.x(p.day), cy: y(p.y), r: 3.5, class: "trace__dot" }));
  }
  for (const p of safety) {
    if (p.day > g.upto) continue;
    const cx = g.x(p.day), cy = y(p.v);
    svg.append(s("path", { d: `M${cx} ${cy - 11} l11 11 l-11 11 l-11 -11z`, class: "safety-mark" }));
    svg.append(tag(g, cx + 16, cy + 6, `Day ${p.day}`, "safety-mark__label"));
  }
  svg.append(s("text", { x: g.padL, y: H - 6, class: "axis" }, "Day 1"));
  svg.append(s("text", { x: g.W - g.padR, y: H - 6, class: "axis", "text-anchor": "end" }, `Day ${g.upto}`));
  return svg;
}

/** The mission-day strip (raise from direction B): one column per day on the shared time axis. */
export function strip(st: State, g: Geo, opts: { shares: boolean }): SVGElement {
  const H = 72;
  const svg = s("svg", { viewBox: `0 0 ${g.W} ${H}`, class: "strip", role: "img", "aria-label": "Mission days: check-ins, loss of signal and shares" });
  let down: number | null = null;
  const spans: [number, number][] = [];
  for (const l of st.links) {
    if (!l.up && down === null) down = l.day;
    if (l.up && down !== null) {
      spans.push([down, l.day - 1]);
      down = null;
    }
  }
  if (down !== null) spans.push([down, g.upto]);
  for (const [a, b] of spans) {
    if (a > g.upto) continue;
    svg.append(s("rect", { x: g.x(a) - 3, y: 4, width: g.x(Math.min(b, g.upto)) - g.x(a) + 6, height: 40, class: "strip__los" }));
    svg.append(tag(g, g.x(a), 64, `Loss of signal ${a}–${b}`, "axis"));
  }
  const step = (g.W - g.padL - g.padR) / Math.max(1, g.upto - 1);
  for (let d = 1; d <= g.upto; d++) {
    const has = st.checkinDays.has(d);
    svg.append(s("rect", { x: g.x(d) - Math.max(1, step * 0.3), y: has ? 14 : 26, width: Math.max(2, step * 0.6), height: has ? 22 : 4, class: has ? "strip__in" : "strip__gap" }));
  }
  if (opts.shares) {
    for (const sh of st.shares.values()) {
      if (sh.day > g.upto) continue;
      svg.append(s("circle", { cx: g.x(sh.day), cy: 8, r: 4, class: `strip__share strip__share--${sh.status}` }));
    }
  }
  svg.append(s("text", { x: g.padL - 10, y: 32, class: "axis", "text-anchor": "end" }, "Days"));
  return svg;
}

/** Plain-language evidence that never contradicts the episode list beside it (Principle 4). */
function evidence(st: State, i: Indicator, upto: number): string {
  const last = i.stats.at(-1);
  if (!last || st.h === null) return "Still learning your usual range.";
  if (i.alarmDay !== null) return `A sustained change has been noticed since day ${i.alarmDay}.`;
  const ep = episodes(st, i.key, upto);
  const hinge = i.hinges.filter((hg) => hg.reverted === null && hg.day <= upto).at(-1);
  const end = ep.at(-1)?.to ?? null;
  const settled = hinge && end === hinge.day ? ` The change noticed from day ${ep.at(-1)!.from} became your new normal on day ${hinge.day}.` : "";
  const since = end === null ? "" : end >= upto ? ` from day ${end} on` : ` since day ${end}`;
  const now = last.statistic / st.h >= 0.5 ? `Some evidence of change is building${since}.` : `No sign of sustained change${since}.`;
  return `${now}${settled}`;
}

export function course(x: Ctx): HTMLElement {
  const { st, cat: c } = x;
  const keys = [...st.indicators.keys()];
  const k = st.indicators.has(x.courseInd) ? x.courseInd : keys[0]!;
  const i = st.indicators.get(k)!;
  const upto = Math.max(1, st.lastClosed);
  const g = geo(upto, x.sheetWidth);
  const covered = i.stats.length;
  const last = i.stats.at(-1);
  const eps = episodes(st, k, upto).map((e) => `${e.from}–${e.to ?? "now"}`);
  const sum = last
    ? `${cap(label(c, k))} over ${upto} days: ${covered} days with data. Latest usual range ${statShown(i, last.lo).toFixed(i.slider ? 0 : 1)} to ${statShown(i, last.hi).toFixed(i.slider ? 0 : 1)}. ${evidence(st, i, upto)}${eps.length ? ` Change noticed on days ${eps.join(", ")}.` : ""}`
    : `${cap(label(c, k))}: no data yet.`;
  const active = i.hinges.filter((hg) => hg.reverted === null).length > 0;
  const table = h(
    "details",
    { class: "datatable" },
    h("summary", {}, "Show the values as a table"),
    h("table", {}, h("thead", {}, h("tr", {}, h("th", {}, "Day"), h("th", {}, "Value"), h("th", {}, "Usual range"), h("th", {}, "Pre-flight level"))),
      h("tbody", {}, ...i.stats.slice(-30).reverse().map((d) => h("tr", {}, h("td", {}, String(d.day)), h("td", {}, statShown(i, d.y).toFixed(1)), h("td", {}, `${statShown(i, d.lo).toFixed(1)}–${statShown(i, d.hi).toFixed(1)}`), h("td", {}, statShown(i, d.anchor).toFixed(1)))))),
  );
  return h(
    "section",
    { class: "view" },
    h("header", { class: "view__head" }, h("h1", {}, "Course"), h("p", { class: "subtitle" }, `Mission days 1–${upto}`)),
    popUp("Measure", [
      { label: "Evening check-in", keys: keys.filter((key) => !VITALS.includes(key) && key !== "sleep_min" && key !== "spo2") },
      { label: "Wearable", keys: keys.filter((key) => key === "sleep_min" || key === "spo2") },
      { label: c.ui.vitalsTitle, keys: keys.filter((key) => VITALS.includes(key)) },
    ].map((g) => ({ label: g.label, options: g.keys.map((key) => ({ value: key, label: cap(label(c, key).replace(/ \(.*\)/, "")) })) })), k, (v) => x.act.setCourse(v)),
    h(
      "figure",
      { class: "sheet" },
      h("figcaption", { class: "sheet__head", id: "chart-sum" }, h("h2", { class: "sheet__title" }, cap(label(c, k))), h("p", {}, last ? evidence(st, i, upto) : sum)),
      corridorChart(c, st, i, g),
      strip(st, g, { shares: false }),
      h(
        "div",
        { class: "legend" },
        h("span", { class: "key key--corridor" }, "Usual range (8 in 10 days)"),
        h("span", { class: "key key--benchmark" }, "Pre-flight level"),
        h("span", { class: "key key--hinge" }, "New normal you set"),
        h("span", { class: "key key--episode" }, "Change noticed"),
      ),
      last ? h("p", { class: "fine sheet__detail" }, `${covered} of ${upto} days with data. Latest usual range ${statShown(i, last.lo).toFixed(i.slider ? 0 : 1)} to ${statShown(i, last.hi).toFixed(i.slider ? 0 : 1)}.${eps.length ? ` Change noticed on days ${eps.join(", ")}.` : ""}`) : null,
      table,
    ),
    h("div", { class: "cols" }, h("div", { class: "col" }, about(c, st, k)),
    h(
      "section",
      { class: "col actions" },
      h("h2", { class: "marg" }, "Your reference"),
      h("p", {}, c.ui.newNormalNote),
      active
        ? h("button", { type: "button", class: "secondary", disabled: !x.canAct, onclick: () => x.act.reanchor(k, true) }, c.ui.undoNormal)
        : h("button", { type: "button", class: "secondary", disabled: !x.canAct, onclick: () => x.act.reanchor(k, false) }, c.ui.newNormal),
    )),
  );
}

// ---------- Sharing ----------

function payloadWords(c: Catalogue, st: State, sh: Share): string {
  const p = sh.payload;
  switch (sh.cls) {
    case "safety": {
      if (p.fault === 1) return "That the safety check could not confirm a value. No value is included.";
      const lines = String(p.lines).split("+").map((id) => st.table?.lines.find((l) => l.id === Number(id))).filter((l) => l !== undefined);
      const what = lines.map((l) => (l.high ? `a yes to ${c.item[l.indicator]?.label ?? "an item"}` : `${label(c, l.indicator)} past its line`)).join(" and ");
      const i = lines[0] && st.indicators.get(lines[0].indicator);
      return `${cap(what)}${p.value !== undefined && i ? `, value ${fmt(c, i, Number(p.value))}` : ", without any value"}.`;
    }
    case "summary":
      return `${cap(label(c, String(p.ind)))}: ${p.days} days of data, the week's average position in your range, and whether a note is open. No daily values.`;
    case "ppc":
      return "That you would like a private conference. Nothing else.";
    case "crew":
      return "That you would like to talk. Nothing else.";
  }
}

/** Consecutive log entries that read the same (class, recipient, status, payload words) form one run. */
function logRuns(c: Catalogue, st: State, shares: Share[]): Share[][] {
  const out: Share[][] = [];
  const key = (sh: Share) => `${sh.cls}|${sh.to}|${sh.status}|${payloadWords(c, st, sh)}`;
  for (const sh of shares) {
    const last = out.at(-1);
    if (last && key(last[0]!) === key(sh)) last.push(sh);
    else out.push([sh]);
  }
  return out;
}

const STATUS: Record<Share["status"], string> = { waiting: "Waiting to send", sent: "Sent", cancelled: "Not sent", undeliverable: "Waiting on this tablet" };

export function sharing(x: Ctx): HTMLElement {
  const { st, cat: c } = x;
  const upto = Math.max(1, st.lastClosed);
  const shares = [...st.shares.values()].reverse();
  const toggle = (on: boolean, labelOn: string, labelOff: string, f: () => void, disabled: boolean) =>
    h("button", { type: "button", class: `switch${on ? " switch--on" : ""}`, role: "switch", "aria-checked": on ? "true" : "false", disabled, onclick: f },
      h("span", { class: "switch__text" }, on ? labelOn : labelOff), h("span", { class: "switch__track", "aria-hidden": "true" }, h("span", { class: "switch__knob" })));
  return h(
    "section",
    { class: "view" },
    h("header", { class: "view__head" }, h("h1", {}, "Sharing"), h("p", { class: "subtitle" }, "Nothing leaves this tablet unless it is listed here first. Journal text and how you use the app never leave.")),
    h("figure", { class: "sheet sheet--strip" }, strip(st, geo(upto, x.sheetWidth), { shares: true }), h("figcaption", {}, "Check-ins, loss of signal, and each listed item (top row) by mission day.")),
    h("div", { class: "cols" }, h("div", { class: "col" },
    h("section", { class: "block" }, h("h2", { class: "marg" }, `Sharing log · ${shares.length} items`),
      shares.length === 0 ? h("p", {}, "Nothing has been listed yet.") : h("ol", { class: "log" }, ...logRuns(c, st, shares).map((r) => {
        const sh = r[0]!;
        const days = r.map((z) => z.day).reverse();
        return h("li", { class: `log__row log__row--${sh.status}` },
          h("span", { class: "log__day" }, r.length > 1 ? `Days ${days[0]}–${days.at(-1)}` : `Day ${sh.day}`),
          h("span", { class: "log__what" }, h("strong", {}, c.shareClass[sh.cls]), ` to ${who(c, sh.to)}${r.length > 1 ? ` · ${r.length} times` : ""}`, h("br", {}),
            h("span", { class: "fine" }, payloadWords(c, st, sh), r.length > 1 ? ` On days ${days.join(", ")}.` : "")),
          h("span", { class: "chip" }, STATUS[sh.status]));
      })))),
    h("div", { class: "col" },
    h("section", { class: "block group" }, h("h2", { class: "marg" }, "Weekly summaries (your choice)"),
      ...[...st.indicators.values()].map((i) => h("div", { class: "setting" },
        h("span", {}, cap(label(c, i.key)), i.shared ? h("br", {}) : null, i.shared ? h("span", { class: "fine" }, `To ${who(c, st.routes.get("summary")?.[0] ?? null)}, once a week.`) : null),
        toggle(i.shared, "Shared weekly", "Not shared", () => x.act.shareCategory(i.key, !i.shared), !x.canAct)))),
    h("section", { class: "block group" }, h("h2", { class: "marg" }, "Safety lines agreed before flight"),
      h("p", { class: "fine" }, `A value past one of these lines always goes to ${who(c, st.routes.get("safety")?.[0] ?? null)}. Two independently written checks evaluate every value.`),
      ...(st.table?.lines ?? []).map((l) => {
        const i = st.indicators.get(l.indicator);
        const what = l.high ? `A yes to ${c.item[l.indicator]?.label ?? l.indicator}` : `${cap(label(c, l.indicator))} ${l.op} ${i ? fmt(c, i, l.limit) : l.limit}`;
        return h("div", { class: "setting" },
          h("span", {}, what, l.high ? h("br", {}) : null, l.high ? h("span", { class: "fine" }, l.share ? `Goes to ${who(c, st.routes.get("safety")?.[0] ?? null)}, as you chose.` : c.ui.staysHere) : null),
          l.high ? toggle(l.share, "Shared", "Stays here", () => x.act.consent(l.id, !l.share), !x.canAct) : h("span", { class: "chip" }, "Fixed"));
      }),
      kernelLine(x)),
    h("section", { class: "block group" }, h("h2", { class: "marg" }, "People"),
      ...[...st.recipients.entries()].map(([k, r]) => h("div", { class: "setting" },
        h("span", {}, cap(who(c, k)), h("br", {}), h("span", { class: "fine" }, `${r.crew ? "On board" : "On the ground"} · receives ${[...st.routes.entries()].filter(([, to]) => to.includes(k)).map(([cl]) => c.shareClass[cl].toLowerCase()).join(", ") || "nothing"}`)),
        toggle(!r.excluded, "Can receive", "Excluded", () => x.act.exclude(k, !r.excluded), !x.canAct)))),
    x.mode === "live" ? null : h("section", { class: "block group" }, h("h2", { class: "marg" }, "Demonstration"),
      h("div", { class: "setting" }, h("span", {}, st.linkUp ? c.ui.linkUp : c.ui.linkDown, h("br", {}), h("span", { class: "fine" }, "In flight this comes from the communication system.")),
        toggle(st.linkUp, "Ground link up", "Loss of signal", () => x.act.link(!st.linkUp), !x.canAct))))),
  );
}

/** The N-version comparison result, beside the safety lines it protects. */
function kernelLine(x: Ctx): HTMLElement | null {
  const k = x.kernel;
  if (k.compared === 0) return null;
  return h("p", { class: "fine" }, k.disagreements === 0
    ? `${x.cat.ui.kernelAgree} (${k.compared} values compared${k.injected ? "; kernel B deliberately altered for this demonstration" : ""}).`
    : `Independent safety check: ${k.disagreements} of ${k.compared} values gave different results in the two versions; each is recorded and treated as unconfirmed.`);
}

// ---------- Support ----------

export function support(x: Ctx): HTMLElement {
  const { st, cat: c } = x;
  const items = (st.table?.lines ?? []).filter((l) => l.high);
  const reqStatus = (cls: "ppc" | "crew") => {
    const last = [...st.shares.values()].reverse().find((sh) => sh.cls === cls);
    return last ? `Last asked on day ${last.day}: ${STATUS[last.status].toLowerCase()}.` : "";
  };
  const supportShown = st.notes.some((n) => n.out.t === "PrivateSupport" && n.day >= st.lastClosed);
  const answered = items.every((l) => x.screen[l.indicator] !== undefined);
  return h(
    "section",
    { class: "view" },
    h("header", { class: "view__head" }, h("h1", {}, "Support"), h("p", { class: "subtitle" }, "A person, whenever you want one. Asking needs no reason.")),
    h("div", { class: "cols" }, h("div", { class: "col" },
    h("div", { class: "row row--big" },
      h("div", { class: "reach" },
        h("button", { type: "button", class: "primary", disabled: !x.canAct, onclick: () => x.act.request("ppc_request") }, c.ui.ppc),
        h("p", { class: "fine" }, `Goes to ${who(c, st.routes.get("ppc")?.[0] ?? null)}. Only the request is sent. ${reqStatus("ppc")}`)),
      h("div", { class: "reach" },
        h("button", { type: "button", class: "primary", disabled: !x.canAct, onclick: () => x.act.request("crew_request") }, c.ui.crew),
        h("p", { class: "fine" }, `Goes to ${who(c, st.routes.get("crew")?.[0] ?? null)}. Only the request is sent. ${reqStatus("crew")}`))),
    h("section", { class: `block plan${supportShown ? " plan--open" : ""}` }, h("h2", { class: "marg" }, c.ui.planTitle), h("ul", {}, h("li", {}, c.ui.plan1), h("li", {}, c.ui.plan2), h("li", {}, c.ui.plan3)), h("p", { class: "fine" }, "Your plan stays on this tablet."))),
    h("div", { class: "col" },
    h("details", { class: "block group screener" }, h("summary", {}, h("span", { class: "marg" }, c.ui.screenerTitle), h("span", { class: "screener__open" }, "Open the two questions")), h("p", {}, c.ui.screenerIntro),
      ...items.map((l) => h("div", { class: "question" },
        h("p", { class: "question__text" }, c.item[l.indicator]?.question ?? l.indicator),
        h("p", { class: "share-state" }, h("span", { class: "marg" }, "If you answer yes"), ` ${l.share ? `${cap(who(c, st.routes.get("safety")?.[0] ?? null))} is told that you answered yes, as you agreed before flight. Nothing else is sent.` : "It stays on this tablet."}`),
        segmented(c.item[l.indicator]?.question ?? l.indicator, [{ value: "0", label: c.ui.no }, { value: "1", label: c.ui.yes }], (x.screen[l.indicator] === undefined ? "" : String(x.screen[l.indicator])) as "0" | "1", (v) => x.act.setScreen(l.indicator, v === "1" ? 1 : 0), "seg--answer"))),
      h("button", { type: "button", class: "secondary", disabled: !x.canAct || !answered, onclick: () => x.act.saveScreen() }, c.ui.sendAnswers),
      h("p", { class: "fine" }, c.ui.screenerPlaceholder)))),
  );
}
