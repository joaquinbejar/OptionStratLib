use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::greeks::Greeks;
use optionstratlib::pos;
use optionstratlib::strategies::BasicAble;
use optionstratlib::strategies::base::{Optimizable, Strategies};
use optionstratlib::strategies::iron_condor::IronCondor;
use optionstratlib::strategies::utils::FindOptimalSide;
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::Graph;
use rust_decimal::Decimal;
use std::error::Error;
use tracing::{debug, info};

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    let underlying_price = option_chain.underlying_price;

    let mut strategy = IronCondor::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        Positive::ZERO,   // short_call_strike
        Positive::ZERO,   // short_put_strike
        Positive::ZERO,   // long_call_strike
        Positive::ZERO,   // long_put_strike
        ExpirationDate::Days(pos!(5.0)),
        Positive::ZERO, // implied_volatility
        Decimal::ZERO,  // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(1.0),      // quantity
        Positive::ZERO, // premium_short_call
        Positive::ZERO, // premium_short_put
        Positive::ZERO, // premium_long_call
        Positive::ZERO, // premium_long_put
        Positive::ONE,  // open_fee
        Positive::ONE,  // close_fee
    );

    strategy.get_best_ratio(&option_chain, FindOptimalSide::All);
    debug!("Option Chain: {}", option_chain);
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
        (range / 2.0) / underlying_price * 100.0
    );
    info!("Profit Ratio: {:.2}%", strategy.get_profit_ratio()?);

    if strategy.get_profit_ratio()? > Positive::ZERO.into() {
        debug!("Strategy:  {:#?}", strategy);
        let path: &std::path::Path =
            "Draws/Strategy/iron_condor_profit_loss_chart_best_ratio.png".as_ref();
        strategy.write_png(path)?;
    }
    info!("Greeks:  {:#?}", strategy.greeks());
    Ok(())
}
