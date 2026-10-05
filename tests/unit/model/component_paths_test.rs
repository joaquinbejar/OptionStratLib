//! Facade paths and direct-component paths name the same types (#520).
//!
//! Each function takes a type from its defining component crate and is
//! called with a value obtained through the `optionstratlib` facade (or the
//! other way round). A facade wrapper or copy would fail to compile here.

use optionstratlib::prelude::pos_or_panic;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn core_options(value: optionstratlib_core::model::Options) -> optionstratlib_core::model::Options {
    value
}

fn core_position(
    value: optionstratlib_core::model::Position,
) -> optionstratlib_core::model::Position {
    value
}

fn math_curve(value: optionstratlib_math::curves::Curve) -> optionstratlib_math::curves::Curve {
    value
}

fn math_surface(
    value: optionstratlib_math::surfaces::Surface,
) -> optionstratlib_math::surfaces::Surface {
    value
}

fn facade_option() -> optionstratlib::Options {
    optionstratlib::Options::new(
        optionstratlib::OptionType::European,
        optionstratlib::Side::Long,
        "XYZ".to_string(),
        pos_or_panic!(100.0),
        optionstratlib::ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        optionstratlib::prelude::Positive::ONE,
        pos_or_panic!(100.0),
        dec!(0.05),
        optionstratlib::OptionStyle::Call,
        optionstratlib::prelude::Positive::ZERO,
        None,
    )
}

#[test]
fn test_core_types_through_facade_root_model_and_prelude() {
    let through_root = facade_option();
    let through_core = core_options(through_root.clone());
    let back: optionstratlib::model::Options = through_core;
    let from_prelude: optionstratlib::prelude::Options = back;
    assert_eq!(from_prelude, through_root);

    let position = optionstratlib::model::Position::new(
        from_prelude,
        pos_or_panic!(2.5),
        chrono::Utc::now(),
        optionstratlib::prelude::Positive::ONE,
        optionstratlib::prelude::Positive::ONE,
        None,
        None,
    );
    let position: optionstratlib::prelude::Position = core_position(position);
    assert_eq!(position.option.strike_price, pos_or_panic!(100.0));
}

#[test]
fn test_core_errors_through_facade_error() {
    let from_core: optionstratlib_core::error::DecimalError =
        optionstratlib_core::error::DecimalError::invalid_value(-1.0, "probe");
    let through_facade: optionstratlib::error::DecimalError = from_core;
    let _: optionstratlib_core::error::PositionError =
        optionstratlib::error::PositionError::invalid_position_size(-1.0, "probe");
    assert!(through_facade.to_string().contains("probe"));
}

#[test]
fn test_math_types_through_facade_modules_and_prelude() {
    use optionstratlib::curves::{Curve, Point2D};
    use optionstratlib::geometrics::GeometricObject;
    use optionstratlib::surfaces::{Point3D, Surface};

    let curve = math_curve(Curve::from_vector(vec![
        Point2D::new(Decimal::ZERO, Decimal::ZERO),
        Point2D::new(Decimal::ONE, Decimal::ONE),
    ]));
    let from_prelude: optionstratlib::prelude::Curve = curve;
    assert_eq!(from_prelude.points.len(), 2);

    let surface = math_surface(Surface::from_vector(vec![Point3D::new(
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
    )]));
    let from_prelude: optionstratlib::prelude::Surface = surface;
    assert_eq!(from_prelude.points.len(), 1);

    let _: optionstratlib_math::geometrics::InterpolationType =
        optionstratlib::geometrics::InterpolationType::Linear;
    let _: optionstratlib_math::error::CurveError =
        optionstratlib::error::CurveError::ConstructionError("probe".to_string());
}

#[test]
fn test_core_modules_root_types_and_macros_through_facade() {
    let _: optionstratlib_core::utils::TimeFrame = optionstratlib::utils::TimeFrame::Day;
    let _: optionstratlib_core::model::ExpirationDate =
        optionstratlib::ExpirationDate::Days(pos_or_panic!(1.0));
    assert_eq!(
        optionstratlib::constants::TOLERANCE,
        optionstratlib_core::constants::TOLERANCE
    );
    let through_facade_macro = optionstratlib::f2du!(1.5);
    let through_core_macro = optionstratlib_core::f2du!(1.5);
    assert!(matches!(
        (through_facade_macro, through_core_macro),
        (Ok(a), Ok(b)) if a == b && a == dec!(1.5)
    ));
}
