// The interface as a projection of the event stream (ADR-0009 §5): every screen is computed
// from (record, outputs) pairs and nothing else, so scrubbing is a prefix of the stream.

export const DAY_MS = 86_400_000;

export type Out =
  | { t: "Safety"; day: number; indicator: string; value: number; hits: { id: number; share: boolean }[] }
  | { t: "SafetyCheckFailed"; day: number; indicator: string; value: number }
  | { t: "KernelUnavailable"; day: number; indicator: string; value: number }
  | { t: "OfferExtraSample"; day: number; indicator: string }
  | { t: "DayStat"; day: number; indicator: string; z: number | null; statistic: number; y: number; lo: number; hi: number; anchor: number }
  | { t: "Alarm"; day: number; indicator: string; statistic: number; h: number }
  | { t: "AlarmCleared"; day: number; indicator: string }
  | { t: "MissingnessAlarm"; day: number }
  | { t: "MissingnessCleared"; day: number }
  | { t: "DriftPrompt"; day: number; indicator: string; n: number; sen_slope: number; z: number }
  | { t: "AnchorRunPrompt"; day: number; indicator: string }
  | { t: "Calibrating"; day: number }
  | { t: "Recalibrated"; day: number; h: number; arl0: number; seed: string; miss_h: number | null }
  | { t: "Rejected"; seq: number; reason: string }
  | { t: "CompanionError"; day: number; reason: string }
  | { t: "PrivateSupport"; day: number; items: string[] }
  | { t: "ConsentChanged"; day: number; line: number; share: boolean; table: string; digest: string }
  | { t: "Reanchored"; day: number; indicator: string; from: number; to: number }
  | { t: "ReanchorReverted"; day: number; indicator: string }
  | { t: "ShareQueued"; share: number; class: ShareClass; to: string | null; payload: string }
  | { t: "ShareSent"; share: number; to: string }
  | { t: "ShareRerouted"; share: number; to: string }
  | { t: "ShareUndeliverable"; share: number; class: ShareClass }
  | { t: "ShareCancelled"; share: number }
  | { t: "ExclusionChanged"; recipient: string; excluded: boolean }
  | { t: "RouteEmpty"; class: ShareClass };

export type OutKind = Out["t"];
export type ShareClass = "safety" | "summary" | "ppc" | "crew";

export type Event = {
  seq: number;
  met: number;
  kind: string;
  f: Record<string, number | string>;
  line: string;
  out: Out[];
};

export type Indicator = {
  key: string;
  dir: 1 | -1;
  slider: boolean;
  decimals: number;
  stats: Extract<Out, { t: "DayStat" }>[];
  /** Raw record values per day (record units, e.g. 0..1000 or tenths of a percent). */
  values: Map<number, number[]>;
  alarmDay: number | null;
  shared: boolean;
  hinges: { day: number; from: number; to: number; reverted: number | null }[];
  safetyDays: number[];
};

export type Share = {
  id: number;
  cls: ShareClass;
  to: string | null;
  payload: Record<string, number | string>;
  day: number;
  status: "waiting" | "sent" | "cancelled" | "undeliverable";
  history: Out[];
};

export type TableLine = { id: number; indicator: string; op: "above" | "below"; limit: number; high: boolean; share: boolean };

export type Note = { seq: number; kind: string; day: number; out: Out };

export type State = {
  lastSeq: number;
  met: number;
  lastClosed: number;
  openDay: number;
  indicators: Map<string, Indicator>;
  table: { text: string; digest: string; lines: TableLine[] } | null;
  recipients: Map<string, { crew: boolean; excluded: boolean }>;
  routes: Map<ShareClass, string[]>;
  linkUp: boolean;
  shares: Map<number, Share>;
  checkinDays: Set<number>;
  /** Everything the astronaut may be told, oldest first. */
  notes: Note[];
  missingness: boolean;
  /** Seq of today's value records with their kernel-A outcome, for the kernel-B comparison. */
  todayValues: { seq: number; key: string; value: number; screen: boolean }[];
  screened: number[];
  calibrated: boolean;
  /** Latest calibrated threshold, for the plain-language evidence level. */
  h: number | null;
  links: { day: number; up: boolean }[];
};

export function parseTable(text: string): TableLine[] {
  const body = text.slice(3);
  if (body === "") return [];
  return body.split(";").map((item) => {
    const [id, indicator, op, limit, cls, consent] = item.split(",");
    return { id: Number(id), indicator: indicator!, op: op as "above" | "below", limit: Number(limit), high: cls === "high", share: consent === "1" };
  });
}

const dayOf = (e: Event) => (typeof e.f.day === "number" ? e.f.day : Math.floor(e.met / DAY_MS));

function parsePayload(p: string): Record<string, number | string> {
  return Object.fromEntries(p.split(";").map((kv) => {
    const [k, v] = kv.split("=") as [string, string];
    return [k, /^-?\d+$/.test(v) ? Number(v) : v];
  }));
}

