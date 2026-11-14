// Language audit (I3, I4; ADR-0009 §8) and boundary tests against the real WebAssembly engine.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { catalogues, isSilent, type Msg, type Slots } from "../src/catalogue.ts";
import { prefixEnd, type OutKind, type ShareClass } from "../src/model.ts";
import { Engine } from "../src/wasm.ts";

/** I3: disease and diagnostic names, alarm vocabulary, blame and pressure. Matched as word stems. */
const BANNED = [
  "depress", "anxi", "disorder", "burnout", "insomnia", "ptsd", "psychos", "psychiatr", "suicid", "illness",
  "disease", "sick", "symptom", "patient", "diagnos", "patholog", "abnormal", "alarm", "alert", "danger",
  "emergenc", "urgent", "critical", "crisis", "severe", "fail", "warning", "risk", "fault", "worse", "bad",
  "poor", "should", "must", "forgot", "missed", "neglect", "lazy", "wrong", "deficien", "decline",
];
const banned = (s: string) => BANNED.filter((b) => new RegExp(`\\b${b}`, "i").test(s));
const sentences = (s: string) => s.split(/(?<=[.?])\s+/).filter((x) => x.trim() !== "");

const KINDS: OutKind[] = [
  "Safety", "SafetyCheckFailed", "KernelUnavailable", "OfferExtraSample", "DayStat", "Alarm", "AlarmCleared",
  "MissingnessAlarm", "MissingnessCleared", "DriftPrompt", "AnchorRunPrompt", "Calibrating", "Recalibrated",
  "Rejected", "CompanionError", "PrivateSupport", "ConsentChanged", "Reanchored", "ReanchorReverted",
  "ShareQueued", "ShareSent", "ShareRerouted", "ShareUndeliverable", "ShareCancelled", "ExclusionChanged", "RouteEmpty",
];

