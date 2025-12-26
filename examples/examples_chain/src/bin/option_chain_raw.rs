use positive::pos_or_panic;
/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 29/1/25
******************************************************************************/
use chrono::DateTime;
use optionstratlib::prelude::*;
use rust_decimal::Decimal;
use tracing::{debug, info};

fn main() -> Result<(), optionstratlib::error::Error> {
    setup_logger();
    let mut option_chain =
        OptionChain::load_from_json("examples/Chains/DAX-30-jan-2025-21637.0.json")?;
    info!("Chain loaded");
    option_chain.update_greeks();
    info!("{}", &option_chain);

    let datetime = DateTime::parse_from_rfc3339("2025-01-30T18:30:00Z").unwrap();

    let mut strategy = ShortStrangle::new(
        "SP500".to_string(),
        option_chain.underlying_price, // underlying_price
        Positive::ZERO,                // call_strike
        Positive::ZERO,                // put_strike
        ExpirationDate::DateTime(DateTime::from(datetime)),
        Positive::ONE,      // implied_volatility
        Positive::ONE,      // implied_volatility
        Decimal::ZERO,      // risk_free_rate
        Positive::ZERO,     // dividend_yield
        Positive::ONE,      // quantity
        Positive::ZERO,     // premium_short_call
        Positive::ZERO,     // premium_short_put
        pos_or_panic!(2.2), // open_fee_short_call
        pos_or_panic!(2.2), // close_fee_short_call
        pos_or_panic!(1.7), // open_fee_short_put
        pos_or_panic!(1.7), // close_fee_short_put
    );

    // strategy.best_area(&option_chain, FindOptimalSide::Range(pos_or_panic!(21600.0), pos_or_panic!(21700.0) ));
    strategy.get_best_area(&option_chain, FindOptimalSide::Upper);
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

    if strategy.get_profit_ratio()? > Positive::ZERO.into() {
        debug!("Strategy:  {:#?}", strategy);
        let file_path = "Draws/Chains/short_strangle_chain_raw_best_area.png";
        let path: &std::path::Path = file_path.as_ref();
        strategy.write_png(path)?;
    }
    info!("Greeks:  {:#?}", strategy.greeks());

    Ok(())
}
