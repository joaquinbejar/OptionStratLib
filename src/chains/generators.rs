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
use rust_decimal_macros::dec;

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
        // `build_chain` rejects IV > 100% and IV == 0; simulated
        // stochastic-vol paths can spike above 100% or touch the zero
        // boundary (CIR truncation), so clamp into (0, 1] to keep the
        // walk alive. The floor literal is compile-time positive, so the
        // fallback branch is unreachable.
        let min_walk_iv = Positive::new_decimal(dec!(0.0001)).unwrap_or(Positive::ONE);
        let volatility = volatility.min(Positive::ONE).max(min_walk_iv);
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
/// each step with the new price, the step's volatility, and the decayed expiration date.
/// For constant-volatility walks the chain IV is the walk type's `volatility`
/// parameter; for stochastic-volatility walks (`Garch`, `Heston`, `Custom`,
/// `Telegraph`) it follows the simulated per-step volatility path; for `Historical`
/// it is an expanding-window estimate over the walked prices that uses no future
/// data (capped at 100% before stamping the chain).
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

    /// Regression for #408: under a stochastic-volatility walk the rebuilt
    /// chains must follow the simulated per-step vol path instead of being
    /// frozen at the walk's initial volatility.
    #[test]
    fn test_generator_optionchain_heston_vol_reaches_chains() {
        use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};

        let price_params = OptionDataPriceParams::new(
            Some(Box::new(Positive::HUNDRED)),
            Some(ExpirationDate::Days(pos_or_panic!(60.0))),
            Some(dec!(0.05)),
            Some(pos_or_panic!(0.02)),
            Some("TEST".to_string()),
        );
        let chain_params = OptionChainBuildParams::new(
            "TEST".to_string(),
            None,
            10,
            Some(pos_or_panic!(5.0)),
            dec!(-0.2),
            dec!(0.1),
            pos_or_panic!(0.02),
            2,
            price_params,
            pos_or_panic!(0.2),
        );
        let initial_chain = match OptionChain::build_chain(&chain_params) {
            Ok(chain) => chain,
            Err(e) => panic!("initial chain build failed: {e}"),
        };

        let walk_params = WalkParams {
            size: 6,
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Day,
                    ExpirationDate::Days(pos_or_panic!(60.0)),
                ),
                y: Ystep::new(0, initial_chain),
            },
            walk_type: WalkType::Heston {
                dt: pos_or_panic!(0.004),
                drift: dec!(0.0),
                volatility: pos_or_panic!(0.2),
                kappa: Positive::TWO,
                theta: pos_or_panic!(0.04),
                xi: Positive::TWO, // high vol-of-vol: the vol path must move
                rho: dec!(-0.5),
            },
            walker: Box::new(WalkerOptionChain::new()),
        };

        let steps = match generator_optionchain(&walk_params) {
            Ok(steps) => steps,
            Err(e) => panic!("generator_optionchain failed: {e}"),
        };
        assert_eq!(steps.len(), 6);

        let atm_ivs: Vec<Positive> = steps
            .iter()
            .skip(1)
            .map(|step| match step.y.value().get_atm_implied_volatility() {
                Ok(iv) => *iv,
                Err(e) => panic!("rebuilt chain has no ATM IV: {e}"),
            })
            .collect();
        let varying = atm_ivs.windows(2).any(|w| w[0] != w[1]);
        assert!(
            varying,
            "chain IVs are frozen despite a stochastic-vol walk: {atm_ivs:?}"
        );
    }

    /// Multi-step behavior under a deterministic ramp walker: rebuilt chains
    /// must track the walked price, keep the walk volatility, decay their
    /// expiration with the x-step, and increment the y index.
    #[test]
    fn test_generator_optionchain_multi_step_behavior() {
        use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
        use crate::simulation::walk_test_support::RampWalker;

        let init_days = 60.0;
        let price_params = OptionDataPriceParams::new(
            Some(Box::new(Positive::HUNDRED)),
            Some(ExpirationDate::Days(pos_or_panic!(init_days))),
            Some(dec!(0.05)),
            Some(pos_or_panic!(0.02)),
            Some("TEST".to_string()),
        );
        let chain_params = OptionChainBuildParams::new(
            "TEST".to_string(),
            None,
            10,
            Some(pos_or_panic!(5.0)),
            dec!(-0.2),
            dec!(0.1),
            pos_or_panic!(0.02),
            2,
            price_params,
            pos_or_panic!(0.25),
        );
        let initial_chain = match OptionChain::build_chain(&chain_params) {
            Ok(chain) => chain,
            Err(e) => panic!("initial chain build failed: {e}"),
        };

        let size = 4;
        let walk_params = WalkParams {
            size,
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Day,
                    ExpirationDate::Days(pos_or_panic!(init_days)),
                ),
                y: Ystep::new(0, initial_chain),
            },
            walk_type: WalkType::GeometricBrownian {
                dt: pos_or_panic!(0.01),
                drift: dec!(0.0),
                volatility: pos_or_panic!(0.25),
            },
            walker: Box::new(RampWalker {
                delta: Positive::TWO,
            }),
        };

        let steps = match generator_optionchain(&walk_params) {
            Ok(steps) => steps,
            Err(e) => panic!("generator_optionchain failed: {e}"),
        };
        assert_eq!(steps.len(), size);

        for (i, step) in steps.iter().enumerate().skip(1) {
            let chain = step.y.value();
            // Underlying tracks the deterministic ramp exactly.
            let expected_price = Positive::HUNDRED + Positive::TWO * i as f64;
            assert_eq!(
                chain.underlying_price, expected_price,
                "underlying at step {i}"
            );
            // Y index increments.
            assert_eq!(*step.y.index(), i as i32, "y index at step {i}");
            // X time-to-expiry decreases one day per step.
            let x_days = match step.x.days_left() {
                Ok(days) => days,
                Err(e) => panic!("days_left failed at step {i}: {e}"),
            };
            assert_eq!(x_days, pos_or_panic!(init_days - i as f64));
            // The rebuilt chain's expiration follows the x-step (parsed back
            // from a date string, so allow a one-day rounding tolerance).
            let chain_days = match chain.get_expiration() {
                Some(exp) => match exp.get_days() {
                    Ok(days) => days,
                    Err(e) => panic!("chain expiration days failed at step {i}: {e}"),
                },
                None => panic!("rebuilt chain has no expiration at step {i}"),
            };
            let diff = (chain_days.to_dec() - x_days.to_dec()).abs();
            assert!(
                diff <= rust_decimal::Decimal::ONE,
                "chain expiration {chain_days} != x-step days {x_days} at step {i}"
            );
            // ATM IV tracks the constant walk volatility (ATM moneyness is
            // near zero, so the skew factor is ~1).
            let atm_iv = match chain.get_atm_implied_volatility() {
                Ok(iv) => *iv,
                Err(e) => panic!("ATM IV missing at step {i}: {e}"),
            };
            let iv_diff = (atm_iv.to_dec() - dec!(0.25)).abs();
            assert!(
                iv_diff < dec!(0.01),
                "ATM IV {atm_iv} does not track walk volatility at step {i}"
            );
        }
    }

    /// Historical multi-step behavior: underlying prices must replay the
    /// provided history and the final step's IV must match the historical
    /// estimate computed over exactly the walked prices (no look-ahead).
    #[test]
    fn test_generator_optionchain_historical_multi_step() {
        use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
        use crate::simulation::walk_test_support::RampWalker;
        use crate::utils::others::calculate_log_returns;
        use crate::volatility::{adjust_volatility as annualize, constant_volatility};
        use rust_decimal::Decimal;

        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(104.0),
            pos_or_panic!(98.0),
            pos_or_panic!(103.0),
            pos_or_panic!(101.0),
            pos_or_panic!(99.0), // beyond the walked slice: must not be used
        ];
        let size = 5;

        let price_params = OptionDataPriceParams::new(
            Some(Box::new(Positive::HUNDRED)),
            Some(ExpirationDate::Days(pos_or_panic!(60.0))),
            Some(dec!(0.05)),
            Some(pos_or_panic!(0.02)),
            Some("TEST".to_string()),
        );
        let chain_params = OptionChainBuildParams::new(
            "TEST".to_string(),
            None,
            10,
            Some(pos_or_panic!(5.0)),
            dec!(-0.2),
            dec!(0.1),
            pos_or_panic!(0.02),
            2,
            price_params,
            pos_or_panic!(0.25),
        );
        let initial_chain = match OptionChain::build_chain(&chain_params) {
            Ok(chain) => chain,
            Err(e) => panic!("initial chain build failed: {e}"),
        };

        let walk_params = WalkParams {
            size,
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Day,
                    ExpirationDate::Days(pos_or_panic!(60.0)),
                ),
                y: Ystep::new(0, initial_chain),
            },
            walk_type: WalkType::Historical {
                timeframe: TimeFrame::Day,
                prices: prices.clone(),
                symbol: None,
            },
            walker: Box::new(RampWalker {
                delta: Positive::ONE, // unused: Historical keeps the default
            }),
        };

        let steps = match generator_optionchain(&walk_params) {
            Ok(steps) => steps,
            Err(e) => panic!("generator_optionchain failed: {e}"),
        };
        assert_eq!(steps.len(), size);

        // Underlying prices replay the provided history.
        for (i, step) in steps.iter().enumerate().skip(1) {
            let expected = match prices.get(i) {
                Some(p) => *p,
                None => panic!("missing price {i}"),
            };
            assert_eq!(
                step.y.value().underlying_price,
                expected,
                "underlying at step {i}"
            );
        }

        // Final step vol == expanding-window estimate over the WALKED slice
        // (prices[..size]), not the full vector.
        let walked = match prices.get(..size) {
            Some(w) => w,
            None => panic!("short prices"),
        };
        let log_returns: Vec<Decimal> = match calculate_log_returns(walked) {
            Ok(returns) => returns.iter().map(|p| p.to_dec()).collect(),
            Err(e) => panic!("log returns failed: {e}"),
        };
        let expected_vol = match constant_volatility(&log_returns) {
            Ok(vol) => match annualize(vol, TimeFrame::Day, TimeFrame::Year) {
                Ok(vol) => vol,
                Err(e) => panic!("annualize failed: {e}"),
            },
            Err(e) => panic!("constant_volatility failed: {e}"),
        };
        let last_iv = match steps.last() {
            Some(step) => match step.y.value().get_atm_implied_volatility() {
                Ok(iv) => *iv,
                Err(e) => panic!("last ATM IV missing: {e}"),
            },
            None => panic!("empty steps"),
        };
        let capped_expected = expected_vol.min(Positive::ONE);
        let diff = (last_iv.to_dec() - capped_expected.to_dec()).abs();
        assert!(
            diff < dec!(0.02),
            "last-step IV {last_iv} != expected historical estimate {capped_expected}"
        );
    }

    /// Contract: an empty walker output yields the init-only walk (no panic,
    /// no error) for the chain generator too.
    #[test]
    fn test_generator_optionchain_empty_walker_output() {
        use crate::simulation::walk_test_support::EmptyWalker;
        use crate::utils::time::get_tomorrow_formatted;
        use positive::spos;

        let chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            get_tomorrow_formatted(),
            Some(dec!(0.05)),
            spos!(0.02),
        );
        let walk_params = WalkParams {
            size: 5,
            init_step: Step {
                x: Xstep::new(
                    Positive::ONE,
                    TimeFrame::Day,
                    ExpirationDate::Days(pos_or_panic!(30.0)),
                ),
                y: Ystep::new(0, chain),
            },
            walk_type: WalkType::GeometricBrownian {
                dt: pos_or_panic!(0.01),
                drift: dec!(0.0),
                volatility: pos_or_panic!(0.2),
            },
            walker: Box::new(EmptyWalker),
        };
        let steps = match generator_optionchain(&walk_params) {
            Ok(steps) => steps,
            Err(e) => panic!("empty walker output must not error: {e}"),
        };
        assert_eq!(steps.len(), 1);
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
