/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 5/11/26
******************************************************************************/

//! Raw SVI volatility slice and its weighted calibration.
//!
//! The raw SVI parametrization of Gatheral and Jacquier, "Arbitrage-free SVI
//! volatility surfaces" (arXiv:1204.0646), gives the total implied variance
//! of one expiry as a function of the log moneyness `k = ln(K / F)`:
//!
//! ```text
//! w(k) = a + b * (rho * (k - m) + sqrt((k - m)^2 + sigma^2))
//! ```
//!
//! with `b >= 0`, `|rho| < 1`, `sigma > 0` and
//! `a + b * sigma * sqrt(1 - rho^2) >= 0`, the last being the minimum of `w`.
//! The implied volatility at expiry `T` (in years) is `sqrt(w(k) / T)`.
//!
//! # Calibration method
//!
//! [`calibrate_svi`] is a quasi-explicit fit. For fixed `(m, sigma)`, `w` is
//! linear in `(a, b * rho, b)`, so the inner problem is a weighted least
//! squares in total variance, solved in closed form. The feasible set of that
//! problem is convex, so when the unconstrained solution breaks a parameter
//! bound the optimum lies on the boundary, and the solver searches the
//! boundary itself: `rho = +-rho_max`, `b = 0`, and the surface where the
//! minimum of `w` is zero. Every candidate it compares is feasible, so a bound
//! changes the fit only as much as the bound requires.
//!
//! The outer problem over `(m, ln sigma)` is not convex. It runs a
//! deterministic Nelder-Mead from a fixed grid of starts, with no random
//! numbers, and keeps the converged start with the smallest objective.
//! Identical input gives identical output on the same build.
//!
//! The kernel works in normalized units (log moneyness over its observed
//! range, total variance over its weighted mean), so the search grid and the
//! tolerance do not depend on the level of volatility, the maturity or the
//! width of the strike range. It searches `m` within one strike range beyond
//! the observed strikes and `sigma` from `1e-3` to `33` times the strike range.
//!
//! # Tolerances
//!
//! On noiseless observations from a known slice, with the default
//! configuration, the calibration tests recover the raw parameters within
//! `1e-6` and the implied volatility within `1e-8`, out to 1.5 times the
//! observed log moneyness, for asymmetric slices with few points in the wings.
//! Real quotes are not an exact SVI, and [`SviFit::rmse`] reports how far the
//! fit is from them.
//!
//! # Failure
//!
//! There is no flat fallback. A fit is checked against one declared limit,
//! [`SviCalibrationConfig::max_relative_rmse`]: its weighted RMSE in implied
//! volatility, divided by the weighted mean observed implied volatility over
//! the points with a positive weight. Above it the fit is rejected with
//! [`SviError::RmseAboveLimit`]. Being relative, the limit means the same at
//! any level of volatility.
//!
//! The least squares can still land on `b = 0` by itself, because a raw SVI
//! slice is convex in `k`: quotes shaped like a frown, or alternating around a
//! level, have no better slice than a flat one. So a flat result can only be
//! returned when the quotes are flat within this limit.
//!
//! The limit is the only judgement made. There is no heuristic on curvature or
//! structure, so a fit within the limit is not otherwise judged good, and quotes
//! with structure smaller than the limit can come back without it. The caller
//! tightens or loosens the limit to say how close is close enough.
//!
//! # Out of scope
//!
//! A calibrated slice is **not** claimed free of arbitrage. The parameter
//! bounds only keep total variance non-negative: a valid slice can still
//! admit butterfly arbitrage, and nothing here relates one expiry to another.
//! The checks of the `arbitrage` module of this crate
//! ([`check_static_arbitrage`](super::check_static_arbitrage)) test a slice or a
//! surface on a grid; they run only when called.
//!
//! # Example
//!
//! ```rust
//! use optionstratlib_core::model::Positive;
//! use optionstratlib_pricing::volatility::{
//!     SviCalibrationConfig, SviObservation, SviParameters, SviSlice, calibrate_svi,
//! };
//! use rust_decimal_macros::dec;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let forward = Positive::HUNDRED;
//! let expiry = Positive::ONE;
//! let parameters =
//!     SviParameters::try_new(dec!(0.02), dec!(0.1), dec!(-0.4), dec!(0.05), dec!(0.2))?;
//! let slice = SviSlice::try_new(parameters, forward, expiry)?;
//!
//! let mut observations = Vec::new();
//! for strike in [dec!(60), dec!(75), dec!(90), dec!(100), dec!(110), dec!(125), dec!(150)] {
//!     let strike = Positive::new_decimal(strike)?;
//!     observations.push(SviObservation {
//!         strike,
//!         implied_volatility: slice.implied_volatility_at_strike(strike)?,
//!         weight: Positive::ONE,
//!     });
//! }
//!
//! let fit = calibrate_svi(&observations, forward, expiry, &SviCalibrationConfig::default())?;
//! assert!(fit.rmse() < Positive::new_decimal(dec!(0.000001))?);
//! # Ok(())
//! # }
//! ```

use crate::error::SviError;
use optionstratlib_core::error::DecimalError;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{
    d_add, d_div, d_ln, d_mul, d_sqrt, d_sub, decimal_to_f64, f64_to_decimal,
};
use rust_decimal::Decimal;
use std::num::NonZeroU64;
use tracing::{debug, instrument, warn};

/// Observations needed to fix the five raw SVI parameters.
const MIN_OBSERVATIONS: usize = 5;

/// Negative total variance forgiven as rounding, relative to the size of the
/// terms summed: `Decimal` carries 28 significant digits, so a slice whose
/// minimum is zero can land a few units below it.
const TOTAL_VARIANCE_ROUNDING_BAND: Decimal = Decimal::from_parts(1, 0, 0, false, 24);

/// How far below the boundary `a + b * sigma * sqrt(1 - rho^2) = 0` a rounded
/// solution may land and still be moved onto it, relative to the `a` it needs
/// and before the amplification [`parameters_from_kernel`] applies. Each value
/// crosses from `f64` to `Decimal` with a rounding far below this.
const ROUNDING_REPAIR_BAND: Decimal = Decimal::from_parts(1, 0, 0, false, 12);

/// The same band as an absolute floor, for a slice so small that the rounding
/// of `Decimal` at 28 places is what remains.
const ROUNDING_REPAIR_FLOOR: Decimal = Decimal::from_parts(1, 0, 0, false, 26);

/// Rounding of the radicand `(k - m)^2 + sigma^2` at the 28 places of a
/// `Decimal`. It reaches the root divided by about twice the root, so it
/// matters at a small root: see [`SviParameters::total_variance`].
const RADICAND_ROUNDING: Decimal = Decimal::from_parts(4, 0, 0, false, 28);

/// The most the root can be off, relative to its clamped value, whatever the
/// radicand rounded to. The true root lies between `max(|x|, sigma)` and `sqrt(2)`
/// times that, and the clamp keeps the value used at or above the first, so it is
/// off by at most `sqrt(2) - 1 = 0.4142...` of it.
const ROOT_RELATIVE_ERROR: Decimal = Decimal::from_parts(42, 0, 0, false, 2);

/// Rounding of the sum itself at the 28 places of a `Decimal`, and of `b` times
/// the shape once `b` scales it. For a small `a` and `b` every band relative to
/// them falls below one unit in the last place.
const SUM_ROUNDING: Decimal = Decimal::from_parts(4, 0, 0, false, 28);

/// Largest [`SviCalibrationConfig::tolerance`]. The tolerance bounds how far the
/// search stops from the optimum, and the parameters are off accordingly: on an
/// equity skew, `rho` is `8e-5` off at this value and `0.13` off at a tolerance
/// of one, where a start stops after a single iteration.
const MAX_TOLERANCE: Decimal = Decimal::from_parts(1, 0, 0, false, 3);

/// Default [`SviCalibrationConfig::max_iterations`], for each search start.
/// Generous: the searches of the calibration tests stop well before it.
const DEFAULT_MAX_ITERATIONS: NonZeroU64 = match NonZeroU64::new(500) {
    Some(iterations) => iterations,
    None => NonZeroU64::MIN,
};

/// Default [`SviCalibrationConfig::tolerance`].
const DEFAULT_TOLERANCE: Decimal = Decimal::from_parts(1, 0, 0, false, 9);

/// Default [`SviCalibrationConfig::max_relative_rmse`]: 3% of the weighted mean
/// observed implied volatility.
const DEFAULT_MAX_RELATIVE_RMSE: Decimal = Decimal::from_parts(3, 0, 0, false, 2);

/// Validated raw SVI parameters of Gatheral and Jacquier (arXiv:1204.0646).
///
/// The fields are private: a value of this type always satisfies the bounds
/// [`SviParameters::try_new`] checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SviParameters {
    a: Decimal,
    b: Decimal,
    rho: Decimal,
    m: Decimal,
    sigma: Decimal,
}

impl SviParameters {
    /// Validates and builds raw SVI parameters.
    ///
    /// `a` is the variance level and may be negative, `b` the slope of the
    /// wings, `rho` the skew, `m` the horizontal shift in log moneyness and
    /// `sigma` the curvature at the vertex. `a` and `b` are in total variance
    /// (annualized variance times years), `b` per unit of log moneyness.
    ///
    /// # Errors
    ///
    /// Returns [`SviError::InvalidParameter`] unless `b >= 0`, `|rho| < 1` and
    /// `sigma > 0`, [`SviError::NegativeMinimumVariance`] unless
    /// `a + b * sigma * sqrt(1 - rho^2) >= 0`, and [`SviError::Decimal`] if
    /// that expression overflows.
    pub fn try_new(
        a: Decimal,
        b: Decimal,
        rho: Decimal,
        m: Decimal,
        sigma: Decimal,
    ) -> Result<Self, SviError> {
        if b.is_sign_negative() && !b.is_zero() {
            return Err(SviError::InvalidParameter {
                parameter: "b",
                value: b,
                reason: "must not be negative",
            });
        }
        if rho.abs() >= Decimal::ONE {
            return Err(SviError::InvalidParameter {
                parameter: "rho",
                value: rho,
                reason: "must lie strictly between -1 and 1",
            });
        }
        if sigma <= Decimal::ZERO {
            return Err(SviError::InvalidParameter {
                parameter: "sigma",
                value: sigma,
                reason: "must be positive",
            });
        }
        let minimum = d_add(
            a,
            minimum_above_level(b, rho, sigma)?,
            "svi::parameters::min",
        )?;
        if minimum.is_sign_negative() && !minimum.is_zero() {
            return Err(SviError::NegativeMinimumVariance { minimum });
        }
        Ok(Self {
            a,
            b,
            rho,
            m,
            sigma,
        })
    }

    /// The variance level `a`, in total variance.
    #[must_use]
    #[inline]
    pub fn a(&self) -> Decimal {
        self.a
    }

    /// The wing slope `b`, in total variance per unit of log moneyness.
    #[must_use]
    #[inline]
    pub fn b(&self) -> Decimal {
        self.b
    }

    /// The skew `rho`, strictly between -1 and 1.
    #[must_use]
    #[inline]
    pub fn rho(&self) -> Decimal {
        self.rho
    }

    /// The horizontal shift `m`, in log moneyness.
    #[must_use]
    #[inline]
    pub fn m(&self) -> Decimal {
        self.m
    }

    /// The vertex curvature `sigma`, in log moneyness.
    #[must_use]
    #[inline]
    pub fn sigma(&self) -> Decimal {
        self.sigma
    }

    /// Total variance at log moneyness `k`:
    /// `a + b * (rho * (k - m) + sqrt((k - m)^2 + sigma^2))`.
    ///
    /// # Errors
    ///
    /// Returns [`SviError::Decimal`] if a step leaves the `Decimal` range and
    /// [`SviError::NegativeTotalVariance`] if the sum is negative by more than
    /// its rounding can explain. A negative sum within that rounding is zero,
    /// so valid parameters evaluate at their minimum without an error, however
    /// small `sigma` is.
    pub fn total_variance(&self, k: Decimal) -> Result<Positive, SviError> {
        let x = d_sub(k, self.m, "svi::total_variance::shift")?;
        let hypotenuse = rounded_hypotenuse(x, self.sigma)?;
        let shape = d_add(
            d_mul(self.rho, x, "svi::total_variance::skew")?,
            hypotenuse,
            "svi::total_variance::shape",
        )?;
        let scaled = d_mul(self.b, shape, "svi::total_variance::scaled")?;
        let variance = d_add(self.a, scaled, "svi::total_variance::sum")?;
        if variance.is_sign_negative() && !variance.is_zero() {
            check_negative_within_rounding(variance, k, self.a, self.b, hypotenuse)?;
            return Ok(Positive::ZERO);
        }
        Ok(Positive::new_decimal(variance)?)
    }

