/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 29/1/25
******************************************************************************/
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::greeks::Greeks;
use optionstratlib::strategies::base::Optimizable;
use optionstratlib::strategies::{DeltaNeutrality, FindOptimalSide, ShortStrangle, Strategies};
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::utils::{Graph, GraphBackend};
use optionstratlib::{pos, ExpirationDate, Positive};
use rust_decimal::Decimal;
use tracing::{debug, info};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let mut option_chain =
        OptionChain::load_from_json("examples/Chains/DAX-30-jan-2025-21637.0.json")?;
    info!("Chain loaded");
    option_chain.update_deltas();
    println!("{}", &option_chain);

    let mut strategy = ShortStrangle::new(
        "SP500".to_string(),
        option_chain.underlying_price, // underlying_price
        Positive::ZERO,                // call_strike
        Positive::ZERO,                // put_strike
        ExpirationDate::Days(pos!(1.0)),
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

    strategy.best_area(
        &option_chain,
        FindOptimalSide::Range(pos!(21600.0), pos!(21700.0)),
    );
    debug!("Strategy:  {:#?}", strategy);
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
        (range / 2.0) / option_chain.underlying_price * 100.0
    );
    info!("Profit Area: {:.2}%", strategy.profit_area()?);

    info!("Delta:  {:#?}", strategy.calculate_net_delta());
    info!("Delta Neutral:  {}", strategy.is_delta_neutral());
    info!(
        "Delta Suggestions:  {:#?}",
        strategy.suggest_delta_adjustments()
    );

    if strategy.profit_ratio()? > Positive::ZERO.into() {
        debug!("Strategy:  {:#?}", strategy);
        strategy.graph(
            &price_range,
            GraphBackend::Bitmap {
                file_path: "Draws/Chains/short_strangle_chain_raw_delta.png",
                size: (1400, 933),
            },
            20,
        )?;
    }
    info!("Greeks:  {:#?}", strategy.greeks());

    Ok(())
}
