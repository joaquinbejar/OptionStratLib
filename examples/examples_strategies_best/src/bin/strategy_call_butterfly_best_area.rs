/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/9/24
******************************************************************************/
use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::pos;
use optionstratlib::strategies::BasicAble;
use optionstratlib::strategies::base::{Optimizable, Strategies};
use optionstratlib::strategies::call_butterfly::CallButterfly;
use optionstratlib::strategies::utils::FindOptimalSide;
use optionstratlib::utils::setup_logger;

use rust_decimal_macros::dec;
use std::error::Error;
use tracing::{debug, info};
use optionstratlib::visualization::Graph;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    let underlying_price = option_chain.underlying_price;
    let mut strategy = CallButterfly::new(
        "".to_string(),
        underlying_price, // underlying_price
        Positive::ZERO,   // long_strike_itm
        Positive::ZERO,   // long_strike_otm
        Positive::ZERO,   // short_strike
        ExpirationDate::Days(pos!(2.0)),
        Positive::ZERO, // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(1.0),      // long quantity
        Positive::ZERO, // short_quantity
        Positive::ZERO, // premium_long_itm
        Positive::ZERO, // premium_long_otm
        pos!(0.95),     //    open_fee_long
        pos!(0.95),     //    close_fee_long
        pos!(0.95),     //    open_fee_short_low
        pos!(0.95),     //    close_fee_short_low
        pos!(0.95),     //    open_fee_short_high
        pos!(0.95),     //    close_fee_short_high
    );

    strategy.get_best_area(&option_chain, FindOptimalSide::Center);
    let range = strategy.get_range_of_profit().unwrap_or(Positive::ZERO);
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
    info!(
        "Range of Profit: ${:.2} {:.2}%",
        range,
        (range / 2.0) / underlying_price * 100.0
    );
    info!("Profit Area: {:.2}%", strategy.get_profit_area()?);
    debug!("Strategy:  {:#?}", strategy);
    let path: &std::path::Path = "Draws/Strategy/call_butterfly_profit_loss_chart_best_area.png".as_ref();
    strategy.write_png(path, 1200, 800)?;


    Ok(())
}