    /// First derivative of total variance in log moneyness at `k`:
    /// `w'(k) = b * (rho + (k - m) / sqrt((k - m)^2 + sigma^2))`, in total
    /// variance per unit of log moneyness.
    ///
    /// The ratio is taken after dividing `k - m` and `sigma` by the larger of
    /// the two, so it keeps its precision however small `sigma` is, where
    /// [`SviParameters::total_variance`] rounds the radicand as it stands. `b` enters
    /// before that division, so a large `b` keeps a ratio far below one.
    ///
    /// # Errors
    ///
    /// Returns [`SviError::Decimal`] if `k - m` or the slope leaves the `Decimal` range.
    pub fn total_variance_slope(&self, k: Decimal) -> Result<Decimal, SviError> {
        const OP: &str = "svi::total_variance_slope";
        let x = d_sub(k, self.m, OP)?;
        let shape = unit_shape(x, self.sigma)?;
        let shift = d_div(
            product_over_powers(self.b, x, shape.scale, 1)?,
            shape.unit_root,
            OP,
        )?;
        Ok(d_add(d_mul(self.b, self.rho, OP)?, shift, OP)?)
    }

    /// Second derivative of total variance in log moneyness at `k`:
    /// `w''(k) = b * sigma^2 / ((k - m)^2 + sigma^2)^(3/2)`, in total variance
    /// per squared unit of log moneyness. It is `b / sigma` at the vertex
    /// `k = m`, so it can leave the `Decimal` range for a tiny `sigma`.
    ///
    /// Far from a tiny `sigma`, `R / sigma` itself can leave the range, with
    /// `R = sqrt((k - m)^2 + sigma^2)`. The result is then zero, not an error, and
    /// that is the correctly rounded value: `sigma` is at least `1e-28`, so `R` is
    /// above 7 and `sigma / R` below `2e-29`, which puts the curvature below
    /// `b * 1e-58`, under the last place for any `b` a `Decimal` can hold.
    ///
    /// # Errors
    ///
    /// Returns [`SviError::Decimal`] if the curvature leaves the `Decimal` range, or
    /// a step on the way does at the edge of it. A value is never rounded away instead:
    /// `b` stays in the computation through the divisions.
    pub fn total_variance_curvature(&self, k: Decimal) -> Result<Decimal, SviError> {
        const OP: &str = "svi::total_variance_curvature";
        let shape = unit_shape(d_sub(k, self.m, OP)?, self.sigma)?;
        // `R / sigma` is at least one, so it keeps 28 significant digits where
        // `sigma / R` would keep few of them far from the vertex. The docs say why
        // its leaving the range means zero.
        let Some(root_over_sigma) = shape
            .scale
            .checked_div(self.sigma)
            .and_then(|ratio| ratio.checked_mul(shape.unit_root))
        else {
            return Ok(Decimal::ZERO);
        };
        // `b / R` is taken through the parts of `R`: rounding `R` itself would cost a
        // tiny `R` its relative precision, and `1 / R` times `b` a large `R` its own.
        let over_root = self
            .b
            .checked_div(shape.scale)
            .and_then(|value| value.checked_div(shape.unit_root));
        match over_root {
            Some(value) => Ok(d_div(
                d_div(value, root_over_sigma, OP)?,
                root_over_sigma,
                OP,
            )?),
            // `R` is below one here, or `b / R` would fit, so `1 / R` keeps its digits.
            // The larger of `b` and `1 / R` then keeps them over `R / sigma`, which is
            // at most `1e28 * R`, since their product does not fit.
            None => product_over_powers(
                self.b,
                d_div(d_div(Decimal::ONE, shape.scale, OP)?, shape.unit_root, OP)?,
                root_over_sigma,
                2,
            ),
        }
    }

    /// An upper bound on the rounding of [`SviParameters::total_variance`] at
    /// `k`, which also bounds that of [`SviParameters::total_variance_slope`].
    ///
    /// # Errors
    ///
    /// Returns [`SviError::Decimal`] if a step leaves the `Decimal` range.
    pub(super) fn total_variance_rounding(&self, k: Decimal) -> Result<Positive, SviError> {
        let x = d_sub(k, self.m, "svi::total_variance_rounding::shift")?;
        let hypotenuse = rounded_hypotenuse(x, self.sigma)?;
        Ok(Positive::new_decimal(rounding_band(
            self.a, self.b, hypotenuse,
        )?)?)
    }
}

/// `sqrt(x^2 + sigma^2)` as [`SviParameters::total_variance`] sums it.
fn rounded_hypotenuse(x: Decimal, sigma: Decimal) -> Result<Decimal, SviError> {
    let root = d_sqrt(
        d_add(
            d_mul(x, x, "svi::total_variance::shift_squared")?,
            d_mul(sigma, sigma, "svi::total_variance::sigma_squared")?,
            "svi::total_variance::radicand",
        )?,
        "svi::total_variance::root",
    )?;
    // The true root is at least `sigma` and at least `|x|`. Rounding the
    // radicand can leave it below both, down to zero for a tiny `sigma`.
    Ok(root.max(x.abs()).max(sigma))
}

/// Checks that a negative total variance is rounding.
#[cold]
#[inline(never)]
fn check_negative_within_rounding(
    variance: Decimal,
    k: Decimal,
    a: Decimal,
    b: Decimal,
    hypotenuse: Decimal,
) -> Result<(), SviError> {
    if variance.abs() > rounding_band(a, b, hypotenuse)? {
        return Err(SviError::NegativeTotalVariance {
            log_moneyness: k,
            total_variance: variance,
        });
    }
    Ok(())
}

/// The rounding that can reach a total variance: the terms summed carry a
/// relative rounding, and the radicand's rounding reaches the root divided by
/// about twice the root, which `b` then scales. At a small root that division
/// says more than the clamp in [`rounded_hypotenuse`] leaves possible, so the root
/// is charged the smaller of the two. `hypotenuse` is positive, since it is at
/// least `sigma`.
fn rounding_band(a: Decimal, b: Decimal, hypotenuse: Decimal) -> Result<Decimal, SviError> {
    const OP: &str = "svi::total_variance::band";
    let magnitude = d_add(a.abs(), d_mul(b, hypotenuse, OP)?, OP)?;
    let relative = d_mul(TOTAL_VARIANCE_ROUNDING_BAND, magnitude, OP)?;
    let root_error =
        d_div(RADICAND_ROUNDING, hypotenuse, OP)?.min(d_mul(ROOT_RELATIVE_ERROR, hypotenuse, OP)?);
    let radicand = d_mul(root_error, b, OP)?;
    let scaled = d_mul(SUM_ROUNDING, b, OP)?;
    Ok(d_add(
        d_add(d_add(relative, radicand, OP)?, scaled, OP)?,
        SUM_ROUNDING,
        OP,
    )?)
}

/// `R = sqrt(x^2 + sigma^2)` kept apart as `scale * unit_root`.
#[derive(Debug, Clone, Copy)]
struct UnitShape {
    scale: Decimal,
    unit_root: Decimal,
}

/// Divides `x` and `sigma` by the larger of the two before the root, so the
/// radicand lies in `[1, 2]` and rounds at 28 significant digits however small
/// or large the inputs are. `sigma` is positive, so the divisor is too.
fn unit_shape(x: Decimal, sigma: Decimal) -> Result<UnitShape, SviError> {
    const OP: &str = "svi::unit_shape";
    let scale = x.abs().max(sigma);
    let unit_x = d_div(x, scale, OP)?;
    let unit_sigma = d_div(sigma, scale, OP)?;
    let unit_root = d_sqrt(
        d_add(
            d_mul(unit_x, unit_x, OP)?,
            d_mul(unit_sigma, unit_sigma, OP)?,
            OP,
        )?,
        OP,
    )?;
    Ok(UnitShape { scale, unit_root })
}

/// `first * second / divisor^divisions`, with no intermediate leaving the range.
///
/// Every step rounds at 28 places, and any factor applied after it multiplies that
/// rounding, so the next step is whichever keeps the largest intermediate: the product,
/// or either factor over the divisor. A product of two small factors keeps few digits,
/// and so does a quotient of a small one, where another order may keep all of them.
///
/// That keeps the digits under one precondition: where the product does not fit, a
/// factor over the divisor is not far below one. Otherwise no step can keep them, and
/// the result loses what the quotient lost, multiplied by the other factor. Both uses
/// meet it. For the slope, `b * sigma` fits in valid parameters, so a `b * x` that does
/// not has `|x| > sigma`, and the divisor is `|x|`, so that `x` over it is exactly one.
/// For the curvature, the divisor `R / sigma` is at most `1e28 * R` and `1 / R` is a
/// factor, so with a product of at least `7.9e28` the larger factor over it stays at
/// `7.9` or more, through each of the two divisions.
///
/// # Errors
///
/// Returns [`SviError::Decimal`] if neither factor can be divided and their product
/// does not fit: the partial result is out of range.
fn product_over_powers(
    first: Decimal,
    second: Decimal,
    divisor: Decimal,
    divisions: u32,
) -> Result<Decimal, SviError> {
    const OP: &str = "svi::product_over_powers";
    let size = |value: Option<Decimal>| value.map(|inner| inner.abs());
    let (mut one, mut other) = (first, second);
    let mut left = divisions;
    while let Some(after) = left.checked_sub(1) {
        let product = size(one.checked_mul(other));
        let one_over = one.checked_div(divisor);
        let other_over = other.checked_div(divisor);
        // `None` orders below any size, so a step that overflows is never chosen.
        let largest_quotient = size(one_over).max(size(other_over));
        if product >= largest_quotient {
            break;
        }
        match (one_over, other_over) {
            (Some(quotient), _) if size(Some(quotient)) == largest_quotient => one = quotient,
            (_, Some(quotient)) => other = quotient,
            _ => break,
        }
        left = after;
    }
    let mut value = d_mul(one, other, OP)?;
    for _ in 0..left {
        value = d_div(value, divisor, OP)?;
    }
    Ok(value)
}

/// `b * sigma * sqrt(1 - rho^2)`: the distance from `a` to the minimum of `w`.
fn minimum_above_level(b: Decimal, rho: Decimal, sigma: Decimal) -> Result<Decimal, SviError> {
    const OP: &str = "svi::parameters::minimum";
    let rho_factor = d_sqrt(d_sub(Decimal::ONE, d_mul(rho, rho, OP)?, OP)?, OP)?;
    Ok(d_mul(d_mul(b, sigma, OP)?, rho_factor, OP)?)
}

/// One expiry of a raw SVI smile: validated parameters, the forward they are
/// centred on and the time to expiry that turns total variance into
/// annualized implied volatility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SviSlice {
    parameters: SviParameters,
    forward: Positive,
    time_to_expiry: Positive,
}

impl SviSlice {
    /// Builds a slice from parameters, the forward of the expiry and the time
    /// to expiry in years.
    ///
    /// # Errors
    ///
    /// Returns [`SviError::ZeroForward`] or [`SviError::ZeroTimeToExpiry`] if
    /// the corresponding input is zero.
    pub fn try_new(
        parameters: SviParameters,
        forward: Positive,
        time_to_expiry: Positive,
    ) -> Result<Self, SviError> {
        if forward.is_zero() {
            return Err(SviError::ZeroForward);
        }
        if time_to_expiry.is_zero() {
            return Err(SviError::ZeroTimeToExpiry);
        }
        Ok(Self {
            parameters,
            forward,
            time_to_expiry,
        })
    }

    /// The raw SVI parameters of the slice.
    #[must_use]
    #[inline]
    pub fn parameters(&self) -> SviParameters {
        self.parameters
    }

    /// The forward the log moneyness is measured against.
    #[must_use]
    #[inline]
    pub fn forward(&self) -> Positive {
        self.forward
    }

    /// The time to expiry, in years.
    #[must_use]
    #[inline]
    pub fn time_to_expiry(&self) -> Positive {
        self.time_to_expiry
    }

    /// The log moneyness `ln(strike / forward)` of a strike.
    ///
    /// # Errors
    ///
    /// Returns [`SviError::ZeroStrike`] for a zero strike and
    /// [`SviError::Decimal`] if the ratio is outside the `Decimal` range.
    pub fn log_moneyness(&self, strike: Positive) -> Result<Decimal, SviError> {
        log_moneyness(strike, self.forward)
    }

    /// Total variance at log moneyness `k`; see
    /// [`SviParameters::total_variance`].
    ///
    /// # Errors
    ///
    /// The errors of [`SviParameters::total_variance`].
    pub fn total_variance(&self, k: Decimal) -> Result<Positive, SviError> {
        self.parameters.total_variance(k)
    }

