/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 6/11/26
******************************************************************************/

//! The multi-factor quadratic rough Heston model: its kernel, its simulation by the
//! explicit-implicit Euler scheme (A.1) and the Monte Carlo price of a vanilla on it.
//! [`QuadraticRoughHestonParams`] documents the model for consumers.

use crate::error::QuadraticRoughHestonError;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{
    d_div, d_sub, decimal_to_f64, f64_to_decimal, finite_decimal,
};
use optionstratlib_core::model::types::OptionStyle;
use optionstratlib_core::utils::deterministic_rng;
use rand::RngExt;
use rand::rngs::StdRng;
use rand_distr::{Distribution, StandardNormal};
use rayon::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::f64::consts::PI;
use std::num::NonZeroUsize;

/// The excluded lower bound of `alpha`.
const ALPHA_LOWER_BOUND: Decimal = dec!(0.5);

/// The fewest paths with a standard error.
const MIN_PATHS: usize = 2;

/// An empty buffer able to hold `len` entries without growing.
fn reserve<T>(len: usize, buffer: &'static str) -> Result<Vec<T>, QuadraticRoughHestonError> {
    let mut reserved = Vec::new();
    reserved
        .try_reserve_exact(len)
        .map_err(|_| QuadraticRoughHestonError::BufferTooLarge { buffer, len })?;
    Ok(reserved)
}

#[cold]
#[inline(never)]
fn invalid_parameter(
    parameter: &'static str,
    value: Decimal,
    reason: &'static str,
) -> QuadraticRoughHestonError {
    QuadraticRoughHestonError::InvalidParameter {
        parameter,
        value,
        reason,
    }
}

/// `raw` as a positive `Decimal`, or the typed error of a value that has none.
fn kernel_decimal(
    raw: f64,
    quantity: &'static str,
    index: usize,
) -> Result<Decimal, QuadraticRoughHestonError> {
    finite_decimal(raw)
        .filter(|value| *value > Decimal::ZERO)
        .ok_or(QuadraticRoughHestonError::KernelOutOfRange { quantity, index })
}

/// The factor weights and discounts of the geometric partition of the fractional kernel.
///
/// With `eta_i = x^(i - n/2)` for `i = 1..=n`, the measure
/// `mu(dg) = g^(-alpha) dg / (Gamma(alpha) Gamma(1 - alpha))` is lumped on each
/// `[eta_(i-1), eta_i]` into one atom, whose mass is the weight and whose mean is the
/// discount:
///
/// ```text
/// c_i     = x^((1-alpha)(i-n/2)) (1 - x^(alpha-1)) / ((1-alpha) Gamma(alpha) Gamma(1-alpha))
/// gamma_i = (1-alpha) x^(i-1-n/2) (x^(2-alpha) - 1) / ((2-alpha) (x^(1-alpha) - 1))
/// ```
///
/// [`value_at`](Self::value_at) is `K^n(t) = sum c_i exp(-gamma_i t)`, which approximates
/// `K(t) = t^(alpha-1) / Gamma(alpha)` on a range of `t` set by `n` and `x`. The ratio that
/// suits a given `n` is the caller's choice: there is no default.
#[derive(Debug, Clone, PartialEq)]
pub struct QuadraticRoughHestonKernel {
    alpha: Decimal,
    ratio: Decimal,
    weights: Vec<Decimal>,
    discounts: Vec<Decimal>,
    raw_weights: Vec<f64>,
    raw_discounts: Vec<f64>,
}

impl QuadraticRoughHestonKernel {
    /// Builds the kernel of `factors` factors from `alpha` and the geometric `ratio`.
    ///
    /// # Errors
    ///
    /// - [`QuadraticRoughHestonError::AlphaOutOfRange`] when `alpha` is outside `(1/2, 1)`.
    /// - [`QuadraticRoughHestonError::NoFactors`] when `factors` is zero.
    /// - [`QuadraticRoughHestonError::RatioNotAboveOne`] when `ratio` is at most one.
    /// - [`QuadraticRoughHestonError::KernelOutOfRange`] when a weight or a discount has no
    ///   positive value in a `Decimal`.
    /// - [`QuadraticRoughHestonError::BufferTooLarge`] when `factors` cannot be allocated.
    /// - [`QuadraticRoughHestonError::Decimal`] when an input has no `f64` value.
    pub fn try_new(
        alpha: Decimal,
        factors: usize,
        ratio: Decimal,
    ) -> Result<Self, QuadraticRoughHestonError> {
        if alpha <= ALPHA_LOWER_BOUND || alpha >= Decimal::ONE {
            return Err(QuadraticRoughHestonError::AlphaOutOfRange { alpha });
        }
        if factors == 0 {
            return Err(QuadraticRoughHestonError::NoFactors);
        }
        if ratio <= Decimal::ONE {
            return Err(QuadraticRoughHestonError::RatioNotAboveOne { ratio });
        }

        // `1 - alpha` and `ratio - 1` come from the `Decimal`, so an `alpha` just below one
        // and a ratio just above it keep their distance from one, which an `f64` of the
        // whole value would round away. `1 - alpha` is in `(0, 1/2)`,
        // `Gamma(alpha) Gamma(1 - alpha)` is `pi / sin(pi (1 - alpha))`, and `exp_m1` keeps
        // the differences of powers accurate for a ratio close to one.
        let beta = decimal_to_f64(d_sub(
            Decimal::ONE,
            alpha,
            "quadratic_rough_heston::kernel",
        )?)?;
        let log_ratio = decimal_to_f64(d_sub(
            ratio,
            Decimal::ONE,
            "quadratic_rough_heston::kernel",
        )?)?
        .ln_1p(); // scan-banned: allow -- f64 `ln_1p` of a positive value: positive and finite, and it does not abort
        let sin_pi_beta = (PI * beta).sin(); // scan-banned: allow -- f64 `sin` of a finite value: finite, and it does not abort
        let weight_scale = -(-beta * log_ratio).exp_m1() * sin_pi_beta / (beta * PI);
        let discount_scale = beta * ((1.0 + beta) * log_ratio).exp_m1()
            / ((1.0 + beta) * (beta * log_ratio).exp_m1());
        let half_factors = factors as f64 / 2.0;

        let mut weights = reserve(factors, "kernel weights")?;
        let mut discounts = reserve(factors, "kernel discounts")?;
        let mut raw_weights = reserve(factors, "kernel weights")?;
        let mut raw_discounts = reserve(factors, "kernel discounts")?;
        for index in 1..=factors {
            let exponent = index as f64 - half_factors;
            let raw_weight = (beta * exponent * log_ratio).exp() * weight_scale; // scan-banned: allow -- f64 `exp`: returns inf on overflow, it does not abort; `kernel_decimal` rejects it
            let raw_discount = ((exponent - 1.0) * log_ratio).exp() * discount_scale; // scan-banned: allow -- f64 `exp`: returns inf on overflow, it does not abort; `kernel_decimal` rejects it
            weights.push(kernel_decimal(raw_weight, "weight", index)?);
            discounts.push(kernel_decimal(raw_discount, "discount", index)?);
            raw_weights.push(raw_weight);
            raw_discounts.push(raw_discount);
        }

        Ok(Self {
            alpha,
            ratio,
            weights,
            discounts,
            raw_weights,
            raw_discounts,
        })
    }

