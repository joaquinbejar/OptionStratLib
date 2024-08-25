/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/8/24
******************************************************************************/
use optionstratlib::model::types::ExpirationDate;
use optionstratlib::strategies::base::Strategies;
use optionstratlib::strategies::bull_call_spread::BullCallSpread;
use optionstratlib::visualization::utils::Graph;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let strategy = BullCallSpread::new(
        "GOLD".to_string(),
        2505.8,
        2460.0,
        2515.0,
        ExpirationDate::Days(30.0),
        0.2,
        0.05,
        0.0,
        1,
        27.26,
        5.33,
        0.58,
        0.58,
        0.55,
        0.55,
    );
    let price_range: Vec<f64> = (2400..2600).map(|x| x as f64).collect();
    println!("Title: {}", strategy.title());
    println!("Break Even {}", strategy.break_even());
    println!("Net Premium Received: {}", strategy.net_premium_received());
    println!("Max Profit: {}", strategy.max_profit());
    println!("Max Loss: {}", strategy.max_loss());
    println!("Total Cost: {}", strategy.total_cost());

    // Generate the intrinsic value graph
    strategy.graph(
        &price_range,
        "Draws/Strategy/bull_call_spread_value_chart.png",
        20,
        (1400, 933),
        (10, 30),
        15
    )?;
    Ok(())
}
