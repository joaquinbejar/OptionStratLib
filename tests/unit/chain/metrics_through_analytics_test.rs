/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/9/26
******************************************************************************/

//! Analytics-plus-market fixture: evaluates one metric curve and one RND call
//! on a small chain, importing the metric trait from `metrics` and the RND
//! trait from `analytics` rather than through the legacy `chains` paths.
//! This is the shape of the cross-crate test that survives the extraction of
//! `optionstratlib-analytics` (ADR-0001 D4, D5; M1-12).

use optionstratlib::analytics::{RNDAnalysis, RNDParameters};
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::metrics::ImpliedVolatilityCurve;
use positive::{Positive, pos_or_panic, spos};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Five strikes around a 100 spot with a flat 17% implied volatility and
/// call asks consistent with a short-dated Black-Scholes price.
fn create_small_chain() -> OptionChain {
    let mut chain = OptionChain::new(
        "TEST",
        Positive::HUNDRED,
        "2030-01-01".to_string(),
        None,
        None,
    );
    let strikes = [90.0, 95.0, 100.0, 105.0, 110.0];
    let call_asks = [10.04, 5.37, 1.95, 0.43, 0.06];
    for (&strike, &call_ask) in strikes.iter().zip(call_asks.iter()) {
        chain.add_option(
            pos_or_panic!(strike),
            spos!(call_ask - 0.02),
            spos!(call_ask),
            None,
            None,
            pos_or_panic!(0.17),
            None,
            None,
            None,
            None,
            None,
            None,
        );
    }
    chain
}

#[test]
fn test_iv_curve_through_metrics_trait_has_one_point_per_strike() {
    let chain = create_small_chain();
    let curve = chain.iv_curve().expect("iv curve on a populated chain");
    assert_eq!(curve.points.len(), 5);
    let points: Vec<_> = curve.points.iter().collect();
    assert_eq!(points[0].x, dec!(90.0));
    assert_eq!(points[4].x, dec!(110.0));
    assert!(points.iter().all(|p| p.y == dec!(0.17)));
}

#[test]
fn test_rnd_through_analytics_trait_normalises_densities() {
    let chain = create_small_chain();
    let params = RNDParameters {
        risk_free_rate: dec!(0.05),
        interpolation_points: 100,
        derivative_tolerance: Positive::ONE,
    };
    let result = chain
        .calculate_rnd(&params)
        .expect("rnd on a populated chain");
    assert!(!result.densities.is_empty());
    let total: Decimal = result.densities.values().sum();
    assert!((total - Decimal::ONE).abs() < dec!(1e-10));
}

#[test]
fn test_skew_through_analytics_trait_is_relative_to_spot() {
    let chain = create_small_chain();
    let skew = chain.calculate_skew().expect("skew on a populated chain");
    assert_eq!(skew.len(), 5);
    let atm = skew
        .iter()
        .find(|(relative_strike, _)| *relative_strike == Positive::ONE)
        .expect("atm strike present");
    assert_eq!(atm.1, Decimal::ZERO);
}
