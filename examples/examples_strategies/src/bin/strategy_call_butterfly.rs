/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/9/24
******************************************************************************/
use optionstratlib::pos;
use optionstratlib::strategies::call_butterfly::CallButterfly;
use optionstratlib::strategies::Strategies;
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::utils::Graph;
use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use std::error::Error;
use tracing::info;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let underlying_price = pos!(5781.88);

    let strategy = CallButterfly::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos!(5750.0),     // long_call_strike
        pos!(5800.0),     // short_call_low_strike
        pos!(5850.0),     // short_call_high_strike
        ExpirationDate::Days(2.0),
        0.18,      // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        pos!(1.0), // long quantity
        85.04,     // premium_long_itm
        53.04,     // premium_long_otm
        28.85,     // premium_short
        0.78,      // premium_short
        0.78,      // open_fee_long
        0.78,      // close_fee_long
        0.73,      // close_fee_short
        0.73,      // close_fee_short
        0.72,      // open_fee_short
    );
    let price_range = strategy.best_range_to_show(pos!(1.0)).unwrap();
    let range = strategy.range_of_profit().unwrap_or(Positive::ZERO);

    info!("Title: {}", strategy.title());
    info!("Break Even Points: {:?}", strategy.break_even_points);
    info!(
        "Net Premium Received: ${:.2}",
        strategy.net_premium_received()?
    );
    info!(
        "Max Profit: ${:.2}",
        strategy.max_profit().unwrap_or(Positive::ZERO)
    );
    info!(
        "Max Loss: ${:0.2}",
        strategy.max_loss().unwrap_or(Positive::ZERO)
    );
    info!("Total Fees: ${:.2}", strategy.fees()?);
    info!(
        "Range of Profit: ${:.2} {:.2}%",
        range,
        (range / 2.0) / underlying_price * 100.0
    );
    info!("Profit Area: {:.2}%", strategy.profit_area()?);
    info!("Profit Ratio: {:.2}%", strategy.profit_ratio()?);

    // Generate the profit/loss graph
    strategy.graph(
        &price_range,
        "Draws/Strategy/call_butterfly_profit_loss_chart.png",
        20,
        (1400, 933),
    )?;

    Ok(())
}