export function project(events: readonly Event[]): State {
  const s: State = {
    lastSeq: -1,
    met: 0,
    lastClosed: 0,
    openDay: 1,
    indicators: new Map(),
    table: null,
    recipients: new Map(),
    routes: new Map(),
    linkUp: false,
    shares: new Map(),
    checkinDays: new Set(),
    notes: [],
    missingness: false,
    todayValues: [],
    screened: [],
    calibrated: false,
    h: null,
    links: [],
  };
  for (const e of events) {
    s.lastSeq = e.seq;
    s.met = e.met;
    const f = e.f;
    const day = dayOf(e);
    const ind = (k: string) => s.indicators.get(k);
    const rejected = e.out.some((o) => o.t === "Rejected");
    if (!rejected) {
      switch (e.kind) {
        case "indicator":
          s.indicators.set(String(f.key_txt), {
            key: String(f.key_txt),
            dir: f.dir === 1 ? 1 : -1,
            slider: f.transform === 0,
            decimals: Number(f.decimals),
            stats: [],
            values: new Map(),
            alarmDay: null,
            shared: false,
            hinges: [],
            safetyDays: [],
          });
          break;
        case "safety_table":
          s.table = { text: String(f.table_txt), digest: String(f.digest_txt), lines: parseTable(String(f.table_txt)) };
          break;
        case "recipient":
          s.recipients.set(String(f.key_txt), { crew: f.crew === 1, excluded: false });
          break;
        case "route": {
          const c = String(f.class_txt) as ShareClass;
          s.routes.set(c, [...(s.routes.get(c) ?? []), String(f.to_txt)]);
          break;
        }
        case "link":
          s.linkUp = f.up === 1;
          s.links.push({ day, up: s.linkUp });
          break;
        case "exclude": {
          const r = s.recipients.get(String(f.key_txt));
          if (r) r.excluded = f.on === 1;
          break;
        }
        case "share_category": {
          const i = ind(String(f.key_txt));
          if (i) i.shared = f.on === 1;
          break;
        }
        case "checkin":
        case "sample":
        case "screen":
          if (e.kind === "checkin") s.checkinDays.add(day);
          if (e.kind === "screen") s.screened.push(day);
          for (const [k, v] of Object.entries(f)) {
            if (k === "day" || typeof v !== "number") continue;
            s.todayValues.push({ seq: e.seq, key: k, value: v, screen: e.kind === "screen" });
            const i = ind(k);
            if (i) i.values.set(day, [...(i.values.get(day) ?? []), v]);
          }
          break;
        case "day_close":
          s.lastClosed = day;
          s.openDay = day + 1;
          s.todayValues = [];
          break;
      }
    }
    for (const o of e.out) {
      let note = true;
      switch (o.t) {
        case "DayStat":
          ind(o.indicator)?.stats.push(o);
          note = false;
          break;
        case "Alarm":
          { const i = ind(o.indicator); if (i) i.alarmDay = o.day; }
          break;
        case "AlarmCleared":
          { const i = ind(o.indicator); if (i) i.alarmDay = null; }
          break;
        case "MissingnessAlarm":
          s.missingness = true;
          break;
        case "MissingnessCleared":
          s.missingness = false;
          break;
        case "Safety":
          ind(o.indicator)?.safetyDays.push(o.day);
          break;
        case "ConsentChanged":
          s.table = { text: o.table, digest: o.digest, lines: parseTable(o.table) };
          break;
        case "Reanchored":
          ind(o.indicator)?.hinges.push({ day: o.day, from: o.from, to: o.to, reverted: null });
          break;
        case "ReanchorReverted": {
          const h = ind(o.indicator)?.hinges.filter((x) => x.reverted === null).at(-1);
          if (h) h.reverted = o.day;
          break;
        }
        case "Recalibrated":
          s.calibrated = true;
          s.h = o.h;
          note = false;
          break;
        case "Calibrating":
          note = !s.calibrated;
          break;
        case "ShareQueued": {
          const payload = parsePayload(o.payload);
          s.shares.set(o.share, {
            id: o.share,
            cls: o.class,
            to: o.to,
            payload,
            day: typeof payload.day === "number" ? payload.day : day,
            status: o.to === null ? "undeliverable" : "waiting",
            history: [o],
          });
          break;
        }
        case "ShareSent":
        case "ShareRerouted":
        case "ShareUndeliverable":
        case "ShareCancelled": {
          const sh = s.shares.get(o.share);
          if (!sh) break;
          sh.history.push(o);
          if (o.t === "ShareSent") sh.status = "sent";
          if (o.t === "ShareRerouted") { sh.to = o.to; sh.status = "waiting"; }
          if (o.t === "ShareUndeliverable") { sh.to = null; sh.status = "undeliverable"; }
          if (o.t === "ShareCancelled") sh.status = "cancelled";
          note = o.t !== "ShareSent";
          break;
        }
      }
      if (note) s.notes.push({ seq: e.seq, kind: e.kind, day: "day" in o ? o.day : day, out: o });
    }
  }
  return s;
}

/** Index just past the `day_close` of `day`: the stream prefix a scrub to that day shows. */
export function prefixEnd(events: readonly Event[], day: number): number {
  let end = events.length;
  for (let i = 0; i < events.length; i++) {
    const e = events[i]!;
    if (e.kind === "day_close" && e.f.day === day) {
      end = i + 1;
      // Acknowledgements written at the close belong to that day too.
      while (end < events.length && events[end]!.kind === "sent" && events[end]!.met === e.met) end++;
      break;
    }
  }
  return end;
}

/** Record-unit value to display units (tenths of a slider point -> 0..100; scaled by decimals). */
export function shown(i: Indicator, v: number): number {
  return i.slider ? v / 10 : v / 10 ** i.decimals;
}

/** Kernel A's outcome for one value, read from its record's outputs (ADR-0009 §7). */
export function kernelA(out: readonly Out[], key: string): { kind: "hits"; hits: { id: number; share: boolean }[] } | { kind: "check_failed" } {
  for (const o of out) {
    if ((o.t === "SafetyCheckFailed" || o.t === "KernelUnavailable") && o.indicator === key) return { kind: "check_failed" };
    if (o.t === "Safety" && o.indicator === key) return { kind: "hits", hits: o.hits };
  }
  return { kind: "hits", hits: [] };
}
