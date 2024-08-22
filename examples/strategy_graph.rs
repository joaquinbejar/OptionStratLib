/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/8/24
******************************************************************************/
use chrono::{NaiveDate, TimeZone, Utc};
use optionstratlib::model::option::Options;
use optionstratlib::model::position::Position;
use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use optionstratlib::strategies::base::{Strategy, StrategyType};
use optionstratlib::visualization::utils::Graph;
use std::error::Error;

fn create_sample_option(
    option_style: OptionStyle,
    side: Side,
    underlying_price: f64,
    strike: f64,
    quantity: u32,
) -> Options {
    let naive_date = NaiveDate::from_ymd_opt(2024, 8, 8)
        .expect("Invalid date")
        .and_hms_opt(0, 0, 0)
        .expect("Invalid time");
    Options::new(
        OptionType::European,
        side,
        "AAPL".to_string(),
        strike,
        ExpirationDate::DateTime(Utc.from_utc_datetime(&naive_date)),
        0.2,
        quantity,
        underlying_price,
        0.05,
        option_style,
        0.01,
        None,
    )
}
fn main() -> Result<(), Box<dyn Error>> {
    let position_long_call = Position::new(
        create_sample_option(OptionStyle::Call, Side::Long, 100.0, 90.0, 1),
        5.71,
        Utc::now(),
        1.0,
        1.0,
    );

    let position_short_call = Position::new(
        create_sample_option(OptionStyle::Call, Side::Short, 100.0, 110.0, 1),
        5.71,
        Utc::now(),
        1.0,
        1.0,
    );

    let mut strategy = Strategy::new(
        "Bull Call Spread".to_string(),
        StrategyType::BullCallSpread,
        "A bull call spread consists of buying a call option and selling another call option with a higher strike price to offset the cost.".to_string(),
    );
    strategy.add_leg(position_long_call.clone());
    strategy.add_leg(position_short_call.clone());
    strategy.add_break_even_point(position_long_call.break_even());
    strategy.add_break_even_point(position_short_call.break_even());

    println!("Title: {}", strategy.title());
    println!("Break Even {}", strategy.break_even());
    // Define a range of prices for the graph
    let price_range: Vec<f64> = (50..150).map(|x| x as f64).collect();

    // Generate the intrinsic value graph
    strategy.graph(&price_range, "Draws/Strategy/intrinsic_value_chart.png")?;

    Ok(())
}
