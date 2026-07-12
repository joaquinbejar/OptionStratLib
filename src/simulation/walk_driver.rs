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
use rust_decimal::Decimal;
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
///   the stochastic process via [`crate::simulation::WalkTypeAble::generate`].
/// * `next_y` - Builds the y-value for one step from `(new_price,
///   walk_volatility, advanced_x_step)`. Returning `Ok(None)` ends the walk
///   gracefully at the previous step.
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
    let y_steps = walk_params.walker.generate(walk_params).map_err(E::from)?;
    if y_steps.len() <= 1 {
        // Preserve the init-step invariant when the walker produces no
        // values beyond the initial one; downstream consumers expect at
        // least the initial step to be present.
        return Ok(vec![walk_params.init_step.clone()]);
    }
    let volatility = walk_volatility(walk_params).map_err(E::from)?;

    let mut steps: Vec<Step<Positive, Y>> = vec![walk_params.init_step.clone()];
    let mut previous_x_step = walk_params.init_step.x;
    let mut y_index = *walk_params.ystep_ref().index();

    // The first walker value duplicates the init step, so skip it.
    for y_step in y_steps.iter().skip(1) {
        previous_x_step = match previous_x_step.next() {
            Ok(x_step) => x_step,
            // Reaching expiration is the normal end of a walk: truncate.
            Err(SimulationError::ExpirationReached) => break,
            // Any other step-advance failure is a real error: propagate.
            Err(e) => return Err(E::from(e)),
        };
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
}
