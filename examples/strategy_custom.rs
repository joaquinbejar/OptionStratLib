use optionstratlib::model::types::PositiveF64;
use optionstratlib::pos;
use optionstratlib::strategies::base::Strategies;
use optionstratlib::strategies::custom::CustomStrategy;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::visualization::utils::Graph;
use std::error::Error;
use tracing::info;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let underlying_price = pos!(7138.5);

    let strategy = CustomStrategy::new(
        "Custom Strategy".to_string(),
        "GOLD".to_string(),
        "Example of a custom strategy".to_string(),
        underlying_price,
        0.01,
        100,
        0.1,
    );
    let price_range = strategy.best_range_to_show(pos!(1.0)).unwrap();

    info!("Title: {}", strategy.title());
    info!(
        "Net Premium Received: ${:.2}",
        strategy.net_premium_received()
    );
    info!("Max Profit: ${:.2}", strategy.max_profit());
    info!("Max Loss: ${}", strategy.max_loss());
    info!("Total Fees: ${:.2}", strategy.fees());

    if strategy.break_even_points.len() < 2 {
        info!("No break even points found");
    } else {
        info!("Break Even Points: {:?}", strategy.break_even_points);
        let range = strategy.break_even_points[1] - strategy.break_even_points[0];
        info!(
            "Range of Profit: ${:.2} {:.2}%",
            range,
            (range / 2.0) / underlying_price * 100.0
        );
    }

    info!("Profit Area: {:.2}%", strategy.profit_area());
    info!("Profit Ratio: {:.2}%", strategy.profit_ratio());

    // Generate the profit/loss graph
    strategy.graph(
        &price_range,
        "Draws/Strategy/custom_strategy_profit_loss_chart.png",
        20,
        (1400, 933),
    )?;

    Ok(())
}
