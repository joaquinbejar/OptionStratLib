/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/9/24
******************************************************************************/

use optionstratlib::model::types::ExpirationDate;
use optionstratlib::strategies::base::Strategies;
use optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::visualization::utils::Graph;
use std::error::Error;
use tracing::info;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let underlying_price = 2703.3;

    let strategy = PoorMansCoveredCall::new(
        "GOLD".to_string(),          // underlying_symbol
        underlying_price,            // underlying_price
        2600.0,                      // long_call_strike
        2800.0,                      // short_call_strike OTM
        ExpirationDate::Days(120.0), // long_call_expiration
        ExpirationDate::Days(30.0),  // short_call_expiration 30-45 days delta 0.30 or less
        0.17,                        // implied_volatility
        0.05,                        // risk_free_rate
        0.0,                         // dividend_yield
        2,                           // quantity
        154.7,                       // premium_short_call
        30.8,                        // premium_short_put
        1.74,                        // open_fee_short_call
        1.74,                        // close_fee_short_call
        0.85,                        // open_fee_short_put
        0.85,                        // close_fee_short_put
    );

    let price_range: Vec<f64> = (2530..=2930).map(|x| x as f64).collect();
    // let range = strategy.break_even_points[1] - strategy.break_even_points[0];

    info!("Title: {}", strategy.title());
    info!("Break Even Points: {:?}", strategy.break_even_points);
    info!("Max Profit: ${:.2}", strategy.max_profit());
    info!("Max Loss: ${}", strategy.max_loss());
    info!("Total Fees: ${:.2}", strategy.fees());

    // Generate the profit/loss graph
    strategy.graph(
        &price_range,
        "Draws/Strategy/poor_mans_covered_call_profit_loss_chart.png",
        20,
        (1400, 933),
    )?;

    Ok(())
}
