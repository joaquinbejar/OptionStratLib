use optionstratlib::f2p;
use optionstratlib::strategies::straddle::LongStraddle;
use optionstratlib::strategies::Strategies;
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::utils::Graph;
use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use std::error::Error;
use tracing::info;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let underlying_price = f2p!(7008.5);

    let strategy = LongStraddle::new(
        "CL".to_string(),
        underlying_price, // underlying_price
        f2p!(7140.0),     // put_strike
        ExpirationDate::Days(45.0),
        0.3745,    // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        f2p!(1.0), // quantity
        84.2,      // premium_short_call
        353.2,     // premium_short_put
        7.0,       // open_fee_short_call
        7.01,      // close_fee_short_call
        7.01,      // open_fee_short_put
        7.01,      // close_fee_short_put
    );
    let price_range = strategy.best_range_to_show(f2p!(1.0)).unwrap();
    let range = strategy.break_even_points[1] - strategy.break_even_points[0];

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
        "Max Loss: ${}",
        strategy.max_loss().unwrap_or(Positive::ZERO)
    );
    info!("Total Fees: ${:.2}", strategy.fees()?);
    info!(
        "Range of Profit: ${:.2} {:.2}%",
        range,
        (range / 2.0) / underlying_price * 100.0
    );
    info!("Profit Area: {:.2}%", strategy.profit_area()?);
    info!("Profit Ratio: {:.2}%", strategy.profit_ratio()?);

    // Generate the profit/loss graph
    strategy.graph(
        &price_range,
        "Draws/Strategy/long_straddle_profit_loss_chart.png",
        20,
        (1400, 933),
    )?;

    Ok(())
}
