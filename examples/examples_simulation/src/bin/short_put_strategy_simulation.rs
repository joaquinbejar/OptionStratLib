/******************************************************************************
use optionstratlib::error::Error;
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 8/11/25
******************************************************************************/
//! Short Put Strategy Simulation using Simulate Trait
//!
//! This simulation evaluates the performance of a Short Put options strategy
//! across multiple random walk scenarios using the `Simulate` trait implementation.
//!
//! # Strategy Overview
//!
//! A Short Put involves selling a put option to collect premium upfront.
//! The strategy is profitable when:
//! - The underlying price stays above the strike price
//! - The premium value decreases over time (theta decay)
//!
//! # Exit Conditions
//!
//! The simulation closes positions when:
//! - **Profit Target**: Premium drops by 50% (we can buy back at 50% of initial cost)
//! - **Loss Limit**: Premium increases by 100% (premium doubles - cut losses)
//! - **Expiration**: The option reaches its expiration date
//!
//! # Running the Simulation
//!
//! ```bash
//! cargo run --package examples_simulation --bin short_put_strategy_simulation
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
//! - PNG visualization of the last simulation in `Draws/Simulation/short_put_strategy_simulation.png`

use optionstratlib::prelude::*;

/// Walker implementation for the simulation.
struct Walker;

impl WalkTypeAble<Positive, Positive> for Walker {}

/// Main function that runs multiple Short Put simulations using the Simulate trait.
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
fn main() -> Result<(), Error> {
    setup_logger();

    // Simulation parameters
    let n_simulations = 10000; // Number of simulations to run
    let n_steps = 10080; // 7 days in minutes
    let underlying_price = pos!(4007.7);
    let days = pos!(7.0);
    let implied_volatility = pos!(0.16); // 27% annual volatility
    let symbol = "GOLD".to_string();
    let strike_price = pos!(3930.0); // Strike price for the short put (delta ~-0.3)

    // First, calculate the premium for the option
    let temp_option = Options::new(
        OptionType::European,
        Side::Short,
        symbol.clone(),
        strike_price,
        ExpirationDate::Days(days),
        implied_volatility,
        Positive::ONE,
        underlying_price,
        dec!(0.0), // risk_free_rate
        OptionStyle::Put,
        pos!(0.0), // dividend_yield
        None,
    );
    let initial_premium = temp_option.calculate_price_black_scholes()?.abs();
    let premium_positive = Positive::new(initial_premium.to_f64().unwrap())?;

    // Create the short put strategy with the calculated premium
    let strategy = ShortPut::new(
        symbol.clone(),
        strike_price,
        ExpirationDate::Days(days),
        implied_volatility,
        Positive::ONE,
        underlying_price,
        dec!(0.0),        // risk_free_rate
        pos!(0.0),        // dividend_yield
        premium_positive, // premium received
        pos!(0.0),        // open_fee
        pos!(0.0),        // close_fee
    );

    // Define exit policy: 50% profit OR 100% loss
    let exit_policy = ExitPolicy::profit_or_loss(dec!(0.5), dec!(1.0));

    info!("========== SHORT PUT SIMULATION (Using Simulate Trait) ==========");
    info!("Starting {} Short Put simulations...", n_simulations);
    info!("Underlying: {} @ ${}", symbol, underlying_price);
    info!("Strike: ${}", strike_price);
    info!("Expiration: {} days ({} steps)", days, n_steps);
    info!("Implied Volatility: {:.2}%", implied_volatility * 100.0);
    info!("Initial Premium: ${:.2}", initial_premium);
    info!("Exit Policy: {}", exit_policy);
    info!("=================================================================");

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
            drift: dec!(-0.1),
            volatility: volatility_dt,
            vov: pos!(0.02),         // Volatility of volatility (2%)
            vol_speed: pos!(0.2),    // Mean reversion speed
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
        "Short Put Simulator".to_string(),
        n_simulations,
        &walk_params,
        generator_positive,
    );

    info!("Running simulations using Simulate trait...");

    // Use the Simulate trait to run all simulations
    // (Progress bar is now handled inside the simulate method)
    let stats = strategy.simulate(&simulator, exit_policy)?;

    // Print final statistics using the built-in formatting methods
    stats.print_summary();

    // Print individual simulation results
    stats.print_individual_results();

    // Save the simulator visualization
    let path: &std::path::Path = "Draws/Simulation/short_put_strategy_simulation.png".as_ref();
    simulator.write_png(path)?;
    info!("Visualization saved to: {:?}", path);

    Ok(())
}
