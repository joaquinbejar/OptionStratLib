use optionstratlib::prelude::*;
use positive::pos_or_panic;

fn main() -> Result<(), Error> {
    setup_logger();
    let underlying_price = pos_or_panic!(23684.0);

    let strategy = IronCondor::new(
        "DAX".to_string(),
        underlying_price,       // underlying_price
        pos_or_panic!(23730.0), // short_call_strike
        pos_or_panic!(23630.0), // short_put_strike
        pos_or_panic!(23775.0), // long_call_strike
        pos_or_panic!(23580.0), // long_put_strike
        ExpirationDate::Days(pos_or_panic!(0.4)),
        pos_or_panic!(0.19), // implied_volatility
        dec!(0.0),           // risk_free_rate
        Positive::ZERO,      // dividend_yield
        Positive::ONE,       // quantity
        pos_or_panic!(40.1), // premium_short_call
        pos_or_panic!(39.4), // premium_short_put
        pos_or_panic!(30.4), // premium_long_call
        pos_or_panic!(30.7), // premium_long_put
        pos_or_panic!(0.1),  // open_fee
        pos_or_panic!(0.1),  // close_fee
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

    let path: &std::path::Path = "Draws/Strategy/iron_condor_profit_loss_chart.png".as_ref();
    strategy.write_png(path)?;

    Ok(())
}
