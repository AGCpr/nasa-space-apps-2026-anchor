import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { compareKernels, evaluate, parseTable, tmrEvaluate, type Hit, type SafetyLine } from "../src/kernel.ts";

const VECTORS = readFileSync(new URL("../../oracle/vectors/kernel_vectors.txt", import.meta.url), "ascii");

const digest = async (s: string) =>
  [...new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(s)))]
    .map((b) => b.toString(16).padStart(2, "0")).join("");

const expected = (s: string): Hit[] =>
  s === "none" ? [] : s.split(",").map((h) => { const [id, sh] = h.split(":"); return { id: Number(id), share: sh === "1" }; });

test("kernel B agrees with the reference on every equivalence-class vector", async () => {
  let table: readonly SafetyLine[] = [];
  let tableText = "";
  let cases = 0;
  for (const line of VECTORS.split("\n")) {
    if (line === "" || line.startsWith("#")) continue;
    const [tag, ...rest] = line.split("\t");
    if (tag === "table") { table = parseTable(rest[0]!); tableText = rest[0]!; }
    else if (tag === "digest") assert.equal(await digest(tableText), rest[0], "WebCrypto SHA-256 differs from Python hashlib");
    else if (tag === "case") {
      assert.deepEqual(evaluate(table, rest[0]!, BigInt(rest[1]!)), expected(rest[2]!), line);
      cases++;
    } else if (tag === "reject") assert.throws(() => parseTable(rest[0]!), line);
    else assert.fail(`unknown tag ${tag}`);
  }
  assert.ok(cases > 1000, `too few cases: ${cases}`);
});

const T = "t1|1,spo2,below,900,phys,1;2,cssrs_plan,above,0,high,0;3,cssrs_intent,above,0,high,1";

test("TMR masks one damaged copy and fails safe on two", async () => {
  const d = await digest(T);
  const bad = T.replace("900", "800");
  const r1 = await tmrEvaluate([T, bad, T], d, "spo2", 850n);
  assert.deepEqual(r1, { outcome: { kind: "hits", hits: [{ id: 1, share: true }] }, damaged: [1] });
  const r2 = await tmrEvaluate([bad, bad, T], d, "spo2", 850n);
  assert.equal(r2.outcome.kind, "check_failed");
});

test("kernel disagreement is never resolved in favour of 'fine'", () => {
  const hit = { kind: "hits" as const, hits: [{ id: 1, share: true }] };
  const none = { kind: "hits" as const, hits: [] };
  assert.equal(compareKernels(hit, none).kind, "check_failed");
  assert.equal(compareKernels(none, hit).kind, "check_failed");
  assert.deepEqual(compareKernels(hit, hit), hit);
});

test("invalid indicator input fails safe", async () => {
  assert.equal((await tmrEvaluate([T, T, T], await digest(T), "SpO2", 1n)).outcome.kind, "check_failed");
});
