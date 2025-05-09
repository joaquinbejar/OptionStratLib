use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::pos;
use optionstratlib::strategies::BasicAble;
use optionstratlib::strategies::base::{Optimizable, Strategies};
use optionstratlib::strategies::long_butterfly_spread::LongButterflySpread;
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
    let mut strategy = LongButterflySpread::new(
        "SP500".to_string(),
        underlying_price,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        ExpirationDate::Days(pos!(5.0)),
        Positive::ZERO,
        Decimal::ZERO,
        Positive::ZERO,
        pos!(1.0),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        pos!(4.0),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    );
    strategy.get_best_area(&option_chain, FindOptimalSide::All);
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
    info!("Profit Area: {:.2}%", strategy.get_profit_area()?);

    if strategy.get_profit_ratio()? > Positive::ZERO.into() {
        debug!("Strategy:  {:#?}", strategy);
        let path: &std::path::Path =
            "Draws/Strategy/long_butterfly_spread_profit_loss_chart_best_area.png".as_ref();
        strategy.write_png(path)?;
    }

    Ok(())
}
