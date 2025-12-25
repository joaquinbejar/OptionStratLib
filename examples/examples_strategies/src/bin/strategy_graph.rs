/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/8/24
******************************************************************************/
use optionstratlib::prelude::*;
use positive::pos_or_panic;

fn main() -> Result<(), Error> {
    setup_logger();
    let strategy = BullCallSpread::new(
        "GOLD".to_string(),
        pos_or_panic!(2505.8),
        pos_or_panic!(2460.0),
        pos_or_panic!(2515.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(27.26),
        pos_or_panic!(5.33),
        pos_or_panic!(0.58),
        pos_or_panic!(0.58),
        pos_or_panic!(0.55),
        pos_or_panic!(0.54),
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
