/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 7/11/26
******************************************************************************/

//! The VIX under the multi-factor quadratic rough Heston model: the index as a function of
//! the factor state, from the conditional moments of the variance, and Monte Carlo prices of
//! VIX futures and options on it. [`QuadraticRoughHestonVix`] documents the method.

use super::quadratic_rough_heston::{
    Moments, QuadraticRoughHestonGrid, QuadraticRoughHestonParams, Schedule, State,
    invalid_parameter, reserve, run_paths,
};
use crate::error::QuadraticRoughHestonError;
use num_traits::ToPrimitive;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{d_div, decimal_to_f64, f64_to_decimal};
use optionstratlib_core::model::types::OptionStyle;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::num::NonZeroUsize;

/// The VIX window, in days: the index measures the variance over the next 30 days.
pub const VIX_WINDOW_DAYS: Decimal = dec!(30);

/// Days in a year, for the window in years of [`QuadraticRoughHestonVix::thirty_days`].
const DAYS_IN_YEAR: Decimal = dec!(365);

/// `100^2`: the index is quoted in volatility points.
const INDEX_SCALE: f64 = 10_000.0;

/// The scaled matrix of the exponential has a 1-norm at most this. With it, the Taylor series
/// of degree [`EXPM_TAYLOR_DEGREE`] has a remainder below `2e-23` in norm.
const EXPM_SCALED_NORM: f64 = 0.5;

/// The degree of the Taylor polynomial of the scaled exponential.
const EXPM_TAYLOR_DEGREE: usize = 18;

/// A square matrix in `f64`, row-major.
#[derive(Debug, Clone, PartialEq)]
struct Matrix {
    dim: usize,
    entries: Vec<f64>,
}

impl Matrix {
    /// `dim` must be positive: the row iterators below are `chunks_exact(dim)`, and the
    /// moment system has at least its integral and its constant.
    fn zeros(dim: usize) -> Result<Self, QuadraticRoughHestonError> {
        debug_assert!(dim > 0, "a matrix needs a positive dimension");
        let len = dim
            .checked_mul(dim)
            .ok_or(QuadraticRoughHestonError::BufferTooLarge {
                buffer: "moment matrix",
                len: dim,
            })?;
        let mut entries = reserve(len, "moment matrix")?;
        entries.resize(len, 0.0);
        Ok(Self { dim, entries })
    }

    fn identity(dim: usize) -> Result<Self, QuadraticRoughHestonError> {
        let mut identity = Self::zeros(dim)?;
        identity.add_diagonal(1.0);
        Ok(identity)
    }

    /// The row `row`, or an empty slice past the end.
    #[inline]
    fn row(&self, row: usize) -> &[f64] {
        self.entries.chunks_exact(self.dim).nth(row).unwrap_or(&[])
    }

    /// The entry at `row`, `column`, or `NaN` past the end.
    #[cfg(test)]
    fn get(&self, row: usize, column: usize) -> f64 {
        self.row(row).get(column).copied().unwrap_or(f64::NAN)
    }

    /// The entry at `row`, `column`, or `None` past the end.
    #[inline]
    fn entry_mut(&mut self, row: usize, column: usize) -> Option<&mut f64> {
        self.entries
            .chunks_exact_mut(self.dim)
            .nth(row)
            .and_then(|entries| entries.get_mut(column))
    }

    #[inline]
    fn set(&mut self, row: usize, column: usize, value: f64) {
        if let Some(entry) = self.entry_mut(row, column) {
            *entry = value;
        }
    }

    #[inline]
    fn add_to(&mut self, row: usize, column: usize, value: f64) {
        if let Some(entry) = self.entry_mut(row, column) {
            *entry += value;
        }
    }

    fn add_diagonal(&mut self, value: f64) {
        for (index, row) in self.entries.chunks_exact_mut(self.dim).enumerate() {
            if let Some(entry) = row.get_mut(index) {
                *entry += value;
            }
        }
    }

    /// The 1-norm: the largest column sum of absolute values.
    fn norm_1(&self) -> f64 {
        let mut sums = vec![0.0_f64; self.dim];
        for row in self.entries.chunks_exact(self.dim) {
            for (sum, entry) in sums.iter_mut().zip(row) {
                *sum += entry.abs();
            }
        }
        sums.into_iter().fold(0.0, f64::max)
    }

    fn scale(&mut self, factor: f64) {
        for entry in &mut self.entries {
            *entry *= factor;
        }
    }

    /// `self * other`, into `into`, a scratch of the same dimension.
    fn multiply_into(&self, other: &Self, into: &mut Self) {
        let dim = self.dim;
        into.entries.fill(0.0);
        for (left_row, out_row) in self
            .entries
            .chunks_exact(dim)
            .zip(into.entries.chunks_exact_mut(dim))
        {
            for (left, right_row) in left_row.iter().zip(other.entries.chunks_exact(dim)) {
                if *left == 0.0 {
                    continue;
                }
                for (out, right) in out_row.iter_mut().zip(right_row) {
                    *out += left * right;
                }
            }
        }
    }

    fn is_finite(&self) -> bool {
        self.entries.iter().all(|entry| entry.is_finite())
    }