    /// First derivative of total variance in log moneyness at `k`; see
    /// [`SviParameters::total_variance_slope`].
    ///
    /// # Errors
    ///
    /// The errors of [`SviParameters::total_variance_slope`].
    pub fn total_variance_slope(&self, k: Decimal) -> Result<Decimal, SviError> {
        self.parameters.total_variance_slope(k)
    }

    /// Second derivative of total variance in log moneyness at `k`; see
    /// [`SviParameters::total_variance_curvature`].
    ///
    /// # Errors
    ///
    /// The errors of [`SviParameters::total_variance_curvature`].
    pub fn total_variance_curvature(&self, k: Decimal) -> Result<Decimal, SviError> {
        self.parameters.total_variance_curvature(k)
    }

    /// Annualized implied volatility at log moneyness `k`:
    /// `sqrt(w(k) / T)`, as a fraction per year (`0.2` is 20%).
    ///
    /// # Errors
    ///
    /// The errors of [`SviParameters::total_variance`], and
    /// [`SviError::Decimal`] if the ratio or its root fails.
    pub fn implied_volatility(&self, k: Decimal) -> Result<Positive, SviError> {
        let variance = self.parameters.total_variance(k)?;
        let annualized = d_div(
            variance.to_dec(),
            self.time_to_expiry.to_dec(),
            "svi::implied_volatility::annualized",
        )?;
        let volatility = d_sqrt(annualized, "svi::implied_volatility::root")?;
        Ok(Positive::new_decimal(volatility)?)
    }

    /// Total variance at a strike, with `k = ln(strike / forward)`.
    ///
    /// # Errors
    ///
    /// The errors of [`SviSlice::log_moneyness`] and
    /// [`SviSlice::total_variance`].
    pub fn total_variance_at_strike(&self, strike: Positive) -> Result<Positive, SviError> {
        self.total_variance(self.log_moneyness(strike)?)
    }

    /// Annualized implied volatility at a strike, with
    /// `k = ln(strike / forward)`.
    ///
    /// # Errors
    ///
    /// The errors of [`SviSlice::log_moneyness`] and
    /// [`SviSlice::implied_volatility`].
    pub fn implied_volatility_at_strike(&self, strike: Positive) -> Result<Positive, SviError> {
        self.implied_volatility(self.log_moneyness(strike)?)
    }
}

/// `ln(strike / forward)`, with both checked to be non-zero.
pub(super) fn log_moneyness(strike: Positive, forward: Positive) -> Result<Decimal, SviError> {
    if strike.is_zero() {
        return Err(SviError::ZeroStrike { strike });
    }
    if forward.is_zero() {
        return Err(SviError::ZeroForward);
    }
    let ratio = d_div(
        strike.to_dec(),
        forward.to_dec(),
        "svi::log_moneyness::ratio",
    )?;
    Ok(d_ln(ratio, "svi::log_moneyness::ln")?)
}

/// One quote a calibration fits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SviObservation {
    /// The strike of the quote. Must be positive.
    pub strike: Positive,
    /// The annualized implied volatility of the quote, as a fraction per year.
    /// Must be positive.
    pub implied_volatility: Positive,
    /// The weight of the quote's squared residual in total variance. Only
    /// ratios between weights matter. A zero-weight observation does not enter
    /// the fit and still gets a residual.
    pub weight: Positive,
}

/// Limits of a calibration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SviCalibrationConfig {
    /// The most Nelder-Mead iterations each search start may run. A start that
    /// reaches it is discarded as not converged.
    pub max_iterations: NonZeroU64,
    /// A start has converged when both its search simplex (in normalized
    /// `m` and `ln sigma`) and the spread of its objective (the weighted RMSE
    /// in total variance over its weighted mean) are at or below this value.
    /// A start whose whole simplex fits the observations to the precision of
    /// the arithmetic has also converged. A smaller value takes more iterations
    /// and recovers the parameters more closely. At most `1e-3`: the parameters
    /// are off in proportion to it, so `rho` of an equity skew is already `8e-5`
    /// off at that value.
    pub tolerance: Positive,
    /// The largest relative error of the fit: its weighted RMSE in implied
    /// volatility over the weighted mean observed implied volatility, across
    /// the points with a positive weight (`0.03` is 3%). A fit above it is an
    /// error. This is the only judgement of the fit, and a flat result can only
    /// be returned when the quotes are flat within it; a fit within it is not
    /// otherwise judged good. Tighten or loosen it to say how close is close
    /// enough.
    pub max_relative_rmse: Positive,
}

impl Default for SviCalibrationConfig {
    /// 500 iterations for each start, a tolerance of `1e-9` and a relative RMSE
    /// limit of 3%.
    fn default() -> Self {
        // Both constants are positive, so neither fallback is taken. Were one
        // taken, zero would reject the fit: the failure stays loud.
        Self {
            max_iterations: DEFAULT_MAX_ITERATIONS,
            tolerance: Positive::new_decimal(DEFAULT_TOLERANCE).unwrap_or(Positive::ZERO),
            max_relative_rmse: Positive::new_decimal(DEFAULT_MAX_RELATIVE_RMSE)
                .unwrap_or(Positive::ZERO),
        }
    }
}

/// How a calibrated slice sits against one observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct SviResidual {
    /// The strike of the observation.
    pub strike: Positive,
    /// `ln(strike / forward)`.
    pub log_moneyness: Decimal,
    /// The observed implied volatility.
    pub observed_implied_volatility: Positive,
    /// The implied volatility of the fitted slice at this strike.
    pub fitted_implied_volatility: Positive,
    /// Fitted minus observed implied volatility.
    pub residual: Decimal,
}

/// The result of [`calibrate_svi`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SviFit {
    slice: SviSlice,
    residuals: Vec<SviResidual>,
    rmse: Positive,
    relative_rmse: Positive,
    iterations: u64,
}

impl SviFit {
    /// The calibrated slice. It is not claimed free of arbitrage.
    #[must_use]
    #[inline]
    pub fn slice(&self) -> &SviSlice {
        &self.slice
    }

    /// One residual per observation, in the order of the input.
    #[must_use]
    #[inline]
    pub fn residuals(&self) -> &[SviResidual] {
        &self.residuals
    }

    /// The weighted root mean square error of the fit, in implied volatility:
    /// `sqrt(sum(weight * residual^2) / sum(weight))`.
    #[must_use]
    #[inline]
    pub fn rmse(&self) -> Positive {
        self.rmse
    }

    /// [`SviFit::rmse`] over the weighted mean observed implied volatility of
    /// the points with a positive weight. This is what
    /// [`SviCalibrationConfig::max_relative_rmse`] limits.
    #[must_use]
    #[inline]
    pub fn relative_rmse(&self) -> Positive {
        self.relative_rmse
    }

    /// The Nelder-Mead iterations run, summed over every search start.
    #[must_use]
    #[inline]
    pub fn iterations(&self) -> u64 {
        self.iterations
    }
}

/// Calibrates a raw SVI slice to weighted implied volatility quotes.
///
/// `forward` is the forward of the expiry and `time_to_expiry` is in years.
/// The fit minimizes the weighted squared error in total variance, as the
/// module docs describe, so at equal weights a wing point counts for more in
/// implied volatility than its neighbours near the vertex: `dw = 2 * iv * T *
/// d(iv)`. [`SviFit::rmse`] is in implied volatility, and the limit is on it
/// relative to the weighted mean observed implied volatility. See the module
/// docs for the declared tolerances, for what the limit does and does not say
/// about a fit, and for why the result is not claimed free of arbitrage.
///
/// # Errors
///
/// - [`SviError::ZeroForward`], [`SviError::ZeroTimeToExpiry`]: a zero input.
/// - [`SviError::InvalidConfig`]: a tolerance above `1e-3`.
/// - [`SviError::TooFewObservations`]: fewer than five observations with a
///   positive weight.
/// - [`SviError::InvalidObservation`]: a zero strike or a zero implied
///   volatility.
/// - [`SviError::NonPositiveTotalWeight`]: every weight is zero.
/// - [`SviError::InsufficientDistinctStrikes`]: repeated strikes leave fewer
///   than five distinct strikes among the weighted observations.
/// - [`SviError::NoConvergence`]: no search start converged.
/// - [`SviError::NoFeasibleFit`]: the optimum is not valid once rounded.
/// - [`SviError::RmseAboveLimit`]: the relative RMSE of the best fit is above
///   [`SviCalibrationConfig::max_relative_rmse`].
/// - [`SviError::NonFinite`]: the `f64` kernel produced a non-finite value.
/// - [`SviError::Decimal`], [`SviError::Positive`]: an input or a result is
///   outside the range of its type.
#[instrument(skip(observations, config), fields(observations = observations.len()), level = "debug")]
pub fn calibrate_svi(
    observations: &[SviObservation],
    forward: Positive,
    time_to_expiry: Positive,
    config: &SviCalibrationConfig,
) -> Result<SviFit, SviError> {
    if forward.is_zero() {
        return Err(SviError::ZeroForward);
    }
    if time_to_expiry.is_zero() {
        return Err(SviError::ZeroTimeToExpiry);
    }
    if config.tolerance.to_dec() > MAX_TOLERANCE {
        return Err(SviError::InvalidConfig {
            parameter: "tolerance",
            value: config.tolerance.to_dec(),
            reason: "must not exceed 0.001",
        });
    }
    if observations.len() < MIN_OBSERVATIONS {
        return Err(SviError::TooFewObservations {
            got: observations.len(),
            required: MIN_OBSERVATIONS,
        });
    }

    let years = decimal_to_f64(time_to_expiry.to_dec())?;
    let mut log_moneyness_values = Vec::with_capacity(observations.len());
    let mut raw_points = Vec::with_capacity(observations.len());
    let mut total_weight = 0.0_f64;
    for (index, observation) in observations.iter().enumerate() {
        if observation.strike.is_zero() {
            return Err(SviError::InvalidObservation {
                index,
                reason: "strike must be positive",
            });
        }
        if observation.implied_volatility.is_zero() {
            return Err(SviError::InvalidObservation {
                index,
                reason: "implied volatility must be positive",
            });
        }
        let k = log_moneyness(observation.strike, forward)?;
        log_moneyness_values.push(k);
        let weight = decimal_to_f64(observation.weight.to_dec())?;
        total_weight += weight;
        if weight > 0.0 {
            let volatility = decimal_to_f64(observation.implied_volatility.to_dec())?;
            raw_points.push(kernel::RawPoint {
                k: decimal_to_f64(k)?,
                variance: volatility * volatility * years,
                weight,
            });
        }
    }
    if total_weight <= 0.0 {
        return Err(SviError::NonPositiveTotalWeight);
    }
    if raw_points.len() < MIN_OBSERVATIONS {
        return Err(SviError::TooFewObservations {
            got: raw_points.len(),
            required: MIN_OBSERVATIONS,
        });
    }

    let (solution, parameters) = kernel::calibrate(
        &raw_points,
        config.max_iterations,
        decimal_to_f64(config.tolerance.to_dec())?,
    )
    .and_then(|solution| parameters_from_kernel(&solution).map(|parameters| (solution, parameters)))
    .inspect_err(|error| warn!(%error, "svi calibration failed"))?;
    let slice = SviSlice::try_new(parameters, forward, time_to_expiry)?;

    let mut residuals = Vec::with_capacity(observations.len());
    let mut weighted_squares = 0.0_f64;
    let mut weighted_volatility = 0.0_f64;
    for (observation, k) in observations.iter().zip(log_moneyness_values) {
        let fitted = slice.implied_volatility(k)?;
        let residual = d_sub(
            fitted.to_dec(),
            observation.implied_volatility.to_dec(),
            "svi::calibrate::residual",
        )?;
        let residual_f64 = decimal_to_f64(residual)?;
        let weight = decimal_to_f64(observation.weight.to_dec())?;
        weighted_squares += weight * residual_f64 * residual_f64;
        weighted_volatility += weight * decimal_to_f64(observation.implied_volatility.to_dec())?;
        residuals.push(SviResidual {
            strike: observation.strike,
            log_moneyness: k,
            observed_implied_volatility: observation.implied_volatility,
            fitted_implied_volatility: fitted,
            residual,
        });
    }
    let rmse_f64 = kernel::root(weighted_squares / total_weight);
    let rmse = Positive::new_decimal(kernel_value(rmse_f64, "svi::calibrate::rmse")?)?;
    // A zero weight adds nothing to either sum, so the mean is over the points
    // with a positive weight, each of which has a positive implied volatility.
    let relative_rmse_f64 = rmse_f64 / (weighted_volatility / total_weight);
    let relative_rmse = Positive::new_decimal(kernel_value(
        relative_rmse_f64,
        "svi::calibrate::relative_rmse",
    )?)?;

    if relative_rmse > config.max_relative_rmse {
        warn!(
            relative_rmse = %relative_rmse,
            limit = %config.max_relative_rmse,
            rmse = %rmse,
            "svi fit rejected: relative rmse above the limit"
        );
        return Err(SviError::RmseAboveLimit {
            relative_rmse,
            limit: config.max_relative_rmse,
            rmse,
        });
    }

    debug!(rmse = %rmse, iterations = solution.iterations, "svi calibration finished");
    Ok(SviFit {
        slice,
        residuals,
        rmse,
        relative_rmse,
        iterations: solution.iterations,
    })
}

