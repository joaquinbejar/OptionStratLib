/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/9/24
******************************************************************************/

use optionstratlib::pos;
use optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall;
use optionstratlib::strategies::Strategies;
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::utils::Graph;
use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use rust_decimal_macros::dec;
use std::error::Error;
use tracing::info;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let underlying_price = pos!(2703.3);

    let strategy = PoorMansCoveredCall::new(
        "GOLD".to_string(),   // underlying_symbol
        underlying_price,   // underlying_price
        pos!(2600.0),   // long_call_strike
        pos!(2800.0),   // short_call_strike OTM
        ExpirationDate::Days(pos!(120.0)),   // long_call_expiration
        ExpirationDate::Days(pos!(30.0)),   // short_call_expiration 30-45 days delta 0.30 or less
        pos!(0.17),   // implied_volatility
        dec!(0.05),   // risk_free_rate
        Positive::ZERO,   // dividend_yield
        pos!(3.0),   // quantity
        154.7,   // premium_short_call
        30.8,   // premium_short_put
        1.74,   // open_fee_short_call
        1.74,   // close_fee_short_call
        0.85,   // open_fee_short_put
        0.85,   // close_fee_short_put
    );

    let price_range = strategy.best_range_to_show(pos!(1.0)).unwrap();

    info!("Title: {}", strategy.title());
    info!("Break Even Points: {:?}", strategy.break_even_points);
    info!(
        "Max Profit: ${:.2}",
        strategy.max_profit().unwrap_or(Positive::ZERO)
    );
    info!(
        "Max Loss: ${}",
        strategy.max_loss().unwrap_or(Positive::ZERO)
    );
    info!("Total Fees: ${:.2}", strategy.fees()?);
    info!("Profit Area: {:.2}%", strategy.profit_area()?);
    info!("Profit Ratio: {:.2}%", strategy.profit_ratio()?);

    // Generate the profit/loss graph
    strategy.graph(
        &price_range,
        "Draws/Strategy/poor_mans_covered_call_profit_loss_chart.png",
        20,
        (1400, 933),
    )?;

    Ok(())
}
