"""Numerical check of the detection-limit figures quoted in DESIGN_PAPER.md.

Pure standard library. Run: py -I analysis/detection_limits.py
"""
import math
import random

random.seed(7)


def siegmund_arl(k, h, mu):
    """Siegmund's approximation for one-sided CUSUM ARL, unit-variance data, shift mu."""
    b = h + 1.166
    d = mu - k
    if abs(d) < 1e-9:
        return b * b
    return (math.exp(-2 * d * b) + 2 * d * b - 1) / (2 * d * d)


def sim_arl(k, h, mu, runs=4000, cap=100_000):
    """Monte-Carlo run length of one-sided upper CUSUM S_t = max(0, S_{t-1} + x_t - k)."""
    total = 0
    for _ in range(runs):
        s, t = 0.0, 0
        while t < cap:
            t += 1
            s = max(0.0, s + random.gauss(mu, 1.0) - k)
            if s > h:
                break
        total += t
    return total / runs


def find_h(k, target_arl0):
    lo, hi = 0.5, 10.0
    for _ in range(60):
        mid = (lo + hi) / 2
        if siegmund_arl(k, mid, 0.0) < target_arl0:
            lo = mid
        else:
            hi = mid
    return hi


def lorden_delay(arl0, delta):
    """Asymptotic minimax lower bound on detection delay: log(ARL0) / KL, KL = delta^2/2."""
    return math.log(arl0) / (delta * delta / 2)


def drift_days(beta, sigma=1.0, z=2.33):
    """Days n until an OLS slope beta (sigma/day units) is significant: beta*n^1.5/(sigma*sqrt12) > z."""
    return (z * sigma * math.sqrt(12) / beta) ** (2 / 3)


if __name__ == "__main__":
    K = 8                  # indicators monitored daily
    budget_days = 30       # system target: <= 1 false alert per 30 days
    arl0_target = K * budget_days  # union bound per indicator
    k = 0.5                # tuned for a 1-sigma shift

    h = find_h(k, arl0_target)
    arl0_sim = sim_arl(k, h, 0.0)
    arl1_sim = sim_arl(k, h, 1.0)
    print(f"per-indicator ARL0 target      : {arl0_target} days")
    print(f"CUSUM k={k}, h                  : {h:.2f}")
    print(f"ARL0 Siegmund / simulated      : {siegmund_arl(k, h, 0):.0f} / {arl0_sim:.0f} days")
    print(f"ARL1 (1 sigma) Siegmund / sim  : {siegmund_arl(k, h, 1):.1f} / {arl1_sim:.1f} days")
    print(f"ARL1 (0.5 sigma) simulated     : {sim_arl(k, h, 0.5):.1f} days")
    print(f"ARL1 (2 sigma) simulated       : {sim_arl(k, h, 2.0):.1f} days")
    print(f"Lorden bound, 1 sigma          : {lorden_delay(arl0_target, 1.0):.1f} days (asymptotic)")
    for beta in (0.02, 0.05, 0.1):
        print(f"drift {beta:.2f} sigma/day detectable after ~{drift_days(beta):.0f} days")

    # self-checks: simulation agrees with approximation within 15%, delays ordered
    assert abs(arl0_sim - siegmund_arl(k, h, 0)) / siegmund_arl(k, h, 0) < 0.15
    assert abs(arl1_sim - siegmund_arl(k, h, 1)) / siegmund_arl(k, h, 1) < 0.15
    assert drift_days(0.1) < drift_days(0.05) < drift_days(0.02)
    print("self-checks passed")
