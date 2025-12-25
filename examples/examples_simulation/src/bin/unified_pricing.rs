use positive::pos_or_panic;
/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2024
******************************************************************************/

//! # Unified Pricing System Example
//!
//! This example demonstrates the unified pricing API that allows pricing options
//! using different models through a single interface.
//!
//! ## Features Demonstrated
//! - Black-Scholes closed-form pricing
//! - Monte Carlo pricing with various stochastic models:
//!   - Geometric Brownian Motion
//!   - Heston stochastic volatility
//!   - Jump Diffusion
//!   - Telegraph process
//! - Using the Priceable trait for clean API

use optionstratlib::prelude::*;
use std::fmt::Display;
use std::ops::AddAssign;

// Simple walker implementation for demonstration
struct SimpleWalker;

impl<X, Y> WalkTypeAble<X, Y> for SimpleWalker
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
}

// Simple generator for demonstration purposes
fn demo_generator(params: &WalkParams<Positive, Positive>) -> Vec<Step<Positive, Positive>> {
    let mut out = Vec::with_capacity(params.size);
    let mut current = params.init_step.clone();
    out.push(current.clone());
    for i in 1..params.size {
        let new_y = current.get_positive_value() + pos_or_panic!(i as f64 * 0.05);
        current = current.next(new_y).expect("step.next should succeed");
        out.push(current.clone());
    }
    out
}

fn main() {
    println!("=== Unified Pricing System Example ===\n");

    // Create a sample European call option
    let option = Options {
        option_type: OptionType::European,
        side: Side::Long,
        underlying_symbol: "AAPL".to_string(),
        strike_price: pos_or_panic!(150.0),
        expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
        implied_volatility: pos_or_panic!(0.25),
        quantity: Positive::ONE,
        underlying_price: pos_or_panic!(155.0),
        risk_free_rate: dec!(0.05),
        option_style: OptionStyle::Call,
        dividend_yield: pos_or_panic!(0.02),
        exotic_params: None,
    };

    println!("Option Details:");
    println!("  Symbol: {}", option.underlying_symbol);
    println!("  Type: {:?} {:?}", option.option_style, option.option_type);
    println!("  Strike: ${}", option.strike_price);
    println!("  Underlying: ${}", option.underlying_price);
    println!("  Volatility: {}", option.implied_volatility);
    println!("  Days to Expiry: {:?}\n", option.expiration_date);

    // 1. Black-Scholes Pricing
    println!("1. Black-Scholes Closed-Form Pricing:");
    let bs_engine = PricingEngine::ClosedFormBS;
    match price_option(&option, &bs_engine) {
        Ok(price) => println!("   Price: ${:.4}", price),
        Err(e) => println!("   Error: {}", e),
    }

    // Alternative using Priceable trait
    match option.price(&bs_engine) {
        Ok(price) => println!("   Price (via trait): ${:.4}\n", price),
        Err(e) => println!("   Error: {}\n", e),
    }

    // 2. Monte Carlo with Geometric Brownian Motion
    println!("2. Monte Carlo with Geometric Brownian Motion:");
    let size = 365;
    let init_step = Step {
        x: Xstep::new(
            Positive::ONE,
            TimeFrame::Day,
            ExpirationDate::Days(pos_or_panic!(size as f64)),
        ),
        y: Ystep::new(0, option.underlying_price),
    };

    let gbm_walk = WalkType::GeometricBrownian {
        dt: Positive::ONE,
        drift: dec!(0.0),
        volatility: option.implied_volatility,
    };

    let gbm_params = WalkParams {
        size,
        init_step: init_step.clone(),
        walk_type: gbm_walk,
        walker: Box::new(SimpleWalker),
    };

    let gbm_simulator = Simulator::new(
        "GBM Simulator".to_string(),
        1000,
        &gbm_params,
        demo_generator,
    );

    let mc_engine = PricingEngine::MonteCarlo {
        simulator: gbm_simulator,
    };
    match option.price(&mc_engine) {
        Ok(price) => println!("   Price: ${:.4}\n", price),
        Err(e) => println!("   Error: {}\n", e),
    }

    // 3. Monte Carlo with Heston Model
    println!("3. Monte Carlo with Heston Stochastic Volatility:");
    let heston_walk = WalkType::Heston {
        dt: Positive::ONE,
        drift: dec!(0.0),
        volatility: option.implied_volatility,
        kappa: Positive::TWO,  // Mean reversion speed
        theta: pos_or_panic!(0.04), // Long-term variance
        xi: pos_or_panic!(0.3),     // Volatility of volatility
        rho: dec!(-0.7),            // Correlation
    };

    let heston_params = WalkParams {
        size,
        init_step: init_step.clone(),
        walk_type: heston_walk,
        walker: Box::new(SimpleWalker),
    };

    let heston_simulator = Simulator::new(
        "Heston Simulator".to_string(),
        1000,
        &heston_params,
        demo_generator,
    );

    let heston_engine = PricingEngine::MonteCarlo {
        simulator: heston_simulator,
    };
    match option.price(&heston_engine) {
        Ok(price) => println!("   Price: ${:.4}\n", price),
        Err(e) => println!("   Error: {}\n", e),
    }

    // 4. Monte Carlo with Jump Diffusion
    println!("4. Monte Carlo with Jump Diffusion:");
    let jump_walk = WalkType::JumpDiffusion {
        dt: Positive::ONE,
        drift: dec!(0.0),
        volatility: option.implied_volatility,
        intensity: pos_or_panic!(0.5),        // Jump frequency
        jump_mean: dec!(-0.02),               // Average jump size
        jump_volatility: pos_or_panic!(0.15), // Jump size volatility
    };

    let jump_params = WalkParams {
        size,
        init_step: init_step.clone(),
        walk_type: jump_walk,
        walker: Box::new(SimpleWalker),
    };

    let jump_simulator = Simulator::new(
        "Jump Diffusion Simulator".to_string(),
        1000,
        &jump_params,
        demo_generator,
    );

    let jump_engine = PricingEngine::MonteCarlo {
        simulator: jump_simulator,
    };
    match option.price(&jump_engine) {
        Ok(price) => println!("   Price: ${:.4}\n", price),
        Err(e) => println!("   Error: {}\n", e),
    }

    // 5. Monte Carlo with Telegraph Process
    println!("5. Monte Carlo with Telegraph Process:");
    let telegraph_walk = WalkType::Telegraph {
        dt: Positive::ONE,
        drift: dec!(0.0),
        volatility: option.implied_volatility,
        lambda_up: pos_or_panic!(0.8),   // Transition rate to up state
        lambda_down: pos_or_panic!(1.2), // Transition rate to down state
        vol_multiplier_up: None,
        vol_multiplier_down: None,
    };

    let telegraph_params = WalkParams {
        size,
        init_step,
        walk_type: telegraph_walk,
        walker: Box::new(SimpleWalker),
    };

    let telegraph_simulator = Simulator::new(
        "Telegraph Simulator".to_string(),
        1000,
        &telegraph_params,
        demo_generator,
    );

    let telegraph_engine = PricingEngine::MonteCarlo {
        simulator: telegraph_simulator,
    };
    match option.price(&telegraph_engine) {
        Ok(price) => println!("   Price: ${:.4}\n", price),
        Err(e) => println!("   Error: {}\n", e),
    }

    println!("=== Example Complete ===");
}
