/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! Integration tests for price metrics implementations on OptionChain.
//!
//! These tests verify the correct behavior of:
//! - VolatilitySkewCurve
//! - PutCallRatioCurve
//! - StrikeConcentrationCurve

use optionstratlib::chains::OptionData;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::metrics::{PutCallRatioCurve, VolatilitySkewCurve};
use optionstratlib::model::ExpirationDate;
use positive::{pos_or_panic, spos};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Creates a test option chain with realistic data for testing price metrics.
fn create_test_chain() -> OptionChain {
    let params = OptionChainBuildParams::new(
        "TEST".to_string(),
        None,
        10,         // chain_size
        spos!(5.0), // strike_interval
        dec!(-0.2),
        dec!(0.1),
        pos_or_panic!(0.02), // spread
        2,                   // decimal_places
        OptionDataPriceParams::new(
            Some(Box::new(pos_or_panic!(100.0))), // underlying_price
            Some(ExpirationDate::Days(pos_or_panic!(30.0))), // expiration_date
            Some(dec!(0.05)),                     // risk_free_rate
            spos!(0.02),                          // dividend_yield
            Some("TEST".to_string()),
        ),
        pos_or_panic!(0.20), // implied_volatility
    );

    OptionChain::build_chain(&params).unwrap()
}

/// Creates an empty option chain for edge case testing.
fn create_empty_chain() -> OptionChain {
    OptionChain::new(
        "EMPTY",
        pos_or_panic!(100.0),
        "2024-12-31".to_string(),
        None,
        None,
    )
}

/// Creates a chain without bid/ask midprice data
fn create_chain_without_bid_ask_mid() -> OptionChain {
    let mut chain = OptionChain::new(
        "NOLIQ",
        pos_or_panic!(450.0),
        "2024-12-31".to_string(),
        None,
        None,
    );

    // Add option without bid/ask midprice, volume, or open interest
    let option_data = OptionData::new(
        pos_or_panic!(450.0),
        None,
        None,
        None,
        None,
        pos_or_panic!(0.20),
        None,
        None,
        None,
        None,
        None,
        Some("NOLIQ".to_string()),
        Some(ExpirationDate::Days(pos_or_panic!(30.0))),
        Some(Box::new(pos_or_panic!(450.0))),
        Some(dec!(0.05)),
        None,
        None,
        None,
    );
    chain.options.insert(option_data);

    chain
}

#[cfg(test)]
mod tests_volatility_skew_curve {
    use super::*;

    #[test]
    fn test_volatility_skew_curve_basic() {
        let chain = create_test_chain();
        let curve = chain.volatility_skew().unwrap();

        // Chain should have multiple points with valid Volatility Skew
        assert!(
            !curve.points.is_empty(),
            "Volatility Skew curve should have at least one point"
        );
    }

    #[test]
    fn test_volatility_skew_curve_values_positive() {
        let chain = create_test_chain();
        let curve = chain.volatility_skew().unwrap();

        // All IV values should be positive
        for point in curve.points.iter() {
            assert!(
                point.y > Decimal::ZERO,
                "Implied Volatility should be positive at strike {}",
                point.x
            );
        }
    }
}

#[cfg(test)]
mod tests_put_call_ratio_curve {
    use super::*;

    #[test]
    fn test_put_call_ratio_premium_weighted_curve_basic() {
        let chain = create_test_chain();
        let curve = chain.premium_weighted_pcr().unwrap();

        // Chain should have multiple points with valid Volatility Skew
        assert!(
            !curve.points.is_empty(),
            "Premium weighted Put/Call ratio curve should have at least one point"
        );
    }

    #[test]
    fn test_put_call_ratio_premium_weighted_curve_values_positive() {
        let chain = create_test_chain();
        let curve = chain.premium_weighted_pcr().unwrap();

        // All PCR values should be positive
        for point in curve.points.iter() {
            assert!(
                point.y > Decimal::ZERO,
                "Premium weighted Put/Call ratio should be positive at strike {}",
                point.x
            );
        }
    }
    #[test]
    fn test_put_call_ratio_premium_weighted_curve_no_midprices() {
        let chain = create_chain_without_bid_ask_mid();
        let result = chain.premium_weighted_pcr();

        assert!(result.is_err());
    }

    #[test]
    fn test_put_call_ratio_premium_weighted_curve_strike_ordering() {
        let chain = create_test_chain();
        let curve = chain.premium_weighted_pcr().unwrap();

        let points: Vec<_> = curve.points.iter().collect();

        // Points should be ordered by strike
        for i in 1..points.len() {
            assert!(points[i].x > points[i - 1].x);
        }
    }
}

#[cfg(test)]
mod tests_strike_concentration_curve {
    use optionstratlib::prelude::StrikeConcentrationCurve;

