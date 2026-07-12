/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/7/26
******************************************************************************/
//! Generic walk driver shared by every step generator.
//!
//! The three public generators (`generator_positive` here,
//! [`crate::chains::generator_optionchain`] and
//! [`crate::series::generator_optionseries`]) used to duplicate the same
//! algorithm: dispatch on [`WalkType`], extract the walk volatility, skip the
//! first walker value, advance the x-step until expiration, build the next
//! y-value and cap the result at `WalkParams::size`. [`walk_steps`] is the
//! single implementation of that algorithm; each generator only supplies the
//! closure that builds its `Y` value from the new price.

use crate::error::{ChainError, SimulationError};
use crate::simulation::steps::{Step, Xstep, Ystep};
use crate::simulation::{WalkParams, WalkType};
use crate::utils::TimeFrame;
use crate::utils::others::calculate_log_returns;
use crate::volatility::{adjust_volatility, constant_volatility};
use positive::Positive;
use rust_decimal::{Decimal, MathematicalOps};
use std::convert::TryInto;
use std::fmt::Display;
use tracing::debug;

/// Annualized volatility driving the walk.
///
/// For the nine synthetic walk types this is the variant's `volatility`
/// parameter (see [`WalkType::volatility`]). For [`WalkType::Historical`] it
/// is the constant volatility estimated from the log returns of the full
/// price history, annualized from the history's timeframe.
///
/// # Errors
///
/// Propagates errors from the historical helpers (`calculate_log_returns`,
/// `constant_volatility`, `adjust_volatility`) as [`SimulationError`], so
/// the driver stays free of chain-layer error types.
fn walk_volatility<Y>(
    walk_params: &WalkParams<Positive, Y>,
) -> Result<Option<Positive>, SimulationError>
where
    Y: TryInto<Positive> + Display + Clone,
{
    match &walk_params.walk_type {
        WalkType::Historical {
            timeframe, prices, ..
        } => {
            let log_returns: Vec<Decimal> = calculate_log_returns(prices)?
                .iter()
                .map(|p| p.to_dec())
                .collect();
            let constant_volatility = constant_volatility(&log_returns)?;
            let implied_volatility =
                adjust_volatility(constant_volatility, *timeframe, TimeFrame::Year)?;
            Ok(Some(implied_volatility))
        }
        walk_type => Ok(walk_type.volatility()),
    }
}

/// Per-step expanding-window volatility estimates for a walked price path,
/// free of look-ahead bias: the estimate at index `i` uses only the log
/// returns of `prices[..=i]`.
///
/// Uses the same sample-variance convention as
/// [`crate::volatility::constant_volatility`], computed incrementally with
/// checked arithmetic, and annualizes each estimate from `timeframe`.
/// Indices with fewer than two returns are backfilled with the first
/// computable estimate. Returns `Ok(None)` when no index has enough data
/// (fewer than three prices).
///
/// # Errors
///
/// Propagates errors from `calculate_log_returns` / `adjust_volatility` and
/// surfaces arithmetic overflow as [`SimulationError`], keeping the driver
/// free of chain-layer error types.
fn expanding_window_vols(
    prices: &[Positive],
    timeframe: TimeFrame,
) -> Result<Option<Vec<Positive>>, SimulationError> {
    let n = prices.len();
    if n < 3 {
        // Fewer than two returns anywhere: no sample variance is computable.
        return Ok(None);
    }
    let log_returns: Vec<Decimal> = calculate_log_returns(prices)?
        .iter()
        .map(|p| p.to_dec())
        .collect();

    let overflow =
        |what: &str| SimulationError::walk_error(&format!("expanding-window {what} overflowed"));

    let mut sum = Decimal::ZERO;
    let mut sq_sum = Decimal::ZERO;
    let mut raw: Vec<Option<Positive>> = Vec::with_capacity(n);
    raw.push(None); // index 0 has no returns yet

    for (i, r) in log_returns.iter().copied().enumerate() {
        sum = sum
            .checked_add(r)
            .ok_or_else(|| overflow("running sum of log returns"))?;
        let r_sq = r
            .checked_mul(r)
            .ok_or_else(|| overflow("squared log return"))?;
        sq_sum = sq_sum
            .checked_add(r_sq)
            .ok_or_else(|| overflow("running sum of squared log returns"))?;

        // `r` is the return between prices[i] and prices[i + 1]; after
        // consuming it, `count` returns are available at price index i + 1.
        let count = i + 1;
        if count < 2 {
            raw.push(None);
            continue;
        }
        let count_dec = Decimal::from(count as u64);
        let denom = count_dec - Decimal::ONE;
        // Sample variance via prefix sums: (Σr² − (Σr)²/n) / (n − 1),
        // algebraically identical to the two-pass form in
        // `constant_volatility`.
        let sum_sq = sum
            .checked_mul(sum)
            .ok_or_else(|| overflow("squared running sum"))?;
        let mean_sq_total = sum_sq
            .checked_div(count_dec)
            .ok_or_else(|| overflow("mean projection"))?;
        let variance = sq_sum
            .checked_sub(mean_sq_total)
            .ok_or_else(|| overflow("variance numerator"))?
            .checked_div(denom)
            .ok_or_else(|| overflow("variance"))?
            .max(Decimal::ZERO);
        let std_dev = variance.sqrt().ok_or_else(|| overflow("volatility sqrt"))?;
        let std_dev = Positive::new_decimal(std_dev).unwrap_or(Positive::ZERO);
        let annualized = adjust_volatility(std_dev, timeframe, TimeFrame::Year)?;
        raw.push(Some(annualized));
    }

    let first_computable = raw.iter().flatten().next().copied();
    match first_computable {
        None => Ok(None),
        Some(fill) => Ok(Some(
            raw.into_iter().map(|vol| vol.unwrap_or(fill)).collect(),
        )),
    }
}

