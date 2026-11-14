export class Engine {
    x;
    constructor(x) {
        this.x = x;
    }
    static async create(bytes) {
        const { instance } = await WebAssembly.instantiate(bytes, {});
        return new Engine(instance.exports);
    }
    read(len) {
        const bytes = new Uint8Array(this.x.memory.buffer, this.x.out_ptr(), len);
        return JSON.parse(new TextDecoder().decode(bytes));
    }
    call(fn, input) {
        const enc = new TextEncoder().encode(input);
        const ptr = this.x.alloc(enc.length);
        new Uint8Array(this.x.memory.buffer, ptr, enc.length).set(enc);
        return this.read(fn(ptr, enc.length));
    }
    /** Rebuild from two stored copies (repaired, then replayed). */
    load(a, b) {
        return this.call(this.x.load, `${a}\0${b}`);
    }
    /** `met|kind|k=v;...` in the log's field grammar; see `wire`. */
    submit(wire) {
        return this.call(this.x.submit, wire);
    }
    transmittable() {
        return this.read(this.x.transmittable()).items;
    }
    demo() {
        return this.read(this.x.demo());
    }
    preflight() {
        return this.read(this.x.preflight());
    }
}
const hex = (s) => [...new TextEncoder().encode(s)].map((b) => b.toString(16).padStart(2, "0")).join("");
/** Encode a record in the log's field grammar: keys sorted, `_txt` values hex (ADR-0002). */
export function wire(met, kind, fields) {
    const body = Object.keys(fields)
        .sort()
        .map((k) => {
        const v = fields[k];
        return `${k}=${k.endsWith("_txt") ? hex(String(v)) : Math.trunc(Number(v))}`;
    })
        .join(";");
    return `${Math.trunc(met)}|${kind}|${body}`;
}
