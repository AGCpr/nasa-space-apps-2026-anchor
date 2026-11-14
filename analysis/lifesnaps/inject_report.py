"""Detection on real data with a known, injected change (docs/VALIDATION.md, "Injected change").

Sleep in the real LifeSnaps series is shortened by k pre-flight SDs from mission day 40 on
(prepare.py shift_sd); everything else, including gaps and day-to-day noise, stays real.
Usage: python -I inject_report.py <results.jsonl> <shift label>
"""
import json
import statistics as st
import sys

INJECT_DAY, HORIZON, MIN_AFTER = 40, 28, 21


def main(path: str, label: str) -> None:
    rows = [json.loads(line) for line in open(path, encoding="utf-8")]
    n = det_a = det_n = 0
    delays_a, delays_n = [], []
    pre_a = pre_n = pre_days = 0
    for r in rows:
        s = r["ind"].get("sleep_min")
        if not s or r["mission_days"] < INJECT_DAY + MIN_AFTER or s["days"] < 28:
            continue
        n += 1
        pre_days += INJECT_DAY - 1
        pre_a += sum(1 for d in s["alarm_days"] if d < INJECT_DAY)
        pre_n += sum(1 for d in s["naive_starts"] if d < INJECT_DAY)
        a = [d for d in s["alarm_days"] if INJECT_DAY <= d < INJECT_DAY + HORIZON]
        b = [d for d in s["naive_starts"] if INJECT_DAY <= d < INJECT_DAY + HORIZON]
        if a:
            det_a += 1
            delays_a.append(a[0] - INJECT_DAY + 1)
        if b:
            det_n += 1
            delays_n.append(b[0] - INJECT_DAY + 1)
    med = lambda xs: f"{st.median(xs):.0f}" if xs else "-"
    print(f"{label}: {n} people | ANCHOR noticed {det_a}/{n} = {det_a / n:.0%} within {HORIZON} d, median {med(delays_a)} d; "
          f"alerts before the change: one per {pre_days / max(pre_a, 1):.0f} d | naive rule {det_n}/{n} = {det_n / n:.0%}, "
          f"median {med(delays_n)} d; alerts before the change: one per {pre_days / max(pre_n, 1):.0f} d")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
