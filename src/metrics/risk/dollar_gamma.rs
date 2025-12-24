/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Dollar Gamma Metrics
//!
//! This module provides traits for computing dollar gamma curves, which measure
//! gamma exposure in monetary terms rather than as a pure sensitivity.
//!
//! ## Overview
//!
//! Dollar gamma (also known as gamma dollars or cash gamma) converts the
//! abstract gamma value into a concrete dollar amount, making it easier to
//! understand and manage gamma risk in portfolio terms.
//!
//! ## Formula
//!
//! ```text
//! Dollar Gamma = Gamma × Spot² × 0.01
//! ```
//!
//! This represents the dollar P&L change from gamma for a 1% move in the
//! underlying price.
//!
//! ## Interpretation
//!
//! - **High Dollar Gamma**: Large P&L swings from price movements
//! - **Low Dollar Gamma**: More stable P&L with respect to price moves
//! - **Positive Dollar Gamma**: Long gamma position (benefits from volatility)
//! - **Negative Dollar Gamma**: Short gamma position (hurt by volatility)
//!
//! ## Use Cases
//!
//! - **Risk Management**: Quantify gamma exposure in dollar terms
//! - **Position Sizing**: Determine appropriate hedge sizes
//! - **P&L Attribution**: Understand gamma contribution to P&L
//! - **Strike Selection**: Identify strikes with concentrated gamma exposure

use crate::curves::Curve;
use crate::error::CurveError;
use crate::model::OptionStyle;

/// A trait for computing dollar gamma curves by strike price.
///
/// Dollar gamma measures the gamma exposure in dollar terms, showing how
/// much the delta (and consequently P&L) will change for a 1% move in
/// the underlying price.
///
/// # Mathematical Background
///
/// For an option with gamma Γ at underlying price S:
/// ```text
/// Dollar Gamma = Γ × S² × 0.01
/// ```
///
/// This can be interpreted as:
/// - The dollar change in delta for a 1% move in spot
/// - The second-order P&L contribution from price movements
///
/// # Returns
///
/// A `Curve` where:
/// - **X-axis**: Strike price in currency units
/// - **Y-axis**: Dollar gamma value in currency units
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::DollarGammaCurve;
/// use optionstratlib::model::OptionStyle;
///
/// let chain = OptionChain::new("SPY", pos!(450.0), "2024-03-15".to_string(), None, None);
/// let dg_curve = chain.dollar_gamma_curve(&OptionStyle::Call)?;
///
/// // Find the strike with maximum dollar gamma exposure
/// let max_dg = dg_curve.points.iter()
///     .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
///     .unwrap();
/// println!("Max dollar gamma at strike {}: ${:.2}", max_dg.x, max_dg.y);
/// ```
///
/// # Practical Considerations
///
/// - Dollar gamma is typically highest for ATM options
/// - It decreases rapidly for deep ITM and OTM options
/// - Short-dated options have higher dollar gamma than long-dated ones
/// - Portfolio dollar gamma is the sum of individual position dollar gammas
pub trait DollarGammaCurve {
    /// Computes the dollar gamma curve by strike price.
    ///
    /// Calculates the dollar gamma for each strike in the option chain,
    /// using the formula: Dollar Gamma = Gamma × Spot² × 0.01
    ///
    /// # Parameters
    ///
    /// - `option_style`: Whether to compute dollar gamma for calls or puts.
    ///   Note that gamma is the same for calls and puts at the same strike,
    ///   but this parameter allows filtering by option type.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The dollar gamma curve with strike prices on x-axis
    ///   and dollar gamma values on y-axis
    /// - `Err(CurveError)`: If the curve cannot be computed
    ///
    /// # Errors
    ///
    /// Returns `CurveError::ConstructionError` if:
    /// - No valid gamma values can be computed
    /// - The option chain is empty
    /// - Required option data is missing
    fn dollar_gamma_curve(&self, option_style: &OptionStyle) -> Result<Curve, CurveError>;
}

#[cfg(test)]
mod tests_dollar_gamma_traits {
    use super::*;
    use crate::curves::Point2D;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    struct TestDollarGamma {
        spot: Decimal,
    }

    impl DollarGammaCurve for TestDollarGamma {
        fn dollar_gamma_curve(&self, _option_style: &OptionStyle) -> Result<Curve, CurveError> {
            let spot_squared = self.spot * self.spot;
            let mut points = BTreeSet::new();

            // Simulate gamma values (highest ATM, lower for OTM/ITM)
            let gammas = vec![
                (dec!(90.0), dec!(0.01)),
                (dec!(95.0), dec!(0.03)),
                (dec!(100.0), dec!(0.05)), // ATM - highest gamma
                (dec!(105.0), dec!(0.03)),
                (dec!(110.0), dec!(0.01)),
            ];

            for (strike, gamma) in gammas {
                let dollar_gamma = gamma * spot_squared * dec!(0.01);
                points.insert(Point2D::new(strike, dollar_gamma));
            }

            Ok(Curve::new(points))
        }
    }

