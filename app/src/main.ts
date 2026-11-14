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
    const r = app.engine!.demo();
    if (!r.ok) throw new Error(r.error);
    app.demo = r.events;
    app.demoDays = project(r.events).lastClosed;
    await auditHistory(app.demo);
    app.busy = "";
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

function top(st: State): HTMLElement {
  const shownDay = app.mode === "replay" ? st.lastClosed : st.openDay;
  return h(
    "header",
    { class: "top" },
    h("div", { class: "top__mark" }, h("span", { class: "wordmark" }, "ANCHOR"), h("span", { class: "marg" }, "Personal survey")),
    h("div", { class: "top__cell" }, h("span", { class: "marg" }, "Mission day"), h("strong", { class: "num" }, String(shownDay))),
    h("div", { class: `top__cell link link--${st.linkUp ? "up" : "down"}` }, h("span", { class: "marg" }, "Ground link"), h("strong", {}, st.linkUp ? "Up" : "Loss of signal")),
    app.mode === "live"
      ? h("div", { class: "top__cell badge" }, h("span", { class: "marg" }, "Mode"), h("strong", {}, "Live · synthetic pre-flight"))
      : h("div", { class: "top__cell badge badge--synthetic" }, h("span", { class: "marg" }, "Mode"), h("strong", {}, c.ui.syntheticReplay)),
    h("div", { class: "top__end" },
      segmented("Lighting", [{ value: "auto", label: c.ui.lightAuto }, { value: "day", label: c.ui.lightDay }, { value: "evening", label: c.ui.lightEvening }], app.light, (v) => { app.light = v; render(); }),
      segmented("Data", [{ value: "replay", label: "Replay" }, { value: "live", label: "Live" }], app.mode === "live" ? "live" : "replay", (v) => void (v === "live" ? startLive() : startReplay()))),
  );
}

const RAIL: { v: V.View; label: string; d: string }[] = [
  { v: "today", label: c.ui.today, d: "M12 4a8 8 0 1 0 0 16a8 8 0 1 0 0-16Zm0 4v4l3 2" },
  { v: "course", label: c.ui.course, d: "M3 17c4-6 6 2 9-4s5-2 9-6M3 21h18" },
  { v: "sharing", label: c.ui.sharing, d: "M6 12h12M14 7l5 5-5 5M4 4v16" },
  { v: "support", label: c.ui.support, d: "M12 20s-7-4.5-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.5-7 10-7 10Z" },
];

function rail(): HTMLElement {
  return h("nav", { class: "rail", "aria-label": "Sections" }, ...RAIL.map((r) => {
    const icon = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    icon.setAttribute("viewBox", "0 0 24 24");
    icon.setAttribute("aria-hidden", "true");
    icon.innerHTML = `<path d="${r.d}"/>`;
    return h("button", { type: "button", "aria-current": app.view === r.v ? "page" : null, onclick: () => { app.view = r.v; app.flash = []; render(); document.querySelector("main")?.scrollTo(0, 0); } }, icon, h("span", {}, r.label));
  }));
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

/** Inner chart width: viewport minus rail, main padding and sheet padding (styles.css). */
function sheetWidth(): number {
  const rail = innerWidth <= 600 ? 0 : innerWidth <= 900 ? 104 : 136; // phone: rail moves to the top
  return Math.max(300, Math.min(1120, innerWidth - rail - (innerWidth <= 900 ? 40 : 80)) - 26);
}

let resizeTimer: number | undefined;
window.addEventListener("resize", () => { clearTimeout(resizeTimer); resizeTimer = window.setTimeout(render, 120); });

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
      saveScreen: () => void act("screen", { day: st.openDay, ...app.screen }).then(() => { app.screen = {}; render(); }),
      go: (v) => { app.view = v; render(); },
    },
  };
  const view = { today: V.today, course: V.course, sharing: V.sharing, support: V.support }[app.view](ctx);
  const flash = app.flash.length ? h("section", { class: "flash", role: "status", "aria-live": "polite" }, ...app.flash.map((m) => card(m))) : null;
  const busy = app.busy ? h("p", { class: "busy", role: "status" }, glyph("info"), ` ${app.busy}…`) : null;
  root.replaceChildren(top(st), rail(), h("main", { id: "main" }, busy ?? "", flash ?? "", view), replayBar(st) ?? "");
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
  await startReplay();
})().catch((e) => { app.fatal = String(e); render(); });
