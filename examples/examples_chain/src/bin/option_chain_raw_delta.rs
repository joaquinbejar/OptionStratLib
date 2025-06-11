/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 29/1/25
******************************************************************************/
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::greeks::Greeks;
use optionstratlib::strategies::base::Optimizable;
use optionstratlib::strategies::{
    BasicAble, DeltaNeutrality, FindOptimalSide, ShortStrangle, Strategies,
};
use optionstratlib::utils::setup_logger;
use optionstratlib::utils::time::get_tomorrow_formatted;

use optionstratlib::visualization::Graph;
use optionstratlib::{ExpirationDate, Positive, pos};
use rust_decimal::Decimal;
use tracing::{debug, info};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let mut option_chain =
        OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    info!("Chain loaded");
    option_chain.update_expiration_date(get_tomorrow_formatted());
    option_chain.update_greeks();
    info!("{}", &option_chain);

    let mut strategy = ShortStrangle::new(
        "SP500".to_string(),
        option_chain.underlying_price, // underlying_price
        Positive::ZERO,                // call_strike
        Positive::ZERO,                // put_strike
        ExpirationDate::Days(pos!(1.0)),
        Positive::ZERO, // implied_volatility
        Positive::ZERO, // implied_volatility
        Decimal::ZERO,  // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(1.0),      // quantity
        Positive::ZERO, // premium_short_call
        Positive::ZERO, // premium_short_put
        pos!(2.2),      // open_fee_short_call
        pos!(2.2),      // close_fee_short_call
        pos!(1.7),      // open_fee_short_put
        pos!(1.7),      // close_fee_short_put
    );

    strategy.get_best_area(
        &option_chain,
        FindOptimalSide::Range(pos!(21600.0), pos!(21700.0)),
    );
    debug!("Strategy:  {:#?}", strategy);
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
        (range / 2.0) / option_chain.underlying_price * 100.0
    );
    info!("Profit Area: {:.2}%", strategy.get_profit_area()?);

    info!("Delta:  {:#?}", strategy.delta_neutrality()?);
    info!("Delta Neutral:  {}", strategy.is_delta_neutral());
    info!("Delta Suggestions:  {:#?}", strategy.delta_adjustments()?);

    if strategy.get_profit_ratio()? > Positive::ZERO.into() {
        debug!("Strategy:  {:#?}", strategy);
        let path: &std::path::Path = "Draws/Chains/short_strangle_chain_raw_delta.png".as_ref();
        strategy.write_png(path)?;
    }
    info!("Greeks:  {:#?}", strategy.greeks());

    Ok(())
}
