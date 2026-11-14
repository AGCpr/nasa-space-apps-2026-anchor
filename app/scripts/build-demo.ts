// Build step (ADR-0009 §5, amended): run the demo mission through the real WebAssembly engine once
// and store its events, so the browser does not redo 22 Monte-Carlo calibrations on page load.
// The interface stays a projection of engine outputs (I5); app/test/audit.test.ts checks that this
// file equals a fresh engine run. Usage: node app/scripts/build-demo.ts
import { readFileSync, writeFileSync } from "node:fs";
import { Engine } from "../src/wasm.ts";

const wasm = readFileSync(new URL("../anchor.wasm", import.meta.url));
const r = (await Engine.create(wasm)).demo();
if (!r.ok) throw new Error(r.error);
writeFileSync(new URL("../demo-events.json", import.meta.url), JSON.stringify(r.events));
console.log(`demo-events.json: ${r.events.length} events`);
