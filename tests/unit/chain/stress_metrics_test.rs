/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Stress Metrics Integration Tests
//!
//! Tests for the stress metrics implementations on OptionChain:
//! - Volatility Sensitivity Curve and Surface
//! - Time Decay Profile Curve and Surface
//! - Price Shock Impact Curve and Surface

use optionstratlib::chains::chain::OptionChain;
use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::metrics::{
    PriceShockCurve, PriceShockSurface, TimeDecayCurve, TimeDecaySurface,
    VolatilitySensitivityCurve, VolatilitySensitivitySurface,
};
use optionstratlib::model::ExpirationDate;
use optionstratlib::{pos, spos};
use rust_decimal_macros::dec;

/// Creates a test option chain with proper Greeks data
fn create_test_chain() -> OptionChain {
    let params = OptionChainBuildParams::new(
        "TEST".to_string(),
        None,
        10,
        spos!(5.0),
        dec!(-0.15),
        dec!(0.08),
        pos_or_panic!(0.02),
        2,
        OptionDataPriceParams::new(
            Some(Box::new(pos_or_panic!(450.0))),
            Some(ExpirationDate::Days(pos_or_panic!(30.0))),
            Some(dec!(0.05)),
            spos!(0.01),
            Some("TEST".to_string()),
        ),
        pos_or_panic!(0.20),
    );

    OptionChain::build_chain(&params)
}

/// Creates an empty option chain for edge case testing
fn create_empty_chain() -> OptionChain {
    OptionChain::new("EMPTY", pos_or_panic!(100.0), "2024-12-31".to_string(), None, None)
}

// ============================================================================
// Volatility Sensitivity Curve Tests
// ============================================================================

mod volatility_sensitivity_curve_tests {
    use super::*;

    #[test]
    fn test_volatility_sensitivity_curve_basic() {
        let chain = create_test_chain();
        let result = chain.volatility_sensitivity_curve();

        assert!(result.is_ok());
        let curve = result.unwrap();
        assert!(!curve.points.is_empty());
    }

    #[test]
    fn test_volatility_sensitivity_curve_vega_positive() {
        let chain = create_test_chain();
        let curve = chain.volatility_sensitivity_curve().unwrap();

        // Vega should be positive for long options
        for point in curve.points.iter() {
            assert!(point.y >= rust_decimal::Decimal::ZERO);
        }
    }

    #[test]
    fn test_volatility_sensitivity_curve_strike_ordering() {
        let chain = create_test_chain();
        let curve = chain.volatility_sensitivity_curve().unwrap();

        let points: Vec<_> = curve.points.iter().collect();

        // Points should be ordered by strike
        for i in 1..points.len() {
            assert!(points[i].x > points[i - 1].x);
        }
    }

    #[test]
    fn test_volatility_sensitivity_curve_empty_chain() {
        let chain = create_empty_chain();
        let result = chain.volatility_sensitivity_curve();

        assert!(result.is_err());
    }
}

// ============================================================================
// Volatility Sensitivity Surface Tests
// ============================================================================

mod volatility_sensitivity_surface_tests {
    use super::*;

    #[test]
    fn test_volatility_sensitivity_surface_basic() {
        let chain = create_test_chain();
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let vol_range = (pos_or_panic!(0.10), pos_or_panic!(0.40));

        let result = chain.volatility_sensitivity_surface(price_range, vol_range, 10, 10);
        assert!(result.is_ok());

        let surface = result.unwrap();
        // (10+1) × (10+1) = 121 points
        assert_eq!(surface.points.len(), 121);
    }

    #[test]
    fn test_volatility_sensitivity_surface_single_point() {
        let chain = create_test_chain();
        let price_range = (pos_or_panic!(450.0), pos_or_panic!(450.0));
        let vol_range = (pos_or_panic!(0.20), pos_or_panic!(0.20));

        let result = chain.volatility_sensitivity_surface(price_range, vol_range, 0, 0);
        assert!(result.is_ok());

        let surface = result.unwrap();
        assert_eq!(surface.points.len(), 1);
    }

    #[test]
    fn test_volatility_sensitivity_surface_empty_chain() {
        let chain = create_empty_chain();
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let vol_range = (pos_or_panic!(0.10), pos_or_panic!(0.40));

        let result = chain.volatility_sensitivity_surface(price_range, vol_range, 10, 10);
        assert!(result.is_err());
    }
}

// ============================================================================
// Time Decay Curve Tests
// ============================================================================

mod time_decay_curve_tests {
    use super::*;

    #[test]
    fn test_time_decay_curve_basic() {
        let chain = create_test_chain();
        let result = chain.time_decay_curve();

        assert!(result.is_ok());
        let curve = result.unwrap();
        assert!(!curve.points.is_empty());
    }

    #[test]
    fn test_time_decay_curve_theta_negative() {
        let chain = create_test_chain();
        let curve = chain.time_decay_curve().unwrap();

        // Theta should be negative for long options (time decay)
        for point in curve.points.iter() {
            assert!(point.y <= rust_decimal::Decimal::ZERO);
        }
    }

    #[test]
    fn test_time_decay_curve_strike_ordering() {
        let chain = create_test_chain();
        let curve = chain.time_decay_curve().unwrap();

        let points: Vec<_> = curve.points.iter().collect();

        // Points should be ordered by strike
        for i in 1..points.len() {
            assert!(points[i].x > points[i - 1].x);
        }
    }

    #[test]
    fn test_time_decay_curve_empty_chain() {
        let chain = create_empty_chain();
        let result = chain.time_decay_curve();

        assert!(result.is_err());
    }
}

// ============================================================================
// Time Decay Surface Tests
// ============================================================================