    struct TestEmptyDollarGamma;

    impl DollarGammaCurve for TestEmptyDollarGamma {
        fn dollar_gamma_curve(&self, _option_style: &OptionStyle) -> Result<Curve, CurveError> {
            Err(CurveError::ConstructionError(
                "No valid gamma values computed".to_string(),
            ))
        }
    }

    #[test]
    fn test_dollar_gamma_implementation() {
        let dg = TestDollarGamma { spot: dec!(100.0) };
        let curve = dg.dollar_gamma_curve(&OptionStyle::Call).unwrap();

        assert_eq!(curve.points.len(), 5);

        let points: Vec<&Point2D> = curve.points.iter().collect();
        assert_eq!(points[0].x, dec!(90.0));
    }

    #[test]
    fn test_dollar_gamma_atm_highest() {
        let dg = TestDollarGamma { spot: dec!(100.0) };
        let curve = dg.dollar_gamma_curve(&OptionStyle::Call).unwrap();

        let points: Vec<&Point2D> = curve.points.iter().collect();

        // ATM (strike 100) should have highest dollar gamma
        let atm_dg = points.iter().find(|p| p.x == dec!(100.0)).unwrap().y;

        for point in points.iter() {
            assert!(point.y <= atm_dg);
        }
    }

    #[test]
    fn test_dollar_gamma_formula() {
        let spot = dec!(100.0);
        let dg = TestDollarGamma { spot };
        let curve = dg.dollar_gamma_curve(&OptionStyle::Call).unwrap();

        let points: Vec<&Point2D> = curve.points.iter().collect();

        // ATM gamma = 0.05, spot = 100
        // Dollar Gamma = 0.05 × 100² × 0.01 = 0.05 × 10000 × 0.01 = 5.0
        let atm_point = points.iter().find(|p| p.x == dec!(100.0)).unwrap();
        assert_eq!(atm_point.y, dec!(5.0));
    }

    #[test]
    fn test_dollar_gamma_spot_sensitivity() {
        let dg_low = TestDollarGamma { spot: dec!(50.0) };
        let dg_high = TestDollarGamma { spot: dec!(200.0) };

        let curve_low = dg_low.dollar_gamma_curve(&OptionStyle::Call).unwrap();
        let curve_high = dg_high.dollar_gamma_curve(&OptionStyle::Call).unwrap();

        let points_low: Vec<&Point2D> = curve_low.points.iter().collect();
        let points_high: Vec<&Point2D> = curve_high.points.iter().collect();

        // Higher spot should result in higher dollar gamma (spot² effect)
        let atm_low = points_low.iter().find(|p| p.x == dec!(100.0)).unwrap().y;
        let atm_high = points_high.iter().find(|p| p.x == dec!(100.0)).unwrap().y;

        assert!(atm_high > atm_low);
        // Ratio should be (200/50)² = 16
        assert_eq!(atm_high / atm_low, dec!(16));
    }

    #[test]
    fn test_dollar_gamma_empty_error() {
        let dg = TestEmptyDollarGamma;
        let result = dg.dollar_gamma_curve(&OptionStyle::Call);

        assert!(result.is_err());
        match result {
            Err(CurveError::ConstructionError(msg)) => {
                assert!(msg.contains("No valid gamma"));
            }
            _ => panic!("Expected ConstructionError"),
        }
    }

    #[test]
    fn test_dollar_gamma_symmetric() {
        let dg = TestDollarGamma { spot: dec!(100.0) };
        let curve = dg.dollar_gamma_curve(&OptionStyle::Call).unwrap();

        let points: Vec<&Point2D> = curve.points.iter().collect();

        // Dollar gamma should be symmetric around ATM
        let dg_90 = points.iter().find(|p| p.x == dec!(90.0)).unwrap().y;
        let dg_110 = points.iter().find(|p| p.x == dec!(110.0)).unwrap().y;

        assert_eq!(dg_90, dg_110);
    }

    #[test]
    fn test_dollar_gamma_call_vs_put() {
        let dg = TestDollarGamma { spot: dec!(100.0) };

        let call_curve = dg.dollar_gamma_curve(&OptionStyle::Call).unwrap();
        let put_curve = dg.dollar_gamma_curve(&OptionStyle::Put).unwrap();

        // Gamma is the same for calls and puts at same strike
        // So dollar gamma should be equal
        let call_points: Vec<&Point2D> = call_curve.points.iter().collect();
        let put_points: Vec<&Point2D> = put_curve.points.iter().collect();

        for (call_p, put_p) in call_points.iter().zip(put_points.iter()) {
            assert_eq!(call_p.y, put_p.y);
        }
    }
}
