/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/9/24
******************************************************************************/
use optionstratlib::constants::ZERO;
use optionstratlib::model::chain::OptionChain;
use optionstratlib::model::types::ExpirationDate;
use optionstratlib::strategies::base::Strategies;
use optionstratlib::strategies::ratio_call_spread::RatioCallSpread;
use optionstratlib::strategies::utils::FindOptimalSide;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::visualization::utils::Graph;
use std::error::Error;
use tracing::{debug, info};

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    let underlying_price = option_chain.underlying_price;
    let mut strategy = RatioCallSpread::new(
        "".to_string(),
        underlying_price, // underlying_price
        ZERO,             // long_strike_itm
        ZERO,             // long_strike_otm
        ZERO,             // short_strike
        ExpirationDate::Days(2.0),
        ZERO, // implied_volatility
        0.05, // risk_free_rate
        ZERO, // dividend_yield
        1,    // long quantity
        2,    // short_quantity
        ZERO, // premium_long_itm
        ZERO, // premium_long_otm
        ZERO, // premium_short
        0.78, // open_fee_long
        0.78, // close_fee_long
        0.73, // close_fee_short
        0.73, // close_fee_short
    );

    strategy.best_area(&option_chain, FindOptimalSide::Upper);
    let price_range = strategy.best_range_to_show(1.0).unwrap();
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
    info!("Profit Area: {:.2}%", strategy.area());
    debug!("Strategy:  {:#?}", strategy);

    strategy.graph(
        &price_range,
        "Draws/Strategy/ratio_call_spread_profit_loss_chart_best_area.png",
        20,
        (1400, 933),
        (10, 30),
        15,
    )?;

    Ok(())
}
