# ADR-0004: Numerics and the personal Bayesian baseline

**Status:** accepted · 10 Nov 2026

## Context

DESIGN_PAPER §4.2 specifies a three-level hierarchy:
1. a population prior;
2. a pre-flight personal anchor, shrunk toward the population;
3. a rolling in-flight posterior.

Deviations are standardised through the posterior predictive (Student-t) and mapped to a normal quantile, so that the detectors downstream (CUSUM, drift) receive approximately N(0,1) inputs under "no change".

## Numerical routines and how they are trusted

| Routine | Method | Why this method | Verification |
|---|---|---|---|
| `ln Γ(x)` | Lanczos (g = 7, n = 9) | Standard; ~1e-15 relative for x > 0 | Against scipy `gammaln` |
| Regularised incomplete beta `I_x(a,b)` | Continued fraction (modified Lentz), with the symmetry `I_x(a,b) = 1 − I_{1−x}(b,a)` applied when x > (a+1)/(a+b+2) | Converges fast on both sides of the switch point; no external library | Against scipy `betainc` on a grid that includes the tails |
| Normal quantile Φ⁻¹(p) | Wichura AS241 (PPND16) | ~1e-16 relative over the full double range; published reference algorithm | Against scipy `ndtri`, p from 1e-300 to 1 − 1e-16 |
| Student-t → normal quantile | Computed **from the smaller tail**: for t > 0, $p_u = \tfrac12 I_{\nu/(\nu+t^2)}(\nu/2, 1/2)$ and $z = -\Phi^{-1}(p_u)$; symmetric for t < 0 | Computing $1-F$ by subtraction loses all precision beyond ~8 SD, so the tail is computed directly | Against scipy `t.sf` + `norm.isf` |

The normal CDF is **not** implemented: no component needs it yet. Code is written when needed, not in advance.

**Debated alternative:** depend on a statistics crate (e.g. `statrs`).
- **Rejected** for the core. It adds a large dependency surface to the component every output depends on, and we would still need the scipy oracle to trust it. Hand-written routines are small, standard, and checked against the oracle to 1e-12.

## Baseline model (per person, per indicator)

### Transform
- Bounded sliders (0..1000, i.e. 0.1-point units) are logit-transformed:

  $$y = \ln\frac{x+\epsilon}{1000-x+\epsilon},\quad \epsilon = 5$$

- **Why:** ratings near the ends are compressed. A step from 95 to 99 is a larger change than one from 50 to 54, and the untransformed scale under-reads it.
- **Why ε = 5 (half a point):** it keeps the end points finite while barely distorting the interior.
- Unbounded physiological values use the identity transform.

### Anchor: pre-flight sessions shrunk toward the population
- Inputs:
  - pre-flight sessions with mean $\bar y$, variance $s^2$, count $n_0$;
  - population mean $\mu_p$, between-person variance $\tau^2$ and within-person variance $\sigma_p^2$.
- Shrinkage of the mean:
  $$B = \frac{\sigma_p^2/n_0}{\sigma_p^2/n_0 + \tau^2},\qquad m_0 = B\mu_p + (1-B)\bar y$$
- Mean precision: $\kappa_0 = n_0 + \sigma_p^2/\tau^2$.
  - **Why** (an implementation-time refinement of the design paper's $\kappa_0 = n_0$): the population contributes $\sigma_p^2/\tau^2$ pseudo-observations to the mean. This makes $m_0$ and $B$ exactly the conjugate posterior mean, and it keeps $\kappa_0 > 0$ when $n_0 = 0$.
- Variance pooling: the variance prior treats the population variance as $\nu_p = 5$ pseudo-observations, so $\alpha_0 = (\nu_p + n_0 - 1)/2$ and $\beta_0 = \big(\nu_p\sigma_p^2 + (n_0-1)s^2\big)/2$.
- **Why:** with few sessions, a person's sample variance is unreliable and can produce an unrealistically narrow band and a flood of alerts. Pooling with $\nu_p$ pseudo-observations is the conjugate way to borrow strength. HELIOS arrives at the same idea heuristically ("spread blended with a population reference").

### Rolling posterior
- Prior: the anchor, **discounted** to $c = \min(\kappa_0, 7)$ pseudo-observations for both parameters:
  - mean weight $\kappa_r = c$;
  - variance $\alpha_r = \min\!\big(\alpha_0, (\nu_p + c - 1)/2\big)$ and $\beta_r = \alpha_r \cdot \hat v_0$, where $\hat v_0 = \beta_0/\alpha_0$ is the anchor's pooled variance.
  - The `min` guarantees that discounting never makes the variance *more* certain than the anchor itself.
  - The anchor's estimates are kept, but their weight is limited, so in-flight data can move both the mean and the spread.
- Data: the last $W = 14$ untagged days, winsorised at ±4 predictive SD (computed from the state before each value).
- Standard NIG conjugate update.
- **Why cap the weight at 7:** the rolling posterior must follow the recent normal, while the anchor keeps it from collapsing when data are sparse. Long-term drift is tested separately against the uncapped anchor (§4.4). That separation is the "dual baseline".

### Standardised deviation
$$z = \Phi^{-1}\big(T_{2\alpha}\big((y-m)/\mathrm{scale}\big)\big),\qquad \mathrm{scale}^2 = \frac{\beta(\kappa+1)}{\alpha\kappa}$$
Under the model, $z \sim N(0,1)$ exactly. This is tested by simulation (calibration test) as well as against the oracle.

**Cap:** the baseline reports $z$ clipped to ±8.
- **Why:** a single extreme reading must not dominate the CUSUM for weeks. Acute extremes are the safety kernel's job, and the kernel acts with no statistical delay.
- **Why ±8:** it is beyond any plausible statistical use; Φ(−8) ≈ 6e-16.
- The numerics layer itself returns the exact value (±∞ on underflow), so the clipping is a visible modelling choice, not a hidden numerical one.

**Minimum data:** the anchor works with any $n_0 \ge 0$.
- With $n_0 < 2$ the sample-variance term is dropped, and the population carries the variance.
- With $n_0 = 0$, $B = 1$ (the population mean is used).
- Confidence is reported from $n_0$ and the window size, never hidden.

## Consequences

- Every parameter ($\epsilon$, $\nu_p$, $\kappa$ cap, $W$, winsorisation limit) is a named constant with a rationale, and is versioned (change control, DESIGN_PAPER §14).
- Population priors are configuration, not code. Until published normative values are wired in, defaults are marked `ILLUSTRATIVE`.
