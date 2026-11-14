// Controller (ADR-0009): owns the engine instance, storage, the kernel-B comparison and the
// simulated transport; renders the projection of the event stream.
import { compareKernels, tmrEvaluate } from "../../kernel-b/src/kernel.ts";
import { en, type Msg } from "./catalogue.ts";
import { DAY_MS, kernelA, prefixEnd, project, type Event, type State } from "./model.ts";
import { day as dayTokens, evening as eveningTokens, toCss } from "./tokens.ts";
import { card, glyph, h, segmented } from "./ui.ts";
import * as V from "./views.ts";
import { Engine, wire } from "./wasm.ts";

const KEY_A = "anchor.log.a";
const KEY_B = "anchor.log.b";
const KEY_LAUNCH = "anchor.launch";
const HOUR = 3_600_000;
const c = en;

type Mode = "replay" | "fork" | "live";
type Light = "auto" | "day" | "evening";

const app = {
  /** Judge's guide on Today in replay; dismissed for the session, or off via #guide=off. */
  guide: sessionStorage.getItem("anchor.guide") !== "off",
  engine: null as Engine | null,
  mode: "replay" as Mode,
  events: [] as Event[],
  demo: [] as Event[],
  demoDays: 150,
  view: "today" as V.View,
  scrub: 125,
  forkDay: 0,
  light: "auto" as Light,
  draft: {} as Record<string, number>,
  courseInd: "mood",
  screen: {} as Record<string, 0 | 1>,
  vitals: {} as Record<string, number>,
  flash: [] as Msg[],
  kernel: { compared: 0, disagreements: 0, injected: new URLSearchParams(location.search).has("inject-kernel-b-fault") },
  busy: "" as string,
  fatal: "",
};

const root = document.getElementById("app")!;

// ---------- time ----------

function launch(): number {
  const stored = Number(localStorage.getItem(KEY_LAUNCH));
  if (stored) return stored;
  const now = new Date();
  // Day 1 is today (GMT, the ISS crew-day clock).
  const t = Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate()) - DAY_MS;
  localStorage.setItem(KEY_LAUNCH, String(t));
  return t;
}

const lastMet = () => app.events.at(-1)?.met ?? 0;

function metFor(kind: string, st: State): number {
  if (app.mode === "live") return Math.max(lastMet(), Date.now() - launch());
  const at = kind === "day_close" ? 23 * HOUR : 20.5 * HOUR;
  return Math.max(lastMet(), st.openDay * DAY_MS + at);
}

// ---------- storage (ADR-0009 §4): two synchronous copies, written before rendering ----------

function persist(line: string) {
  for (const k of [KEY_A, KEY_B]) localStorage.setItem(k, (localStorage.getItem(k) ?? "") + line + "\n");
}

// ---------- engine calls ----------

function current(): Event[] {
  return app.mode === "replay" ? app.demo.slice(0, prefixEnd(app.demo, app.scrub)) : app.events;
}

function notesOf(seqs: Set<number>): Msg[] {
  const st = project(app.events);
  return st.notes.filter((n) => seqs.has(n.seq)).map((n) => V.message(c, st, n)).filter((m): m is Msg => m !== null);
}

/** Submit one record; returns the appended events, or null after showing why it was not saved. */
function submit(kind: string, fields: Record<string, number | string>, met?: number): Event[] | null {
  const st = project(app.events);
  const r = app.engine!.submit(wire(met ?? metFor(kind, st), kind, fields));
  if (!r.ok) {
    const t = c.messages.Rejected;
    if (typeof t === "function") {
      app.flash.push(t({ ind: "", dir: -1, to: null, value: c.reasons[r.error] ?? "it did not pass a check", n: 0, cls: "safety", item: "", share: false, screen: false }));
    }
    return null;
  }
  for (const e of r.events) {
    app.events.push(e);
    if (app.mode === "live") persist(e.line);
  }
  return r.events;
}

/** N-version comparison (ADR-0009 §7): kernel B on every value, disagreements into the log. */
async function compare(e: Event, table: { text: string; digest: string }, record: boolean) {
  if (!["checkin", "sample", "screen"].includes(e.kind)) return;
  for (const [k, v] of Object.entries(e.f)) {
    if (k === "day" || typeof v !== "number") continue;
    const a = kernelA(e.out, k);
    // Three separately built copies, each digest-checked by kernel B itself.
    const copies: [string, string, string] = [table.text, `${table.text} `.slice(0, -1), [...table.text].join("")];
    let b = (await tmrEvaluate(copies, table.digest, k, BigInt(v))).outcome;
    if (app.kernel.injected) b = { kind: "hits", hits: [] }; // demonstration: kernel B misses every line
    app.kernel.compared++;
    if (a.kind === "hits" && compareKernels(a, b).kind === "check_failed") {
      app.kernel.disagreements++;
      if (record) submit("kernel_disagree", { day: Number(e.f.day), ref: e.seq, key_txt: k });
    }
  }
}

