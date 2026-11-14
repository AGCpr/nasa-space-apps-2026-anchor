// R1, R3 and R10 as tests (ADR-0009 §9): the tokens checked are the tokens rendered.
import { test } from "node:test";
import assert from "node:assert/strict";
import { contrast, day, evening, hue, rgb, type Tokens } from "../src/tokens.ts";

const sat = (h: string) => {
  const c = rgb(h);
  const max = Math.max(...c);
  return max === 0 ? 0 : (max - Math.min(...c)) / max;
};
const textPairs: [keyof Tokens, keyof Tokens][] = [
  ["text", "ground"], ["text", "surface"], ["muted", "ground"], ["muted", "surface"], ["trace", "ground"],
  ["onAccent", "accent"], ["onSignal", "caution"], ["onSignal", "warning"],
  ["caution", "ground"], ["warning", "ground"], ["advisory", "ground"], ["caution", "surface"], ["warning", "surface"],
];

for (const [name, t] of [["day", day], ["evening", evening]] as const) {
  test(`${name}: every text pair reaches 6:1 (R3)`, () => {
    for (const [fg, bg] of textPairs) {
      const c = contrast(t[fg], t[bg]);
      assert.ok(c >= 6, `${fg} on ${bg}: ${c.toFixed(2)}`);
    }
  });
  test(`${name}: only the advisory token is blue (R1)`, () => {
    for (const [k, v] of Object.entries(t)) {
      const blue = sat(v) >= 0.25 && hue(v) >= 180 && hue(v) <= 260;
      assert.ok(!blue || k === "advisory", `${k} ${v} is blue`);
    }
  });
}

test("evening: no token bluer than a 2700 K source (R10)", () => {
  for (const [k, v] of Object.entries(evening)) {
    const [r, g, b] = rgb(v);
    assert.ok(b <= 0.35 * Math.max(r, g, b), `${k} ${v}`);
  }
});

test("day: decorative hues keep 15 degrees from caution and warning (R1)", () => {
  const signals = [hue(day.caution), hue(day.warning)];
  for (const k of ["contour", "corridor", "accent", "muted", "text", "trace", "line", "surface", "ground"] as const) {
    if (sat(day[k]) < 0.25) continue;
    for (const s of signals) {
      const d = Math.min(Math.abs(hue(day[k]) - s), 360 - Math.abs(hue(day[k]) - s));
      assert.ok(d >= 15, `${k} ${day[k]} is ${d.toFixed(0)} degrees from a signal colour`);
    }
  }
});

test("contour ochre is never a text colour (tokens.ts: lines only)", async () => {
  const { readFileSync } = await import("node:fs");
  const css = readFileSync(new URL("../styles.css", import.meta.url), "utf8");
  const misuse = css.split("}").filter((rule) => /(^|[;{\s])color:\s*var\(--contour\)/.test(rule) || (/text|label/.test(rule.split("{")[0] ?? "") && /fill:\s*var\(--contour\)/.test(rule)));
  assert.deepEqual(misuse, []);
});
