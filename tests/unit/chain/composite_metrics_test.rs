/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Composite Metrics Integration Tests
//!
//! Tests for the composite metrics implementations on OptionChain:
//! - Vanna-Volga Hedge Surface
//! - Delta-Gamma Profile Curve and Surface
//! - Smile Dynamics Curve and Surface

use optionstratlib::chains::OptionData;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::metrics::{
    DeltaGammaProfileCurve, DeltaGammaProfileSurface, SmileDynamicsCurve, SmileDynamicsSurface,
    VannaVolgaSurface,
};
use optionstratlib::model::ExpirationDate;
use positive::{Positive, pos_or_panic, spos};
use rust_decimal_macros::dec;

/// Creates a test option chain with proper IV values for testing
fn create_test_chain() -> OptionChain {
    let params = OptionChainBuildParams::new(
        "TEST".to_string(),
        None,
        10,                  // 10 strikes on each side
        spos!(5.0),          // $5 strike intervals
        dec!(-0.15),         // Negative skew
        dec!(0.08),          // Smile curvature
        pos_or_panic!(0.02), // Spread
        2,                   // Decimal places
        OptionDataPriceParams::new(
            Some(Box::new(Positive::HUNDRED)),               // Underlying price
            Some(ExpirationDate::Days(pos_or_panic!(30.0))), // 30 days to expiry
            Some(dec!(0.05)),                                // Risk-free rate
            spos!(0.01),                                     // Dividend yield
            Some("TEST".to_string()),
        ),
        pos_or_panic!(0.20), // Base IV of 20%
    );

    OptionChain::build_chain(&params)
}

/// Creates an empty option chain for edge case testing
fn create_empty_chain() -> OptionChain {
    OptionChain::new(
        "EMPTY",
        Positive::HUNDRED,
        "2024-12-31".to_string(),
        None,
        None,
    )
}

// ============================================================================
// Vanna-Volga Surface Tests
// ============================================================================

mod vanna_volga_tests {
    use super::*;

    #[test]
    fn test_vanna_volga_surface_basic() {
        let chain = create_test_chain();
        let price_range = (pos_or_panic!(80.0), pos_or_panic!(120.0));
        let vol_range = (pos_or_panic!(0.10), pos_or_panic!(0.40));

        let result = chain.vanna_volga_surface(price_range, vol_range, 10, 10);
        assert!(result.is_ok());

        let surface = result.unwrap();
        // (10+1) × (10+1) = 121 points
        assert_eq!(surface.points.len(), 121);
    }

    #[test]
    fn test_vanna_volga_surface_single_point() {
        let chain = create_test_chain();
        let price_range = (Positive::HUNDRED, Positive::HUNDRED);
        let vol_range = (pos_or_panic!(0.20), pos_or_panic!(0.20));

        let result = chain.vanna_volga_surface(price_range, vol_range, 0, 0);
        assert!(result.is_ok());

        let surface = result.unwrap();
        assert_eq!(surface.points.len(), 1);
    }

    #[test]
    fn test_vanna_volga_surface_cost_at_atm() {
        let chain = create_test_chain();
        let price_range = (Positive::HUNDRED, Positive::HUNDRED); // ATM only
        let vol_range = (pos_or_panic!(0.20), pos_or_panic!(0.20)); // ATM vol

        let result = chain.vanna_volga_surface(price_range, vol_range, 0, 0);
        assert!(result.is_ok());

        let surface = result.unwrap();
        let point = surface.points.iter().next().unwrap();

        // At ATM with ATM vol, cost should be minimal
        assert!(point.z < dec!(1.0));
    }

    #[test]
    fn test_vanna_volga_surface_cost_increases_otm() {
        let chain = create_test_chain();
        let price_range = (pos_or_panic!(80.0), pos_or_panic!(120.0));
        let vol_range = (pos_or_panic!(0.20), pos_or_panic!(0.20)); // Fixed vol

        let result = chain.vanna_volga_surface(price_range, vol_range, 10, 0);
        assert!(result.is_ok());

        let surface = result.unwrap();
        let points: Vec<_> = surface.points.iter().collect();

        // Find ATM point
        let atm_point = points
            .iter()
            .min_by(|a, b| {
                let a_dist = (a.x - dec!(100.0)).abs();
                let b_dist = (b.x - dec!(100.0)).abs();
                a_dist.partial_cmp(&b_dist).unwrap()
            })
            .unwrap();

        // OTM points should have higher cost
        for point in points.iter() {
            if (point.x - dec!(100.0)).abs() > dec!(5.0) {
                assert!(point.z >= atm_point.z);
            }
        }
    }
}

