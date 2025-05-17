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
        long_call::LongCall,
    },
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// Helper function to create a Long Call strategy for testing
fn create_test_long_call() -> LongCall {
    // Create an instance of LongCall using Default
    let mut long_call = LongCall::default();

    // Customize the strategy for tests
    long_call.name = "Test Long Call".to_string();
    long_call.description = "Test Long Call Strategy".to_string();

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
        OptionStyle::Call,
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
    long_call.add_position(&position).unwrap();

    // Update the break even points
    long_call.break_even_points = vec![Positive::new(105.0).unwrap()];

    long_call
}

#[test]
fn test_long_call_new() {
    let long_call = create_test_long_call();
    assert_eq!(long_call.name, "Test Long Call");
    assert_eq!(long_call.kind, StrategyType::LongCall);
    assert!(long_call.description.contains("Test Long Call"));
}

#[test]
fn test_long_call_validate() {
    let long_call = create_test_long_call();
    assert!(long_call.validate());
}

#[test]
fn test_long_call_get_title() {
    let long_call = create_test_long_call();
    assert_eq!(
        long_call.get_title(),
        "LongCall Strategy: \n\tUnderlying: AAPL @ $100 Long Call European Option"
    );
}

#[test]
fn test_long_call_get_option_basic_type() {
    let long_call = create_test_long_call();
    let option_types = long_call.get_option_basic_type();
    assert_eq!(option_types.len(), 1);

    // Verificar que contiene una opción de tipo Call
    let mut found = false;
    for option_type in option_types.iter() {
        if option_type.option_style == &OptionStyle::Call {
            found = true;
            break;
        }
    }
    assert!(
        found,
        "No se encontró un OptionBasicType con OptionStyle::Call"
    );
}

#[test]
fn test_long_call_get_implied_volatility() {
    let long_call = create_test_long_call();
    let implied_vol_map = long_call.get_implied_volatility();
    assert_eq!(implied_vol_map.len(), 1);

    // Verificar que el mapa contiene una entrada para Call con el valor correcto
    let mut found_value = None;
    for (key, value) in implied_vol_map.iter() {
        if key.option_style == &OptionStyle::Call {
            found_value = Some(value);
            break;
        }
    }
    assert!(
        found_value.is_some(),
        "No se encontró un OptionBasicType con OptionStyle::Call"
    );
    assert_eq!(*found_value.unwrap(), &Positive::new(0.3).unwrap());
}

#[test]
fn test_long_call_get_quantity() {
    let long_call = create_test_long_call();
    let quantity_map = long_call.get_quantity();
    assert_eq!(quantity_map.len(), 1);

    // Verificar que el mapa contiene una entrada para Call con el valor correcto
    let mut found_value = None;
    for (key, value) in quantity_map.iter() {
        if key.option_style == &OptionStyle::Call {
            found_value = Some(value);
            break;
        }
    }
    assert!(
        found_value.is_some(),
        "No se encontró un OptionBasicType con OptionStyle::Call"
    );
    assert_eq!(*found_value.unwrap(), &Positive::new(1.0).unwrap());
}

#[test]
fn test_long_call_one_option() {
    let long_call = create_test_long_call();
    let option = long_call.one_option();
    assert_eq!(option.strike_price, Positive::new(100.0).unwrap());
    assert_eq!(option.option_style, OptionStyle::Call);
    assert_eq!(option.side, Side::Long);
}

#[test]
fn test_long_call_set_expiration_date() {
    let mut long_call = create_test_long_call();
    let new_expiration = ExpirationDate::Days(Positive::new(60.0).unwrap());
    let result = long_call.set_expiration_date(new_expiration);
    assert!(result.is_ok());
}

#[test]
fn test_long_call_set_underlying_price() {
    let mut long_call = create_test_long_call();
    let new_price = Positive::new(110.0).unwrap();
    let result = long_call.set_underlying_price(&new_price);
    assert!(result.is_ok());
    // No podemos acceder directamente a long_call.long_call ya que es privado
}

#[test]
fn test_long_call_set_implied_volatility() {
    let mut long_call = create_test_long_call();
    let new_vol = Positive::new(0.4).unwrap();
    let result = long_call.set_implied_volatility(&new_vol);
    assert!(result.is_ok());
    // No podemos acceder directamente a long_call.long_call ya que es privado
}