    /// The roughness exponent `alpha`, in `(1/2, 1)`.
    #[must_use]
    #[inline]
    pub fn alpha(&self) -> Decimal {
        self.alpha
    }

    /// The geometric ratio `x_n` of the partition.
    #[must_use]
    #[inline]
    pub fn ratio(&self) -> Decimal {
        self.ratio
    }

    /// The number of factors `n`.
    #[must_use]
    #[inline]
    pub fn factors(&self) -> usize {
        self.weights.len()
    }

    /// The weights `c_i`, from `i = 1` to `n`, rounded to a `Decimal`.
    #[must_use]
    #[inline]
    pub fn weights(&self) -> &[Decimal] {
        &self.weights
    }

    /// The discounts `gamma_i`, per year, from `i = 1` to `n`, rounded to a `Decimal`.
    #[must_use]
    #[inline]
    pub fn discounts(&self) -> &[Decimal] {
        &self.discounts
    }

    /// The approximated kernel `K^n(t) = sum c_i exp(-gamma_i t)` at `t` years.
    ///
    /// # Errors
    ///
    /// Returns [`QuadraticRoughHestonError::Decimal`] when `t` has no `f64` value or the
    /// result does not fit a `Decimal`.
    pub fn value_at(&self, t: Positive) -> Result<Decimal, QuadraticRoughHestonError> {
        let t = decimal_to_f64(t.to_dec())?;
        let value: f64 = self
            .raw_weights
            .iter()
            .zip(&self.raw_discounts)
            .map(|(weight, discount)| weight * (-discount * t).exp()) // scan-banned: allow -- f64 `exp` of a non-positive value: finite, and it does not abort
            .sum();
        Ok(f64_to_decimal(value)?)
    }
}

/// The parameters of the model and its initial state.
///
/// # Model
///
/// The multi-factor approximation of the quadratic rough Heston model, driven by one
/// Brownian motion `W` under the risk-neutral measure:
///
/// ```text
/// dS   = S sqrt(V) dW,   V = a (Z - b)^2 + c,   Z = sum c_i Z^i
/// dZ^i = (-gamma_i Z^i - lambda Z) dt + eta sqrt(V) dW,   Z^i_0 given
/// ```
///
/// `(c_i, gamma_i)` come from the [`QuadraticRoughHestonKernel`]. The initial factors are
/// free parameters, which replace the function `theta_0` of the rough model and set the
/// initial term structure of `Z`.
///
/// # Scheme
///
/// [`simulate_quadratic_rough_heston_path`] and [`price_vanilla_quadratic_rough_heston`]
/// use the explicit-implicit Euler scheme (A.1) of the calibration paper, with `dt = T / N`
/// and `dW ~ N(0, dt)`:
///
/// ```text
/// V_k       = a (Z_k - b)^2 + c,   Z_k = sum c_i Z^i_k
/// S_(k+1)   = S_k + S_k sqrt(V_k) dW_k
/// Z^i_(k+1) = (Z^i_k - lambda Z_k dt + eta sqrt(V_k) dW_k) / (1 + gamma_i dt)
/// ```
///
/// The implicit denominator keeps the scheme stable for a large `gamma_i`.
///
/// # What this implementation does and does not do
///
/// - **The spot can go negative.** Scheme (A.1) is an arithmetic Euler step, so `S_(k+1)` is
///   not above zero when `sqrt(V_k) dW_k <= -1`. It is not clamped, since a clamp would break
///   the martingale property. [`QuadraticRoughHestonEstimate::non_positive_paths`] counts the
///   affected paths, and on one of them a put can pay more than its strike. A low count does
///   not show that the step is fine enough, as the last point below shows.
/// - **The variance is not truncated.** The calibration paper's footnote 1 mentions a
///   truncated square that secures the martingale property. This is the plain square, so a
///   path whose state stops being finite returns [`QuadraticRoughHestonError::NonFinite`]
///   with its step, and no value is replaced.
/// - **There are no rates or dividends.** `S` is driftless, and a price is the undiscounted
///   mean payoff, as with `r = q = 0`.
/// - **No parameter set is a default.** The papers' values are for SPX and VIX, and nothing
///   here is calibrated to any underlying.
/// - **One run does not show that a price has converged.** A strong feedback gives heavy
///   tails, and neither a small standard error, a low count of non positive paths nor a mean
///   of `S_T` close to `S_0` rules out a wrong price. Take `alpha = 0.51`, `n = 10`, `x = 3.92`,
///   `a = 1`, `b = 0.1`, `c = 0.01`, `lambda = 1.5`, `eta = 1`, `Z^i_0 = -0.05`,
///   `S_0 = K = 100`, `T = 0.25` and 100,000 paths, as an illustration. At `N = 100`, about
///   0.15% of the paths are non positive, and an ATM call is `150589 +/- 150584` at seed 1 and
///   `5.32 +/- 0.02` at seed 3, while `N = 800` gives about `4.89`. So the small standard error
///   at seed 3 hides a bias of about 20 of them, and the blown-up run has a mean of `S_T` about
///   one standard error from `S_0`, because the standard error grows with the mean. **Compare
///   several seeds and several step counts before trusting a price.**
///
/// # References
///
/// - Rosenbaum and Zhang, *Deep calibration of the quadratic rough Heston model*,
///   [arXiv:2107.01611v2](https://arxiv.org/abs/2107.01611v2), Section 2 and Appendix A.
/// - *Multi-asset market making under the quadratic rough Heston*,
///   [arXiv:2212.10164v1](https://arxiv.org/abs/2212.10164v1), Section 2.1.
///
/// # Examples
///
/// ```rust
/// use optionstratlib_core::model::types::OptionStyle;
/// use optionstratlib_core::pos_or_panic;
/// use optionstratlib_simulation::simulation::{
///     QuadraticRoughHestonGrid, QuadraticRoughHestonKernel, QuadraticRoughHestonParams,
///     price_vanilla_quadratic_rough_heston,
/// };
/// use rust_decimal_macros::dec;
/// use std::num::NonZeroUsize;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Illustrative values, calibrated to nothing.
/// let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.6), 5, dec!(5))?;
/// let params = QuadraticRoughHestonParams::try_new(
///     dec!(0.05),
///     dec!(0.1),
///     dec!(0.04),
///     dec!(1),
///     dec!(0.1),
///     kernel,
///     vec![dec!(0); 5],
///     pos_or_panic!(100.0),
/// )?;
/// let grid = QuadraticRoughHestonGrid::try_new(dec!(0.25), 50)?;
/// let paths = NonZeroUsize::try_from(200_usize)?;
/// let estimate = price_vanilla_quadratic_rough_heston(
///     &params,
///     &grid,
///     OptionStyle::Call,
///     pos_or_panic!(100.0),
///     paths,
///     42,
/// )?;
/// assert!(estimate.price() > dec!(0));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct QuadraticRoughHestonParams {
    a: Decimal,
    b: Decimal,
    c: Decimal,
    lambda: Decimal,
    eta: Decimal,
    kernel: QuadraticRoughHestonKernel,
    initial_factors: Vec<Decimal>,
    spot: Positive,
    initial_z: Decimal,
    initial_variance: Decimal,
    raw: RawParams,
    raw_initial: State,
}

