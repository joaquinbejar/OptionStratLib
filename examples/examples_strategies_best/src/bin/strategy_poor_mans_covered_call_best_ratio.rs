use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::pos;
use optionstratlib::strategies::BasicAble;
use optionstratlib::strategies::base::{Optimizable, Strategies};
use optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall;
use optionstratlib::strategies::utils::FindOptimalSide;
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::utils::{Graph, GraphBackend};
use rust_decimal::Decimal;
use std::error::Error;
use tracing::{debug, info};

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    let underlying_price = option_chain.underlying_price;

    let mut strategy = PoorMansCoveredCall::new(
        "SP500".to_string(),               // underlying_symbol
        underlying_price,                  // underlying_price
        Positive::ZERO,                    // long_call_strike
        Positive::ZERO,                    // short_call_strike OTM
        ExpirationDate::Days(pos!(120.0)), // long_call_expiration
        ExpirationDate::Days(pos!(30.0)),  // short_call_expiration 30-45 days delta 0.30 or less
        Positive::ZERO,                    // implied_volatility
        Decimal::ZERO,                     // risk_free_rate
        Positive::ZERO,                    // dividend_yield
        pos!(2.0),                         // quantity
        Positive::ZERO,                    // premium_short_call
        Positive::ZERO,                    // premium_short_put
        pos!(1.74),                        // open_fee_short_call
        pos!(1.74),                        // close_fee_short_call
        pos!(0.85),                        // open_fee_short_put
        pos!(0.85),                        // close_fee_short_put
    );

    strategy.get_best_ratio(&option_chain, FindOptimalSide::Upper);
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
        strategy.graph(
            GraphBackend::Bitmap {
                file_path: "Draws/Strategy/poor_mans_covered_call_profit_loss_chart_best_ratio.png",
                size: (1400, 933),
            },
            20,
        )?;
    }

    Ok(())
}
