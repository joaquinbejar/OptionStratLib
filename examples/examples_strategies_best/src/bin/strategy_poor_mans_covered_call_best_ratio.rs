use optionstratlib::chains::chain::OptionChain;
use optionstratlib::constants::ZERO;
use optionstratlib::f2p;
use optionstratlib::model::types::ExpirationDate;
use optionstratlib::strategies::base::{Optimizable, Strategies};
use optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall;
use optionstratlib::strategies::utils::FindOptimalSide;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::visualization::utils::Graph;
use optionstratlib::Positive;
use std::error::Error;
use tracing::{debug, info};

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    let underlying_price = option_chain.underlying_price;

    let mut strategy = PoorMansCoveredCall::new(
        "SP500".to_string(),         // underlying_symbol
        underlying_price,            // underlying_price
        Positive::ZERO,              // long_call_strike
        Positive::ZERO,              // short_call_strike OTM
        ExpirationDate::Days(120.0), // long_call_expiration
        ExpirationDate::Days(30.0),  // short_call_expiration 30-45 days delta 0.30 or less
        ZERO,                        // implied_volatility
        ZERO,                        // risk_free_rate
        ZERO,                        // dividend_yield
        f2p!(2.0),                   // quantity
        ZERO,                        // premium_short_call
        ZERO,                        // premium_short_put
        1.74,                        // open_fee_short_call
        1.74,                        // close_fee_short_call
        0.85,                        // open_fee_short_put
        0.85,                        // close_fee_short_put
    );

    strategy.best_ratio(&option_chain, FindOptimalSide::Upper);
    debug!("Option Chain: {}", option_chain);
    debug!("Strategy:  {:#?}", strategy);
    let price_range = strategy.best_range_to_show(f2p!(1.0)).unwrap();
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
    info!("Profit Ratio: {:.2}%", strategy.profit_ratio()?);

    if strategy.profit_ratio()? > Positive::ZERO.into() {
        debug!("Strategy:  {:#?}", strategy);
        strategy.graph(
            &price_range,
            "Draws/Strategy/poor_mans_covered_call_profit_loss_chart_best_ratio.png",
            20,
            (1400, 933),
        )?;
    }

    Ok(())
}
