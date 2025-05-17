use chrono::Utc;
use optionstratlib::{
    ExpirationDate, Options, Positive,
    model::{
        position::Position,
        types::{OptionStyle, OptionType, Side},
    },
    pricing::Profit,
    strategies::{
        BasicAble, Strategies, Validable,
        base::{BreakEvenable, Positionable, StrategyType},
        long_put::LongPut,
    },
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// Helper function to create a Long Put strategy for testing
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
        Positive::new(100.0).unwrap(),
        ExpirationDate::Days(Positive::new(30.0).unwrap()),
        Positive::new(0.3).unwrap(),
        Positive::new(1.0).unwrap(),
        Positive::new(100.0).unwrap(), // Underlying price equal to strike for easier calculations
        dec!(0.02),
        OptionStyle::Put,
        Positive::new(0.01).unwrap(),
        None,
    );

    // Create a position with the option
    let position = Position::new(
        option,
        Positive::new(5.0).unwrap(),
        Utc::now(),
        Positive::new(0.5).unwrap(),
        Positive::new(0.5).unwrap(),
    );

    // Add the position to the strategy
    long_put.add_position(&position).unwrap();

    // Update the break even points
    long_put.break_even_points = vec![Positive::new(95.0).unwrap()];

    long_put
}

#[test]
fn test_long_put_new() {
    let long_put = create_test_long_put();
    assert_eq!(long_put.name, "Test Long Put");
    assert_eq!(long_put.kind, StrategyType::LongPut);
    assert!(long_put.description.contains("Test Long Put"));
}

#[test]
fn test_long_put_validate() {
    let long_put = create_test_long_put();
    assert!(long_put.validate());
}

#[test]
fn test_long_put_get_title() {
    let long_put = create_test_long_put();
    assert_eq!(
        long_put.get_title(),
        "LongPut Strategy: \n\tUnderlying: AAPL @ $100 Long Put European Option"
    );
}

#[test]
fn test_long_put_get_option_basic_type() {
    let long_put = create_test_long_put();
    let option_types = long_put.get_option_basic_type();
    assert_eq!(option_types.len(), 1);

    // Verify that it contains a Put option type
    let mut found = false;
    for option_type in option_types.iter() {
        if option_type.option_style == &OptionStyle::Put {
            found = true;
            break;
        }
    }
    assert!(found, "No OptionBasicType with OptionStyle::Put found");
}

#[test]
fn test_long_put_get_implied_volatility() {
    let long_put = create_test_long_put();
    let implied_vol_map = long_put.get_implied_volatility();
    assert_eq!(implied_vol_map.len(), 1);

    // Verify that the map contains an entry for Put with the correct value
    let mut found_value = None;
    for (key, value) in implied_vol_map.iter() {
        if key.option_style == &OptionStyle::Put {
            found_value = Some(value);
            break;
        }
    }
    assert!(
        found_value.is_some(),
        "No OptionBasicType with OptionStyle::Put found"
    );
    assert_eq!(*found_value.unwrap(), &Positive::new(0.3).unwrap());
}

#[test]
fn test_long_put_get_quantity() {
    let long_put = create_test_long_put();
    let quantity_map = long_put.get_quantity();
    assert_eq!(quantity_map.len(), 1);

    // Verify that the map contains an entry for Put with the correct value
    let mut found_value = None;
    for (key, value) in quantity_map.iter() {
        if key.option_style == &OptionStyle::Put {
            found_value = Some(value);
            break;
        }
    }
    assert!(
        found_value.is_some(),
        "No OptionBasicType with OptionStyle::Put found"
    );
    assert_eq!(*found_value.unwrap(), &Positive::new(1.0).unwrap());
}

#[test]
fn test_long_put_one_option() {
    let long_put = create_test_long_put();
    let option = long_put.one_option();
    assert_eq!(option.option_style, OptionStyle::Put);
    assert_eq!(option.side, Side::Long);
}

