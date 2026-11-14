// Safety kernel B (ADR-0003). Independent of kernel A: different language, runtime and
// integer model (BigInt). Written against the ADR specification only.

export const MAX_LINES = 64;
const I64_MIN = -(2n ** 63n);
const I64_MAX = 2n ** 63n - 1n;
const KEY_RE = /^[a-z0-9_]{1,32}$/;
const INT_RE = /^(0|-?[1-9][0-9]*)$/;

export type SafetyLine = Readonly<{
  id: number;
  indicator: string;
  op: "above" | "below";
  limit: bigint;
  share: boolean; // phys lines always true; high lines = per-item consent
}>;

export type Hit = Readonly<{ id: number; share: boolean }>;
export type Outcome = { kind: "hits"; hits: Hit[] } | { kind: "check_failed" };

export class TableError extends Error {}

function canonicalInt(s: string): bigint {
  if (!INT_RE.test(s)) throw new TableError(`not canonical integer: ${s}`);
  const v = BigInt(s);
  if (v < I64_MIN || v > I64_MAX) throw new TableError(`out of i64 range: ${s}`);
  return v;
}

export function parseTable(s: string): readonly SafetyLine[] {
  if (!s.startsWith("t1|")) throw new TableError("missing t1| header");
  const body = s.slice(3);
  if (body === "") return [];
  const lines: SafetyLine[] = [];
  for (const item of body.split(";")) {
    const f = item.split(",");
    if (f.length !== 6) throw new TableError("line needs 6 fields");
    const id = canonicalInt(f[0]!);
    if (id < 0n || id > 65535n) throw new TableError("id out of u16 range");
    const prev = lines.at(-1);
    if (prev !== undefined && BigInt(prev.id) >= id) throw new TableError("ids not strictly increasing");
    const indicator = f[1]!;
    if (!KEY_RE.test(indicator)) throw new TableError("invalid indicator");
    const op = f[2];
    if (op !== "above" && op !== "below") throw new TableError("invalid op");
    const limit = canonicalInt(f[3]!);
    const cc = `${f[4]},${f[5]}`;
    const share = cc === "phys,1" ? true : cc === "high,1" ? true : cc === "high,0" ? false : null;
    if (share === null) throw new TableError("invalid class/consent");
    lines.push(Object.freeze({ id: Number(id), indicator, op, limit, share }));
    if (lines.length > MAX_LINES) throw new TableError("more than 64 lines");
  }
  return Object.freeze(lines);
}

export function evaluate(table: readonly SafetyLine[], indicator: string, value: bigint): Hit[] {
  if (value < I64_MIN || value > I64_MAX) throw new RangeError("value outside i64");
  const hits: Hit[] = [];
  for (const l of table) {
    if (l.indicator !== indicator) continue;
    const crossed = l.op === "above" ? value > l.limit : value < l.limit;
    if (crossed) hits.push({ id: l.id, share: l.share });
  }
  return hits;
}

async function sha256Hex(s: string): Promise<string> {
  const d = await globalThis.crypto.subtle.digest("SHA-256", new TextEncoder().encode(s));
  return [...new Uint8Array(d)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

const sameHits = (a: Hit[], b: Hit[]) =>
  a.length === b.length && a.every((h, i) => h.id === b[i]!.id && h.share === b[i]!.share);

/** TMR over three stored copies; a copy counts only if its digest matches and it parses. */
export async function tmrEvaluate(
  copies: readonly [string, string, string],
  expectedDigest: string,
  indicator: string,
  value: bigint,
): Promise<{ outcome: Outcome; damaged: number[] }> {
  if (!KEY_RE.test(indicator)) return { outcome: { kind: "check_failed" }, damaged: [] };
  const results: Hit[][] = [];
  const damaged: number[] = [];
  for (const [i, c] of copies.entries()) {
    try {
      if ((await sha256Hex(c)) !== expectedDigest) throw new TableError("digest mismatch");
      results.push(evaluate(parseTable(c), indicator, value));
    } catch {
      damaged.push(i);
    }
  }
  const agreed = results.find((r) => results.filter((o) => sameHits(o, r)).length >= 2);
  return { outcome: agreed ? { kind: "hits", hits: agreed } : { kind: "check_failed" }, damaged };
}

/** N-version comparison: any disagreement between kernels must be treated as CheckFailed. */
export function compareKernels(a: Outcome, b: Outcome): Outcome {
  if (a.kind === "hits" && b.kind === "hits" && sameHits(a.hits, b.hits)) return a;
  return { kind: "check_failed" };
}
