use chrono::Utc;
use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use optionstratlib::model::option::Options;
use optionstratlib::model::position::Position;
use optionstratlib::ExpirationDate;
use optionstratlib::strategies::custom::CustomStrategy;
use optionstratlib::strategies::base::{Strategies, BreakEvenable, BasicAble};
use optionstratlib::pricing::Profit;
use optionstratlib::greeks::Greeks;
use optionstratlib::pos;
use optionstratlib::visualization::Graph;
use rust_decimal_macros::dec;
use tracing::info;
use optionstratlib::utils::setup_logger;

fn main() {
    setup_logger();

    info!("=== CUSTOM STRATEGY - DAX Complex Strategy ===");

    // Strategy parameters
    let underlying_symbol = "DAX".to_string();
    let underlying_price = pos!(24067.0);
    let expiration = ExpirationDate::Days(pos!(1.0));
    let implied_volatility = pos!(0.18); // 18%
    let risk_free_rate = dec!(0.0);
    let dividend_yield = pos!(0.0);
    let fee = pos!(0.10);

    // We'll create the strategy after creating positions

    // Position 1: Long Call Strike 24210, Premium 27.3, Size 1
    let long_call_option = Options::new(
        OptionType::European,
        Side::Long,
        underlying_symbol.clone(),
        pos!(24210.0), // strike
        expiration.clone(),
        implied_volatility,
        pos!(1.0), // quantity
        underlying_price,
        risk_free_rate,
        OptionStyle::Call,
        dividend_yield,
        None,
    );
    let long_call = Position::new(
        long_call_option,
        pos!(27.3), // premium
        Utc::now(),
        fee, // open_fee
        fee, // close_fee
        None,
        None,
    );

    // Position 2: Long Put Strike 24070, Premium 30.4, Size 1
    let long_put_option = Options::new(
        OptionType::European,
        Side::Long,
        underlying_symbol.clone(),
        pos!(24070.0), // strike
        expiration.clone(),
        implied_volatility,
        pos!(1.0), // quantity
        underlying_price,
        risk_free_rate,
        OptionStyle::Put,
        dividend_yield,
        None,
    );
    let long_put = Position::new(
        long_put_option,
        pos!(30.4), // premium
        Utc::now(),
        fee, // open_fee
        fee, // close_fee
        None,
        None,
    );

    // Position 3: Short Put Strike 24050, Premium 45.5, Size 1
    let short_put_option = Options::new(
        OptionType::European,
        Side::Short,
        underlying_symbol.clone(),
        pos!(24060.0), // strike
        expiration.clone(),
        implied_volatility,
        pos!(1.0), // quantity
        underlying_price,
        risk_free_rate,
        OptionStyle::Put,
        dividend_yield,
        None,
    );
    let short_put = Position::new(
        short_put_option,
        pos!(60.0), // premium
        Utc::now(),
        fee, // open_fee
        fee, // close_fee
        None,
        None,
    );

    // Create CustomStrategy with the positions
    let positions = vec![long_call, long_put, short_put];
    let strategy = CustomStrategy::new(
        "DAX Complex Strategy".to_string(),
        underlying_symbol,
        "A complex DAX strategy with long call, long put, and short put positions".to_string(),
        underlying_price,
        positions,
        pos!(1.0), // Default quantity
        1, // days to expiration
        implied_volatility,
    );

    // Fees are already set in Position::new() above

    // Display strategy information
    info!("Title: {}", strategy.get_title());
    match strategy.get_break_even_points() {
        Ok(be_points) => info!("Break Even Points: {:?}", be_points),
        Err(_) => info!("Break Even Points: Unable to calculate"),
    }
    
    // Calculate net premium
    let net_premium = strategy.get_net_premium_received().unwrap_or(pos!(0.0));
    info!("Net Premium: ${:.2}", net_premium);
    
    // Calculate max profit and loss
    let max_profit = strategy.get_max_profit().unwrap_or(pos!(0.0));
    let max_loss = strategy.get_max_loss().unwrap_or(pos!(0.0));
    info!("Max Profit: ${:.2}", max_profit);
    info!("Max Loss: ${:.2}", max_loss);
    
    // Calculate total fees
    let total_fees = strategy.get_fees().unwrap_or(pos!(0.0));
    info!("Total Fees: ${:.2}", total_fees);
    
    // Calculate profit area and ratio
    let profit_area = strategy.get_profit_area().unwrap_or(dec!(0.0));
    info!("Profit Area: {:.2}%", profit_area * dec!(100.0));
    
    // Calculate profit ratio
    let profit_ratio = strategy.get_profit_ratio().unwrap_or(dec!(0.0));
    info!("Profit Ratio: {:.2}%", profit_ratio * dec!(100.0));

    // Calculate Greeks
    let greeks = strategy.greeks();
    info!("Greeks: {:?}", greeks);

    // Test profit calculation at various price points
    info!("\n=== PROFIT/LOSS AT DIFFERENT PRICES ===");
    let test_prices = vec![23800.0, 23950.0, 24050.0, 24067.0, 24070.0, 24100.0, 24210.0, 24350.0];
    
    for &price in &test_prices {
        let test_price = pos!(price);
        match strategy.calculate_profit_at(&test_price) {
            Ok(profit) => {
                info!("At ${:.0}: Profit/Loss = ${:.2}", price, profit);
            }
            Err(e) => {
                info!("At ${:.0}: Error calculating profit: {:?}", price, e);
            }
        }
    }

    // Generate and save charts
    info!("\n=== GENERATING CHARTS ===");
    match strategy.get_best_range_to_show(pos!(1.0)) {
        Ok(range) => {
            if range.len() >= 2 {
                info!("Chart range: ${:.0} to ${:.0}", range[0], range[range.len()-1]);
            }
            
            // Generate PNG chart
            let png_path: &std::path::Path = "Draws/Strategy/custom_dax_strategy_chart.png".as_ref();
            match strategy.write_png(png_path) {
                Ok(_) => info!("PNG chart saved to: {:?}", png_path),
                Err(e) => info!("Error saving PNG chart: {:?}", e),
            }

            // Generate HTML chart
            let html_path: &std::path::Path = "Draws/Strategy/custom_dax_strategy_chart.html".as_ref();
            match strategy.write_html(html_path) {
                Ok(_) => info!("HTML chart saved to: {:?}", html_path),
                Err(e) => info!("Error saving HTML chart: {:?}", e),
            }
        }
        Err(e) => {
            info!("Error generating chart range: {:?}", e);
        }
    }

    info!("\n=== STRATEGY ANALYSIS ===");
    info!("This DAX strategy combines:");
    info!("1. Long Call (24210) - Bullish exposure above 24210");
    info!("2. Long Put (24070) - Bearish protection below 24070"); 
    info!("3. Short Put (24050) - Premium collection, risk below 24050");
    info!("");
    info!("Strategy characteristics:");
    info!("- Limited upside potential above 24210");
    info!("- Protected downside between 24070-24050");
    info!("- Maximum risk below 24050");
    info!("- Net premium: ${:.2}", net_premium);
    info!("- Current underlying: ${:.0}", underlying_price);
    info!("- Time to expiration: 1 day");
    info!("- Implied volatility: 18%");
}
