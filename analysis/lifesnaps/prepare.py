"""Turn the LifeSnaps daily CSV into per-person ANCHOR logs in the wire format (ADR-0009 §2).

LifeSnaps (Yfantidou et al., Scientific Data 2022; Zenodo 7229547, CC BY 4.0): 71 adults, about 4 months,
Fitbit Sense plus EMA. The data stays outside the repository; only this script and the summary are committed.

Usage: python -I prepare.py <daily_fitbit_sema_df_unprocessed.csv> <out.txt>

Mapping (docs/VALIDATION.md):
  sleep_min  <- minutesAsleep              sample, harmful direction down
  rhr_d1     <- resting_hr (one decimal)   sample, harmful direction up
  mood       <- EMA valence proxy          checkin, harmful direction down
               slider = 500 + 200 * (HAPPY + RESTED/RELAXED - SAD - TENSE/ANXIOUS), clipped to 0..1000
The first PREFLIGHT_DAYS calendar days of each person are the "pre-flight" anchor period.
"""
import csv
import datetime as dt
import hashlib
import sys

PREFLIGHT_DAYS = 21
MIN_MISSION_DAYS = 42
DAY_MS = 86_400_000
TABLE = "t1|1,sleep_min,below,180,phys,1"
# key, dir, transform, decimals, mu, tau2, sigma2, nu_p (model scale, x 1e6). Population priors are
# generic adult values (sleep ~ 7 h, RHR ~ 65 bpm, mood as in the demo), deliberately weak (nu_p = 5).
INDICATORS = [
    ("mood", -1, 0, 0, 400_000, 300_000, 250_000, 5_000_000),
    ("rhr_d1", 1, 1, 1, 65_000_000, 49_000_000, 4_000_000, 5_000_000),
    ("sleep_min", -1, 1, 0, 400_000_000, 1_600_000_000, 2_500_000_000, 5_000_000),
]


def hx(s: str) -> str:
    return s.encode().hex()


def wire(met: int, kind: str, fields: dict) -> str:
    body = ";".join(f"{k}={hx(v) if k.endswith('_txt') else int(v)}" for k, v in sorted(fields.items()))
    return f"{met}|{kind}|{body}"


def num(row: dict, key: str):
    v = row.get(key, "")
    try:
        f = float(v)
    except ValueError:
        return None
    return None if f != f else f


def config() -> list[str]:
    out = [wire(0, "setup", {"budget_days": 30})]
    for key, d, tr, dec, mu, tau2, s2, nu in INDICATORS:
        out.append(wire(0, "indicator", {"key_txt": key, "dir": d, "transform": tr, "decimals": dec,
                                         "mu_e6": mu, "tau2_e6": tau2, "sigma2_e6": s2, "nu_p_e6": nu}))
    out.append(wire(0, "safety_table", {"table_txt": TABLE, "digest_txt": hashlib.sha256(TABLE.encode()).hexdigest()}))
    for key, crew in [("surgeon", 0), ("cmo", 1)]:
        out.append(wire(0, "recipient", {"key_txt": key, "crew": crew}))
    for to in ["surgeon", "cmo"]:
        out.append(wire(0, "route", {"class_txt": "safety", "to_txt": to}))
    out.append(wire(0, "link", {"up": 1}))
    return out


def values(row: dict) -> tuple[dict, dict]:
    sample, checkin = {}, {}
    s = num(row, "minutesAsleep")
    if s is not None and s > 0:
        sample["sleep_min"] = round(s)
    r = num(row, "resting_hr")
    if r is not None and r > 0:
        sample["rhr_d1"] = round(r * 10)
    flags = [num(row, k) for k in ("HAPPY", "RESTED/RELAXED", "SAD", "TENSE/ANXIOUS", "NEUTRAL", "ALERT", "TIRED")]
    if any(f is not None for f in flags):
        f = [x or 0.0 for x in flags]
        checkin["mood"] = max(0, min(1000, round(500 + 200 * (f[0] + f[1] - f[2] - f[3]))))
    return sample, checkin


def person(rows: list[dict]) -> list[str] | None:
    rows.sort(key=lambda r: r["date"])
    start = dt.date.fromisoformat(rows[0]["date"])
    by_day = {(dt.date.fromisoformat(r["date"]) - start).days: r for r in rows}
    last = max(by_day)
    if last - PREFLIGHT_DAYS < MIN_MISSION_DAYS:
        return None
    out = config()
    for d in range(PREFLIGHT_DAYS):
        if d in by_day:
            s, c = values(by_day[d])
            if s or c:
                out.append(wire(0, "preflight", {**s, **c}))
    for d in range(PREFLIGHT_DAYS, last + 1):
        day = d - PREFLIGHT_DAYS + 1
        base = day * DAY_MS
        if d in by_day:
            s, c = values(by_day[d])
            if s:
                out.append(wire(base + 7 * 3_600_000, "sample", {"day": day, **s}))
            if c:
                out.append(wire(base + 20 * 3_600_000, "checkin", {"day": day, **c}))
        out.append(wire(base + 23 * 3_600_000, "day_close", {"day": day}))
    return out


def main(src: str, dst: str) -> None:
    people: dict[str, list[dict]] = {}
    with open(src, encoding="utf-8") as f:
        for row in csv.DictReader(f):
            people.setdefault(row["id"], []).append(row)
    kept = 0
    with open(dst, "w", encoding="utf-8", newline="\n") as f:  # LF: the reader splits blocks on blank lines
        for pid, rows in sorted(people.items()):
            log = person(rows)
            if log is None:
                continue
            kept += 1
            f.write(f"# {pid}\n" + "\n".join(log) + "\n\n")
    print(f"{kept} of {len(people)} people with >= {MIN_MISSION_DAYS} days after a {PREFLIGHT_DAYS}-day pre-flight")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
