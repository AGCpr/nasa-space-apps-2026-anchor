"""ADR-0007 §6: power of the anchor-run prompt, daily-run rule vs 14-day-mean rule.
AR(1) daily distances (phi=0.3, unit marginal SD) shifted by delta pre-flight SDs; one weekly check."""
import numpy as np

rng = np.random.default_rng(7)
N, W, phi = 200_000, 14, 0.3
e = rng.standard_normal((N, W))
x = np.empty_like(e)
x[:, 0] = e[:, 0]
for t in range(1, W):
    x[:, t] = phi * x[:, t - 1] + np.sqrt(1 - phi**2) * e[:, t]
print("delta  P(daily run: all 14 > 1)  P(14-day mean > 1)")
for d in (0.0, 0.5, 1.0, 1.5, 2.0):
    v = x + d
    print(f"{d:4.1f}   {np.mean((v > 1).all(axis=1)):.4f}                   {np.mean(v.mean(axis=1) > 1):.4f}")