/// Brings a value of the kernel to a `Decimal`: a non-finite value is
/// [`SviError::NonFinite`], and a finite one outside the `Decimal` range is
/// [`SviError::Decimal`].
fn kernel_value(value: f64, context: &'static str) -> Result<Decimal, SviError> {
    if !value.is_finite() {
        return Err(SviError::non_finite(context, value));
    }
    Ok(f64_to_decimal(value)?)
}

/// Rounds a kernel solution to validated `Decimal` parameters.
///
/// A solution on the boundary `a + b * sigma * sqrt(1 - rho^2) = 0` can land
/// just below it when each value is rounded to `Decimal` on its own. Only
/// that rounding is repaired, by moving `a` to the boundary; anything further
/// away is reported as infeasible. A relative rounding of `rho` reaches
/// `sqrt(1 - rho^2)` multiplied by `rho^2 / (1 - rho^2)`, so the band grows
/// with `1 / (1 - rho^2)` as `|rho|` nears one.
fn parameters_from_kernel(solution: &kernel::Solution) -> Result<SviParameters, SviError> {
    let a = kernel_value(solution.a, "svi::calibrate::a")?;
    let b = kernel_value(solution.b, "svi::calibrate::b")?;
    let rho = kernel_value(solution.rho, "svi::calibrate::rho")?;
    let m = kernel_value(solution.m, "svi::calibrate::m")?;
    let sigma = kernel_value(solution.sigma, "svi::calibrate::sigma")?;

    const OP: &str = "svi::calibrate::repair";
    let infeasible = |error: SviError| SviError::NoFeasibleFit {
        reason: error.to_string(),
    };
    let decimal_infeasible = |error: DecimalError| infeasible(error.into());
    let above_level = minimum_above_level(b, rho, sigma).map_err(infeasible)?;
    let gap = d_add(a, above_level, OP).map_err(decimal_infeasible)?;
    let repaired = if gap.is_sign_negative() && !gap.is_zero() {
        let rho_room = d_sub(
            Decimal::ONE,
            d_mul(rho, rho, OP).map_err(decimal_infeasible)?,
            OP,
        )
        .map_err(decimal_infeasible)?;
        let amplified = d_div(
            d_mul(ROUNDING_REPAIR_BAND, above_level, OP).map_err(decimal_infeasible)?,
            rho_room,
            OP,
        )
        .map_err(decimal_infeasible)?;
        if gap.abs() <= amplified.max(ROUNDING_REPAIR_FLOOR) {
            -above_level
        } else {
            a
        }
    } else {
        a
    };
    SviParameters::try_new(repaired, b, rho, m, sigma).map_err(infeasible)
}

/// The `f64` kernel of the calibration. Nothing in it is public.
///
/// All arithmetic is on `f64` and keeps to normalized values: log moneyness
/// in `[-0.5, 0.5]`, total variance around one, weights summing to one. A
/// non-finite input or objective comes back as [`SviError::NonFinite`].
mod kernel {
    use super::{MIN_OBSERVATIONS, SviError};
    use std::num::NonZeroU64;

    /// The largest `|rho|` the fit may reach: the domain `|rho| < 1` is open,
    /// and the least squares needs a closed set to have an optimum.
    const RHO_LIMIT: f64 = 1.0 - 1e-9;

    /// `1 - corr^2` below which the two shape columns of the linear system
    /// count as collinear. The unconstrained solve is then skipped: the
    /// boundary candidates stay well posed.
    const COLLINEARITY_RATIO: f64 = 1e-12;

    /// Separation in normalized log moneyness below which two strikes count
    /// as one, since the linear system cannot tell them apart.
    const STRIKE_SEPARATION: f64 = 1e-9;

    /// Number of intervals of the `rho` grid on the surface where the minimum
    /// of `w` is zero. That surface is not convex, so the best node brackets
    /// the refinement.
    const RHO_GRID_INTERVALS: u32 = 8;

    /// Golden-section steps of the refinement of `rho`: each keeps 0.618 of
    /// the bracket, from `0.5` wide to about `2e-9`.
    const RHO_REFINEMENT_STEPS: u32 = 40;

    /// `(sqrt(5) - 1) / 2`.
    const GOLDEN_RATIO: f64 = 0.618_033_988_749_894_9;

    /// Normalized `m` starts of the outer search.
    const M_STARTS: [f64; 5] = [-0.75, -0.375, 0.0, 0.375, 0.75];

    /// Normalized `ln sigma` starts of the outer search: `sigma` is about
    /// `0.018`, `0.08`, `0.37` and `1.6` strike ranges.
    const LOG_SIGMA_STARTS: [f64; 4] = [-4.0, -2.5, -1.0, 0.5];

    /// Size of the initial simplex in normalized `m`.
    const M_STEP: f64 = 0.1;

    /// Size of the initial simplex in normalized `ln sigma`.
    const LOG_SIGMA_STEP: f64 = 0.3;

    /// Bounds of normalized `m`: one strike range beyond the observed ones.
    const M_BOUNDS: (f64, f64) = (-1.5, 1.5);

    /// Bounds of normalized `ln sigma`: `sigma` from about `1e-3` to `33`
    /// strike ranges.
    const LOG_SIGMA_BOUNDS: (f64, f64) = (-7.0, 3.5);

    /// Objective root below which the observations are matched to the
    /// precision of the arithmetic and nothing is left to search for. Without
    /// this, a flat input (whose objective ignores `m` and `sigma`) would walk
    /// on rounding noise until the iteration limit.
    const EXACT_FIT_ROOT: f64 = 1e-13;

    /// Nelder-Mead reflection coefficient.
    const REFLECTION: f64 = 1.0;
    /// Nelder-Mead expansion coefficient.
    const EXPANSION: f64 = 2.0;
    /// Nelder-Mead contraction and shrink coefficient.
    const CONTRACTION: f64 = 0.5;

    /// IEEE-754 `sqrt` is correctly rounded, so the kernel's only root gives
    /// the same bits on every platform.
    #[inline]
    #[must_use]
    pub(super) fn root(value: f64) -> f64 {
        value.sqrt() // scan-banned: allow -- f64 `sqrt` returns NaN on a negative input and does not abort; a NaN objective is reported as `NonFinite`
    }

    /// `e^value`, used to map the search variable back to `sigma`.
    #[inline]
    #[must_use]
    fn exponential(value: f64) -> f64 {
        value.exp() // scan-banned: allow -- f64 `exp` of a value within `LOG_SIGMA_BOUNDS`: finite, and it does not abort
    }

    /// One observation as the public function hands it over: log moneyness,
    /// total variance and a positive weight.
    #[derive(Debug, Clone, Copy)]
    pub(super) struct RawPoint {
        pub(super) k: f64,
        pub(super) variance: f64,
        pub(super) weight: f64,
    }

    /// One observation in normalized units.
    #[derive(Debug, Clone, Copy)]
    struct Point {
        k: f64,
        variance: f64,
        weight: f64,
    }

    /// The optimum in the units of the input.
    #[derive(Debug, Clone, Copy)]
    pub(super) struct Solution {
        pub(super) a: f64,
        pub(super) b: f64,
        pub(super) rho: f64,
        pub(super) m: f64,
        pub(super) sigma: f64,
        pub(super) iterations: u64,
    }

    /// The solution of the linear problem at one `(m, sigma)`, with its
    /// objective: the weighted mean of the squared residuals.
    #[derive(Debug, Clone, Copy)]
    struct LinearFit {
        a: f64,
        b: f64,
        rho: f64,
        objective: f64,
    }

    /// Weighted means and centred second moments of the columns at one
    /// `(m, sigma)`.
    #[derive(Debug, Clone, Copy, Default)]
    struct Moments {
        mean_x: f64,
        mean_y: f64,
        mean_v: f64,
        xx: f64,
        yy: f64,
        xy: f64,
        xv: f64,
        yv: f64,
    }

    /// A vertex of the search simplex.
    #[derive(Debug, Clone, Copy)]
    struct Vertex {
        m: f64,
        log_sigma: f64,
        fit: LinearFit,
    }

    /// The end of one search from one start.
    #[derive(Debug, Clone, Copy)]
    struct SearchRun {
        best: Vertex,
        converged: bool,
        iterations: u64,
    }

    /// Calibrates over `points`, each with a positive weight.
    ///
    /// # Errors
    ///
    /// [`SviError::NonFinite`] for a non-finite input or objective,
    /// [`SviError::InsufficientDistinctStrikes`] if too few strikes are
    /// distinct and [`SviError::NoConvergence`] if no start converges.
    pub(super) fn calibrate(
        points: &[RawPoint],
        limit: NonZeroU64,
        tolerance: f64,
    ) -> Result<Solution, SviError> {
        let mut sorted = Vec::with_capacity(points.len());
        let mut total_weight = 0.0;
        let mut weighted_variance = 0.0;
        for point in points {
            for (value, context) in [
                (point.k, "svi::kernel::input_k"),
                (point.variance, "svi::kernel::input_variance"),
                (point.weight, "svi::kernel::input_weight"),
            ] {
                if !value.is_finite() {
                    return Err(SviError::non_finite(context, value));
                }
            }
            sorted.push(point.k);
            total_weight += point.weight;
            weighted_variance += point.weight * point.variance;
        }
        sorted.sort_by(f64::total_cmp);

        let (lowest, highest) = match (sorted.first(), sorted.last()) {
            (Some(lowest), Some(highest)) => (*lowest, *highest),
            _ => {
                return Err(SviError::InsufficientDistinctStrikes {
                    distinct: 0,
                    required: MIN_OBSERVATIONS,
                });
            }
        };
        let span = highest - lowest;
        let distinct = count_distinct(&sorted, span * STRIKE_SEPARATION);
        if distinct < MIN_OBSERVATIONS || span <= 0.0 {
            return Err(SviError::InsufficientDistinctStrikes {
                distinct,
                required: MIN_OBSERVATIONS,
            });
        }

        let centre = lowest + span / 2.0;
        let mean_variance = weighted_variance / total_weight;
        let normalized: Vec<Point> = points
            .iter()
            .map(|point| Point {
                k: (point.k - centre) / span,
                variance: point.variance / mean_variance,
                weight: point.weight / total_weight,
            })
            .collect();

        let mut best: Option<Vertex> = None;
        let mut iterations = 0_u64;
        for start_m in M_STARTS {
            for start_log_sigma in LOG_SIGMA_STARTS {
                let run = search(&normalized, start_m, start_log_sigma, limit, tolerance)?;
                iterations = iterations.checked_add(run.iterations).ok_or_else(|| {
                    SviError::NoConvergence {
                        iterations: u64::MAX,
                        limit: limit.get(),
                    }
                })?;
                let improves = best
                    .as_ref()
                    .is_none_or(|current| run.best.fit.objective < current.fit.objective);
                if run.converged && improves {
                    best = Some(run.best);
                }
            }
        }
        let best = best.ok_or_else(|| SviError::NoConvergence {
            iterations,
            limit: limit.get(),
        })?;

        Ok(Solution {
            a: best.fit.a * mean_variance,
            b: best.fit.b * mean_variance / span,
            rho: best.fit.rho,
            m: centre + best.m * span,
            sigma: exponential(best.log_sigma) * span,
            iterations,
        })
    }

    /// Counts the values of a sorted slice more than `separation` apart from
    /// the previous counted value.
    fn count_distinct(sorted: &[f64], separation: f64) -> usize {
        let mut previous: Option<f64> = None;
        sorted
            .iter()
            .filter(|&&value| {
                let new = previous.is_none_or(|last| value - last > separation);
                if new {
                    previous = Some(value);
                }
                new
            })
            .count()
    }

