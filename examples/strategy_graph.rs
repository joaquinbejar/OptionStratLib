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
        2,
        27.26,
        5.33,
        0.58,
        0.58,
        0.55,
        0.55,
    );
    let price_range: Vec<f64> = (2400..2600).map(|x| x as f64).collect();
    // let strategy = BullCallSpread::new(
    //     "GOLD".to_string(),
    //     100.0,
    //     90.0,
    //     110.0,
    //     ExpirationDate::Days(30.0),
    //     0.2,
    //     0.0,
    //     0.0,
    //     1,
    //     5.71,
    //     5.71,
    //     1.0,
    //     1.0,
    //     1.0,
    //     1.0,
    // );
    // let price_range: Vec<f64> = (60..140).map(|x| x as f64).collect();

    _ = strategy.max_profit();
    _ = strategy.max_loss();
    println!("Title: {}", strategy.title());
    println!("Break Even {}", strategy.break_even());
    // print net_premium_received, max_profit, max_loss, total_cost
    println!("Net Premium Received: {}", strategy.net_premium_received());
    println!("Max Profit: {}", strategy.max_profit());
    println!("Max Loss: {}", strategy.max_loss());
    println!("Total Cost: {}", strategy.total_cost());

    // Generate the intrinsic value graph
    strategy.graph(
        &price_range,
        "Draws/Strategy/bull_call_spread_value_chart.png",
    )?;

    Ok(())
}
