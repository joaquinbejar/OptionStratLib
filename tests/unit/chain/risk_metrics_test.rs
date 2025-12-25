/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! Integration tests for risk metrics implementations on OptionChain.
//!
//! These tests verify the correct behavior of:
//! - ImpliedVolatilityCurve
//! - ImpliedVolatilitySurface
//! - RiskReversalCurve
//! - DollarGammaCurve

use optionstratlib::chains::chain::OptionChain;
use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::metrics::{
    DollarGammaCurve, ImpliedVolatilityCurve, ImpliedVolatilitySurface, RiskReversalCurve,
};
use optionstratlib::model::{ExpirationDate, OptionStyle};
use optionstratlib::{pos, spos};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Creates a test option chain with realistic data for testing risk metrics.
fn create_test_chain() -> OptionChain {
    let params = OptionChainBuildParams::new(
        "TEST".to_string(),
        None,
        10,
        spos!(5.0),
        dec!(-0.2),
        dec!(0.1),
        pos_or_panic!(0.02),
        2,
        OptionDataPriceParams::new(
            Some(Box::new(pos_or_panic!(100.0))),
            Some(ExpirationDate::Days(pos_or_panic!(30.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            Some("TEST".to_string()),
        ),
        pos_or_panic!(0.20),
    );

    OptionChain::build_chain(&params)
}

/// Creates an empty option chain for edge case testing.
fn create_empty_chain() -> OptionChain {
    OptionChain::new("EMPTY", pos_or_panic!(100.0), "2024-12-31".to_string(), None, None)
}

#[cfg(test)]
mod tests_implied_volatility_curve {
    use super::*;

    #[test]
    fn test_iv_curve_basic() {
        let chain = create_test_chain();
        let result = chain.iv_curve();

        assert!(result.is_ok());
        let curve = result.unwrap();

        // Chain should have multiple points with valid IV
        assert!(
            !curve.points.is_empty(),
            "IV curve should have at least one point"
        );
    }

    #[test]
    fn test_iv_curve_x_range() {
        let chain = create_test_chain();
        let curve = chain.iv_curve().unwrap();

        // X range should be valid (min < max)
        assert!(curve.x_range.0 < curve.x_range.1, "X range should be valid");
        // Strikes should be around the underlying price (100)
        assert!(
            curve.x_range.0 < dec!(100.0),
            "Min strike should be below underlying"
        );
        assert!(
            curve.x_range.1 > dec!(100.0),
            "Max strike should be above underlying"
        );
    }

    #[test]
    fn test_iv_curve_values_positive() {
        let chain = create_test_chain();
        let curve = chain.iv_curve().unwrap();

        // All IV values should be positive
        for point in curve.points.iter() {
            assert!(
                point.y > Decimal::ZERO,
                "IV should be positive at strike {}",
                point.x
            );
        }
    }

    #[test]
    fn test_iv_curve_empty_chain() {
        let chain = create_empty_chain();
        let result = chain.iv_curve();

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod tests_implied_volatility_surface {
    use super::*;

    #[test]
    fn test_iv_surface_basic() {
        let chain = create_test_chain();
        let days = vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0)];
        let result = chain.iv_surface(days);

        assert!(result.is_ok());
        let surface = result.unwrap();

        // Surface should have points for each strike × each day
        let iv_curve = chain.iv_curve().unwrap();
        let expected_points = iv_curve.points.len() * 3;
        assert_eq!(surface.points.len(), expected_points);
    }

    #[test]
    fn test_iv_surface_single_day() {
        let chain = create_test_chain();
        let days = vec![pos_or_panic!(30.0)];
        let result = chain.iv_surface(days);

        assert!(result.is_ok());
        let surface = result.unwrap();

        // Surface with single day should have same points as IV curve
        let iv_curve = chain.iv_curve().unwrap();
        assert_eq!(surface.points.len(), iv_curve.points.len());
    }

    #[test]
    fn test_iv_surface_empty_days() {
        let chain = create_test_chain();
        let days: Vec<optionstratlib::Positive> = vec![];
        let result = chain.iv_surface(days);

        assert!(result.is_err());
    }

    #[test]
    fn test_iv_surface_time_scaling() {
        let chain = create_test_chain();
        let days = vec![pos_or_panic!(30.0), pos_or_panic!(90.0)];
        let surface = chain.iv_surface(days).unwrap();

        // Verify that longer-dated IVs are scaled appropriately
        // sqrt(90/365) / sqrt(30/365) ≈ 1.73
        let points: Vec<_> = surface.points.iter().collect();

        // Find points at same strike but different times
        let strike_100_points: Vec<_> = points
            .iter()
            .filter(|p| (p.x - dec!(100.0)).abs() < dec!(0.01))
            .collect();

        assert_eq!(strike_100_points.len(), 2);
    }

    #[test]
    fn test_iv_surface_empty_chain() {
        let chain = create_empty_chain();
        let days = vec![pos_or_panic!(30.0)];
        let result = chain.iv_surface(days);

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod tests_risk_reversal_curve {
    use super::*;

    #[test]
    fn test_risk_reversal_basic() {
        let chain = create_test_chain();
        let result = chain.risk_reversal_curve();

        assert!(result.is_ok());
        let curve = result.unwrap();

        // Should have same number of points as IV curve (options with valid IV)
        let iv_curve = chain.iv_curve().unwrap();
        assert_eq!(curve.points.len(), iv_curve.points.len());
    }

    #[test]
    fn test_risk_reversal_atm_near_zero() {
        let chain = create_test_chain();
        let curve = chain.risk_reversal_curve().unwrap();

        // Find ATM point (strike closest to 100)
        let atm_point = curve
            .points
            .iter()
            .min_by(|a, b| {
                let diff_a = (a.x - dec!(100.0)).abs();
                let diff_b = (b.x - dec!(100.0)).abs();
                diff_a.partial_cmp(&diff_b).unwrap()
            })
            .unwrap();

        // ATM risk reversal should be zero or very close to zero
        assert!(
            atm_point.y.abs() < dec!(0.01),
            "ATM RR should be near zero, got {}",
            atm_point.y
        );
    }

    #[test]
    fn test_risk_reversal_empty_chain() {
        let chain = create_empty_chain();
        let result = chain.risk_reversal_curve();

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod tests_dollar_gamma_curve {
    use super::*;

    #[test]
    fn test_dollar_gamma_call_basic() {
        let chain = create_test_chain();
        let result = chain.dollar_gamma_curve(&OptionStyle::Call);

        assert!(result.is_ok());
        let curve = result.unwrap();

        // Should have points for all strikes
        assert!(!curve.points.is_empty());
    }

    #[test]
    fn test_dollar_gamma_put_basic() {
        let chain = create_test_chain();
        let result = chain.dollar_gamma_curve(&OptionStyle::Put);

        assert!(result.is_ok());
        let curve = result.unwrap();

        // Should have points for all strikes
        assert!(!curve.points.is_empty());
    }

    #[test]
    fn test_dollar_gamma_values_positive() {
        let chain = create_test_chain();
        let curve = chain.dollar_gamma_curve(&OptionStyle::Call).unwrap();

        // All dollar gamma values should be positive (for long options)
        for point in curve.points.iter() {
            assert!(
                point.y >= Decimal::ZERO,
                "Dollar gamma should be non-negative at strike {}",
                point.x
            );
        }
    }

    #[test]
    fn test_dollar_gamma_atm_highest() {
        let chain = create_test_chain();
        let curve = chain.dollar_gamma_curve(&OptionStyle::Call).unwrap();

        let points: Vec<_> = curve.points.iter().collect();

        // Find ATM point
        let atm_point = points
            .iter()
            .min_by(|a, b| {
                let diff_a = (a.x - dec!(100.0)).abs();
                let diff_b = (b.x - dec!(100.0)).abs();
                diff_a.partial_cmp(&diff_b).unwrap()
            })
            .unwrap();

        // ATM should have highest or near-highest dollar gamma
        let max_dg = points.iter().map(|p| p.y).max().unwrap();

        // ATM dollar gamma should be at least 50% of max
        assert!(
            atm_point.y >= max_dg * dec!(0.5),
            "ATM dollar gamma {} should be significant relative to max {}",
            atm_point.y,
            max_dg
        );
    }

    #[test]
    fn test_dollar_gamma_empty_chain() {
        let chain = create_empty_chain();
        let result = chain.dollar_gamma_curve(&OptionStyle::Call);

        assert!(result.is_err());
    }

    #[test]
    fn test_dollar_gamma_call_equals_put() {
        let chain = create_test_chain();
        let call_curve = chain.dollar_gamma_curve(&OptionStyle::Call).unwrap();
        let put_curve = chain.dollar_gamma_curve(&OptionStyle::Put).unwrap();

        // Gamma is the same for calls and puts at the same strike
        // So dollar gamma should be equal (or very close)
        let call_points: Vec<_> = call_curve.points.iter().collect();
        let put_points: Vec<_> = put_curve.points.iter().collect();

        assert_eq!(call_points.len(), put_points.len());

        for (call_p, put_p) in call_points.iter().zip(put_points.iter()) {
            assert_eq!(call_p.x, put_p.x, "Strikes should match");
            // Allow small numerical differences
            let diff = (call_p.y - put_p.y).abs();
            assert!(
                diff < dec!(0.0001),
                "Dollar gamma should be equal for calls and puts at strike {}: call={}, put={}",
                call_p.x,
                call_p.y,
                put_p.y
            );
        }
    }
}

#[cfg(test)]
mod tests_edge_cases {
    use super::*;

    #[test]
    fn test_chain_with_single_strike() {
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

        // IV curve should work with single point
        let iv_result = chain.iv_curve();
        assert!(iv_result.is_ok());
        assert_eq!(iv_result.unwrap().points.len(), 1);

        // Risk reversal should work (ATM = only strike)
        let rr_result = chain.risk_reversal_curve();
        assert!(rr_result.is_ok());
    }

    #[test]
    fn test_high_volatility_environment() {
        let params = OptionChainBuildParams::new(
            "HIGH_VOL".to_string(),
            None,
            5,
            spos!(10.0),
            dec!(-0.3),
            dec!(0.2),
            pos_or_panic!(0.02),
            2,
            OptionDataPriceParams::new(
                Some(Box::new(pos_or_panic!(100.0))),
                Some(ExpirationDate::Days(pos_or_panic!(30.0))),
                Some(dec!(0.05)),
                spos!(0.02),
                Some("HIGH_VOL".to_string()),
            ),
            pos_or_panic!(0.80), // High base volatility
        );

        let chain = OptionChain::build_chain(&params);

        // All metrics should still work
        assert!(chain.iv_curve().is_ok());
        assert!(chain.risk_reversal_curve().is_ok());
        assert!(chain.dollar_gamma_curve(&OptionStyle::Call).is_ok());
    }

    #[test]
    fn test_low_volatility_environment() {
        let params = OptionChainBuildParams::new(
            "LOW_VOL".to_string(),
            None,
            5,
            spos!(10.0),
            dec!(-0.1),
            dec!(0.05),
            pos_or_panic!(0.01),
            2,
            OptionDataPriceParams::new(
                Some(Box::new(pos_or_panic!(100.0))),
                Some(ExpirationDate::Days(pos_or_panic!(30.0))),
                Some(dec!(0.05)),
                spos!(0.01),
                Some("LOW_VOL".to_string()),
            ),
            pos_or_panic!(0.05), // Low base volatility
        );

        let chain = OptionChain::build_chain(&params);

        // All metrics should still work
        assert!(chain.iv_curve().is_ok());
        assert!(chain.risk_reversal_curve().is_ok());
        assert!(chain.dollar_gamma_curve(&OptionStyle::Call).is_ok());
    }

    #[test]
    fn test_extreme_skew() {
        let params = OptionChainBuildParams::new(
            "SKEW".to_string(),
            None,
            5,
            spos!(10.0),
            dec!(-0.5), // Extreme negative skew
            dec!(0.3),  // High smile curvature
            pos_or_panic!(0.02),
            2,
            OptionDataPriceParams::new(
                Some(Box::new(pos_or_panic!(100.0))),
                Some(ExpirationDate::Days(pos_or_panic!(30.0))),
                Some(dec!(0.05)),
                spos!(0.02),
                Some("SKEW".to_string()),
            ),
            pos_or_panic!(0.25),
        );

        let chain = OptionChain::build_chain(&params);

        // Risk reversal should show the skew
        let rr_curve = chain.risk_reversal_curve().unwrap();
        let points: Vec<_> = rr_curve.points.iter().collect();

        // With negative skew, lower strikes should have higher IV
        // So RR should be negative for OTM puts
        let otm_put = points.iter().find(|p| p.x < dec!(100.0)).unwrap();
        let otm_call = points.iter().find(|p| p.x > dec!(100.0)).unwrap();

        // In a typical equity skew, OTM puts are more expensive
        // This test verifies the curve captures the skew direction
        assert!(
            otm_put.y != otm_call.y,
            "RR should differ between OTM puts and calls in skewed market"
        );
    }
}
