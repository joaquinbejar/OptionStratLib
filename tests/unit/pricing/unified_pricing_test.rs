/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2024
******************************************************************************/

use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use optionstratlib::pricing::{Priceable, PricingEngine, price_option};
use optionstratlib::simulation::simulator::Simulator;
use optionstratlib::simulation::steps::{Step, Xstep, Ystep};
use optionstratlib::simulation::{WalkParams, WalkType, WalkTypeAble};
use optionstratlib::utils::TimeFrame;
use optionstratlib::{ExpirationDate, Options, Positive, pos_or_panic};
use rust_decimal_macros::dec;
use std::fmt::Display;
use std::ops::AddAssign;

// A minimal walker for testing
struct TestWalker;

impl<X, Y> WalkTypeAble<X, Y> for TestWalker
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
}

/// Simple generator function for testing
fn simple_generator(params: &WalkParams<Positive, Positive>) -> Vec<Step<Positive, Positive>> {
    let mut out = Vec::with_capacity(params.size);
    let mut current = params.init_step.clone();
    out.push(current.clone());
    for i in 1..params.size {
        let new_y = current.get_positive_value() + pos_or_panic!(i as f64 * 0.1);
        current = current.next(new_y).expect("step.next should succeed");
        out.push(current.clone());
    }
    out
}

/// Helper function to create a standard test option
fn create_test_option() -> Options {
    Options {
        option_type: OptionType::European,
        side: Side::Long,
        underlying_symbol: "TEST".to_string(),
        strike_price: pos_or_panic!(100.0),
        expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
        implied_volatility: pos_or_panic!(0.2),
        quantity: Positive::ONE,
        underlying_price: pos_or_panic!(105.0),
        risk_free_rate: dec!(0.05),
        option_style: OptionStyle::Call,
        dividend_yield: pos_or_panic!(0.01),
        exotic_params: None,
    }
}

#[test]
fn test_price_option_black_scholes() {
    let option = create_test_option();
    let engine = PricingEngine::ClosedFormBS;

    let result = price_option(&option, &engine);

    assert!(result.is_ok(), "Black-Scholes pricing should succeed");
    let price = result.unwrap();
    assert!(price > Positive::ZERO, "Price should be positive");

    // For a call option with S > K, price should be reasonable
    assert!(
        price < pos_or_panic!(50.0),
        "Price should be less than underlying price"
    );
}

#[test]
fn test_priceable_trait_black_scholes() {
    let option = create_test_option();
    let engine = PricingEngine::ClosedFormBS;

    let result = option.price(&engine);

    assert!(
        result.is_ok(),
        "Priceable trait should work with Black-Scholes"
    );
    let price = result.unwrap();
    assert!(price > Positive::ZERO, "Price should be positive");
}

#[test]
fn test_price_option_monte_carlo() {
    let option = create_test_option();
    let init_price = option.underlying_price;
    let size = 365;

    let init_step = Step {
        x: Xstep::new(
            pos_or_panic!(1.0),
            TimeFrame::Day,
            ExpirationDate::Days(pos_or_panic!(size as f64)),
        ),
        y: Ystep::new(0, init_price),
    };

    let walk = WalkType::GeometricBrownian {
        dt: pos_or_panic!(1.0),
        drift: dec!(0.0),
        volatility: option.implied_volatility,
    };

    let params = WalkParams {
        size,
        init_step,
        walk_type: walk,
        walker: Box::new(TestWalker),
    };
    let simulator = Simulator::new("MC Test".to_string(), 1000, &params, simple_generator);

    let engine = PricingEngine::MonteCarlo { simulator };
    let result = price_option(&option, &engine);

    assert!(result.is_ok(), "Monte Carlo pricing should succeed");
    let price = result.unwrap();
    assert!(price > Positive::ZERO, "Price should be positive");
}

#[test]
fn test_priceable_trait_monte_carlo() {
    let option = create_test_option();
    let init_price = option.underlying_price;
    let size = 365;

    let init_step = Step {
        x: Xstep::new(
            pos_or_panic!(1.0),
            TimeFrame::Day,
            ExpirationDate::Days(pos_or_panic!(size as f64)),
        ),
        y: Ystep::new(0, init_price),
    };

    let walk = WalkType::GeometricBrownian {
        dt: pos_or_panic!(1.0),
        drift: dec!(0.0),
        volatility: option.implied_volatility,
    };

    let params = WalkParams {
        size,
        init_step,
        walk_type: walk,
        walker: Box::new(TestWalker),
    };
    let simulator = Simulator::new("MC Test".to_string(), 1000, &params, simple_generator);

    let engine = PricingEngine::MonteCarlo { simulator };
    let result = option.price(&engine);

    assert!(
        result.is_ok(),
        "Priceable trait should work with Monte Carlo"
    );
    let price = result.unwrap();
    assert!(price > Positive::ZERO, "Price should be positive");
}

