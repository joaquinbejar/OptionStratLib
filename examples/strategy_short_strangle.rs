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

    let underlying_price = 7138.5;

    let strategy = ShortStrangle::new(
        "CL".to_string(),
        underlying_price, // underlying_price
        7450.0,           // call_strike
        7050.0,           // put_strike
        ExpirationDate::Days(45.0),
        0.3745,    // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        pos!(1.0), // quantity
        84.2,      // premium_short_call
        353.2,     // premium_short_put
        7.01,      // open_fee_short_call
        7.01,      // close_fee_short_call
        7.01,      // open_fee_short_put
        7.01,      // close_fee_short_put
    );

    let price_range: Vec<f64> = (6400..=8100).map(|x| x as f64).collect();
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
        "Draws/Strategy/short_strangle_profit_loss_chart.png",
        20,
        (1400, 933),
    )?;

    Ok(())
}
