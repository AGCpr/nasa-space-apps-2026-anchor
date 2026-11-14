"""Simulations that decide two ADR-0005 questions before any detector code is written.

Q1. Autocorrelated daily data (AR(1), phi): run CUSUM on AR(1) residuals (pre-whitening, the
    DESIGN_PAPER v3 plan) or on the observations themselves with an autocorrelation-aware threshold?
    Both are calibrated to the same in-control ARL; compare detection delay for sustained shifts.
Q2. Adaptive sampling with a second same-day check-in: how much does within-day correlation rho_w
    erode the gain that review_checks.py computed under rho_w = 0?

Run: py -I analysis/adr0005_debates.py
"""
import numpy as np

rng = np.random.default_rng(5)
K, ARL0, RUNS, CAP = 0.5, 240.0, 4000, 4000


def ar1_paths(phi, n, horizon, shift=0.0):
    e = rng.normal(size=(n, horizon)) * np.sqrt(1 - phi ** 2)
    x = np.empty((n, horizon))
    x[:, 0] = rng.normal(size=n)
    for t in range(1, horizon):
        x[:, t] = phi * x[:, t - 1] + e[:, t]
    return x + shift


def run_lengths(series, h):
    s = np.zeros(series.shape[0])
    rl = np.full(series.shape[0], series.shape[1])
    alive = np.ones(series.shape[0], bool)
    for t in range(series.shape[1]):
        s = np.maximum(0, s + series[:, t] - K)
        hit = alive & (s > h)
        rl[hit] = t + 1
        alive &= ~hit
        if not alive.any():
            break
    return rl


def whiten(x, phi):
    r = np.empty_like(x)
    r[:, 0] = x[:, 0]
    r[:, 1:] = (x[:, 1:] - phi * x[:, :-1]) / np.sqrt(1 - phi ** 2)
    return r


def calibrate(make, target=ARL0):
    base = make(0.0)  # common random numbers across h
    lo, hi = 0.5, 30.0
    for _ in range(30):
        mid = (lo + hi) / 2
        lo, hi = (mid, hi) if run_lengths(base, mid).mean() < target else (lo, mid)
    return hi


print("Q1: sustained shift detection, both methods calibrated to ARL0 = 240 days")
print(f"{'phi':>4} {'method':>12} {'h':>6} {'1.0 sd':>8} {'0.5 sd':>8}")
for phi in (0.0, 0.3, 0.6):
    for name, f in (("prewhiten", lambda x: whiten(x, phi)), ("raw+cal", lambda x: x)):
        h = calibrate(lambda s: f(ar1_paths(phi, RUNS, CAP, s)))
        d1 = run_lengths(f(ar1_paths(phi, RUNS, 400, 1.0)), h).mean()
        d05 = run_lengths(f(ar1_paths(phi, RUNS, 600, 0.5)), h).mean()
        print(f"{phi:4.1f} {name:>12} {h:6.2f} {d1:8.1f} {d05:8.1f}")

print("\nQ2: gain factor of a 2nd same-day sample on the standardised daily value")
for rho in (0.0, 0.3, 0.5, 0.7):
    print(f"  rho_w={rho:.1f}: signal x{np.sqrt(2 / (1 + rho)):.3f}")
