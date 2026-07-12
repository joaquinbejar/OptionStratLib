/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/3/25
******************************************************************************/
use crate::ExpirationDate;
use crate::chains::OptionChain;
use crate::chains::utils::OptionChainBuildParams;
use crate::error::{ChainError, SimulationError};
use crate::simulation::steps::{Step, Ystep};
use crate::simulation::{WalkParams, WalkType};
use crate::utils::TimeFrame;
use crate::utils::others::calculate_log_returns;
use crate::volatility::{adjust_volatility, constant_volatility};
use core::option::Option;
use positive::Positive;
#[cfg(test)]
use positive::pos_or_panic;
use rust_decimal::Decimal;
use tracing::debug;

/// Creates a new `OptionChain` from pre-derived build parameters and a new price.
///
/// This function clones the provided build parameters and overrides the underlying
/// price, and optionally the implied volatility and expiration date, before building
/// the new chain. Deriving the parameters once (outside the simulation loop) instead
/// of from each freshly built chain keeps every step's chain shape anchored to the
/// initial chain and avoids re-scanning the chain on every step.
///
/// Greeks (delta/gamma) are computed by `OptionChain::build_chain` itself for every
/// strike, so no separate greek pass is needed here.
///
/// # Arguments
///
/// * `build_params` - Build parameters derived from the initial `OptionChain`.
/// * `new_price` - A reference to the new underlying price.
/// * `volatility` - An optional new implied volatility.
/// * `expiration_date` - An optional new expiration date.
///
/// # Returns
///
/// * `Ok(OptionChain)` - A new `OptionChain` with the updated parameters.
/// * `Err(ChainError)` - If an error occurs during the creation of the new `OptionChain`.
///
fn create_chain_from_step(
    build_params: &OptionChainBuildParams,
    new_price: &Positive,
    volatility: Option<Positive>,
    expiration_date: Option<ExpirationDate>,
) -> Result<OptionChain, ChainError> {
    let mut chain_params = build_params.clone();
    chain_params.set_underlying_price(Some(Box::new(*new_price)));
    if let Some(volatility) = volatility {
        chain_params.set_implied_volatility(volatility);
    }
    if let Some(exp_date) = expiration_date {
        chain_params.price_params.expiration_date = Some(exp_date);
    }

    OptionChain::build_chain(&chain_params)
}