/// The parameters in the `f64` of the kernel.
#[derive(Debug, Clone, Copy, PartialEq)]
struct RawParams {
    a: f64,
    b: f64,
    c: f64,
    lambda: f64,
    eta: f64,
}

impl QuadraticRoughHestonParams {
    /// Validates the parameters and reads the initial state.
    ///
    /// `a >= 0`, `c > 0`, `lambda > 0`, `eta > 0` and a positive `spot`; `b` is free. The
    /// `initial_factors` are the `Z^i_0`, one per factor of the `kernel`.
    ///
    /// # Errors
    ///
    /// - [`QuadraticRoughHestonError::InvalidParameter`] when a parameter breaks its condition.
    /// - [`QuadraticRoughHestonError::InitialFactorCount`] when `initial_factors` does not hold
    ///   one value per factor.
    /// - [`QuadraticRoughHestonError::Decimal`] when the initial `Z` or `V` does not fit a
    ///   `Decimal`, or a parameter has no `f64` value.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        a: Decimal,
        b: Decimal,
        c: Decimal,
        lambda: Decimal,
        eta: Decimal,
        kernel: QuadraticRoughHestonKernel,
        initial_factors: Vec<Decimal>,
        spot: Positive,
    ) -> Result<Self, QuadraticRoughHestonError> {
        if a < Decimal::ZERO {
            return Err(invalid_parameter("a", a, "must not be negative"));
        }
        if c <= Decimal::ZERO {
            return Err(invalid_parameter("c", c, "must be positive"));
        }
        if lambda <= Decimal::ZERO {
            return Err(invalid_parameter("lambda", lambda, "must be positive"));
        }
        if eta <= Decimal::ZERO {
            return Err(invalid_parameter("eta", eta, "must be positive"));
        }
        if spot.to_dec() <= Decimal::ZERO {
            return Err(invalid_parameter("spot", spot.to_dec(), "must be positive"));
        }
        if initial_factors.len() != kernel.factors() {
            return Err(QuadraticRoughHestonError::InitialFactorCount {
                expected: kernel.factors(),
                actual: initial_factors.len(),
            });
        }

        let raw = RawParams {
            a: decimal_to_f64(a)?,
            b: decimal_to_f64(b)?,
            c: decimal_to_f64(c)?,
            lambda: decimal_to_f64(lambda)?,
            eta: decimal_to_f64(eta)?,
        };
        let raw_factors = initial_factors
            .iter()
            .map(|factor| decimal_to_f64(*factor))
            .collect::<Result<Vec<f64>, _>>()?;
        let (z, variance) = read(&kernel.raw_weights, &raw, &raw_factors);
        let raw_initial = State {
            spot: decimal_to_f64(spot.to_dec())?,
            z,
            variance,
            factors: raw_factors,
        };
        let initial_z = f64_to_decimal(z)?;
        let initial_variance = f64_to_decimal(variance)?;

        Ok(Self {
            a,
            b,
            c,
            lambda,
            eta,
            kernel,
            initial_factors,
            spot,
            initial_z,
            initial_variance,
            raw,
            raw_initial,
        })
    }

    /// The strength `a` of the feedback on the variance.
    #[must_use]
    #[inline]
    pub fn a(&self) -> Decimal {
        self.a
    }

    /// The asymmetry `b` of the feedback.
    #[must_use]
    #[inline]
    pub fn b(&self) -> Decimal {
        self.b
    }

    /// The base level `c` of the variance, per year.
    #[must_use]
    #[inline]
    pub fn c(&self) -> Decimal {
        self.c
    }

    /// The mean reversion `lambda` of `Z`.
    #[must_use]
    #[inline]
    pub fn lambda(&self) -> Decimal {
        self.lambda
    }

    /// The volatility `eta` of the factors.
    #[must_use]
    #[inline]
    pub fn eta(&self) -> Decimal {
        self.eta
    }

    /// The kernel.
    #[must_use]
    #[inline]
    pub fn kernel(&self) -> &QuadraticRoughHestonKernel {
        &self.kernel
    }

    /// The initial factors `Z^i_0`.
    #[must_use]
    #[inline]
    pub fn initial_factors(&self) -> &[Decimal] {
        &self.initial_factors
    }

    /// The initial spot `S_0`.
    #[must_use]
    #[inline]
    pub fn spot(&self) -> Positive {
        self.spot
    }

    /// The initial `Z_0 = sum c_i Z^i_0`.
    #[must_use]
    #[inline]
    pub fn initial_z(&self) -> Decimal {
        self.initial_z
    }

    /// The initial variance `V_0 = a (Z_0 - b)^2 + c`, per year.
    #[must_use]
    #[inline]
    pub fn initial_variance(&self) -> Decimal {
        self.initial_variance
    }
}

/// A uniform time grid: `steps` steps over `horizon` years.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuadraticRoughHestonGrid {
    horizon: Decimal,
    steps: usize,
    dt: Decimal,
}

impl QuadraticRoughHestonGrid {
    /// Builds the grid of `steps` steps over `horizon` years.
    ///
    /// # Errors
    ///
    /// - [`QuadraticRoughHestonError::NonPositiveStep`] when `horizon` is not above zero,
    ///   `steps` is zero, or the step rounds to zero in a `Decimal`.
    /// - [`QuadraticRoughHestonError::Decimal`] when the division fails.
    pub fn try_new(horizon: Decimal, steps: usize) -> Result<Self, QuadraticRoughHestonError> {
        let non_positive = || QuadraticRoughHestonError::NonPositiveStep { horizon, steps };
        if horizon <= Decimal::ZERO || steps == 0 {
            return Err(non_positive());
        }
        let dt = d_div(
            horizon,
            Decimal::from(steps),
            "quadratic_rough_heston::grid",
        )?;
        if dt.is_zero() {
            return Err(non_positive());
        }
        Ok(Self { horizon, steps, dt })
    }

