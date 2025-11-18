/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/8/24
******************************************************************************/
use optionstratlib::prelude::*;

fn main() -> Result<(), Error> {
    setup_logger();
    let strategy = BullCallSpread::new(
        "GOLD".to_string(),
        pos!(2505.8),
        pos!(2460.0),
        pos!(2515.0),
        ExpirationDate::Days(pos!(30.0)),
        pos!(0.2),
        dec!(0.05),
        Positive::ZERO,
        pos!(1.0),
        pos!(27.26),
        pos!(5.33),
        pos!(0.58),
        pos!(0.58),
        pos!(0.55),
        pos!(0.54),
    );

    info!("Title: {}", strategy.get_title());
    info!("Break Even {:?}", strategy.get_break_even_points());
    info!(
        "Net Premium Received: {}",
        strategy.get_net_premium_received()?
    );
    info!(
        "Max Profit: {}",
        strategy.get_max_profit().unwrap_or(Positive::ZERO)
    );
    info!(
        "Max Loss: {}",
        strategy.get_max_loss().unwrap_or(Positive::ZERO)
    );
    info!("Total Cost: {}", strategy.get_total_cost()?);

    let path: &std::path::Path = "Draws/Strategy/bull_call_spread_value_chart.png".as_ref();
    strategy.write_png(path)?;

    Ok(())
}
