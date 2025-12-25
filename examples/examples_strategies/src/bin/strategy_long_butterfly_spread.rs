/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/9/24
******************************************************************************/

use optionstratlib::prelude::*;
use positive::pos_or_panic;

fn main() -> Result<(), Error> {
    setup_logger();
    let underlying_price = pos_or_panic!(23762.0);

    let strategy = LongButterflySpread::new(
        "DAX".to_string(),
        underlying_price,       // underlying_price
        pos_or_panic!(23600.0), // long_strike_itm
        pos_or_panic!(23750.0), // short_strike
        pos_or_panic!(23900.0), // long_strike_otm
        ExpirationDate::Days(pos_or_panic!(63.0)),
        pos_or_panic!(0.14),  // implied_volatility
        dec!(0.0),            // risk_free_rate
        Positive::ZERO,       // dividend_yield
        Positive::ONE,   // long quantity
        pos_or_panic!(645.3), // premium_long_low
        pos_or_panic!(545.6), // premium_short
        pos_or_panic!(477.1), // premium_long_high
        pos_or_panic!(0.05),  // open_fee_short_call
        pos_or_panic!(0.05),  // close_fee_short_call
        pos_or_panic!(0.05),  // open_fee_long_call_low
        pos_or_panic!(0.05),  // close_fee_long_call_low
        pos_or_panic!(0.05),  // open_fee_long_call_high
        pos_or_panic!(0.05),  // close_fee_long_call_high
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
        "Draws/Strategy/long_butterfly_spread_profit_loss_chart.png".as_ref();
    strategy.write_png(path)?;

    info!("Greeks:  {:#?}", strategy.greeks());

    let prob = strategy.probability_of_profit(None, None)?;
    info!("Probability of Profit: {:.2}%", prob);

    let prob = strategy.probability_of_loss(None, None)?;
    info!("Probability of Loss: {:.2}%", prob);

    Ok(())
}
