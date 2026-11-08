/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 8/11/26
******************************************************************************/

//! The delta along the Brownian of the quadratic rough Heston model, for vanillas on the spot
//! and for instruments on the VIX. [`brownian_delta_quadratic_rough_heston`] documents it.

use super::quadratic_rough_heston::{
    Moments, QuadraticRoughHestonGrid, QuadraticRoughHestonParams, Schedule, State,
    invalid_parameter, reserve, run_paths,
};
use super::quadratic_rough_heston_vix::{
    QuadraticRoughHestonVix, QuadraticRoughHestonVixPayoff, checked_vix_squared,
};
use crate::error::QuadraticRoughHestonError;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{d_add, d_div, d_mul, d_sub, decimal_to_f64};
use optionstratlib_core::model::types::OptionStyle;
use rust_decimal::Decimal;
use std::num::NonZeroUsize;

const OPERATION: &str = "quadratic_rough_heston::brownian_delta";

/// An instrument whose delta along the Brownian can be computed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum QuadraticRoughHestonInstrument {
    /// A European call or put on the spot at the horizon of the grid.
    Vanilla {
        /// Call or put.
        style: OptionStyle,
        /// The strike.
        strike: Positive,
    },
    /// A future or a vanilla on the VIX at the horizon of the grid.
    Vix {
        /// The window of the index, in years.
        window: Positive,
        /// What the instrument pays.
        payoff: QuadraticRoughHestonVixPayoff,
    },
}

/// The delta along the Brownian of an instrument, with its Monte Carlo standard error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuadraticRoughHestonBrownianDelta {
    delta: Decimal,
    standard_error: Decimal,
    paths: usize,
}

impl QuadraticRoughHestonBrownianDelta {
    /// The change of the price per unit of spot, along the Brownian.
    #[must_use]
    #[inline]
    pub fn delta(&self) -> Decimal {
        self.delta
    }

    /// The standard error of [`delta`](Self::delta), for the bump it was computed with. It
    /// does not include the bias of the bump or of the time step.
    #[must_use]
    #[inline]
    pub fn standard_error(&self) -> Decimal {
        self.standard_error
    }

    /// The number of simulated paths.
    #[must_use]
    #[inline]
    pub fn paths(&self) -> usize {
        self.paths
    }
}

/// An instrument with its inputs in the `f64` of the kernel.
enum Resolved {
    Vanilla {
        style: OptionStyle,
        strike: f64,
    },
    Vix {
        index: QuadraticRoughHestonVix,
        option: Option<(OptionStyle, f64)>,
    },
}

impl Resolved {
    fn new(
        params: &QuadraticRoughHestonParams,
        instrument: QuadraticRoughHestonInstrument,
    ) -> Result<Self, QuadraticRoughHestonError> {
        Ok(match instrument {
            QuadraticRoughHestonInstrument::Vanilla { style, strike } => Self::Vanilla {
                style,
                strike: decimal_to_f64(strike.to_dec())?,
            },
            QuadraticRoughHestonInstrument::Vix { window, payoff } => Self::Vix {
                index: QuadraticRoughHestonVix::try_new(params, window)?,
                option: match payoff {
                    QuadraticRoughHestonVixPayoff::Future => None,
                    QuadraticRoughHestonVixPayoff::Vanilla { style, strike } => {
                        Some((style, decimal_to_f64(strike.to_dec())?))
                    }
                },
            },
        })
    }

    /// What a path observes at the horizon: the spot, or the squared index.
    #[inline]
    fn observe(&self, state: &State) -> f64 {
        match self {
            Self::Vanilla { .. } => state.spot,
            Self::Vix { index, .. } => index.raw_squared(&state.factors),
        }
    }

    /// The payoff of a path from what it observed.
    #[inline]
    fn payoff(&self, observed: f64) -> Result<f64, QuadraticRoughHestonError> {
        Ok(match self {
            Self::Vanilla { style, strike } => intrinsic(*style, observed, *strike),
            Self::Vix { option, .. } => {
                let index = checked_vix_squared(observed)?.sqrt(); // scan-banned: allow -- f64 `sqrt` of a positive finite value: finite, and it does not abort
                match option {
                    None => index,
                    Some((style, strike)) => intrinsic(*style, index, *strike),
                }
            }
        })
    }
}

