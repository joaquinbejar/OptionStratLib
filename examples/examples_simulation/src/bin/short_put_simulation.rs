//! Short Put Strategy Simulation
//!
//! This simulation evaluates the performance of a Short Put options strategy
//! across multiple random walk scenarios.
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
//! cargo run --package examples_simulation --bin short_put_simulation
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
//! - PNG visualization of the last simulation in `Draws/Simulation/short_put_simulation.png`

use indicatif::{ProgressBar, ProgressStyle};
use optionstratlib::prelude::*;
use std::collections::HashMap;

/// Walker implementation for the simulation.
#[warn(dead_code)]
struct Walker {}

impl Walker {
    /// Creates a new `Walker` instance.
    ///
    /// A Walker implementation for price simulations.
    fn new() -> Self {
        Walker {}
    }
}

impl WalkTypeAble<Positive, Positive> for Walker {}

/// Evaluates a Short Put strategy throughout a random walk simulation.
///
/// This function monitors the strategy's premium value at each step and closes
/// the position based on the provided exit policy.
///
/// # Parameters
///
/// * `simulation_count` - The simulation number for tracking
/// * `random_walk` - The random walk simulation containing price data
/// * `option` - The short put option to evaluate
/// * `initial_premium` - The initial premium received when selling the put in dollars
/// * `implied_volatility` - The implied volatility for pricing
/// * `exit_policy` - The exit policy defining when to close the position
///
/// # Returns
///
/// A `SimulationResult` containing all metrics for this simulation
fn evaluate_short_put_strategy(
    simulation_count: usize,
    random_walk: &RandomWalk<Positive, Positive>,
    option: &Options,
    initial_premium: Decimal,
    _implied_volatility: &Positive,
    exit_policy: &ExitPolicy,
) -> SimulationResult {
    let profit_target = initial_premium * dec!(0.5); // 50% profit (premium drops to 50%)
    let loss_limit = initial_premium * dec!(2.0); // 100% loss (premium doubles)

    debug!(
        "Evaluating strategy with {} steps in random walk",
        random_walk.len()
    );

    let mut max_premium = initial_premium;
    let mut min_premium = initial_premium;
    let mut premium_sum = initial_premium;
    let mut premium_count = 1;

    for (step_num, step) in random_walk.get_steps().iter().enumerate() {
        // Skip step 0 (initial position)
        if step_num == 0 {
            continue;
        }

        let days_left = match step.x.days_left() {
            Ok(days) => days,
            Err(_) => break, // Expiration reached
        };

        let market_price = step.get_value();

        // For a short put, we sold the option and want to buy it back
        // The current premium is what we'd pay to buy it back now
        // We need to calculate the current market value of the option
        let mut current_option = option.clone();
        current_option.underlying_price = *market_price;
        current_option.expiration_date = ExpirationDate::Days(days_left);

        let current_premium = match current_option.calculate_price_black_scholes() {
            Ok(price) => price.abs(),
            Err(e) => {
                error!(
                    "Warning: Failed to calculate option price at step {}: {}",
                    step_num, e
                );
                continue;
            }
        };

        // Debug: Print premium evolution for first simulation
        if simulation_count == 1 && step_num % 1000 == 0 {
            let pnl = initial_premium - current_premium;
            info!(
                "  Step {}: Premium = ${:.2}, Underlying = ${:.2}, Days left = {:.2}, P&L = ${:.2}",
                step_num, current_premium, market_price, days_left, pnl
            );
        }

        // Track premium statistics
        if current_premium > max_premium {
            max_premium = current_premium;
        }
        if current_premium < min_premium {
            min_premium = current_premium;
        }
        premium_sum += current_premium;
        premium_count += 1;

        // Check exit policy
        if let Some(exit_reason) = check_exit_policy(
            exit_policy,
            initial_premium,
            current_premium,
            step_num,
            days_left,
            *market_price,
            false, // is_long = false for Short Put
        ) {
            // Calculate P&L based on current premium
            let pnl = PnL {
                realized: Some(initial_premium - current_premium),
                unrealized: None,
                initial_costs: Default::default(),
                initial_income: Default::default(),
                date_time: Default::default(),
            };

            debug!(
                "Simulation {}: Exit at step {} - Strike: ${}, Underlying: ${}, Premium: ${}, P&L: ${}, Reason: {}",
                simulation_count,
                step_num,
                option.strike_price,
                market_price,
                current_premium,
                pnl,
                exit_reason
            );

            // Determine which exit condition was hit for backward compatibility
            let hit_take_profit = current_premium <= profit_target;
            let hit_stop_loss = current_premium >= loss_limit;

            return SimulationResult {
                simulation_count,
                risk_metrics: None,
                final_equity_percentiles: HashMap::new(),
                max_premium,
                min_premium,
                avg_premium: premium_sum / Decimal::from(premium_count),
                hit_take_profit,
                hit_stop_loss,
                expired: false,
                expiration_premium: None,
                pnl,
                holding_period: step_num,
                exit_reason,
            };
        }
    }

    // If we reach here, the trade expired
    // At expiration, the put is worth max(strike - underlying, 0)
    if let Some(last_step) = random_walk.last() {
        let market_price = last_step.get_value();
        let strike_price = option.strike_price;

        // Calculate intrinsic value at expiration
        // For a SHORT PUT: we sold it, so we received initial_premium
        // At expiration, if ITM (strike > underlying), we must pay (strike - underlying)
        // If OTM (strike <= underlying), it expires worthless and we keep the full premium
        let final_premium = if strike_price > *market_price {
            (strike_price - *market_price).to_dec()
        } else {
            dec!(0.0)
        };
        let pnl = PnL {
            realized: Some(initial_premium - final_premium),
            unrealized: None,
            initial_costs: Default::default(),
            initial_income: Default::default(),
            date_time: Default::default(),
        };

        // Check if expiration would have triggered exit policy
        // This handles cases where the intrinsic value at expiration exceeds stop loss
        let days_left = pos_or_panic!(0.0);
        if let Some(exit_reason) = check_exit_policy(
            exit_policy,
            initial_premium,
            final_premium,
            random_walk.len(),
            days_left,
            *market_price,
            false, // is_long = false for Short Put
        ) {
            let moneyness = if strike_price > *market_price {
                "ITM"
            } else {
                "OTM"
            };
            info!(
                "Simulation {}: Expired {} with Exit Policy - Strike: ${}, Final Underlying: ${}, Intrinsic Value: ${}, P&L: ${}, Exit: {}",
                simulation_count,
                moneyness,
                strike_price,
                market_price,
                final_premium,
                pnl,
                exit_reason
            );

            // Determine which exit condition was hit
            let hit_take_profit = final_premium <= profit_target;
            let hit_stop_loss = final_premium >= loss_limit;

            return SimulationResult {
                simulation_count,
                risk_metrics: None,
                final_equity_percentiles: HashMap::new(),
                max_premium,
                min_premium,
                avg_premium: premium_sum / Decimal::from(premium_count),
                hit_take_profit,
                hit_stop_loss,
                expired: false,
                expiration_premium: Some(final_premium),
                pnl,
                holding_period: random_walk.len(),
                exit_reason,
            };
        }

        // If no exit policy triggered, it truly expired
        let moneyness = if strike_price > *market_price {
            "ITM (loss)"
        } else {
            "OTM (profit)"
        };
        info!(
            "Simulation {}: Expired {} - Strike: ${}, Final Underlying: ${}, Intrinsic Value: ${}, P&L: ${}",
            simulation_count, moneyness, strike_price, market_price, final_premium, pnl
        );

        return SimulationResult {
            simulation_count,
            risk_metrics: None,
            final_equity_percentiles: HashMap::new(),
            max_premium,
            min_premium,
            avg_premium: premium_sum / Decimal::from(premium_count),
            hit_take_profit: false,
            hit_stop_loss: false,
            expired: true,
            expiration_premium: Some(final_premium),
            pnl,
            holding_period: random_walk.len(),
            exit_reason: ExitPolicy::Expiration,
        };
    }

    // Fallback if we can't determine final value
    SimulationResult {
        simulation_count,
        risk_metrics: None,
        final_equity_percentiles: HashMap::new(),
        max_premium,
        min_premium,
        avg_premium: premium_sum / Decimal::from(premium_count),
        hit_take_profit: false,
        hit_stop_loss: false,
        expired: true,
        expiration_premium: None,
        pnl: PnL {
            realized: None,
            unrealized: None,
            initial_costs: Default::default(),
            initial_income: Default::default(),
            date_time: Default::default(),
        },
        holding_period: random_walk.len(),
        exit_reason: ExitPolicy::Expiration,
    }
}

