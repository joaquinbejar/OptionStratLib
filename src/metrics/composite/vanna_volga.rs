/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Vanna-Volga Hedge Metrics
//!
//! This module provides traits for computing Vanna-Volga hedge surfaces,
//! which are essential for understanding volatility hedging costs and
//! smile-consistent option pricing.
//!
//! ## Overview
//!
//! The Vanna-Volga method is a pricing and hedging technique that accounts
//! for the volatility smile by using three vanilla options to hedge:
//!
//! - **Vega risk**: Sensitivity to volatility level changes
//! - **Vanna risk**: Sensitivity to correlation between spot and volatility
//! - **Volga risk**: Sensitivity to volatility of volatility
//!
//! ## Mathematical Background
//!
//! The Vanna-Volga adjustment to Black-Scholes price is:
//!
//! ```text
//! Price_VV = Price_BS + Vanna × (σ_market - σ_ATM) × S × √T
//!          + 0.5 × Volga × (σ_market - σ_ATM)²
//! ```
//!
//! The hedge cost surface shows how these adjustments vary across:
//! - Different underlying prices (x-axis)
//! - Different volatility levels (y-axis)
//!
//! ## Surface Representation
//!
//! The surface provides:
//! - **X-axis**: Underlying price
//! - **Y-axis**: Implied volatility level
//! - **Z-axis**: Vanna-Volga hedge cost/adjustment
//!
//! This helps traders understand:
//! - Where hedging costs are highest
//! - How smile effects impact pricing at different spots
//! - Optimal hedge ratios across the price-vol space

use crate::Positive;
use crate::error::SurfaceError;
use crate::surfaces::Surface;

/// A trait for computing Vanna-Volga hedge surfaces.
///
/// The Vanna-Volga surface shows hedge costs as a function of both
/// underlying price and volatility level, providing insights into
/// smile-consistent hedging strategies.
///
/// # Mathematical Background
///
/// The Vanna-Volga method uses three benchmark options (typically 25-delta
/// put, ATM, and 25-delta call) to construct a hedge portfolio that
/// neutralizes vega, vanna, and volga risks.
///
/// The hedge cost at any point (S, σ) is computed as:
///
/// ```text
/// Cost = w₁ × (σ₁ - σ_BS) + w₂ × (σ₂ - σ_BS) + w₃ × (σ₃ - σ_BS)
/// ```
///
/// where wᵢ are the hedge weights derived from the Greeks.
///
/// # Returns
///
/// A `Surface` where:
/// - **X-axis**: Underlying price in currency units
/// - **Y-axis**: Implied volatility as a decimal (e.g., 0.20 for 20%)
/// - **Z-axis**: Vanna-Volga hedge cost/adjustment
///
/// # Example
///
/// ```ignore
/// use optionstratlib::chains::chain::OptionChain;
/// use optionstratlib::metrics::VannaVolgaSurface;
/// use optionstratlib::pos_or_panic;
///
/// let chain = OptionChain::load_from_json("options.json")?;
/// let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
/// let vol_range = (pos_or_panic!(0.10), pos_or_panic!(0.40));
/// let surface = chain.vanna_volga_surface(price_range, vol_range, 20, 20)?;
/// ```
pub trait VannaVolgaSurface {
    /// Computes the Vanna-Volga hedge surface (price vs volatility).
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
    /// - `Ok(Surface)`: The Vanna-Volga surface with price on x-axis,
    ///   volatility on y-axis, and hedge cost on z-axis
    /// - `Err(SurfaceError)`: If the surface cannot be computed
    ///
    /// # Errors
    ///
    /// Returns `SurfaceError::ConstructionError` if:
    /// - The option chain lacks sufficient data for hedge calculation
    /// - Price or volatility ranges are invalid
    /// - No valid surface points can be generated
    fn vanna_volga_surface(
        &self,
        price_range: (Positive, Positive),
        vol_range: (Positive, Positive),
        price_steps: usize,
        vol_steps: usize,
    ) -> Result<Surface, SurfaceError>;
}

