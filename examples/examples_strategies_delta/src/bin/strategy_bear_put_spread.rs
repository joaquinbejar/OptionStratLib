/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/9/24
******************************************************************************/
use optionstratlib::pos;
use optionstratlib::strategies::bear_put_spread::BearPutSpread;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::Strategies;
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::utils::Graph;
use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use std::error::Error;
use rust_decimal_macros::dec;
use tracing::info;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let underlying_price = pos!(5781.88);

    let strategy = BearPutSpread::new(
        "SP500".to_string(),
        underlying_price,   // underlying_price
        pos!(5850.0),   // long_strike
        pos!(5720.0),   // short_strike
        ExpirationDate::Days(2.0),
        pos!(0.18),   // implied_volatility
        dec!(0.05),   // risk_free_rate
        Positive::ZERO,   // dividend_yield
        pos!(2.0),   // long quantity
        85.04,   // premium_long
        29.85,   // premium_short
        0.78,   // open_fee_long
        0.78,   // open_fee_long
        0.73,   // close_fee_long
        0.73,   // close_fee_short
    );

    info!("Title: {}", strategy.title());
    info!("Break Even Points: {:?}", strategy.break_even_points);
    info!(
        "Net Premium Received: ${:.2}",
        strategy.net_premium_received()?
    );
    info!(
        "Max Profit: ${:.2}",
        strategy.max_profit().unwrap_or(Positive::ZERO)
    );
    info!(
        "Max Loss: ${:0.2}",
        strategy.max_loss().unwrap_or(Positive::ZERO)
    );
    info!("Total Fees: ${:.2}", strategy.fees()?);
    info!("Profit Area: {:.2}%", strategy.profit_area()?);
    info!("Profit Ratio: {:.2}%", strategy.profit_ratio()?);

    info!("Delta:  {:#?}", strategy.calculate_net_delta());
    info!("Delta Neutral:  {}", strategy.is_delta_neutral());
    info!(
        "Delta Suggestions:  {:#?}",
        strategy.suggest_delta_adjustments()
    );

    Ok(())
}