/// Main function that runs multiple Short Put simulations and collects statistics.
///
/// # Returns
///
/// A `Result` indicating success or an error if the simulation fails.
///
/// # Errors
///
/// - The option chain cannot be built
/// - The random walk simulation fails
/// - File I/O operations fail
fn main() -> Result<(), Error> {
    setup_logger();

    // Simulation parameters
    let n_simulations = 10; // Number of simulations to run
    let n_steps = 10080; // 7 days in minutes
    let underlying_price = pos_or_panic!(4011.95);
    let days = pos_or_panic!(7.0);
    let implied_volatility = pos_or_panic!(0.27); // 27% annual volatility
    let symbol = "GOLD".to_string();
    let strike_price = pos_or_panic!(3930.0); // Strike price for the short put (delta ~-0.3)

    // Create the short put option
    let option = Options::new(
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
        pos_or_panic!(0.0), // dividend_yield
        None,
    );

    let initial_premium = option.calculate_price_black_scholes()?.abs();
    // Define exit policy: 50% profit OR 100% loss
    let exit_policy = ExitPolicy::profit_or_loss(dec!(0.5), dec!(1.0));

    let mut stats = SimulationStats::new();

    info!("========== SHORT PUT SIMULATION ==========");
    info!("Starting {} Short Put simulations...", n_simulations);
    info!("Underlying: {} @ ${}", symbol, underlying_price);
    info!("Strike: ${}", strike_price);
    info!("Expiration: {} days ({} steps)", days, n_steps);
    info!("Implied Volatility: {:.2}%", implied_volatility * 100.0);
    info!("Initial Premium: ${:.2}", initial_premium);
    info!("Exit Policy: {}", exit_policy);
    info!("==========================================");

    // Create WalkParams for the Simulator
    let walker = Box::new(Walker::new());
    let dt = convert_time_frame(
        pos_or_panic!(1.0) / days,
        &TimeFrame::Minute,
        &TimeFrame::Day,
    );

    // Adjust volatility for the specific dt in the random walk
    let volatility_dt =
        volatility_for_dt(implied_volatility, dt, TimeFrame::Minute, TimeFrame::Day)?;

    // Custom walk parameters for varying volatility
    // vov (volatility of volatility): controls how much the volatility changes
    // vol_speed: speed of mean reversion (higher = faster return to vol_mean)
    // vol_mean: the long-term average volatility level
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
            vov: pos_or_panic!(0.02),      // Volatility of volatility (30%)
            vol_speed: pos_or_panic!(0.5), // Mean reversion speed
            vol_mean: volatility_dt,       // Mean volatility level (same as initial)
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

    // Iterate over all random walks and evaluate each one
    for (sim_num, random_walk) in simulator.into_iter().enumerate() {
        debug!(
            "Simulation {}: Short Put at strike {} with initial premium ${:.2}",
            sim_num + 1,
            strike_price,
            initial_premium
        );

        // Evaluate the strategy
        let result = evaluate_short_put_strategy(
            sim_num + 1,
            random_walk,
            &option,
            initial_premium,
            &implied_volatility,
            &exit_policy,
        );

        // Update statistics
        stats.update(result);

        // Update progress bar
        progress_bar.inc(1);
    }

    // Finish progress bar
    progress_bar.finish_with_message("Simulations completed!");

    // Print final statistics
    stats.print_summary();

    // Print individual simulation results
    stats.print_individual_results();

    // Save the simulator visualization
    let path: &std::path::Path = "Draws/Simulation/short_put_simulation.png".as_ref();
    simulator.write_png(path)?;
    info!("Visualization saved to: {:?}", path);

    Ok(())
}
