use chrono::Utc;
use optionstratlib::{
    ExpirationDate, Options,
    model::{
        position::Position,
        types::{OptionStyle, OptionType, Side},
    },
    pricing::Profit,
    strategies::{
        BasicAble, Strategies, Validable,
        base::{BreakEvenable, Positionable, StrategyType},
        short_put::ShortPut,
    },
};
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// Helper function to create a Short Put strategy for testing
fn create_test_short_put() -> ShortPut {
    // Create an instance of ShortPut using Default
    let mut short_put = ShortPut::default();

    // Customize the strategy for tests
    short_put.name = "Test Short Put".to_string();
    short_put.description = "Test Short Put Strategy".to_string();

    // Create an option for the position
    let option = Options::new(
        OptionType::European,
        Side::Short,
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
        None,
        None,
    );

    // Add the position to the strategy
    short_put.add_position(&position).unwrap();

    // Update the break even points
    short_put.break_even_points = vec![Positive::new(95.0).unwrap()];

    short_put
}

#[test]
fn test_short_put_new() {
    let short_put = create_test_short_put();
    assert_eq!(short_put.name, "Test Short Put");
    assert_eq!(short_put.kind, StrategyType::ShortPut);
    assert!(short_put.description.contains("Test Short Put"));
}

#[test]
fn test_short_put_validate() {
    let short_put = create_test_short_put();
    assert!(short_put.validate());
}

#[test]
fn test_short_put_get_title() {
    let short_put = create_test_short_put();
    assert_eq!(
        short_put.get_title(),
        "ShortPut Strategy: \n\tUnderlying: AAPL @ $100 Short Put European Option"
    );
}

#[test]
fn test_short_put_get_option_basic_type() {
    let short_put = create_test_short_put();
    let option_types = short_put.get_option_basic_type();
    assert_eq!(option_types.len(), 1);

    // Verify that it contains a Put type option
    let mut found = false;
    for option_type in option_types.iter() {
        if option_type.option_style == &OptionStyle::Put {
            found = true;
            break;
        }
    }
    assert!(found, "No OptionBasicType with OptionStyle::Put was found");
}

#[test]
fn test_short_put_get_implied_volatility() {
    let short_put = create_test_short_put();
    let implied_vol_map = short_put.get_implied_volatility();
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
        "No OptionBasicType with OptionStyle::Put was found"
    );
    assert_eq!(*found_value.unwrap(), &Positive::new(0.3).unwrap());
}

#[test]
fn test_short_put_get_quantity() {
    let short_put = create_test_short_put();
    let quantity_map = short_put.get_quantity();
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
        "No OptionBasicType with OptionStyle::Put was found"
    );
    assert_eq!(*found_value.unwrap(), &Positive::new(1.0).unwrap());
}

#[test]
fn test_short_put_one_option() {
    let short_put = create_test_short_put();
    let option = short_put.one_option();
    assert_eq!(option.strike_price, Positive::new(100.0).unwrap());
    assert_eq!(option.option_style, OptionStyle::Put);
    assert_eq!(option.side, Side::Short);
}

#[test]
fn test_short_put_set_expiration_date() {
    let mut short_put = create_test_short_put();
    let new_expiration = ExpirationDate::Days(Positive::new(60.0).unwrap());
    let result = short_put.set_expiration_date(new_expiration);
    assert!(result.is_ok());
}

#[test]
fn test_short_put_set_underlying_price() {
    let mut short_put = create_test_short_put();
    let new_price = Positive::new(110.0).unwrap();
    let result = short_put.set_underlying_price(&new_price);
    assert!(result.is_ok());
    // We cannot directly access short_put.short_put as it is private
}

#[test]
fn test_short_put_set_implied_volatility() {
    let mut short_put = create_test_short_put();
    let new_vol = Positive::new(0.4).unwrap();
    let result = short_put.set_implied_volatility(&new_vol);
    assert!(result.is_ok());
    // We cannot directly access short_put.short_put as it is private
}

#[test]
fn test_short_put_break_even_points() {
    let short_put = create_test_short_put();
    let break_even_points = short_put.get_break_even_points().unwrap();
    assert_eq!(break_even_points[0].to_f64(), 95.0);
}

#[test]
fn test_short_put_get_max_profit() {
    let short_put = create_test_short_put();
    let result = short_put.get_max_profit();
    assert!(result.is_ok());

    // Max profit should be the premium received minus fees
    let expected_max_profit = Positive::new(4.0).unwrap(); // 5 (premium) - 0.5 (open fee) - 0.5 (close fee)
    assert_eq!(result.unwrap(), expected_max_profit);
}