    /// Nelder-Mead on `(m, ln sigma)` from one start, projected on the bounds.
    fn search(
        points: &[Point],
        start_m: f64,
        start_log_sigma: f64,
        limit: NonZeroU64,
        tolerance: f64,
    ) -> Result<SearchRun, SviError> {
        let mut simplex = [
            vertex(points, start_m, start_log_sigma)?,
            vertex(points, start_m + M_STEP, start_log_sigma)?,
            vertex(points, start_m, start_log_sigma + LOG_SIGMA_STEP)?,
        ];
        sort_simplex(&mut simplex);

        for iteration in 1..=limit.get() {
            let [best, middle, worst] = simplex;
            let centroid_m = (best.m + middle.m) / 2.0;
            let centroid_log_sigma = (best.log_sigma + middle.log_sigma) / 2.0;
            let along = |coefficient: f64| {
                (
                    centroid_m + coefficient * (centroid_m - worst.m),
                    centroid_log_sigma + coefficient * (centroid_log_sigma - worst.log_sigma),
                )
            };
            let (reflected_m, reflected_log_sigma) = along(REFLECTION);
            let reflected = vertex(points, reflected_m, reflected_log_sigma)?;

            simplex = if reflected.fit.objective < best.fit.objective {
                let (expanded_m, expanded_log_sigma) = along(EXPANSION);
                let expanded = vertex(points, expanded_m, expanded_log_sigma)?;
                if expanded.fit.objective < reflected.fit.objective {
                    [best, middle, expanded]
                } else {
                    [best, middle, reflected]
                }
            } else if reflected.fit.objective < middle.fit.objective {
                [best, middle, reflected]
            } else {
                let outside = reflected.fit.objective < worst.fit.objective;
                let coefficient = if outside { CONTRACTION } else { -CONTRACTION };
                let (contracted_m, contracted_log_sigma) = along(REFLECTION * coefficient);
                let contracted = vertex(points, contracted_m, contracted_log_sigma)?;
                // Strict on the inside (Lagarias et al., 1998): on a plateau of
                // equal objectives, as the flat face gives, the simplex then
                // shrinks instead of collapsing on the edge `best`-`middle`.
                let accepted = if outside {
                    contracted.fit.objective <= reflected.fit.objective
                } else {
                    contracted.fit.objective < worst.fit.objective
                };
                if accepted {
                    [best, middle, contracted]
                } else {
                    [
                        best,
                        vertex(
                            points,
                            best.m + CONTRACTION * (middle.m - best.m),
                            best.log_sigma + CONTRACTION * (middle.log_sigma - best.log_sigma),
                        )?,
                        vertex(
                            points,
                            best.m + CONTRACTION * (worst.m - best.m),
                            best.log_sigma + CONTRACTION * (worst.log_sigma - best.log_sigma),
                        )?,
                    ]
                }
            };
            sort_simplex(&mut simplex);

            if has_converged(&simplex, tolerance) {
                let [best, _, _] = simplex;
                return Ok(SearchRun {
                    best,
                    converged: true,
                    iterations: iteration,
                });
            }
        }

        let [best, _, _] = simplex;
        Ok(SearchRun {
            best,
            converged: false,
            iterations: limit.get(),
        })
    }

    /// Orders a simplex by objective. Stable, and `total_cmp` is a total
    /// order, so the result is fully determined by the input.
    #[inline]
    fn sort_simplex(simplex: &mut [Vertex; 3]) {
        simplex.sort_by(|left, right| left.fit.objective.total_cmp(&right.fit.objective));
    }

    /// The stopping rule documented on `SviCalibrationConfig::tolerance`, or
    /// an exact fit across the whole simplex.
    #[inline]
    fn has_converged(simplex: &[Vertex; 3], tolerance: f64) -> bool {
        let [best, middle, worst] = simplex;
        let worst_root = root(worst.fit.objective);
        if worst_root <= EXACT_FIT_ROOT {
            return true;
        }
        let spread = worst_root - root(best.fit.objective);
        let distance = |vertex: &Vertex| {
            (vertex.m - best.m)
                .abs()
                .max((vertex.log_sigma - best.log_sigma).abs())
        };
        spread <= tolerance && distance(middle).max(distance(worst)) <= tolerance
    }

    /// Projects a trial point on the bounds and solves the linear problem.
    ///
    /// # Errors
    ///
    /// [`SviError::NonFinite`] if the solution is not finite.
    fn vertex(points: &[Point], m: f64, log_sigma: f64) -> Result<Vertex, SviError> {
        let m = m.max(M_BOUNDS.0).min(M_BOUNDS.1);
        let log_sigma = log_sigma.max(LOG_SIGMA_BOUNDS.0).min(LOG_SIGMA_BOUNDS.1);
        let fit = solve_linear(points, m, exponential(log_sigma));
        for (value, context) in [
            (fit.objective, "svi::kernel::objective"),
            (fit.a, "svi::kernel::a"),
            (fit.b, "svi::kernel::b"),
            (fit.rho, "svi::kernel::rho"),
        ] {
            if !value.is_finite() {
                return Err(SviError::non_finite(context, value));
            }
        }
        Ok(Vertex { m, log_sigma, fit })
    }

    /// The moments of the columns `x = k - m`, `y = sqrt(x^2 + sigma^2)` and
    /// the variance. Centring removes `a` from the system, which also removes
    /// the near-collinearity of `y` with the constant when `sigma` is large.
    fn moments(points: &[Point], m: f64, sigma: f64) -> Moments {
        let sigma_squared = sigma * sigma;
        let mut moments = Moments::default();
        for point in points {
            let x = point.k - m;
            moments.mean_x += point.weight * x;
            moments.mean_y += point.weight * root(x * x + sigma_squared);
            moments.mean_v += point.weight * point.variance;
        }
        for point in points {
            let x = point.k - m;
            let dx = x - moments.mean_x;
            let dy = root(x * x + sigma_squared) - moments.mean_y;
            let dv = point.variance - moments.mean_v;
            moments.xx += point.weight * dx * dx;
            moments.yy += point.weight * dy * dy;
            moments.xy += point.weight * dx * dy;
            moments.xv += point.weight * dx * dv;
            moments.yv += point.weight * dy * dv;
        }
        moments
    }

    /// The weighted mean of the squared residuals of a candidate, summed from
    /// the residuals themselves so it stays accurate for a fit close to exact.
    fn objective(points: &[Point], m: f64, sigma: f64, a: f64, b: f64, rho: f64) -> f64 {
        let sigma_squared = sigma * sigma;
        points
            .iter()
            .map(|point| {
                let x = point.k - m;
                let residual = a + b * (rho * x + root(x * x + sigma_squared)) - point.variance;
                point.weight * residual * residual
            })
            .sum()
    }

    /// Solves the linear problem at `(m, sigma)` within the parameter bounds.
    ///
    /// The objective is a convex quadratic and the feasible set of
    /// `(a, b * rho, b)` is convex, so an unconstrained solution that is
    /// feasible is the optimum, and otherwise the optimum lies on the
    /// boundary. The boundary is the union of the faces searched below, each
    /// of which yields feasible candidates only.
    fn solve_linear(points: &[Point], m: f64, sigma: f64) -> LinearFit {
        let moments = moments(points, m, sigma);
        if let Some((a, b, rho)) = unconstrained(&moments, sigma) {
            return LinearFit {
                a,
                b,
                rho,
                objective: objective(points, m, sigma, a, b, rho),
            };
        }

        // `b = 0`: the flat face. It is one candidate among the others, kept
        // only if no other candidate fits the observations better.
        let mut best = LinearFit {
            a: moments.mean_v,
            b: 0.0,
            rho: 0.0,
            objective: objective(points, m, sigma, moments.mean_v, 0.0, 0.0),
        };
        let mut keep = |candidate: LinearFit| {
            if candidate.objective < best.objective {
                best = candidate;
            }
        };
        for rho in [RHO_LIMIT, -RHO_LIMIT] {
            if let Some((a, b)) = extreme_skew(&moments, sigma, rho) {
                keep(LinearFit {
                    a,
                    b,
                    rho,
                    objective: objective(points, m, sigma, a, b, rho),
                });
            }
        }
        keep(zero_minimum(points, m, sigma));
        best
    }

    /// The unconstrained least squares, if the system is well posed and its
    /// solution feasible, as `(a, b, rho)`.
    fn unconstrained(moments: &Moments, sigma: f64) -> Option<(f64, f64, f64)> {
        let scale = moments.xx * moments.yy;
        let determinant = scale - moments.xy * moments.xy;
        if determinant.is_nan() || determinant <= COLLINEARITY_RATIO * scale {
            return None;
        }
        let skew = (moments.xv * moments.yy - moments.yv * moments.xy) / determinant;
        let b = (moments.yv * moments.xx - moments.xv * moments.xy) / determinant;
        let a = moments.mean_v - skew * moments.mean_x - b * moments.mean_y;
        let feasible = b >= 0.0
            && skew.abs() <= RHO_LIMIT * b
            && a + sigma * root((b - skew).max(0.0) * (b + skew).max(0.0)) >= 0.0;
        feasible.then(|| {
            let rho = if b > 0.0 { skew / b } else { 0.0 };
            (a, b, rho)
        })
    }

    /// The face `rho = +-RHO_LIMIT`: the least squares in `(a, b)`, if its
    /// solution is feasible, as `(a, b)`.
    fn extreme_skew(moments: &Moments, sigma: f64, rho: f64) -> Option<(f64, f64)> {
        let column_variance = rho * rho * moments.xx + 2.0 * rho * moments.xy + moments.yy;
        if column_variance <= 0.0 {
            return None;
        }
        let b = (rho * moments.xv + moments.yv) / column_variance;
        let a = moments.mean_v - b * (rho * moments.mean_x + moments.mean_y);
        let feasible = b > 0.0 && a + b * sigma * root((1.0 - rho) * (1.0 + rho)) >= 0.0;
        feasible.then_some((a, b))
    }

    /// The face where the minimum of `w` is zero: `a = -b * sigma *
    /// sqrt(1 - rho^2)`. Fixing `rho` leaves a one-column least squares in `b`,
    /// so this searches `rho` over a grid and refines the best node by golden
    /// section.
    fn zero_minimum(points: &[Point], m: f64, sigma: f64) -> LinearFit {
        let step = 2.0 * RHO_LIMIT / f64::from(RHO_GRID_INTERVALS);
        let mut best = zero_minimum_at(points, m, sigma, -RHO_LIMIT);
        for node in 1..=RHO_GRID_INTERVALS {
            let candidate = zero_minimum_at(points, m, sigma, -RHO_LIMIT + f64::from(node) * step);
            if candidate.objective < best.objective {
                best = candidate;
            }
        }

        let mut low = (best.rho - step).max(-RHO_LIMIT);
        let mut high = (best.rho + step).min(RHO_LIMIT);
        let mut left_rho = high - GOLDEN_RATIO * (high - low);
        let mut right_rho = low + GOLDEN_RATIO * (high - low);
        let mut left = zero_minimum_at(points, m, sigma, left_rho);
        let mut right = zero_minimum_at(points, m, sigma, right_rho);
        for _ in 0..RHO_REFINEMENT_STEPS {
            if left.objective < right.objective {
                if left.objective < best.objective {
                    best = left;
                }
                high = right_rho;
                right_rho = left_rho;
                right = left;
                left_rho = high - GOLDEN_RATIO * (high - low);
                left = zero_minimum_at(points, m, sigma, left_rho);
            } else {
                if right.objective < best.objective {
                    best = right;
                }
                low = left_rho;
                left_rho = right_rho;
                left = right;
                right_rho = low + GOLDEN_RATIO * (high - low);
                right = zero_minimum_at(points, m, sigma, right_rho);
            }
        }
        best
    }

