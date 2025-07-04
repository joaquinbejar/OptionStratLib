/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/9/24
******************************************************************************/

use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::greeks::Greeks;
use optionstratlib::pos;
use optionstratlib::strategies::long_butterfly_spread::LongButterflySpread;
use optionstratlib::strategies::{BasicAble, Strategies};
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::Graph;
use rust_decimal_macros::dec;
use std::error::Error;
use tracing::info;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let underlying_price = pos!(5795.88);

    let strategy = LongButterflySpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos!(5710.0),     // long_strike_itm
        pos!(5780.0),     // short_strike
        pos!(5850.0),     // long_strike_otm
        ExpirationDate::Days(pos!(2.0)),
        pos!(0.18),     // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(2.1),      // long quantity
        pos!(113.3),    // premium_long_low
        pos!(64.20),    // premium_short
        pos!(31.65),    // premium_long_high
        pos!(0.05),     // open_fee_short_call
        pos!(0.05),     // close_fee_short_call
        pos!(0.05),     // open_fee_long_call_low
        pos!(0.05),     // close_fee_long_call_low
        pos!(0.05),     // open_fee_long_call_high
        pos!(0.05),     // close_fee_long_call_high
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

    Ok(())
}