    /// The matrix exponential by scaling and squaring with a Taylor polynomial: `self / 2^s`
    /// has a 1-norm at most [`EXPM_SCALED_NORM`], its exponential is the polynomial of degree
    /// [`EXPM_TAYLOR_DEGREE`], and the result is squared `s` times.
    ///
    /// Returns `None` when the matrix is not finite, or so large that its scaling has no
    /// representable exponent.
    fn exponential(&self) -> Result<Option<Self>, QuadraticRoughHestonError> {
        let dim = self.dim;
        let norm = self.norm_1();
        if !norm.is_finite() {
            return Ok(None);
        }
        let squarings = if norm > EXPM_SCALED_NORM {
            (norm / EXPM_SCALED_NORM)
                .log2() // scan-banned: allow -- f64 `log2` of a value above one: positive and finite, and it does not abort
                .ceil()
                .to_i32()
        } else {
            Some(0)
        };
        let Some(squarings) = squarings.filter(|count| *count >= 0) else {
            return Ok(None);
        };
        let mut scaled = self.clone();
        scaled.scale(0.5_f64.powi(squarings));

        // Horner: `I + A (I + A/2 (I + A/3 (...)))`.
        let mut result = Self::identity(dim)?;
        let mut scratch = Self::zeros(dim)?;
        for degree in (1..=EXPM_TAYLOR_DEGREE).rev() {
            scaled.multiply_into(&result, &mut scratch);
            scratch.scale(1.0 / degree as f64);
            scratch.add_diagonal(1.0);
            std::mem::swap(&mut result, &mut scratch);
        }
        let mut squared = Self::zeros(dim)?;
        for _ in 0..squarings {
            result.multiply_into(&result, &mut squared);
            std::mem::swap(&mut result, &mut squared);
        }
        Ok(Some(result))
    }
}

/// The column of each second moment `M_ij` in the state vector, as a table by `i` and `j`:
/// the pairs `i <= j` are packed by rows, `(0,0), (0,1), ..., (0,n-1), (1,1), ...`, after the
/// `n` first moments, and the table is symmetric.
fn pair_columns(factors: usize) -> Result<Vec<Vec<usize>>, QuadraticRoughHestonError> {
    let mut table = reserve(factors, "moment columns")?;
    for _ in 0..factors {
        let mut row = reserve(factors, "moment columns")?;
        row.resize(factors, 0);
        table.push(row);
    }
    let pairs = (0..factors).flat_map(|i| (i..factors).map(move |j| (i, j)));
    for (column, (i, j)) in (factors..).zip(pairs) {
        if let Some(entry) = table.get_mut(i).and_then(|row| row.get_mut(j)) {
            *entry = column;
        }
        if let Some(entry) = table.get_mut(j).and_then(|row| row.get_mut(i)) {
            *entry = column;
        }
    }
    Ok(table)
}

/// The VIX of the model as a function of the factor state.
///
/// # Definition
///
/// At a date `t` with factors `Z^i_t`, over a window of `Delta` years,
///
/// ```text
/// VIX_t^2 = (100^2 / Delta) E[ int_t^(t+Delta) V_s ds | F_t ]
/// ```
///
/// which is the index in volatility points, 20 for a variance swap rate of 4% per year. The
/// model has no jumps and no rates, so this is the variance swap rate, and it depends on the
/// factors alone: the spot plays no part.
///
/// # Method
///
/// The factors are affine, `dZ^i = (-gamma_i Z^i - lambda Z) dt + eta sqrt(V) dW`, and the
/// variance `V = a (Z - b)^2 + c` is quadratic in them, so the conditional first moments
/// `m_i(s) = E[Z^i_s | F_t]` and second moments `M_ij(s) = E[Z^i_s Z^j_s | F_t]` solve a
/// closed linear system:
///
/// ```text
/// m_i'  = -gamma_i m_i - lambda sum_k c_k m_k
/// M_ij' = -(gamma_i + gamma_j) M_ij - lambda sum_k c_k (M_kj + M_ik) + eta^2 E[V_s | F_t]
/// E[V_s | F_t] = a (sum_kl c_k c_l M_kl - 2 b sum_k c_k m_k + b^2) + c
/// ```
///
/// from `m_i(0) = Z^i_t` and `M_ij(0) = Z^i_t Z^j_t`. The system, with the running integral
/// of `E[V_s | F_t]` and a constant appended, is `y' = G y` for a constant `G`, so
/// `y(Delta) = exp(G Delta) y(0)`, and the integral is one row of the exponential applied to
/// the initial state. That row makes `VIX_t^2` a quadratic form in the factors, which
/// [`try_new`](Self::try_new) computes once per parameter set, by the scaling and squaring of
/// a Taylor polynomial, and [`squared_at`](Self::squared_at) evaluates in `O(n^2)`. No Monte
/// Carlo and no nested simulation is involved, and the value is the model's, not the one of
/// a discretization of it.
///
/// # Cost
///
/// The exponential is of a matrix of dimension `n + n (n + 1) / 2 + 2`, so the construction is
/// `O(n^6)` in the number of factors: milliseconds for `n = 10`, seconds for `n = 30`. One
/// evaluation is `O(n^2)`.
///
/// # References
///
/// - Rosenbaum and Zhang, *Deep calibration of the quadratic rough Heston model*,
///   [arXiv:2107.01611v2](https://arxiv.org/abs/2107.01611v2), Section 2, for the model and
///   the joint SPX and VIX calibration it serves.
/// - Gatheral, Jusselin and Rosenbaum, *The quadratic rough Heston model and the joint S&P
///   500/VIX smile calibration problem*, [arXiv:2001.01789](https://arxiv.org/abs/2001.01789),
///   for the definition of the VIX in the model.
///
/// # Examples
///
/// ```rust
/// use optionstratlib_core::pos_or_panic;
/// use optionstratlib_simulation::simulation::{
///     QuadraticRoughHestonKernel, QuadraticRoughHestonParams, QuadraticRoughHestonVix,
/// };
/// use rust_decimal_macros::dec;
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
/// let vix = QuadraticRoughHestonVix::thirty_days(&params)?;
/// let today = vix.at(params.initial_factors())?;
/// // `V >= c = 0.04` everywhere, so the index is at least 20 points.
/// assert!(today >= pos_or_panic!(20.0));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct QuadraticRoughHestonVix {
    window: Positive,
    factors: usize,
    /// `100^2 / Delta`.
    scale: f64,
    /// The coefficient of each `Z^i_t`.
    linear: Vec<f64>,
    /// The coefficient of each `Z^i_t Z^j_t`, `i <= j`, packed by [`pair_columns`].
    pairs: Vec<f64>,
    constant: f64,
}