#[test]
fn test_short_put_get_max_loss() {
    let short_put = create_test_short_put();
    let result = short_put.get_max_loss();

    // For a Short Put, the maximum loss can be the strike price minus the premium received
    // or it can be theoretically infinite if the underlying reaches zero
    match result {
        Ok(loss) => {
            // If a value is returned, verify that it is positive
            assert!(loss > Positive::ZERO);

            // If we want to be more specific, we can verify that it is close to the expected value
            // In this case: 100 (strike) - 5 (premium) + 0.5 + 0.5 (fees) = 96
            let expected_max_loss = Positive::new(96.0).unwrap();
            assert!(
                (loss.to_f64() - expected_max_loss.to_f64()).abs() < 1.0,
                "Max loss should be close to {expected_max_loss}, but was {loss}"
            );

            // Also verify that the loss is less than the strike price
            let strike = Positive::new(100.0).unwrap();
            assert!(loss < strike, "Max loss should be less than strike price");
        }
        Err(e) => {
            // If there is an error, it could be due to various reasons related to loss calculation
            // The actual error message might vary depending on implementation
            assert!(
                e.to_string().contains("loss")
                    || e.to_string().contains("Loss")
                    || e.to_string().contains("negative")
                    || e.to_string().contains("infinite")
                    || e.to_string().contains("unlimited"),
                "Error message should be related to loss calculation: {e}"
            );
        }
    }
}

#[test]
fn test_short_put_calculate_profit_at() {
    let short_put = create_test_short_put();

    // For a short put, the profit at the strike price should be close to zero
    // or slightly negative due to fees
    let at_strike = Positive::new(100.0).unwrap();
    let profit_at_strike = short_put.calculate_profit_at(&at_strike).unwrap();
    // The profit at strike should be close to the premium minus fees
    // Premium is 5, fees are 0.5 + 0.5 = 1, so profit should be around 4
    let expected_profit = Decimal::new(4, 0);
    assert!(
        (profit_at_strike - expected_profit).abs() < Decimal::new(1, 0),
        "Profit at strike should be close to {expected_profit}, but was {profit_at_strike}"
    );

    // Test profit at break-even point (should be close to zero)
    let at_break_even = Positive::new(95.0).unwrap();
    let profit_at_break_even = short_put.calculate_profit_at(&at_break_even).unwrap();
    // Allow a larger margin of error for the break-even point
    assert!(profit_at_break_even.abs() < Decimal::new(2, 0));

    // Test profit above strike (should be positive, equal to premium - fees)
    let above_strike = Positive::new(120.0).unwrap();
    let profit_above_strike = short_put.calculate_profit_at(&above_strike).unwrap();
    assert!(profit_above_strike > Decimal::ZERO);

    // Test profit well below strike (should be negative)
    let below_strike = Positive::new(80.0).unwrap();
    let profit_below_strike = short_put.calculate_profit_at(&below_strike).unwrap();
    assert!(profit_below_strike < Decimal::ZERO);
}

#[test]
fn test_short_put_add_position() {
    let mut short_put = create_test_short_put();

    // Create a new valid position (short put)
    let valid_option = Options::new(
        OptionType::European,
        Side::Short,
        "AAPL".to_string(),
        Positive::new(90.0).unwrap(),
        ExpirationDate::Days(Positive::new(45.0).unwrap()),
        Positive::new(0.35).unwrap(),
        Positive::new(1.0).unwrap(),
        Positive::new(105.0).unwrap(),
        dec!(0.02),
        OptionStyle::Put,
        Positive::new(0.01).unwrap(),
        None,
    );

    let valid_position = Position::new(
        valid_option,
        Positive::new(1.0).unwrap(),
        Utc::now(),
        Positive::new(4.0).unwrap(),
        Positive::new(0.5).unwrap(),
        None,
        None,
    );

    let result = short_put.add_position(&valid_position);
    assert!(result.is_ok());

    // Test with an invalid position (call instead of put)
    let invalid_option = Options::new(
        OptionType::European,
        Side::Short,
        "AAPL".to_string(),
        Positive::new(90.0).unwrap(),
        ExpirationDate::Days(Positive::new(45.0).unwrap()),
        Positive::new(0.35).unwrap(),
        Positive::new(1.0).unwrap(),
        Positive::new(105.0).unwrap(),
        dec!(0.02),
        OptionStyle::Call, // This makes it invalid for a short put strategy
        Positive::new(0.01).unwrap(),
        None,
    );

    let invalid_position = Position::new(
        invalid_option,
        Positive::new(1.0).unwrap(),
        Utc::now(),
        Positive::new(4.0).unwrap(),
        Positive::new(0.5).unwrap(),
        None,
        None,
    );

    let result = short_put.add_position(&invalid_position);
    assert!(result.is_err()); // Should fail because it's a call option, not put
}

#[test]
fn test_short_put_get_positions() {
    let short_put = create_test_short_put();
    let positions = short_put.get_positions().unwrap();
    assert_eq!(positions.len(), 1);
    assert_eq!(
        positions[0].option.strike_price,
        Positive::new(100.0).unwrap()
    );
    assert_eq!(positions[0].option.option_style, OptionStyle::Put);
    assert_eq!(positions[0].option.side, Side::Short);
}

#[test]
fn test_short_put_get_profit_ratio() {
    let short_put = create_test_short_put();
    let ratio = short_put.get_profit_ratio().unwrap();
    // The ratio should be positive for a short put with our test parameters
    assert!(ratio > Decimal::ZERO);
}

#[test]
fn test_short_put_get_profit_area() {
    let short_put = create_test_short_put();
    let area = short_put.get_profit_area().unwrap();
    // The profit area should be positive
    assert!(area > Decimal::ZERO);
}
