/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Price Shock Impact Metrics
//!
//! This module provides traits for computing price shock impact curves and
//! surfaces, which are essential for understanding how options respond to
//! sudden price movements.
//!
//! ## Overview
//!
//! Price shock analysis evaluates the impact of sudden price movements on
//! option values. This is crucial for:
//!
//! - **Stress testing**: Understanding worst-case scenarios
//! - **Risk management**: Quantifying tail risk exposure
//! - **Hedging**: Ensuring adequate protection
//!
//! ## Mathematical Background
//!
//! The P&L from a price shock is approximated using Taylor expansion:
//!
//! ```text
//! P&L ≈ Delta × ΔS + 0.5 × Gamma × ΔS² + Vanna × ΔS × Δσ
//! ```
//!
//! where:
//! - Delta: First-order price sensitivity
//! - Gamma: Second-order price sensitivity
//! - Vanna: Cross-sensitivity to price and volatility
//! - ΔS: Price change
//! - Δσ: Volatility change (often correlated with price moves)
//!
//! ## Curve Representation
//!
//! The curve shows P&L impact by strike for a given shock:
//! - **X-axis**: Strike price
//! - **Y-axis**: P&L from price shock
//!
//! ## Surface Representation
//!
//! The surface shows P&L across price and volatility shocks:
//! - **X-axis**: Underlying price (or price shock %)
//! - **Y-axis**: Volatility level (or vol shock %)
//! - **Z-axis**: P&L or option value

use crate::curves::Curve;
use crate::error::CurveError;
use crate::error::SurfaceError;
use crate::surfaces::Surface;
use positive::Positive;
use rust_decimal::Decimal;

/// A trait for computing price shock impact curves by strike price.
///
/// The curve shows how a price shock affects options at each strike,
/// helping identify where price risk is concentrated.
///
/// # Returns
///
/// A `Curve` where:
/// - **X-axis**: Strike price in currency units
/// - **Y-axis**: P&L from the specified price shock
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::PriceShockCurve;
/// use rust_decimal_macros::dec;
///
/// let chain = OptionChain::load_from_json("options.json")?;
/// let shock_pct = dec!(-0.10); // -10% price shock
/// let shock_curve = chain.price_shock_curve(shock_pct)?;
/// ```
pub trait PriceShockCurve {
    /// Computes the price shock impact curve by strike price.
    ///
    /// # Parameters
    ///
    /// - `shock_pct`: Price shock as a decimal (e.g., -0.10 for -10%)
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The shock curve with strike on x-axis and P&L on y-axis
    /// - `Err(CurveError)`: If the curve cannot be computed
    ///
    /// # Errors
    ///
    /// Returns `CurveError::ConstructionError` if:
    /// - No options have valid delta/gamma values
    /// - The option chain is empty
    fn price_shock_curve(&self, shock_pct: Decimal) -> Result<Curve, CurveError>;
}

/// A trait for computing price shock impact surfaces.
///
/// The surface shows P&L across combined price and volatility shocks,
/// providing a complete stress testing view.
///
/// # Returns
///
/// A `Surface` where:
/// - **X-axis**: Underlying price in currency units
/// - **Y-axis**: Implied volatility as a decimal
/// - **Z-axis**: Option value or P&L
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::PriceShockSurface;
/// use positive::pos_or_panic;
///
/// let chain = OptionChain::load_from_json("options.json")?;
/// let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
/// let vol_range = (pos_or_panic!(0.10), pos_or_panic!(0.50));
/// let surface = chain.price_shock_surface(price_range, vol_range, 20, 20)?;
/// ```
pub trait PriceShockSurface {
    /// Computes the price shock impact surface (price vs volatility).
    ///
    /// # Parameters
    ///
    /// - `price_range`: Tuple of (min_price, max_price) for the underlying
    /// - `vol_range`: Tuple of (min_vol, max_vol) for implied volatility
    /// - `price_steps`: Number of steps along the price axis
    /// - `vol_steps`: Number of steps along the volatility axis
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: The shock surface with price on x-axis,
    ///   volatility on y-axis, and option value on z-axis
    /// - `Err(SurfaceError)`: If the surface cannot be computed
    fn price_shock_surface(
        &self,
        price_range: (Positive, Positive),
        vol_range: (Positive, Positive),
        price_steps: usize,
        vol_steps: usize,
    ) -> Result<Surface, SurfaceError>;
}

#[cfg(test)]
mod tests_price_shock {
    use super::*;
    use crate::curves::Point2D;

    use crate::surfaces::Point3D;
    use positive::pos_or_panic;
    use rust_decimal::MathematicalOps;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    struct TestPriceShock {
        underlying_price: Positive,
    }

    impl PriceShockCurve for TestPriceShock {
        fn price_shock_curve(&self, shock_pct: Decimal) -> Result<Curve, CurveError> {
            let mut points = BTreeSet::new();
            let spot = self.underlying_price.to_dec();
            let price_move = spot * shock_pct;

            let strikes = [
                dec!(380.0),
                dec!(400.0),
                dec!(420.0),
                dec!(440.0),
                dec!(450.0),
                dec!(460.0),
                dec!(480.0),
                dec!(500.0),
                dec!(520.0),
            ];

            for strike in strikes {
                // Simplified delta/gamma model
                let moneyness = (spot - strike) / spot;
                let delta = dec!(0.5) + moneyness * dec!(2.0);
                let delta = delta.max(dec!(0.0)).min(dec!(1.0));

                // Gamma peaks at ATM
                let gamma = dec!(0.02) * (-((strike - spot) / dec!(30.0)).powi(2)).exp();

                // P&L = Delta × ΔS + 0.5 × Gamma × ΔS²
                let pnl = delta * price_move + dec!(0.5) * gamma * price_move * price_move;

                points.insert(Point2D::new(strike, pnl));
            }

            Ok(Curve::new(points))
        }
    }

