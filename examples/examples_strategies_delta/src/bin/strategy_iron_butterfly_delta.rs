use optionstratlib::prelude::*;
use positive::pos_or_panic;

fn main() -> Result<(), Error> {
    setup_logger();
    let underlying_price = pos_or_panic!(2810.9);

    let strategy = IronButterfly::new(
        "GOLD".to_string(),
        underlying_price,      // underlying_price
        pos_or_panic!(2725.0), // short_call_strike
        pos_or_panic!(2800.0), // long_call_strike
        pos_or_panic!(2500.0), // long_put_strike
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.1548), // implied_volatility
        dec!(0.05),            // risk_free_rate
        Positive::ZERO,        // dividend_yield
        Positive::TWO,         // quantity
        pos_or_panic!(38.8),   // premium_short_call
        pos_or_panic!(30.4),   // premium_short_put
        pos_or_panic!(23.3),   // premium_long_call
        pos_or_panic!(16.8),   // premium_long_put
        pos_or_panic!(0.96),   // open_fee
        pos_or_panic!(0.96),   // close_fee
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
        "Max Loss: ${}",
        strategy.get_max_loss().unwrap_or(Positive::ZERO)
    );
    info!("Total Fees: ${:.2}", strategy.get_fees()?);
    info!("Profit Area: {:.2}%", strategy.get_profit_area()?);
    info!("Profit Ratio: {:.2}%", strategy.get_profit_ratio()?);

    info!("Delta:  {:#?}", strategy.delta_neutrality()?);
    info!("Delta Neutral:  {}", strategy.is_delta_neutral());
    info!("Delta Suggestions:  {:#?}", strategy.delta_adjustments()?);

    Ok(())
}