// ============================================================================
// Delta-Gamma Profile Curve Tests
// ============================================================================

mod delta_gamma_curve_tests {
    use super::*;

    #[test]
    fn test_delta_gamma_curve_basic() {
        let chain = create_test_chain();
        let result = chain.delta_gamma_curve();

        assert!(result.is_ok());
        let curve = result.unwrap();
        assert!(!curve.points.is_empty());
    }

    #[test]
    fn test_delta_gamma_curve_values_positive() {
        let chain = create_test_chain();
        let curve = chain.delta_gamma_curve().unwrap();

        // For long calls, combined delta + gamma should be positive
        for point in curve.points.iter() {
            // Delta is positive for calls, gamma is always positive
            // Combined metric should be positive
            assert!(point.y > dec!(0.0));
        }
    }

    #[test]
    fn test_delta_gamma_curve_empty_chain() {
        let chain = create_empty_chain();
        let result = chain.delta_gamma_curve();

        assert!(result.is_err());
    }

    #[test]
    fn test_delta_gamma_curve_strike_ordering() {
        let chain = create_test_chain();
        let curve = chain.delta_gamma_curve().unwrap();

        let points: Vec<_> = curve.points.iter().collect();

        // Points should be ordered by strike
        for i in 1..points.len() {
            assert!(points[i].x >= points[i - 1].x);
        }
    }
}

// ============================================================================
// Delta-Gamma Profile Surface Tests
// ============================================================================

mod delta_gamma_surface_tests {
    use super::*;

