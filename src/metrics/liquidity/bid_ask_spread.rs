/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Bid-Ask Spread Metrics
//!
//! This module provides traits for computing bid-ask spread curves,
//! which are essential for understanding market liquidity and transaction costs.
//!
//! ## Overview
//!
//! The bid-ask spread is the difference between the highest price a buyer is
//! willing to pay (bid) and the lowest price a seller is willing to accept (ask).
//! It serves as a key indicator of:
//!
//! - **Market liquidity**: Tighter spreads indicate more liquid markets
//! - **Transaction costs**: Wider spreads mean higher costs to trade
//! - **Market maker activity**: Active market makers typically narrow spreads
//!
//! ## Mathematical Background
//!
//! Two common spread measures are provided:
//!
//! ### Absolute Spread
//! ```text
//! Absolute Spread = Ask - Bid
//! ```
//!
//! ### Relative Spread (as percentage of mid price)
//! ```text
//! Relative Spread = (Ask - Bid) / Mid × 100%
//! where Mid = (Bid + Ask) / 2
//! ```
//!
//! ## Curve Representation
//!
//! The curve shows spread across strike prices:
//! - **X-axis**: Strike price
//! - **Y-axis**: Spread (absolute or relative)
//!
//! Typical patterns:
//! - Tighter spreads near ATM (more liquid)
//! - Wider spreads for deep OTM/ITM options (less liquid)

use crate::curves::Curve;
use crate::error::CurveError;

/// A trait for computing bid-ask spread curves by strike price.
///
/// The spread curve shows how liquidity varies across different strike
/// prices, helping identify the most cost-effective strikes for trading.
///
/// # Mathematical Background
///
/// For each strike, the spread is calculated as:
/// - **Call spread**: (Call Ask - Call Bid) / Call Mid
/// - **Put spread**: (Put Ask - Put Bid) / Put Mid
/// - **Combined**: Average of call and put spreads
///
/// # Returns
///
/// A `Curve` where:
/// - **X-axis**: Strike price in currency units
/// - **Y-axis**: Relative spread as a decimal (e.g., 0.05 for 5%)
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::BidAskSpreadCurve;
///
/// let chain = OptionChain::load_from_json("options.json")?;
/// let spread_curve = chain.bid_ask_spread_curve()?;
///
/// // Find strike with tightest spread (most liquid)
/// let min_spread = spread_curve.points.iter()
///     .min_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
/// ```
pub trait BidAskSpreadCurve {
    /// Computes the bid-ask spread curve by strike price.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The spread curve with strike on x-axis and
    ///   relative spread on y-axis
    /// - `Err(CurveError)`: If the curve cannot be computed
    ///
    /// # Errors
    ///
    /// Returns `CurveError::ConstructionError` if:
    /// - No options have valid bid/ask data
    /// - The option chain is empty
    fn bid_ask_spread_curve(&self) -> Result<Curve, CurveError>;
}

#[cfg(test)]
mod tests_bid_ask_spread {
    use super::*;
    use crate::curves::Point2D;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    struct TestBidAskSpread;

    impl BidAskSpreadCurve for TestBidAskSpread {
        fn bid_ask_spread_curve(&self) -> Result<Curve, CurveError> {
            let mut points = BTreeSet::new();

            // Simulate typical spread pattern: tighter near ATM
            let strikes = [
                (dec!(380.0), dec!(0.08)), // Deep OTM - wide spread
                (dec!(400.0), dec!(0.05)), // OTM - moderate spread
                (dec!(420.0), dec!(0.03)), // Near ATM - tight spread
                (dec!(440.0), dec!(0.02)), // ATM - tightest spread
                (dec!(450.0), dec!(0.02)), // ATM - tightest spread
                (dec!(460.0), dec!(0.02)), // ATM - tightest spread
                (dec!(480.0), dec!(0.03)), // Near ATM - tight spread
                (dec!(500.0), dec!(0.05)), // OTM - moderate spread
                (dec!(520.0), dec!(0.08)), // Deep OTM - wide spread
            ];

            for (strike, spread) in strikes {
                points.insert(Point2D::new(strike, spread));
            }

            Ok(Curve::new(points))
        }
    }

    #[test]
    fn test_bid_ask_spread_curve_creation() {
        let spread = TestBidAskSpread;
        let curve = spread.bid_ask_spread_curve();
        assert!(curve.is_ok());

        let curve = curve.unwrap();
        assert_eq!(curve.points.len(), 9);
    }

    #[test]
    fn test_bid_ask_spread_atm_tightest() {
        let spread = TestBidAskSpread;
        let curve = spread.bid_ask_spread_curve().unwrap();

        let points: Vec<&Point2D> = curve.points.iter().collect();

        // Find minimum spread
        let min_spread = points.iter().min_by(|a, b| a.y.partial_cmp(&b.y).unwrap());

        if let Some(min) = min_spread {
            // ATM should have tightest spread (around 450)
            assert!(min.x >= dec!(440.0) && min.x <= dec!(460.0));
            assert_eq!(min.y, dec!(0.02));
        }
    }

    #[test]
    fn test_bid_ask_spread_otm_wider() {
        let spread = TestBidAskSpread;
        let curve = spread.bid_ask_spread_curve().unwrap();

        let points: Vec<&Point2D> = curve.points.iter().collect();

        // Deep OTM should have wider spreads
        let deep_otm_low = points.iter().find(|p| p.x == dec!(380.0)).unwrap();
        let deep_otm_high = points.iter().find(|p| p.x == dec!(520.0)).unwrap();
        let atm = points.iter().find(|p| p.x == dec!(450.0)).unwrap();

        assert!(deep_otm_low.y > atm.y);
        assert!(deep_otm_high.y > atm.y);
    }

    #[test]
    fn test_bid_ask_spread_positive_values() {
        let spread = TestBidAskSpread;
        let curve = spread.bid_ask_spread_curve().unwrap();

        // All spreads should be positive
        for point in curve.points.iter() {
            assert!(point.y > rust_decimal::Decimal::ZERO);
        }
    }
}