    /// The horizon `T`, in years.
    #[must_use]
    #[inline]
    pub fn horizon(&self) -> Decimal {
        self.horizon
    }

    /// The number of steps `N`.
    #[must_use]
    #[inline]
    pub fn steps(&self) -> usize {
        self.steps
    }

    /// The step `dt = T / N`, in years, rounded to a `Decimal`.
    #[must_use]
    #[inline]
    pub fn dt(&self) -> Decimal {
        self.dt
    }
}

/// One simulated path, by step of the grid from 0 to `N`.
///
/// `spot` and `z` can be negative. See [`QuadraticRoughHestonParams`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuadraticRoughHestonPath {
    spot: Vec<Decimal>,
    variance: Vec<Decimal>,
    z: Vec<Decimal>,
    factors: Vec<Decimal>,
    factor_count: usize,
}

impl QuadraticRoughHestonPath {
    /// The spot `S_k` at each step.
    #[must_use]
    #[inline]
    pub fn spot(&self) -> &[Decimal] {
        &self.spot
    }

    /// The variance `V_k`, per year, at each step.
    #[must_use]
    #[inline]
    pub fn variance(&self) -> &[Decimal] {
        &self.variance
    }

    /// `Z_k` at each step.
    #[must_use]
    #[inline]
    pub fn z(&self) -> &[Decimal] {
        &self.z
    }

    /// The factors `Z^i_k` at `step`, or `None` past the end of the path.
    #[must_use]
    pub fn factors_at(&self, step: usize) -> Option<&[Decimal]> {
        let start = step.checked_mul(self.factor_count)?;
        let end = start.checked_add(self.factor_count)?;
        self.factors.get(start..end)
    }
}

/// The Monte Carlo price of a vanilla, with its diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuadraticRoughHestonEstimate {
    price: Decimal,
    standard_error: Decimal,
    mean_terminal_spot: Decimal,
    terminal_spot_standard_error: Decimal,
    paths: usize,
    non_positive_paths: usize,
}

impl QuadraticRoughHestonEstimate {
    /// The mean payoff per unit of underlying, undiscounted.
    #[must_use]
    #[inline]
    pub fn price(&self) -> Decimal {
        self.price
    }

    /// The standard error of [`price`](Self::price).
    #[must_use]
    #[inline]
    pub fn standard_error(&self) -> Decimal {
        self.standard_error
    }

    /// The mean of `S_T`, which estimates `S_0`.
    #[must_use]
    #[inline]
    pub fn mean_terminal_spot(&self) -> Decimal {
        self.mean_terminal_spot
    }

    /// The standard error of [`mean_terminal_spot`](Self::mean_terminal_spot).
    #[must_use]
    #[inline]
    pub fn terminal_spot_standard_error(&self) -> Decimal {
        self.terminal_spot_standard_error
    }

    /// The number of simulated paths.
    #[must_use]
    #[inline]
    pub fn paths(&self) -> usize {
        self.paths
    }

    /// The paths with `S_k <= 0` at some step. See [`QuadraticRoughHestonParams`].
    #[must_use]
    #[inline]
    pub fn non_positive_paths(&self) -> usize {
        self.non_positive_paths
    }
}

/// `Z = sum c_i Z^i` and `V = a (Z - b)^2 + c`.
#[inline]
fn read(weights: &[f64], raw: &RawParams, factors: &[f64]) -> (f64, f64) {
    let z: f64 = weights
        .iter()
        .zip(factors)
        .map(|(weight, factor)| weight * factor)
        .sum();
    let shifted = z - raw.b;
    (z, raw.a * shifted * shifted + raw.c)
}

/// The state of the scheme at one step, with its `Z` and `V`.
#[derive(Debug, Clone, PartialEq)]
struct State {
    spot: f64,
    z: f64,
    variance: f64,
    factors: Vec<f64>,
}

/// A value of the state that stopped being finite: the kernel's small form of
/// [`QuadraticRoughHestonError::NonFinite`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NonFiniteValue {
    quantity: &'static str,
    step: usize,
}

impl NonFiniteValue {
    #[cold]
    #[inline(never)]
    fn into_error(self) -> QuadraticRoughHestonError {
        QuadraticRoughHestonError::NonFinite {
            quantity: self.quantity,
            step: self.step,
        }
    }
}

impl State {
    /// Every weight is positive, so a factor that is not finite leaves `z` not finite.
    #[inline]
    fn ensure_finite(&self, step: usize) -> Result<(), NonFiniteValue> {
        let quantity = if !self.spot.is_finite() {
            "spot"
        } else if !self.z.is_finite() {
            "z"
        } else if !self.variance.is_finite() {
            "variance"
        } else {
            return Ok(());
        };
        Err(NonFiniteValue { quantity, step })
    }
}

/// One step of the scheme (A.1) from `state` into `next`, for the increment `dw` over `dt`.
///
/// `next` is scratch of the same shape: nothing of it is read, and nothing allocates.
#[inline(always)]
fn step(params: &QuadraticRoughHestonParams, state: &State, dw: f64, dt: f64, next: &mut State) {
    let raw = &params.raw;
    let diffusion = state.variance.sqrt() * dw; // scan-banned: allow -- f64 `sqrt`: returns NaN for negative input, it does not abort; `ensure_finite` rejects it
    let forcing = -raw.lambda * state.z * dt + raw.eta * diffusion;
    next.spot = state.spot + state.spot * diffusion;
    for ((next_factor, factor), discount) in next
        .factors
        .iter_mut()
        .zip(&state.factors)
        .zip(&params.kernel.raw_discounts)
    {
        *next_factor = (factor + forcing) / (1.0 + discount * dt);
    }
    (next.z, next.variance) = read(&params.kernel.raw_weights, raw, &next.factors);
}

/// The increments of one path, and the `next` scratch of its steps.
struct Walk<'a> {
    params: &'a QuadraticRoughHestonParams,
    dt: f64,
    sqrt_dt: f64,
    rng: StdRng,
    state: State,
    scratch: State,
}

impl<'a> Walk<'a> {
    fn new(params: &'a QuadraticRoughHestonParams, (dt, sqrt_dt): (f64, f64), seed: u64) -> Self {
        Self {
            params,
            dt,
            sqrt_dt,
            rng: deterministic_rng(seed),
            state: params.raw_initial.clone(),
            scratch: params.raw_initial.clone(),
        }
    }