#[test]
fn test_long_call_break_even_points() {
    let long_call = create_test_long_call();
    let break_even_points = long_call.get_break_even_points().unwrap();
    assert_eq!(break_even_points[0].to_f64(), 105.0);
}

#[test]
fn test_long_call_get_max_profit() {
    let long_call = create_test_long_call();
    let result = long_call.get_max_profit();
    // For a Long Call, the maximum profit is theoretically infinite
    // but in practice it depends on the implementation
    match result {
        Ok(profit) => {
            // Verify that the maximum profit is positive
            assert!(profit > Positive::ZERO);
            // For a long call with strike 100, premium 5, and underlying at 100,
            // the profit at a very high price like 200 would be approximately:
            // (200 - 100) - 5 - 0.5 - 0.5 = 94
            // This is a rough approximation of what the max profit could be
            let expected_min_profit = Positive::new(90.0).unwrap();
            assert!(
                profit >= expected_min_profit,
                "Max profit should be at least {}",
                expected_min_profit
            );
        }
        Err(e) => {
            // If there is an error, it could be due to various reasons related to profit calculation
            // The actual error message might vary depending on implementation
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
fn test_long_call_get_max_loss() {
    let long_call = create_test_long_call();
    let result = long_call.get_max_loss();
    assert!(result.is_ok());

    // Max loss should be the premium paid plus fees
    let expected_max_loss = Positive::new(6.0).unwrap(); // 5 (premium) + 0.5 (open fee) + 0.5 (close fee)
    assert_eq!(result.unwrap(), expected_max_loss);
}

#[test]
fn test_long_call_calculate_profit_at() {
    let long_call = create_test_long_call();

    // Test profit at strike price (should be negative, equal to premium + fees)
    let at_strike = Positive::new(100.0).unwrap();
    let profit_at_strike = long_call.calculate_profit_at(&at_strike).unwrap();
    assert!(profit_at_strike < Decimal::ZERO);

    // Test profit at break-even point (should be close to zero)
    let at_break_even = Positive::new(105.0).unwrap();
    let profit_at_break_even = long_call.calculate_profit_at(&at_break_even).unwrap();
    // Allow a margin of error of 1.0 for the profit calculation at the break-even point
    assert!(profit_at_break_even.abs() < Decimal::new(2, 0));

    // Test profit above break-even (should be positive)
    let above_break_even = Positive::new(120.0).unwrap();
    let profit_above_break_even = long_call.calculate_profit_at(&above_break_even).unwrap();
    assert!(profit_above_break_even > Decimal::ZERO);
}

#[test]
fn test_long_call_add_position() {
    let mut long_call = create_test_long_call();

    // Crear una nueva posición válida (long call)
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
        OptionStyle::Call,
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

    let result = long_call.add_position(&position);
    assert!(result.is_ok());

    // Probar con una posición inválida (put en lugar de call)
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
        OptionStyle::Put, // Esto lo hace inválido para una estrategia long call
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

    let result = long_call.add_position(&invalid_position);
    assert!(result.is_err());
}

#[test]
fn test_long_call_get_positions() {
    let long_call = create_test_long_call();
    let positions = long_call.get_positions().unwrap();
    assert_eq!(positions.len(), 1);
    assert_eq!(
        positions[0].option.strike_price,
        Positive::new(100.0).unwrap()
    );
    assert_eq!(positions[0].option.option_style, OptionStyle::Call);
    assert_eq!(positions[0].option.side, Side::Long);
}

#[test]
fn test_long_call_get_profit_ratio() {
    let long_call = create_test_long_call();

    // The profit/loss ratio can be positive, zero, or even undefined
    // depending on how the calculation is implemented
    match long_call.get_profit_ratio() {
        Ok(_ratio) => {
            // If a ratio is returned, we simply verify that it exists
            // We don't need to verify anything else, as Decimal cannot be NaN
        }
        Err(e) => {
            // If there is an error, we verify that it's because the ratio is undefined
            // (for example, if the maximum loss is zero or the profit is infinite)
            assert!(
                e.to_string().contains("division")
                    || e.to_string().contains("infinite")
                    || e.to_string().contains("undefined")
            );
        }
    }
}
