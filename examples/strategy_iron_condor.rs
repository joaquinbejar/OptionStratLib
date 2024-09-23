use optionstratlib::model::types::ExpirationDate;
use optionstratlib::strategies::base::Strategies;
use optionstratlib::strategies::iron_condor::IronCondor;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::visualization::utils::Graph;
use std::error::Error;
use tracing::info;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let underlying_price = 2646.9;

    let strategy = IronCondor::new(
        "GOLD".to_string(),
        underlying_price,     // underlying_price
        2725.0,                // short_call_strike
        2560.0,                 // short_put_strike
        2800.0,                // long_call_strike
        2500.0,                 // long_put_strike
        ExpirationDate::Days(30.0),
        0.1548, // implied_volatility
        0.05,   // risk_free_rate
        0.0,    // dividend_yield
        2,      // quantity
        38.8,                  // premium_short_call
        30.4,                  // premium_short_put
        23.3,                  // premium_long_call
        16.8,                  // premium_long_put
        0.96,                 // open_fee
        0.96,                 // close_fee
    );

    let price_range: Vec<f64> = (2450..=2950).map(|x| x as f64).collect();
    let range = strategy.break_even_points[1] - strategy.break_even_points[0];

    info!("Title: {}", strategy.title());
    info!("Break Even Points: {:?}", strategy.break_even_points);
    info!(
        "Net Premium Received: ${:.2}",
        strategy.net_premium_received()
    );
    info!("Max Profit: ${:.2}", strategy.max_profit());
    info!("Max Loss: ${:.2}", strategy.max_loss());
    info!("Total Fees: ${:.2}", strategy.fees());
    info!(
        "Range of Profit: ${:.2} {:.2}%",
        range,
        (range / 2.0) / underlying_price * 100.0
    );
    info!("Profit Area: {:.2}%", strategy.area());

    // Generate the profit/loss graph
    strategy.graph(
        &price_range,
        "Draws/Strategy/iron_condor_profit_loss_chart.png",
        20,
        (1400, 933),
        (10, 30),
        15,
    )?;

    Ok(())
}