    impl PriceShockSurface for TestPriceShock {
        fn price_shock_surface(
            &self,
            price_range: (Positive, Positive),
            vol_range: (Positive, Positive),
            price_steps: usize,
            vol_steps: usize,
        ) -> Result<Surface, SurfaceError> {
            let mut points = BTreeSet::new();

            let price_step = if price_steps > 0 {
                (price_range.1 - price_range.0).to_dec() / Decimal::from(price_steps)
            } else {
                Decimal::ZERO
            };

            let vol_step = if vol_steps > 0 {
                (vol_range.1 - vol_range.0).to_dec() / Decimal::from(vol_steps)
            } else {
                Decimal::ZERO
            };

            let strike = self.underlying_price.to_dec();

            for p in 0..=price_steps {
                let price = price_range.0.to_dec() + price_step * Decimal::from(p);

                for v in 0..=vol_steps {
                    let vol = vol_range.0.to_dec() + vol_step * Decimal::from(v);

                    // Simplified option value under shock scenario
                    let intrinsic = (price - strike).max(Decimal::ZERO);
                    let time_value = vol * price * dec!(0.1);
                    let option_value = intrinsic + time_value;

                    points.insert(Point3D::new(price, vol, option_value));
                }
            }

            Ok(Surface::new(points))
        }
    }

    #[test]
    fn test_price_shock_curve_creation() {
        let ps = TestPriceShock {
            underlying_price: pos_or_panic!(450.0),
        };
        let curve = ps.price_shock_curve(dec!(-0.10));
        assert!(curve.is_ok());

        let curve = curve.unwrap();
        assert_eq!(curve.points.len(), 9);
    }

    #[test]
    fn test_price_shock_curve_negative_shock() {
        let ps = TestPriceShock {
            underlying_price: pos_or_panic!(450.0),
        };
        let curve = ps.price_shock_curve(dec!(-0.10)).unwrap();

        let points: Vec<&Point2D> = curve.points.iter().collect();

        // For calls with negative shock, ITM calls lose more
        // Lower strikes (more ITM) should have more negative P&L
        let itm_pnl = points.iter().find(|p| p.x == dec!(380.0)).map(|p| p.y);
        let otm_pnl = points.iter().find(|p| p.x == dec!(520.0)).map(|p| p.y);

        if let (Some(itm), Some(otm)) = (itm_pnl, otm_pnl) {
            // ITM should lose more (more negative P&L) than OTM
            assert!(itm < otm);
        }
    }

    #[test]
    fn test_price_shock_curve_positive_shock() {
        let ps = TestPriceShock {
            underlying_price: pos_or_panic!(450.0),
        };
        let curve = ps.price_shock_curve(dec!(0.10)).unwrap();

        let points: Vec<&Point2D> = curve.points.iter().collect();

        // For calls with positive shock, ITM calls gain more
        let itm_pnl = points.iter().find(|p| p.x == dec!(380.0)).map(|p| p.y);
        let otm_pnl = points.iter().find(|p| p.x == dec!(520.0)).map(|p| p.y);

        if let (Some(itm), Some(otm)) = (itm_pnl, otm_pnl) {
            // ITM should gain more than OTM
            assert!(itm > otm);
        }
    }

    #[test]
    fn test_price_shock_surface_creation() {
        let ps = TestPriceShock {
            underlying_price: pos_or_panic!(450.0),
        };
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let vol_range = (pos_or_panic!(0.10), pos_or_panic!(0.40));

        let surface = ps.price_shock_surface(price_range, vol_range, 10, 10);
        assert!(surface.is_ok());

        let surface = surface.unwrap();
        // (10+1) × (10+1) = 121 points
        assert_eq!(surface.points.len(), 121);
    }

    #[test]
    fn test_price_shock_surface_price_effect() {
        let ps = TestPriceShock {
            underlying_price: pos_or_panic!(450.0),
        };
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let vol_range = (pos_or_panic!(0.20), pos_or_panic!(0.20)); // Fixed vol

        let surface = ps
            .price_shock_surface(price_range, vol_range, 10, 0)
            .unwrap();

        let points: Vec<&Point3D> = surface.points.iter().collect();

        // Higher price should mean higher call value
        for i in 1..points.len() {
            assert!(points[i].z >= points[i - 1].z);
        }
    }

    #[test]
    fn test_price_shock_surface_vol_effect() {
        let ps = TestPriceShock {
            underlying_price: pos_or_panic!(450.0),
        };
        let price_range = (pos_or_panic!(450.0), pos_or_panic!(450.0)); // Fixed price (ATM)
        let vol_range = (pos_or_panic!(0.10), pos_or_panic!(0.40));

        let surface = ps
            .price_shock_surface(price_range, vol_range, 0, 10)
            .unwrap();

        let points: Vec<&Point3D> = surface.points.iter().collect();

        // Higher vol should mean higher option value
        for i in 1..points.len() {
            assert!(points[i].z >= points[i - 1].z);
        }
    }
}