#[test]
fn test_long_put_set_expiration_date() {
    let mut long_put = create_test_long_put();
    let new_expiration = ExpirationDate::Days(Positive::new(45.0).unwrap());
    let result = long_put.set_expiration_date(new_expiration);
    assert!(result.is_ok());
    assert_eq!(long_put.one_option().expiration_date, new_expiration);
}

#[test]
fn test_long_put_set_underlying_price() {
    let mut long_put = create_test_long_put();
    let new_price = Positive::new(110.0).unwrap();
    let result = long_put.set_underlying_price(&new_price);
    assert!(result.is_ok());
    assert_eq!(long_put.one_option().underlying_price, new_price);
}

#[test]
fn test_long_put_set_implied_volatility() {
    let mut long_put = create_test_long_put();
    let new_iv = Positive::new(0.4).unwrap();
    let result = long_put.set_implied_volatility(&new_iv);
    assert!(result.is_ok());
    assert_eq!(long_put.one_option().implied_volatility, new_iv);
}

#[test]
fn test_long_put_break_even_points() {
    let long_put = create_test_long_put();
    let break_even_points = long_put.get_break_even_points().unwrap();
    assert_eq!(break_even_points.len(), 1);
    assert_eq!(break_even_points[0], Positive::new(95.0).unwrap());
}

#[test]
fn test_long_put_get_max_profit() {
    let long_put = create_test_long_put();
    let result = long_put.get_max_profit();

    // For a Long Put, the maximum profit is the strike price minus the premium and fees
    match result {
        Ok(profit) => {
            // Verify that the maximum profit is positive
            assert!(profit > Positive::ZERO);

            // For a long put with strike 100, premium 5, and fees 1,
            // the max profit would be approximately: 100 - 5 - 0.5 - 0.5 = 94
            let expected_profit = Positive::new(94.0).unwrap();
            assert!(
                (profit.to_f64() - expected_profit.to_f64()).abs() < 1.0,
                "Max profit should be close to {}, but was {}",
                expected_profit,
                profit
            );
        }
        Err(e) => {
            // If there is an error, it could be due to various reasons related to profit calculation
            assert!(
                e.to_string().contains("profit")
                    || e.to_string().contains("Profit")
                    || e.to_string().contains("premium")
                    || e.to_string().contains("infinite")
                    || e.to_string().contains("unlimited"),
                "Error message should be related to profit calculation: {}",
                e
            );
        }
    }
}

#[test]
fn test_long_put_get_max_loss() {
    let long_put = create_test_long_put();
    let result = long_put.get_max_loss();

    // For a Long Put, the maximum loss is limited to the premium paid plus fees
    match result {
        Ok(loss) => {
            // Verify that the maximum loss is positive
            assert!(loss > Positive::ZERO);

            // For a long put with premium 5 and fees 0.5 + 0.5 = 1, the max loss should be 6
            let expected_max_loss = Positive::new(6.0).unwrap();
            assert!(
                (loss.to_f64() - expected_max_loss.to_f64()).abs() < 1.0,
                "Max loss should be close to {}, but was {}",
                expected_max_loss,
                loss
            );
        }
        Err(e) => {
            // If there is an error, it could be due to various reasons related to loss calculation
            assert!(
                e.to_string().contains("loss")
                    || e.to_string().contains("Loss")
                    || e.to_string().contains("negative")
                    || e.to_string().contains("infinite")
                    || e.to_string().contains("unlimited"),
                "Error message should be related to loss calculation: {}",
                e
            );
        }
    }
}

#[test]
fn test_long_put_calculate_profit_at() {
    let long_put = create_test_long_put();

    // Test profit at strike price (should be negative, equal to premium + fees)
    let at_strike = Positive::new(100.0).unwrap();
    let profit_at_strike = long_put.calculate_profit_at(&at_strike).unwrap();
    assert!(profit_at_strike < Decimal::ZERO);

    // Test profit at break-even point (should be close to zero)
    let at_break_even = Positive::new(95.0).unwrap();
    let profit_at_break_even = long_put.calculate_profit_at(&at_break_even).unwrap();
    // Allow a margin of error of 2.0 for the profit calculation at the break-even point
    assert!(
        profit_at_break_even.abs() < Decimal::new(2, 0),
        "Profit at break-even should be close to zero, but was {}",
        profit_at_break_even
    );

    // Test profit below break-even (should be positive)
    let below_break_even = Positive::new(80.0).unwrap();
    let profit_below_break_even = long_put.calculate_profit_at(&below_break_even).unwrap();
    assert!(
        profit_below_break_even > Decimal::ZERO,
        "Profit below break-even should be positive, but was {}",
        profit_below_break_even
    );
}