#[test]
fn test_put_option_pricing() {
    let mut option = create_test_option();
    option.option_style = OptionStyle::Put;
    option.underlying_price = pos_or_panic!(95.0); // Out of the money put

    let engine = PricingEngine::ClosedFormBS;
    let result = price_option(&option, &engine);

    assert!(result.is_ok(), "Put option pricing should succeed");
    let price = result.unwrap();
    assert!(price > Positive::ZERO, "Put price should be positive");
}

#[test]
fn test_short_position_pricing() {
    let mut option = create_test_option();
    option.side = Side::Short;

    let engine = PricingEngine::ClosedFormBS;
    let result = price_option(&option, &engine);

    // Short positions should return negative prices in Black-Scholes
    // but the unified API should handle this appropriately
    assert!(result.is_ok(), "Short position pricing should succeed");
}

#[test]
fn test_monte_carlo_with_heston() {
    let option = create_test_option();
    let init_price = option.underlying_price;
    let size = 365;

    let init_step = Step {
        x: Xstep::new(
            pos_or_panic!(1.0),
            TimeFrame::Day,
            ExpirationDate::Days(pos_or_panic!(size as f64)),
        ),
        y: Ystep::new(0, init_price),
    };

    let walk = WalkType::Heston {
        dt: pos_or_panic!(1.0),
        drift: dec!(0.0),
        volatility: option.implied_volatility,
        kappa: pos_or_panic!(1.5),
        theta: pos_or_panic!(0.04),
        xi: pos_or_panic!(0.5),
        rho: dec!(-0.7),
    };

    let params = WalkParams {
        size,
        init_step,
        walk_type: walk,
        walker: Box::new(TestWalker),
    };
    let simulator = Simulator::new("Heston Test".to_string(), 500, &params, simple_generator);

    let engine = PricingEngine::MonteCarlo { simulator };
    let result = price_option(&option, &engine);

    assert!(result.is_ok(), "Heston model pricing should succeed");
    let price = result.unwrap();
    assert!(price > Positive::ZERO, "Price should be positive");
}

#[test]
fn test_monte_carlo_with_jump_diffusion() {
    let option = create_test_option();
    let init_price = option.underlying_price;
    let size = 365;

    let init_step = Step {
        x: Xstep::new(
            pos_or_panic!(1.0),
            TimeFrame::Day,
            ExpirationDate::Days(pos_or_panic!(size as f64)),
        ),
        y: Ystep::new(0, init_price),
    };

    let walk = WalkType::JumpDiffusion {
        dt: pos_or_panic!(1.0),
        drift: dec!(0.0),
        volatility: option.implied_volatility,
        intensity: pos_or_panic!(0.5),
        jump_mean: dec!(-0.03),
        jump_volatility: pos_or_panic!(0.2),
    };

    let params = WalkParams {
        size,
        init_step,
        walk_type: walk,
        walker: Box::new(TestWalker),
    };
    let simulator = Simulator::new(
        "Jump Diffusion Test".to_string(),
        500,
        &params,
        simple_generator,
    );

    let engine = PricingEngine::MonteCarlo { simulator };
    let result = price_option(&option, &engine);

    assert!(
        result.is_ok(),
        "Jump Diffusion model pricing should succeed"
    );
    let price = result.unwrap();
    assert!(price > Positive::ZERO, "Price should be positive");
}

#[test]
fn test_monte_carlo_with_telegraph() {
    let option = create_test_option();
    let init_price = option.underlying_price;
    let size = 365;

    let init_step = Step {
        x: Xstep::new(
            pos_or_panic!(1.0),
            TimeFrame::Day,
            ExpirationDate::Days(pos_or_panic!(size as f64)),
        ),
        y: Ystep::new(0, init_price),
    };

    let walk = WalkType::Telegraph {
        dt: pos_or_panic!(1.0),
        drift: dec!(0.0),
        volatility: option.implied_volatility,
        lambda_up: pos_or_panic!(0.8),
        lambda_down: pos_or_panic!(1.2),
        vol_multiplier_up: None,
        vol_multiplier_down: None,
    };

    let params = WalkParams {
        size,
        init_step,
        walk_type: walk,
        walker: Box::new(TestWalker),
    };
    let simulator = Simulator::new("Telegraph Test".to_string(), 500, &params, simple_generator);

    let engine = PricingEngine::MonteCarlo { simulator };
    let result = price_option(&option, &engine);

    assert!(result.is_ok(), "Telegraph model pricing should succeed");
    let price = result.unwrap();
    assert!(price > Positive::ZERO, "Price should be positive");
}

#[test]
fn test_error_handling() {
    // Test with an option that might cause issues
    let mut option = create_test_option();
    option.expiration_date = ExpirationDate::Days(pos_or_panic!(0.1)); // Very short expiry

    let engine = PricingEngine::ClosedFormBS;
    let result = price_option(&option, &engine);

    // Should still work, even with edge cases
    assert!(result.is_ok(), "Should handle edge cases gracefully");
}

// Note: A full pricing consistency test between Black-Scholes and Monte Carlo
// would require a more sophisticated random walk generator that properly
// implements the stochastic differential equations for each model.
// The simple_generator used in these tests is deterministic and for testing
// the API structure only.