impl QuadraticRoughHestonVix {
    /// Builds the index of `params` over a window of `window` years.
    ///
    /// # Errors
    ///
    /// - [`QuadraticRoughHestonError::InvalidParameter`] when `window` is zero.
    /// - [`QuadraticRoughHestonError::VixMomentsNotFinite`] when the conditional moments grow
    ///   beyond an `f64` within the window.
    /// - [`QuadraticRoughHestonError::BufferTooLarge`] when the moment system cannot be
    ///   allocated.
    /// - [`QuadraticRoughHestonError::Decimal`] when `window` has no `f64` value.
    #[tracing::instrument(level = "debug", skip(params))]
    pub fn try_new(
        params: &QuadraticRoughHestonParams,
        window: Positive,
    ) -> Result<Self, QuadraticRoughHestonError> {
        if window.is_zero() {
            return Err(invalid_parameter(
                "window",
                window.to_dec(),
                "must be positive",
            ));
        }
        let delta = decimal_to_f64(window.to_dec())?;
        let kernel = params.kernel();
        let factors = kernel.factors();
        let weights = &kernel.raw_weights;
        let discounts = &kernel.raw_discounts;
        let raw = &params.raw;

        // The state `y = (m_1..m_n, M_11, M_12, .., M_nn, w, 1)`, with `w` the running
        // integral of `E[V]`.
        let columns = pair_columns(factors)?;
        let pair = |i: usize, j: usize| -> usize {
            columns
                .get(i)
                .and_then(|row| row.get(j))
                .copied()
                .unwrap_or(usize::MAX)
        };
        let pair_count = (0..factors).flat_map(|i| i..factors).count();
        let integral_index = factors
            .checked_add(pair_count)
            .ok_or_else(|| too_many_factors(factors))?;
        let one_index = integral_index
            .checked_add(1)
            .ok_or_else(|| too_many_factors(factors))?;
        let dim = one_index
            .checked_add(1)
            .ok_or_else(|| too_many_factors(factors))?;

        let mut generator = Matrix::zeros(dim)?;
        // First moments: `m_i' = -gamma_i m_i - lambda sum_k c_k m_k`.
        for i in 0..factors {
            generator.add_to(i, i, -discounts.get(i).copied().unwrap_or(f64::NAN));
            for (k, weight) in weights.iter().enumerate() {
                generator.add_to(i, k, -raw.lambda * weight);
            }
        }
        // `E[V]` as a row over the state: `a sum_kl c_k c_l M_kl - 2 a b sum_k c_k m_k`, and
        // `a b^2 + c` on the constant.
        let mut variance_row = vec![0.0_f64; dim];
        for (k, weight_k) in weights.iter().enumerate() {
            if let Some(entry) = variance_row.get_mut(k) {
                *entry = -2.0 * raw.a * raw.b * weight_k;
            }
            for (l, weight_l) in weights.iter().enumerate().skip(k) {
                let multiplicity = if k == l { 1.0 } else { 2.0 };
                if let Some(entry) = variance_row.get_mut(pair(k, l)) {
                    *entry = multiplicity * raw.a * weight_k * weight_l;
                }
            }
        }
        if let Some(entry) = variance_row.get_mut(one_index) {
            *entry = raw.a * raw.b * raw.b + raw.c;
        }
        // Second moments: `M_ij' = -(gamma_i + gamma_j) M_ij - lambda sum_k c_k (M_kj + M_ik)
        // + eta^2 E[V]`.
        let eta_squared = raw.eta * raw.eta;
        for i in 0..factors {
            for j in i..factors {
                let row = pair(i, j);
                let decay = discounts.get(i).copied().unwrap_or(f64::NAN)
                    + discounts.get(j).copied().unwrap_or(f64::NAN);
                generator.add_to(row, row, -decay);
                for (k, weight) in weights.iter().enumerate() {
                    generator.add_to(row, pair(k, j), -raw.lambda * weight);
                    generator.add_to(row, pair(i, k), -raw.lambda * weight);
                }
                for (column, coefficient) in variance_row.iter().enumerate() {
                    generator.add_to(row, column, eta_squared * coefficient);
                }
            }
        }
        // The integral: `w' = E[V]`. The constant has no dynamics.
        for (column, coefficient) in variance_row.iter().enumerate() {
            generator.set(integral_index, column, *coefficient);
        }

        generator.scale(delta);
        let exponential = generator
            .exponential()?
            .filter(Matrix::is_finite)
            .ok_or_else(|| QuadraticRoughHestonError::VixMomentsNotFinite {
                window: window.to_dec(),
            })?;
        let integral_row = exponential.row(integral_index);
        let linear = integral_row.get(..factors).unwrap_or(&[]).to_vec();
        let pairs = integral_row
            .get(factors..integral_index)
            .unwrap_or(&[])
            .to_vec();
        let constant = integral_row.get(one_index).copied().unwrap_or(f64::NAN);
        if linear.len() != factors || pairs.len() != pair_count || !constant.is_finite() {
            return Err(QuadraticRoughHestonError::VixMomentsNotFinite {
                window: window.to_dec(),
            });
        }

        Ok(Self {
            window,
            factors,
            scale: INDEX_SCALE / delta,
            linear,
            pairs,
            constant,
        })
    }

