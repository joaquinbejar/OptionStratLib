use optionstratlib::chains::chain::OptionChain;
use optionstratlib::constants::ZERO;
use optionstratlib::pos;
use optionstratlib::strategies::base::{Optimizable, Strategies};
use optionstratlib::strategies::bear_put_spread::BearPutSpread;
use optionstratlib::strategies::utils::FindOptimalSide;
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::utils::Graph;
use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use std::error::Error;
use rust_decimal::Decimal;
use tracing::{debug, info};

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    let underlying_price = option_chain.underlying_price;
    let mut strategy = BearPutSpread::new(
        "SP500".to_string(),
        underlying_price,   // underlying_price
        Positive::ZERO,   // short_strike
        Positive::ZERO,   // long_strike
        ExpirationDate::Days(5.0),
        Positive::ZERO,   // implied_volatility
        Decimal::ZERO,   // risk_free_rate
        Positive::ZERO,   // dividend_yield
        pos!(1.0),   // quantity
        ZERO,   // premium_short_put
        ZERO,   // premium_short_put
        0.82,   // open_fee_short_put
        0.82,   // close_fee_short_put
        0.82,   // open_fee_short_put
        0.82,   // close_fee_short_put
    );
    strategy.best_area(&option_chain, FindOptimalSide::All);
    debug!("Option Chain: {}", option_chain);
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
        (range / 2.0) / underlying_price * 100.0
    );
    info!("Profit Area: {:.2}%", strategy.profit_area()?);

    if strategy.profit_ratio()? > Positive::ZERO.into() {
        debug!("Strategy:  {:#?}", strategy);
        strategy.graph(
            &price_range,
            "Draws/Strategy/bear_put_spread_profit_loss_chart_best_area.png",
            20,
            (1400, 933),
        )?;
    }

    Ok(())
}