#[inline]
fn intrinsic(style: OptionStyle, underlying: f64, strike: f64) -> f64 {
    match style {
        OptionStyle::Call => (underlying - strike).max(0.0),
        OptionStyle::Put => (strike - underlying).max(0.0),
    }
}

/// `params` moved by `signed_bump` along the Brownian: every factor by the bump, and the spot
/// by the fraction `signed_bump / eta` of itself.
fn bumped(
    params: &QuadraticRoughHestonParams,
    signed_bump: Decimal,
) -> Result<QuadraticRoughHestonParams, QuadraticRoughHestonError> {
    let spot = params.spot().to_dec();
    let relative = d_div(signed_bump, params.eta(), OPERATION)?;
    let moved = d_add(spot, d_mul(spot, relative, OPERATION)?, OPERATION)?;
    let moved_spot = Positive::new_decimal(moved)
        .map_err(|_| invalid_parameter("bump", signed_bump, "moves the spot below zero"))?;
    let mut factors = reserve(params.initial_factors().len(), "bumped factors")?;
    for factor in params.initial_factors() {
        factors.push(d_add(*factor, signed_bump, OPERATION)?);
    }
    QuadraticRoughHestonParams::try_new(
        params.a(),
        params.b(),
        params.c(),
        params.lambda(),
        params.eta(),
        params.kernel().clone(),
        factors,
        moved_spot,
    )
}

/// The payoff of `instrument` on each path started from `params`.
fn payoffs(
    params: &QuadraticRoughHestonParams,
    schedule: &Schedule,
    instrument: &Resolved,
    paths: NonZeroUsize,
    seed: u64,
) -> Result<Vec<f64>, QuadraticRoughHestonError> {
    run_paths(
        params,
        schedule,
        paths,
        seed,
        |_, state: &State, observed: &mut f64| {
            *observed = instrument.observe(state);
        },
    )?
    .into_iter()
    .map(|run| instrument.payoff(run.observed))
    .collect()
}

