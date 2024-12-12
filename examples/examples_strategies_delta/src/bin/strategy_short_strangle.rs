use optionstratlib::model::types::PositiveF64;
use optionstratlib::model::types::{ExpirationDate, PZERO};
use optionstratlib::pos;
use optionstratlib::strategies::base::Strategies;
use optionstratlib::strategies::strangle::ShortStrangle;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::visualization::utils::Graph;
use std::error::Error;
use tracing::info;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let underlying_price = pos!(7138.5);

    let strategy = ShortStrangle::new(
        "CL".to_string(),
        underlying_price, // underlying_price
        pos!(7450.0),     // call_strike 7450 (delta -0.415981)
        pos!(7250.0),     // put_strike 7050 (delta 0.417810) 
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
    // let price_range = strategy.best_range_to_show(pos!(1.0)).unwrap();
    let range = strategy.break_even_points[1] - strategy.break_even_points[0];

    info!("Title: {}", strategy.title());
    info!("Break Even Points: {:?}", strategy.break_even_points);
    info!(
        "Net Premium Received: ${:.2}",
        strategy.net_premium_received()
    );
    info!("Max Profit: ${:.2}", strategy.max_profit().unwrap_or(PZERO));
    info!("Max Loss: ${}", strategy.max_loss().unwrap_or(PZERO));
    info!("Total Fees: ${:.2}", strategy.fees());
    info!(
        "Range of Profit: ${:.2} {:.2}%",
        range,
        (range / 2.0) / underlying_price * 100.0
    );
    info!("Profit Area: {:.2}%", strategy.profit_area());
    info!("Profit Ratio: {:.2}%", strategy.profit_ratio());
    
    info!("Delta:  {:#?}", strategy.calculate_net_delta());
    info!("Delta Neutral:  {}", strategy.is_delta_neutral());
    info!("Delta Suggestions:  {:#?}", strategy.suggest_delta_adjustments());
    

    // Generate the profit/loss graph
    // strategy.graph(
    //     &price_range,
    //     "Draws/Strategy/short_strangle_profit_loss_chart.png",
    //     20,
    //     (1400, 933),
    // )?;

    Ok(())
}
