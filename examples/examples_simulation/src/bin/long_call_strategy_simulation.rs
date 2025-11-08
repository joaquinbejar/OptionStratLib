/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 8/11/25
******************************************************************************/
//! Long Call Strategy Simulation using Simulate Trait
//!
//! This simulation evaluates the performance of a Long Call options strategy
//! across multiple random walk scenarios using the `Simulate` trait implementation.
//!
//! # Strategy Overview
//!
//! A Long Call involves buying a call option to profit from upward price movement.
//! The strategy is profitable when:
//! - The underlying price rises above the strike price plus premium paid
//! - The option gains intrinsic value
//!
//! # Exit Conditions
//!
//! The simulation closes positions when:
//! - **Profit Target**: Premium increases by 100% (we can sell at double the cost)
//! - **Expiration**: The option reaches its expiration date and P&L is calculated
//!
//! # Running the Simulation
//!
//! ```bash
//! cargo run --package examples_simulation --bin long_call_strategy_simulation
//! ```
//!
//! # Output
//!
//! The simulation provides:
//! - Win rate and loss rate statistics
//! - Average P&L per trade
//! - Maximum profit and loss
//! - Average holding period
//! - Distribution of exit reasons
//! - PNG visualization of the last simulation in `Draws/Simulation/long_call_strategy_simulation.png`

use indicatif::{ProgressBar, ProgressStyle};
use optionstratlib::chains::generator_positive;
use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use optionstratlib::prelude::*;
use optionstratlib::simulation::simulator::Simulator;
use optionstratlib::simulation::steps::{Step, Xstep, Ystep};
use optionstratlib::simulation::{ExitPolicy, Simulate, WalkParams, WalkType, WalkTypeAble};
use optionstratlib::strategies::LongCall;
use optionstratlib::utils::setup_logger;
use optionstratlib::utils::time::{TimeFrame, convert_time_frame};
use optionstratlib::volatility::volatility_for_dt;
use optionstratlib::Options;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use std::path::Path;
use tracing::info;

/// Walker implementation for the simulation.
struct Walker;

impl WalkTypeAble<Positive, Positive> for Walker {}

/// Main function that runs multiple Long Call simulations using the Simulate trait.
///
/// # Returns
///
/// A `Result` indicating success or an error if the simulation fails.
///
/// # Errors
///
/// - The strategy cannot be created
/// - The random walk simulation fails
/// - The simulate trait execution fails
/// - File I/O operations fail
fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    // Simulation parameters
    let n_simulations = 1000; // Number of simulations to run
    let n_steps = 10080; // 7 days in minutes
    let underlying_price = pos!(4011.95);
    let days = pos!(7.0);
    let implied_volatility = pos!(0.27); // 27% annual volatility
    let symbol = "GOLD".to_string();
    
    // For a Long Call with delta ~0.70, we need a strike slightly in-the-money
    // Delta 0.70 for a call means the strike is below current price
    // Approximate: strike = underlying * 0.98 for delta ~0.70
    let strike_price = pos!(3930.0); // Strike price for the long call (delta ~0.70)

    // First, calculate the premium for the option
    let temp_option = Options::new(
        OptionType::European,
        Side::Long,
        symbol.clone(),
        strike_price,
        ExpirationDate::Days(days),
        implied_volatility,
        Positive::ONE,
        underlying_price,
        dec!(0.0), // risk_free_rate
        OptionStyle::Call,
        pos!(0.0), // dividend_yield
        None,
    );
    let initial_premium = temp_option.calculate_price_black_scholes()?.abs();
    let premium_positive = Positive::new(initial_premium.to_f64().unwrap())?;

    // Create the long call strategy with the calculated premium
    let strategy = LongCall::new(
        symbol.clone(),
        strike_price,
        ExpirationDate::Days(days),
        implied_volatility,
        Positive::ONE,
        underlying_price,
        dec!(0.0),         // risk_free_rate
        pos!(0.0),         // dividend_yield
        premium_positive,  // premium paid
        pos!(0.0),         // open_fee
        pos!(0.0),         // close_fee
    );

    // Define exit policy: 100% profit OR expiration
    let exit_policy = ExitPolicy::Or(vec![
        ExitPolicy::ProfitPercent(dec!(0.5)), // 100% profit
        ExitPolicy::Expiration,                // Or let it expire
    ]);

    info!("========== LONG CALL SIMULATION (Using Simulate Trait) ==========");
    info!("Starting {} Long Call simulations...", n_simulations);
    info!("Underlying: {} @ ${}", symbol, underlying_price);
    info!("Strike: ${}", strike_price);
    info!("Expiration: {} days ({} steps)", days, n_steps);
    info!("Implied Volatility: {:.2}%", implied_volatility * 100.0);
    info!("Initial Premium Paid: ${:.2}", initial_premium);
    info!("Exit Policy: 100% profit OR expiration");
    info!("================================================================\n");

    // Create WalkParams for the Simulator
    let walker = Box::new(Walker);
    let dt = convert_time_frame(pos!(1.0) / days, &TimeFrame::Minute, &TimeFrame::Day);

    // Adjust volatility for the specific dt in the random walk
    let volatility_dt =
        volatility_for_dt(implied_volatility, dt, TimeFrame::Minute, TimeFrame::Day)?;

    // Custom walk parameters for varying volatility
    let walk_params = WalkParams {
        size: n_steps,
        init_step: Step {
            x: Xstep::new(Positive::ONE, TimeFrame::Minute, ExpirationDate::Days(days)),
            y: Ystep::new(0, underlying_price),
        },
        walk_type: WalkType::Custom {
            dt,
            drift: dec!(0.1), // Slight upward drift for long call
            volatility: volatility_dt,
            vov: pos!(0.02),         // Volatility of volatility (2%)
            vol_speed: pos!(0.5),    // Mean reversion speed
            vol_mean: volatility_dt, // Mean volatility level (same as initial)
        },
        walker,
    };

    // Create Simulator with all random walks at once
    info!(
        "Generating {} random walks with {} steps each...",
        n_simulations, n_steps
    );
    let simulator = Simulator::new(
        "Long Call Simulator".to_string(),
        n_simulations,
        &walk_params,
        generator_positive,
    );

    // Create progress bar
    let progress_bar = ProgressBar::new(n_simulations as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .expect("Failed to set progress bar template")
            .progress_chars("#>-"),
    );

    info!("Running simulations using Simulate trait...");
    progress_bar.set_message("Simulating...");

    // Use the Simulate trait to run all simulations
    let stats = strategy.simulate(&simulator, exit_policy)?;

    // Finish progress bar
    progress_bar.finish_with_message("Simulations completed!");

    // Print final statistics using the built-in formatting methods
    stats.print_summary();

    // Print individual simulation results
    stats.print_individual_results();

    // Save the simulator visualization
    let path = Path::new("Draws/Simulation/long_call_strategy_simulation.png");
    simulator.write_png(path)?;
    info!("Visualization saved to: {:?}", path);

    Ok(())
}