/// The delta along the Brownian of a vanilla or a VIX instrument, by a central difference on
/// common random numbers.
///
/// # Definition
///
/// One Brownian `W` drives the spot, with `S sqrt(V) dW`, and each factor, with
/// `eta sqrt(V) dW`. So the part of `dP` in `dW` is `sqrt(V) (S dP/dS + eta sum_i dP/dZ^i)`,
/// and the units of spot that carry the same noise are
///
/// ```text
/// Delta = dP/dS + (eta / S) sum_i dP/dZ^i
/// ```
///
/// `sum_i dP/dZ^i` is the derivative under a parallel shift of every factor, the direction of
/// the diffusion, and not under a bump of `Z = sum c_i Z^i`. The price of a VIX instrument
/// does not depend on the spot, so its delta is the second term alone.
///
/// # Method
///
/// With `h = bump`, the model is started from `Z^i_0 +/- h` and `S_0 (1 +/- h / eta)`, which
/// moves it along that direction, and
///
/// ```text
/// Delta = (eta / S_0) (P_up - P_down) / (2 h)
/// ```
///
/// Both runs use the paths of `seed`, and the standard error is the one of the differences
/// path by path, so it accounts for their correlation.
///
/// There is no default `bump`: the bias falls as `h^2` for a smooth price, while the standard
/// error of a kinked payoff grows as `h` shrinks, and it is the caller who sets that balance.
/// See [`QuadraticRoughHestonParams`] for the scheme and the caution on one run: compare
/// several seeds, step counts and bumps before trusting a value.
///
/// # Errors
///
/// - [`QuadraticRoughHestonError::InvalidParameter`] when `bump` is zero or not below `eta`,
///   for a single path, which has no standard error, or for a zero `window`.
/// - [`QuadraticRoughHestonError::VixMomentsNotFinite`] when the conditional moments grow
///   beyond an `f64` within the window.
/// - [`QuadraticRoughHestonError::NonFinite`] when a path stops being finite, with its step.
/// - [`QuadraticRoughHestonError::VixNotPositive`] when the index of a path has no positive
///   finite value.
/// - [`QuadraticRoughHestonError::BufferTooLarge`] when `paths` cannot be allocated.
/// - [`QuadraticRoughHestonError::Decimal`] when a bumped input or the result has no value.
///
/// # Examples
///
/// ```rust
/// use optionstratlib_core::model::types::OptionStyle;
/// use optionstratlib_core::pos_or_panic;
/// use optionstratlib_simulation::simulation::{
///     QuadraticRoughHestonGrid, QuadraticRoughHestonInstrument, QuadraticRoughHestonKernel,
///     QuadraticRoughHestonParams, brownian_delta_quadratic_rough_heston,
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
/// let call = QuadraticRoughHestonInstrument::Vanilla {
///     style: OptionStyle::Call,
///     strike: pos_or_panic!(100.0),
/// };
/// let paths = NonZeroUsize::try_from(500_usize)?;
/// let estimate = brownian_delta_quadratic_rough_heston(
///     &params,
///     &grid,
///     call,
///     pos_or_panic!(0.01),
///     paths,
///     42,
/// )?;
/// assert!(estimate.delta() > dec!(0));
/// # Ok(())
/// # }
/// ```
#[tracing::instrument(level = "debug", skip(params))]
pub fn brownian_delta_quadratic_rough_heston(
    params: &QuadraticRoughHestonParams,
    grid: &QuadraticRoughHestonGrid,
    instrument: QuadraticRoughHestonInstrument,
    bump: Positive,
    paths: NonZeroUsize,
    seed: u64,
) -> Result<QuadraticRoughHestonBrownianDelta, QuadraticRoughHestonError> {
    let shift = bump.to_dec();
    if shift.is_zero() {
        return Err(invalid_parameter("bump", shift, "must be positive"));
    }
    if shift >= params.eta() {
        return Err(invalid_parameter("bump", shift, "must be below eta"));
    }

    // The quadratic form of the index depends on neither `Z_0` nor `S_0`, so one serves both.
    let resolved = Resolved::new(params, instrument)?;
    let schedule = Schedule::from_grid(grid)?;
    let up = bumped(params, shift)?;
    let down = bumped(params, d_sub(Decimal::ZERO, shift, OPERATION)?)?;
    let payoffs_up = payoffs(&up, &schedule, &resolved, paths, seed)?;
    let payoffs_down = payoffs(&down, &schedule, &resolved, paths, seed)?;

    let eta = decimal_to_f64(params.eta())?;
    let spot = decimal_to_f64(params.spot().to_dec())?;
    let scale = eta / (spot * 2.0 * decimal_to_f64(shift)?);
    let mut differences = Moments::default();
    for (payoff_up, payoff_down) in payoffs_up.iter().zip(&payoffs_down) {
        differences.add((payoff_up - payoff_down) * scale);
    }
    let (delta, standard_error) = differences.estimate()?;
    Ok(QuadraticRoughHestonBrownianDelta {
        delta,
        standard_error,
        paths: payoffs_up.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::super::quadratic_rough_heston::QuadraticRoughHestonKernel;
    use super::*;
    use optionstratlib_core::pos_or_panic;
    use rust_decimal_macros::dec;

    fn params() -> QuadraticRoughHestonParams {
        let kernel = match QuadraticRoughHestonKernel::try_new(dec!(0.6), 3, dec!(5)) {
            Ok(kernel) => kernel,
            Err(error) => panic!("kernel: {error}"),
        };
        match QuadraticRoughHestonParams::try_new(
            dec!(0.5),
            dec!(0.1),
            dec!(0.04),
            dec!(1),
            dec!(0.5),
            kernel,
            vec![dec!(0.01), dec!(0.02), dec!(0.03)],
            pos_or_panic!(100.0),
        ) {
            Ok(params) => params,
            Err(error) => panic!("params: {error}"),
        }
    }

    #[test]
    fn test_bumped_moves_every_factor_and_the_spot_along_the_brownian() {
        let moved = match bumped(&params(), dec!(-0.05)) {
            Ok(moved) => moved,
            Err(error) => panic!("bumped: {error}"),
        };
        assert_eq!(
            moved.initial_factors(),
            &[dec!(-0.04), dec!(-0.03), dec!(-0.02)]
        );
        assert_eq!(moved.spot().to_dec(), dec!(90));
    }

    #[test]
    fn test_intrinsic_pays_the_positive_part_for_each_style() {
        assert_eq!(intrinsic(OptionStyle::Call, 12.0, 10.0), 2.0);
        assert_eq!(intrinsic(OptionStyle::Call, 8.0, 10.0), 0.0);
        assert_eq!(intrinsic(OptionStyle::Put, 8.0, 10.0), 2.0);
        assert_eq!(intrinsic(OptionStyle::Put, 12.0, 10.0), 0.0);
    }
}
