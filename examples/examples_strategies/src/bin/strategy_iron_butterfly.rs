use optionstratlib::prelude::*;

fn main() -> Result<(), Error> {
    setup_logger();
    let underlying_price = pos!(2646.9);

    let strategy = IronButterfly::new(
        "GOLD".to_string(),
        underlying_price, // underlying_price
        pos!(2725.0),     // short_call_strike
        pos!(2800.0),     // long_call_strike
        pos!(2500.0),     // long_put_strike
        ExpirationDate::Days(pos!(30.0)),
        pos!(0.1548),   // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(2.1),      // quantity
        pos!(38.8),     // premium_short_call
        pos!(30.4),     // premium_short_put
        pos!(23.3),     // premium_long_call
        pos!(16.8),     // premium_long_put
        pos!(0.96),     // open_fee
        pos!(0.96),     // close_fee
    );
    if !strategy.validate() {
        return Err("Invalid strategy".into());
    }
    let range = strategy.break_even_points[1] - strategy.break_even_points[0];

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
        "Max Loss: ${:.2}",
        strategy.get_max_loss().unwrap_or(Positive::ZERO)
    );
    info!("Total Fees: ${:.2}", strategy.get_fees()?);
    info!(
        "Range of Profit: ${:.2} {:.2}%",
        range,
        (range / 2.0) / underlying_price * 100.0
    );
    info!("Profit Area: {:.2}%", strategy.get_profit_area()?);

    let path: &std::path::Path = "Draws/Strategy/iron_butterfly_profit_loss_chart.png".as_ref();
    strategy.write_png(path)?;

    Ok(())
}