    /// Builds the index of `params` over the standard window of [`VIX_WINDOW_DAYS`] days,
    /// `30 / 365` years.
    ///
    /// # Errors
    ///
    /// As [`try_new`](Self::try_new).
    pub fn thirty_days(
        params: &QuadraticRoughHestonParams,
    ) -> Result<Self, QuadraticRoughHestonError> {
        let window = d_div(
            VIX_WINDOW_DAYS,
            DAYS_IN_YEAR,
            "quadratic_rough_heston::vix::window",
        )?;
        let window = Positive::new_decimal(window)
            .map_err(|_| invalid_parameter("window", window, "must be positive"))?;
        Self::try_new(params, window)
    }

    /// The window `Delta`, in years.
    #[must_use]
    #[inline]
    pub fn window(&self) -> Positive {
        self.window
    }

    /// The number of factors `n` of the state the index reads.
    #[must_use]
    #[inline]
    pub fn factors(&self) -> usize {
        self.factors
    }

    /// `VIX_t^2`, in squared volatility points, at the raw factor state.
    ///
    /// `factors` must hold `n` values: the sum runs over what it holds.
    #[inline]
    pub(super) fn raw_squared(&self, factors: &[f64]) -> f64 {
        let mut value = self.constant;
        for (coefficient, factor) in self.linear.iter().zip(factors) {
            value += coefficient * factor;
        }
        let mut pairs = self.pairs.iter();
        for (i, factor_i) in factors.iter().enumerate() {
            for factor_j in factors.iter().skip(i) {
                let Some(coefficient) = pairs.next() else {
                    return f64::NAN;
                };
                value += coefficient * factor_i * factor_j;
            }
        }
        self.scale * value
    }

    /// `VIX_t^2` at the factor state `factors`, the `Z^i_t`, in squared volatility points.
    ///
    /// # Errors
    ///
    /// - [`QuadraticRoughHestonError::InitialFactorCount`] when `factors` does not hold one
    ///   value per factor.
    /// - [`QuadraticRoughHestonError::VixNotPositive`] when the value is not finite or not
    ///   above zero, which a finite state cannot produce: `V >= c > 0`.
    /// - [`QuadraticRoughHestonError::Decimal`] when a factor has no `f64` value or the result
    ///   does not fit a `Decimal`.
    pub fn squared_at(&self, factors: &[Decimal]) -> Result<Decimal, QuadraticRoughHestonError> {
        Ok(f64_to_decimal(self.checked_squared_at(factors)?)?)
    }

    /// `VIX_t` at the factor state `factors`, the `Z^i_t`, in volatility points.
    ///
    /// # Errors
    ///
    /// As [`squared_at`](Self::squared_at).
    pub fn at(&self, factors: &[Decimal]) -> Result<Positive, QuadraticRoughHestonError> {
        let squared = self.checked_squared_at(factors)?;
        let index = f64_to_decimal(squared.sqrt())?; // scan-banned: allow -- f64 `sqrt` of a positive finite value: finite, and it does not abort
        Positive::new_decimal(index).map_err(|_| QuadraticRoughHestonError::VixNotPositive {
            reason: "the square root is not positive",
        })
    }

    /// `VIX_t^2` in `f64`, validated.
    fn checked_squared_at(&self, factors: &[Decimal]) -> Result<f64, QuadraticRoughHestonError> {
        if factors.len() != self.factors {
            return Err(QuadraticRoughHestonError::InitialFactorCount {
                expected: self.factors,
                actual: factors.len(),
            });
        }
        let mut raw = reserve(factors.len(), "vix factors")?;
        for factor in factors {
            raw.push(decimal_to_f64(*factor)?);
        }
        checked_vix_squared(self.raw_squared(&raw))
    }
}

