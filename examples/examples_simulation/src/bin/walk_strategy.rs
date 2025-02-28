/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 22/10/24
******************************************************************************/
use rust_decimal::Decimal;
use optionstratlib::simulation::walk::{RandomWalkGraph, Walkable};
use optionstratlib::utils::setup_logger;
use optionstratlib::utils::time::TimeFrame;
use optionstratlib::{pos, spos, ExpirationDate, Positive};
use optionstratlib::strategies::ShortStrangle;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    let symbol = "CL";
    let initial_price = pos!(7250.0);
    let days = pos!(45.0);
    let n_steps = (24.0 * days) as usize;

    let mean = 0.0;
    let std_dev = pos!(0.2);
    let std_dev_change = pos!(0.1);
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
    random_walk.generate_random_walk(n_steps, initial_price, mean, std_dev, std_dev_change)?;
    let mut strategy = ShortStrangle::new(
        symbol.to_string(),
        initial_price,  // underlying_price
        pos!(7450.0), // call_strike
        pos!(7050.0),  // put_strike
        ExpirationDate::Days(days),
        std_dev,   // implied_volatility
        risk_free_rate.unwrap(),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(2.0),      // quantity
        pos!(84.2),     // premium_short_call
        pos!(353.2),    // premium_short_put
        pos!(7.01),     // open_fee_short_call
        pos!(7.01),     // close_fee_short_call
        pos!(7.01),     // open_fee_short_put
        pos!(7.01),     // close_fee_short_put
    );
    let walk_result = random_walk.walk_strategy(&mut strategy, TimeFrame::Hour)?;
    println!("{:?}", walk_result);
    Ok(())
}