    /// The best `b` for one `rho` on the face where the minimum of `w` is zero.
    /// The shape `h = rho * x + y - sigma * sqrt(1 - rho^2)` is non-negative
    /// and so is the variance, so `b` needs no bound of its own beyond guarding
    /// rounding.
    fn zero_minimum_at(points: &[Point], m: f64, sigma: f64, rho: f64) -> LinearFit {
        let sigma_squared = sigma * sigma;
        let anchor = sigma * root((1.0 - rho) * (1.0 + rho));
        let mut shape_shape = 0.0;
        let mut shape_variance = 0.0;
        for point in points {
            let x = point.k - m;
            let shape = rho * x + root(x * x + sigma_squared) - anchor;
            shape_shape += point.weight * shape * shape;
            shape_variance += point.weight * shape * point.variance;
        }
        let b = if shape_shape > 0.0 {
            (shape_variance / shape_shape).max(0.0)
        } else {
            0.0
        };
        let a = -b * anchor;
        LinearFit {
            a,
            b,
            rho,
            objective: objective(points, m, sigma, a, b, rho),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// `(a, b, rho, m, sigma)` as normalized points on a slice.
        fn points_on(a: f64, b: f64, rho: f64, m: f64, sigma: f64) -> Vec<Point> {
            (0..11)
                .map(|node| {
                    let k = -0.5 + f64::from(node) / 10.0;
                    let x = k - m;
                    Point {
                        k,
                        variance: a + b * (rho * x + (x * x + sigma * sigma).sqrt()),
                        weight: 1.0 / 11.0,
                    }
                })
                .collect()
        }

        fn feasible(a: f64, b: f64, rho: f64, sigma: f64) -> bool {
            b >= 0.0
                && rho.abs() < 1.0
                && a + b * sigma * ((1.0 - rho) * (1.0 + rho)).sqrt() >= -1e-12
        }

        /// The objective over a fine grid of feasible `(rho, b, a)`: an
        /// independent check of the active-set solver.
        fn brute_force_minimum(points: &[Point], m: f64, sigma: f64) -> f64 {
            let mut lowest = f64::INFINITY;
            for rho_node in -100..=100 {
                let rho = f64::from(rho_node) / 100.0 * RHO_LIMIT;
                let shape_floor = sigma * ((1.0 - rho) * (1.0 + rho)).sqrt();
                for b_node in 0..=400 {
                    let b = f64::from(b_node) / 100.0;
                    // For fixed (b, rho) the objective is a parabola in `a`:
                    // its minimum, clipped to the bound, is exact.
                    let mean: f64 = points
                        .iter()
                        .map(|point| {
                            let x = point.k - m;
                            point.weight
                                * (point.variance - b * (rho * x + (x * x + sigma * sigma).sqrt()))
                        })
                        .sum();
                    let a = mean.max(-b * shape_floor);
                    lowest = lowest.min(objective(points, m, sigma, a, b, rho));
                }
            }
            lowest
        }

        #[test]
        fn test_solve_linear_inside_the_bounds_recovers_the_exact_solution() {
            let points = points_on(0.2, 0.5, -0.4, 0.1, 0.3);
            let fit = solve_linear(&points, 0.1, 0.3);
            assert!((fit.a - 0.2).abs() < 1e-12, "{fit:?}");
            assert!((fit.b - 0.5).abs() < 1e-12, "{fit:?}");
            assert!((fit.rho + 0.4).abs() < 1e-12, "{fit:?}");
            assert!(fit.objective < 1e-24, "{fit:?}");
        }

        #[test]
        fn test_solve_linear_with_a_boundary_optimum_matches_brute_force() {
            // Each input pulls the unconstrained solution out of the feasible
            // set a different way, at an `(m, sigma)` that is not the truth.
            let inputs = [
                points_on(0.1, 2.0, -0.95, 0.3, 0.05),
                points_on(0.1, 2.0, 0.95, -0.3, 0.05),
                points_on(1.0, 0.0, 0.0, 0.0, 0.1),
            ];
            let decreasing: Vec<Point> = (0..11)
                .map(|node| {
                    let k = -0.5 + f64::from(node) / 10.0;
                    Point {
                        k,
                        variance: 1.0 - k * k,
                        weight: 1.0 / 11.0,
                    }
                })
                .collect();
            for points in inputs.iter().chain([&decreasing]) {
                for (m, sigma) in [(0.0, 0.2), (0.4, 0.02), (-0.4, 1.0), (1.2, 0.1)] {
                    let fit = solve_linear(points, m, sigma);
                    assert!(feasible(fit.a, fit.b, fit.rho, sigma), "{fit:?}");
                    let brute = brute_force_minimum(points, m, sigma);
                    assert!(
                        fit.objective <= brute + 1e-9,
                        "m={m} sigma={sigma} solver {} brute {brute}",
                        fit.objective
                    );
                }
            }
        }

        #[test]
        fn test_solve_linear_never_returns_an_infeasible_point() {
            for (a, b, rho, m_truth, sigma_truth) in [
                (0.5, 1.0, -0.99, 0.2, 0.01),
                (0.0, 3.0, 0.8, -0.4, 0.2),
                (2.0, 0.1, 0.0, 0.0, 1.0),
            ] {
                let points = points_on(a, b, rho, m_truth, sigma_truth);
                for m in [-1.5, -0.5, 0.0, 0.5, 1.5] {
                    for sigma in [0.001, 0.05, 0.5, 5.0, 30.0] {
                        let fit = solve_linear(&points, m, sigma);
                        assert!(feasible(fit.a, fit.b, fit.rho, sigma), "{fit:?}");
                    }
                }
            }
        }

        #[test]
        fn test_zero_minimum_recovers_a_slice_touching_zero() {
            let (b, rho, m, sigma) = (0.8, -0.5, 0.1, 0.2);
            let a = -b * sigma * ((1.0_f64 - rho) * (1.0 + rho)).sqrt();
            let points = points_on(a, b, rho, m, sigma);
            let fit = solve_linear(&points, m, sigma);
            assert!((fit.rho - rho).abs() < 1e-6, "{fit:?}");
            assert!((fit.b - b).abs() < 1e-6, "{fit:?}");
        }

        #[test]
        fn test_calibrate_keeps_the_best_start_rather_than_the_first_converged() {
            // The first start of the grid converges to a poor local minimum on
            // this slice; the calibration must not stop there.
            let (a, b, rho, m, sigma) = (0.0161, 0.2141, -0.275, 0.114, 0.3896);
            let points = points_on(a, b, rho, m, sigma);
            let normalized_mean: f64 = points.iter().map(|p| p.weight * p.variance).sum();
            let normalized: Vec<Point> = points
                .iter()
                .map(|p| Point {
                    variance: p.variance / normalized_mean,
                    ..*p
                })
                .collect();
            let first = search(
                &normalized,
                M_STARTS[0],
                LOG_SIGMA_STARTS[0],
                NonZeroU64::new(500).unwrap(),
                1e-9,
            )
            .unwrap();
            assert!(first.converged);
            assert!(first.best.fit.objective > 1e-8, "{first:?}");

            let raw: Vec<RawPoint> = points
                .iter()
                .map(|p| RawPoint {
                    k: p.k,
                    variance: p.variance,
                    weight: 1.0,
                })
                .collect();
            let solution = calibrate(&raw, NonZeroU64::new(500).unwrap(), 1e-9).unwrap();
            assert!((solution.m - m).abs() < 1e-6, "{solution:?}");
            assert!((solution.sigma - sigma).abs() < 1e-6, "{solution:?}");
            assert!((solution.rho - rho).abs() < 1e-6, "{solution:?}");
        }

        #[test]
        fn test_calibrate_non_finite_input_is_a_typed_error() {
            for (k, variance, weight, context) in [
                (f64::NAN, 1.0, 1.0, "svi::kernel::input_k"),
                (0.0, f64::INFINITY, 1.0, "svi::kernel::input_variance"),
                (0.0, 1.0, f64::NEG_INFINITY, "svi::kernel::input_weight"),
            ] {
                let mut points = vec![
                    RawPoint {
                        k: 0.1,
                        variance: 1.0,
                        weight: 1.0
                    };
                    5
                ];
                points[2] = RawPoint {
                    k,
                    variance,
                    weight,
                };
                let limit = NonZeroU64::MIN;
                assert!(matches!(
                    calibrate(&points, limit, 1e-9),
                    Err(SviError::NonFinite { context: found, .. }) if found == context
                ));
            }
        }

        #[test]
        fn test_vertex_non_finite_objective_is_a_typed_error() {
            let points = [Point {
                k: f64::NAN,
                variance: 1.0,
                weight: 1.0,
            }];
            assert!(matches!(
                vertex(&points, 0.0, 0.0),
                Err(SviError::NonFinite { .. })
            ));
        }

        #[test]
        fn test_calibrate_duplicates_are_counted_once() {
            let points: Vec<RawPoint> = [-0.2, -0.1, 0.0, 0.0, 0.0, 0.1]
                .iter()
                .map(|&k| RawPoint {
                    k,
                    variance: 1.0,
                    weight: 1.0,
                })
                .collect();
            assert!(matches!(
                calibrate(&points, NonZeroU64::MIN, 1e-9),
                Err(SviError::InsufficientDistinctStrikes {
                    distinct: 4,
                    required: 5
                })
            ));
        }

        #[test]
        fn test_calibrate_empty_input_is_a_typed_error() {
            assert!(matches!(
                calibrate(&[], NonZeroU64::MIN, 1e-9),
                Err(SviError::InsufficientDistinctStrikes { distinct: 0, .. })
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn pos(value: Decimal) -> Positive {
        Positive::new_decimal(value).unwrap()
    }

    fn kernel_solution(a: f64, b: f64, rho: f64, sigma: f64) -> kernel::Solution {
        kernel::Solution {
            a,
            b,
            rho,
            m: 0.1,
            sigma,
            iterations: 1,
        }
    }

    fn sample() -> SviParameters {
        SviParameters::try_new(dec!(0.04), dec!(0.2), dec!(-0.4), dec!(0.05), dec!(0.3)).unwrap()
    }

    #[test]
    fn test_svi_parameters_valid_values_are_kept() {
        let parameters = sample();
        assert_eq!(parameters.a(), dec!(0.04));
        assert_eq!(parameters.b(), dec!(0.2));
        assert_eq!(parameters.rho(), dec!(-0.4));
        assert_eq!(parameters.m(), dec!(0.05));
        assert_eq!(parameters.sigma(), dec!(0.3));
    }

    #[test]
    fn test_svi_parameters_each_bound_has_a_typed_error() {
        let invalid = |a, b, rho, sigma| SviParameters::try_new(a, b, rho, dec!(0), sigma);
        assert!(matches!(
            invalid(dec!(0.1), dec!(-0.1), dec!(0), dec!(0.1)),
            Err(SviError::InvalidParameter { parameter: "b", .. })
        ));
        for rho in [dec!(1), dec!(-1), dec!(1.5)] {
            assert!(matches!(
                invalid(dec!(0.1), dec!(0.1), rho, dec!(0.1)),
                Err(SviError::InvalidParameter {
                    parameter: "rho",
                    ..
                })
            ));
        }
        for sigma in [dec!(0), dec!(-0.1)] {
            assert!(matches!(
                invalid(dec!(0.1), dec!(0.1), dec!(0), sigma),
                Err(SviError::InvalidParameter {
                    parameter: "sigma",
                    ..
                })
            ));
        }
        assert!(matches!(
            invalid(dec!(-0.1), dec!(0.1), dec!(0), dec!(0.1)),
            Err(SviError::NegativeMinimumVariance { .. })
        ));
    }

    #[test]
    fn test_svi_parameters_negative_level_above_the_floor_is_valid() {
        // a = -0.0410 is negative and b * sigma * sqrt(1 - rho^2) is 0.0526.
        assert!(
            SviParameters::try_new(
                dec!(-0.0410),
                dec!(0.1331),
                dec!(0.3060),
                dec!(0.3586),
                dec!(0.4153)
            )
            .is_ok()
        );
    }

    #[test]
    fn test_total_variance_derivatives_match_finite_differences() {
        let parameters = sample();
        let h = dec!(0.000001);
        let w = |k: Decimal| parameters.total_variance(k).unwrap().to_dec();
        for k in [dec!(-1), dec!(-0.1), dec!(0.05), dec!(0.3), dec!(2)] {
            let (below, here, above) = (w(k - h), w(k), w(k + h));
            let slope = (above - below) / (h * dec!(2));
            let curvature = (above - dec!(2) * here + below) / (h * h);
            let slope_error = (parameters.total_variance_slope(k).unwrap() - slope).abs();
            let curvature_error =
                (parameters.total_variance_curvature(k).unwrap() - curvature).abs();
            assert!(slope_error < dec!(0.000000001), "k = {k}: {slope_error}");
            assert!(
                curvature_error < dec!(0.000000001),
                "k = {k}: {curvature_error}"
            );
        }
    }

    #[test]
    fn test_total_variance_derivatives_at_the_vertex_are_in_closed_form() {
        // At k = m: w' = b * rho and w'' = b / sigma.
        let parameters = sample();
        let m = parameters.m();
        assert_eq!(parameters.total_variance_slope(m).unwrap(), dec!(-0.08));
        let curvature = parameters.total_variance_curvature(m).unwrap();
        assert!((curvature - dec!(0.2) / dec!(0.3)).abs() < dec!(0.000000000000000000000001));
    }

    #[test]
    fn test_total_variance_derivatives_keep_their_precision_at_a_tiny_sigma() {
        // At k - m = sigma: w' = b * (rho + 1 / sqrt(2)) and
        // w'' = b / (2 * sqrt(2) * sigma), whatever the size of sigma, where the
        // radicand of the root as it stands is below the last place.
        let sigma = dec!(0.00000000000000000001);
        let parameters =
            SviParameters::try_new(dec!(1), dec!(0.1), dec!(0), dec!(0), sigma).unwrap();
        let root_two = dec!(1.4142135623730950488016887242);
        let slope = parameters.total_variance_slope(sigma).unwrap();
        assert!((slope - dec!(0.1) / root_two).abs() < dec!(0.000000000000000000000001));
        let curvature = parameters.total_variance_curvature(sigma).unwrap();
        // Dividing by sigma last: `2 * sqrt(2) * sigma` would round to 8 digits.
        let expected = dec!(0.1) / (dec!(2) * root_two) / sigma;
        assert!(((curvature - expected) / expected).abs() < dec!(0.000000000000000000000001));
    }

    #[test]
    fn test_total_variance_curvature_keeps_its_small_end_far_from_the_vertex() {
        // x = 1e5, sigma = 1e-5: sigma / R squares to 1e-20 and over R is 1e-25.
        let parameters =
            SviParameters::try_new(dec!(1), dec!(1), dec!(0), dec!(0), dec!(0.00001)).unwrap();
        let curvature = parameters.total_variance_curvature(dec!(100000)).unwrap();
        assert_eq!(curvature, dec!(0.0000000000000000000000000001) * dec!(1000));
    }

    #[test]
    fn test_total_variance_curvature_beyond_the_range_of_the_ratio_is_zero() {
        // R / sigma overflows at x = 1e10 and sigma = 1e-28, and the curvature is
        // about 1e-66, which nothing at 28 places can hold. It is not an error.
        let parameters = SviParameters::try_new(
            dec!(1),
            dec!(1),
            dec!(0),
            dec!(0),
            dec!(0.0000000000000000000000000001),
        )
        .unwrap();
        assert_eq!(
            parameters
                .total_variance_curvature(dec!(10000000000))
                .unwrap(),
            dec!(0)
        );
    }

    #[test]
    fn test_total_variance_curvature_at_a_large_b_is_not_rounded_away() {
        // x = 1e-9, sigma = 1e-28, b = 1e20: b / R is 1e29, out of range, and both
        // divisions by R / sigma = 1e19 before b would leave 1e-29, which rounds to
        // zero. The true w'' = b * sigma^2 / R^3 is 9.99999999999999999999999999999999999985e-10,
        // which is 1e-9 to all the places a `Decimal` has for it.
        let parameters = SviParameters::try_new(
            dec!(0),
            dec!(100000000000000000000),
            dec!(0),
            dec!(0),
            dec!(0.0000000000000000000000000001),
        )
        .unwrap();
        let curvature = parameters
            .total_variance_curvature(dec!(0.000000001))
            .unwrap();
        let truth = dec!(0.000000001);
        let relative = ((curvature - truth) / truth).abs();
        assert!(relative < dec!(0.00000000000000000001), "{curvature}");
    }

    #[test]
    fn test_total_variance_curvature_at_a_large_b_keeps_a_large_root_precise() {
        // b / R at R = sigma = 3e5 and b = 1e20: taking 1 / R first would multiply its
        // rounding at 28 places by b. At k = m, w'' = b / sigma.
        let parameters = SviParameters::try_new(
            dec!(0),
            dec!(100000000000000000000),
            dec!(0),
            dec!(0),
            dec!(300000),
        )
        .unwrap();
        let curvature = parameters.total_variance_curvature(dec!(0)).unwrap();
        let error = (curvature - dec!(333333333333333.3333333333333)).abs();
        assert!(error < dec!(0.000000000001), "{error}");
    }

    #[test]
    fn test_total_variance_slope_at_a_large_b_is_not_rounded_away() {
        // x = 3e-28, sigma = 7, b = 1e20: x / sigma is below the last place and rounds
        // to zero, while the true slope is b * x / R = 3e-8 / 7 = 4.2857...e-9.
        let parameters = SviParameters::try_new(
            dec!(1),
            dec!(100000000000000000000),
            dec!(0),
            dec!(0),
            dec!(7),
        )
        .unwrap();
        let slope = parameters
            .total_variance_slope(dec!(0.0000000000000000000000000003))
            .unwrap();
        assert_eq!(slope, dec!(0.0000000042857142857142857143));
    }

    #[test]
    fn test_total_variance_slope_at_a_tiny_root_with_a_moderate_b_keeps_its_digits() {
        // x = 3e-28, sigma = 7e-28, b = 99.37: b / sigma is out of range and b * x keeps
        // 3 digits, while x / sigma keeps all of them. Reference from the same script.
        let parameters = SviParameters::try_new(
            dec!(0),
            dec!(99.37),
            dec!(0.5),
            dec!(0),
            dec!(0.0000000000000000000000000007),
        )
        .unwrap();
        let slope = parameters
            .total_variance_slope(dec!(0.0000000000000000000000000003))
            .unwrap();
        let error = (slope - dec!(88.8287606998118913627331593)).abs();
        assert!(error < dec!(0.0000000000000000000000001), "{error}");
    }

    #[test]
    fn test_total_variance_slope_keeps_a_product_larger_than_its_quotients() {
        // b = x = 1e7, sigma = 3e14: b / sigma keeps 20 digits and b * x all of them, so
        // the product comes first. The true slope is about 1 / 3.
        let parameters = SviParameters::try_new(
            dec!(1),
            dec!(10000000),
            dec!(0),
            dec!(0),
            dec!(300000000000000),
        )
        .unwrap();
        let slope = parameters.total_variance_slope(dec!(10000000)).unwrap();
        let error = (slope - dec!(0.3333333333333331481481481481)).abs();
        assert!(error < dec!(0.000000000000000000000001), "{error}");
    }

    #[test]
    fn test_total_variance_slope_past_b_times_x_keeps_its_value() {
        // b * x = 1e30 is out of range and the slope is about b = 1e20.
        let parameters = SviParameters::try_new(
            dec!(1),
            dec!(100000000000000000000),
            dec!(0),
            dec!(0),
            dec!(0.000001),
        )
        .unwrap();
        let slope = parameters.total_variance_slope(dec!(10000000000)).unwrap();
        assert_eq!(slope, dec!(100000000000000000000));
    }

    #[test]
    fn test_product_over_powers_out_of_range_is_a_typed_error() {
        // Nothing can bring 1e28 * 1e28 back with one division by 1.
        assert!(matches!(
            product_over_powers(
                dec!(10000000000000000000000000000),
                dec!(10000000000000000000000000000),
                dec!(1),
                1
            ),
            Err(SviError::Decimal(_))
        ));
        assert_eq!(
            product_over_powers(dec!(10000000000000000000000000000), dec!(10), dec!(100), 2)
                .unwrap(),
            dec!(10000000000000000000000000)
        );
    }

    #[test]
    fn test_total_variance_derivatives_of_a_flat_slice_are_zero() {
        let flat =
            SviParameters::try_new(dec!(0.04), dec!(0), dec!(0.3), dec!(0.1), dec!(0.2)).unwrap();
        assert_eq!(flat.total_variance_slope(dec!(1)).unwrap(), dec!(0));
        assert_eq!(flat.total_variance_curvature(dec!(1)).unwrap(), dec!(0));
    }

    #[test]
    fn test_total_variance_derivatives_overflow_is_a_typed_error() {
        // w'' = b / sigma leaves the range, and k - m does for an extreme k.
        let sharp = SviParameters::try_new(
            dec!(0),
            dec!(1000),
            dec!(0),
            dec!(0),
            dec!(0.0000000000000000000000000001),
        )
        .unwrap();
        assert!(matches!(
            sharp.total_variance_curvature(dec!(0)),
            Err(SviError::Decimal(_))
        ));
        let shifted =
            SviParameters::try_new(dec!(0.1), dec!(0.1), dec!(0), dec!(-1), dec!(0.1)).unwrap();
        assert!(matches!(
            shifted.total_variance_slope(Decimal::MAX),
            Err(SviError::Decimal(_))
        ));
        assert!(matches!(
            shifted.total_variance_curvature(Decimal::MAX),
            Err(SviError::Decimal(_))
        ));
    }

    #[test]
    fn test_total_variance_rounding_is_the_band_it_declares() {
        // 1e-24 * (|a| + b * R) + 4e-28 * b / R + 4e-28.
        let flat = SviParameters::try_new(dec!(1), dec!(0), dec!(0), dec!(0), dec!(1)).unwrap();
        assert_eq!(
            flat.total_variance_rounding(dec!(0)).unwrap().to_dec(),
            dec!(0.0000000000000000000000010004)
        );
        let at_one =
            SviParameters::try_new(dec!(0.1), dec!(0.1), dec!(0), dec!(0), dec!(1)).unwrap();
        assert_eq!(
            at_one.total_variance_rounding(dec!(0)).unwrap().to_dec(),
            dec!(0.0000000000000000000000002004)
        );
    }

    #[test]
    fn test_total_variance_rounding_is_capped_by_what_the_clamp_leaves_possible() {
        // At x = sigma = 1e-20 the radicand rounds to zero, so the root is clamped to
        // 1e-20 against a true 1.41421356...e-20: off by 0.41421356...e-20. That is
        // what the band must cover and about all it should: 4e-28 / R would be 4e-8.
        let sigma = dec!(0.00000000000000000001);
        let parameters = SviParameters::try_new(dec!(0), dec!(1), dec!(0), dec!(0), sigma).unwrap();
        let rounding = parameters.total_variance_rounding(sigma).unwrap().to_dec();
        assert_eq!(parameters.total_variance(sigma).unwrap().to_dec(), sigma);
        assert!(rounding > dec!(0.0000000000000000000041422), "{rounding}");
        assert!(rounding < dec!(0.0000000000000000000043), "{rounding}");
    }

    #[test]
    fn test_total_variance_rounding_keeps_the_radicand_term_above_the_crossover() {
        // Above R of about 3e-14 the division is the smaller one: 4e-28 / 1e-10.
        let parameters =
            SviParameters::try_new(dec!(0), dec!(1), dec!(0), dec!(0), dec!(0.0000000001)).unwrap();
        assert_eq!(
            parameters
                .total_variance_rounding(dec!(0))
                .unwrap()
                .to_dec(),
            dec!(0.0000000000000000040000000008)
        );
    }

    #[test]
    fn test_total_variance_rounding_covers_the_slope_at_a_large_b_and_tiny_root() {
        // At k = sigma = 1e-28 the slope is 1000 * (0.3 + 1 / sqrt(2)), which rounds:
        // its fraction, past the 1007 taken away, has more than 28 places. The capped
        // radicand term is below the last place here, so only the term in `b` covers
        // that rounding. Reference to 60 digits from the same script.
        let tiny = dec!(0.0000000000000000000000000001);
        let parameters =
            SviParameters::try_new(dec!(0), dec!(1000), dec!(0.3), dec!(0), tiny).unwrap();
        let slope = parameters.total_variance_slope(tiny).unwrap() - dec!(1007);
        let error = (slope - dec!(0.1067811865475244008443621048)).abs();
        let rounding = parameters.total_variance_rounding(tiny).unwrap().to_dec();
        assert!(error > dec!(0.0000000000000000000000000004), "{error}");
        assert!(error <= rounding, "{error} vs {rounding}");
    }

    #[test]
    fn test_total_variance_curvature_with_a_large_b_keeps_its_value_past_b_over_root() {
        // x = 1e-20, sigma = 1e-24, b = 1e9: b / R is 1e29, out of range, while
        // w'' = b * sigma^2 / R^3 is 1e21 * (1 - 1.5e-8 + ...). Same script.
        let parameters = SviParameters::try_new(
            dec!(1),
            dec!(1000000000),
            dec!(0),
            dec!(0),
            dec!(0.000000000000000000000001),
        )
        .unwrap();
        let curvature = parameters
            .total_variance_curvature(dec!(0.00000000000000000001))
            .unwrap();
        let truth = dec!(999999985000000187499.9978125);
        assert!(((curvature - truth) / truth).abs() < dec!(0.00000000000000000000000001));
    }

    #[test]
    fn test_svi_slice_derivatives_are_those_of_its_parameters() {
        let parameters = sample();
        let slice = SviSlice::try_new(parameters, Positive::HUNDRED, Positive::ONE).unwrap();
        assert_eq!(
            slice.total_variance_slope(dec!(0.2)).unwrap(),
            parameters.total_variance_slope(dec!(0.2)).unwrap()
        );
        assert_eq!(
            slice.total_variance_curvature(dec!(0.2)).unwrap(),
            parameters.total_variance_curvature(dec!(0.2)).unwrap()
        );
    }

    #[test]
    fn test_svi_parameters_overflow_is_a_typed_error() {
        assert!(matches!(
            SviParameters::try_new(dec!(0), Decimal::MAX, dec!(0), dec!(0), Decimal::MAX),
            Err(SviError::Decimal(_))
        ));
    }

    #[test]
    fn test_svi_total_variance_matches_the_definition_at_the_vertex() {
        // At k = m the root is sigma: w(m) = a + b * sigma.
        let variance = sample().total_variance(dec!(0.05)).unwrap();
        assert_eq!(variance.to_dec(), dec!(0.04) + dec!(0.2) * dec!(0.3));
    }

    #[test]
    fn test_svi_total_variance_minimum_matches_the_closed_form() {
        // w is minimal at k* = m - rho * sigma / sqrt(1 - rho^2), where it
        // equals a + b * sigma * sqrt(1 - rho^2). Checked against a scan.
        let parameters = sample();
        let root = d_sqrt(dec!(1) - dec!(0.16), "test").unwrap();
        let k_star = dec!(0.05) + dec!(0.4) * dec!(0.3) / root;
        let closed_form = dec!(0.04) + dec!(0.2) * dec!(0.3) * root;
        let at_star = parameters.total_variance(k_star).unwrap().to_dec();
        assert!((at_star - closed_form).abs() < dec!(1e-20));
        for node in -200..=200 {
            let k = Decimal::from(node) * dec!(0.01);
            let value = parameters.total_variance(k).unwrap().to_dec();
            assert!(value >= closed_form - dec!(1e-20), "k={k} w={value}");
        }
    }

    #[test]
    fn test_svi_total_variance_wings_have_the_declared_slopes() {
        // Far from m, w tends to slope b * (1 + rho) on the right and
        // b * (1 - rho) on the left. Finite differences far out.
        let parameters = sample();
        let at = |k: Decimal| parameters.total_variance(k).unwrap().to_dec();
        let right = at(dec!(1001)) - at(dec!(1000));
        let left = at(dec!(-1001)) - at(dec!(-1000));
        assert!((right - dec!(0.12)).abs() < dec!(1e-6), "{right}");
        assert!((left - dec!(0.28)).abs() < dec!(1e-6), "{left}");
    }

    #[test]
    fn test_svi_total_variance_at_a_zero_minimum_stays_non_negative() {
        let rho = dec!(-0.5);
        let sigma = dec!(0.2);
        let b = dec!(0.8);
        let root = d_sqrt(dec!(0.75), "test").unwrap();
        let a = -(b * sigma * root);
        let parameters = SviParameters::try_new(a, b, rho, dec!(0.1), sigma).unwrap();
        let k_star = dec!(0.1) - rho * sigma / root;
        assert!(parameters.total_variance(k_star).unwrap().to_dec() < dec!(1e-24));
    }

    #[test]
    fn test_svi_slice_zero_inputs_are_typed_errors() {
        assert!(matches!(
            SviSlice::try_new(sample(), Positive::ZERO, Positive::ONE),
            Err(SviError::ZeroForward)
        ));
        assert!(matches!(
            SviSlice::try_new(sample(), Positive::ONE, Positive::ZERO),
            Err(SviError::ZeroTimeToExpiry)
        ));
    }

    #[test]
    fn test_svi_slice_implied_volatility_is_root_of_annualized_variance() {
        let slice = SviSlice::try_new(sample(), Positive::HUNDRED, pos(dec!(0.25))).unwrap();
        // w(m) = 0.1 and T = 0.25, so iv = sqrt(0.4).
        let iv = slice.implied_volatility(dec!(0.05)).unwrap();
        assert!((iv.to_dec() - d_sqrt(dec!(0.4), "test").unwrap()).abs() < dec!(1e-24));
    }

    #[test]
    fn test_svi_slice_strike_helpers_use_log_moneyness_against_the_forward() {
        let slice = SviSlice::try_new(sample(), Positive::HUNDRED, Positive::ONE).unwrap();
        let k = slice.log_moneyness(pos(dec!(120))).unwrap();
        assert!((k - d_ln(dec!(1.2), "test").unwrap()).abs() < dec!(1e-24));
        assert_eq!(
            slice.total_variance_at_strike(pos(dec!(120))).unwrap(),
            slice.total_variance(k).unwrap()
        );
        assert_eq!(
            slice.implied_volatility_at_strike(pos(dec!(120))).unwrap(),
            slice.implied_volatility(k).unwrap()
        );
        assert_eq!(slice.log_moneyness(Positive::HUNDRED).unwrap(), dec!(0));
    }

    #[test]
    fn test_svi_slice_zero_strike_is_a_typed_error() {
        let slice = SviSlice::try_new(sample(), Positive::HUNDRED, Positive::ONE).unwrap();
        assert!(matches!(
            slice.implied_volatility_at_strike(Positive::ZERO),
            Err(SviError::ZeroStrike { .. })
        ));
    }

    #[test]
    fn test_svi_total_variance_overflow_is_a_typed_error() {
        assert!(matches!(
            sample().total_variance(Decimal::MAX),
            Err(SviError::Decimal(_))
        ));
    }

    #[test]
    fn test_svi_config_default_has_the_declared_values() {
        let config = SviCalibrationConfig::default();
        assert_eq!(config.max_iterations.get(), 500);
        assert_eq!(config.tolerance.to_dec(), dec!(0.000000001));
        assert_eq!(config.max_relative_rmse.to_dec(), dec!(0.03));
    }

    #[test]
    fn test_parameters_from_kernel_repairs_only_rounding() {
        let solution = |a: f64| kernel_solution(a, 0.8, -0.5, 0.2);
        let floor = -0.8 * 0.2 * 0.75_f64.sqrt();
        // Below the floor by rounding: `a` moves to the boundary.
        let repaired = parameters_from_kernel(&solution(floor * (1.0 + 1e-15))).unwrap();
        let minimum = repaired.a()
            + minimum_above_level(repaired.b(), repaired.rho(), repaired.sigma()).unwrap();
        assert_eq!(minimum, dec!(0));
        // Far below the floor: rejected, not moved.
        assert!(matches!(
            parameters_from_kernel(&solution(floor * 1.5)),
            Err(SviError::NoFeasibleFit { .. })
        ));
    }

    #[test]
    fn test_parameters_from_kernel_non_finite_is_a_typed_error() {
        let solution = kernel_solution(f64::NAN, 0.1, 0.0, 0.1);
        assert!(matches!(
            parameters_from_kernel(&solution),
            Err(SviError::NonFinite {
                context: "svi::calibrate::a",
                ..
            })
        ));
    }

    #[test]
    fn test_parameters_from_kernel_finite_out_of_range_is_a_decimal_error() {
        let solution = kernel_solution(1e40, 0.1, 0.0, 0.1);
        assert!(matches!(
            parameters_from_kernel(&solution),
            Err(SviError::Decimal(_))
        ));
    }

    #[test]
    fn test_parameters_from_kernel_repairs_rounding_near_the_rho_limit() {
        // Rounding `rho` moves sqrt(1 - rho^2) by far more than it moves `rho`:
        // about 1 / (1 - rho^2) times more, so 5000 times at -0.9999.
        for rho in [-0.9999_f64, 0.9999, -0.999_999] {
            let (b, sigma) = (0.3_f64, 0.2_f64);
            let floor = -b * sigma * ((1.0 - rho) * (1.0 + rho)).sqrt();
            let below = floor * (1.0 + 1e-11);
            let repaired = parameters_from_kernel(&kernel_solution(below, b, rho, sigma)).unwrap();
            let minimum = repaired.a()
                + minimum_above_level(repaired.b(), repaired.rho(), repaired.sigma()).unwrap();
            assert_eq!(minimum, dec!(0), "rho {rho}");
            // A real violation stays one.
            assert!(
                matches!(
                    parameters_from_kernel(&kernel_solution(floor * 1.5, b, rho, sigma)),
                    Err(SviError::NoFeasibleFit { .. })
                ),
                "rho {rho}"
            );
        }
    }

    #[test]
    fn test_parameters_from_kernel_rejects_a_violation_just_beyond_the_band() {
        // At rho = -0.9999 the band is 1e-12 / 2e-4 = 5e-9 of `a`.
        let (b, rho, sigma) = (0.3_f64, -0.9999_f64, 0.2_f64);
        let floor = -b * sigma * ((1.0 - rho) * (1.0 + rho)).sqrt();
        assert!(matches!(
            parameters_from_kernel(&kernel_solution(floor * (1.0 + 1e-6), b, rho, sigma)),
            Err(SviError::NoFeasibleFit { .. })
        ));
    }

    #[test]
    fn test_parameters_from_kernel_repairs_rounding_of_a_tiny_slice() {
        // Here the 28 places of a Decimal are what rounds, not the 15 or so
        // digits that survive the f64.
        let (b, rho, sigma) = (1e-20_f64, -0.5_f64, 0.2_f64);
        let floor = -b * sigma * ((1.0 - rho) * (1.0 + rho)).sqrt();
        assert!(parameters_from_kernel(&kernel_solution(floor - 5e-27, b, rho, sigma)).is_ok());
    }

    #[test]
    fn test_svi_total_variance_at_the_vertex_of_a_small_sigma_is_zero_not_an_error() {
        // The minimum of each slice is exactly zero. Rounding the radicand
        // moves the root by about 1e-28 / (2 * root), which grows as `sigma`
        // shrinks; below 1e-14 the radicand rounds to zero altogether. That
        // bounds what is left at the vertex at about `b * 1e-14`.
        for sigma in [
            dec!(0.00123456789012345),
            dec!(0.000000123456789),
            dec!(0.00000000001),
            dec!(0.00000000000001),
            dec!(0.000000000000001),
            dec!(0.00000000000000000001),
            dec!(0.0000000000000000000000000001),
        ] {
            for rho in [dec!(0), dec!(-0.5), dec!(0.9), dec!(-0.9999)] {
                for b in [dec!(1), dec!(0.3), dec!(1000), dec!(0.001), dec!(0.0001)] {
                    assert_vertex_is_near_zero(a_at_zero_minimum(b, rho, sigma), b, rho, sigma);
                }
            }
        }
    }

    /// `a` of a slice whose minimum is exactly zero.
    fn a_at_zero_minimum(b: Decimal, rho: Decimal, sigma: Decimal) -> Decimal {
        -(b * sigma * d_sqrt(dec!(1) - rho * rho, "test").unwrap())
    }

    fn assert_vertex_is_near_zero(a: Decimal, b: Decimal, rho: Decimal, sigma: Decimal) {
        let root = d_sqrt(dec!(1) - rho * rho, "test").unwrap();
        let parameters = SviParameters::try_new(a, b, rho, dec!(0.1), sigma).unwrap();
        let vertex = dec!(0.1) - rho * sigma / root;
        let variance = parameters.total_variance(vertex);
        assert!(
            variance
                .as_ref()
                .is_ok_and(|value| value.to_dec() <= b * dec!(1e-13) + dec!(1e-26)),
            "sigma {sigma} rho {rho} b {b}: {variance:?}"
        );
    }

    #[test]
    fn test_svi_total_variance_at_the_vertex_of_a_small_slice_is_zero_not_an_error() {
        // With `a` and `b` this small, every band relative to them is below one
        // unit in the last place, which is what the rounding of the sum can be.
        for (b, sigma_base) in [(dec!(0.001), dec!(0.011)), (dec!(0.0001), dec!(0.0011))] {
            for sigma_node in 0..40 {
                for rho_node in 0..40 {
                    let sigma = sigma_base + Decimal::from(sigma_node) * sigma_base * dec!(0.00137);
                    let rho = dec!(-0.95) + Decimal::from(rho_node) * dec!(0.0487);
                    assert_vertex_is_near_zero(a_at_zero_minimum(b, rho, sigma), b, rho, sigma);
                }
            }
        }
    }

    #[test]
    fn test_svi_total_variance_negative_beyond_rounding_is_a_typed_error() {
        // Valid parameters do not reach this: it guards the arithmetic. The
        // check itself must still tell rounding from a real negative value.
        assert!(
            check_negative_within_rounding(
                dec!(-1e-26),
                dec!(0),
                dec!(-0.001),
                dec!(1),
                dec!(0.001)
            )
            .is_ok()
        );
        assert!(matches!(
            check_negative_within_rounding(dec!(-0.5), dec!(0.1), dec!(-1), dec!(1), dec!(0.5)),
            Err(SviError::NegativeTotalVariance { log_moneyness, total_variance })
                if log_moneyness == dec!(0.1) && total_variance == dec!(-0.5)
        ));
    }
}
