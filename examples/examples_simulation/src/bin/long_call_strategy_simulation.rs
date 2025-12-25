/******************************************************************************
use optionstratlib::error::Error;
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

use optionstratlib::prelude::*;
use positive::pos_or_panic;

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
fn main() -> Result<(), Error> {
    setup_logger();

    // Simulation parameters
    let n_simulations = 100; // Number of simulations to run
    let n_steps = 10080; // 7 days in minutes
    let underlying_price = pos_or_panic!(4088.85);
    let days = pos_or_panic!(7.0);
    let implied_volatility = pos_or_panic!(0.24); // 27% annual volatility
    let symbol = "GOLD".to_string();

    // For a Long Call with delta ~0.70, we need a strike slightly in-the-money
    // Delta 0.70 for a call means the strike is below current price
    // Approximate: strike = underlying * 0.98 for delta ~0.70
    let strike_price = pos_or_panic!(4150.0); // Strike price for the long call (delta ~0.30)

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
        Positive::ZERO, // dividend_yield
        None,
    );
    let initial_premium = temp_option.calculate_price_black_scholes()?.abs();
    let premium_positive = Positive::new_decimal(initial_premium)?;

    // Create the long call strategy with the calculated premium
    let strategy = LongCall::new(
        symbol.clone(),
        strike_price,
        ExpirationDate::Days(days),
        implied_volatility,
        Positive::ONE,
        underlying_price,
        dec!(0.0),          // risk_free_rate
        Positive::ZERO, // dividend_yield
        premium_positive,   // premium paid
        Positive::ZERO, // open_fee
        Positive::ZERO, // close_fee
    );

    // Define exit policy: 100% profit OR expiration
    let exit_policy = ExitPolicy::Or(vec![
        ExitPolicy::ProfitPercent(dec!(0.5)), // 50% profit (premium doubles)
        ExitPolicy::Expiration,               // Or let it expire
    ]);

    info!("========== LONG CALL SIMULATION (Using Simulate Trait) ==========");
    info!("Starting {} Long Call simulations...", n_simulations);
    info!("Underlying: {} @ ${}", symbol, underlying_price);
    info!("Strike: ${}", strike_price);
    info!("Expiration: {} days ({} steps)", days, n_steps);
    info!("Implied Volatility: {:.2}%", implied_volatility * 100.0);
    info!("Initial Premium Paid: ${:.2}", initial_premium);
    info!("Exit Policy: 50% profit OR expiration");
    info!("================================================================");

    // Create WalkParams for the Simulator
    let walker = Box::new(Walker);
    let dt = convert_time_frame(
        Positive::ONE / days,
        &TimeFrame::Minute,
        &TimeFrame::Day,
    );

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
        walk_type: WalkType::Heston {
            dt,
            drift: dec!(0.01),
            volatility: volatility_dt,
            kappa: Positive::TWO,
            theta: pos_or_panic!(0.0225),
            xi: pos_or_panic!(0.3),
            rho: dec!(-0.3),
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

    info!("Running simulations using Simulate trait...");

    // Use the Simulate trait to run all simulations
    // (Progress bar is now handled inside the simulate method)
    let stats = strategy.simulate(&simulator, exit_policy)?;

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
