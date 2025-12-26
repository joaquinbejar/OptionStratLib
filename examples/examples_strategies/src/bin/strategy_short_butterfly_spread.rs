use optionstratlib::prelude::*;
use positive::pos_or_panic;

fn main() -> Result<(), Error> {
    setup_logger();
    let underlying_price = pos_or_panic!(5781.88);

    let strategy = ShortButterflySpread::new(
        "SP500".to_string(),
        underlying_price,      // underlying_price
        pos_or_panic!(5700.0), // short_strike_itm
        pos_or_panic!(5780.0), // long_strike
        pos_or_panic!(5850.0), // short_strike_otm
        ExpirationDate::Days(Positive::TWO),
        pos_or_panic!(0.18),   // implied_volatility
        dec!(0.05),            // risk_free_rate
        Positive::ZERO,        // dividend_yield
        pos_or_panic!(1.1),    // long quantity
        pos_or_panic!(119.01), // premium_long
        pos_or_panic!(66.0),   // premium_short
        pos_or_panic!(29.85),  // open_fee_long
        pos_or_panic!(0.05),
        pos_or_panic!(0.05),
        pos_or_panic!(0.05),
        pos_or_panic!(0.05),
        pos_or_panic!(0.05),
        pos_or_panic!(0.05),
    );

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
    info!("Profit Area: {:.2}%", strategy.get_profit_area()?);
    info!("Profit Ratio: {:.2}%", strategy.get_profit_ratio()?);

    let path: &std::path::Path =
        "Draws/Strategy/short_butterfly_spread_profit_loss_chart.png".as_ref();
    strategy.write_png(path)?;

    let prob = strategy.probability_of_profit(None, None)?;
    info!("Probability of Profit: {:.2}%", prob);

    let prob = strategy.probability_of_loss(None, None)?;
    info!("Probability of Loss: {:.2}%", prob);

    Ok(())
}