function tableAt(events: readonly Event[]): { text: string; digest: string } | null {
  let t: { text: string; digest: string } | null = null;
  for (const e of events) {
    if (e.kind === "safety_table" && !e.out.some((o) => o.t === "Rejected")) t = { text: String(e.f.table_txt), digest: String(e.f.digest_txt) };
    for (const o of e.out) if (o.t === "ConsentChanged") t = { text: o.table, digest: o.digest };
  }
  return t;
}

/** Audit of a loaded history: kernel B over every past value, displayed, not appended. */
async function auditHistory(events: readonly Event[]) {
  app.kernel.compared = 0;
  app.kernel.disagreements = 0;
  for (let i = 0; i < events.length; i++) {
    const t = tableAt(events.slice(0, i));
    if (t) await compare(events[i]!, t, false);
  }
}

let transport: number | undefined;
/** Simulated transport (ADR-0008 §1): acknowledges listed items after they have been visible. */
function scheduleTransport() {
  clearTimeout(transport);
  transport = window.setTimeout(() => {
    if (app.mode === "replay" || !app.engine) return;
    for (const t of app.engine.transmittable()) submit("sent", { share: t.share }, app.mode === "live" ? undefined : lastMet());
    render();
  }, 2500);
}

async function act(kind: string, fields: Record<string, number | string>) {
  if (app.busy || app.mode === "replay") return;
  app.flash = [];
  const st = project(app.events);
  const evs = submit(kind, fields);
  if (evs) {
    app.busy = "Checking";
    render();
    const table = tableAt(app.events);
    if (table) for (const e of evs) await compare(e, table, true);
    app.busy = "";
    const seqs = new Set(app.events.filter((e) => e.seq >= evs[0]!.seq).map((e) => e.seq));
    app.flash.push(...notesOf(seqs).filter((m) => !(kind === "checkin" && m.register === "info" && m.title === c.shareClass.safety)));
    if (kind === "day_close") {
      app.draft = {};
      app.screen = {};
    }
    scheduleTransport();
  }
  void st;
  render();
  // Show the outcome where it is written: the top of the view.
  if (app.flash.length || kind === "checkin") document.querySelector("main")?.scrollTo(0, 0);
}

// ---------- modes ----------

async function startReplay() {
  app.mode = "replay";
  app.flash = [];
  if (app.demo.length === 0) {
    app.busy = "Preparing the synthetic mission";
    render();
    await new Promise((r) => setTimeout(r, 30));
    // Prebuilt by the same engine at build time (scripts/build-demo.ts); computed here only as a fallback.
    const pre = await fetch("demo-events.json").then((x) => (x.ok ? (x.json() as Promise<Event[]>) : null)).catch(() => null);
    const events = pre ?? (() => { const r = app.engine!.demo(); if (!r.ok) throw new Error(r.error); return r.events; })();
    app.demo = events;
    app.demoDays = project(events).lastClosed;
    app.busy = "";
    // The kernel-B audit of the history runs after the first render; its line updates when done.
    void auditHistory(app.demo).then(render);
  }
  render();
}

async function continueHere() {
  app.busy = "Replaying up to this day";
  render();
  await new Promise((r) => setTimeout(r, 30));
  const lines = app.demo.slice(0, prefixEnd(app.demo, app.scrub)).map((e) => e.line).join("\n");
  const r = app.engine!.load(lines, lines);
  app.busy = "";
  if (!r.ok) throw new Error(r.error);
  app.events = r.events;
  app.mode = "fork";
  app.forkDay = app.scrub;
  app.draft = {};
  app.screen = {};
  app.view = "today";
  render();
}

