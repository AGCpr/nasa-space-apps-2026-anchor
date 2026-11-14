# ADR-0002: Event log format and integrity

**Status:** accepted · 10 Nov 2026

## Context

The log is the single source of truth (DESIGN_PAPER §5.3); everything else is recomputed from it. Threats it must withstand:
- crash during a write;
- single-event upsets (bit flips) on COTS storage and memory, which have no assumed ECC;
- tampering;
- divergence between two stored copies.

It must also be verifiable from TypeScript (UI and kernel B) and from Python (oracle).

## Decisions and debated alternatives

### 1. Values are integers in fixed-point units, never floats
- **Alternative:** floats with canonical formatting (RFC 8785 JCS).
- **Rejected because:** float-to-text canonicalisation differs across languages (shortest round-trip printing), which makes cross-language hashing fragile. It also gives a safety threshold comparison a representation-dependent edge.
- **Chosen:** every measurement is an `i64` in a declared unit with a declared scale. Example: SpO₂ in 0.1 % units (`975` = 97.5 %); a slider in 0.1 points (`0..=1000`).
- **Consequence:** exact comparison and exact hashing everywhere.

### 2. Canonical encoding: strict ASCII line grammar
- **Alternatives:** CBOR or a custom binary format (compact, but opaque and needing a codec in three languages); JSON (ambiguous key order and number forms).
- **Chosen:** one record per line:
  `v1|<seq>|<met_ms>|<kind>|<k1>=<v1>;<k2>=<v2>…|<prev_hash>|<hash>`
  - Keys are sorted, lowercase `[a-z0-9_]`.
  - Values are canonical signed decimal integers: no leading zeros, no `+` sign, no `-0`.
  - Text (journal) is allowed only under keys ending in `_txt`, and is stored as lowercase hex of its UTF-8 bytes. The key itself therefore says how to read the value, and no separate type tag is needed.
  - `kind` is lowercase `[a-z_]`.
  - Any deviation is a parse error, never a silent repair.
- **Why:** trivial to implement identically in three languages, human-inspectable, and with exactly one valid encoding per record.

### 3. Integrity: SHA-256 hash chain, no separate CRC
- The DESIGN_PAPER (v1–v4) specified CRC32 **plus** a hash chain. On debate, the CRC is redundant:
  - The per-record SHA-256 already detects any corruption of that record.
  - Detection can be done per record without recomputing the whole chain.
  - A CRC adds a second mechanism to implement three times, and adds no detection power.
- **Chosen:** `hash = SHA-256(canonical body ‖ prev_hash)`, lowercase hex. Genesis `prev_hash` = 64 zeros.
- **Why SHA-256 rather than BLAKE3:** it is FIPS 180-4 standardised and native in browsers (WebCrypto), so the TypeScript side verifies with no dependency. Speed is irrelevant at ~10 records per day.
- **Not claimed:** the chain is not a signature, so an attacker who rewrites the whole chain is not detected by it. Tamper resistance against a capable adversary comes from encryption at rest and from signing the safety table (later ADR).

### 4. Atomicity
- Appends are validated (sequence = last + 1, `met_ms` non-decreasing, hash correct) **before** being accepted into memory.
- The persistence adapters (later) use write-ahead + commit.
- A truncated final line fails parsing and is reported as an incomplete write, not as corruption of earlier history.

### 5. Two copies and repair
- `verify(copy)` returns the first invalid index, or `Ok`.
- `repair(a, b)`:
  - takes the longest prefix on which both copies are valid and identical;
  - after a divergence, prefers the copy whose remaining chain verifies;
  - if both are corrupt at the same index, it returns an explicit `Unrecoverable(index)` and **never guesses**.

## Consequences

- The model layer converts fixed-point to `f64` only at computation time, so derived state never feeds back into the log.
- The same test vectors (`oracle/vectors/log_*.txt`) must pass in Rust, Python and TypeScript.
