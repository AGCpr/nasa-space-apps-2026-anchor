#!/usr/bin/env bash
# One command to re-derive every reference vector and run every implementation against it.
# Usage: bash verify.sh [--full]   (--full adds the ~1 min end-to-end false-alert budget test)
set -euo pipefail
cd "$(dirname "$0")"

echo "== oracles (Python reference implementations, scipy)"
# First interpreter that has numpy and scipy (the default `py` may be a bare install).
PY=""
for cand in ${ANCHOR_PY:-} "py" "py -3.11" "py -3.12" "py -3.13" "python3" "python"; do
  if $cand -c "import numpy, scipy" 2>/dev/null; then PY="$cand"; break; fi
done
[[ -n "$PY" ]] || { echo "no Python with numpy+scipy found (set ANCHOR_PY)"; exit 1; }
echo "using: $PY"
$PY -I oracle/kernel_ref.py
$PY -I oracle/numerics_ref.py
$PY -I oracle/baseline_ref.py
$PY -I oracle/rng_ref.py
$PY -I oracle/detect_ref.py
git diff --quiet -- oracle/vectors || echo "note: reference vectors changed - review the diff"

echo "== core (Rust): tests + lints"
(cd core && cargo test -q && cargo clippy -q --all-targets -- -D warnings)
if [[ "${1:-}" == "--full" ]]; then
  echo "== end-to-end false-alert budget (N4)"
  (cd core && cargo test -q budget -- --ignored --nocapture 2>&1 | grep -E "healthy|test result")
fi

echo "== kernel B (TypeScript): typecheck + tests"
(cd kernel-b && npm test --silent)

echo "== WebAssembly boundary (ADR-0009): host tests + lints + wasm32 build"
(cd wasm && cargo test -q && cargo clippy -q --all-targets -- -D warnings   && cargo build -q --release --target wasm32-unknown-unknown)
cp wasm/target/wasm32-unknown-unknown/release/anchor_wasm.wasm app/anchor.wasm
node app/scripts/build-demo.ts

echo "== interface: typecheck, build, language audit, tokens, boundary"
TSC=kernel-b/node_modules/.bin/tsc
# One command per line: under set -e, a failure left of && would not stop the run.
$TSC -p app
$TSC -p app/tsconfig.test.json
(cd app && node --test --test-reporter=dot test/*.test.ts)

echo "all checks passed"