/// Runs a walk and materializes it as a vector of [`Step`]s, delegating the
/// construction of each y-value to `next_y`.
///
/// This is the shared driver behind [`generator_positive`],
/// [`crate::chains::generator_optionchain`] and
/// [`crate::series::generator_optionseries`].
///
/// # Contract
///
/// * The returned vector always starts with `walk_params.init_step`.
/// * If the walker yields no values beyond the initial one (e.g. a size-1
///   walk), only the initial step is returned.
/// * The walk is truncated when the x-step reaches expiration
///   (`SimulationError::ExpirationReached`) or when `next_y` returns
///   `Ok(None)`; any other step-advance error is propagated.
/// * The result is truncated to at most `walk_params.size` steps.
///
/// # Parameters
///
/// * `walk_params` - The walk configuration; `walk_params.walk_type` selects
///   the stochastic process via
///   [`crate::simulation::WalkTypeAble::generate_with_vol`].
/// * `next_y` - Builds the y-value for one step from `(new_price,
///   step_volatility, advanced_x_step)`. Returning `Ok(None)` ends the walk
///   gracefully at the previous step.
///
/// # Volatility
///
/// The volatility passed to `next_y` is per-step wherever the model provides
/// one: the simulated vol path for stochastic-volatility walk types
/// (`Garch`, `Heston`, `Custom`, `Telegraph`), an expanding-window estimate
/// free of look-ahead bias for `Historical` walks, and the walk type's
/// constant `volatility` parameter otherwise.
///
/// # Returns
///
/// * `Ok(Vec<Step<Positive, Y>>)` - The simulated walk.
/// * `Err(E)` - If the walker, the volatility estimation, the x-step
///   advance, or `next_y` fail.
///
/// # Errors
///
/// The driver is generic over its error type: every internal failure is a
/// [`SimulationError`] lifted into `E` via `From`, so the simulation layer
/// carries no chain-specific error types — the chain and series generators
/// simply instantiate `E = ChainError`. Walker errors include
/// `SimulationError::InsufficientHistoricalData` when a `Historical` walk
/// has fewer prices than `walk_params.size`; `next_y` errors are returned
/// verbatim.
pub fn walk_steps<Y, E, F>(
    walk_params: &WalkParams<Positive, Y>,
    mut next_y: F,
) -> Result<Vec<Step<Positive, Y>>, E>
where
    Y: TryInto<Positive> + Display + Clone,
    E: From<SimulationError>,
    F: FnMut(&Positive, Option<Positive>, &Xstep<Positive>) -> Result<Option<Y>, E>,
{
    debug!("{}", walk_params);
    let path = walk_params
        .walker
        .generate_with_vol(walk_params)
        .map_err(E::from)?;
    let y_steps = path.prices;
    if y_steps.len() <= 1 {
        // Preserve the init-step invariant when the walker produces no
        // values beyond the initial one; downstream consumers expect at
        // least the initial step to be present.
        return Ok(vec![walk_params.init_step.clone()]);
    }

    // Per-step volatilities: from the walker when the model simulates a vol
    // path (Garch/Heston/Custom/Telegraph); for Historical walks, an
    // expanding-window estimate over the walked prices that uses no future
    // data. Otherwise a single constant from the walk type.
    let step_vols: Option<Vec<Positive>> = match path.vols {
        Some(vols) => Some(vols),
        None => match &walk_params.walk_type {
            WalkType::Historical { timeframe, .. } => {
                expanding_window_vols(&y_steps, *timeframe).map_err(E::from)?
            }
            _ => None,
        },
    };
    let constant_vol = if step_vols.is_some() {
        None
    } else {
        walk_volatility(walk_params).map_err(E::from)?
    };

    let mut steps: Vec<Step<Positive, Y>> = vec![walk_params.init_step.clone()];
    let mut previous_x_step = walk_params.init_step.x;
    let mut y_index = *walk_params.ystep_ref().index();

    // The first walker value duplicates the init step, so skip it.
    for (i, y_step) in y_steps.iter().enumerate().skip(1) {
        previous_x_step = match previous_x_step.next() {
            Ok(x_step) => x_step,
            // Reaching expiration is the normal end of a walk: truncate.
            Err(SimulationError::ExpirationReached) => break,
            // Any other step-advance failure is a real error: propagate.
            Err(e) => return Err(E::from(e)),
        };
        let volatility = step_vols
            .as_ref()
            .and_then(|vols| vols.get(i).copied())
            .or(constant_vol);
        let Some(y_value) = next_y(y_step, volatility, &previous_x_step)? else {
            break;
        };
        y_index += 1;
        steps.push(Step {
            x: previous_x_step,
            y: Ystep::new(y_index, y_value),
        });
    }

    if steps.len() > walk_params.size {
        debug!(
            "walk produced {} steps, truncating to configured size {}",
            steps.len(),
            walk_params.size
        );
        steps.truncate(walk_params.size);
    }
    Ok(steps)
}