/// Generates a vector of `Step`s containing `Positive` x-values and `OptionChain` y-values.
///
/// Simulates the stochastic process selected by `walk_params.walk_type` (Brownian,
/// GeometricBrownian, LogReturns, MeanReverting, JumpDiffusion, Garch, Heston, Custom,
/// Telegraph, or Historical) for the underlying price and rebuilds the option chain at
/// each step with the new price, the walk's volatility, and the decayed expiration date.
/// For synthetic walks the chain IV is the walk type's `volatility` parameter; for
/// `Historical` it is the annualized constant volatility estimated from the provided
/// price history.
///
/// # Contract (shared with [`generator_positive`] and
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
/// * `Ok(Vec<Step<Positive, OptionChain>>)` - A vector of `Step`s representing the simulated walk.
/// * `Err(ChainError)` - If the underlying simulator, volatility helpers, or chain construction
///   fails for any reason.
///
/// # Errors
///
/// Returns [`ChainError::Simulation`] (via the `From<SimulationError>` conversion) if the
/// random-walk generator returns an error — including
/// `SimulationError::InsufficientHistoricalData` when a `Historical` walk has fewer
/// prices than `walk_params.size` — and propagates errors from the historical helpers
/// (`calculate_log_returns`, `constant_volatility`, `adjust_volatility`) or from
/// rebuilding the chain.
pub fn generator_optionchain(
    walk_params: &WalkParams<Positive, OptionChain>,
) -> Result<Vec<Step<Positive, OptionChain>>, ChainError> {
    debug!("{}", walk_params);
    let (y_steps, volatility) = match &walk_params.walk_type {
        WalkType::Brownian { volatility, .. } => {
            (walk_params.walker.brownian(walk_params)?, Some(*volatility))
        }
        WalkType::GeometricBrownian { volatility, .. } => (
            walk_params.walker.geometric_brownian(walk_params)?,
            Some(*volatility),
        ),
        WalkType::LogReturns { volatility, .. } => (
            walk_params.walker.log_returns(walk_params)?,
            Some(*volatility),
        ),
        WalkType::MeanReverting { volatility, .. } => (
            walk_params.walker.mean_reverting(walk_params)?,
            Some(*volatility),
        ),
        WalkType::JumpDiffusion { volatility, .. } => (
            walk_params.walker.jump_diffusion(walk_params)?,
            Some(*volatility),
        ),
        WalkType::Garch { volatility, .. } => {
            (walk_params.walker.garch(walk_params)?, Some(*volatility))
        }
        WalkType::Heston { volatility, .. } => {
            (walk_params.walker.heston(walk_params)?, Some(*volatility))
        }
        WalkType::Custom { volatility, .. } => {
            (walk_params.walker.custom(walk_params)?, Some(*volatility))
        }
        WalkType::Telegraph { volatility, .. } => (
            walk_params.walker.telegraph(walk_params)?,
            Some(*volatility),
        ),
        WalkType::Historical {
            timeframe, prices, ..
        } => {
            // Let the walker validate the price history first: insufficient
            // data surfaces as a typed
            // `SimulationError::InsufficientHistoricalData` instead of a
            // silent init-only walk that callers cannot distinguish from a
            // legitimate size-1 walk.
            let steps = walk_params.walker.historical(walk_params)?;
            let log_returns: Vec<Decimal> = calculate_log_returns(prices)?
                .iter()
                .map(|p| p.to_dec())
                .collect();
            let constant_volatility = constant_volatility(&log_returns)?;
            let implied_volatility =
                adjust_volatility(constant_volatility, *timeframe, TimeFrame::Year)?;
            (steps, Some(implied_volatility))
        }
    };
    if y_steps.len() <= 1 {
        // Preserve the init-step invariant when the underlying walk produces
        // no points beyond the initial one (e.g., Historical with
        // insufficient `prices`, or a size-1 walk); downstream consumers
        // expect at least the initial step to be present. Returning early
        // also avoids deriving build params from a chain that will never be
        // rebuilt.
        return Ok(vec![walk_params.init_step.clone()]);
    }

    // Derive the build parameters once from the initial chain; every step's
    // chain is anchored to the initial chain's shape instead of feeding back
    // params re-derived from the previous rebuilt chain on each iteration.
    let build_params = walk_params.ystep_ref().value().to_build_params()?;
    let mut steps: Vec<Step<Positive, OptionChain>> = vec![walk_params.init_step.clone()];
    let mut previous_x_step = walk_params.init_step.x;
    let mut y_index = *walk_params.ystep_ref().index();

    // The first walker value duplicates the init step, so skip it.
    for y_step in y_steps.iter().skip(1) {
        previous_x_step = match previous_x_step.next() {
            Ok(x_step) => x_step,
            // Reaching expiration is the normal end of a walk: truncate.
            Err(SimulationError::ExpirationReached) => break,
            // Any other step-advance failure is a real error: propagate.
            Err(e) => return Err(e.into()),
        };
        // convert y_step to OptionChain with updated expiration date
        let expiration_date = *previous_x_step.datetime();
        let y_step_chain: OptionChain =
            create_chain_from_step(&build_params, y_step, volatility, Some(expiration_date))?;
        y_index += 1;
        let step = Step {
            x: previous_x_step,
            y: Ystep::new(y_index, y_step_chain),
        };

        steps.push(step)
    }

    if steps.len() > walk_params.size {
        debug!(
            "generator_optionchain produced {} steps, truncating to configured size {}",
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
/// # Contract (shared with [`generator_optionchain`] and
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
    debug!("{}", walk_params);
    let y_steps = match &walk_params.walk_type {
        WalkType::Brownian { .. } => walk_params.walker.brownian(walk_params)?,
        WalkType::GeometricBrownian { .. } => walk_params.walker.geometric_brownian(walk_params)?,
        WalkType::LogReturns { .. } => walk_params.walker.log_returns(walk_params)?,
        WalkType::MeanReverting { .. } => walk_params.walker.mean_reverting(walk_params)?,
        WalkType::JumpDiffusion { .. } => walk_params.walker.jump_diffusion(walk_params)?,
        WalkType::Garch { .. } => walk_params.walker.garch(walk_params)?,
        WalkType::Heston { .. } => walk_params.walker.heston(walk_params)?,
        WalkType::Custom { .. } => walk_params.walker.custom(walk_params)?,
        WalkType::Telegraph { .. } => walk_params.walker.telegraph(walk_params)?,
        WalkType::Historical { .. } => walk_params.walker.historical(walk_params)?,
    };

    if y_steps.len() <= 1 {
        // Preserve the init-step invariant when the walker produces no
        // values beyond the initial one; downstream consumers expect at
        // least the initial step to be present.
        return Ok(vec![walk_params.init_step.clone()]);
    }

    let mut steps: Vec<Step<Positive, Positive>> = vec![walk_params.init_step.clone()];

    let mut previous_x_step = walk_params.init_step.x;
    let mut previous_y_step = walk_params.init_step.y.clone();

    // The first walker value duplicates the init step, so skip it.
    for y_step in y_steps.iter().skip(1) {
        previous_x_step = match previous_x_step.next() {
            Ok(x_step) => x_step,
            // Reaching expiration is the normal end of a walk: truncate.
            Err(SimulationError::ExpirationReached) => break,
            // Any other step-advance failure is a real error: propagate.
            Err(e) => return Err(e.into()),
        };
        previous_y_step = previous_y_step.next(*y_step);
        let step = Step {
            x: previous_x_step,
            y: previous_y_step.clone(),
        };
        steps.push(step)
    }
    if steps.len() > walk_params.size {
        debug!(
            "generator_positive produced {} steps, truncating to configured size {}",
            steps.len(),
            walk_params.size
        );
        steps.truncate(walk_params.size);
    }

    Ok(steps)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ExpirationDate;

    use crate::simulation::randomwalk::RandomWalk;
    use crate::simulation::steps::Xstep;
    use crate::simulation::{WalkType, WalkTypeAble};
    use crate::utils::time::{convert_time_frame, get_x_days_formatted};
    use crate::utils::{Len, TimeFrame};
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_chain_from_step() {
        let mut initial_price =
            OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json").unwrap();
        initial_price.update_expiration_date(get_x_days_formatted(2));
        let new_price: Positive = pos_or_panic!(5790.0);
        let step = Step {
            x: Xstep::new(
                Positive::ONE,
                TimeFrame::Minute,
                ExpirationDate::Days(pos_or_panic!(30.0)),
            ),
            y: Ystep::new(0, initial_price),
        };

        let build_params = match step.y.value().to_build_params() {
            Ok(params) => params,
            Err(e) => panic!("to_build_params failed: {e}"),
        };
        let result = create_chain_from_step(&build_params, &new_price, None, None);
        assert!(result.is_ok());

        let new_chain = result.unwrap();
        assert!(new_chain.len() > 1);
        assert_eq!(*new_chain.atm_strike().unwrap(), pos_or_panic!(5790.0));
        for option in new_chain.get_single_iter() {
            assert!(option.valid_put() || option.valid_call());
        }
    }

    #[derive(Clone)]
    struct WalkerOptionChain {}
    impl WalkerOptionChain {
        fn new() -> Self {
            WalkerOptionChain {}
        }
    }
    impl WalkTypeAble<Positive, OptionChain> for WalkerOptionChain {}

    #[test]
    fn test_create_chain_from_step_with_volatility_change() {
        let n_steps = 4;
        let mut initial_chain =
            OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json").unwrap();
        initial_chain.update_expiration_date(get_x_days_formatted(2));
        let days = pos_or_panic!(30.0);
        let std_dev = pos_or_panic!(20.0);
        let walker = Box::new(WalkerOptionChain::new());

        let walk_params = WalkParams {
            size: n_steps,
            init_step: Step {
                x: Xstep::new(Positive::ONE, TimeFrame::Minute, ExpirationDate::Days(days)),
                y: Ystep::new(0, initial_chain),
            },
            walk_type: WalkType::GeometricBrownian {
                dt: convert_time_frame(Positive::ONE / days, &TimeFrame::Minute, &TimeFrame::Day),
                drift: dec!(0.0),
                volatility: std_dev / 100.0,
            },
            walker,
        };

        let random_walk = RandomWalk::new(
            "Random Walk".to_string(),
            &walk_params,
            generator_optionchain,
        )
        .expect("random walk construction");
        assert_eq!(random_walk.len(), n_steps);
    }

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
}

#[cfg(test)]
mod generators_coverage_tests {
    use super::*;
    use positive::{Positive, spos};

    use crate::ExpirationDate;
    use crate::chains::generators::{generator_optionchain, generator_positive};
    use crate::simulation::steps::{Step, Xstep, Ystep};
    use crate::simulation::{WalkParams, WalkType, WalkTypeAble};
    use crate::utils::TimeFrame;
    use crate::utils::time::get_tomorrow_formatted;
    use rust_decimal_macros::dec;

    #[derive(Clone)]
    struct TestWalker {}
    impl TestWalker {
        fn new() -> Self {
            TestWalker {}
        }
    }
    impl WalkTypeAble<Positive, Positive> for TestWalker {}
    impl WalkTypeAble<Positive, OptionChain> for TestWalker {}

    // Test for line 73 in generators.rs
    #[test]
    fn test_generator_optionchain_early_return() {
        // Create a small walk with only one step to test early return
        let chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            get_tomorrow_formatted(),
            Some(dec!(0.05)),
            spos!(0.02),
        );

        let walker = Box::new(TestWalker::new());

        let walk_params = WalkParams {
            size: 1, // Just one step to trigger early return
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Minute,
                    ExpirationDate::Days(pos_or_panic!(30.0)),
                ),
                y: Ystep::new(0, chain),
            },
            walk_type: WalkType::GeometricBrownian {
                dt: pos_or_panic!(0.01),
                drift: dec!(0.0),
                volatility: pos_or_panic!(0.2),
            },
            walker,
        };

        let steps = generator_optionchain(&walk_params).unwrap();

        // We should just get the initial step back
        assert_eq!(steps.len(), 1);
    }

    // Test for line 118 in generators.rs
    #[test]
    fn test_generator_positive_early_return() {
        // Create a small walk with only one step to test early return
        let initial_price = Positive::HUNDRED;
        let walker = Box::new(TestWalker::new());

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

        let steps = generator_positive(&walk_params).unwrap();

        // We should just get the initial step back
        assert_eq!(steps.len(), 1);
    }

    #[test]
    fn test_generator_optionchain_brownian() {
        // Create a small walk with only one step to test early return
        let chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            get_tomorrow_formatted(),
            Some(dec!(0.05)),
            spos!(0.02),
        );

        let walker = Box::new(TestWalker::new());

        let walk_params = WalkParams {
            size: 1, // Just one step to trigger early return
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Minute,
                    ExpirationDate::Days(pos_or_panic!(30.0)),
                ),
                y: Ystep::new(0, chain),
            },
            walk_type: WalkType::Brownian {
                dt: pos_or_panic!(0.01),
                drift: dec!(0.0),
                volatility: pos_or_panic!(0.2),
            },
            walker,
        };

        let steps = generator_optionchain(&walk_params).unwrap();

        // We should just get the initial step back
        assert_eq!(steps.len(), 1);
    }

    #[test]
    fn test_generator_optionchain_log_returns() {
        // Create a small walk with only one step to test early return
        let chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            get_tomorrow_formatted(),
            Some(dec!(0.05)),
            spos!(0.02),
        );

        let walker = Box::new(TestWalker::new());

        let walk_params = WalkParams {
            size: 1, // Just one step to trigger early return
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Minute,
                    ExpirationDate::Days(pos_or_panic!(30.0)),
                ),
                y: Ystep::new(0, chain),
            },
            walk_type: WalkType::LogReturns {
                dt: pos_or_panic!(0.01),
                expected_return: Default::default(),
                volatility: pos_or_panic!(0.2),
                autocorrelation: None,
            },
            walker,
        };

        let steps = generator_optionchain(&walk_params).unwrap();

        // We should just get the initial step back
        assert_eq!(steps.len(), 1);
    }

    #[test]
    fn test_generator_optionchain_mean_reverting() {
        // Create a small walk with only one step to test early return
        let chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            get_tomorrow_formatted(),
            Some(dec!(0.05)),
            spos!(0.02),
        );

        let walker = Box::new(TestWalker::new());

        let walk_params = WalkParams {
            size: 1, // Just one step to trigger early return
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Minute,
                    ExpirationDate::Days(pos_or_panic!(30.0)),
                ),
                y: Ystep::new(0, chain),
            },
            walk_type: WalkType::MeanReverting {
                dt: pos_or_panic!(0.01),
                volatility: pos_or_panic!(0.2),
                speed: Default::default(),
                mean: Default::default(),
            },
            walker,
        };

        let steps = generator_optionchain(&walk_params).unwrap();

        // We should just get the initial step back
        assert_eq!(steps.len(), 1);
    }

    #[test]
    fn test_generator_optionchain_jump_diffusion() {
        // Create a small walk with only one step to test early return
        let chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            get_tomorrow_formatted(),
            Some(dec!(0.05)),
            spos!(0.02),
        );

        let walker = Box::new(TestWalker::new());

        let walk_params = WalkParams {
            size: 1, // Just one step to trigger early return
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Minute,
                    ExpirationDate::Days(pos_or_panic!(30.0)),
                ),
                y: Ystep::new(0, chain),
            },
            walk_type: WalkType::JumpDiffusion {
                dt: pos_or_panic!(0.01),
                drift: Default::default(),
                volatility: pos_or_panic!(0.2),
                intensity: Default::default(),
                jump_mean: Default::default(),
                jump_volatility: Default::default(),
            },
            walker,
        };

        let steps = generator_optionchain(&walk_params).unwrap();

        // We should just get the initial step back
        assert_eq!(steps.len(), 1);
    }

    #[test]
    fn test_generator_optionchain_garch() {
        // Create a small walk with only one step to test early return
        let chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            get_tomorrow_formatted(),
            Some(dec!(0.05)),
            spos!(0.02),
        );

        let walker = Box::new(TestWalker::new());

        let walk_params = WalkParams {
            size: 1, // Just one step to trigger early return
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Minute,
                    ExpirationDate::Days(pos_or_panic!(30.0)),
                ),
                y: Ystep::new(0, chain),
            },
            walk_type: WalkType::Garch {
                dt: pos_or_panic!(0.01),
                drift: Default::default(),
                volatility: pos_or_panic!(0.2),
                alpha: Default::default(),
                beta: Default::default(),
            },
            walker,
        };

        let steps = generator_optionchain(&walk_params).unwrap();

        // We should just get the initial step back
        assert_eq!(steps.len(), 1);
    }

    #[test]
    fn test_generator_optionchain_heston() {
        // Create a small walk with only one step to test early return
        let chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            get_tomorrow_formatted(),
            Some(dec!(0.05)),
            spos!(0.02),
        );

        let walker = Box::new(TestWalker::new());

        let walk_params = WalkParams {
            size: 1, // Just one step to trigger early return
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Minute,
                    ExpirationDate::Days(pos_or_panic!(30.0)),
                ),
                y: Ystep::new(0, chain),
            },
            walk_type: WalkType::Heston {
                dt: pos_or_panic!(0.01),
                drift: Default::default(),
                volatility: pos_or_panic!(0.2),
                kappa: Default::default(),
                theta: Default::default(),
                xi: Default::default(),
                rho: Default::default(),
            },
            walker,
        };

        let steps = generator_optionchain(&walk_params).unwrap();

        // We should just get the initial step back
        assert_eq!(steps.len(), 1);
    }

    #[test]
    fn test_generator_optionchain_custom() {
        // Create a small walk with only one step to test early return
        let chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            get_tomorrow_formatted(),
            Some(dec!(0.05)),
            spos!(0.02),
        );

        let walker = Box::new(TestWalker::new());

        let walk_params = WalkParams {
            size: 1, // Just one step to trigger early return
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Minute,
                    ExpirationDate::Days(pos_or_panic!(30.0)),
                ),
                y: Ystep::new(0, chain),
            },
            walk_type: WalkType::Custom {
                dt: pos_or_panic!(0.01),
                drift: Default::default(),
                volatility: pos_or_panic!(0.2),
                vov: Default::default(),
                vol_speed: Default::default(),
                vol_mean: Default::default(),
            },
            walker,
        };

        let steps = generator_optionchain(&walk_params).unwrap();

        // We should just get the initial step back
        assert_eq!(steps.len(), 1);
    }

    #[test]
    fn test_generator_optionchain_historical() {
        // Create a small walk with only one step to test early return
        let chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            get_tomorrow_formatted(),
            Some(dec!(0.05)),
            spos!(0.02),
        );

        let walker = Box::new(TestWalker::new());

        let walk_params = WalkParams {
            size: 1, // Just one step to trigger early return
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Minute,
                    ExpirationDate::Days(pos_or_panic!(30.0)),
                ),
                y: Ystep::new(0, chain),
            },
            walk_type: WalkType::Historical {
                timeframe: TimeFrame::Microsecond,
                prices: vec![
                    Positive::HUNDRED,
                    pos_or_panic!(101.0),
                    pos_or_panic!(102.0),
                ],
                symbol: None,
            },
            walker,
        };

        let steps = generator_optionchain(&walk_params).unwrap();

        // We should just get the initial step back
        assert_eq!(steps.len(), 1);
    }
}
