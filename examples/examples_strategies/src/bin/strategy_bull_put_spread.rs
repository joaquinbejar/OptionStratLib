/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/9/24
******************************************************************************/
use optionstratlib::f2p;
use optionstratlib::strategies::bull_put_spread::BullPutSpread;
use optionstratlib::strategies::Strategies;
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::utils::Graph;
use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use std::error::Error;
use tracing::info;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let underlying_price = f2p!(5781.88);

    let strategy = BullPutSpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        f2p!(5750.0),     // long_strike_itm
        f2p!(5920.0),     // short_strike
        ExpirationDate::Days(2.0),
        0.18,      // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        f2p!(2.0), // long quantity
        15.04,     // premium_long
        89.85,     // premium_short
        0.78,      // open_fee_long
        0.78,      // open_fee_long
        0.73,      // close_fee_long
        0.73,      // close_fee_short
    );

    let price_range = strategy.best_range_to_show(f2p!(1.0)).unwrap();

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
    info!("Profit Area: {:.2}%", strategy.profit_area()?);
    info!("Profit Ratio: {:.2}%", strategy.profit_ratio()?);

    // Generate the profit/loss graph
    strategy.graph(
        &price_range,
        "Draws/Strategy/bull_put_spread_profit_loss_chart.png",
        20,
        (1400, 933),
    )?;

    Ok(())
}
