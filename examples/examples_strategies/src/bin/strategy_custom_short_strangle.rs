use chrono::Utc;
use optionstratlib::greeks::Greeks;
use optionstratlib::model::position::Position;
use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use optionstratlib::pos;
use optionstratlib::strategies::custom::CustomStrategy;
use optionstratlib::strategies::{BasicAble, Strategies};
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::Graph;
use optionstratlib::{ExpirationDate, Options, Positive};
use rust_decimal_macros::dec;
use std::error::Error;
use tracing::info;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let underlying_price = pos!(7138.5);
    let underlying_symbol = "CL".to_string();
    let expiration = ExpirationDate::Days(pos!(45.0));
    let risk_free_rate = dec!(0.05);
    let dividend_yield = Positive::ZERO;
    let quantity = pos!(2.0);

    // Create the same positions as ShortStrangle example
    // Short Call at 7450.0 strike
    let short_call_option = Options::new(
        OptionType::European,
        Side::Short,
        underlying_symbol.clone(),
        pos!(7450.0), // call_strike
        expiration,
        pos!(0.3745), // call_implied_volatility
        quantity,
        underlying_price,
        risk_free_rate,
        OptionStyle::Call,
        dividend_yield,
        None,
    );
    let short_call = Position::new(
        short_call_option,
        pos!(84.2), // premium_short_call
        Utc::now(),
        pos!(7.01), // open_fee_short_call
        pos!(7.01), // close_fee_short_call
        None,
        None,
    );

    // Short Put at 7050.0 strike
    let short_put_option = Options::new(
        OptionType::European,
        Side::Short,
        underlying_symbol.clone(),
        pos!(7050.0), // put_strike
        expiration,
        pos!(0.3745), // put_implied_volatility
        quantity,
        underlying_price,
        risk_free_rate,
        OptionStyle::Put,
        dividend_yield,
        None,
    );
    let short_put = Position::new(
        short_put_option,
        pos!(353.2), // premium_short_put
        Utc::now(),
        pos!(7.01), // open_fee_short_put
        pos!(7.01), // close_fee_short_put
        None,
        None,
    );

    // Create CustomStrategy with the same positions
    let positions = vec![short_call, short_put];
    let strategy = CustomStrategy::new(
        "Custom Short Strangle".to_string(),
        underlying_symbol,
        "Custom implementation of Short Strangle strategy for comparison".to_string(),
        underlying_price,
        positions,
        Default::default(),
        100,
        Default::default(),
    );

    // Calculate range between break-even points (if we have at least 2)
    let range = if strategy.break_even_points.len() >= 2 {
        strategy.break_even_points[1] - strategy.break_even_points[0]
    } else {
        pos!(0.0)
    };

    // Display strategy information
    info!("=== CUSTOM STRATEGY (Short Strangle Implementation) ===");
    info!("Title: {}", strategy.get_title());
    info!("Break Even Points: {:?}", strategy.break_even_points);
    info!(
        "Net Premium Received: ${:.2}",
        strategy.get_net_premium_received()?
    );
    info!(
        "Max Profit: ${:.2}",
        strategy.get_max_profit().unwrap_or(Positive::ZERO)
    );
    info!(
        "Max Loss: ${}",
        strategy.get_max_loss().unwrap_or(Positive::ZERO)
    );
    info!("Total Fees: ${:.2}", strategy.get_fees()?);

    if range > pos!(0.0) {
        info!(
            "Range of Profit: ${:.2} {:.2}%",
            range,
            (range / 2.0) / underlying_price * 100.0
        );
    }

    info!("Profit Area: {:.2}%", strategy.get_profit_area()?);
    info!("Profit Ratio: {:.2}%", strategy.get_profit_ratio()?);

    // Generate charts
    let path: &std::path::Path =
        "Draws/Strategy/custom_short_strangle_profit_loss_chart.png".as_ref();
    strategy.write_png(path)?;

    let path: &std::path::Path =
        "Draws/Strategy/custom_short_strangle_profit_loss_chart.html".as_ref();
    strategy.write_html(path)?;

    info!("Greeks: {:#?}", strategy.greeks());

    info!("=== COMPARISON NOTES ===");
    info!("This CustomStrategy replicates the exact same positions as the ShortStrangle example:");
    info!("- Short Call: Strike 7450.0, Premium 84.2, Fees 7.01 each");
    info!("- Short Put: Strike 7050.0, Premium 353.2, Fees 7.01 each");
    info!("- Underlying: CL at 7138.5");
    info!("- Expiration: 45 days");
    info!("- IV: 37.45% for both options");
    info!("Charts saved to Draws/Strategy/ directory for comparison.");

    Ok(())
}