    /// Moves to grid step `to`, drawing its normal increment.
    #[inline]
    fn advance(&mut self, to: usize) -> Result<(), NonFiniteValue> {
        let normal: f64 = StandardNormal.sample(&mut self.rng);
        self.apply(self.sqrt_dt * normal, to)
    }

    /// Moves to grid step `to` on the increment `dw`.
    #[inline]
    fn apply(&mut self, dw: f64, to: usize) -> Result<(), NonFiniteValue> {
        step(self.params, &self.state, dw, self.dt, &mut self.scratch);
        std::mem::swap(&mut self.state, &mut self.scratch);
        self.state.ensure_finite(to)
    }
}

/// `dt` and its square root, in `f64`.
fn raw_step(grid: &QuadraticRoughHestonGrid) -> Result<(f64, f64), QuadraticRoughHestonError> {
    let dt = decimal_to_f64(grid.dt)?;
    Ok((dt, dt.sqrt())) // scan-banned: allow -- f64 `sqrt` of a positive finite value: finite, and it does not abort
}

/// Simulates one path of the model by the scheme (A.1), from `seed`.
///
/// The same arguments give the same path on the same build and platform. Across platforms the
/// `f64` `exp`, `ln` and `sin` may round differently. See [`QuadraticRoughHestonParams`] for
/// the model and the scheme.
///
/// # Errors
///
/// - [`QuadraticRoughHestonError::NonFinite`] when the state stops being finite, with its step.
/// - [`QuadraticRoughHestonError::BufferTooLarge`] when the path cannot be allocated.
/// - [`QuadraticRoughHestonError::Decimal`] when a value does not fit a `Decimal`.
#[tracing::instrument(level = "debug", skip(params))]
pub fn simulate_quadratic_rough_heston_path(
    params: &QuadraticRoughHestonParams,
    grid: &QuadraticRoughHestonGrid,
    seed: u64,
) -> Result<QuadraticRoughHestonPath, QuadraticRoughHestonError> {
    let factor_count = params.kernel.factors();
    let points = grid
        .steps
        .checked_add(1)
        .ok_or(QuadraticRoughHestonError::BufferTooLarge {
            buffer: "path",
            len: grid.steps,
        })?;
    let factor_points =
        points
            .checked_mul(factor_count)
            .ok_or(QuadraticRoughHestonError::BufferTooLarge {
                buffer: "path factors",
                len: points,
            })?;
    let mut path = QuadraticRoughHestonPath {
        spot: reserve(points, "path spots")?,
        variance: reserve(points, "path variances")?,
        z: reserve(points, "path z")?,
        factors: reserve(factor_points, "path factors")?,
        factor_count,
    };

    let mut walk = Walk::new(params, raw_step(grid)?, seed);
    record(&mut path, &walk.state)?;
    for to in 1..=grid.steps {
        walk.advance(to).map_err(NonFiniteValue::into_error)?;
        record(&mut path, &walk.state)?;
    }
    Ok(path)
}

/// Appends `state` to `path`.
fn record(
    path: &mut QuadraticRoughHestonPath,
    state: &State,
) -> Result<(), QuadraticRoughHestonError> {
    path.spot.push(f64_to_decimal(state.spot)?);
    path.variance.push(f64_to_decimal(state.variance)?);
    path.z.push(f64_to_decimal(state.z)?);
    for factor in &state.factors {
        path.factors.push(f64_to_decimal(*factor)?);
    }
    Ok(())
}

/// What the price keeps of one path.
#[derive(Debug, Clone, Copy, PartialEq)]
struct PathOutcome {
    terminal_spot: f64,
    non_positive: bool,
}

/// Runs one path without recording it.
fn run_path(
    params: &QuadraticRoughHestonParams,
    grid: &QuadraticRoughHestonGrid,
    step_sizes: (f64, f64),
    seed: u64,
) -> Result<PathOutcome, NonFiniteValue> {
    let mut walk = Walk::new(params, step_sizes, seed);
    let mut non_positive = false;
    for to in 1..=grid.steps {
        walk.advance(to)?;
        non_positive |= walk.state.spot <= 0.0;
    }
    Ok(PathOutcome {
        terminal_spot: walk.state.spot,
        non_positive,
    })
}

/// The running mean and squared deviations of Welford's algorithm.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Moments {
    count: f64,
    mean: f64,
    squared_deviations: f64,
}

impl Moments {
    #[inline]
    fn add(&mut self, value: f64) {
        self.count += 1.0;
        let deviation = value - self.mean;
        self.mean += deviation / self.count;
        self.squared_deviations += deviation * (value - self.mean);
    }

    /// The mean and its standard error, as `Decimal`s. Needs two values or more.
    fn estimate(&self) -> Result<(Decimal, Decimal), QuadraticRoughHestonError> {
        let variance_of_mean = self.squared_deviations / (self.count - 1.0) / self.count;
        Ok((
            f64_to_decimal(self.mean)?,
            f64_to_decimal(variance_of_mean.sqrt())?, // scan-banned: allow -- f64 `sqrt`: returns NaN for negative input, it does not abort; `f64_to_decimal` rejects it
        ))
    }
}

