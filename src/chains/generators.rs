/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/3/25
******************************************************************************/
use core::option::Option;
use crate::chains::OptionChain;
use crate::simulation::WalkParams;
use crate::simulation::steps::{Step, Ystep};
use crate::{Positive, pos};
use tracing::{debug, info};

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
/// * `Err(Box<dyn std::error::Error>)` - If an error occurs during the creation of the new `OptionChain`.
///
fn create_chain_from_step(
    previous_y_step: &Ystep<OptionChain>,
    new_price: &Positive,
    volatility: Option<Positive>,
) -> Result<OptionChain, Box<dyn std::error::Error>> {
    let chain = previous_y_step.value();
    let mut chain_params = chain.to_build_params()?;
    chain_params.set_underlying_price(new_price);
    if let Some(volatility) = volatility {
        chain_params.set_implied_volatility(Some(volatility));
    }

    let new_chain = OptionChain::build_chain(&chain_params);
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
    let mut y_steps = walk_params.walker.geometric_brownian(walk_params).unwrap();
    let _ = y_steps.remove(0);
    let mut steps: Vec<Step<Positive, OptionChain>> = vec![walk_params.init_step.clone()];
    let mut previous_x_step = walk_params.init_step.x;
    let mut previous_y_step = walk_params.ystep();

    for y_step in y_steps.iter() {
        previous_x_step = match previous_x_step.next() {
            Ok(x_step) => x_step,
            Err(_) => break,
        };
        // convert y_step to OptionChain
        let y_step_chain: OptionChain =
            create_chain_from_step(&previous_y_step, y_step, Some(pos!(0.20))).unwrap();
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
    info!("{}", walk_params);
    let mut y_steps = walk_params.walker.geometric_brownian(walk_params).unwrap();
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
        let new_price: Positive = pos!(5790.0);
        let step = Step {
            x: Xstep::new(
                Positive::ONE,
                TimeFrame::Minute,
                ExpirationDate::Days(pos!(30.0)),
            ),
            y: Ystep::new(0, initial_price),
        };

        let result = create_chain_from_step(&step.y, &new_price, None);
        assert!(result.is_ok());

        let new_chain = result.unwrap();
        assert!(new_chain.len() > 1);
        assert_eq!(*new_chain.atm_strike().unwrap(), pos!(5790.0));
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
        let days = pos!(30.0);
        let std_dev = pos!(20.0);
        let walker = Box::new(WalkerOptionChain::new());

        let walk_params = WalkParams {
            size: n_steps,
            init_step: Step {
                x: Xstep::new(Positive::ONE, TimeFrame::Minute, ExpirationDate::Days(days)),
                y: Ystep::new(0, initial_chain),
            },
            walk_type: WalkType::GeometricBrownian {
                dt: convert_time_frame(pos!(1.0) / days, &TimeFrame::Minute, &TimeFrame::Day), // TODO
                drift: dec!(0.0),
                volatility: std_dev,
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
        let initial_price = pos!(100.0);
        let std_dev = pos!(20.0);
        let walker = Box::new(Walker::new());
        let days = pos!(30.0);

        let walk_params = WalkParams {
            size: n_steps,
            init_step: Step {
                x: Xstep::new(Positive::ONE, TimeFrame::Minute, ExpirationDate::Days(days)),
                y: Ystep::new(0, initial_price),
            },
            walk_type: WalkType::GeometricBrownian {
                dt: convert_time_frame(pos!(1.0) / days, &TimeFrame::Minute, &TimeFrame::Day), // TODO
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