/// Parallel variant of [`walk_steps`]: identical contract and output, with
/// the per-step `next_y` calls executed on the rayon thread pool.
///
/// The walk itself (walker output, per-step volatilities, x-step sequence)
/// is computed serially and is identical to [`walk_steps`]; only the y-value
/// construction — the dominant cost when `next_y` rebuilds an option chain
/// or series — is fanned out. Output is deterministic and equal to the
/// serial driver's for the same inputs because every per-step input is
/// precomputed and order is preserved on collection.
///
/// Semantics notes versus the serial driver:
///
/// * `next_y` must be `Fn + Sync` (stateless or internally synchronized)
///   instead of `FnMut`.
/// * `Ok(None)` still ends the walk at the previous step; steps past the
///   first `None` are computed speculatively and their results — values AND
///   errors — are discarded, exactly as the serial driver never evaluates
///   them.
/// * On failure, the first error in step order that precedes any stop
///   signal is returned.
///
/// # Errors
///
/// Same as [`walk_steps`].
pub fn walk_steps_par<Y, E, F>(
    walk_params: &WalkParams<Positive, Y>,
    next_y: F,
) -> Result<Vec<Step<Positive, Y>>, E>
where
    Y: TryInto<Positive> + Display + Clone + Send,
    E: From<SimulationError> + Send,
    F: Fn(&Positive, Option<Positive>, &Xstep<Positive>) -> Result<Option<Y>, E> + Sync,
{
    use rayon::prelude::*;

    debug!("{}", walk_params);
    let path = walk_params
        .walker
        .generate_with_vol(walk_params)
        .map_err(E::from)?;
    let y_steps = path.prices;
    if y_steps.len() <= 1 {
        // Preserve the init-step invariant when the walker produces no
        // values beyond the initial one; downstream consumers expect at
        // least the initial step to be present.
        return Ok(vec![walk_params.init_step.clone()]);
    }

    // Per-step volatilities: same policy as the serial driver.
    let step_vols: Option<Vec<Positive>> = match path.vols {
        Some(vols) => Some(vols),
        None => match &walk_params.walk_type {
            WalkType::Historical { timeframe, .. } => {
                expanding_window_vols(&y_steps, *timeframe).map_err(E::from)?
            }
            _ => None,
        },
    };
    let constant_vol = if step_vols.is_some() {
        None
    } else {
        walk_volatility(walk_params).map_err(E::from)?
    };

    // Precompute the x-step sequence serially, honoring the
    // expiration-truncation contract; Xstep::next is deterministic.
    let mut x_steps: Vec<Xstep<Positive>> = Vec::with_capacity(y_steps.len().saturating_sub(1));
    let mut current_x = walk_params.init_step.x;
    for _ in 1..y_steps.len() {
        current_x = match current_x.next() {
            Ok(x_step) => x_step,
            // Reaching expiration is the normal end of a walk: truncate.
            Err(SimulationError::ExpirationReached) => break,
            // Any other step-advance failure is a real error: propagate.
            Err(e) => return Err(E::from(e)),
        };
        x_steps.push(current_x);
    }

    // Every step's inputs are known up front: build the y-values in
    // parallel, preserving order. Collect WITHOUT short-circuiting so a
    // speculative error past a stop signal cannot mask the stop: the
    // ordered assembly below decides what the serial driver would have
    // observed.
    let built: Vec<Result<Option<Y>, E>> = x_steps
        .par_iter()
        .enumerate()
        .map(|(offset, x_step)| {
            let price_index = offset + 1;
            let y_step = y_steps.get(price_index).copied().ok_or_else(|| {
                E::from(SimulationError::walk_error(
                    "walker path shorter than x-steps",
                ))
            })?;
            let volatility = step_vols
                .as_ref()
                .and_then(|vols| vols.get(price_index).copied())
                .or(constant_vol);
            next_y(&y_step, volatility, x_step)
        })
        .collect();

    // Assemble serially in step order, mirroring the serial driver exactly:
    // an `Err` is propagated only if it occurs before any `Ok(None)` stop
    // signal; results past the stop are speculative and discarded.
    let mut steps: Vec<Step<Positive, Y>> = vec![walk_params.init_step.clone()];
    let mut y_index = *walk_params.ystep_ref().index();
    for (x_step, y_result) in x_steps.into_iter().zip(built) {
        let y_value = match y_result {
            Ok(Some(y_value)) => y_value,
            // The serial driver would have stopped here and never evaluated
            // the remaining steps.
            Ok(None) => break,
            Err(e) => return Err(e),
        };
        y_index += 1;
        steps.push(Step {
            x: x_step,
            y: Ystep::new(y_index, y_value),
        });
    }

    if steps.len() > walk_params.size {
        debug!(
            "walk produced {} steps, truncating to configured size {}",
            steps.len(),
            walk_params.size
        );
        steps.truncate(walk_params.size);
    }
    Ok(steps)
}

