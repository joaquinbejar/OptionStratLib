use chrono::Utc;
use optionstratlib::{
    ExpirationDate, Options,
    model::{
        position::Position,
        types::{OptionStyle, OptionType, Side},
    },
    pos,
    strategies::{BasicAble, base::Positionable, long_put::LongPut, short_call::ShortCall},
    visualization::{Graph, GraphData},
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::error::Error;

// Helper function to create a test LongPut strategy
fn create_test_long_put() -> LongPut {
    // Create an instance of LongPut using Default
    let mut long_put = LongPut::default();

    // Customize the strategy for tests
    long_put.name = "Test Long Put".to_string();
    long_put.description = "Test Long Put Strategy".to_string();

    // Create an option for the position
    let option = Options::new(
        OptionType::European,
        Side::Long,
        "AAPL".to_string(),
        pos!(100.0), // Strike price
        ExpirationDate::Days(pos!(30.0)),
        pos!(0.3),        // Implied volatility
        pos!(1.0),        // Quantity
        pos!(95.0),       // Underlying price
        dec!(0.02),       // Risk-free rate
        OptionStyle::Put, // Put option
        pos!(0.01),       // Dividend yield
        None,             // Exotic parameters
    );

    // Create a position with the option
    let position = Position::new(
        option,
        pos!(5.0),  // Premium
        Utc::now(), // Entry date
        pos!(0.5),  // Open fee
        pos!(0.5),  // Close fee
    );

    // Add the position to the strategy
    long_put.add_position(&position).unwrap();

    // Update the break even points
    long_put.break_even_points = vec![pos!(95.0)];

    long_put
}

// Helper function to create a test ShortCall strategy
fn create_test_short_call() -> ShortCall {
    // Create an instance of ShortCall using Default
    let mut short_call = ShortCall::default();

    // Customize the strategy for tests
    short_call.name = "Test Short Call".to_string();
    short_call.description = "Test Short Call Strategy".to_string();

    // Create an option for the position
    let option = Options::new(
        OptionType::European,
        Side::Short, // Short side
        "AAPL".to_string(),
        pos!(100.0), // Strike price
        ExpirationDate::Days(pos!(30.0)),
        pos!(0.3),         // Implied volatility
        pos!(1.0),         // Quantity
        pos!(95.0),        // Underlying price
        dec!(0.02),        // Risk-free rate
        OptionStyle::Call, // Call option
        pos!(0.01),        // Dividend yield
        None,              // Exotic parameters
    );

    // Create a position with the option
    let position = Position::new(
        option,
        pos!(4.0),  // Premium
        Utc::now(), // Entry date
        pos!(0.5),  // Open fee
        pos!(0.5),  // Close fee
    );

    // Add the position to the strategy
    short_call.add_position(&position).unwrap();

    // Update the break even points
    short_call.break_even_points = vec![pos!(104.0)];

    short_call
}

#[test]
fn test_long_put_graph_data() -> Result<(), Box<dyn Error>> {
    let long_put = create_test_long_put();

    // Test graph_data
    let graph_data = long_put.graph_data();

    // Verify that we got a MultiSeries with 2 series (positive and negative)
    match graph_data {
        GraphData::MultiSeries(series) => {
            assert_eq!(series.len(), 2, "Should have positive and negative series");

            // Check that the series have the correct names
            assert_eq!(series[0].name, "Positive Payoff");
            assert_eq!(series[1].name, "Negative Payoff");

            // Verify that the series have data points
            assert!(
                !series[0].x.is_empty() || !series[1].x.is_empty(),
                "At least one series should have data points"
            );

            // Verify that positive series has positive y values (if not empty)
            if !series[0].y.is_empty() {
                for y in &series[0].y {
                    assert!(
                        *y >= Decimal::ZERO,
                        "Positive series should have non-negative values"
                    );
                }
            }

            // Verify that negative series has negative y values (if not empty)
            if !series[1].y.is_empty() && series[1].y != vec![Decimal::ZERO] {
                for y in &series[1].y {
                    assert!(
                        *y <= Decimal::ZERO,
                        "Negative series should have non-positive values"
                    );
                }
            }
        }
        _ => panic!("Expected MultiSeries graph data"),
    }

    Ok(())
}

#[test]
fn test_long_put_graph_config() -> Result<(), Box<dyn Error>> {
    let long_put = create_test_long_put();

    // Test graph_config
    let config = long_put.graph_config();

    // Verify config properties
    assert_eq!(config.title, long_put.get_title());
    assert_eq!(config.width, 1600);
    assert_eq!(config.height, 900);
    assert_eq!(config.x_label, Some("Underlying Price".to_string()));
    assert_eq!(config.y_label, Some("Profit/Loss".to_string()));
    assert_eq!(config.z_label, None);

    Ok(())
}

#[test]
fn test_short_call_graph_data() -> Result<(), Box<dyn Error>> {
    let short_call = create_test_short_call();

    // Test graph_data
    let graph_data = short_call.graph_data();

    // Verify that we got a MultiSeries with 2 series (positive and negative)
    match graph_data {
        GraphData::MultiSeries(series) => {
            assert_eq!(series.len(), 2, "Should have positive and negative series");

            // Check that the series have the correct names
            assert_eq!(series[0].name, "Positive Payoff");
            assert_eq!(series[1].name, "Negative Payoff");

            // Verify that the series have data points
            assert!(
                !series[0].x.is_empty() || !series[1].x.is_empty(),
                "At least one series should have data points"
            );

            // Verify that positive series has positive y values (if not empty)
            if !series[0].y.is_empty() {
                for y in &series[0].y {
                    assert!(
                        *y >= Decimal::ZERO,
                        "Positive series should have non-negative values"
                    );
                }
            }

            // Verify that negative series has negative y values (if not empty)
            if !series[1].y.is_empty() && series[1].y != vec![Decimal::ZERO] {
                for y in &series[1].y {
                    assert!(
                        *y <= Decimal::ZERO,
                        "Negative series should have non-positive values"
                    );
                }
            }
        }
        _ => panic!("Expected MultiSeries graph data"),
    }

    Ok(())
}

#[test]
fn test_short_call_graph_config() -> Result<(), Box<dyn Error>> {
    let short_call = create_test_short_call();

    // Test graph_config
    let config = short_call.graph_config();

    // Verify config properties
    assert_eq!(config.title, short_call.get_title());
    assert_eq!(config.width, 1600);
    assert_eq!(config.height, 900);
    assert_eq!(config.x_label, Some("Underlying Price".to_string()));
    assert_eq!(config.y_label, Some("Profit/Loss".to_string()));
    assert_eq!(config.z_label, None);

    Ok(())
}
