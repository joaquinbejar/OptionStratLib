//! A downstream analytics workflow built on core, math, pricing, market and
//! analytics alone (#533): P&L of a position at expiration, SPAN margin, the neutral
//! probability kernels, the risk-neutral density and skew of a chain, and a
//! chain metric. Every value is a canonical lower-layer type; no strategy
//! type, trait or crate is involved.

use chrono::Utc;
use optionstratlib_analytics::analytics::probability::{
    VolatilityAdjustment, calculate_price_probability, calculate_single_point_probability,
};
use optionstratlib_analytics::analytics::{RNDAnalysis, RNDParameters};
use optionstratlib_analytics::metrics::ImpliedVolatilityCurve;
use optionstratlib_analytics::pnl::PnLCalculator;
use optionstratlib_analytics::risk::SPANMargin;
use optionstratlib_core::model::{
    ExpirationDate, OptionStyle, OptionType, Options, Position, Positive, Side,
};
use optionstratlib_core::{pos_or_panic, spos};
use optionstratlib_market::chains::chain::OptionChain;
use optionstratlib_market::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// How far two evaluations of the same lognormal tail may differ: both
/// kernels compute it through the same normal CDF, so only Decimal rounding
/// separates them.
const SAME_TAIL_TOLERANCE: Decimal = dec!(0.000000001);

/// How far the risk-neutral density may sum from one.
const DENSITY_SUM_TOLERANCE: Decimal = dec!(0.000000001);

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
fn test_pnl_of_a_short_put_at_expiration() {
    // Short 95 put sold for 1.5: keeps the premium above the strike and
    // loses the intrinsic value below it.
    let position = short_put();
    let cases = [
        (Positive::HUNDRED, dec!(1.5)),
        (pos_or_panic!(90.0), dec!(-3.5)),
        (pos_or_panic!(80.0), dec!(-13.5)),
    ];
    for (spot, expected) in cases {
        match position.calculate_pnl_at_expiration(&spot) {
            Ok(pnl) => assert_eq!(pnl.realized, Some(expected), "spot {spot}"),
            Err(error) => panic!("P&L at expiration, spot {spot}: {error}"),
        }
    }
}

#[test]
fn test_span_margin_of_a_short_put() {
    // With a 10% short-option minimum the floor (0.1 x 100 x 1) decides.
    let floor_bound = SPANMargin::new(dec!(0.1), dec!(0.05), dec!(0.1));
    match floor_bound.calculate_margin(&short_put()) {
        Ok(margin) => assert_eq!(margin, dec!(10)),
        Err(error) => panic!("SPAN margin: {error}"),
    }
    // With a 1% minimum the scanned scenario losses decide, above the floor.
    let scenario_bound = SPANMargin::new(dec!(0.01), dec!(0.05), dec!(0.1));
    match scenario_bound.calculate_margin(&short_put()) {
        Ok(margin) => assert!(
            margin > Decimal::ONE,
            "scenario losses set the margin: {margin}"
        ),
        Err(error) => panic!("SPAN margin: {error}"),
    }
}

#[test]
fn test_neutral_probability_kernels_agree() {
    // The range kernel's tails are the single-point kernel's tails at each
    // bound, whatever drift convention both share.
    let volatility = VolatilityAdjustment {
        base_volatility: pos_or_panic!(0.2),
        std_dev_adjustment: Positive::ZERO,
    };
    let expiry = ExpirationDate::Days(pos_or_panic!(30.0));
    let rate = Some(dec!(0.05));
    let tails = |target: f64| match calculate_single_point_probability(
        &Positive::HUNDRED,
        &pos_or_panic!(target),
        volatility,
        None,
        &expiry,
        rate,
    ) {
        Ok((below, above)) => (below.to_dec(), above.to_dec()),
        Err(error) => panic!("single point probability at {target}: {error}"),
    };
    let (below_95, _) = tails(95.0);
    let (below_105, above_105) = tails(105.0);
    assert!(
        below_105 > above_105,
        "a 5% higher target is more likely missed"
    );
    match calculate_price_probability(
        &Positive::HUNDRED,
        &pos_or_panic!(95.0),
        &pos_or_panic!(105.0),
        volatility,
        None,
        &expiry,
        rate,
    ) {
        Ok((below, inside, above)) => {
            assert!((below.to_dec() - below_95).abs() < SAME_TAIL_TOLERANCE);
            assert!((above.to_dec() - above_105).abs() < SAME_TAIL_TOLERANCE);
            assert!(inside > below && inside > above);
        }
        Err(error) => panic!("range probability: {error}"),
    }
}

#[test]
fn test_risk_neutral_density_and_skew_of_a_chain() {
    let chain = chain();
    let params = RNDParameters {
        risk_free_rate: dec!(0.05),
        derivative_tolerance: pos_or_panic!(0.1),
    };
    match chain.calculate_rnd(&params) {
        Ok(result) => {
            assert!(!result.densities.is_empty());
            assert!(
                result
                    .densities
                    .values()
                    .all(|density| *density >= Decimal::ZERO)
            );
            let total: Decimal = result.densities.values().sum();
            assert!(
                (total - Decimal::ONE).abs() < DENSITY_SUM_TOLERANCE,
                "{total}"
            );
            let mean = result.statistics.mean;
            assert!(mean > dec!(95) && mean < dec!(105), "mean {mean}");
        }
        Err(error) => panic!("risk-neutral density: {error}"),
    }
    // The chain is built with a negative skew slope: implied volatility
    // falls as moneyness rises, and the skew is zero at the money.
    match chain.calculate_skew() {
        Ok(skew) => {
            assert!(skew.windows(2).all(|pair| match pair {
                [lower, higher] => higher.1 < lower.1,
                _ => true,
            }));
            assert!(
                skew.iter()
                    .any(|(moneyness, value)| *moneyness == Positive::ONE && value.is_zero())
            );
        }
        Err(error) => panic!("skew: {error}"),
    }
}

#[test]
fn test_implied_volatility_curve_of_a_chain() {
    let chain = chain();
    match chain.iv_curve() {
        Ok(curve) => {
            assert_eq!(curve.points.len(), chain.options.len());
            let iv_at = |strike: Decimal| {
                curve
                    .points
                    .iter()
                    .find(|point| point.x == strike)
                    .map(|point| point.y)
            };
            assert_eq!(iv_at(dec!(100)), Some(dec!(0.2)));
            match (iv_at(dec!(80)), iv_at(dec!(100)), iv_at(dec!(120))) {
                (Some(low), Some(atm), Some(high)) => assert!(low > atm && atm > high),
                other => panic!("missing strikes on the curve: {other:?}"),
            }
        }
        Err(error) => panic!("implied volatility curve: {error}"),
    }
}