/// Generates a vector of `Step`s containing `Positive` x-values and `Positive` y-values.
///
/// Simulates the stochastic process selected by `walk_params.walk_type` for a plain
/// positive value (typically an underlying price) without rebuilding option chains.
///
/// # Contract (shared with [`crate::chains::generator_optionchain`] and
/// [`crate::series::generator_optionseries`])
///
/// * The returned vector always starts with `walk_params.init_step`.
/// * If the walker yields no values beyond the initial one (e.g. a size-1 walk),
///   only the initial step is returned.
/// * The walk is truncated when the x-step reaches expiration
///   (`SimulationError::ExpirationReached`); any other step-advance error is propagated.
/// * The result is truncated to at most `walk_params.size` steps.
///
/// # Arguments
///
/// * `walk_params` - A reference to the `WalkParams` struct containing the walk parameters.
///
/// # Returns
///
/// * `Ok(Vec<Step<Positive, Positive>>)` - A vector of `Step`s representing the simulated walk.
/// * `Err(ChainError)` - If the underlying simulator fails for any reason.
///
/// # Errors
///
/// Returns [`ChainError::Simulation`] (via the `From<SimulationError>` conversion) if the
/// random-walk generator returns an error — including
/// `SimulationError::InsufficientHistoricalData` when a `Historical` walk has fewer
/// prices than `walk_params.size`.
pub fn generator_positive(
    walk_params: &WalkParams<Positive, Positive>,
) -> Result<Vec<Step<Positive, Positive>>, ChainError> {
    // ChainError-typed adapter over the generic driver, kept for API
    // compatibility with the chain/series generator family; the driver
    // itself is error-generic and does not depend on the chains layer.
    walk_steps(walk_params, |new_price, _volatility, _x_step| {
        Ok(Some(*new_price))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExpirationDate;
    use crate::simulation::WalkTypeAble;
    use crate::simulation::randomwalk::RandomWalk;
    use crate::utils::time::convert_time_frame;
    use crate::utils::{Len, TimeFrame};
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    #[derive(Clone)]
    struct Walker {}
    impl Walker {
        fn new() -> Self {
            Walker {}
        }
    }
    impl WalkTypeAble<Positive, Positive> for Walker {}

    #[test]
    fn test_generator_positive() {
        let n_steps = 100;
        let initial_price = Positive::HUNDRED;
        let std_dev = pos_or_panic!(20.0);
        let walker = Box::new(Walker::new());
        let days = pos_or_panic!(30.0);

        let walk_params = WalkParams {
            size: n_steps,
            init_step: Step {
                x: Xstep::new(Positive::ONE, TimeFrame::Minute, ExpirationDate::Days(days)),
                y: Ystep::new(0, initial_price),
            },
            walk_type: WalkType::GeometricBrownian {
                dt: convert_time_frame(Positive::ONE / days, &TimeFrame::Minute, &TimeFrame::Day),
                drift: dec!(0.0),
                volatility: std_dev,
            },
            walker,
        };
        let random_walk =
            RandomWalk::new("Random Walk".to_string(), &walk_params, generator_positive)
                .expect("random walk construction");
        assert_eq!(random_walk.len(), n_steps);
    }

    #[test]
    fn test_generator_positive_early_return() {
        let initial_price = Positive::HUNDRED;
        let walker = Box::new(Walker::new());

        let walk_params = WalkParams {
            size: 1, // Just one step to trigger early return
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Minute,
                    ExpirationDate::Days(pos_or_panic!(30.0)),
                ),
                y: Ystep::new(0, initial_price),
            },
            walk_type: WalkType::GeometricBrownian {
                dt: pos_or_panic!(0.01),
                drift: dec!(0.0),
                volatility: pos_or_panic!(0.2),
            },
            walker,
        };

        let steps = match generator_positive(&walk_params) {
            Ok(steps) => steps,
            Err(e) => panic!("generator_positive failed: {e}"),
        };

        // We should just get the initial step back
        assert_eq!(steps.len(), 1);
    }

    /// A walker that returns an empty path; the driver must fall back to the
    /// init-only walk instead of panicking or erroring.
    #[test]
    fn test_walk_steps_empty_walker_output() {
        #[derive(Clone)]
        struct EmptyWalker {}
        impl WalkTypeAble<Positive, Positive> for EmptyWalker {
            fn brownian(
                &self,
                _params: &WalkParams<Positive, Positive>,
            ) -> Result<Vec<Positive>, SimulationError> {
                Ok(Vec::new())
            }
        }

        let walk_params = WalkParams {
            size: 5,
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Day,
                    ExpirationDate::Days(pos_or_panic!(30.0)),
                ),
                y: Ystep::new(0, Positive::HUNDRED),
            },
            walk_type: WalkType::Brownian {
                dt: pos_or_panic!(0.01),
                drift: dec!(0.0),
                volatility: pos_or_panic!(0.2),
            },
            walker: Box::new(EmptyWalker {}),
        };

        let steps = match generator_positive(&walk_params) {
            Ok(steps) => steps,
            Err(e) => panic!("empty walker output must not error: {e}"),
        };
        assert_eq!(steps.len(), 1, "empty walker output yields init-only walk");
    }

    /// Stochastic-vol walkers must expose a vol path aligned with prices
    /// and actually varying when vol-of-vol is high.
    #[test]
    fn test_heston_with_vol_exposes_varying_vols() {
        use rust_decimal_macros::dec;
        let walker = Walker::new();
        let params: WalkParams<Positive, Positive> = WalkParams {
            size: 50,
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Day,
                    ExpirationDate::Days(pos_or_panic!(365.0)),
                ),
                y: Ystep::new(0, Positive::HUNDRED),
            },
            walk_type: WalkType::Heston {
                dt: pos_or_panic!(0.004),
                drift: dec!(0.0),
                volatility: pos_or_panic!(0.2),
                kappa: Positive::TWO,
                theta: pos_or_panic!(0.04),
                xi: Positive::TWO, // high vol-of-vol so the path must move
                rho: dec!(-0.5),
            },
            walker: Box::new(Walker::new()),
        };
        let path = match walker.heston_with_vol(&params) {
            Ok(path) => path,
            Err(e) => panic!("heston_with_vol failed: {e}"),
        };
        let vols = match path.vols {
            Some(vols) => vols,
            None => panic!("heston must expose a vol path"),
        };
        assert_eq!(vols.len(), path.prices.len());
        assert_eq!(vols.first(), Some(&pos_or_panic!(0.2)));
        let varying = vols.windows(2).any(|w| w[0] != w[1]);
        assert!(varying, "high vol-of-vol Heston vol path cannot be flat");
    }

    /// Constant-vol walk types expose no vol path.
    #[test]
    fn test_generate_with_vol_none_for_gbm() {
        use rust_decimal_macros::dec;
        let walker = Walker::new();
        let params: WalkParams<Positive, Positive> = WalkParams {
            size: 10,
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Day,
                    ExpirationDate::Days(pos_or_panic!(30.0)),
                ),
                y: Ystep::new(0, Positive::HUNDRED),
            },
            walk_type: WalkType::GeometricBrownian {
                dt: pos_or_panic!(0.01),
                drift: dec!(0.0),
                volatility: pos_or_panic!(0.2),
            },
            walker: Box::new(Walker::new()),
        };
        let path = match walker.generate_with_vol(&params) {
            Ok(path) => path,
            Err(e) => panic!("generate_with_vol failed: {e}"),
        };
        assert!(path.vols.is_none());
        assert_eq!(path.prices.len(), 10);
    }

    /// The expanding-window Historical estimator must not see future data:
    /// with a flat prefix followed by a jump, the early estimates are zero
    /// while the post-jump estimates are strictly positive.
    #[test]
    fn test_expanding_window_vols_no_look_ahead() {
        let mut prices = vec![Positive::HUNDRED; 8];
        prices.push(pos_or_panic!(200.0));
        prices.push(pos_or_panic!(150.0));

        let vols = match expanding_window_vols(&prices, TimeFrame::Day) {
            Ok(Some(vols)) => vols,
            Ok(None) => panic!("estimator returned no vols"),
            Err(e) => panic!("estimator failed: {e}"),
        };
        assert_eq!(vols.len(), prices.len());
        // Index 5 sees only the flat prefix: zero variance, zero vol.
        assert_eq!(vols.get(5), Some(&Positive::ZERO));
        // The last index has seen the jump: strictly positive vol.
        match vols.last() {
            Some(last) => assert!(*last > Positive::ZERO),
            None => panic!("empty vols"),
        }
    }

    /// `next_y` returning `Ok(None)` must end the walk gracefully.
    #[test]
    fn test_walk_steps_next_y_none_stops_walk() {
        let walker = Box::new(Walker::new());
        let walk_params = WalkParams {
            size: 10,
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Day,
                    ExpirationDate::Days(pos_or_panic!(30.0)),
                ),
                y: Ystep::new(0, Positive::HUNDRED),
            },
            walk_type: WalkType::GeometricBrownian {
                dt: pos_or_panic!(0.01),
                drift: dec!(0.0),
                volatility: pos_or_panic!(0.2),
            },
            walker,
        };

        let mut calls = 0;
        let steps = match walk_steps(
            &walk_params,
            |price, _vol, _x| -> Result<Option<Positive>, ChainError> {
                calls += 1;
                if calls > 3 {
                    Ok(None)
                } else {
                    Ok(Some(*price))
                }
            },
        ) {
            Ok(steps) => steps,
            Err(e) => panic!("walk_steps failed: {e}"),
        };
        // init + 3 accepted steps
        assert_eq!(steps.len(), 4);
    }

    /// Multi-step behavior: prices, indices and time-to-expiry must all be
    /// exact under a deterministic ramp walker.
    #[test]
    fn test_generator_positive_multi_step_behavior() {
        use crate::simulation::walk_test_support::RampWalker;
        use rust_decimal_macros::dec;

        let size = 5;
        let walk_params = WalkParams {
            size,
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Day,
                    ExpirationDate::Days(pos_or_panic!(30.0)),
                ),
                y: Ystep::new(0, Positive::HUNDRED),
            },
            walk_type: WalkType::GeometricBrownian {
                dt: pos_or_panic!(0.01),
                drift: dec!(0.0),
                volatility: pos_or_panic!(0.2),
            },
            walker: Box::new(RampWalker {
                delta: Positive::TWO,
            }),
        };

        let steps = match generator_positive(&walk_params) {
            Ok(steps) => steps,
            Err(e) => panic!("generator_positive failed: {e}"),
        };
        assert_eq!(steps.len(), size);
        for (i, step) in steps.iter().enumerate() {
            // Price follows the ramp exactly.
            let expected_price = Positive::HUNDRED + Positive::TWO * i as f64;
            assert_eq!(*step.y.value(), expected_price, "price at step {i}");
            // Y index increments from the init step's index.
            assert_eq!(*step.y.index(), i as i32, "y index at step {i}");
            // Time-to-expiry decreases by exactly one day per step.
            let days = match step.x.days_left() {
                Ok(days) => days,
                Err(e) => panic!("days_left failed at step {i}: {e}"),
            };
            assert_eq!(days, pos_or_panic!(30.0 - i as f64), "days at step {i}");
        }
    }

    /// The walk must truncate exactly at expiration: with a 3-day expiry and
    /// daily steps, a size-10 request yields init + 3 steps.
    #[test]
    fn test_generator_positive_truncates_at_expiration() {
        use crate::simulation::walk_test_support::RampWalker;
        use rust_decimal_macros::dec;

        let walk_params = WalkParams {
            size: 10,
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Day,
                    ExpirationDate::Days(pos_or_panic!(3.0)),
                ),
                y: Ystep::new(0, Positive::HUNDRED),
            },
            walk_type: WalkType::GeometricBrownian {
                dt: pos_or_panic!(0.01),
                drift: dec!(0.0),
                volatility: pos_or_panic!(0.2),
            },
            walker: Box::new(RampWalker {
                delta: Positive::ONE,
            }),
        };

        let steps = match generator_positive(&walk_params) {
            Ok(steps) => steps,
            Err(e) => panic!("generator_positive failed: {e}"),
        };
        // init(3d) -> 2d -> 1d -> 0d, then ExpirationReached breaks.
        assert_eq!(steps.len(), 4);
        match steps.last() {
            Some(last) => match last.x.days_left() {
                Ok(days) => assert_eq!(days, Positive::ZERO),
                Err(e) => panic!("days_left failed: {e}"),
            },
            None => panic!("empty steps"),
        }
    }

    /// size = 0 yields the init-only walk from the shared driver.
    #[test]
    fn test_generator_positive_size_zero() {
        use crate::simulation::walk_test_support::RampWalker;
        use rust_decimal_macros::dec;

        let walk_params = WalkParams {
            size: 0,
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Day,
                    ExpirationDate::Days(pos_or_panic!(30.0)),
                ),
                y: Ystep::new(0, Positive::HUNDRED),
            },
            walk_type: WalkType::GeometricBrownian {
                dt: pos_or_panic!(0.01),
                drift: dec!(0.0),
                volatility: pos_or_panic!(0.2),
            },
            walker: Box::new(RampWalker {
                delta: Positive::ONE,
            }),
        };

        let steps = match generator_positive(&walk_params) {
            Ok(steps) => steps,
            Err(e) => panic!("size-0 walk must not error: {e}"),
        };
        assert_eq!(steps.len(), 1);
    }

    /// The parallel driver must produce exactly the serial driver's output
    /// for identical inputs.
    #[test]
    fn test_walk_steps_par_matches_serial() {
        use crate::simulation::walk_test_support::RampWalker;
        use rust_decimal_macros::dec;

        let walk_params = WalkParams {
            size: 8,
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Day,
                    ExpirationDate::Days(pos_or_panic!(30.0)),
                ),
                y: Ystep::new(0, Positive::HUNDRED),
            },
            walk_type: WalkType::GeometricBrownian {
                dt: pos_or_panic!(0.01),
                drift: dec!(0.0),
                volatility: pos_or_panic!(0.2),
            },
            walker: Box::new(RampWalker {
                delta: Positive::TWO,
            }),
        };

        let double = |price: &Positive,
                      _vol: Option<Positive>,
                      _x: &Xstep<Positive>|
         -> Result<Option<Positive>, ChainError> { Ok(Some(*price * 2.0)) };

        let serial = match walk_steps(&walk_params, double) {
            Ok(steps) => steps,
            Err(e) => panic!("serial driver failed: {e}"),
        };
        let parallel = match walk_steps_par(&walk_params, double) {
            Ok(steps) => steps,
            Err(e) => panic!("parallel driver failed: {e}"),
        };

        assert_eq!(serial.len(), parallel.len());
        for (i, (s, p)) in serial.iter().zip(parallel.iter()).enumerate() {
            assert_eq!(s.y.value(), p.y.value(), "y value differs at step {i}");
            assert_eq!(s.y.index(), p.y.index(), "y index differs at step {i}");
            let (sd, pd) = match (s.x.days_left(), p.x.days_left()) {
                (Ok(sd), Ok(pd)) => (sd, pd),
                _ => panic!("days_left failed at step {i}"),
            };
            assert_eq!(sd, pd, "x days differ at step {i}");
        }
    }

    /// Review regression (#419): an error from a speculative step past an
    /// `Ok(None)` stop must be discarded (the serial driver never evaluates
    /// those steps), while an error before the stop still propagates.
    #[test]
    fn test_walk_steps_par_error_after_stop_is_discarded() {
        use crate::simulation::walk_test_support::RampWalker;
        use rust_decimal_macros::dec;

        let make_params = || WalkParams {
            size: 8,
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Day,
                    ExpirationDate::Days(pos_or_panic!(30.0)),
                ),
                y: Ystep::new(0, Positive::HUNDRED),
            },
            walk_type: WalkType::GeometricBrownian {
                dt: pos_or_panic!(0.01),
                drift: dec!(0.0),
                volatility: pos_or_panic!(0.2),
            },
            walker: Box::new(RampWalker {
                delta: Positive::ONE, // prices 100, 101, 102, ...
            }),
        };

        // Stop at price 102 (step 2), error at price 103 (step 3): the
        // speculative error must be discarded and the walk succeed.
        let steps = match walk_steps_par(
            &make_params(),
            |price: &Positive, _vol, _x| -> Result<Option<Positive>, ChainError> {
                if *price == pos_or_panic!(103.0) {
                    Err(ChainError::invalid_parameters("test", "speculative error"))
                } else if *price == pos_or_panic!(102.0) {
                    Ok(None)
                } else {
                    Ok(Some(*price))
                }
            },
        ) {
            Ok(steps) => steps,
            Err(e) => panic!("error past the stop signal must be discarded: {e}"),
        };
        // init + step 1 (price 101); price 102 stopped the walk.
        assert_eq!(steps.len(), 2);

        // Error at price 101 (step 1), stop at price 102 (step 2): the
        // error precedes the stop and must propagate, as in the serial
        // driver.
        let result = walk_steps_par(
            &make_params(),
            |price: &Positive, _vol, _x| -> Result<Option<Positive>, ChainError> {
                if *price == pos_or_panic!(101.0) {
                    Err(ChainError::invalid_parameters("test", "early error"))
                } else if *price == pos_or_panic!(102.0) {
                    Ok(None)
                } else {
                    Ok(Some(*price))
                }
            },
        );
        assert!(result.is_err(), "error before the stop must propagate");
    }
}