/// `squared` when it is finite and positive, or the typed error.
#[inline]
pub(super) fn checked_vix_squared(squared: f64) -> Result<f64, QuadraticRoughHestonError> {
    if !squared.is_finite() {
        return Err(QuadraticRoughHestonError::VixNotPositive {
            reason: "the conditional variance is not finite",
        });
    }
    if squared <= 0.0 {
        return Err(QuadraticRoughHestonError::VixNotPositive {
            reason: "the conditional variance is not above zero",
        });
    }
    Ok(squared)
}

#[cold]
#[inline(never)]
fn too_many_factors(factors: usize) -> QuadraticRoughHestonError {
    QuadraticRoughHestonError::BufferTooLarge {
        buffer: "moment system",
        len: factors,
    }
}

/// What a VIX instrument pays at its maturity `T`, in volatility points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuadraticRoughHestonVixPayoff {
    /// The future: `VIX_T` itself, so the price is `E[VIX_T]`.
    Future,
    /// A vanilla on the index: `(VIX_T - K)^+` for a call, `(K - VIX_T)^+` for a put.
    Vanilla {
        /// Call or put.
        style: OptionStyle,
        /// The strike `K`, in volatility points.
        strike: Positive,
    },
}

/// The Monte Carlo price of a VIX instrument, with the future on the same paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuadraticRoughHestonVixEstimate {
    price: Decimal,
    standard_error: Decimal,
    future: Decimal,
    future_standard_error: Decimal,
    paths: usize,
}

impl QuadraticRoughHestonVixEstimate {
    /// The mean payoff, in volatility points, undiscounted.
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

    /// The future `E[VIX_T]` on the same paths, in volatility points. It is the price when
    /// the payoff is [`QuadraticRoughHestonVixPayoff::Future`].
    #[must_use]
    #[inline]
    pub fn future(&self) -> Decimal {
        self.future
    }

    /// The standard error of [`future`](Self::future).
    #[must_use]
    #[inline]
    pub fn future_standard_error(&self) -> Decimal {
        self.future_standard_error
    }

    /// The number of simulated paths.
    #[must_use]
    #[inline]
    pub fn paths(&self) -> usize {
        self.paths
    }
}

