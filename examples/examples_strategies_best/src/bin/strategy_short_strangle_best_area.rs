use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::greeks::Greeks;
use optionstratlib::pos;
use optionstratlib::strategies::{DeltaNeutrality, ShortStrangle};
use optionstratlib::strategies::base::{Optimizable, Strategies};
use optionstratlib::strategies::utils::FindOptimalSide;
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::utils::{Graph, GraphBackend};
use rust_decimal::Decimal;
use std::error::Error;
use rust_decimal_macros::dec;
use tracing::{debug, info};
use optionstratlib::utils::time::get_tomorrow_formatted;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let mut option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    option_chain.update_expiration_date(get_tomorrow_formatted());
    info!("Option Chain:  {}", option_chain);
    let underlying_price = option_chain.underlying_price;
    let mut strategy = ShortStrangle::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        Positive::ZERO,   // call_strike
        Positive::ZERO,   // put_strike
        ExpirationDate::Days(pos!(5.0)),
        Positive::ZERO, // implied_volatility
        Decimal::ZERO,  // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(1.0),      // quantity
        Positive::ZERO, // premium_short_call
        Positive::ZERO, // premium_short_put
        pos!(0.82),     // open_fee_short_call
        pos!(0.82),     // close_fee_short_call
        pos!(0.82),     // open_fee_short_put
        pos!(0.82),     // close_fee_short_put
    );
    // strategy.best_area(&option_chain, FindOptimalSide::Range(pos!(5700.0), pos!(6100.0)));
    strategy.best_area(&option_chain, FindOptimalSide::DeltaRange(dec!(-0.3), dec!(0.3)));
    // strategy.best_area(&option_chain, FindOptimalSide::Center);
    debug!("Strategy:  {:#?}", strategy);
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
        (range / 2.0) / underlying_price * 100.0
    );
    info!("Profit Area: {:.2}%", strategy.profit_area()?);
    info!("Delta:  {:#?}", strategy.delta_neutrality()?);
    if strategy.profit_ratio()? > Positive::ZERO.into() {
        debug!("Strategy:  {:#?}", strategy);
        strategy.graph(
            GraphBackend::Bitmap {
                file_path: "Draws/Strategy/short_strangle_profit_loss_chart_best_area.png",
                size: (1400, 933),
            },
            20,
        )?;
    }
    info!("Greeks:  {:#?}", strategy.greeks());

    Ok(())
}
