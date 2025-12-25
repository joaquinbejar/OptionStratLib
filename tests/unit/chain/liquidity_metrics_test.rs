/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Liquidity Metrics Integration Tests
//!
//! Tests for the liquidity metrics implementations on OptionChain:
//! - Bid-Ask Spread Curve
//! - Volume Profile Curve and Surface
//! - Open Interest Distribution Curve

use optionstratlib::chains::OptionData;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::metrics::{
    BidAskSpreadCurve, OpenInterestCurve, VolumeProfileCurve, VolumeProfileSurface,
};
use optionstratlib::model::ExpirationDate;
use positive::{pos_or_panic, spos, Positive};
use rust_decimal_macros::dec;

/// Creates a test option chain with liquidity data (bid/ask, volume, OI)
fn create_test_chain_with_liquidity() -> OptionChain {
    let mut chain = OptionChain::new("TEST", pos_or_panic!(450.0), "2024-12-31".to_string(), None, None);

    // Add options with liquidity data
    let strikes_data = [
        // (strike, call_bid, call_ask, put_bid, put_ask, volume, open_interest)
        (
            pos_or_panic!(400.0),
            pos_or_panic!(52.0),
            pos_or_panic!(54.0),
            Positive::TWO,
            pos_or_panic!(2.5),
            pos_or_panic!(500.0),
            2500u64,
        ),
        (
            pos_or_panic!(420.0),
            pos_or_panic!(35.0),
            pos_or_panic!(36.0),
            pos_or_panic!(5.0),
            pos_or_panic!(5.5),
            pos_or_panic!(1200.0),
            5000u64,
        ),
        (
            pos_or_panic!(440.0),
            pos_or_panic!(18.0),
            pos_or_panic!(18.5),
            pos_or_panic!(12.0),
            pos_or_panic!(12.5),
            pos_or_panic!(2500.0),
            8000u64,
        ),
        (
            pos_or_panic!(450.0),
            pos_or_panic!(10.0),
            pos_or_panic!(10.2),
            pos_or_panic!(18.0),
            pos_or_panic!(18.2),
            pos_or_panic!(5000.0),
            15000u64,
        ),
        (
            pos_or_panic!(460.0),
            pos_or_panic!(5.0),
            pos_or_panic!(5.2),
            pos_or_panic!(28.0),
            pos_or_panic!(28.5),
            pos_or_panic!(4200.0),
            12000u64,
        ),
        (
            pos_or_panic!(480.0),
            pos_or_panic!(1.5),
            pos_or_panic!(1.8),
            pos_or_panic!(45.0),
            pos_or_panic!(46.0),
            pos_or_panic!(2800.0),
            7000u64,
        ),
        (
            pos_or_panic!(500.0),
            pos_or_panic!(0.5),
            pos_or_panic!(0.8),
            pos_or_panic!(62.0),
            pos_or_panic!(64.0),
            pos_or_panic!(1500.0),
            4000u64,
        ),
    ];

    for (strike, call_bid, call_ask, put_bid, put_ask, volume, oi) in strikes_data {
        let option_data = OptionData::new(
            strike,
            spos!(call_bid.to_f64()),
            spos!(call_ask.to_f64()),
            spos!(put_bid.to_f64()),
            spos!(put_ask.to_f64()),
            pos_or_panic!(0.20),
            Some(dec!(0.5)),
            Some(dec!(-0.5)),
            Some(dec!(0.05)),
            spos!(volume.to_f64()),
            Some(oi),
            Some("TEST".to_string()),
            Some(ExpirationDate::Days(pos_or_panic!(30.0))),
            Some(Box::new(pos_or_panic!(450.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            None,
            None,
        );
        chain.options.insert(option_data);
    }

    chain
}

/// Creates an empty option chain for edge case testing
fn create_empty_chain() -> OptionChain {
    OptionChain::new("EMPTY", Positive::HUNDRED, "2024-12-31".to_string(), None, None)
}

/// Creates a chain without liquidity data
fn create_chain_without_liquidity() -> OptionChain {
    let mut chain = OptionChain::new("NOLIQ", pos_or_panic!(450.0), "2024-12-31".to_string(), None, None);

    // Add option without bid/ask, volume, or OI
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

// ============================================================================
// Bid-Ask Spread Curve Tests
// ============================================================================

mod bid_ask_spread_tests {
    use super::*;

    #[test]
    fn test_bid_ask_spread_curve_basic() {
        let chain = create_test_chain_with_liquidity();
        let result = chain.bid_ask_spread_curve();

        assert!(result.is_ok());
        let curve = result.unwrap();
        assert_eq!(curve.points.len(), 7);
    }

    #[test]
    fn test_bid_ask_spread_curve_values_positive() {
        let chain = create_test_chain_with_liquidity();
        let curve = chain.bid_ask_spread_curve().unwrap();

        // All spreads should be positive
        for point in curve.points.iter() {
            assert!(point.y > rust_decimal::Decimal::ZERO);
        }
    }

    #[test]
    fn test_bid_ask_spread_curve_atm_tighter() {
        let chain = create_test_chain_with_liquidity();
        let curve = chain.bid_ask_spread_curve().unwrap();

        let points: Vec<_> = curve.points.iter().collect();

        // Find ATM spread (strike 450)
        let atm_spread = points.iter().find(|p| p.x == dec!(450.0)).map(|p| p.y);

        // Find OTM spread (strike 500)
        let otm_spread = points.iter().find(|p| p.x == dec!(500.0)).map(|p| p.y);

        // ATM should have tighter spread than deep OTM
        if let (Some(atm), Some(otm)) = (atm_spread, otm_spread) {
            assert!(atm < otm);
        }
    }

    #[test]
    fn test_bid_ask_spread_curve_empty_chain() {
        let chain = create_empty_chain();
        let result = chain.bid_ask_spread_curve();

        assert!(result.is_err());
    }

    #[test]
    fn test_bid_ask_spread_curve_no_liquidity() {
        let chain = create_chain_without_liquidity();
        let result = chain.bid_ask_spread_curve();

        assert!(result.is_err());
    }

    #[test]
    fn test_bid_ask_spread_curve_strike_ordering() {
        let chain = create_test_chain_with_liquidity();
        let curve = chain.bid_ask_spread_curve().unwrap();

        let points: Vec<_> = curve.points.iter().collect();

        // Points should be ordered by strike
        for i in 1..points.len() {
            assert!(points[i].x > points[i - 1].x);
        }
    }
}

// ============================================================================
// Volume Profile Curve Tests
// ============================================================================

mod volume_profile_curve_tests {
    use super::*;

    #[test]
    fn test_volume_profile_curve_basic() {
        let chain = create_test_chain_with_liquidity();
        let result = chain.volume_profile_curve();

        assert!(result.is_ok());
        let curve = result.unwrap();
        assert_eq!(curve.points.len(), 7);
    }

    #[test]
    fn test_volume_profile_curve_values_positive() {
        let chain = create_test_chain_with_liquidity();
        let curve = chain.volume_profile_curve().unwrap();

        // All volumes should be positive
        for point in curve.points.iter() {
            assert!(point.y > rust_decimal::Decimal::ZERO);
        }
    }

    #[test]
    fn test_volume_profile_curve_atm_highest() {
        let chain = create_test_chain_with_liquidity();
        let curve = chain.volume_profile_curve().unwrap();

        let points: Vec<_> = curve.points.iter().collect();

        // Find maximum volume
        let max_vol = points.iter().max_by(|a, b| a.y.partial_cmp(&b.y).unwrap());

        // ATM (450) should have highest volume
        if let Some(max) = max_vol {
            assert_eq!(max.x, dec!(450.0));
        }
    }

    #[test]
    fn test_volume_profile_curve_empty_chain() {
        let chain = create_empty_chain();
        let result = chain.volume_profile_curve();

        assert!(result.is_err());
    }

    #[test]
    fn test_volume_profile_curve_no_liquidity() {
        let chain = create_chain_without_liquidity();
        let result = chain.volume_profile_curve();

        assert!(result.is_err());
    }
}

// ============================================================================
// Volume Profile Surface Tests
// ============================================================================

mod volume_profile_surface_tests {
    use super::*;

    #[test]
    fn test_volume_profile_surface_basic() {
        let chain = create_test_chain_with_liquidity();
        let days = vec![pos_or_panic!(5.0), pos_or_panic!(15.0), pos_or_panic!(30.0)];

        let result = chain.volume_profile_surface(days);
        assert!(result.is_ok());

        let surface = result.unwrap();
        // 7 strikes × 3 days = 21 points
        assert_eq!(surface.points.len(), 21);
    }

    #[test]
    fn test_volume_profile_surface_single_day() {
        let chain = create_test_chain_with_liquidity();
        let days = vec![pos_or_panic!(30.0)];

        let result = chain.volume_profile_surface(days);
        assert!(result.is_ok());

        let surface = result.unwrap();
        assert_eq!(surface.points.len(), 7);
    }

    #[test]
    fn test_volume_profile_surface_empty_days() {
        let chain = create_test_chain_with_liquidity();
        let days: Vec<_> = vec![];

        let result = chain.volume_profile_surface(days);
        assert!(result.is_err());
    }

    #[test]
    fn test_volume_profile_surface_time_effect() {
        let chain = create_test_chain_with_liquidity();
        let days = vec![pos_or_panic!(5.0), pos_or_panic!(30.0)];

        let surface = chain.volume_profile_surface(days).unwrap();

        // Find ATM volume at different times
        let points: Vec<_> = surface.points.iter().collect();

        let vol_5d = points
            .iter()
            .find(|p| p.x == dec!(450.0) && p.y == dec!(5.0))
            .map(|p| p.z);
        let vol_30d = points
            .iter()
            .find(|p| p.x == dec!(450.0) && p.y == dec!(30.0))
            .map(|p| p.z);

        // Volume should be higher closer to expiration (5 days vs 30 days)
        if let (Some(v5), Some(v30)) = (vol_5d, vol_30d) {
            assert!(v5 > v30);
        }
    }

    #[test]
    fn test_volume_profile_surface_no_liquidity() {
        let chain = create_chain_without_liquidity();
        let days = vec![pos_or_panic!(30.0)];

        let result = chain.volume_profile_surface(days);
        assert!(result.is_err());
    }
}

// ============================================================================
// Open Interest Curve Tests
// ============================================================================

mod open_interest_curve_tests {
    use super::*;

    #[test]
    fn test_open_interest_curve_basic() {
        let chain = create_test_chain_with_liquidity();
        let result = chain.open_interest_curve();

        assert!(result.is_ok());
        let curve = result.unwrap();
        assert_eq!(curve.points.len(), 7);
    }

    #[test]
    fn test_open_interest_curve_values_positive() {
        let chain = create_test_chain_with_liquidity();
        let curve = chain.open_interest_curve().unwrap();

        // All OI values should be positive
        for point in curve.points.iter() {
            assert!(point.y > rust_decimal::Decimal::ZERO);
        }
    }

    #[test]
    fn test_open_interest_curve_atm_highest() {
        let chain = create_test_chain_with_liquidity();
        let curve = chain.open_interest_curve().unwrap();

        let points: Vec<_> = curve.points.iter().collect();

        // Find maximum OI
        let max_oi = points.iter().max_by(|a, b| a.y.partial_cmp(&b.y).unwrap());

        // ATM (450) should have highest OI
        if let Some(max) = max_oi {
            assert_eq!(max.x, dec!(450.0));
            assert_eq!(max.y, dec!(15000));
        }
    }

    #[test]
    fn test_open_interest_curve_empty_chain() {
        let chain = create_empty_chain();
        let result = chain.open_interest_curve();

        assert!(result.is_err());
    }

    #[test]
    fn test_open_interest_curve_no_liquidity() {
        let chain = create_chain_without_liquidity();
        let result = chain.open_interest_curve();

        assert!(result.is_err());
    }

    #[test]
    fn test_open_interest_curve_strike_ordering() {
        let chain = create_test_chain_with_liquidity();
        let curve = chain.open_interest_curve().unwrap();

        let points: Vec<_> = curve.points.iter().collect();

        // Points should be ordered by strike
        for i in 1..points.len() {
            assert!(points[i].x > points[i - 1].x);
        }
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_single_option_with_liquidity() {
        let mut chain = create_empty_chain();

        let option_data = OptionData::new(
            Positive::HUNDRED,
            spos!(5.0),
            spos!(5.5),
            spos!(5.0),
            spos!(5.5),
            pos_or_panic!(0.20),
            None,
            None,
            None,
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

        // All metrics should work with single option
        assert!(chain.bid_ask_spread_curve().is_ok());
        assert!(chain.volume_profile_curve().is_ok());
        assert!(chain.open_interest_curve().is_ok());
    }

    #[test]
    fn test_partial_liquidity_data() {
        let mut chain = create_empty_chain();

        // Option with only volume (no bid/ask, no OI)
        let option_data = OptionData::new(
            Positive::HUNDRED,
            None,
            None,
            None,
            None,
            pos_or_panic!(0.20),
            None,
            None,
            None,
            spos!(1000.0),
            None,
            Some("TEST".to_string()),
            Some(ExpirationDate::Days(pos_or_panic!(30.0))),
            Some(Box::new(Positive::HUNDRED)),
            Some(dec!(0.05)),
            None,
            None,
            None,
        );
        chain.options.insert(option_data);

        // Only volume curve should work
        assert!(chain.bid_ask_spread_curve().is_err());
        assert!(chain.volume_profile_curve().is_ok());
        assert!(chain.open_interest_curve().is_err());
    }

    #[test]
    fn test_extreme_spread_values() {
        let mut chain = create_empty_chain();

        // Option with very wide spread (illiquid)
        let option_data = OptionData::new(
            Positive::HUNDRED,
            spos!(1.0),
            spos!(5.0), // 400% spread!
            None,
            None,
            pos_or_panic!(0.20),
            None,
            None,
            None,
            spos!(100.0),
            Some(100),
            Some("TEST".to_string()),
            Some(ExpirationDate::Days(pos_or_panic!(30.0))),
            Some(Box::new(Positive::HUNDRED)),
            Some(dec!(0.05)),
            spos!(0.02),
            None,
            None,
        );
        chain.options.insert(option_data);

        let curve = chain.bid_ask_spread_curve().unwrap();
        let point = curve.points.iter().next().unwrap();

        // Spread should be (5-1)/3 = 1.33... (133%)
        assert!(point.y > dec!(1.0));
    }

    #[test]
    fn test_high_open_interest_concentration() {
        let mut chain = create_empty_chain();

        // Add options with extreme OI concentration at one strike
        let strikes_data = [
            (pos_or_panic!(90.0), 100u64),
            (pos_or_panic!(95.0), 500u64),
            (Positive::HUNDRED, 100000u64), // Extreme concentration
            (pos_or_panic!(105.0), 500u64),
            (pos_or_panic!(110.0), 100u64),
        ];

        for (strike, oi) in strikes_data {
            let option_data = OptionData::new(
                strike,
                spos!(5.0),
                spos!(5.5),
                spos!(5.0),
                spos!(5.5),
                pos_or_panic!(0.20),
                None,
                None,
                None,
                spos!(1000.0),
                Some(oi),
                Some("TEST".to_string()),
                Some(ExpirationDate::Days(pos_or_panic!(30.0))),
                Some(Box::new(Positive::HUNDRED)),
                Some(dec!(0.05)),
                spos!(0.02),
                None,
                None,
            );
            chain.options.insert(option_data);
        }

        let curve = chain.open_interest_curve().unwrap();
        let points: Vec<_> = curve.points.iter().collect();

        // Max OI should be at strike 100
        let max_oi = points
            .iter()
            .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
            .unwrap();

        assert_eq!(max_oi.x, dec!(100.0));
        assert_eq!(max_oi.y, dec!(100000));
    }
}
