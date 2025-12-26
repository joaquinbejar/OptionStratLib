use optionstratlib::prelude::*;
use positive::pos_or_panic;

fn main() -> Result<(), Error> {
    setup_logger();
    let underlying_price = pos_or_panic!(7138.5);

    let strategy = ShortStraddle::new(
        "CL".to_string(),
        underlying_price,      // underlying_price
        pos_or_panic!(7460.0), // call_strike
        ExpirationDate::Days(pos_or_panic!(45.0)),
        pos_or_panic!(0.3745), // implied_volatility
        Decimal::ZERO,         // risk_free_rate
        Positive::ZERO,        // dividend_yield
        Positive::ONE,         // quantity
        pos_or_panic!(84.2),   // premium_short_call
        pos_or_panic!(353.2),  // premium_short_put
        pos_or_panic!(7.01),   // open_fee_short_call
        pos_or_panic!(7.01),   // close_fee_short_call
        pos_or_panic!(7.01),   // open_fee_short_put
        pos_or_panic!(7.01),   // close_fee_short_put
    );
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
        "Max Loss: ${}",
        strategy.get_max_loss().unwrap_or(Positive::ZERO)
    );
    info!("Total Fees: ${:.2}", strategy.get_fees()?);
    info!(
        "Range of Profit: ${:.2} {:.2}%",
        range,
        (range / 2.0) / underlying_price * 100.0
    );
    info!("Profit Area: {:.2}%", strategy.get_profit_area()?);
    info!("Profit Ratio: {:.2}%", strategy.get_profit_ratio()?);

    info!("Delta:  {:#?}", strategy.delta_neutrality()?);
    info!("Delta Neutral:  {}", strategy.is_delta_neutral());
    info!("Delta Suggestions:  {:#?}", strategy.delta_adjustments()?);

    Ok(())
}
