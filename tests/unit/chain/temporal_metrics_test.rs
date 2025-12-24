/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/24
******************************************************************************/

//! # Temporal Metrics Integration Tests
//!
//! Tests for the temporal metrics implementations on OptionChain:
//! - Theta Curve and Surface
//! - Charm Curve and Surface
//! - Color Curve and Surface

use optionstratlib::chains::chain::OptionChain;
use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::metrics::{CharmSurface, ColorSurface, ThetaSurface};
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
        pos!(0.02),
        2,
        OptionDataPriceParams::new(
            Some(Box::new(pos!(450.0))),
            Some(ExpirationDate::Days(pos!(30.0))),
            Some(dec!(0.05)),
            spos!(0.01),
            Some("TEST".to_string()),
        ),
        pos!(0.20),
    );

    OptionChain::build_chain(&params)
}

/// Creates an empty option chain for edge case testing
fn create_empty_chain() -> OptionChain {
    OptionChain::new("EMPTY", pos!(100.0), "2024-12-31".to_string(), None, None)
}

// ============================================================================
// Theta Curve Tests
// ============================================================================

mod theta_curve_tests {
    use super::*;

    #[test]
    fn test_theta_curve_basic() {
        let chain = create_test_chain();
        let result = chain.theta_curve();

        assert!(result.is_ok());
        let curve = result.unwrap();
        assert!(!curve.points.is_empty());
    }

    #[test]
    fn test_theta_curve_negative_values() {
        let chain = create_test_chain();
        let curve = chain.theta_curve().unwrap();

        // Theta should be negative for long options (time decay)
        for point in curve.points.iter() {
            assert!(point.y <= rust_decimal::Decimal::ZERO);
        }
    }

    #[test]
    fn test_theta_curve_strike_ordering() {
        let chain = create_test_chain();
        let curve = chain.theta_curve().unwrap();

        let points: Vec<_> = curve.points.iter().collect();

        // Points should be ordered by strike
        for i in 1..points.len() {
            assert!(points[i].x > points[i - 1].x);
        }
    }

    #[test]
    fn test_theta_curve_empty_chain() {
        let chain = create_empty_chain();
        let result = chain.theta_curve();

        // Empty chain returns error or empty curve
        assert!(result.is_err() || result.unwrap().points.is_empty());
    }
}

// ============================================================================
// Theta Surface Tests
// ============================================================================

mod theta_surface_tests {
    use super::*;

    #[test]
    fn test_theta_surface_basic() {
        let chain = create_test_chain();
        let price_range = (pos!(400.0), pos!(500.0));
        let days = vec![pos!(7.0), pos!(14.0), pos!(30.0)];

        let result = chain.theta_surface(price_range, days, 10);
        assert!(result.is_ok());

        let surface = result.unwrap();
        // (10+1) × 3 = 33 points
        assert_eq!(surface.points.len(), 33);
    }

    #[test]
    fn test_theta_surface_single_day() {
        let chain = create_test_chain();
        let price_range = (pos!(400.0), pos!(500.0));
        let days = vec![pos!(30.0)];

        let result = chain.theta_surface(price_range, days, 10);
        assert!(result.is_ok());

        let surface = result.unwrap();
        assert_eq!(surface.points.len(), 11);
    }

