/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/3/25
******************************************************************************/
use crate::ExpirationDate;
use crate::chains::OptionChain;
use crate::chains::utils::OptionChainBuildParams;
use crate::error::ChainError;
use crate::simulation::steps::Step;
use crate::simulation::{WalkParams, walk_steps};
use core::option::Option;
use positive::Positive;
#[cfg(test)]
use positive::pos_or_panic;

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
/// Implemented on top of the shared walk driver
/// ([`crate::simulation::walk_steps`]); this function only supplies the chain-rebuild
/// closure.
///
/// # Contract (shared with [`crate::simulation::generator_positive`] and
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
    // Derived lazily on the first rebuilt step so that walks that never
    // rebuild (size <= 1) do not require a parameterizable initial chain.
    let mut build_params: Option<OptionChainBuildParams> = None;
    walk_steps(walk_params, |new_price, volatility, x_step| {
        let params = match &mut build_params {
            Some(params) => params,
            // Derive the build parameters once from the initial chain; every
            // step's chain is anchored to the initial chain's shape instead
            // of feeding back params re-derived from the previous rebuilt
            // chain on each iteration.
            none => none.insert(walk_params.ystep_ref().value().to_build_params()?),
        };
        let chain =
            create_chain_from_step(params, new_price, volatility, Some(*x_step.datetime()))?;
        Ok(Some(chain))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ExpirationDate;

    use crate::simulation::randomwalk::RandomWalk;
    use crate::simulation::steps::{Xstep, Ystep};
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
}

#[cfg(test)]
mod generators_coverage_tests {
    use super::*;
    use positive::{Positive, spos};

    use crate::ExpirationDate;
    use crate::chains::generators::generator_optionchain;
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
    impl WalkTypeAble<Positive, OptionChain> for TestWalker {}

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
