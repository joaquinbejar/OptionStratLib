/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Risk Reversal Metrics
//!
//! This module provides traits for computing risk reversal curves, which measure
//! the difference between implied volatilities of out-of-the-money calls and puts.
//!
//! ## Overview
//!
//! Risk reversal (RR) is a key metric for understanding market sentiment and
//! the directional bias implied by option prices. It quantifies the relative
//! demand for upside versus downside protection.
//!
//! ## Formula
//!
//! ```text
//! Risk Reversal = IV(Call) - IV(Put)
//! ```
//!
//! At the same strike price or equivalent delta.
//!
//! ## Interpretation
//!
//! - **Positive RR**: Calls are more expensive than puts, suggesting bullish
//!   sentiment or expectations of upside moves
//! - **Negative RR**: Puts are more expensive than calls, suggesting bearish
//!   sentiment or demand for downside protection
//! - **Zero RR**: Balanced market with no directional bias
//!
//! ## Use Cases
//!
//! - **Sentiment Analysis**: Gauge market expectations for directional moves
//! - **Skew Trading**: Identify opportunities in volatility skew
//! - **Hedging Decisions**: Understand relative costs of upside vs downside protection
//! - **Risk Management**: Monitor changes in market sentiment over time

use crate::curves::Curve;
use crate::error::CurveError;

/// A trait for computing risk reversal curves by strike price.
///
/// Risk reversal measures the difference between implied volatilities of
/// calls and puts at the same strike price. This metric is widely used
/// in options trading to assess market sentiment and volatility skew.
///
/// # Mathematical Background
///
/// For each strike price K:
/// ```text
/// RR(K) = IV_call(K) - IV_put(K)
/// ```
///
/// In practice, risk reversals are often quoted at specific deltas
/// (e.g., 25-delta risk reversal), but this implementation uses
/// strike prices directly.
///
/// # Returns
///
/// A `Curve` where:
/// - **X-axis**: Strike price in currency units
/// - **Y-axis**: Risk reversal value (Call IV - Put IV) as a decimal
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::RiskReversalCurve;
///
/// let chain = OptionChain::new("SPY", pos!(450.0), "2024-03-15".to_string(), None, None);
/// let rr_curve = chain.risk_reversal_curve()?;
///
/// // Analyze market sentiment
/// for point in rr_curve.points.iter() {
///     let sentiment = if point.y > Decimal::ZERO { "bullish" } else { "bearish" };
///     println!("Strike {}: RR = {:.4} ({})", point.x, point.y, sentiment);
/// }
/// ```
///
/// # Market Conventions
///
/// In FX markets, risk reversals are typically quoted as:
/// - 25-delta RR: IV(25Δ call) - IV(25Δ put)
/// - 10-delta RR: IV(10Δ call) - IV(10Δ put)
///
/// In equity markets, the skew is often negative (puts more expensive),
/// reflecting demand for downside protection.
pub trait RiskReversalCurve {
    /// Computes the risk reversal curve by strike price.
    ///
    /// For each strike in the option chain, calculates the difference
    /// between call implied volatility and put implied volatility.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The risk reversal curve with strike prices on x-axis
    ///   and RR values on y-axis
    /// - `Err(CurveError)`: If the curve cannot be computed
    ///
    /// # Errors
    ///
    /// Returns `CurveError::ConstructionError` if:
    /// - No options have both call and put IV available
    /// - The option chain is empty
    /// - All strikes have missing IV data for either calls or puts
    fn risk_reversal_curve(&self) -> Result<Curve, CurveError>;
}

#[cfg(test)]
mod tests_risk_reversal_traits {
    use super::*;
    use crate::curves::Point2D;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    struct TestRiskReversal;

