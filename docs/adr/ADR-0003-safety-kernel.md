# ADR-0003: Safety kernel — specification, N-version, TMR

**Status:** accepted · 10 Nov 2026

## Context

Safety lines are the only safety-critical function (NASA-STD-8739.8 sense). Invariants:
- **I1:** a crossed line always yields a safety output.
- **I8:** high-stakes auto-share only with per-item pre-consent.
- **I6:** a share is always logged before transmission.

The kernel must not depend on any companion component.

## Specification (the single source both implementations are written against)

**Safety line**

| Field | Type / meaning |
|---|---|
| `id` | `u16` |
| `indicator` | ASCII key, same grammar as log keys |
| `op` | `Above` (value > limit) or `Below` (value < limit); strict comparison |
| `limit` | `i64`, fixed-point in the indicator's unit |
| `class` | `Physiological` or `HighStakes` |
| `auto_share_consented` | `bool`; meaningful only for `HighStakes`; `Physiological` lines always auto-share, because the astronaut agreed to the table as a whole before flight |

**Table:** at most 64 lines (bounded); `id` unique.

**Input:** `(indicator, value)`.

**Output:** a list, sorted by `id`, of every line whose indicator matches and whose comparison holds. Each element is `(id, share)`, where `share` is:
- `true` for `Physiological`;
- `auto_share_consented` for `HighStakes`.

Empty list = no line crossed.

**Fail-safe:** if the evaluation cannot be trusted, the output is `CheckFailed`. The UI must render `CheckFailed` as "re-measure / contact your chosen crew contact", never as "fine".

## Debated design points

1. **Strict vs inclusive comparison.**
   - Clinical limits are conventionally written as "below 90 %".
   - The two operators `Above` and `Below` are strict, and inclusive limits are expressed by shifting the fixed-point limit by one unit.
   - **Why:** one comparison semantics, no `>=`/`>` mix-ups between implementations.
2. **TMR placement.**
   - **Alternatives:**
     - (a) run the whole kernel three times on one table: masks transient CPU faults, but not table corruption;
     - (b) three CRC-protected table copies plus a vote.
   - **Chosen:** three independently stored table copies, each with a SHA-256 digest:
     - verify each digest;
     - evaluate with every copy that verifies;
     - majority-vote the outputs.
   - Outcomes:
     - ≥ 2 agreeing evaluations ⇒ that output;
     - otherwise ⇒ `CheckFailed`.
   - A copy whose digest fails is reported, so it can be repaired from the others.
3. **N-version.**
   - Kernel A is Rust (`core/src/kernel.rs`); kernel B is TypeScript (`kernel-b/`).
   - Written separately from this specification.
   - At runtime the UI runs both. Disagreement ⇒ `CheckFailed`, and the event is logged.
   - Offline, both are run against the same exhaustive vector file.
4. **Exhaustive verification.**
   - Input-space reduction: for each line only the values `limit−1`, `limit`, `limit+1`, `i64::MIN`, `i64::MAX` and 0 can change the outcome; for every other value the behaviour is determined by which interval it falls in.
   - The vector generator therefore enumerates, for each indicator, the union of all critical points of all its lines, plus one point inside each interval between them. This is exhaustive over the equivalence classes of the input space, not a sample.
4b. **Why not formal proof tools now:** model checkers (e.g. Kani for Rust) would add assurance. They are recorded as future work, because equivalence-class exhaustion already covers this finite, loop-free specification completely.

## Status of verification (10 Nov 2026)

- Three implementations — kernel A (Rust), kernel B (TypeScript, BigInt) and the Python reference — agree on:
  - all 2,147 equivalence-class cases across 28 tables (3 hand-built, including i64 extremes and the empty table, plus 25 seeded random tables of up to 64 lines);
  - all 13 invalid tables.
- SHA-256 of every table agrees across Python `hashlib`, Rust `sha2` and WebCrypto.
- **Known limitation of this N-version claim:** all three implementations currently share one author. Language, runtime and integer-model diversity remove compiler-, library- and representation-level common modes. They do not remove a shared misreading of the specification. Before any operational use, kernel B must be re-implemented by a different person from the specification alone.

## Consequences

- The kernel has no allocation beyond the bounded output, no recursion, no I/O and no clock reads.
- Each function asserts its preconditions. The table loader rejects invalid tables (duplicate ids, > 64 lines, invalid keys) at load time, never at evaluation.
