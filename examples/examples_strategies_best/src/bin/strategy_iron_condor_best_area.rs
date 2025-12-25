use positive::pos_or_panic;
use optionstratlib::prelude::*;

fn main() -> Result<(), Error> {
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
        ExpirationDate::Days(pos_or_panic!(5.0)),
        Positive::ZERO,     // implied_volatility
        Decimal::ZERO,      // risk_free_rate
        Positive::ZERO,     // dividend_yield
        Positive::ONE, // quantity
        Positive::ZERO,     // premium_short_call
        Positive::ZERO,     // premium_short_put
        Positive::ZERO,     // premium_long_call
        Positive::ZERO,     // premium_long_put
        Positive::ONE,      // open_fee
        Positive::ONE,      // close_fee
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
            "Draws/Strategy/iron_condor_profit_loss_chart_best_area.png".as_ref();
        strategy.write_png(path)?;
    }
    info!("Greeks:  {:#?}", strategy.greeks());
    Ok(())
}
