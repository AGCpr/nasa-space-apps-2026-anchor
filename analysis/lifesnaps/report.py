"""Summarise replay_people output (docs/VALIDATION.md). Standard library only.
Usage: python -I report.py <results.jsonl>
"""
import json
import statistics as st
import sys

MIN_DAYS = 28  # an indicator counts for a person only with at least this many scored days


def q(xs, p):
    xs = sorted(xs)
    return xs[min(len(xs) - 1, int(p * (len(xs) - 1) + 0.5))]


def main(path: str) -> None:
    rows = [json.loads(line) for line in open(path, encoding="utf-8")]
    print(f"people: {len(rows)}; records: {sum(r['records'] for r in rows)}; rejected: {sum(r['rejected'] for r in rows)}; "
          f"companion errors: {sum(r['errors'] for r in rows)}")
    ms_per_rec = sum(r["ms"] for r in rows) / sum(r["records"] for r in rows)
    print(f"engine time per record (native, incl. weekly Monte-Carlo recalibration): {ms_per_rec:.2f} ms")
    for key in ["mood", "sleep_min", "rhr_d1"]:
        ppl = [r["ind"][key] for r in rows if key in r["ind"] and r["ind"][key]["days"] >= MIN_DAYS]
        days = sum(p["days"] for p in ppl)
        cov = [p["in_band"] / p["days"] for p in ppl]
        alarms = sum(p["alarms"] for p in ppl)
        runs = sum(p["naive_runs"] for p in ppl)
        ndays = sum(p["naive_days"] for p in ppl)
        print(f"\n[{key}] people with >= {MIN_DAYS} scored days: {len(ppl)}; scored person-days: {days}")
        print(f"  80% band coverage: pooled {sum(p['in_band'] for p in ppl) / days:.3f}; per person median {st.median(cov):.3f} "
              f"(IQR {q(cov, .25):.3f}-{q(cov, .75):.3f})")
        print(f"  ANCHOR sustained-change notes: {alarms} = one per {days / max(alarms, 1):.1f} scored days")
        print(f"  naive single-day rule (harmful z >= 2): {ndays} alert days in {runs} runs = one run per {days / max(runs, 1):.1f} days")
    miss = sum(r["miss_alarms"] for r in rows)
    md = sum(r["mission_days"] for r in rows)
    print(f"\nfewer-check-ins notes: {miss} in {md} person-days = one per {md / max(miss, 1):.1f} days")


if __name__ == "__main__":
    main(sys.argv[1])