async function startLive() {
  app.busy = "Opening your log";
  app.flash = [];
  render();
  await new Promise((r) => setTimeout(r, 30));
  const a = localStorage.getItem(KEY_A), b = localStorage.getItem(KEY_B);
  const r = a !== null || b !== null ? app.engine!.load(a ?? "", b ?? "") : app.engine!.preflight();
  if (!r.ok) throw new Error(r.error);
  app.mode = "live";
  app.events = r.events;
  const repaired = r.events.map((e) => `${e.line}
`).join("");
  // A copy repaired on load is written back, so the two copies are independent again (ADR-0002 §5).
  if (a !== repaired || b !== repaired) for (const k of [KEY_A, KEY_B]) localStorage.setItem(k, repaired);
  launch();
  rollover();
  await auditHistory(app.events);
  app.busy = "";
  render();
}

/** Live mode: close every mission day that has ended (GMT crew day). */
function rollover() {
  if (app.mode !== "live") return;
  const today = Math.floor((Date.now() - launch()) / DAY_MS);
  for (let st = project(app.events); st.openDay < today; st = project(app.events)) {
    if (!submit("day_close", { day: st.openDay })) break;
  }
}

// ---------- rendering ----------

function evening(st: State): boolean {
  if (app.light !== "auto") return app.light === "evening";
  const tod = app.mode === "live" ? (Date.now() - launch()) % DAY_MS : st.met % DAY_MS;
  return tod >= 19.5 * HOUR || tod < 6 * HOUR;
}

/** Lighting and data-source controls: in the sidebar on wide layouts, in a disclosure on narrow ones. */
function displayControls(): HTMLElement {
  return h("div", { class: "controls" },
    h("p", { class: "marg" }, "Lighting"),
    segmented("Lighting", [{ value: "auto", label: c.ui.lightAuto }, { value: "day", label: c.ui.lightDay }, { value: "evening", label: c.ui.lightEvening }], app.light, (v) => { app.light = v; render(); }, "seg--fill"),
    h("p", { class: "marg" }, "Data"),
    segmented("Data", [{ value: "replay", label: "Replay" }, { value: "live", label: "Live" }], app.mode === "live" ? "live" : "replay", (v) => void (v === "live" ? startLive() : startReplay()), "seg--fill"));
}

/** Mission status, always visible above the content (HIG: the current location and state stay visible). */
function statusBar(st: State): HTMLElement {
  const shownDay = app.mode === "replay" ? st.lastClosed : st.openDay;
  return h("header", { class: "status" },
    h("span", { class: "status__mark" }, "ANCHOR"),
    h("span", { class: "pill" }, h("span", { class: "marg" }, "Mission day"), h("strong", { class: "num" }, h("span", { class: "pill__short" }, "Day "), String(shownDay))),
    h("span", { class: `pill link link--${st.linkUp ? "up" : "down"}` }, h("strong", {}, st.linkUp ? c.ui.linkUp : c.ui.linkDown)),
    h("span", { class: `pill badge${app.mode === "live" ? "" : " badge--synthetic"}` }, h("strong", {}, app.mode === "live" ? "Live · synthetic pre-flight" : c.ui.syntheticReplay)),
    h("details", { class: "status__more" }, h("summary", {}, "Display and data"), displayControls()));
}

const RAIL: { v: V.View; label: string; d: string }[] = [
  { v: "today", label: c.ui.today, d: "M12 4a8 8 0 1 0 0 16a8 8 0 1 0 0-16Zm0 4v4l3 2" },
  { v: "course", label: c.ui.course, d: "M3 17c4-6 6 2 9-4s5-2 9-6M3 21h18" },
  { v: "sharing", label: c.ui.sharing, d: "M6 12h12M14 7l5 5-5 5M4 4v16" },
  { v: "support", label: c.ui.support, d: "M12 20s-7-4.5-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.5-7 10-7 10Z" },
];

function navItems(): HTMLElement[] {
  return RAIL.map((r) => {
    const icon = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    icon.setAttribute("viewBox", "0 0 24 24");
    icon.setAttribute("aria-hidden", "true");
    icon.innerHTML = `<path d="${r.d}"/>`;
    return h("button", { type: "button", "aria-current": app.view === r.v ? "page" : null, onclick: () => { app.view = r.v; app.flash = []; render(); document.querySelector("main")?.scrollTo(0, 0); } }, icon, h("span", {}, r.label));
  });
}

/** Regular width: a sidebar (HIG layout.md: tab bar becomes a sidebar when space allows). */
function sidebar(): HTMLElement {
  return h("aside", { class: "sidebar" },
    h("div", { class: "sidebar__brand" }, h("span", { class: "wordmark" }, "ANCHOR"), h("span", { class: "fine" }, "A personal baseline for long missions")),
    h("nav", { class: "sidebar__nav", "aria-label": "Sections" }, ...navItems()),
    displayControls());
}

