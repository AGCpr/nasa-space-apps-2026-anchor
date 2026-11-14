"""Regression-to-the-mean (RTM) bias in symptom-triggered "what worked for you" evaluation.

Scenario: an indicator follows AR(1) noise around the personal mean. A countermeasure
with ZERO true effect is started whenever the value drops below -1.5 SD; its "effect" is
judged as the change 5 days later (the pre/post design used in the field). We compare
with a randomised N-of-1 design (coin flip at each trigger, effect = treated - untreated).

Run: py -I analysis/rtm_check.py
"""
import random

random.seed(3)
PHI, TRIGGER, LAG, DAYS = 0.6, -1.5, 5, 200_000


def series():
    x, out = 0.0, []
    sd_innov = (1 - PHI ** 2) ** 0.5  # keeps stationary SD = 1
    for _ in range(DAYS):
        x = PHI * x + random.gauss(0, sd_innov)
        out.append(x)
    return out


if __name__ == "__main__":
    x = series()
    prepost, treated, control = [], [], []
    t = 0
    while t < DAYS - LAG:
        if x[t] < TRIGGER:
            change = x[t + LAG] - x[t]          # true effect is zero
            prepost.append(change)
            (treated if random.random() < 0.5 else control).append(change)
            t += LAG                             # one episode at a time
        t += 1
    mean = lambda v: sum(v) / len(v)
    naive = mean(prepost)
    rand = mean(treated) - mean(control)
    print(f"episodes: {len(prepost)}")
    print(f"pre/post 'improvement' with zero true effect : {naive:+.2f} SD")
    print(f"randomised N-of-1 estimate                   : {rand:+.2f} SD")
    assert naive > 1.0, "pre/post should show a large spurious improvement"
    assert abs(rand) < 0.1, "randomised estimate should be near zero"
    print("self-checks passed")
