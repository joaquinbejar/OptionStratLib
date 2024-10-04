use optionstratlib::model::types::ExpirationDate;
use optionstratlib::model::types::PositiveF64;
use optionstratlib::pos;
use optionstratlib::strategies::base::Strategies;
use optionstratlib::strategies::strangle::ShortStrangle;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::visualization::utils::Graph;
use std::error::Error;
use tracing::info;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let underlying_price = pos!(2655.6);

    let strategy = ShortStrangle::new(
        "GOLD".to_string(),
        underlying_price, // underlying_price
        pos!(2480.0),     // call_strike
        pos!(2650.0),     // put_strike
        ExpirationDate::Days(3.0),
        0.1548,    // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        pos!(2.0), // quantity
        46.3,      // premium_short_call
        4.6,       // premium_short_put
        0.96,      // open_fee_short_call
        0.96,      // close_fee_short_call
        0.96,      // open_fee_short_put
        0.96,      // close_fee_short_put
    );

    let price_range: Vec<PositiveF64> = (2450..=2850).map(|x| pos!(x as f64)).collect();

    let range = strategy.break_even_points[1] - strategy.break_even_points[0];

    info!("Title: {}", strategy.title());
    info!("Break Even Points: {:?}", strategy.break_even_points);
    info!(
        "Net Premium Received: ${:.2}",
        strategy.net_premium_received()
    );
    info!("Max Profit: ${:.2}", strategy.max_profit());
    info!("Max Loss: ${}", strategy.max_loss());
    info!("Total Fees: ${:.2}", strategy.fees());
    info!(
        "Range of Profit: ${:.2} {:.2}%",
        range,
        (range / 2.0) / underlying_price * 100.0
    );
    info!("Profit Area: {:.2}%", strategy.profit_area());

    // Generate the profit/loss graph
    strategy.graph(
        &price_range,
        "Draws/Strategy/short_strangle_delta_profit_loss_chart.png",
        20,
        (1400, 933),
    )?;

    Ok(())
}
