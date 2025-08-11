use optionstratlib::{ExpirationDate, Options, Positive};
use optionstratlib::greeks::Greeks;
use optionstratlib::model::position::Position;
use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use optionstratlib::pos;
use optionstratlib::strategies::{BasicAble, Strategies};
use optionstratlib::strategies::custom::CustomStrategy;
use optionstratlib::pricing::Profit;
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::Graph;
use rust_decimal_macros::dec;
use std::error::Error;
use tracing::info;
use chrono::Utc;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let underlying_price = pos!(150.0);
    let underlying_symbol = "SPY".to_string();
    let expiration = ExpirationDate::Days(pos!(21.0));
    let risk_free_rate = dec!(0.05);
    let dividend_yield = pos!(0.02); // 2% dividend yield

    // Create a simple covered call strategy using CustomStrategy
    // This demonstrates how to build standard strategies with CustomStrategy
    
    // Long stock position (simulated with deep ITM call with very low premium)
    let stock_position_option = Options::new(
        OptionType::European,
        Side::Long,
        underlying_symbol.clone(),
        pos!(50.0), // Deep ITM call to simulate stock
        expiration,
        pos!(0.01), // Very low volatility for deep ITM
        pos!(100.0), // 100 shares equivalent
        underlying_price,
        risk_free_rate,
        OptionStyle::Call,
        dividend_yield,
        None,
    );
    let stock_position = Position::new(
        stock_position_option,
        pos!(100.0), // Intrinsic value (150-50)
        Utc::now(),
        pos!(0.5), // Low fee
        pos!(0.5),
        None,
        None,
    );

    // Short call (covered call)
    let covered_call_option = Options::new(
        OptionType::European,
        Side::Short,
        underlying_symbol.clone(),
        pos!(155.0), // Strike above current price
        expiration,
        pos!(0.20), // 20% implied volatility
        pos!(100.0), // 100 shares covered
        underlying_price,
        risk_free_rate,
        OptionStyle::Call,
        dividend_yield,
        None,
    );
    let covered_call = Position::new(
        covered_call_option,
        pos!(3.50), // Premium received
        Utc::now(),
        pos!(1.0),
        pos!(1.0),
        None,
        None,
    );

    // Create CustomStrategy
    let positions = vec![stock_position, covered_call];
    let strategy = CustomStrategy::new(
        "Custom Covered Call".to_string(),
        underlying_symbol,
        "Simple covered call strategy implemented with CustomStrategy".to_string(),
        underlying_price,
        positions,
        Default::default(),
        50, // Fewer calculation points for simplicity
        Default::default(),
    );

    // Display strategy information
    info!("=== CUSTOM COVERED CALL STRATEGY ===");
    info!("Title: {}", strategy.get_title());
    info!("Description: {}", strategy.description);
    info!("Underlying Price: ${:.2}", underlying_price);
    info!("Break Even Points: {:?}", strategy.break_even_points);

    info!("=== POSITION DETAILS ===");
    info!("Position 1: Long Stock (simulated with deep ITM call)");
    info!("  - Strike: ${:.2}", strategy.positions[0].option.strike_price);
    info!("  - Quantity: {}", strategy.positions[0].option.quantity);
    info!("  - Premium Paid: ${:.2}", strategy.positions[0].premium);
    
    info!("Position 2: Short Call (covered call)");
    info!("  - Strike: ${:.2}", strategy.positions[1].option.strike_price);
    info!("  - Quantity: {}", strategy.positions[1].option.quantity);
    info!("  - Premium Received: ${:.2}", strategy.positions[1].premium);

    info!("=== FINANCIAL ANALYSIS ===");
    info!(
        "Net Premium: ${:.2}",
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
    info!("Profit Area: {:.2}%", strategy.get_profit_area()?);
    info!("Profit Ratio: {:.2}%", strategy.get_profit_ratio()?);

    // Test profit calculation at different price points
    info!("=== PROFIT/LOSS AT DIFFERENT PRICES ===");
    let test_prices = vec![140.0, 145.0, 150.0, 155.0, 160.0, 165.0];
    for price in test_prices {
        let test_price = pos!(price);
        match strategy.calculate_profit_at(&test_price) {
            Ok(profit) => info!("At ${:.2}: Profit/Loss = ${:.2}", price, profit),
            Err(e) => info!("At ${:.2}: Error calculating profit: {}", price, e),
        }
    }

    // Generate charts
    let path: &std::path::Path = "Draws/Strategy/custom_covered_call_chart.png".as_ref();
    strategy.write_png(path)?;

    let path: &std::path::Path = "Draws/Strategy/custom_covered_call_chart.html".as_ref();
    strategy.write_html(path)?;

    info!("Greeks: {:#?}", strategy.greeks());

    info!("=== STRATEGY SUMMARY ===");
    info!("This covered call strategy:");
    info!("- Provides income from call premium");
    info!("- Limits upside potential above strike");
    info!("- Offers some downside protection");
    info!("- Demonstrates CustomStrategy's versatility");
    info!("Charts saved for detailed analysis.");

    Ok(())
}
