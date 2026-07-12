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
/// `constant_volatility`, `adjust_volatility`).
fn walk_volatility<Y>(walk_params: &WalkParams<Positive, Y>) -> Result<Option<Positive>, ChainError>
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
/// surfaces arithmetic overflow as `ChainError`.
fn expanding_window_vols(
    prices: &[Positive],
    timeframe: TimeFrame,
) -> Result<Option<Vec<Positive>>, ChainError> {
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
        |what: &str| ChainError::invalid_parameters("prices", &format!("{what} overflowed"));

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
/// * `Err(ChainError)` - If the walker, the volatility estimation, the
///   x-step advance, or `next_y` fail.
///
/// # Errors
///
/// Returns [`ChainError::Simulation`] (via the `From<SimulationError>`
/// conversion) if the walker returns an error — including
/// `SimulationError::InsufficientHistoricalData` when a `Historical` walk has
/// fewer prices than `walk_params.size` — and propagates errors from the
/// volatility estimation and from `next_y`.
pub fn walk_steps<Y, F>(
    walk_params: &WalkParams<Positive, Y>,
    mut next_y: F,
) -> Result<Vec<Step<Positive, Y>>, ChainError>
where
    Y: TryInto<Positive> + Display + Clone,
    F: FnMut(&Positive, Option<Positive>, &Xstep<Positive>) -> Result<Option<Y>, ChainError>,
{
    debug!("{}", walk_params);
    let path = walk_params.walker.generate_with_vol(walk_params)?;
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
            WalkType::Historical { timeframe, .. } => expanding_window_vols(&y_steps, *timeframe)?,
            _ => None,
        },
    };
    let constant_vol = if step_vols.is_some() {
        None
    } else {
        walk_volatility(walk_params)?
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
            Err(e) => return Err(e.into()),
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
        let steps = match walk_steps(&walk_params, |price, _vol, _x| {
            calls += 1;
            if calls > 3 {
                Ok(None)
            } else {
                Ok(Some(*price))
            }
        }) {
            Ok(steps) => steps,
            Err(e) => panic!("walk_steps failed: {e}"),
        };
        // init + 3 accepted steps
        assert_eq!(steps.len(), 4);
    }
}
