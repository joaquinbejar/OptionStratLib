use chrono::Utc;
use optionstratlib::model::option::Options;
use optionstratlib::model::position::Position;
use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, PositiveF64, Side};
use optionstratlib::pos;
use optionstratlib::strategies::base::Strategies;
use optionstratlib::strategies::custom::CustomStrategy;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::visualization::utils::Graph;
use std::error::Error;
use tracing::info;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let underlying_price = pos!(5780.0);
    let underlying_symbol = "SP500".to_string();
    let expiration = ExpirationDate::Days(60.0);
    let implied_volatility = 0.18;
    let risk_free_rate = 0.05;
    let dividend_yield = 0.0;

    // Short Call
    let short_strike = pos!(5800.0);
    let short_quantity = pos!(2.0);
    let premium_short = 53.04;
    let open_fee_short = 0.78;
    let close_fee_short = 0.78;

    // Long Call ITM
    let long_strike_itm = pos!(5750.0);
    let long_quantity = pos!(1.0);
    let premium_long_itm = 85.04;
    let open_fee_long = 0.78;
    let close_fee_long = 0.78;

    // Long Call OTM
    let long_strike_otm = pos!(5850.0);
    let premium_long_otm = 31.65;

    let short_call = Position::new(
        Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_strike,
            expiration.clone(),
            implied_volatility,
            short_quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        ),
        premium_short,
        Utc::now(),
        open_fee_short,
        close_fee_short,
    );

    let long_call_itm = Position::new(
        Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            long_strike_itm,
            expiration.clone(),
            implied_volatility,
            long_quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        ),
        premium_long_itm,
        Utc::now(),
        open_fee_long,
        close_fee_long,
    );

    let long_call_otm = Position::new(
        Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            long_strike_otm,
            expiration.clone(),
            implied_volatility,
            long_quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        ),
        premium_long_otm,
        Utc::now(),
        open_fee_long,
        close_fee_long,
    );

    let positions: Vec<Position> = vec![short_call, long_call_itm, long_call_otm];
    // let positions: Vec<Position> = vec![short_call, long_call_itm];

    let mut strategy = CustomStrategy::new(
        "Custom Strategy".to_string(),
        underlying_symbol,
        "Example of a custom strategy".to_string(),
        underlying_price,
        positions,
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
    info!("Max Profit: ${:.2}", strategy.max_profit_iter());
    info!("Max Loss: ${:.2}", strategy.max_loss_iter());
    info!("Total Fees: ${:.2}", strategy.fees());

    match strategy.break_even_points.len() {
        0 => info!("No break even points found"),
        1 => info!("Break Even Point: {:.2}", strategy.break_even_points[0]),
        2 => {
            info!(
                "Break Even Points: {}",
                strategy
                    .break_even_points
                    .iter()
                    .map(|p| format!("{:.2}", p))
                    .collect::<Vec<String>>()
                    .join(", ")
            );
            let range = strategy.break_even_points[1] - strategy.break_even_points[0];
            info!(
                "Range of Profit: ${:.2} {:.2}%",
                range,
                (range / 2.0) / underlying_price * 100.0
            );
        }
        _ => info!(
            "Break Even Points: {}",
            strategy
                .break_even_points
                .iter()
                .map(|p| format!("{:.2}", p))
                .collect::<Vec<String>>()
                .join(", ")
        ),
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
