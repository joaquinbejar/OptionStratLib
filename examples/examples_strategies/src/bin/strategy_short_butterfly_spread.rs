use optionstratlib::prelude::*;
use rust_decimal_macros::dec;
use std::error::Error;
use tracing::info;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let underlying_price = pos!(5781.88);

    let strategy = ShortButterflySpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos!(5700.0),     // short_strike_itm
        pos!(5780.0),     // long_strike
        pos!(5850.0),     // short_strike_otm
        ExpirationDate::Days(pos!(2.0)),
        pos!(0.18),     // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(1.1),      // long quantity
        pos!(119.01),   // premium_long
        pos!(66.0),     // premium_short
        pos!(29.85),    // open_fee_long
        pos!(0.05),
        pos!(0.05),
        pos!(0.05),
        pos!(0.05),
        pos!(0.05),
        pos!(0.05),
    );

    info!("Title: {}", strategy.get_title());
    info!("Break Even Points: {:?}", strategy.break_even_points);
    info!(
        "Net Premium Received: ${:.2}",
        strategy.get_net_premium_received()?
    );
    info!(
        "Max Profit: ${:.2}",
        strategy.get_max_profit().unwrap_or(Positive::ZERO)
    );
    info!(
        "Max Loss: ${:0.2}",
        strategy.get_max_loss().unwrap_or(Positive::ZERO)
    );
    info!("Total Fees: ${:.2}", strategy.get_fees()?);
    info!("Profit Area: {:.2}%", strategy.get_profit_area()?);
    info!("Profit Ratio: {:.2}%", strategy.get_profit_ratio()?);

    let path: &std::path::Path =
        "Draws/Strategy/short_butterfly_spread_profit_loss_chart.png".as_ref();
    strategy.write_png(path)?;

    let prob = strategy.probability_of_profit(None, None)?;
    info!("Probability of Profit: {:.2}%", prob);

    let prob = strategy.probability_of_loss(None, None)?;
    info!("Probability of Loss: {:.2}%", prob);

    Ok(())
}
