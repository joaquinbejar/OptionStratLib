use optionstratlib::prelude::*;
use positive::pos_or_panic;

fn main() -> Result<(), Error> {
    setup_logger();
    let underlying_price = Positive::HUNDRED;
    let underlying_symbol = "AAPL".to_string();
    let expiration = ExpirationDate::Days(pos_or_panic!(30.0));
    let risk_free_rate = dec!(0.05);
    let dividend_yield = Positive::ZERO;

    // Create a complex custom strategy with multiple positions
    // This creates a modified Iron Condor with unequal quantities

    // Long Put (protective) - Strike 90
    let long_put_option = Options::new(
        OptionType::European,
        Side::Long,
        underlying_symbol.clone(),
        pos_or_panic!(90.0),
        expiration,
        pos_or_panic!(0.25),
        Positive::ONE, // quantity
        underlying_price,
        risk_free_rate,
        OptionStyle::Put,
        dividend_yield,
        None,
    );
    let long_put = Position::new(
        long_put_option,
        pos_or_panic!(2.5), // premium paid
        Utc::now(),
        Positive::ONE, // open fee
        Positive::ONE, // close fee
        None,
        None,
    );

    // Short Put - Strike 95
    let short_put_option = Options::new(
        OptionType::European,
        Side::Short,
        underlying_symbol.clone(),
        pos_or_panic!(95.0),
        expiration,
        pos_or_panic!(0.28),
        Positive::TWO, // double quantity
        underlying_price,
        risk_free_rate,
        OptionStyle::Put,
        dividend_yield,
        None,
    );
    let short_put = Position::new(
        short_put_option,
        pos_or_panic!(4.2), // premium received
        Utc::now(),
        Positive::ONE,
        Positive::ONE,
        None,
        None,
    );

    // Short Call - Strike 105
    let short_call_option = Options::new(
        OptionType::European,
        Side::Short,
        underlying_symbol.clone(),
        pos_or_panic!(105.0),
        expiration,
        pos_or_panic!(0.26),
        Positive::TWO, // double quantity
        underlying_price,
        risk_free_rate,
        OptionStyle::Call,
        dividend_yield,
        None,
    );
    let short_call = Position::new(
        short_call_option,
        pos_or_panic!(3.8), // premium received
        Utc::now(),
        Positive::ONE,
        Positive::ONE,
        None,
        None,
    );

    // Long Call (protective) - Strike 110
    let long_call_option = Options::new(
        OptionType::European,
        Side::Long,
        underlying_symbol.clone(),
        pos_or_panic!(110.0),
        expiration,
        pos_or_panic!(0.23),
        Positive::ONE, // quantity
        underlying_price,
        risk_free_rate,
        OptionStyle::Call,
        dividend_yield,
        None,
    );
    let long_call = Position::new(
        long_call_option,
        pos_or_panic!(1.8), // premium paid
        Utc::now(),
        Positive::ONE,
        Positive::ONE,
        None,
        None,
    );

    // Create CustomStrategy with all positions
    let positions = vec![long_put, short_put, short_call, long_call];
    let strategy = CustomStrategy::new(
        "Custom Complex Strategy".to_string(),
        underlying_symbol,
        "Modified Iron Condor with unequal quantities - demonstrates CustomStrategy flexibility"
            .to_string(),
        underlying_price,
        positions,
        Default::default(),
        100,
        Default::default(),
    );

    // Display strategy information
    info!("=== CUSTOM COMPLEX STRATEGY ===");
    info!("Title: {}", strategy.get_title());
    info!("Description: {}", strategy.description);
    info!("Number of Positions: {}", strategy.positions.len());
    info!("Break Even Points: {:?}", strategy.break_even_points);

    // Show individual positions
    info!("=== POSITIONS BREAKDOWN ===");
    for (i, position) in strategy.positions.iter().enumerate() {
        info!(
            "Position {}: {} {} {} Strike:{} Qty:{} Premium:{}",
            i + 1,
            match position.option.side {
                Side::Long => "Long",
                Side::Short => "Short",
            },
            match position.option.option_style {
                OptionStyle::Call => "Call",
                OptionStyle::Put => "Put",
            },
            position.option.underlying_symbol,
            position.option.strike_price,
            position.option.quantity,
            position.premium
        );
    }

    info!("=== FINANCIAL METRICS ===");
    info!("Net Premium: ${:.2}", strategy.get_net_premium_received()?);
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

    // Generate charts
    let path: &std::path::Path = "Draws/Strategy/custom_complex_strategy_chart.png".as_ref();
    strategy.write_png(path)?;

    let path: &std::path::Path = "Draws/Strategy/custom_complex_strategy_chart.html".as_ref();
    strategy.write_html(path)?;

    info!("Greeks: {:#?}", strategy.greeks());

    info!("=== STRATEGY NOTES ===");
    info!("This demonstrates CustomStrategy's flexibility:");
    info!("- Mixed position types (Long/Short Calls/Puts)");
    info!("- Unequal quantities (1x protective, 2x short)");
    info!("- Different strike prices and volatilities");
    info!("- Automatic break-even calculation");
    info!("- Full integration with visualization and Greeks");

    Ok(())
}
