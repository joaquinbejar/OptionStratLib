/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/8/24
******************************************************************************/
use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::pos;
use optionstratlib::strategies::Strategies;
use optionstratlib::strategies::base::BreakEvenable;
use optionstratlib::strategies::bull_call_spread::BullCallSpread;
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::utils::Graph;
use optionstratlib::visualization::utils::GraphBackend;
use rust_decimal_macros::dec;
use std::error::Error;
use tracing::info;

fn main() -> Result<(), Box<dyn Error>> {
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
    info!("Net Premium Received: {}", strategy.get_net_premium_received()?);
    info!(
        "Max Profit: {}",
        strategy.get_max_profit().unwrap_or(Positive::ZERO)
    );
    info!(
        "Max Loss: {}",
        strategy.get_max_loss().unwrap_or(Positive::ZERO)
    );
    info!("Total Cost: {}", strategy.get_total_cost()?);

    // Generate the intrinsic value graph
    strategy.graph(
        GraphBackend::Bitmap {
            file_path: "Draws/Strategy/bull_call_spread_value_chart.png",
            size: (1400, 933),
        },
        20,
    )?;
    Ok(())
}
