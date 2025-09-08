use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::pos;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::long_butterfly_spread::LongButterflySpread;
use optionstratlib::strategies::{BasicAble, Strategies};
use optionstratlib::utils::setup_logger;
use rust_decimal_macros::dec;
use std::error::Error;
use tracing::info;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let underlying_price = pos!(5781.88);

    let strategy = LongButterflySpread::new(
        "SP500".to_string(),
        underlying_price,
        pos!(5710.0),
        pos!(5820.0),
        pos!(6100.0),
        ExpirationDate::Days(pos!(2.0)),
        pos!(0.18),
        dec!(0.05),
        Positive::ZERO,
        pos!(1.0),
        pos!(49.65),
        pos!(42.93),
        Positive::ONE,
        pos!(4.0),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
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
