use optionstratlib::Positive;
use optionstratlib::model::types::{ExpirationDate, Positive::ZERO};
use optionstratlib::f2p;
use optionstratlib::strategies::base::Strategies;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::iron_butterfly::IronButterfly;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::visualization::utils::Graph;
use std::error::Error;
use tracing::info;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let underlying_price = f2p!(2810.9);

    let strategy = IronButterfly::new(
        "GOLD".to_string(),
        underlying_price, // underlying_price
        f2p!(2725.0),     // short_call_strike
        f2p!(2800.0),     // long_call_strike
        f2p!(2500.0),     // long_put_strike
        ExpirationDate::Days(30.0),
        0.1548,    // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        f2p!(2.0), // quantity
        38.8,      // premium_short_call
        30.4,      // premium_short_put
        23.3,      // premium_long_call
        16.8,      // premium_long_put
        0.96,      // open_fee
        0.96,      // close_fee
    );

    info!("Title: {}", strategy.title());
    info!("Break Even Points: {:?}", strategy.break_even_points);
    info!(
        "Net Premium Received: ${:.2}",
        strategy.net_premium_received()
    );
    info!("Max Profit: ${:.2}", strategy.max_profit().unwrap_or(Positive::ZERO));
    info!("Max Loss: ${}", strategy.max_loss().unwrap_or(Positive::ZERO));
    info!("Total Fees: ${:.2}", strategy.fees());
    info!("Profit Area: {:.2}%", strategy.profit_area());
    info!("Profit Ratio: {:.2}%", strategy.profit_ratio());

    info!("Delta:  {:#?}", strategy.calculate_net_delta());
    info!("Delta Neutral:  {}", strategy.is_delta_neutral());
    info!(
        "Delta Suggestions:  {:#?}",
        strategy.suggest_delta_adjustments()
    );

    Ok(())
}
