/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/3/25
******************************************************************************/
use positive::{pos_or_panic, spos};
use crate::chains::OptionChain;
use crate::error::ChainError;
use crate::simulation::steps::{Step, Ystep};
use crate::simulation::{WalkParams, WalkType};
use crate::utils::TimeFrame;
use crate::utils::others::calculate_log_returns;
use crate::volatility::{adjust_volatility, constant_volatility};
use crate::{ExpirationDate, Positive};
use core::option::Option;
use rust_decimal::Decimal;
use tracing::debug;

/// Creates a new `OptionChain` from a previous `Ystep` and a new price.
///
/// This function takes a reference to the previous `Ystep` containing an `OptionChain`,
/// a reference to the new underlying price, and an optional new volatility. It creates a new
/// `OptionChain` with the updated price and volatility.
///
/// # Arguments
///
/// * `previous_y_step` - A reference to the previous `Ystep` containing the `OptionChain`.
/// * `new_price` - A reference to the new underlying price.
/// * `volatility` - An optional new implied volatility.
///
/// # Returns
///
/// * `Ok(OptionChain)` - A new `OptionChain` with the updated parameters.
/// * `Err(optionstratlib::error::Error)` - If an error occurs during the creation of the new `OptionChain`.
///
fn create_chain_from_step(
    previous_y_step: &Ystep<OptionChain>,
    new_price: Option<Box<Positive>>,
    volatility: Option<Positive>,
    expiration_date: Option<ExpirationDate>,
) -> Result<OptionChain, ChainError> {
    let chain = previous_y_step.value();
    let mut chain_params = chain.to_build_params()?;
    chain_params.set_underlying_price(new_price);
    if let Some(volatility) = volatility {
        chain_params.set_implied_volatility(volatility);
    }
    if let Some(exp_date) = expiration_date {
        chain_params.price_params.expiration_date = Some(exp_date);
    }

    let mut new_chain = OptionChain::build_chain(&chain_params);
    new_chain.update_greeks();
    Ok(new_chain)
}

/// Generates a vector of `Step`s containing `Positive` x-values and `OptionChain` y-values.
///
/// This function simulates a geometric Brownian motion walk for option chains, generating a sequence
/// of steps with updated option chains based on the changing underlying price. It uses a fixed volatility
/// of 0.20.
///
/// # Arguments
///
/// * `walk_params` - A reference to the `WalkParams` struct containing the walk parameters.
///
/// # Returns
///
/// * `Vec<Step<Positive, OptionChain>>` - A vector of `Step`s representing the simulated walk.
///
pub fn generator_optionchain(
    walk_params: &WalkParams<Positive, OptionChain>,
) -> Vec<Step<Positive, OptionChain>> {
    debug!("{}", walk_params);
    let (mut y_steps, volatility) = match &walk_params.walk_type {
        WalkType::Brownian { volatility, .. } => (
            walk_params.walker.brownian(walk_params).unwrap(),
            Some(*volatility),
        ),
        WalkType::GeometricBrownian { volatility, .. } => (
            walk_params.walker.geometric_brownian(walk_params).unwrap(),
            Some(*volatility),
        ),
        WalkType::LogReturns { volatility, .. } => (
            walk_params.walker.log_returns(walk_params).unwrap(),
            Some(*volatility),
        ),
        WalkType::MeanReverting { volatility, .. } => (
            walk_params.walker.mean_reverting(walk_params).unwrap(),
            Some(*volatility),
        ),
        WalkType::JumpDiffusion { volatility, .. } => (
            walk_params.walker.jump_diffusion(walk_params).unwrap(),
            Some(*volatility),
        ),
        WalkType::Garch { volatility, .. } => (
            walk_params.walker.garch(walk_params).unwrap(),
            Some(*volatility),
        ),
        WalkType::Heston { volatility, .. } => (
            walk_params.walker.heston(walk_params).unwrap(),
            Some(*volatility),
        ),
        WalkType::Custom { volatility, .. } => (
            walk_params.walker.custom(walk_params).unwrap(),
            Some(*volatility),
        ),
        WalkType::Telegraph { volatility, .. } => (
            walk_params.walker.telegraph(walk_params).unwrap(),
            Some(*volatility),
        ),
        WalkType::Historical {
            timeframe, prices, ..
        } => {
            if prices.is_empty() || prices.len() < walk_params.size {
                (Vec::new(), None)
            } else {
                let log_returns: Vec<Decimal> = calculate_log_returns(prices)
                    .unwrap()
                    .iter()
                    .map(|p| p.to_dec())
                    .collect();
                let constant_volatility = constant_volatility(&log_returns).unwrap();
                let implied_volatility =
                    adjust_volatility(constant_volatility, *timeframe, TimeFrame::Year).unwrap();
                (
                    walk_params.walker.historical(walk_params).unwrap(),
                    Some(implied_volatility),
                )
            }
        }
    };
    if y_steps.is_empty() {
        return vec![];
    }

    let _ = y_steps.remove(0); // remove initial step from y_steps to avoid early return
    let mut steps: Vec<Step<Positive, OptionChain>> = vec![walk_params.init_step.clone()];
    let mut previous_x_step = walk_params.init_step.x;
    let mut previous_y_step = walk_params.ystep();

    if let Some(volatility) = volatility {
        volatility
    } else {
        pos_or_panic!(0.20)
    };

    for y_step in y_steps.iter() {
        previous_x_step = match previous_x_step.next() {
            Ok(x_step) => x_step,
            Err(_) => break,
        };
        // convert y_step to OptionChain with updated expiration date
        let expiration_date = *previous_x_step.datetime();
        let y_step_chain: OptionChain = create_chain_from_step(
            &previous_y_step,
            Some(Box::new(*y_step)),
            volatility,
            Some(expiration_date),
        )
        .unwrap();
        previous_y_step = previous_y_step.next(y_step_chain).clone();
        let step = Step {
            x: previous_x_step,
            y: previous_y_step.clone(),
        };

        steps.push(step)
    }

    assert!(steps.len() <= walk_params.size);
    steps
}