/// Prices a European call or put on `S_T` by Monte Carlo, undiscounted, as with `r = q = 0`.
///
/// One path is drawn per seed of a master generator seeded with `seed`. The seeds are drawn
/// serially and the paths summed in their order, so the result does not depend on the number
/// of threads of the rayon pool it runs on. A path that fails makes the price fail, and the
/// failure reported is the one of the first such path. See [`QuadraticRoughHestonParams`] for
/// the model, the scheme and the treatment of a spot that goes negative.
///
/// # Errors
///
/// - [`QuadraticRoughHestonError::InvalidParameter`] for a single path, which has no standard
///   error.
/// - [`QuadraticRoughHestonError::NonFinite`] when a path stops being finite, with its step.
/// - [`QuadraticRoughHestonError::BufferTooLarge`] when `paths` cannot be allocated.
/// - [`QuadraticRoughHestonError::Decimal`] when `strike` has no `f64` value or a result does
///   not fit a `Decimal`.
#[tracing::instrument(level = "debug", skip(params))]
pub fn price_vanilla_quadratic_rough_heston(
    params: &QuadraticRoughHestonParams,
    grid: &QuadraticRoughHestonGrid,
    style: OptionStyle,
    strike: Positive,
    paths: NonZeroUsize,
    seed: u64,
) -> Result<QuadraticRoughHestonEstimate, QuadraticRoughHestonError> {
    let path_count = paths.get();
    if path_count < MIN_PATHS {
        return Err(invalid_parameter(
            "paths",
            Decimal::from(path_count),
            "a standard error needs at least two paths",
        ));
    }
    let raw_strike = decimal_to_f64(strike.to_dec())?;
    let step_sizes = raw_step(grid)?;

    let mut master = deterministic_rng(seed);
    let mut seeds = reserve::<u64>(path_count, "path seeds")?;
    seeds.extend((0..path_count).map(|_| master.random::<u64>()));

    let mut outcomes = reserve(path_count, "path outcomes")?;
    seeds
        .par_iter()
        .map(|path_seed| run_path(params, grid, step_sizes, *path_seed))
        .collect_into_vec(&mut outcomes);

    let non_positive_paths = outcomes
        .iter()
        .filter(|outcome| matches!(outcome, Ok(path) if path.non_positive))
        .count();
    let mut payoff = Moments::default();
    let mut terminal = Moments::default();
    for outcome in outcomes {
        let outcome = outcome.map_err(NonFiniteValue::into_error)?;
        payoff.add(match style {
            OptionStyle::Call => (outcome.terminal_spot - raw_strike).max(0.0),
            OptionStyle::Put => (raw_strike - outcome.terminal_spot).max(0.0),
        });
        terminal.add(outcome.terminal_spot);
    }

    let (price, standard_error) = payoff.estimate()?;
    let (mean_terminal_spot, terminal_spot_standard_error) = terminal.estimate()?;
    Ok(QuadraticRoughHestonEstimate {
        price,
        standard_error,
        mean_terminal_spot,
        terminal_spot_standard_error,
        paths: path_count,
        non_positive_paths,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use optionstratlib_core::pos_or_panic;

    /// Largest difference accepted against a value computed by hand with `f64`.
    const HAND_TOLERANCE: f64 = 1e-12;

    /// `n = 2`, `alpha = 3/4`, `x = 4`: `eta = (1/4, 1, 4)`. By hand, with
    /// `G = Gamma(3/4) Gamma(1/4) = pi sqrt(2)` and `mu(dg) = g^(-3/4) dg / G`:
    /// `c_i = 4 (eta_i^(1/4) - eta_(i-1)^(1/4)) / G`, and
    /// `gamma_i = (4/5) (eta_i^(5/4) - eta_(i-1)^(5/4)) / (G c_i)`.
    const SMALL_WEIGHTS: [f64; 2] = [0.26369654378952456, 0.37292322857805654];
    const SMALL_DISCOUNTS: [f64; 2] = [0.5621320343559644, 2.2485281374238566];

    fn small_kernel() -> QuadraticRoughHestonKernel {
        QuadraticRoughHestonKernel::try_new(dec!(0.75), 2, dec!(4)).unwrap()
    }

    /// `a = 0.5`, `b = 0.1`, `c = 0.04`, `lambda = 2`, `eta = 0.5`, `S_0 = 100`, `Z^i_0 = (0.1, 0.2)`.
    fn small_params() -> QuadraticRoughHestonParams {
        QuadraticRoughHestonParams::try_new(
            dec!(0.5),
            dec!(0.1),
            dec!(0.04),
            dec!(2),
            dec!(0.5),
            small_kernel(),
            vec![dec!(0.1), dec!(0.2)],
            Positive::HUNDRED,
        )
        .unwrap()
    }

    fn close(actual: f64, expected: f64) -> bool {
        (actual - expected).abs() < HAND_TOLERANCE
    }

    #[test]
    fn test_kernel_small_case_matches_hand_values() {
        let kernel = small_kernel();
        assert_eq!(kernel.alpha(), dec!(0.75));
        assert_eq!(kernel.ratio(), dec!(4));
        assert_eq!(kernel.factors(), 2);
        for i in 0..2 {
            assert!(close(kernel.raw_weights[i], SMALL_WEIGHTS[i]), "c_{i}");
            assert!(close(kernel.raw_discounts[i], SMALL_DISCOUNTS[i]), "g_{i}");
            assert!(close(
                decimal_to_f64(kernel.weights()[i]).unwrap(),
                SMALL_WEIGHTS[i]
            ));
            assert!(close(
                decimal_to_f64(kernel.discounts()[i]).unwrap(),
                SMALL_DISCOUNTS[i]
            ));
        }
    }

    /// `n = 3` has a half-integer `n / 2`: `eta = (4^(-3/2), 4^(-1/2), 4^(1/2), 4^(3/2))`.
    #[test]
    fn test_kernel_odd_factors_match_hand_values() {
        let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.75), 3, dec!(4)).unwrap();
        let weights = [0.2217414783874054, 0.3135898060761293, 0.4434829567748108];
        let discounts = [0.2810660171779822, 1.1242640687119285, 4.497056274847715];
        for i in 0..3 {
            assert!(close(kernel.raw_weights[i], weights[i]), "c_{i}");
            assert!(close(kernel.raw_discounts[i], discounts[i]), "g_{i}");
        }
    }

    /// The one figure of the calibration paper that reproduces: `gamma_10 = 542.32`, printed
    /// to two decimals. It is a check on the formula, and not a suggested ratio.
    #[test]
    fn test_kernel_discount_matches_the_printed_figure() {
        let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.51), 10, dec!(3.92)).unwrap();
        let discount = kernel.discounts().last().copied().unwrap();
        assert!((discount - dec!(542.32)).abs() < dec!(0.005), "{discount}");
    }

    /// `K^n(t)` from the hand values, at zero and away from it.
    #[test]
    fn test_kernel_value_at_is_the_sum_of_exponentials() {
        let kernel = small_kernel();
        let at_zero = kernel.value_at(Positive::ZERO).unwrap();
        assert!(close(
            decimal_to_f64(at_zero).unwrap(),
            SMALL_WEIGHTS[0] + SMALL_WEIGHTS[1]
        ));
        let t = 0.5_f64;
        let value = kernel.value_at(pos_or_panic!(0.5)).unwrap();
        let expected = SMALL_WEIGHTS[0] * (-SMALL_DISCOUNTS[0] * t).exp()
            + SMALL_WEIGHTS[1] * (-SMALL_DISCOUNTS[1] * t).exp();
        assert!(close(decimal_to_f64(value).unwrap(), expected));
        let far = kernel.value_at(Positive::MAX).unwrap();
        assert_eq!(far, Decimal::ZERO);
    }

    #[test]
    fn test_kernel_invalid_alpha_returns_typed_error() {
        for alpha in [dec!(-1), dec!(0), dec!(0.5), dec!(1), dec!(2)] {
            assert!(
                matches!(
                    QuadraticRoughHestonKernel::try_new(alpha, 2, dec!(4)),
                    Err(QuadraticRoughHestonError::AlphaOutOfRange { alpha: rejected })
                        if rejected == alpha
                ),
                "{alpha}"
            );
        }
    }

    #[test]
    fn test_kernel_alpha_next_to_its_bounds_is_accepted() {
        let tiny = Decimal::new(1, 20);
        for alpha in [dec!(0.5) + tiny, Decimal::ONE - tiny] {
            let kernel = QuadraticRoughHestonKernel::try_new(alpha, 2, dec!(4)).unwrap();
            assert!(kernel.raw_weights.iter().all(|weight| *weight > 0.0));
        }
    }

    #[test]
    fn test_kernel_ratio_just_above_one_keeps_its_distance_from_one() {
        // An `f64` of the whole ratio rounds 1 + 1e-16 to one, and 1 + 3e-16 to 1 + 2.2e-16.
        let lowest = QuadraticRoughHestonKernel::try_new(dec!(0.75), 2, dec!(1.0000000000000001));
        assert!(lowest.is_ok());

        // With `alpha = 0.75`, `n = 2`, `i = 1`, the exponent is zero and the first weight is
        // `(1 - x^(-1/4)) sin(pi / 4) / (pi / 4)`, about `(x - 1) sin(pi / 4) / pi`.
        let kernel =
            QuadraticRoughHestonKernel::try_new(dec!(0.75), 2, dec!(1.0000000000000003)).unwrap();
        let weight = decimal_to_f64(kernel.weights()[0]).unwrap();
        let expected = 3e-16 * (PI / 4.0).sin() / PI;
        assert!(
            (weight / expected - 1.0).abs() < 1e-6,
            "weight {weight}, expected {expected}"
        );
    }

    #[test]
    fn test_kernel_no_factors_returns_typed_error() {
        assert!(matches!(
            QuadraticRoughHestonKernel::try_new(dec!(0.75), 0, dec!(4)),
            Err(QuadraticRoughHestonError::NoFactors)
        ));
    }

    #[test]
    fn test_kernel_ratio_not_above_one_returns_typed_error() {
        for ratio in [dec!(-2), dec!(0), dec!(1)] {
            assert!(
                matches!(
                    QuadraticRoughHestonKernel::try_new(dec!(0.75), 2, ratio),
                    Err(QuadraticRoughHestonError::RatioNotAboveOne { ratio: rejected })
                        if rejected == ratio
                ),
                "{ratio}"
            );
        }
    }

    /// The first factor of a very wide partition has no positive value in a `Decimal`, the
    /// last has none that fits, and a ratio that reads as one in `f64` has no partition.
    #[test]
    fn test_kernel_out_of_decimal_range_returns_typed_error() {
        assert!(matches!(
            QuadraticRoughHestonKernel::try_new(dec!(0.75), 1000, dec!(4)),
            Err(QuadraticRoughHestonError::KernelOutOfRange {
                quantity: "weight",
                index: 1
            })
        ));
        assert!(matches!(
            QuadraticRoughHestonKernel::try_new(dec!(0.75), 100, dec!(4)),
            Err(QuadraticRoughHestonError::KernelOutOfRange {
                quantity: "discount",
                index: 1
            })
        ));
        assert!(matches!(
            QuadraticRoughHestonKernel::try_new(dec!(0.75), 2, Decimal::ONE + Decimal::new(1, 28)),
            Err(QuadraticRoughHestonError::KernelOutOfRange { index: 1, .. })
        ));
    }

    #[test]
    fn test_kernel_unallocatable_factors_return_typed_error() {
        assert!(matches!(
            QuadraticRoughHestonKernel::try_new(dec!(0.75), usize::MAX, dec!(4)),
            Err(QuadraticRoughHestonError::BufferTooLarge { .. })
        ));
    }

    #[test]
    fn test_params_read_the_initial_state_and_keep_the_inputs() {
        let params = small_params();
        assert_eq!(params.a(), dec!(0.5));
        assert_eq!(params.b(), dec!(0.1));
        assert_eq!(params.c(), dec!(0.04));
        assert_eq!(params.lambda(), dec!(2));
        assert_eq!(params.eta(), dec!(0.5));
        assert_eq!(params.kernel(), &small_kernel());
        assert_eq!(params.initial_factors(), &[dec!(0.1), dec!(0.2)]);
        assert_eq!(params.spot(), Positive::HUNDRED);
        // By hand: `Z_0 = c_1 0.1 + c_2 0.2` and `V_0 = 0.5 (Z_0 - 0.1)^2 + 0.04`.
        assert!(close(
            decimal_to_f64(params.initial_z()).unwrap(),
            0.10095430009456377
        ));
        assert!(close(
            decimal_to_f64(params.initial_variance()).unwrap(),
            0.040000455344335245
        ));
    }

    #[test]
    fn test_params_invalid_values_return_typed_errors() {
        let build = |a, c, lambda, eta, spot| {
            QuadraticRoughHestonParams::try_new(
                a,
                dec!(0.1),
                c,
                lambda,
                eta,
                small_kernel(),
                vec![dec!(0.1), dec!(0.2)],
                spot,
            )
        };
        let one = Decimal::ONE;
        let hundred = Positive::HUNDRED;
        let cases = [
            ("a", build(dec!(-0.1), one, one, one, hundred)),
            ("c", build(one, Decimal::ZERO, one, one, hundred)),
            ("c", build(one, dec!(-1), one, one, hundred)),
            ("lambda", build(one, one, Decimal::ZERO, one, hundred)),
            ("eta", build(one, one, one, Decimal::ZERO, hundred)),
            ("spot", build(one, one, one, one, Positive::ZERO)),
        ];
        for (expected, result) in cases {
            assert!(
                matches!(
                    result,
                    Err(QuadraticRoughHestonError::InvalidParameter { parameter, .. })
                        if parameter == expected
                ),
                "{expected}"
            );
        }
    }

    #[test]
    fn test_params_wrong_factor_count_returns_typed_error() {
        let result = QuadraticRoughHestonParams::try_new(
            dec!(0.5),
            dec!(0.1),
            dec!(0.04),
            dec!(2),
            dec!(0.5),
            small_kernel(),
            vec![dec!(0.1)],
            Positive::HUNDRED,
        );
        assert!(matches!(
            result,
            Err(QuadraticRoughHestonError::InitialFactorCount {
                expected: 2,
                actual: 1
            })
        ));
    }

    /// `V_0` is finite in `f64` and overflows a `Decimal`.
    #[test]
    fn test_params_initial_variance_out_of_decimal_range_returns_typed_error() {
        let result = QuadraticRoughHestonParams::try_new(
            dec!(1000000000000000000000),
            dec!(1000000000000000000000),
            dec!(0.04),
            dec!(2),
            dec!(0.5),
            small_kernel(),
            vec![dec!(0.1), dec!(0.2)],
            Positive::HUNDRED,
        );
        assert!(matches!(result, Err(QuadraticRoughHestonError::Decimal(_))));
    }

    #[test]
    fn test_grid_keeps_the_step_and_rejects_a_non_positive_one() {
        let grid = QuadraticRoughHestonGrid::try_new(dec!(0.5), 4).unwrap();
        assert_eq!(grid.horizon(), dec!(0.5));
        assert_eq!(grid.steps(), 4);
        assert_eq!(grid.dt(), dec!(0.125));
        for (horizon, steps) in [
            (Decimal::ZERO, 4),
            (dec!(-1), 4),
            (dec!(1), 0),
            (Decimal::new(1, 28), 1000),
        ] {
            assert!(
                matches!(
                    QuadraticRoughHestonGrid::try_new(horizon, steps),
                    Err(QuadraticRoughHestonError::NonPositiveStep { .. })
                ),
                "{horizon} over {steps}"
            );
        }
    }

    /// One step against `dW = 0.01`, `dt = 0.25`, computed by hand from `SMALL_WEIGHTS`,
    /// `SMALL_DISCOUNTS` and `small_params`:
    /// `S_1 = 100 + 100 sqrt(V_0) dW`, and
    /// `Z^i_1 = (Z^i_0 - 2 Z_0 0.25 + 0.5 sqrt(V_0) dW) / (1 + gamma_i 0.25)`.
    #[test]
    fn test_step_matches_a_hand_computation_with_the_implicit_denominator() {
        let params = small_params();
        let state = params.raw_initial.clone();
        let mut next = state.clone();
        step(&params, &state, 0.01, 0.25, &mut next);
        assert!(close(next.spot, 100.20000113835759));
        assert!(close(next.factors[0], 0.044297583028316205));
        assert!(close(next.factors[1], 0.09635731956970187));
        assert!(close(next.z, 0.04761500225385726));
        assert!(close(next.variance, 0.04137209399443169));
    }

    /// A zero increment leaves the spot, and each factor keeps only its drift.
    #[test]
    fn test_step_zero_increment_keeps_the_spot() {
        let params = small_params();
        let state = params.raw_initial.clone();
        let mut next = state.clone();
        step(&params, &state, 0.0, 0.25, &mut next);
        assert_eq!(next.spot, state.spot);
        let expected = (0.1 - 2.0 * state.z * 0.25) / (1.0 + SMALL_DISCOUNTS[0] * 0.25);
        assert!(close(next.factors[0], expected));
    }

    /// Halving the step on one Brownian path, without the noise of two independent estimates:
    /// each coarse `dW` is the sum of two fine ones.
    ///
    /// `alpha = 0.6`, `n = 5`, `x = 5`, `a = 0.5`, `b = 0.1`, `c = 0.01`, `lambda = 1.5`,
    /// `eta = 1`, `Z^i_0 = -0.05`, `S_0 = K = 100`, `T = 0.25`, an ATM call, `N = 100` against
    /// `2N = 200`, on 20,000 paths. The change measures `0.0087`, with a standard error of
    /// `0.0013`, against a tolerance of `0.02`.
    ///
    /// The coarse side goes through [`Walk`], and the fine side drives [`step`] with its own
    /// `dt`. If both went through `Walk`, a wrong `dt` there would pass: it is a consistent
    /// scheme for another model, and halving its step changes the price just as little.
    #[test]
    fn test_walk_halving_the_step_on_coupled_increments_stays_within_its_tolerance() {
        const COARSE_STEPS: usize = 100;
        const PATHS: usize = 20_000;
        const TOLERANCE: f64 = 0.02;
        let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.6), 5, dec!(5)).unwrap();
        let params = QuadraticRoughHestonParams::try_new(
            dec!(0.5),
            dec!(0.1),
            dec!(0.01),
            dec!(1.5),
            dec!(1),
            kernel,
            vec![dec!(-0.05); 5],
            Positive::HUNDRED,
        )
        .unwrap();
        let coarse_grid = QuadraticRoughHestonGrid::try_new(dec!(0.25), COARSE_STEPS).unwrap();
        let coarse_step = raw_step(&coarse_grid).unwrap();
        let fine_dt = 0.25 / (2.0 * COARSE_STEPS as f64);
        let fine_sqrt_dt = fine_dt.sqrt();

        let mut rng = deterministic_rng(958);
        let mut change = Moments::default();
        for _ in 0..PATHS {
            let mut coarse = Walk::new(&params, coarse_step, 0);
            let mut fine = params.raw_initial.clone();
            let mut scratch = fine.clone();
            for to in 1..=COARSE_STEPS {
                let first: f64 = StandardNormal.sample(&mut rng);
                let second: f64 = StandardNormal.sample(&mut rng);
                let (first, second) = (fine_sqrt_dt * first, fine_sqrt_dt * second);
                step(&params, &fine, first, fine_dt, &mut scratch);
                step(&params, &scratch, second, fine_dt, &mut fine);
                coarse.apply(first + second, to).unwrap();
            }
            let payoff = |spot: f64| (spot - 100.0).max(0.0);
            change.add(payoff(coarse.state.spot) - payoff(fine.spot));
        }
        assert!(change.mean.abs() < TOLERANCE, "{change:?}");
    }

    #[test]
    fn test_state_not_finite_names_its_quantity_and_step() {
        let finite = small_params().raw_initial;
        assert_eq!(finite.ensure_finite(3), Ok(()));
        for (quantity, state) in [
            (
                "spot",
                State {
                    spot: f64::NAN,
                    ..finite.clone()
                },
            ),
            (
                "z",
                State {
                    z: f64::INFINITY,
                    ..finite.clone()
                },
            ),
            (
                "variance",
                State {
                    variance: f64::INFINITY,
                    ..finite.clone()
                },
            ),
        ] {
            assert_eq!(
                state.ensure_finite(3),
                Err(NonFiniteValue { quantity, step: 3 })
            );
        }
    }

    #[test]
    fn test_moments_match_the_sample_mean_and_standard_error() {
        let mut moments = Moments::default();
        for value in [1.0, 2.0, 3.0, 4.0] {
            moments.add(value);
        }
        let (mean, standard_error) = moments.estimate().unwrap();
        assert_eq!(mean, dec!(2.5));
        // Sample variance 5/3, over 4 values.
        let expected = (5.0_f64 / 3.0 / 4.0).sqrt();
        assert!(close(decimal_to_f64(standard_error).unwrap(), expected));
    }
}
