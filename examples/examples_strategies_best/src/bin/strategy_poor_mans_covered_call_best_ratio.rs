use optionstratlib::prelude::*;
use positive::pos_or_panic;

fn main() -> Result<(), Error> {
    setup_logger();
    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    let underlying_price = option_chain.underlying_price;

    let mut strategy = PoorMansCoveredCall::new(
        "SP500".to_string(),                        // underlying_symbol
        underlying_price,                           // underlying_price
        Positive::ZERO,                             // long_call_strike
        Positive::ZERO,                             // short_call_strike OTM
        ExpirationDate::Days(pos_or_panic!(120.0)), // long_call_expiration
        ExpirationDate::Days(pos_or_panic!(30.0)), // short_call_expiration 30-45 days delta 0.30 or less
        Positive::ZERO,                            // implied_volatility
        Decimal::ZERO,                             // risk_free_rate
        Positive::ZERO,                            // dividend_yield
        Positive::TWO,                        // quantity
        Positive::ZERO,                            // premium_short_call
        Positive::ZERO,                            // premium_short_put
        pos_or_panic!(1.74),                       // open_fee_short_call
        pos_or_panic!(1.74),                       // close_fee_short_call
        pos_or_panic!(0.85),                       // open_fee_short_put
        pos_or_panic!(0.85),                       // close_fee_short_put
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
        let path: &std::path::Path =
            "Draws/Strategy/poor_mans_covered_call_profit_loss_chart_best_ratio.png".as_ref();
        strategy.write_png(path)?;
    }

    Ok(())
}