mod time_decay_surface_tests {
    use super::*;

    #[test]
    fn test_time_decay_surface_basic() {
        let chain = create_test_chain();
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let days = vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0)];

        let result = chain.time_decay_surface(price_range, days, 10);
        assert!(result.is_ok());

        let surface = result.unwrap();
        // (10+1) × 3 = 33 points
        assert_eq!(surface.points.len(), 33);
    }

    #[test]
    fn test_time_decay_surface_single_day() {
        let chain = create_test_chain();
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let days = vec![pos_or_panic!(30.0)];

        let result = chain.time_decay_surface(price_range, days, 10);
        assert!(result.is_ok());

        let surface = result.unwrap();
        assert_eq!(surface.points.len(), 11);
    }

    #[test]
    fn test_time_decay_surface_empty_days() {
        let chain = create_test_chain();
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let days: Vec<_> = vec![];

        let result = chain.time_decay_surface(price_range, days, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_time_decay_surface_empty_chain() {
        let chain = create_empty_chain();
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let days = vec![pos_or_panic!(30.0)];

        let result = chain.time_decay_surface(price_range, days, 10);
        assert!(result.is_err());
    }
}

// ============================================================================
// Price Shock Curve Tests
// ============================================================================

mod price_shock_curve_tests {
    use super::*;

    #[test]
    fn test_price_shock_curve_basic() {
        let chain = create_test_chain();
        let result = chain.price_shock_curve(dec!(-0.10));

        assert!(result.is_ok());
        let curve = result.unwrap();
        assert!(!curve.points.is_empty());
    }

    #[test]
    fn test_price_shock_curve_negative_shock() {
        let chain = create_test_chain();
        let curve = chain.price_shock_curve(dec!(-0.10)).unwrap();

        // For calls with negative shock, P&L should generally be negative
        let points: Vec<_> = curve.points.iter().collect();
        let negative_count = points.iter().filter(|p| p.y < dec!(0)).count();

        // Most points should show negative P&L for a down move
        assert!(negative_count > points.len() / 2);
    }

    #[test]
    fn test_price_shock_curve_positive_shock() {
        let chain = create_test_chain();
        let curve = chain.price_shock_curve(dec!(0.10)).unwrap();

        // For calls with positive shock, P&L should generally be positive
        let points: Vec<_> = curve.points.iter().collect();
        let positive_count = points.iter().filter(|p| p.y > dec!(0)).count();

        // Most points should show positive P&L for an up move
        assert!(positive_count > points.len() / 2);
    }

    #[test]
    fn test_price_shock_curve_zero_shock() {
        let chain = create_test_chain();
        let curve = chain.price_shock_curve(dec!(0.0)).unwrap();

        // With zero shock, P&L should be near zero
        for point in curve.points.iter() {
            assert!(point.y.abs() < dec!(0.01));
        }
    }

    #[test]
    fn test_price_shock_curve_empty_chain() {
        let chain = create_empty_chain();
        let result = chain.price_shock_curve(dec!(-0.10));

        assert!(result.is_err());
    }
}

// ============================================================================
// Price Shock Surface Tests
// ============================================================================

mod price_shock_surface_tests {
    use super::*;

    #[test]
    fn test_price_shock_surface_basic() {
        let chain = create_test_chain();
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let vol_range = (pos_or_panic!(0.10), pos_or_panic!(0.40));

        let result = chain.price_shock_surface(price_range, vol_range, 10, 10);
        assert!(result.is_ok());

        let surface = result.unwrap();
        assert_eq!(surface.points.len(), 121);
    }

    #[test]
    fn test_price_shock_surface_single_point() {
        let chain = create_test_chain();
        let price_range = (pos_or_panic!(450.0), pos_or_panic!(450.0));
        let vol_range = (pos_or_panic!(0.20), pos_or_panic!(0.20));

        let result = chain.price_shock_surface(price_range, vol_range, 0, 0);
        assert!(result.is_ok());

        let surface = result.unwrap();
        assert_eq!(surface.points.len(), 1);
    }

    #[test]
    fn test_price_shock_surface_empty_chain() {
        let chain = create_empty_chain();
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let vol_range = (pos_or_panic!(0.10), pos_or_panic!(0.40));

        let result = chain.price_shock_surface(price_range, vol_range, 10, 10);
        assert!(result.is_err());
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_extreme_price_shock() {
        let chain = create_test_chain();

        // Test extreme negative shock (-50%)
        let result = chain.price_shock_curve(dec!(-0.50));
        assert!(result.is_ok());

        // Test extreme positive shock (+50%)
        let result = chain.price_shock_curve(dec!(0.50));
        assert!(result.is_ok());
    }

    #[test]
    fn test_extreme_volatility_range() {
        let chain = create_test_chain();
        let price_range = (pos_or_panic!(450.0), pos_or_panic!(450.0));
        let vol_range = (pos_or_panic!(0.01), pos_or_panic!(1.00)); // 1% to 100% vol

        let result = chain.volatility_sensitivity_surface(price_range, vol_range, 0, 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_near_expiration() {
        let chain = create_test_chain();
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let days = vec![pos_or_panic!(1.0), pos_or_panic!(0.5)]; // Very short time to expiration

        let result = chain.time_decay_surface(price_range, days, 5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_wide_price_range() {
        let chain = create_test_chain();
        let price_range = (pos_or_panic!(200.0), pos_or_panic!(700.0)); // Very wide range
        let vol_range = (pos_or_panic!(0.20), pos_or_panic!(0.20));

        let result = chain.volatility_sensitivity_surface(price_range, vol_range, 20, 0);
        assert!(result.is_ok());
    }
}
