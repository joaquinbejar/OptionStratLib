//! A downstream analytics workflow on the `optionstratlib` facade with
//! `default-features = false, features = ["analytics"]` (#535): the
//! convenience `prelude` for the common path, the canonical module paths for
//! the rest, and proof that both name the items the analytics crate defines.
//! No strategy, simulation, I/O, async or charting code is compiled.

use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::metrics::ImpliedVolatilityCurve;
use optionstratlib::model::Position;
use optionstratlib::prelude::*;

/// Compiles only when both arguments have the same type. Every function item
/// has its own type, so this proves a facade path re-exports the component's
/// function rather than wrapping it.
fn same_item<T>(_: T, _: T) {}

/// Compiles only when the facade's `SPANMargin` is the analytics crate's.
fn analytics_span(
    span: optionstratlib_analytics::risk::SPANMargin,
) -> optionstratlib_analytics::risk::SPANMargin {
    span
}

fn short_put() -> Position {
    let option = Options::new(
        OptionType::European,
        Side::Short,
        "XYZ".to_string(),
        pos_or_panic!(95.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        OptionStyle::Put,
        Positive::ZERO,
        None,
    );
    Position::new(
        option,
        pos_or_panic!(1.5),
        Utc::now(),
        Positive::ZERO,
        Positive::ZERO,
        None,
        None,
    )
}

fn chain() -> OptionChain {
    let params = OptionChainBuildParams::new(
        "XYZ".to_string(),
        None,
        10,
        spos!(5.0),
        dec!(-0.2),
        dec!(0.1),
        pos_or_panic!(0.02),
        2,
        OptionDataPriceParams::new(
            Some(Box::new(Positive::HUNDRED)),
            Some(ExpirationDate::Days(pos_or_panic!(30.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            Some("XYZ".to_string()),
        ),
        pos_or_panic!(0.20),
    );
    match OptionChain::build_chain(&params) {
        Ok(chain) => chain,
        Err(error) => panic!("build_chain: {error}"),
    }
}

#[test]
fn test_pnl_through_the_prelude() {
    // Short 95 put sold for 1.5: keeps the premium above the strike and loses
    // the intrinsic value below it.
    let position = short_put();
    for (spot, expected) in [
        (Positive::HUNDRED, dec!(1.5)),
        (pos_or_panic!(90.0), dec!(-3.5)),
    ] {
        match position.calculate_pnl_at_expiration(&spot) {
            Ok(pnl) => assert_eq!(pnl.realized, Some(expected), "spot {spot}"),
            Err(error) => panic!("P&L at expiration, spot {spot}: {error}"),
        }
    }
}

#[test]
fn test_chain_metric_through_the_prelude() {
    let chain = chain();
    match chain.iv_curve() {
        Ok(curve) => {
            assert_eq!(curve.points.len(), chain.options.len());
            let atm = curve.points.iter().find(|point| point.x == dec!(100));
            assert_eq!(atm.map(|point| point.y), Some(dec!(0.2)));
        }
        Err(error) => panic!("implied volatility curve: {error}"),
    }
}

#[test]
fn test_span_margin_through_the_risk_module() {
    let span = optionstratlib::risk::SPANMargin::new(dec!(0.1), dec!(0.05), dec!(0.1));
    match span.calculate_margin(&short_put()) {
        Ok(margin) => assert_eq!(margin, dec!(10)),
        Err(error) => panic!("SPAN margin: {error}"),
    }
}

#[test]
fn test_facade_paths_are_the_analytics_items() {
    same_item(
        optionstratlib::analytics::probability::calculate_single_point_probability,
        optionstratlib_analytics::analytics::probability::calculate_single_point_probability,
    );
    same_item(
        <Position as optionstratlib::pnl::PnLCalculator>::calculate_pnl_at_expiration,
        <Position as optionstratlib_analytics::pnl::PnLCalculator>::calculate_pnl_at_expiration,
    );
    // `metrics` too, through the facade's `OptionChain`, which ties the
    // market type to the analytics trait implemented on it.
    same_item(
        <OptionChain as optionstratlib::metrics::ImpliedVolatilityCurve>::iv_curve,
        <OptionChain as optionstratlib_analytics::metrics::ImpliedVolatilityCurve>::iv_curve,
    );
    let span = analytics_span(optionstratlib::risk::SPANMargin::new(
        dec!(0.1),
        dec!(0.05),
        dec!(0.1),
    ));
    match span.calculate_margin(&short_put()) {
        Ok(margin) => assert_eq!(margin, dec!(10)),
        Err(error) => panic!("SPAN margin: {error}"),
    }
    let error: optionstratlib_analytics::error::ProbabilityError =
        optionstratlib::error::ProbabilityError::invalid_expiration("probe");
    assert!(error.to_string().contains("probe"));
}