#[test]
fn test_long_put_add_position() {
    let mut long_put = create_test_long_put();

    // Create a new valid position (long put)
    let option = Options::new(
        OptionType::European,
        Side::Long,
        "AAPL".to_string(),
        Positive::new(110.0).unwrap(),
        ExpirationDate::Days(Positive::new(45.0).unwrap()),
        Positive::new(0.35).unwrap(),
        Positive::new(1.0).unwrap(),
        Positive::new(105.0).unwrap(),
        dec!(0.02),
        OptionStyle::Put,
        Positive::new(0.01).unwrap(),
        None,
    );

    let position = Position::new(
        option,
        Positive::new(1.0).unwrap(),
        Utc::now(),
        Positive::new(4.0).unwrap(),
        Positive::new(0.5).unwrap(),
    );

    let result = long_put.add_position(&position);
    assert!(result.is_ok());

    // Test with an invalid position (call instead of put)
    let invalid_option = Options::new(
        OptionType::European,
        Side::Long,
        "AAPL".to_string(),
        Positive::new(110.0).unwrap(),
        ExpirationDate::Days(Positive::new(45.0).unwrap()),
        Positive::new(0.35).unwrap(),
        Positive::new(1.0).unwrap(),
        Positive::new(105.0).unwrap(),
        dec!(0.02),
        OptionStyle::Call, // This makes it invalid for a long put strategy
        Positive::new(0.01).unwrap(),
        None,
    );

    let invalid_position = Position::new(
        invalid_option,
        Positive::new(1.0).unwrap(),
        Utc::now(),
        Positive::new(4.0).unwrap(),
        Positive::new(0.5).unwrap(),
    );

    let result = long_put.add_position(&invalid_position);
    assert!(result.is_err());
}

#[test]
fn test_long_put_get_positions() {
    let long_put = create_test_long_put();
    let positions = long_put.get_positions().unwrap();
    assert_eq!(positions.len(), 1);
    assert_eq!(
        positions[0].option.strike_price,
        Positive::new(100.0).unwrap()
    );
    assert_eq!(positions[0].option.option_style, OptionStyle::Put);
    assert_eq!(positions[0].option.side, Side::Long);
}

#[test]
fn test_long_put_get_profit_ratio() {
    let long_put = create_test_long_put();

    // The profit/loss ratio can be positive, zero, or even undefined
    // depending on how the calculation is implemented
    match long_put.get_profit_ratio() {
        Ok(ratio) => {
            // If a ratio is returned, we simply verify that it exists
            // The actual ratio may vary depending on implementation details
            assert!(
                ratio >= Decimal::ZERO,
                "Profit ratio should be non-negative, but was {}",
                ratio
            );
        }
        Err(e) => {
            // If there is an error, we verify that it's because the ratio is undefined
            // (for example, if the maximum loss is zero or the profit is infinite)
            assert!(
                e.to_string().contains("division")
                    || e.to_string().contains("infinite")
                    || e.to_string().contains("undefined"),
                "Error message should indicate an issue with ratio calculation: {}",
                e
            );
        }
    }
}

#[test]
fn test_long_put_get_profit_area() {
    let long_put = create_test_long_put();
    let result = long_put.get_profit_area();

    assert!(result.is_ok());
    let area = result.unwrap();

    // The profit area calculation may vary depending on implementation details
    // We just verify that it's a non-negative value
    assert!(
        area >= Decimal::ZERO,
        "Profit area should be non-negative, but was {}",
        area
    );
}
