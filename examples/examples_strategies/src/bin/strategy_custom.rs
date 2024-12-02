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

    let underlying_price = pos!(2340.0);
    let underlying_symbol = "GAS".to_string();
    let expiration = ExpirationDate::Days(6.0);
    let implied_volatility = 0.73;
    let risk_free_rate = 0.05;
    let dividend_yield = 0.0;

    // Short Call 1
    let short_strike_1_strike = pos!(2100.0);
    let short_strike_1_quantity = pos!(2.0);
    let short_strike_1_premium = 192.0;
    let short_strike_1_open_fee = 7.51;
    let short_strike_1_close_fee = 7.51;

    // Short Call 2
    let short_strike_2_strike = pos!(2250.0);
    let short_strike_2_quantity = pos!(2.0);
    let short_strike_2_premium = 88.0;
    let short_strike_2_open_fee = 6.68;
    let short_strike_2_close_fee = 6.68;

    // Short Put
    let short_put_strike = pos!(2500.0);
    let short_put_premium = 55.0;
    let short_put_quantity = pos!(1.0);
    let short_put_open_fee = 6.68;
    let short_put_close_fee = 6.68;

    let short_strike_1 = Position::new(
        Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_strike_1_strike,
            expiration.clone(),
            implied_volatility,
            short_strike_1_quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        ),
        short_strike_1_premium,
        Utc::now(),
        short_strike_1_open_fee,
        short_strike_1_close_fee,
    );

    let short_strike_2 = Position::new(
        Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_strike_2_strike,
            expiration.clone(),
            implied_volatility,
            short_strike_2_quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        ),
        short_strike_2_premium,
        Utc::now(),
        short_strike_2_open_fee,
        short_strike_2_close_fee,
    );

    let short_put = Position::new(
        Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_put_strike,
            expiration.clone(),
            implied_volatility,
            short_put_quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Put,
            dividend_yield,
            None,
        ),
        short_put_premium,
        Utc::now(),
        short_put_open_fee,
        short_put_close_fee,
    );

    let extra_strike = pos!(2150.0);
    let extra_quantity = pos!(2.5);
    let extra_premium = 21.0;
    let extra_open_fee = 4.91;
    let extra_close_fee = 4.91;

    let extra = Position::new(
        Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            extra_strike,
            expiration.clone(),
            implied_volatility,
            extra_quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Put,
            dividend_yield,
            None,
        ),
        extra_premium,
        Utc::now(),
        extra_open_fee,
        extra_close_fee,
    );

    let positions: Vec<Position> = vec![short_strike_1, short_strike_2, short_put, extra];

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