/// Prices a VIX future or a vanilla on the index by Monte Carlo, undiscounted, as with
/// `r = q = 0`.
///
/// The factors are simulated to the horizon `T` of `grid` by the scheme (A.1), and on each
/// path `VIX_T` is read from the terminal factors by the [`QuadraticRoughHestonVix`] of
/// `params` over `window` years. The paths are those of
/// [`price_vanilla_quadratic_rough_heston`](super::price_vanilla_quadratic_rough_heston) at
/// the same `seed`, so a vanilla on the spot and an instrument on the index at one seed
/// share their noise. The estimate carries the future on the same paths whatever the
/// payoff. See [`QuadraticRoughHestonParams`] for the scheme and the caution on one run.
///
/// # Errors
///
/// - [`QuadraticRoughHestonError::InvalidParameter`] for a single path, which has no standard
///   error, or a zero `window`.
/// - [`QuadraticRoughHestonError::VixMomentsNotFinite`] when the conditional moments grow
///   beyond an `f64` within the window.
/// - [`QuadraticRoughHestonError::NonFinite`] when a path stops being finite, with its step.
/// - [`QuadraticRoughHestonError::VixNotPositive`] when the index of a path has no positive
///   finite value.
/// - [`QuadraticRoughHestonError::BufferTooLarge`] when `paths` cannot be allocated.
/// - [`QuadraticRoughHestonError::Decimal`] when a strike or the window has no `f64` value, or
///   a result does not fit a `Decimal`.
#[tracing::instrument(level = "debug", skip(params))]
pub fn price_vix_quadratic_rough_heston(
    params: &QuadraticRoughHestonParams,
    grid: &QuadraticRoughHestonGrid,
    window: Positive,
    payoff: QuadraticRoughHestonVixPayoff,
    paths: NonZeroUsize,
    seed: u64,
) -> Result<QuadraticRoughHestonVixEstimate, QuadraticRoughHestonError> {
    let vix = QuadraticRoughHestonVix::try_new(params, window)?;
    let raw_payoff = match payoff {
        QuadraticRoughHestonVixPayoff::Future => None,
        QuadraticRoughHestonVixPayoff::Vanilla { style, strike } => {
            Some((style, decimal_to_f64(strike.to_dec())?))
        }
    };
    let schedule = Schedule::from_grid(grid)?;
    let runs = run_paths(
        params,
        &schedule,
        paths,
        seed,
        |_, state: &State, squared: &mut f64| {
            *squared = vix.raw_squared(&state.factors);
        },
    )?;

    let mut payoffs = Moments::default();
    let mut futures = Moments::default();
    for run in &runs {
        let index = checked_vix_squared(run.observed)?.sqrt(); // scan-banned: allow -- f64 `sqrt` of a positive finite value: finite, and it does not abort
        payoffs.add(match raw_payoff {
            None => index,
            Some((OptionStyle::Call, strike)) => (index - strike).max(0.0),
            Some((OptionStyle::Put, strike)) => (strike - index).max(0.0),
        });
        futures.add(index);
    }

    let (price, standard_error) = payoffs.estimate()?;
    let (future, future_standard_error) = futures.estimate()?;
    Ok(QuadraticRoughHestonVixEstimate {
        price,
        standard_error,
        future,
        future_standard_error,
        paths: runs.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::QuadraticRoughHestonKernel;
    use optionstratlib_core::pos_or_panic;

    /// Largest difference accepted against a value computed by hand or by an independent
    /// integration, relative.
    const RELATIVE_TOLERANCE: f64 = 1e-9;

    fn window() -> Positive {
        pos_or_panic!(30.0 / 365.0)
    }

    fn params(
        a: Decimal,
        b: Decimal,
        c: Decimal,
        lambda: Decimal,
        eta: Decimal,
        kernel: QuadraticRoughHestonKernel,
        initial: Vec<Decimal>,
    ) -> QuadraticRoughHestonParams {
        QuadraticRoughHestonParams::try_new(
            a,
            b,
            c,
            lambda,
            eta,
            kernel,
            initial,
            Positive::HUNDRED,
        )
        .unwrap()
    }

    fn close(actual: f64, expected: f64) -> bool {
        (actual - expected).abs() <= RELATIVE_TOLERANCE * expected.abs().max(1.0)
    }

    #[test]
    fn test_pair_columns_enumerate_the_upper_triangle_by_rows_after_the_first_moments() {
        let n = 4;
        let columns = pair_columns(n).unwrap();
        let mut expected = n;
        for (i, row) in columns.iter().enumerate() {
            for (j, column) in row.iter().enumerate().skip(i) {
                assert_eq!(*column, expected, "({i}, {j})");
                assert_eq!(columns[j][i], expected, "({j}, {i})");
                expected += 1;
            }
        }
        assert_eq!(expected, n + n * (n + 1) / 2);
        assert!(pair_columns(0).unwrap().is_empty());
    }

    #[test]
    fn test_matrix_exp_of_a_diagonal_matrix_is_the_exponential_of_its_entries() {
        let mut matrix = Matrix::zeros(3).unwrap();
        for (index, value) in [-2.0, 0.5, 40.0].into_iter().enumerate() {
            matrix.set(index, index, value);
        }
        let exponential = matrix.exponential().unwrap().unwrap();
        for (index, value) in [-2.0_f64, 0.5, 40.0].into_iter().enumerate() {
            assert!(
                close(exponential.get(index, index), value.exp()),
                "{index}: {}",
                exponential.get(index, index)
            );
        }
        assert_eq!(exponential.get(0, 1), 0.0);
        assert_eq!(exponential.get(2, 0), 0.0);
    }

    /// `exp([[0, t], [0, 0]]) = [[1, t], [0, 1]]`, and a rotation generator gives a rotation.
    #[test]
    fn test_matrix_exp_matches_closed_forms() {
        let mut nilpotent = Matrix::zeros(2).unwrap();
        nilpotent.set(0, 1, 7.5);
        let exponential = nilpotent.exponential().unwrap().unwrap();
        assert!(close(exponential.get(0, 0), 1.0));
        assert!(close(exponential.get(0, 1), 7.5));
        assert!(close(exponential.get(1, 0), 0.0));
        assert!(close(exponential.get(1, 1), 1.0));

        let angle = 2.0_f64;
        let mut rotation = Matrix::zeros(2).unwrap();
        rotation.set(0, 1, -angle);
        rotation.set(1, 0, angle);
        let exponential = rotation.exponential().unwrap().unwrap();
        assert!(close(exponential.get(0, 0), angle.cos()));
        assert!(close(exponential.get(0, 1), -angle.sin()));
        assert!(close(exponential.get(1, 0), angle.sin()));
        assert!(close(exponential.get(1, 1), angle.cos()));
    }

    #[test]
    fn test_matrix_exponential_of_a_non_finite_or_unscalable_matrix_is_none() {
        let mut matrix = Matrix::zeros(2).unwrap();
        matrix.set(0, 0, f64::INFINITY);
        assert!(matches!(matrix.exponential(), Ok(None)));
        matrix.set(0, 0, f64::MAX);
        assert!(matches!(matrix.exponential(), Ok(None)));
    }

    /// With `a = 0`, `V = c` whatever the state: `VIX^2 = 100^2 c` at every state.
    #[test]
    fn test_vix_zero_feedback_is_the_root_of_c() {
        let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.6), 5, dec!(5)).unwrap();
        let params = params(
            Decimal::ZERO,
            dec!(0.1),
            dec!(0.04),
            dec!(1.5),
            dec!(1),
            kernel,
            vec![dec!(-0.05); 5],
        );
        let vix = QuadraticRoughHestonVix::try_new(&params, window()).unwrap();
        for state in [
            vec![Decimal::ZERO; 5],
            vec![dec!(-0.05); 5],
            vec![dec!(3); 5],
        ] {
            let squared = decimal_to_f64(vix.squared_at(&state).unwrap()).unwrap();
            assert!(close(squared, 400.0), "{squared}");
            let index = decimal_to_f64(vix.at(&state).unwrap().to_dec()).unwrap();
            assert!(close(index, 20.0), "{index}");
        }
    }

    /// One factor and a negligible `eta`: `Z_s = c_1 Z^1_0 exp(-kappa s)` with
    /// `kappa = gamma_1 + lambda c_1`, and the integral of `a (Z_s - b)^2 + c` over the window
    /// has a closed form. `eta = 1e-7` contributes `O(eta^2)`, below the tolerance.
    #[test]
    fn test_vix_one_factor_without_noise_matches_the_closed_form() {
        let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.75), 1, dec!(4)).unwrap();
        let weight = kernel.raw_weights[0];
        let discount = kernel.raw_discounts[0];
        let (a, b, c, lambda, z0) = (0.5_f64, 0.1_f64, 0.02_f64, 2.0_f64, -0.3_f64);
        let params = params(
            dec!(0.5),
            dec!(0.1),
            dec!(0.02),
            dec!(2),
            dec!(0.0000001),
            kernel,
            vec![dec!(-0.3)],
        );
        let delta = 30.0 / 365.0;
        let vix = QuadraticRoughHestonVix::try_new(&params, window()).unwrap();

        let kappa = discount + lambda * weight;
        let amplitude = weight * z0;
        let integral = a
            * (amplitude * amplitude * (1.0 - (-2.0 * kappa * delta).exp()) / (2.0 * kappa)
                - 2.0 * b * amplitude * (1.0 - (-kappa * delta).exp()) / kappa
                + b * b * delta)
            + c * delta;
        let expected = 10_000.0 / delta * integral;
        let actual = decimal_to_f64(vix.squared_at(&[dec!(-0.3)]).unwrap()).unwrap();
        assert!(
            (actual - expected).abs() < 1e-6 * expected,
            "{actual} against {expected}"
        );
    }

    /// The quadratic form against a fourth-order Runge-Kutta integration of the same moment
    /// system, on the calibration paper's kernel, whose largest discount is `542` per year: a
    /// stiff system over a window of `0.082` years.
    #[test]
    fn test_vix_matches_an_independent_integration_of_the_moments() {
        let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.51), 10, dec!(3.92)).unwrap();
        let weights = kernel.raw_weights.clone();
        let discounts = kernel.raw_discounts.clone();
        let (a, b, c, lambda, eta) = (0.384_f64, 0.095_f64, 0.0025_f64, 1.2_f64, 0.5_f64);
        let initial: Vec<Decimal> = (0..10_i64).map(|i| Decimal::new(-10 + i * 3, 2)).collect();
        let params = params(
            dec!(0.384),
            dec!(0.095),
            dec!(0.0025),
            dec!(1.2),
            dec!(0.5),
            kernel,
            initial.clone(),
        );
        let delta = 30.0 / 365.0;
        let vix = QuadraticRoughHestonVix::try_new(&params, window()).unwrap();

        // The moment system, written out independently of the packed matrix: `m` is the
        // vector of first moments and `second` the full `n x n` matrix of second moments.
        let n = 10;
        let x: Vec<f64> = initial
            .iter()
            .map(|value| decimal_to_f64(*value).unwrap())
            .collect();
        let expected_variance = |m: &[f64], second: &[f64]| {
            let mut quadratic = 0.0;
            for k in 0..n {
                for l in 0..n {
                    quadratic += weights[k] * weights[l] * second[k * n + l];
                }
            }
            let linear: f64 = (0..n).map(|k| weights[k] * m[k]).sum();
            a * (quadratic - 2.0 * b * linear + b * b) + c
        };
        let derivative = |m: &[f64], second: &[f64]| {
            let z: f64 = (0..n).map(|k| weights[k] * m[k]).sum();
            let dm: Vec<f64> = (0..n).map(|i| -discounts[i] * m[i] - lambda * z).collect();
            let variance = expected_variance(m, second);
            let mut dsecond = vec![0.0; n * n];
            for i in 0..n {
                for j in 0..n {
                    let mut value = -(discounts[i] + discounts[j]) * second[i * n + j];
                    for k in 0..n {
                        value -= lambda * weights[k] * (second[k * n + j] + second[i * n + k]);
                    }
                    dsecond[i * n + j] = value + eta * eta * variance;
                }
            }
            (dm, dsecond, variance)
        };
        let mut m = x.clone();
        let mut second: Vec<f64> = (0..n * n)
            .map(|index| x[index / n] * x[index % n])
            .collect();
        let mut integral = 0.0;
        let steps = 200_000;
        let h = delta / steps as f64;
        let axpy = |base: &[f64], slope: &[f64], scale: f64| -> Vec<f64> {
            base.iter()
                .zip(slope)
                .map(|(value, rate)| value + scale * rate)
                .collect()
        };
        for _ in 0..steps {
            let (k1m, k1s, k1w) = derivative(&m, &second);
            let (k2m, k2s, k2w) =
                derivative(&axpy(&m, &k1m, h / 2.0), &axpy(&second, &k1s, h / 2.0));
            let (k3m, k3s, k3w) =
                derivative(&axpy(&m, &k2m, h / 2.0), &axpy(&second, &k2s, h / 2.0));
            let (k4m, k4s, k4w) = derivative(&axpy(&m, &k3m, h), &axpy(&second, &k3s, h));
            for i in 0..n {
                m[i] += h / 6.0 * (k1m[i] + 2.0 * k2m[i] + 2.0 * k3m[i] + k4m[i]);
            }
            for index in 0..n * n {
                second[index] +=
                    h / 6.0 * (k1s[index] + 2.0 * k2s[index] + 2.0 * k3s[index] + k4s[index]);
            }
            integral += h / 6.0 * (k1w + 2.0 * k2w + 2.0 * k3w + k4w);
        }
        let expected = 10_000.0 / delta * integral;
        let actual = decimal_to_f64(vix.squared_at(&initial).unwrap()).unwrap();
        assert!(
            (actual - expected).abs() < 1e-7 * expected,
            "{actual} against {expected}"
        );
        // The index is in volatility points, and above `100 sqrt(c) = 5`.
        let index = decimal_to_f64(vix.at(&initial).unwrap().to_dec()).unwrap();
        assert!(close(index, actual.sqrt()));
        assert!(index > 5.0, "{index}");
    }

    #[test]
    fn test_vix_keeps_its_window_and_factor_count() {
        let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.6), 3, dec!(5)).unwrap();
        let params = params(
            dec!(0.5),
            dec!(0.1),
            dec!(0.04),
            dec!(1.5),
            dec!(1),
            kernel,
            vec![Decimal::ZERO; 3],
        );
        let vix = QuadraticRoughHestonVix::thirty_days(&params).unwrap();
        // `30 / 365` as a `Decimal`, to its 28 places.
        let window = Positive::new_decimal(dec!(30) / dec!(365)).unwrap();
        assert_eq!(vix.window(), window);
        assert_eq!(vix.factors(), 3);
        assert_eq!(
            vix,
            QuadraticRoughHestonVix::try_new(&params, window).unwrap()
        );
    }

    #[test]
    fn test_vix_zero_window_returns_typed_error() {
        let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.6), 2, dec!(5)).unwrap();
        let params = params(
            dec!(0.5),
            dec!(0.1),
            dec!(0.04),
            dec!(1.5),
            dec!(1),
            kernel,
            vec![Decimal::ZERO; 2],
        );
        assert!(matches!(
            QuadraticRoughHestonVix::try_new(&params, Positive::ZERO),
            Err(QuadraticRoughHestonError::InvalidParameter {
                parameter: "window",
                ..
            })
        ));
    }

    #[test]
    fn test_vix_wrong_factor_count_returns_typed_error() {
        let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.6), 2, dec!(5)).unwrap();
        let params = params(
            dec!(0.5),
            dec!(0.1),
            dec!(0.04),
            dec!(1.5),
            dec!(1),
            kernel,
            vec![Decimal::ZERO; 2],
        );
        let vix = QuadraticRoughHestonVix::try_new(&params, window()).unwrap();
        assert!(matches!(
            vix.squared_at(&[Decimal::ZERO]),
            Err(QuadraticRoughHestonError::InitialFactorCount {
                expected: 2,
                actual: 1
            })
        ));
        assert!(matches!(
            vix.at(&[Decimal::ZERO; 3]),
            Err(QuadraticRoughHestonError::InitialFactorCount {
                expected: 2,
                actual: 3
            })
        ));
    }

    /// `eta` so large that the second moments grow beyond an `f64` within the window.
    #[test]
    fn test_vix_exploding_moments_return_typed_error() {
        let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.6), 2, dec!(5)).unwrap();
        let params = params(
            dec!(1000000),
            Decimal::ZERO,
            dec!(0.04),
            dec!(1),
            dec!(1000000),
            kernel,
            vec![Decimal::ZERO; 2],
        );
        assert!(matches!(
            QuadraticRoughHestonVix::try_new(&params, window()),
            Err(QuadraticRoughHestonError::VixMomentsNotFinite { .. })
        ));
    }

    #[test]
    fn test_checked_vix_squared_rejects_non_finite_and_non_positive_values() {
        assert!(checked_vix_squared(1.5).is_ok());
        for value in [f64::NAN, f64::INFINITY, 0.0, -1.0] {
            assert!(
                matches!(
                    checked_vix_squared(value),
                    Err(QuadraticRoughHestonError::VixNotPositive { .. })
                ),
                "{value}"
            );
        }
    }

    #[test]
    fn test_vix_state_beyond_decimal_returns_typed_error() {
        let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.6), 2, dec!(5)).unwrap();
        let params = params(
            dec!(0.5),
            dec!(0.1),
            dec!(0.04),
            dec!(1.5),
            dec!(1),
            kernel,
            vec![Decimal::ZERO; 2],
        );
        let vix = QuadraticRoughHestonVix::try_new(&params, window()).unwrap();
        assert!(matches!(
            vix.squared_at(&[Decimal::MAX, Decimal::MAX]),
            Err(QuadraticRoughHestonError::Decimal(_))
        ));
    }
}
