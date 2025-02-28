/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 22/10/24
******************************************************************************/
use optionstratlib::simulation::walk::{RandomWalkGraph, Walkable};
use optionstratlib::strategies::ShortStrangle;
use optionstratlib::utils::setup_logger;
use optionstratlib::utils::time::TimeFrame;
use optionstratlib::visualization::utils::{Graph, GraphBackend};
use optionstratlib::{ExpirationDate, Positive, pos, spos};
use rust_decimal::Decimal;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    let symbol = "CL";
    let initial_price = pos!(7250.0);
    let days = pos!(45.0);
    let n_steps = (24.0 * days) as usize;

    let mean = 0.01;
    let std_dev = pos!(0.2);
    let std_dev_change = pos!(0.01);
    let risk_free_rate = Some(Decimal::ZERO);
    let dividend_yield = spos!(0.0);
    let volatility_window = 20;
    let initial_volatility = Some(std_dev);
    let mut random_walk = RandomWalkGraph::new(
        symbol.to_string(),
        risk_free_rate,
        dividend_yield,
        TimeFrame::Day,
        volatility_window,
        initial_volatility,
    );
    random_walk.generate_random_walk_timeframe(
        n_steps,
        initial_price,
        mean,
        std_dev,
        std_dev_change,
        TimeFrame::Hour,
        Some((pos!(0.15), pos!(0.28))),
    )?;
    let mut strategy = ShortStrangle::new(
        symbol.to_string(),
        initial_price, // underlying_price
        pos!(7450.0),  // call_strike
        pos!(7050.0),  // put_strike
        ExpirationDate::Days(days),
        std_dev,                 // implied_volatility
        risk_free_rate.unwrap(), // risk_free_rate
        Positive::ZERO,          // dividend_yield
        pos!(2.0),               // quantity
        pos!(84.2),              // premium_short_call
        pos!(353.2),             // premium_short_put
        pos!(7.01),              // open_fee_short_call
        pos!(7.01),              // close_fee_short_call
        pos!(7.01),              // open_fee_short_put
        pos!(7.01),              // close_fee_short_put
    );
    random_walk.graph(
        &random_walk.get_x_values(),
        GraphBackend::Bitmap {
            file_path: "Draws/Simulation/strategy_walk.png",
            size: (1200, 800),
        },
        20,
    )?;
    let walk_result = random_walk.walk_strategy(&mut strategy, TimeFrame::Hour)?;
    let _json_output = serde_json::to_string_pretty(&walk_result)?;
    // println!("{}", json_output);
    Ok(())
}
