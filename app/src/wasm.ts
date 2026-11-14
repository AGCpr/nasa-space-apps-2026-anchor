// The WebAssembly boundary seen from the interface (ADR-0009 §1-3). The engine owns the log;
// this module only moves strings across linear memory and parses the JSON it returns.
import type { Event } from "./model.ts";

type Exports = {
  memory: WebAssembly.Memory;
  alloc(len: number): number;
  out_ptr(): number;
  load(ptr: number, len: number): number;
  submit(ptr: number, len: number): number;
  transmittable(): number;
  demo(): number;
  preflight(): number;
};

export type Reply = { ok: true; events: Event[] } | { ok: false; error: string };
export type Transmittable = { share: number; to: string; payload: string };

export class Engine {
  private readonly x: Exports;
  private constructor(x: Exports) {
    this.x = x;
  }

  static async create(bytes: BufferSource): Promise<Engine> {
    const { instance } = await WebAssembly.instantiate(bytes, {});
    return new Engine(instance.exports as unknown as Exports);
  }

  private read(len: number): unknown {
    const bytes = new Uint8Array(this.x.memory.buffer, this.x.out_ptr(), len);
    return JSON.parse(new TextDecoder().decode(bytes));
  }

  private call(fn: (ptr: number, len: number) => number, input: string): Reply {
    const enc = new TextEncoder().encode(input);
    const ptr = this.x.alloc(enc.length);
    new Uint8Array(this.x.memory.buffer, ptr, enc.length).set(enc);
    return this.read(fn(ptr, enc.length)) as Reply;
  }

  /** Rebuild from two stored copies (repaired, then replayed). */
  load(a: string, b: string): Reply {
    return this.call(this.x.load, `${a}\0${b}`);
  }

  /** `met|kind|k=v;...` in the log's field grammar; see `wire`. */
  submit(wire: string): Reply {
    return this.call(this.x.submit, wire);
  }

  transmittable(): Transmittable[] {
    return (this.read(this.x.transmittable()) as { items: Transmittable[] }).items;
  }

  demo(): Reply {
    return this.read(this.x.demo()) as Reply;
  }

  preflight(): Reply {
    return this.read(this.x.preflight()) as Reply;
  }
}

const hex = (s: string) => [...new TextEncoder().encode(s)].map((b) => b.toString(16).padStart(2, "0")).join("");

/** Encode a record in the log's field grammar: keys sorted, `_txt` values hex (ADR-0002). */
export function wire(met: number, kind: string, fields: Record<string, number | string>): string {
  const body = Object.keys(fields)
    .sort()
    .map((k) => {
      const v = fields[k]!;
      return `${k}=${k.endsWith("_txt") ? hex(String(v)) : Math.trunc(Number(v))}`;
    })
    .join(";");
  return `${Math.trunc(met)}|${kind}|${body}`;
}
