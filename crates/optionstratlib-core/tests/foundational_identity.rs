//! Compile-time proof that core re-exports the foundational types without
//! changing their identity (#515).
//!
//! Every function below takes the type from its defining crate and is called
//! with a value obtained through an `optionstratlib_core` path, or the other
//! way round. A wrapper or copied type would fail to compile here, which is
//! the check; the runtime assertions only confirm the values round-trip.

use optionstratlib_core::model::types::{
    Action, AsianAveragingType, BarrierType, BinaryType, LookbackType, OptionBasicType,
    UnderlyingAssetType,
};
use optionstratlib_core::model::{
    ExpirationDate, ExpirationDateError, OptionStyle, OptionType, Positive, PositiveError,
    RainbowType, Side,
};
use optionstratlib_core::{assert_pos_relative_eq, pos_or_panic, spos};

fn takes_positive(value: positive::Positive) -> positive::Positive {
    value
}

fn takes_expiration(value: expiration_date::ExpirationDate) -> expiration_date::ExpirationDate {
    value
}

fn takes_side(value: financial_types::Side) -> financial_types::Side {
    value
}

fn takes_style(value: financial_types::OptionStyle) -> financial_types::OptionStyle {
    value
}

fn takes_option_type(value: option_type::OptionType) -> option_type::OptionType {
    value
}

fn core_positive(value: Positive) -> Positive {
    value
}

#[test]
fn test_positive_identity_both_directions() {
    let through_core: Positive = pos_or_panic!(1.5);
    let original: positive::Positive = takes_positive(through_core);
    assert_eq!(core_positive(original), through_core);
    assert_eq!(spos!(2.0), Some(positive::Positive::TWO));
    assert_pos_relative_eq!(original, Positive::ONE + pos_or_panic!(0.5), Positive::ZERO);
}

#[test]
fn test_positive_error_identity() {
    let from_core: Result<Positive, PositiveError> = Positive::new(-1.0);
    let original: Result<positive::Positive, positive::PositiveError> = from_core;
    assert!(original.is_err());
}

#[test]
fn test_expiration_identity_both_directions() {
    let through_core = ExpirationDate::Days(pos_or_panic!(30.0));
    let original: expiration_date::ExpirationDate = takes_expiration(through_core);
    let back: ExpirationDate = original;
    assert_eq!(back, ExpirationDate::Days(pos_or_panic!(30.0)));
    let _error_type: Option<ExpirationDateError> =
        None::<expiration_date::error::ExpirationDateError>;
}

#[test]
fn test_financial_types_identity() {
    assert_eq!(takes_side(Side::Long), financial_types::Side::Long);
    assert_eq!(
        takes_style(OptionStyle::Put),
        financial_types::OptionStyle::Put
    );
    let _action: financial_types::Action = Action::Buy;
    let _asset: financial_types::UnderlyingAssetType = UnderlyingAssetType::Stock;
}

#[test]
fn test_option_type_identity() {
    assert_eq!(
        takes_option_type(OptionType::European),
        option_type::OptionType::European
    );
    let _basic: Option<option_type::OptionBasicType> = None::<OptionBasicType>;
    let _asian: Option<option_type::AsianAveragingType> = None::<AsianAveragingType>;
    let _barrier: Option<option_type::BarrierType> = None::<BarrierType>;
    let _binary: Option<option_type::BinaryType> = None::<BinaryType>;
    let _lookback: Option<option_type::LookbackType> = None::<LookbackType>;
    let _rainbow: Option<option_type::RainbowType> = None::<RainbowType>;
}

/// With `schema` on, the `ToSchema` impl a core path reaches is the one the
/// defining crate provides: same type, same schema name.
#[cfg(feature = "schema")]
#[test]
fn test_schema_feature_keeps_identity() {
    use utoipa::ToSchema;
    assert_eq!(
        <Positive as ToSchema>::name(),
        <positive::Positive as ToSchema>::name()
    );
    assert_eq!(
        <Side as ToSchema>::name(),
        <financial_types::Side as ToSchema>::name()
    );
}