/// Generates a vector of `Step`s containing `Positive` x-values and `Positive` y-values.
///
/// This function simulates a geometric Brownian motion walk for positive values, generating a sequence
/// of steps with updated positive values.
///
/// # Arguments
///
/// * `walk_params` - A reference to the `WalkParams` struct containing the walk parameters.
///
/// # Returns
///
/// * `Vec<Step<Positive, Positive>>` - A vector of `Step`s representing the simulated walk.
///
pub fn generator_positive(
    walk_params: &WalkParams<Positive, Positive>,
) -> Vec<Step<Positive, Positive>> {
    debug!("{}", walk_params);
    let mut y_steps = match &walk_params.walk_type {
        WalkType::Brownian { .. } => walk_params.walker.brownian(walk_params).unwrap(),
        WalkType::GeometricBrownian { .. } => {
            walk_params.walker.geometric_brownian(walk_params).unwrap()
        }
        WalkType::LogReturns { .. } => walk_params.walker.log_returns(walk_params).unwrap(),
        WalkType::MeanReverting { .. } => walk_params.walker.mean_reverting(walk_params).unwrap(),
        WalkType::JumpDiffusion { .. } => walk_params.walker.jump_diffusion(walk_params).unwrap(),
        WalkType::Garch { .. } => walk_params.walker.garch(walk_params).unwrap(),
        WalkType::Heston { .. } => walk_params.walker.heston(walk_params).unwrap(),
        WalkType::Custom { .. } => walk_params.walker.custom(walk_params).unwrap(),
        WalkType::Telegraph { .. } => walk_params.walker.telegraph(walk_params).unwrap(),
        WalkType::Historical { .. } => walk_params.walker.historical(walk_params).unwrap(),
    };

    let _ = y_steps.remove(0);
    let mut steps: Vec<Step<Positive, Positive>> = vec![walk_params.init_step.clone()];

    let mut previous_x_step = walk_params.init_step.x;
    let mut previous_y_step = walk_params.init_step.y.clone();

    for y_step in y_steps.iter() {
        previous_x_step = match previous_x_step.next() {
            Ok(x_step) => x_step,
            Err(_) => break,
        };
        previous_y_step = previous_y_step.next(*y_step);
        let step = Step {
            x: previous_x_step,
            y: previous_y_step.clone(),
        };
        steps.push(step)
    }
    assert!(steps.len() <= walk_params.size);

    steps
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

        let result = create_chain_from_step(&step.y, Some(Box::new(new_price)), None, None);
        assert!(result.is_ok());

        let new_chain = result.unwrap();
        assert!(new_chain.len() > 1);
        assert_eq!(*new_chain.atm_strike().unwrap(), pos_or_panic!(5790.0));
        for option in new_chain.get_single_iter() {
            assert!(option.valid_put() || option.valid_call());
        }
    }

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
                dt: convert_time_frame(
                    Positive::ONE / days,
                    &TimeFrame::Minute,
                    &TimeFrame::Day,
                ),
                drift: dec!(0.0),
                volatility: std_dev / 100.0,
            },
            walker,
        };

        let random_walk = RandomWalk::new(
            "Random Walk".to_string(),
            &walk_params,
            generator_optionchain,
        );
        assert_eq!(random_walk.len(), n_steps);
    }

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
                dt: convert_time_frame(
                    Positive::ONE / days,
                    &TimeFrame::Minute,
                    &TimeFrame::Day,
                ),
                drift: dec!(0.0),
                volatility: std_dev,
            },
            walker,
        };
        let random_walk =
            RandomWalk::new("Random Walk".to_string(), &walk_params, generator_positive);
        assert_eq!(random_walk.len(), n_steps);
    }
}

#[cfg(test)]
mod generators_coverage_tests {
    use positive::Positive;
    use super::*;

    use crate::ExpirationDate;
    use crate::chains::generators::{generator_optionchain, generator_positive};
    use crate::simulation::steps::{Step, Xstep, Ystep};
    use crate::simulation::{WalkParams, WalkType, WalkTypeAble};
    use crate::utils::TimeFrame;
    use crate::utils::time::get_tomorrow_formatted;
    use rust_decimal_macros::dec;

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

        let steps = generator_optionchain(&walk_params);

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

        let steps = generator_positive(&walk_params);

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

        let steps = generator_optionchain(&walk_params);

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

        let steps = generator_optionchain(&walk_params);

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

        let steps = generator_optionchain(&walk_params);

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

        let steps = generator_optionchain(&walk_params);

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

        let steps = generator_optionchain(&walk_params);

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

        let steps = generator_optionchain(&walk_params);

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

        let steps = generator_optionchain(&walk_params);

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

        let steps = generator_optionchain(&walk_params);

        // We should just get the initial step back
        assert_eq!(steps.len(), 1);
    }
}