    impl RiskReversalCurve for TestRiskReversal {
        fn risk_reversal_curve(&self) -> Result<Curve, CurveError> {
            let mut points = BTreeSet::new();
            // Typical equity skew: negative RR (puts more expensive)
            points.insert(Point2D::new(dec!(90.0), dec!(-0.05)));
            points.insert(Point2D::new(dec!(95.0), dec!(-0.03)));
            points.insert(Point2D::new(dec!(100.0), dec!(-0.01)));
            points.insert(Point2D::new(dec!(105.0), dec!(0.00)));
            points.insert(Point2D::new(dec!(110.0), dec!(0.01)));
            Ok(Curve::new(points))
        }
    }

    struct TestPositiveRiskReversal;

    impl RiskReversalCurve for TestPositiveRiskReversal {
        fn risk_reversal_curve(&self) -> Result<Curve, CurveError> {
            let mut points = BTreeSet::new();
            // Bullish market: positive RR (calls more expensive)
            points.insert(Point2D::new(dec!(90.0), dec!(0.01)));
            points.insert(Point2D::new(dec!(95.0), dec!(0.02)));
            points.insert(Point2D::new(dec!(100.0), dec!(0.03)));
            points.insert(Point2D::new(dec!(105.0), dec!(0.04)));
            points.insert(Point2D::new(dec!(110.0), dec!(0.05)));
            Ok(Curve::new(points))
        }
    }

    struct TestEmptyRiskReversal;

    impl RiskReversalCurve for TestEmptyRiskReversal {
        fn risk_reversal_curve(&self) -> Result<Curve, CurveError> {
            Err(CurveError::ConstructionError(
                "No options with both call and put IV available".to_string(),
            ))
        }
    }

    #[test]
    fn test_risk_reversal_implementation() {
        let rr = TestRiskReversal;
        let curve = rr.risk_reversal_curve().unwrap();

        assert_eq!(curve.points.len(), 5);

        let points: Vec<&Point2D> = curve.points.iter().collect();
        assert_eq!(points[0].x, dec!(90.0));
        assert_eq!(points[0].y, dec!(-0.05));
    }

    #[test]
    fn test_risk_reversal_negative_skew() {
        let rr = TestRiskReversal;
        let curve = rr.risk_reversal_curve().unwrap();

        let points: Vec<&Point2D> = curve.points.iter().collect();

        // Lower strikes should have more negative RR (puts more expensive)
        assert!(points[0].y < points[4].y);
    }

    #[test]
    fn test_risk_reversal_positive_skew() {
        let rr = TestPositiveRiskReversal;
        let curve = rr.risk_reversal_curve().unwrap();

        let points: Vec<&Point2D> = curve.points.iter().collect();

        // All RR values should be positive
        for point in points.iter() {
            assert!(point.y > rust_decimal::Decimal::ZERO);
        }
    }

    #[test]
    fn test_risk_reversal_empty_error() {
        let rr = TestEmptyRiskReversal;
        let result = rr.risk_reversal_curve();

        assert!(result.is_err());
        match result {
            Err(CurveError::ConstructionError(msg)) => {
                assert!(msg.contains("No options"));
            }
            _ => panic!("Expected ConstructionError"),
        }
    }

    #[test]
    fn test_risk_reversal_atm_near_zero() {
        let rr = TestRiskReversal;
        let curve = rr.risk_reversal_curve().unwrap();

        let points: Vec<&Point2D> = curve.points.iter().collect();

        // ATM (strike 100) should have RR close to zero
        let atm_point = points.iter().find(|p| p.x == dec!(100.0)).unwrap();
        assert!(atm_point.y.abs() < dec!(0.02));
    }

    #[test]
    fn test_risk_reversal_monotonic_increase() {
        let rr = TestRiskReversal;
        let curve = rr.risk_reversal_curve().unwrap();

        let points: Vec<&Point2D> = curve.points.iter().collect();

        // RR should generally increase with strike (in typical equity skew)
        for i in 1..points.len() {
            assert!(points[i].y >= points[i - 1].y);
        }
    }
}