    use super::*;

    #[test]
    fn test_strike_concentration_premium_weighted_curve_basic() {
        let chain = create_test_chain();
        let curve = chain.premium_concentration().unwrap();

        // Chain should have multiple points with valid Strike Concentration
        assert!(
            !curve.points.is_empty(),
            "Premium weighted Strike Concentration curve should have at least one point"
        );
    }

    #[test]
    fn test_strike_concentration_premium_weighted_curve_values_positive() {
        let chain = create_test_chain();
        let curve = chain.premium_concentration().unwrap();

        // All Strike Concentration values should be positive
        for point in curve.points.iter() {
            assert!(
                point.y > Decimal::ZERO,
                "Premium weighted Strike Concentration should be positive at strike {}",
                point.x
            );
        }
    }
    #[test]
    fn test_strike_concentration_premium_weighted_curve_no_midprices() {
        let chain = create_chain_without_bid_ask_mid();
        let result = chain.premium_concentration();

        assert!(result.is_err());
    }

    #[test]
    fn test_strike_concentration_premium_weighted_curve_strike_ordering() {
        let chain = create_test_chain();
        let curve = chain.premium_concentration().unwrap();

        let points: Vec<_> = curve.points.iter().collect();

        // Points should be ordered by strike
        for i in 1..points.len() {
            assert!(points[i].x > points[i - 1].x);
        }
    }
}

#[cfg(test)]
mod tests_edge_cases {
    use optionstratlib::prelude::StrikeConcentrationCurve;

    use super::*;

    #[test]
    fn test_chain_volatility_skew_with_single_strike() {
        // Create a minimal chain with just one option
        let mut chain = create_empty_chain();

        // Add a single option data point
        use optionstratlib::chains::OptionData;
        let option_data = OptionData::new(
            pos_or_panic!(100.0),
            spos!(5.0),
            spos!(5.5),
            spos!(4.5),
            spos!(5.0),
            pos_or_panic!(0.20),
            Some(dec!(0.5)),
            Some(dec!(-0.5)),
            Some(dec!(0.05)),
            spos!(1000.0),
            Some(5000),
            Some("TEST".to_string()),
            Some(ExpirationDate::Days(pos_or_panic!(30.0))),
            Some(Box::new(pos_or_panic!(100.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            None,
            None,
        );
        chain.options.insert(option_data);

        // Volatility Skew should work with a single point
        let volatility_skew_curve = chain.volatility_skew().unwrap();
        assert_eq!(volatility_skew_curve.points.len(), 1);
    }

    #[test]
    fn test_chain_put_call_ratio_premium_weighted_with_single_strike() {
        // Create a minimal chain with just one option
        let mut chain = create_empty_chain();

        // Add a single option data point
        use optionstratlib::chains::OptionData;
        let mut option_data = OptionData::new(
            pos_or_panic!(100.0),
            spos!(5.0),
            spos!(5.5),
            spos!(4.5),
            spos!(5.0),
            pos_or_panic!(0.20),
            Some(dec!(0.5)),
            Some(dec!(-0.5)),
            Some(dec!(0.05)),
            spos!(1000.0),
            Some(5000),
            Some("TEST".to_string()),
            Some(ExpirationDate::Days(pos_or_panic!(30.0))),
            Some(Box::new(pos_or_panic!(100.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            None,
            None,
        );
        option_data.set_mid_prices(); // calculate bid / ask mid prices
        chain.options.insert(option_data);

        // Premium weighted Put/Call ratio should work with a single point
        let pcr_curve = chain.premium_weighted_pcr().unwrap();
        assert_eq!(pcr_curve.points.len(), 1);
    }

    #[test]
    fn test_chain_strike_concentration_premium_weighted_with_single_strike() {
        // Create a minimal chain with just one option
        let mut chain = create_empty_chain();

        // Add a single option data point
        use optionstratlib::chains::OptionData;
        let mut option_data = OptionData::new(
            pos_or_panic!(100.0),
            spos!(5.0),
            spos!(5.5),
            spos!(4.5),
            spos!(5.0),
            pos_or_panic!(0.20),
            Some(dec!(0.5)),
            Some(dec!(-0.5)),
            Some(dec!(0.05)),
            spos!(1000.0),
            Some(5000),
            Some("TEST".to_string()),
            Some(ExpirationDate::Days(pos_or_panic!(30.0))),
            Some(Box::new(pos_or_panic!(100.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            None,
            None,
        );
        option_data.set_mid_prices(); // calculate bid / ask mid prices
        chain.options.insert(option_data);

        // Premium weighted Strike Concentration should work with a single point
        let strike_concentration_curve = chain.premium_concentration().unwrap();
        assert_eq!(strike_concentration_curve.points.len(), 1);
    }
}
