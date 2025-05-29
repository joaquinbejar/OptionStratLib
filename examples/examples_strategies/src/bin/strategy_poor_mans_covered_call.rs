use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::pos;
use optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall;
use optionstratlib::strategies::{BasicAble, Strategies};
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::Graph;
use rust_decimal_macros::dec;
use std::error::Error;
use tracing::info;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let underlying_price = pos!(2703.3);

    let strategy = PoorMansCoveredCall::new(
        "GOLD".to_string(),                // underlying_symbol
        underlying_price,                  // underlying_price
        pos!(2600.0),                      // long_call_strike
        pos!(2800.0),                      // short_call_strike OTM
        ExpirationDate::Days(pos!(120.0)), // long_call_expiration
        ExpirationDate::Days(pos!(30.0)),  // short_call_expiration 30-45 days delta 0.30 or less
        pos!(0.17),                        // implied_volatility
        dec!(0.05),                        // risk_free_rate
        Positive::ZERO,                    // dividend_yield
        pos!(2.1),                         // quantity
        pos!(154.7),                       // premium_short_call
        pos!(30.8),                        // premium_short_put
        pos!(1.74),                        // open_fee_short_call
        pos!(1.74),                        // close_fee_short_call
        pos!(0.85),                        // open_fee_short_put
        pos!(0.85),                        // close_fee_short_put
    );
    
    info!("Title: {}", strategy.get_title());
    info!("Break Even Points: {:?}", strategy.break_even_points);
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

    let path: &std::path::Path =
        "Draws/Strategy/poor_mans_covered_call_profit_loss_chart.png".as_ref();
    strategy.write_png(path)?;

    Ok(())
}