#[cfg(test)]
mod tests_vanna_volga {
    use super::*;

    use crate::surfaces::Point3D;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;
    use positive::pos_or_panic;

    struct TestVannaVolgaSurface;

    impl VannaVolgaSurface for TestVannaVolgaSurface {
        fn vanna_volga_surface(
            &self,
            price_range: (Positive, Positive),
            vol_range: (Positive, Positive),
            price_steps: usize,
            vol_steps: usize,
        ) -> Result<Surface, SurfaceError> {
            let mut points = BTreeSet::new();

            let price_step = if price_steps > 0 {
                (price_range.1 - price_range.0).to_dec() / rust_decimal::Decimal::from(price_steps)
            } else {
                rust_decimal::Decimal::ZERO
            };
            let vol_step = if vol_steps > 0 {
                (vol_range.1 - vol_range.0).to_dec() / rust_decimal::Decimal::from(vol_steps)
            } else {
                rust_decimal::Decimal::ZERO
            };

            for p in 0..=price_steps {
                let price = price_range.0.to_dec() + price_step * rust_decimal::Decimal::from(p);
                for v in 0..=vol_steps {
                    let vol = vol_range.0.to_dec() + vol_step * rust_decimal::Decimal::from(v);

                    // Simplified Vanna-Volga cost model for testing
                    // Cost increases with distance from ATM and with volatility
                    let atm_price = dec!(450.0);
                    let moneyness = (price - atm_price).abs() / atm_price;
                    let vv_cost = moneyness * vol * dec!(100.0);

                    points.insert(Point3D::new(price, vol, vv_cost));
                }
            }

            Ok(Surface::new(points))
        }
    }

    #[test]
    fn test_vanna_volga_surface_creation() {
        let vv = TestVannaVolgaSurface;
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let vol_range = (pos_or_panic!(0.10), pos_or_panic!(0.40));

        let surface = vv.vanna_volga_surface(price_range, vol_range, 10, 10);
        assert!(surface.is_ok());

        let surface = surface.unwrap();
        // (10+1) × (10+1) = 121 points
        assert_eq!(surface.points.len(), 121);
    }

    #[test]
    fn test_vanna_volga_surface_cost_increases_with_moneyness() {
        let vv = TestVannaVolgaSurface;
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(500.0));
        let vol_range = (pos_or_panic!(0.20), pos_or_panic!(0.20)); // Fixed vol

        let surface = vv
            .vanna_volga_surface(price_range, vol_range, 10, 0)
            .unwrap();

        let points: Vec<&Point3D> = surface.points.iter().collect();

        // Find ATM point (around 450)
        let atm_point = points.iter().min_by(|a, b| {
            let a_dist = (a.x - dec!(450.0)).abs();
            let b_dist = (b.x - dec!(450.0)).abs();
            a_dist.partial_cmp(&b_dist).unwrap()
        });

        // ATM should have lowest cost
        if let Some(atm) = atm_point {
            for point in points.iter() {
                assert!(point.z >= atm.z || (point.z - atm.z).abs() < dec!(0.01));
            }
        }
    }

    #[test]
    fn test_vanna_volga_surface_cost_increases_with_volatility() {
        let vv = TestVannaVolgaSurface;
        let price_range = (pos_or_panic!(400.0), pos_or_panic!(400.0)); // Fixed price (OTM)
        let vol_range = (pos_or_panic!(0.10), pos_or_panic!(0.40));

        let surface = vv
            .vanna_volga_surface(price_range, vol_range, 0, 10)
            .unwrap();

        let points: Vec<&Point3D> = surface.points.iter().collect();

        // Cost should increase with volatility for OTM options
        for i in 1..points.len() {
            assert!(points[i].z >= points[i - 1].z);
        }
    }
}