    #[test]
    fn test_theta_surface_empty_days() {
        let chain = create_test_chain();
        let price_range = (pos!(400.0), pos!(500.0));
        let days: Vec<_> = vec![];

        let result = chain.theta_surface(price_range, days, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_theta_surface_empty_chain() {
        let chain = create_empty_chain();
        let price_range = (pos!(400.0), pos!(500.0));
        let days = vec![pos!(30.0)];

        let result = chain.theta_surface(price_range, days, 10);
        assert!(result.is_err());
    }
}

// ============================================================================
// Charm Curve Tests
// ============================================================================

mod charm_curve_tests {
    use super::*;

    #[test]
    fn test_charm_curve_basic() {
        let chain = create_test_chain();
        let result = chain.charm_curve();

        assert!(result.is_ok());
        let curve = result.unwrap();
        assert!(!curve.points.is_empty());
    }

    #[test]
    fn test_charm_curve_strike_ordering() {
        let chain = create_test_chain();
        let curve = chain.charm_curve().unwrap();

        let points: Vec<_> = curve.points.iter().collect();

        // Points should be ordered by strike
        for i in 1..points.len() {
            assert!(points[i].x > points[i - 1].x);
        }
    }

    #[test]
    fn test_charm_curve_empty_chain() {
        let chain = create_empty_chain();
        let result = chain.charm_curve();

        // Empty chain returns error or empty curve
        assert!(result.is_err() || result.unwrap().points.is_empty());
    }
}

// ============================================================================
// Charm Surface Tests
// ============================================================================

mod charm_surface_tests {
    use super::*;

    #[test]
    fn test_charm_surface_basic() {
        let chain = create_test_chain();
        let price_range = (pos!(400.0), pos!(500.0));
        let days = vec![pos!(7.0), pos!(14.0), pos!(30.0)];

        let result = chain.charm_surface(price_range, days, 10);
        assert!(result.is_ok());

        let surface = result.unwrap();
        assert_eq!(surface.points.len(), 33);
    }

    #[test]
    fn test_charm_surface_single_day() {
        let chain = create_test_chain();
        let price_range = (pos!(400.0), pos!(500.0));
        let days = vec![pos!(30.0)];

        let result = chain.charm_surface(price_range, days, 10);
        assert!(result.is_ok());

        let surface = result.unwrap();
        assert_eq!(surface.points.len(), 11);
    }

    #[test]
    fn test_charm_surface_empty_days() {
        let chain = create_test_chain();
        let price_range = (pos!(400.0), pos!(500.0));
        let days: Vec<_> = vec![];

        let result = chain.charm_surface(price_range, days, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_charm_surface_empty_chain() {
        let chain = create_empty_chain();
        let price_range = (pos!(400.0), pos!(500.0));
        let days = vec![pos!(30.0)];

        let result = chain.charm_surface(price_range, days, 10);
        assert!(result.is_err());
    }
}

// ============================================================================
// Color Curve Tests
// ============================================================================

mod color_curve_tests {
    use super::*;

    #[test]
    fn test_color_curve_basic() {
        let chain = create_test_chain();
        let result = chain.color_curve();

        assert!(result.is_ok());
        let curve = result.unwrap();
        assert!(!curve.points.is_empty());
    }

    #[test]
    fn test_color_curve_strike_ordering() {
        let chain = create_test_chain();
        let curve = chain.color_curve().unwrap();

        let points: Vec<_> = curve.points.iter().collect();

        // Points should be ordered by strike
        for i in 1..points.len() {
            assert!(points[i].x > points[i - 1].x);
        }
    }

    #[test]
    fn test_color_curve_empty_chain() {
        let chain = create_empty_chain();
        let result = chain.color_curve();

        // Empty chain returns error or empty curve
        assert!(result.is_err() || result.unwrap().points.is_empty());
    }
}

// ============================================================================
// Color Surface Tests
// ============================================================================

mod color_surface_tests {
    use super::*;

    #[test]
    fn test_color_surface_basic() {
        let chain = create_test_chain();
        let price_range = (pos!(400.0), pos!(500.0));
        let days = vec![pos!(7.0), pos!(14.0), pos!(30.0)];

        let result = chain.color_surface(price_range, days, 10);
        assert!(result.is_ok());

        let surface = result.unwrap();
        assert_eq!(surface.points.len(), 33);
    }

    #[test]
    fn test_color_surface_single_day() {
        let chain = create_test_chain();
        let price_range = (pos!(400.0), pos!(500.0));
        let days = vec![pos!(30.0)];

        let result = chain.color_surface(price_range, days, 10);
        assert!(result.is_ok());

        let surface = result.unwrap();
        assert_eq!(surface.points.len(), 11);
    }

    #[test]
    fn test_color_surface_empty_days() {
        let chain = create_test_chain();
        let price_range = (pos!(400.0), pos!(500.0));
        let days: Vec<_> = vec![];

        let result = chain.color_surface(price_range, days, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_color_surface_empty_chain() {
        let chain = create_empty_chain();
        let price_range = (pos!(400.0), pos!(500.0));
        let days = vec![pos!(30.0)];

        let result = chain.color_surface(price_range, days, 10);
        assert!(result.is_err());
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_near_expiration_theta() {
        let chain = create_test_chain();
        let price_range = (pos!(400.0), pos!(500.0));
        let days = vec![pos!(1.0), pos!(2.0), pos!(3.0)];

        let result = chain.theta_surface(price_range, days, 5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_near_expiration_charm() {
        let chain = create_test_chain();
        let price_range = (pos!(400.0), pos!(500.0));
        let days = vec![pos!(1.0), pos!(2.0), pos!(3.0)];

        let result = chain.charm_surface(price_range, days, 5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_near_expiration_color() {
        let chain = create_test_chain();
        let price_range = (pos!(400.0), pos!(500.0));
        let days = vec![pos!(1.0), pos!(2.0), pos!(3.0)];

        let result = chain.color_surface(price_range, days, 5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_wide_price_range() {
        let chain = create_test_chain();
        let price_range = (pos!(200.0), pos!(700.0));
        let days = vec![pos!(30.0)];

        let theta_result = chain.theta_surface(price_range, days.clone(), 20);
        let charm_result = chain.charm_surface(price_range, days.clone(), 20);
        let color_result = chain.color_surface(price_range, days, 20);

        assert!(theta_result.is_ok());
        assert!(charm_result.is_ok());
        assert!(color_result.is_ok());
    }

    #[test]
    fn test_single_point_surface() {
        let chain = create_test_chain();
        let price_range = (pos!(450.0), pos!(450.0));
        let days = vec![pos!(30.0)];

        let theta_result = chain.theta_surface(price_range, days.clone(), 0);
        let charm_result = chain.charm_surface(price_range, days.clone(), 0);
        let color_result = chain.color_surface(price_range, days, 0);

        assert!(theta_result.is_ok());
        assert!(charm_result.is_ok());
        assert!(color_result.is_ok());

        assert_eq!(theta_result.unwrap().points.len(), 1);
        assert_eq!(charm_result.unwrap().points.len(), 1);
        assert_eq!(color_result.unwrap().points.len(), 1);
    }
}
