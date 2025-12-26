use optionstratlib::prelude::*;
use positive::pos_or_panic;

fn main() -> Result<(), Error> {
    setup_logger();
    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    let underlying_price = option_chain.underlying_price;
    let mut strategy = ShortButterflySpread::new(
        "SP500".to_string(),
        underlying_price,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        ExpirationDate::Days(pos_or_panic!(5.0)),
        Positive::ZERO,
        Decimal::ZERO,
        Positive::ZERO,
        Positive::ONE,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        pos_or_panic!(4.0),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
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
            "Draws/Strategy/short_butterfly_spread_profit_loss_chart_best_ratio.png".as_ref();
        strategy.write_png(path)?;
    }

    Ok(())
}
