//! Values built from any path to a foundational type reach facade APIs
//! without conversion (#515).
//!
//! An `Options` is assembled from the defining crates, from
//! `optionstratlib_core` paths and from `optionstratlib` paths, and each one
//! is priced by the facade. If any path named a different type, this file
//! would not compile.

use optionstratlib::model::Options;
use optionstratlib::pricing::black_scholes;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn options_from_defining_crates() -> optionstratlib_core::model::Options {
    optionstratlib_core::model::Options::new(
        option_type::OptionType::European,
        financial_types::Side::Long,
        "XYZ".to_string(),
        positive::pos_or_panic!(100.0),
        expiration_date::ExpirationDate::Days(positive::pos_or_panic!(30.0)),
        positive::pos_or_panic!(0.2),
        positive::Positive::ONE,
        positive::pos_or_panic!(100.0),
        dec!(0.05),
        financial_types::OptionStyle::Call,
        positive::Positive::ZERO,
        None,
    )
}

fn options_from_core_paths() -> Options {
    use optionstratlib_core::model::{ExpirationDate, OptionStyle, OptionType, Positive, Side};
    use optionstratlib_core::pos_or_panic;
    Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".to_string(),
        pos_or_panic!(100.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        pos_or_panic!(100.0),
        dec!(0.05),
        OptionStyle::Call,
        Positive::ZERO,
        None,
    )
}

fn options_from_facade_paths() -> Options {
    use optionstratlib::model::Positive;
    use optionstratlib::prelude::pos_or_panic;
    use optionstratlib::{ExpirationDate, OptionStyle, OptionType, Side};
    Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".to_string(),
        pos_or_panic!(100.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        pos_or_panic!(100.0),
        dec!(0.05),
        OptionStyle::Call,
        Positive::ZERO,
        None,
    )
}

#[test]
fn test_every_path_builds_the_same_option() {
    let original = options_from_defining_crates();
    assert_eq!(original, options_from_core_paths());
    assert_eq!(original, options_from_facade_paths());
}

#[test]
fn test_facade_prices_options_built_from_any_path() {
    let prices: Vec<Decimal> = [
        options_from_defining_crates(),
        options_from_core_paths(),
        options_from_facade_paths(),
    ]
    .iter()
    .map(|option| match black_scholes(option) {
        Ok(price) => price,
        Err(error) => panic!("black_scholes failed: {error}"),
    })
    .collect();
    match prices.first() {
        Some(first) => {
            assert!(prices.iter().all(|price| price == first));
            assert!(*first > Decimal::ZERO);
        }
        None => panic!("no price computed"),
    }
}

#[test]
fn test_prelude_positive_is_the_core_positive() {
    let from_prelude: optionstratlib::prelude::Positive = positive::Positive::TWO;
    let into_core: optionstratlib_core::model::Positive = from_prelude;
    let into_facade_model: optionstratlib::model::Positive = into_core;
    assert_eq!(into_facade_model, positive::Positive::TWO);
}