/** Compact width: a bottom tab bar with the same four destinations. */
function tabBar(): HTMLElement {
  return h("nav", { class: "tabbar", "aria-label": "Sections" }, ...navItems());
}

function replayBar(st: State): HTMLElement | null {
  if (app.mode === "live") return null;
  if (app.mode === "fork") {
    return h("footer", { class: "replay" },
      h("p", {}, h("span", { class: "marg" }, c.ui.syntheticReplay), ` Continuing from day ${app.forkDay}.`, h("span", { class: "replay__note" }, ` ${c.ui.continueNote}`)),
      h("button", { type: "button", class: "secondary", disabled: !!app.busy, onclick: () => void act("day_close", { day: st.openDay }) }, `${c.ui.nextDay} (day ${st.openDay})`),
      h("button", { type: "button", class: "ghost", onclick: () => void startReplay() }, "Back to the replay"));
  }
  const range = h("input", { type: "range", min: 1, max: app.demoDays, value: app.scrub, "aria-label": "Replay day", class: "scrub" }) as HTMLInputElement;
  range.addEventListener("input", () => {
    app.scrub = Number(range.value);
    render();
    (document.querySelector(".scrub") as HTMLInputElement | null)?.focus();
  });
  return h("footer", { class: "replay" },
    h("p", {}, h("span", { class: "marg" }, c.ui.syntheticReplay), ` Day ${app.scrub} of ${app.demoDays}.`, h("span", { class: "replay__note" }, ` ${c.ui.syntheticNote}`)),
    range,
    h("button", { type: "button", class: "secondary", onclick: () => void continueHere() }, c.ui.continueHere));
}

/** Inner chart width: the measured content column minus the sheet's padding (styles.css .sheet). */
function sheetWidth(): number {
  const content = document.querySelector<HTMLElement>(".content");
  const w = content ? content.clientWidth : Math.min(1200, innerWidth - (innerWidth >= 1000 ? 300 : 0) - 64);
  return Math.max(300, w - 2 * 24 - 2);
}

let resizeTimer: number | undefined;
window.addEventListener("resize", () => { clearTimeout(resizeTimer); resizeTimer = window.setTimeout(render, 120); });

/** Three places that show what ANCHOR does, one tap each (replay only). */
function guideCard(): HTMLElement {
  const go = (view: V.View, day: number, why = false) => () => {
    app.view = view;
    app.scrub = day;
    render();
    document.querySelector("main")?.scrollTo(0, 0);
    if (why) document.querySelectorAll<HTMLDetailsElement>("details.why").forEach((d) => (d.open = true));
  };
  return h("section", { class: "guide", "aria-label": "Guide to this demonstration" },
    h("div", { class: "guide__head" },
      h("h2", {}, "A 150-day synthetic mission, replayed"),
      h("button", { type: "button", class: "ghost guide__close", onclick: () => { app.guide = false; sessionStorage.setItem("anchor.guide", "off"); render(); } }, "Hide guide")),
    h("p", { class: "fine" }, "Everything here runs in this browser from a generated log. Three places to start:"),
    h("div", { class: "guide__links" },
      h("button", { type: "button", class: "secondary", onclick: () => { app.view = "course"; app.scrub = 20; render(); playMission(); } }, "Play the mission on Course"),
      h("button", { type: "button", class: "secondary", onclick: go("today", 110, true) }, "Open a note and its reasons, day 110"),
      h("button", { type: "button", class: "secondary", onclick: go("sharing", 150) }, "See everything that was shared")),
    h("p", { class: "fine" }, `To try it yourself, choose “${c.ui.continueHere}” in the bar below.`));
}

/** Course time-lapse: steps the replay from the current day to the end. */
function playMission() {
  const step = () => {
    if (app.mode !== "replay" || app.scrub >= app.demoDays) return;
    app.scrub = Math.min(app.demoDays, app.scrub + 2);
    render();
    setTimeout(step, 60);
  };
  step();
}

