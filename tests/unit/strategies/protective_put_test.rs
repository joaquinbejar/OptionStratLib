use approx::assert_relative_eq;
use chrono::Utc;
use num_traits::ToPrimitive;
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
        protective_put::ProtectivePut,
    },
};
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn create_test_protective_put() -> ProtectivePut {
    ProtectivePut::new(
        "AAPL".to_string(),
        Positive::new(150.0).unwrap(),
        Positive::new(145.0).unwrap(),
        ExpirationDate::Days(Positive::new(30.0).unwrap()),
        Positive::new(0.25).unwrap(),
        dec!(0.05),
        Positive::new(0.01).unwrap(),
        Positive::new(100.0).unwrap(),
        Positive::new(3.50).unwrap(),
        Positive::new(1.0).unwrap(),
        Positive::new(1.0).unwrap(),
        Positive::new(0.65).unwrap(),
        Positive::new(0.65).unwrap(),
    )
}

#[test]
fn test_protective_put_new() {
    let pp = create_test_protective_put();
    assert_eq!(pp.name, "ProtectivePut_AAPL");
    assert_eq!(pp.kind, StrategyType::ProtectivePut);
    assert_eq!(pp.spot_leg.symbol, "AAPL");
    assert_eq!(pp.spot_leg.quantity, Positive::new(100.0).unwrap());
}

#[test]
fn test_protective_put_validate() {
    let pp = create_test_protective_put();
    assert!(pp.validate());
}

#[test]
fn test_protective_put_get_title() {
    let pp = create_test_protective_put();
    let title = pp.get_title();
    assert!(title.contains("Protective Put Strategy"));
    assert!(title.contains("AAPL"));
}

#[test]
fn test_protective_put_break_even_points() {
    let pp = create_test_protective_put();
    let break_even_points = pp.get_break_even_points().unwrap();
    assert_eq!(break_even_points.len(), 1);
    // Break-even = 150 + 3.5 + (1+1+0.65+0.65)/100 = 153.5 + 0.033 = 153.533
    assert!((break_even_points[0].to_f64() - 153.53).abs() < 0.01);
}

#[test]
fn test_protective_put_max_profit() {
    let pp = create_test_protective_put();
    let result = pp.get_max_profit();
    assert!(result.is_ok());
    assert!(result.unwrap() > Positive::new(1000.0).unwrap()); // Practically unlimited
}

#[test]
fn test_protective_put_max_loss() {
    let pp = create_test_protective_put();
    let result = pp.get_max_loss();
    assert!(result.is_ok());
    // Max loss when price <= strike (145)
    // Loss = (150 - 145) * 100 + 3.5 * 100 + fees = 500 + 350 + 3.3 = 853.3
    assert!((result.unwrap().to_f64() - 853.3).abs() < 0.1);
}

#[test]
fn test_protective_put_calculate_profit_at() {
    let pp = create_test_protective_put();

    // Price at 100 (below strike 145)
    let price_100 = Positive::new(100.0).unwrap();
    let profit_100 = pp.calculate_profit_at(&price_100).unwrap();
    // Should be max loss
    assert_relative_eq!(profit_100.to_f64().unwrap(), -853.3, epsilon = 0.1);

    // Price at 153.53 (break-even)
    let price_be = Positive::new(153.53).unwrap();
    let profit_be = pp.calculate_profit_at(&price_be).unwrap();
    assert!(profit_be.abs() < dec!(1.0));

    // Price at 200 (well above)
    let price_200 = Positive::new(200.0).unwrap();
    let profit_200 = pp.calculate_profit_at(&price_200).unwrap();
    // Profit = (200 - 150) * 100 - 3.5 * 100 - fees = 5000 - 350 - 3.3 = 4646.7
    assert_relative_eq!(profit_200.to_f64().unwrap(), 4646.7, epsilon = 0.1);
}

#[test]
fn test_protective_put_add_position() {
    let mut pp = create_test_protective_put();

    let option = Options::new(
        OptionType::European,
        Side::Long,
        "AAPL".to_string(),
        Positive::new(140.0).unwrap(),
        ExpirationDate::Days(Positive::new(30.0).unwrap()),
        Positive::new(0.25).unwrap(),
        Positive::new(1.0).unwrap(),
        Positive::new(150.0).unwrap(),
        dec!(0.05),
        OptionStyle::Put,
        Positive::new(0.01).unwrap(),
        None,
    );

    let position = Position::new(
        option,
        Positive::new(2.50).unwrap(),
        Utc::now(),
        Positive::new(0.5).unwrap(),
        Positive::new(0.5).unwrap(),
        None,
        None,
    );

    let result = pp.add_position(&position);
    assert!(result.is_ok());
    assert_eq!(
        pp.long_put.option.strike_price,
        Positive::new(140.0).unwrap()
    );
}

#[test]
fn test_protective_put_modify_position() {
    let mut pp = create_test_protective_put();
    let mut position = pp.long_put.clone();
    position.premium = Positive::new(4.0).unwrap();

    let result = pp.modify_position(&position);
    assert!(result.is_ok());
    assert_eq!(pp.long_put.premium, Positive::new(4.0).unwrap());
}

#[test]
fn test_protective_put_get_position() {
    let mut pp = create_test_protective_put();
    let strike = pp.long_put.option.strike_price;
    let result = pp.get_position(&OptionStyle::Put, &Side::Long, &strike);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 1);
}

#[test]
fn test_protective_put_effective_cost_basis() {
    let pp = create_test_protective_put();
    let effective = pp.effective_cost_basis();
    // 150 + 3.5 = 153.5
    assert_eq!(effective, Positive::new(153.5).unwrap());
}

#[test]
fn test_protective_put_protection_level() {
    let pp = create_test_protective_put();
    let protection = pp.protection_level();
    // (150 - 145) / 150 = 5 / 150 = 0.0333... = 3.33%
    assert_relative_eq!(
        protection.to_f64().unwrap(),
        3.3333333333333335,
        epsilon = 0.0001
    );
}

#[test]
fn test_protective_put_net_delta() {
    let pp = create_test_protective_put();
    let delta = pp.net_delta().unwrap();
    // Spot delta is 100
    // Long Put delta is negative
    assert!(delta > Decimal::ZERO);
    assert!(delta < dec!(100.0));
}
