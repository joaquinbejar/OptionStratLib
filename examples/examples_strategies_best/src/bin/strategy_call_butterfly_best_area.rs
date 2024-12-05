/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/9/24
******************************************************************************/
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::constants::ZERO;
use optionstratlib::model::types::PositiveF64;
use optionstratlib::model::types::{ExpirationDate, PZERO};
use optionstratlib::pos;
use optionstratlib::strategies::base::Strategies;
use optionstratlib::strategies::call_butterfly::CallButterfly;
use optionstratlib::strategies::utils::FindOptimalSide;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::visualization::utils::Graph;
use std::env;
use std::error::Error;
use tracing::{debug, info};

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    let underlying_price = option_chain.underlying_price;
    let mut strategy = CallButterfly::new(
        "".to_string(),
        underlying_price, // underlying_price
        PZERO,            // long_strike_itm
        PZERO,            // long_strike_otm
        PZERO,            // short_strike
        ExpirationDate::Days(2.0),
        ZERO,      // implied_volatility
        0.05,      // risk_free_rate
        ZERO,      // dividend_yield
        pos!(2.0), // long quantity
        pos!(4.0), // short_quantity
        ZERO,      // premium_long_itm
        ZERO,      // premium_long_otm
        ZERO,      // premium_short
        0.78,      // open_fee_long
        0.78,      // close_fee_long
        0.73,      // close_fee_short
        0.73,      // close_fee_short
    );

    strategy.best_area(
        &option_chain,
        FindOptimalSide::Range(pos!(5700.0), pos!(6300.0)),
    );
    let price_range = strategy.best_range_to_show(pos!(1.0)).unwrap();
    let range = strategy.break_even_points[1] - strategy.break_even_points[0];
    info!("Title: {}", strategy.title());
    info!("Break Even Points: {:?}", strategy.break_even_points);
    info!(
        "Net Premium Received: ${:.2}",
        strategy.net_premium_received()
    );
    info!("Max Profit: ${:.2}", strategy.max_profit());
    info!("Max Loss: ${:0.2}", strategy.max_loss());
    info!("Total Fees: ${:.2}", strategy.fees());
    info!(
        "Range of Profit: ${:.2} {:.2}%",
        range,
        (range / 2.0) / underlying_price * 100.0
    );
    info!("Profit Area: {:.2}%", strategy.profit_area());
    debug!("Strategy:  {:#?}", strategy);

    strategy.graph(
        &price_range,
        "Draws/Strategy/call_butterfly_profit_loss_chart_best_area.png",
        20,
        (1400, 933),
    )?;

    Ok(())
}