function render() {
  const scroll = document.querySelector("main")?.scrollTop ?? 0;
  if (app.fatal) {
    root.replaceChildren(h("main", { class: "fatal" }, h("h1", {}, "ANCHOR stopped"), h("p", {}, "The engine stopped unexpectedly. Your stored log is unchanged. Reload to replay it."), h("pre", {}, app.fatal)));
    return;
  }
  if (!app.engine || (app.mode === "replay" && app.demo.length === 0)) {
    root.replaceChildren(h("main", { class: "loading", "aria-busy": "true" }, h("p", { class: "marg" }, "ANCHOR"), h("p", {}, `${app.busy || "Loading"}…`)));
    return;
  }
  const st = project(current());
  const ev = evening(st);
  document.documentElement.dataset.light = ev ? "evening" : "day";
  document.documentElement.style.cssText = toCss(ev ? eveningTokens : dayTokens);
  const ctx: V.Ctx = {
    st,
    cat: c,
    mode: app.mode,
    canAct: app.mode !== "replay" && !app.busy,
    draft: app.draft,
    courseInd: app.courseInd,
    screen: app.screen,
    vitals: app.vitals,
    kernel: app.kernel,
    sheetWidth: sheetWidth(),
    flash: app.flash,
    act: {
      setDraft: (k, v) => { app.draft[k] = v; render(); },
      saveCheckin: () => void act("checkin", { day: st.openDay, ...app.draft }).then(() => { app.draft = {}; render(); }),
      setCourse: (k) => { app.courseInd = k; render(); },
      reanchor: (k, revert) => void act(revert ? "reanchor_revert" : "reanchor", { day: st.openDay, key_txt: k }),
      shareCategory: (k, on) => void act("share_category", { key_txt: k, on: on ? 1 : 0 }),
      consent: (line, share) => void act("consent", { day: st.openDay, line, share: share ? 1 : 0 }),
      exclude: (k, on) => void act("exclude", { key_txt: k, on: on ? 1 : 0 }),
      link: (up) => void act("link", { up: up ? 1 : 0 }),
      request: (kind) => void act(kind, { day: st.openDay }),
      setScreen: (k, v) => { app.screen[k] = v; render(); },
      setVital: (k, v) => { app.vitals[k] = v; render(); },
      saveVitals: () => {
        // Untouched steppers save the value shown (the last measurement), so every field is explicit.
        const st2 = project(app.events);
        const vals: Record<string, number> = { day: st2.openDay };
        for (const k of V.VITALS) {
          const i = st2.indicators.get(k);
          if (!i) continue;
          vals[k] = app.vitals[k] ?? [...i.values.values()].at(-1)?.[0] ?? 0;
        }
        void act("sample", vals).then(() => { app.vitals = {}; render(); });
      },
      saveScreen: () => void act("screen", { day: st.openDay, ...app.screen }).then(() => { app.screen = {}; render(); }),
      go: (v) => { app.view = v; render(); },
    },
  };
  const view = { today: V.today, course: V.course, sharing: V.sharing, support: V.support }[app.view](ctx);
  const guide = app.mode === "replay" && app.guide && app.view === "today" ? guideCard() : null;
  const flash = app.flash.length ? h("section", { class: "flash", role: "status", "aria-live": "polite" }, ...app.flash.map((m) => card(m))) : null;
  const busy = app.busy ? h("p", { class: "busy", role: "status" }, glyph("info"), ` ${app.busy}…`) : null;
  root.replaceChildren(sidebar(), h("main", { id: "main" }, statusBar(st), h("div", { class: "content" }, busy ?? "", guide ?? "", flash ?? "", view)), replayBar(st) ?? "", tabBar());
  const main = document.querySelector("main");
  if (main) main.scrollTop = scroll;
}

window.addEventListener("error", (e) => { app.fatal = String(e.error ?? e.message); render(); });
window.addEventListener("unhandledrejection", (e) => { app.fatal = String(e.reason); render(); });
setInterval(() => { if (app.mode === "live") { rollover(); render(); } }, 60_000);

(async () => {
  render();
  void navigator.storage?.persist?.();
  const bytes = await (await fetch("anchor.wasm")).arrayBuffer();
  app.engine = await Engine.create(bytes);
  // Shareable replay links: #view=course&day=150&light=day&why=open (read-only replay state only).
  const p = new URLSearchParams(location.hash.slice(1));
  if (p.get("guide") === "off") app.guide = false;
  const view = p.get("view");
  if (view === "today" || view === "course" || view === "sharing" || view === "support") app.view = view;
  const light = p.get("light");
  if (light === "auto" || light === "day" || light === "evening") app.light = light;
  await startReplay();
  const day = Number(p.get("day"));
  if (Number.isInteger(day) && day >= 1 && day <= app.demoDays) app.scrub = day;
  render();
  if (p.get("why") === "open") document.querySelectorAll<HTMLDetailsElement>("details.why").forEach((d) => (d.open = true));
  document.documentElement.dataset.ready = "true";
})().catch((e) => { app.fatal = String(e); render(); });
