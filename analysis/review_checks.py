"""Numerical checks for claims raised in CRITICAL_REVIEW.md.

1. Union-bound thresholds are conservative when indicators are correlated.
2. Adaptive sampling (second check-in only when evidence accumulates) shortens detection.
3. Lag-1 autocorrelation from a 21-day window is too noisy to alert on.

Pure standard library. Run: py -I analysis/review_checks.py
"""
import math
import random

random.seed(11)
K, K_SHIFT = 8, 0.5


def corr_normals(rho):
    """K equicorrelated N(0,1) draws: x_i = sqrt(rho)*z + sqrt(1-rho)*e_i."""
    z = random.gauss(0, 1)
    return [math.sqrt(rho) * z + math.sqrt(1 - rho) * random.gauss(0, 1) for _ in range(K)]


def system_run_length(h, rho, shift=0.0, runs=1500, cap=5000):
    """Days until ANY of K one-sided CUSUMs alarms. shift applies to indicator 0 only."""
    total = 0
    for _ in range(runs):
        s = [0.0] * K
        for t in range(1, cap + 1):
            x = corr_normals(rho)
            x[0] += shift
            hit = False
            for i in range(K):
                s[i] = max(0.0, s[i] + x[i] - K_SHIFT)
                hit = hit or s[i] > h
            if hit:
                break
        total += t
    return total / runs


def calibrate_system_h(rho, target=30.0):
    lo, hi = 2.0, 6.0
    for _ in range(14):
        mid = (lo + hi) / 2
        if system_run_length(mid, rho, runs=600) < target:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2


def adaptive_run_length(h, mu, trigger, runs=3000, cap=20000):
    """One indicator. If S > trigger, take a 2nd measurement that day (mean of 2, std'ised)."""
    total_days, extra = 0, 0
    for _ in range(runs):
        s, t = 0.0, 0
        while t < cap:
            t += 1
            m = 2 if (trigger is not None and s > trigger) else 1
            extra += m - 1
            z = mu * math.sqrt(m) + random.gauss(0, 1)
            s = max(0.0, s + z - K_SHIFT)
            if s > h:
                break
        total_days += t
    return total_days / runs, extra / max(1, total_days)


def calibrate_adaptive_h(trigger, target=240.0):
    lo, hi = 2.5, 6.0
    for _ in range(14):
        mid = (lo + hi) / 2
        if adaptive_run_length(mid, 0.0, trigger, runs=800)[0] < target:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2


if __name__ == "__main__":
    h_union = 3.67  # from detection_limits.py: per-indicator ARL0 = 240 d (K=8, 30-day budget)

    print("1) correlated indicators, union-bound h = 3.67")
    for rho in (0.0, 0.5):
        print(f"   rho={rho}: system ARL0 = {system_run_length(h_union, rho):.0f} days (target 30)")
    h_cal = calibrate_system_h(0.5)
    d_union = system_run_length(h_union, 0.5, shift=1.0)
    d_cal = system_run_length(h_cal, 0.5, shift=1.0)
    print(f"   rho=0.5 calibrated h = {h_cal:.2f}; 1-sigma delay {d_union:.1f} -> {d_cal:.1f} days")

    print("2) adaptive sampling (2nd check-in only when S > 1.5)")
    h_ad = calibrate_adaptive_h(1.5)
    arl0_ad, burden0 = adaptive_run_length(h_ad, 0.0, 1.5)
    arl1_ad, _ = adaptive_run_length(h_ad, 1.0, 1.5)
    arl1_fix, _ = adaptive_run_length(h_union, 1.0, None)
    print(f"   h = {h_ad:.2f}, ARL0 = {arl0_ad:.0f} d, extra check-ins in stable periods = {100*burden0:.0f}% of days")
    print(f"   1-sigma delay: fixed {arl1_fix:.1f} d -> adaptive {arl1_ad:.1f} d")

    n = 21
    se = 1 / math.sqrt(n)
    print(f"3) lag-1 autocorrelation SE with n={n}: {se:.2f} (95% CI half-width {1.96*se:.2f})")

    assert d_cal <= d_union * 1.05, "calibrated threshold must not slow detection"
    assert arl1_ad < arl1_fix, "adaptive sampling must shorten detection"
    print("self-checks passed")