function check(where: string, m: Msg) {
  const all = `${m.title} ${m.text} ${m.action} ${m.owner}`;
  assert.deepEqual(banned(all), [], `${where}: banned term in "${all}"`);
  assert.ok(!all.includes("!"), `${where}: exclamation mark`);
  assert.ok(m.action.trim().length > 0 && m.owner.trim().length > 0, `${where}: missing action or owner (I4)`);
  for (const s of sentences(m.text)) assert.ok(s.split(/\s+/).length <= 25, `${where}: sentence over 25 words: ${s}`);
  if (m.register === "warning") assert.ok(sentences(`${m.text} ${m.action}.`).length <= 3, `${where}: warning over 3 sentences`);
  assert.ok(!/undefined|null|NaN|\$\{/.test(all), `${where}: unfilled slot in "${all}"`);
}

for (const cat of Object.values(catalogues)) {
  test(`${cat.locale}: every template x slot value passes the audit (I3, I4)`, () => {
    const inds = [...Object.keys(cat.indicator), "unknown_key"];
    const tos = [...Object.keys(cat.recipient), null];
    const classes: ShareClass[] = ["safety", "summary", "ppc", "crew"];
    const items = [...Object.keys(cat.item), "other_item"];
    let n = 0;
    for (const kind of KINDS) {
      const t = cat.messages[kind];
      if (isSilent(t)) {
        assert.ok(t.silent.length > 0, `${kind}: silent without a reason`);
        continue;
      }
      for (const ind of inds) for (const to of tos) for (const cls of classes) for (const item of items)
        for (const dir of [1, -1] as const) for (const flag of [true, false]) for (const screen of [true, false]) {
          const s: Slots = { ind, dir, to, value: "88.4 %", n: 28, cls, item, share: flag, screen };
          check(`${kind}(${JSON.stringify(s)})`, t(s));
          n++;
        }
    }
    const rejected = cat.messages.Rejected;
    assert.ok(!isSilent(rejected));
    for (const r of Object.values(cat.reasons)) {
      check("reason", rejected({ ind: "mood", dir: -1, to: null, value: r, n: 0, cls: "safety", item: "", share: false, screen: false }));
    }
    for (const a of inds) for (const b of inds) for (const dir of [1, -1] as const) {
      check("group", cat.group([{ ind: a, since: 93, dir }, { ind: b, since: 104, dir: -1 }]));
      check("group3", cat.group([{ ind: a, since: 93, dir }, { ind: b, since: 102, dir }, { ind: "mood", since: 104, dir }]));
    }
    for (const ind of inds) {
      check("insight.inside", cat.insight.inside(ind, 7, 10));
      check("insight.outside", cat.insight.outside(ind, "lower", 0, 10));
      check("insight.early", cat.insight.early(ind));
    }
    const statics = [
      ...Object.values(cat.ui),
      ...Object.values(cat.indicator).flatMap((i) => [i.label, i.low, i.mid, i.high]),
      ...Object.values(cat.recipient),
      ...Object.values(cat.shareClass),
      ...Object.values(cat.item).flatMap((i) => [i.label, i.question]),
      ...Object.values(cat.area).map((a) => a.name),
      ...Object.values(cat.about).flatMap((a) => [a.how, a.space, a.helps]),
    ];
    for (const s of statics) {
      assert.deepEqual(banned(s), [], `static copy: "${s}"`);
      assert.ok(!s.includes("!"), `static copy: "${s}"`);
    }
    assert.ok(n > 10_000, `rendered ${n}`);
  });
}

/** Interface chrome outside the catalogue (views, controller): every string literal of two or more
 * words is user-facing copy and gets the same lexical checks (ADR-0009 §8). */
test("interface copy outside the catalogue passes the lexical audit (I3)", () => {
  let total = 0;
  for (const f of ["views.ts", "main.ts", "ui.ts"]) {
    const src = readFileSync(new URL(`../src/${f}`, import.meta.url), "utf8");
    const literals = [...src.matchAll(/"([^"\n]*)"|`([^`]*)`/g)].map((m) => (m[1] ?? m[2] ?? "").replace(/\$\{[^}]*\}/g, " "));
    const copy = literals.filter((l) => /[A-Za-z]+ [A-Za-z]+/.test(l) && !/^[a-z0-9 _.:=-]+$/.test(l.trim()));
    total += copy.length;
    for (const l of copy) {
      assert.deepEqual(banned(l), [], `${f}: "${l}"`);
      assert.ok(!l.includes("!"), `${f}: "${l}"`);
    }
  }
  assert.ok(total > 40, `scanned ${total} literals`);
});

const wasm = readFileSync(new URL("../anchor.wasm", import.meta.url));

test("every output kind the engine emits in the demo has a catalogue entry", async () => {
  const eng = await Engine.create(wasm);
  const r = eng.demo();
  assert.ok(r.ok);
  const emitted = new Set(r.events.flatMap((e) => e.out.map((o) => o.t)));
  for (const k of emitted) assert.ok(KINDS.includes(k), `unknown output kind ${k}`);
  assert.ok(emitted.size >= 14, [...emitted].join(","));
  // Every record kind the demo contains is valid JSON-mapped with its sealed line.
  for (const e of r.events) assert.match(e.line, /^v1\|\d+\|/);
});

test("a scrub prefix shows exactly what a fresh replay of that prefix produces (I5, ADR-0009 §5)", async () => {
  const eng = await Engine.create(wasm);
  const r = eng.demo();
  assert.ok(r.ok);
  for (const day of [1, 60, 121]) {
    const end = prefixEnd(r.events, day);
    const prefix = r.events.slice(0, end);
    const lines = prefix.map((e) => e.line).join("\n");
    const fresh = (await Engine.create(wasm)).load(lines, lines);
    assert.ok(fresh.ok);
    assert.deepEqual(fresh.events.map((e) => e.out), prefix.map((e) => e.out), `day ${day}`);
  }
});

test("the boundary rejects bad records without appending them", async () => {
  const eng = await Engine.create(wasm);
  const pre = eng.preflight();
  assert.ok(pre.ok);
  const bad = eng.submit("86400000|checkin|day=5;mood=500");
  assert.deepEqual(bad, { ok: false, error: "day is not the open day" });
  const good = eng.submit("86400000|checkin|day=1;mood=500");
  assert.ok(good.ok && good.events[0]!.seq === pre.events.length);
});

test("the prebuilt demo events equal a fresh engine run (I5; scripts/build-demo.ts)", async () => {
  const pre = JSON.parse(readFileSync(new URL("../demo-events.json", import.meta.url), "utf8"));
  const r = (await Engine.create(wasm)).demo();
  assert.ok(r.ok);
  assert.deepEqual(pre, JSON.parse(JSON.stringify(r.events)));
});