    #[test]
    fn test_delta_gamma_surface_basic() {
        let chain = create_test_chain();
        let price_range = (pos_or_panic!(80.0), pos_or_panic!(120.0));
        let days = vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0)];

        let result = chain.delta_gamma_surface(price_range, days, 10);
        assert!(result.is_ok());

        let surface = result.unwrap();
        // (10+1) × 3 = 33 points
        assert_eq!(surface.points.len(), 33);
    }

    #[test]
    fn test_delta_gamma_surface_single_day() {
        let chain = create_test_chain();
        let price_range = (pos_or_panic!(90.0), pos_or_panic!(110.0));
        let days = vec![pos_or_panic!(30.0)];

        let result = chain.delta_gamma_surface(price_range, days, 5);
        assert!(result.is_ok());

        let surface = result.unwrap();
        // (5+1) × 1 = 6 points
        assert_eq!(surface.points.len(), 6);
    }

    #[test]
    fn test_delta_gamma_surface_empty_days() {
        let chain = create_test_chain();
        let price_range = (pos_or_panic!(80.0), pos_or_panic!(120.0));
        let days: Vec<_> = vec![];

        let result = chain.delta_gamma_surface(price_range, days, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_delta_gamma_surface_delta_range() {
        let chain = create_test_chain();
        let price_range = (pos_or_panic!(80.0), pos_or_panic!(120.0));
        let days = vec![pos_or_panic!(30.0)];

        let surface = chain.delta_gamma_surface(price_range, days, 10).unwrap();

        // Delta should be between 0 and 1 for calls
        for point in surface.points.iter() {
            assert!(point.z >= dec!(0.0) && point.z <= dec!(1.0));
        }
    }
}

// ============================================================================
// Smile Dynamics Curve Tests
// ============================================================================

mod smile_dynamics_curve_tests {
    use super::*;

    #[test]
    fn test_smile_dynamics_curve_basic() {
        let chain = create_test_chain();
        let result = chain.smile_dynamics_curve();

        assert!(result.is_ok());
        let curve = result.unwrap();
        assert!(!curve.points.is_empty());
    }

    #[test]
    fn test_smile_dynamics_curve_iv_positive() {
        let chain = create_test_chain();
        let curve = chain.smile_dynamics_curve().unwrap();

        // All IV values should be positive
        for point in curve.points.iter() {
            assert!(point.y > dec!(0.0));
        }
    }

    #[test]
    fn test_smile_dynamics_curve_empty_chain() {
        let chain = create_empty_chain();
        let result = chain.smile_dynamics_curve();

        assert!(result.is_err());
    }

    #[test]
    fn test_smile_dynamics_curve_strike_ordering() {
        let chain = create_test_chain();
        let curve = chain.smile_dynamics_curve().unwrap();

        let points: Vec<_> = curve.points.iter().collect();

        // Points should be ordered by strike
        for i in 1..points.len() {
            assert!(points[i].x >= points[i - 1].x);
        }
    }
}

// ============================================================================
// Smile Dynamics Surface Tests
// ============================================================================

mod smile_dynamics_surface_tests {
    use super::*;

    #[test]
    fn test_smile_dynamics_surface_basic() {
        let chain = create_test_chain();
        let days = vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0)];

        let result = chain.smile_dynamics_surface(days);
        assert!(result.is_ok());

        let surface = result.unwrap();
        // Number of strikes × 3 days
        let curve = chain.smile_dynamics_curve().unwrap();
        let expected_points = curve.points.len() * 3;
        assert_eq!(surface.points.len(), expected_points);
    }

    #[test]
    fn test_smile_dynamics_surface_single_day() {
        let chain = create_test_chain();
        let days = vec![pos_or_panic!(30.0)];

        let result = chain.smile_dynamics_surface(days);
        assert!(result.is_ok());

        let surface = result.unwrap();
        let curve = chain.smile_dynamics_curve().unwrap();
        assert_eq!(surface.points.len(), curve.points.len());
    }

    #[test]
    fn test_smile_dynamics_surface_empty_days() {
        let chain = create_test_chain();
        let days: Vec<_> = vec![];

        let result = chain.smile_dynamics_surface(days);
        assert!(result.is_err());
    }

    #[test]
    fn test_smile_dynamics_surface_iv_positive() {
        let chain = create_test_chain();
        let days = vec![pos_or_panic!(7.0), pos_or_panic!(30.0)];

        let surface = chain.smile_dynamics_surface(days).unwrap();

        // All IV values should be positive
        for point in surface.points.iter() {
            assert!(point.z > dec!(0.0));
        }
    }

    #[test]
    fn test_smile_dynamics_surface_skew_steepens() {
        let chain = create_test_chain();
        let days = vec![pos_or_panic!(7.0), pos_or_panic!(30.0)];

        let surface = chain.smile_dynamics_surface(days).unwrap();

        // Find OTM put points at different expirations
        let points: Vec<_> = surface.points.iter().collect();

        // Get min strike (OTM put)
        let min_strike = points.iter().map(|p| p.x).min().unwrap();

        let iv_7d = points
            .iter()
            .find(|p| p.x == min_strike && p.y == dec!(7.0))
            .map(|p| p.z);
        let iv_30d = points
            .iter()
            .find(|p| p.x == min_strike && p.y == dec!(30.0))
            .map(|p| p.z);

        // For OTM options with negative skew, shorter expiry should have steeper skew
        // (higher IV for OTM puts)
        if let (Some(iv7), Some(iv30)) = (iv_7d, iv_30d) {
            // The skew effect should be visible
            assert!(iv7 != iv30);
        }
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_single_option_chain() {
        let mut chain = create_empty_chain();

        // Add a single option
        let option_data = OptionData::new(
            Positive::HUNDRED,
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
            Some(Box::new(Positive::HUNDRED)),
            Some(dec!(0.05)),
            spos!(0.02),
            None,
            None,
        );
        chain.options.insert(option_data);

        // Smile dynamics curve should work with single point
        let smile_result = chain.smile_dynamics_curve();
        assert!(smile_result.is_ok());
        assert_eq!(smile_result.unwrap().points.len(), 1);

        // Vanna-Volga surface should work
        let vv_result = chain.vanna_volga_surface(
            (pos_or_panic!(90.0), pos_or_panic!(110.0)),
            (pos_or_panic!(0.15), pos_or_panic!(0.25)),
            5,
            5,
        );
        assert!(vv_result.is_ok());
    }

    #[test]
    fn test_extreme_price_range() {
        let chain = create_test_chain();

        // Very wide price range
        let result = chain.vanna_volga_surface(
            (pos_or_panic!(10.0), pos_or_panic!(1000.0)),
            (pos_or_panic!(0.05), Positive::ONE),
            20,
            20,
        );
        assert!(result.is_ok());

        let surface = result.unwrap();
        assert_eq!(surface.points.len(), 441); // 21 × 21
    }

    #[test]
    fn test_very_short_expiration() {
        let chain = create_test_chain();

        // Very short expiration (1 day)
        let days = vec![Positive::ONE];
        let result = chain.smile_dynamics_surface(days);
        assert!(result.is_ok());
    }

    #[test]
    fn test_very_long_expiration() {
        let chain = create_test_chain();

        // Very long expiration (365 days)
        let days = vec![pos_or_panic!(365.0)];
        let result = chain.smile_dynamics_surface(days);
        assert!(result.is_ok());
    }
}
