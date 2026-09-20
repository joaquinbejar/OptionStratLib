//! [`crate::chains::OptionChain`] projections: `impl BasicCurves for OptionChain`,
//! `impl BasicSurfaces for OptionChain`, and the inherent convenience
//! wrappers (`gamma_curve`, `vanna_surface`, `theta_time_surface`, ...)
//! that call them.
//!
//! The traits are analytics-owned and `OptionChain` is market-owned, so the
//! impls live here rather than in `chains::chain` (local trait, foreign type:
//! coherent), and `chains` never imports `analytics` (ADR-0001 D4, M1-05).
//!
//! The inherent wrappers are placed here, not in `chains::chain`, so that the
//! market module carries no production reference to the projection traits.
//! Rust allows an inherent `impl` in any module of the defining crate, so
//! every `OptionChain::gamma_curve`-style path is unchanged. When analytics
//! becomes its own crate those wrappers become an extension trait; that
//! public-shape change is batched behind the 0.22.0 bump.

use crate::analytics::projections::{BasicCurves, BasicSurfaces};
use crate::chains::OptionChain;
use crate::curves::{Curve, Point2D};
use crate::error::ProjectionError;
use crate::model::{BasicAxisTypes, OptionStyle, Options, Side};
use crate::surfaces::{Point3D, Surface};
use positive::Positive;
use std::collections::BTreeSet;
use std::sync::Arc;

impl BasicCurves for OptionChain {
    fn curve(
        &self,
        axis: &BasicAxisTypes,
        option_style: &OptionStyle,
        side: &Side,
    ) -> Result<Curve, ProjectionError> {
        if axis == &BasicAxisTypes::UnderlyingPrice
            || axis == &BasicAxisTypes::Strike
            || axis == &BasicAxisTypes::Expiration
        {
            return Err(ProjectionError::UnsupportedAxis {
                axis: format!("{axis:?}"),
            });
        }
        let points = self
            .get_single_iter()
            .filter_map(|opt| {
                // Select the appropriate option based on style and side
                let option = match (option_style, side) {
                    (OptionStyle::Call, Side::Long) => {
                        opt.get_option(Side::Long, OptionStyle::Call)
                    }
                    (OptionStyle::Call, Side::Short) => {
                        opt.get_option(Side::Short, OptionStyle::Call)
                    }
                    (OptionStyle::Put, Side::Long) => opt.get_option(Side::Long, OptionStyle::Put),
                    (OptionStyle::Put, Side::Short) => {
                        opt.get_option(Side::Short, OptionStyle::Put)
                    }
                };
                let option: Arc<Options> = match option {
                    Ok(o) => Arc::new(o),
                    Err(_) => return None, // Skip options that cannot be retrieved
                };

                // Get x and y values based on the axis types
                match self.get_curve_strike_versus(axis, &option) {
                    Ok(point) => Some(Point2D::new(point.0, point.1)),
                    Err(_) => None,
                }
            })
            .collect();

        Ok(Curve::new(points))
    }
}

impl BasicSurfaces for OptionChain {
    fn surface(
        &self,
        axis: &BasicAxisTypes,
        option_style: &OptionStyle,
        volatility: Option<Vec<Positive>>,
        side: &Side,
    ) -> Result<Surface, ProjectionError> {
        if axis == &BasicAxisTypes::UnderlyingPrice
            || axis == &BasicAxisTypes::Strike
            || axis == &BasicAxisTypes::Expiration
        {
            return Err(ProjectionError::UnsupportedAxis {
                axis: format!("{axis:?}"),
            });
        }

        let mut points = BTreeSet::new();

        for opt in self.get_single_iter() {
            // Select the appropriate option based on style and side
            let option = match (option_style, side) {
                (OptionStyle::Call, Side::Long) => opt.get_option(Side::Long, OptionStyle::Call),
                (OptionStyle::Call, Side::Short) => opt.get_option(Side::Short, OptionStyle::Call),
                (OptionStyle::Put, Side::Long) => opt.get_option(Side::Long, OptionStyle::Put),
                (OptionStyle::Put, Side::Short) => opt.get_option(Side::Short, OptionStyle::Put),
            };
            let option: Arc<Options> = match option {
                Ok(o) => Arc::new(o),
                Err(_) => {
                    return Err(ProjectionError::MissingOptionData);
                }
            };

            match &volatility {
                // If volatility vector is provided, use get_volatility_versus for each volatility
                Some(vols) => {
                    for vol in vols {
                        match self.get_surface_volatility_versus(axis, &option, *vol) {
                            Ok((x, y, z)) => {
                                points.insert(Point3D::new(x, y, z));
                            }
                            Err(_) => continue,
                        }
                    }
                }
                // If no volatility vector is provided, use get_strike_versus with original volatility
                None => match self.get_surface_strike_versus(axis, &option) {
                    Ok((x, y, z)) => {
                        points.insert(Point3D::new(x, y, z));
                    }
                    Err(_) => continue,
                },
            }
        }

        if points.is_empty() {
            return Err(ProjectionError::NoPoints { kind: "surface" });
        }

        Ok(Surface::new(points))
    }

    fn time_surface(
        &self,
        axis: &BasicAxisTypes,
        option_style: &OptionStyle,
        days_to_expiry: Vec<Positive>,
        side: &Side,
    ) -> Result<Surface, ProjectionError> {
        if axis == &BasicAxisTypes::UnderlyingPrice
            || axis == &BasicAxisTypes::Strike
            || axis == &BasicAxisTypes::Expiration
            || axis == &BasicAxisTypes::Volatility
        {
            return Err(ProjectionError::UnsupportedAxis {
                axis: format!("{axis:?}"),
            });
        }

        let mut points = BTreeSet::new();

        for opt in self.get_single_iter() {
            let option = match (option_style, side) {
                (OptionStyle::Call, Side::Long) => opt.get_option(Side::Long, OptionStyle::Call),
                (OptionStyle::Call, Side::Short) => opt.get_option(Side::Short, OptionStyle::Call),
                (OptionStyle::Put, Side::Long) => opt.get_option(Side::Long, OptionStyle::Put),
                (OptionStyle::Put, Side::Short) => opt.get_option(Side::Short, OptionStyle::Put),
            };
            let option: Arc<Options> = match option {
                Ok(o) => Arc::new(o),
                Err(_) => {
                    return Err(ProjectionError::MissingOptionData);
                }
            };

            for days in &days_to_expiry {
                match self.get_surface_time_versus(axis, &option, *days) {
                    Ok((x, y, z)) => {
                        points.insert(Point3D::new(x, y, z));
                    }
                    Err(_) => continue,
                }
            }
        }

        if points.is_empty() {
            return Err(ProjectionError::NoPoints {
                kind: "time surface",
            });
        }

        Ok(Surface::new(points))
    }
}

impl OptionChain {
    /// Generates a gamma curve for visualization and analysis.
    ///
    /// Creates a curve representing gamma values across different strike prices
    /// or other relevant parameters for long call options in the chain.
    ///
    /// # Returns
    ///
    /// * `Result<Curve, ProjectionError>` - A curve object containing gamma data points,
    ///   or an error if curve generation fails
    ///
    /// # Errors
    ///
    /// Returns a [`ProjectionError`] if the curve cannot be generated due to missing data
    /// or calculation errors
    pub fn gamma_curve(&self) -> Result<Curve, ProjectionError> {
        self.curve(&BasicAxisTypes::Gamma, &OptionStyle::Call, &Side::Long)
    }

    /// Generates a delta curve for visualization and analysis.
    ///
    /// Creates a curve representing delta values across different strike prices
    /// or other relevant parameters for long call options in the chain.
    ///
    /// # Returns
    ///
    /// * `Result<Curve, ProjectionError>` - A curve object containing delta data points,
    ///   or an error if curve generation fails
    ///
    /// # Errors
    ///
    /// Returns a [`ProjectionError`] if the curve cannot be generated due to missing data
    /// or calculation errors
    pub fn delta_curve(&self) -> Result<Curve, ProjectionError> {
        self.curve(&BasicAxisTypes::Delta, &OptionStyle::Call, &Side::Long)
    }

    /// Generates a vega curve for visualization and analysis.
    ///
    /// Creates a curve representing vega values across different strike prices
    /// or other relevant parameters for long call options in the chain.
    ///
    /// # Returns
    ///
    /// * `Result<Curve, ProjectionError>` - A curve object containing vega data points,
    ///   or an error if curve generation fails
    ///
    /// # Errors
    ///
    /// Returns a [`ProjectionError`] if the curve cannot be generated due to missing data
    /// or calculation errors
    pub fn vega_curve(&self) -> Result<Curve, ProjectionError> {
        self.curve(&BasicAxisTypes::Vega, &OptionStyle::Call, &Side::Long)
    }

    /// Generates a theta curve for visualization and analysis.
    ///
    /// Creates a curve representing theta values across different strike prices
    /// or other relevant parameters for long call options in the chain.
    ///
    /// # Returns
    ///
    /// * `Result<Curve, ProjectionError>` - A curve object containing theta data points,
    ///   or an error if curve generation fails
    ///
    /// # Errors
    ///
    /// Returns a [`ProjectionError`] if the curve cannot be generated due to missing data
    /// or calculation errors
    pub fn theta_curve(&self) -> Result<Curve, ProjectionError> {
        self.curve(&BasicAxisTypes::Theta, &OptionStyle::Call, &Side::Long)
    }

    /// Generates a vanna curve for visualization and analysis.
    ///
    /// Creates a curve representing vanna values across different strike prices
    /// or other relevant parameters for long call options in the chain.
    ///
    /// # Returns
    ///
    /// * `Result<Curve, ProjectionError>` - A curve object containing vanna data points,
    ///   or an error if curve generation fails
    ///
    /// # Errors
    ///
    /// Returns a [`ProjectionError`] if the curve cannot be generated due to missing data
    /// or calculation errors
    pub fn vanna_curve(&self) -> Result<Curve, ProjectionError> {
        self.curve(&BasicAxisTypes::Vanna, &OptionStyle::Call, &Side::Long)
    }

    /// Generates a veta curve for visualization and analysis.
    ///
    /// Creates a curve representing veta values across different strike prices
    /// or other relevant parameters for long call options in the chain.
    ///
    /// # Returns
    ///
    /// * `Result<Curve, ProjectionError>` - A curve object containing veta data points,
    ///   or an error if curve generation fails
    ///
    /// # Errors
    ///
    /// Returns a [`ProjectionError`] if the curve cannot be generated due to missing data
    /// or calculation errors
    pub fn veta_curve(&self) -> Result<Curve, ProjectionError> {
        self.curve(&BasicAxisTypes::Veta, &OptionStyle::Call, &Side::Long)
    }

    /// Generates a charm curve for visualization and analysis.
    ///
    /// Creates a curve representing charm values across different strike prices
    /// or other relevant parameters for long call options in the chain.
    ///
    /// # Returns
    ///
    /// * `Result<Curve, ProjectionError>` - A curve object containing charm data points, or an error if curve generation fails
    ///
    /// # Errors
    ///
    /// Returns a [`ProjectionError`] if the curve cannot be generated due to missing
    /// data or calculation errors
    pub fn charm_curve(&self) -> Result<Curve, ProjectionError> {
        self.curve(&BasicAxisTypes::Charm, &OptionStyle::Call, &Side::Long)
    }

    /// Generates a color curve for visualization and analysis.
    ///
    /// Creates a curve representing color values across different strike prices
    /// or other relevant parameters for long call options in the chain.
    ///
    /// # Returns
    ///
    /// * `Result<Curve, ProjectionError>` - A curve object containing color data points, or an error if curve generation fails
    ///
    /// # Errors
    ///
    /// Returns a [`ProjectionError`] if the curve cannot be generated due to missing
    /// data or calculation errors
    pub fn color_curve(&self) -> Result<Curve, ProjectionError> {
        self.curve(&BasicAxisTypes::Color, &OptionStyle::Call, &Side::Long)
    }

    /// Generates a Veta time surface for visualization and analysis.
    ///
    /// Creates a 3D surface representing Veta values across different strike prices
    /// and time horizons. Veta measures the rate of change of Vega with respect to time,
    /// making this surface particularly useful for understanding how volatility sensitivity
    /// evolves as expiration approaches.
    ///
    /// # Parameters
    ///
    /// * `days_to_expiry` - Vector of days to expiration values to use for surface calculations.
    ///   Common values might be `vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0), pos_or_panic!(60.0), pos_or_panic!(90.0)]`
    ///
    /// # Returns
    ///
    /// * `Result<Surface, ProjectionError>` - A surface object containing Veta data points,
    ///   or an error if surface generation fails
    ///
    /// # Errors
    ///
    /// Returns a [`ProjectionError`] if the surface cannot be generated due to missing data
    /// or calculation errors
    ///
    /// # Example
    ///
    /// ```ignore
    /// use positive::pos_or_panic;
    ///
    /// let days = vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0), pos_or_panic!(60.0), pos_or_panic!(90.0)];
    /// let veta_surface = chain.veta_time_surface(days)?;
    /// ```
    pub fn veta_time_surface(
        &self,
        days_to_expiry: Vec<Positive>,
    ) -> Result<Surface, ProjectionError> {
        self.time_surface(
            &BasicAxisTypes::Veta,
            &OptionStyle::Call,
            days_to_expiry,
            &Side::Long,
        )
    }

    /// Generates a Theta time surface for visualization and analysis.
    ///
    /// Creates a 3D surface representing Theta values across different strike prices
    /// and time horizons. Theta measures the sensitivity of the options's value to the passage of
    /// time (time decay). As time passes with decreasing time to expiry an option's value
    /// decreases.
    ///
    /// # Parameters
    ///
    /// * `days_to_expiry` - Vector of days to expiration values to use for surface calculations.
    ///   Common values might be `vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0), pos_or_panic!(60.0), pos_or_panic!(90.0)]`
    ///
    /// # Returns
    ///
    /// * `Result<Surface, ProjectionError>` - A surface object containing Theta data points,
    ///   or an error if surface generation fails
    ///
    /// # Errors
    ///
    /// Returns a [`ProjectionError`] if the surface cannot be generated due to missing data
    /// or calculation errors
    ///
    /// # Example
    ///
    /// ```ignore
    /// use positive::pos_or_panic;
    ///
    /// let days = vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0), pos_or_panic!(60.0), pos_or_panic!(90.0)];
    /// let theta_surface = chain.theta_time_surface(days)?;
    /// ```
    pub fn theta_time_surface(
        &self,
        days_to_expiry: Vec<Positive>,
    ) -> Result<Surface, ProjectionError> {
        self.time_surface(
            &BasicAxisTypes::Theta,
            &OptionStyle::Call,
            days_to_expiry,
            &Side::Long,
        )
    }

    /// Generates a Charm time surface for visualization and analysis.
    ///
    /// Creates a 3D surface representing Charm values across different strike prices
    /// and time horizons. Charm, also called DdeltaDtime or Delta decay,  measures the rate of
    /// change of Delta over the passage of time.
    ///
    /// # Parameters
    ///
    /// * `days_to_expiry` - Vector of days to expiration values to use for surface calculations.
    ///   Common values might be `vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0), pos_or_panic!(60.0), pos_or_panic!(90.0)]`
    ///
    /// # Returns
    ///
    /// * `Result<Surface, ProjectionError>` - A surface object containing Charm data points,
    ///   or an error if surface generation fails
    ///
    /// # Errors
    ///
    /// Returns a [`ProjectionError`] if the surface cannot be generated due to missing data
    /// or calculation errors
    ///
    /// # Example
    ///
    /// ```ignore
    /// use positive::pos_or_panic;
    ///
    /// let days = vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0), pos_or_panic!(60.0), pos_or_panic!(90.0)];
    /// let charm_surface = chain.charm_time_surface(days)?;
    /// ```
    pub fn charm_time_surface(
        &self,
        days_to_expiry: Vec<Positive>,
    ) -> Result<Surface, ProjectionError> {
        self.time_surface(
            &BasicAxisTypes::Charm,
            &OptionStyle::Call,
            days_to_expiry,
            &Side::Long,
        )
    }

    /// Generates a Color time surface for visualization and analysis.
    ///
    /// Creates a 3D surface representing Color values across different strike prices
    /// and time horizons. Color, also called DgammaDtime or Gamma decay,  measures the rate of
    /// change of Gamma over the passage of time.
    ///
    /// # Parameters
    ///
    /// * `days_to_expiry` - Vector of days to expiration values to use for surface calculations.
    ///   Common values might be `vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0), pos_or_panic!(60.0), pos_or_panic!(90.0)]`
    ///
    /// # Returns
    ///
    /// * `Result<Surface, ProjectionError>` - A surface object containing Color data points,
    ///   or an error if surface generation fails
    ///
    /// # Errors
    ///
    /// Returns a [`ProjectionError`] if the surface cannot be generated due to missing data
    /// or calculation errors
    ///
    /// # Example
    ///
    /// ```ignore
    /// use positive::pos_or_panic;
    ///
    /// let days = vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0), pos_or_panic!(60.0), pos_or_panic!(90.0)];
    /// let color_surface = chain.color_time_surface(days)?;
    /// ```
    pub fn color_time_surface(
        &self,
        days_to_expiry: Vec<Positive>,
    ) -> Result<Surface, ProjectionError> {
        self.time_surface(
            &BasicAxisTypes::Color,
            &OptionStyle::Call,
            days_to_expiry,
            &Side::Long,
        )
    }

    /// Generates a Vanna volatility surface for visualization and analysis.
    ///
    /// Creates a 3D surface representing Vanna values across different strike prices
    /// and volatility levels. Vanna measures the sensitivity of Delta to changes in
    /// implied volatility, making this surface useful for understanding how delta
    /// hedging effectiveness changes with volatility.
    ///
    /// # Parameters
    ///
    /// * `volatilities` - Vector of volatility values to use for surface calculations.
    ///   Common values might be `vec![pos_or_panic!(0.1), pos_or_panic!(0.2), pos_or_panic!(0.3), pos_or_panic!(0.4), pos_or_panic!(0.5)]`
    ///
    /// # Returns
    ///
    /// * `Result<Surface, ProjectionError>` - A surface object containing Vanna data points,
    ///   or an error if surface generation fails
    ///
    /// # Errors
    ///
    /// Returns a [`ProjectionError`] if the surface cannot be generated due to missing data
    /// or calculation errors
    pub fn vanna_surface(&self, volatilities: Vec<Positive>) -> Result<Surface, ProjectionError> {
        self.surface(
            &BasicAxisTypes::Vanna,
            &OptionStyle::Call,
            Some(volatilities),
            &Side::Long,
        )
    }

    /// Generates a Vomma volatility surface for visualization and analysis.
    ///
    /// Creates a 3D surface representing Vomma (Volga) values across different strike prices
    /// and volatility levels. Vomma measures the second-order sensitivity of option price
    /// to volatility (rate of change of Vega with respect to volatility).
    ///
    /// # Parameters
    ///
    /// * `volatilities` - Vector of volatility values to use for surface calculations.
    ///   Common values might be `vec![pos_or_panic!(0.1), pos_or_panic!(0.2), pos_or_panic!(0.3), pos_or_panic!(0.4), pos_or_panic!(0.5)]`
    ///
    /// # Returns
    ///
    /// * `Result<Surface, ProjectionError>` - A surface object containing Vomma data points,
    ///   or an error if surface generation fails
    ///
    /// # Errors
    ///
    /// Returns a [`ProjectionError`] if the surface cannot be generated due to missing data
    /// or calculation errors
    pub fn vomma_surface(&self, volatilities: Vec<Positive>) -> Result<Surface, ProjectionError> {
        self.surface(
            &BasicAxisTypes::Vomma,
            &OptionStyle::Call,
            Some(volatilities),
            &Side::Long,
        )
    }
}

#[cfg(test)]
mod tests_basic_curves {
    #![allow(clippy::indexing_slicing)]
    use super::*;
    use crate::utils::Len;
    use positive::{pos_or_panic, spos};
    use rust_decimal::Decimal;

    use crate::model::types::{OptionStyle, Side};
    use crate::utils::time::get_x_days_formatted;

    use rust_decimal_macros::dec;

    // Helper function to create a sample OptionChain for testing
    fn create_test_option_chain() -> OptionChain {
        let tomorrow_date = get_x_days_formatted(30);
        let mut chain = OptionChain::new("TEST", Positive::HUNDRED, tomorrow_date, None, None);

        // Add some test options
        chain.add_option(
            pos_or_panic!(90.0), // strike_price
            spos!(5.0),          // call_bid
            spos!(5.5),          // call_ask
            spos!(1.0),          // put_bid
            spos!(1.5),          // put_ask
            pos_or_panic!(0.2),  // implied_volatility
            Some(dec!(0.6)),     // delta
            Some(dec!(100.0)),   // volume
            Some(dec!(50.0)),    // open_interest
            None,
            None,
            None,
        );

        chain.add_option(
            Positive::HUNDRED,
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            pos_or_panic!(0.25),
            Some(dec!(0.5)),
            Some(dec!(150.0)),
            Some(dec!(75)),
            None,
            None,
            None,
        );

        chain.add_option(
            pos_or_panic!(110.0),
            spos!(1.0),
            spos!(1.5),
            spos!(5.0),
            spos!(5.5),
            pos_or_panic!(0.3),
            Some(dec!(0.4)),
            Some(dec!(80.0)),
            Some(dec!(40)),
            None,
            None,
            None,
        );
        chain.update_greeks();
        chain
    }

    #[test]
    fn test_curve_delta_long_call() {
        let chain = create_test_option_chain();

        let curve = chain.curve(&BasicAxisTypes::Delta, &OptionStyle::Call, &Side::Long);

        assert!(curve.is_ok());
        let curve = curve.unwrap();

        // Check that we have the expected number of points
        assert_eq!(curve.points.len(), 3);

        // Verify points are in reasonable ranges
        for point in &curve.points {
            assert!(
                point.x >= dec!(90.0) && point.x <= dec!(110.0),
                "Delta out of expected range"
            );
        }
    }

    #[test]
    fn test_curve_delta_short_put() {
        let chain = create_test_option_chain();

        let curve = chain.curve(&BasicAxisTypes::Delta, &OptionStyle::Put, &Side::Short);

        assert!(curve.is_ok());
        let curve = curve.unwrap();

        // Check that we have the expected number of points
        assert!(!curve.points.is_empty());

        // Verify points are in reasonable ranges
        for point in &curve.points {
            assert!(point.x > dec!(0.0), "Price should be positive");
        }
    }

    #[test]
    fn test_curve_price_short_put() {
        let chain = create_test_option_chain();

        let curve = chain.curve(&BasicAxisTypes::Price, &OptionStyle::Put, &Side::Short);

        assert!(curve.is_ok());
        let curve = curve.unwrap();

        // Check that we have the expected number of points
        assert!(!curve.points.is_empty());

        // Verify points are in reasonable ranges
        for point in &curve.points {
            assert!(point.x > dec!(0.0), "Price should be positive");
        }
    }

    #[test]
    fn test_curve_length() {
        let chain = create_test_option_chain();

        assert_eq!(chain.len(), 3);
    }

    #[test]
    fn test_curve_with_empty_chain() {
        let chain = OptionChain::new(
            "EMPTY",
            Positive::HUNDRED,
            "2024-12-31".to_string(),
            None,
            None,
        );

        let curve = chain.curve(&BasicAxisTypes::Delta, &OptionStyle::Call, &Side::Long);

        assert!(curve.is_ok());
        let curve = curve.unwrap();

        // Curve should be empty
        assert_eq!(curve.points.len(), 0);
    }

    #[test]
    fn test_curve_multiple_axes() {
        let chain = create_test_option_chain();

        // Test various axis combinations
        let axes = vec![
            BasicAxisTypes::Delta,
            BasicAxisTypes::Price,
            BasicAxisTypes::Volatility,
            BasicAxisTypes::Gamma,
            BasicAxisTypes::Theta,
            BasicAxisTypes::Vega,
            BasicAxisTypes::Vanna,
            BasicAxisTypes::Veta,
            BasicAxisTypes::Charm,
            BasicAxisTypes::Color,
        ];

        for axis in axes {
            let curve = chain.curve(&axis, &OptionStyle::Call, &Side::Long);

            assert!(curve.is_ok(), "Failed to create curve for axis: {axis:?}");
            let curve = curve.unwrap();

            // Each curve should have at least one point
            assert!(!curve.points.is_empty(), "Curve for axis {axis:?} is empty");
        }
    }

    #[test]
    fn test_curve_point_order() {
        let chain = create_test_option_chain();

        let curve = chain.curve(&BasicAxisTypes::Delta, &OptionStyle::Call, &Side::Long);

        assert!(curve.is_ok());
        let curve = curve.unwrap();

        // Verify points are in order (sorted by x-coordinate)
        let mut prev_x = Decimal::MIN;
        for point in &curve.points {
            assert!(point.x >= prev_x, "Points are not in ascending order");
            prev_x = point.x;
        }
    }
}

#[cfg(test)]
mod tests_option_chain_surfaces {
    #![allow(clippy::indexing_slicing)]
    use super::*;
    use positive::{pos_or_panic, spos};

    use crate::utils::time::get_x_days_formatted;

    use rust_decimal_macros::dec;

    fn create_test_option_chain() -> OptionChain {
        let tomorrow_date = get_x_days_formatted(30);
        let mut chain = OptionChain::new("TEST", Positive::HUNDRED, tomorrow_date, None, None);

        // Add some test options
        chain.add_option(
            pos_or_panic!(90.0), // strike_price
            spos!(5.0),          // call_bid
            spos!(5.5),          // call_ask
            spos!(1.0),          // put_bid
            spos!(1.5),          // put_ask
            pos_or_panic!(0.2),  // implied_volatility
            Some(dec!(0.6)),     // delta
            Some(dec!(100.0)),   // volume
            Some(dec!(50.0)),    // open_interest
            None,
            None,
            None,
        );

        chain.add_option(
            Positive::HUNDRED,
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            pos_or_panic!(0.25),
            Some(dec!(0.5)),
            Some(dec!(150.0)),
            Some(dec!(75)),
            None,
            None,
            None,
        );

        chain.add_option(
            pos_or_panic!(110.0),
            spos!(1.0),
            spos!(1.5),
            spos!(5.0),
            spos!(5.5),
            pos_or_panic!(0.3),
            Some(dec!(0.4)),
            Some(dec!(80.0)),
            Some(dec!(40)),
            None,
            None,
            None,
        );
        chain.update_greeks();
        chain
    }

    #[test]
    fn test_surface_invalid_axis() {
        let chain = create_test_option_chain();
        let result = chain.surface(
            &BasicAxisTypes::Strike,
            &OptionStyle::Call,
            None,
            &Side::Long,
        );

        assert!(result.is_err());
        match result {
            Err(ProjectionError::UnsupportedAxis { axis }) => {
                // The rejected axis is a field, not text to parse.
                assert!(!axis.is_empty(), "the unsupported axis is reported");
            }
            other => panic!("unexpected projection error: {other:?}"),
        }
    }

    #[test]
    fn test_surface_with_no_volatility() {
        let chain = create_test_option_chain();
        let result = chain.surface(
            &BasicAxisTypes::Delta,
            &OptionStyle::Call,
            None,
            &Side::Long,
        );

        assert!(result.is_ok());
        let surface = result.unwrap();
        assert!(!surface.points.is_empty());

        // Should have one point per strike
        assert_eq!(surface.points.len(), 3);
    }

    #[test]
    fn test_surface_with_custom_volatilities() {
        let chain = create_test_option_chain();
        let volatilities = vec![
            pos_or_panic!(0.15),
            pos_or_panic!(0.20),
            pos_or_panic!(0.25),
        ];

        let result = chain.surface(
            &BasicAxisTypes::Delta,
            &OptionStyle::Call,
            Some(volatilities),
            &Side::Long,
        );

        assert!(result.is_ok());
        let surface = result.unwrap();
        assert!(!surface.points.is_empty());
        assert_eq!(surface.points.len(), 9);
    }

    #[test]
    fn test_surface_empty_volatility_vector() {
        let chain = create_test_option_chain();
        let empty_vols: Vec<Positive> = vec![];

        let result = chain.surface(
            &BasicAxisTypes::Delta,
            &OptionStyle::Call,
            Some(empty_vols),
            &Side::Long,
        );

        assert!(result.is_err());
        match result {
            Err(ProjectionError::NoPoints { kind }) => {
                assert_eq!(kind, "surface");
            }
            other => panic!("unexpected projection error: {other:?}"),
        }
    }

    #[test]
    fn test_surface_different_option_styles() {
        let chain = create_test_option_chain();

        // Test for calls
        let call_result = chain.surface(
            &BasicAxisTypes::Delta,
            &OptionStyle::Call,
            None,
            &Side::Long,
        );
        assert!(call_result.is_ok());

        // Test for puts
        let put_result =
            chain.surface(&BasicAxisTypes::Delta, &OptionStyle::Put, None, &Side::Long);
        assert!(put_result.is_ok());
    }

    #[test]
    fn test_surface_different_sides() {
        let chain = create_test_option_chain();

        // Test for long position
        let long_result = chain.surface(
            &BasicAxisTypes::Delta,
            &OptionStyle::Call,
            None,
            &Side::Long,
        );
        assert!(long_result.is_ok());

        // Test for short position
        let short_result = chain.surface(
            &BasicAxisTypes::Delta,
            &OptionStyle::Call,
            None,
            &Side::Short,
        );
        assert!(short_result.is_ok());
    }

    #[test]
    fn test_surface_different_greeks() {
        let chain = create_test_option_chain();
        let axes = vec![
            BasicAxisTypes::Delta,
            BasicAxisTypes::Gamma,
            BasicAxisTypes::Theta,
            BasicAxisTypes::Vega,
            BasicAxisTypes::Price,
            BasicAxisTypes::Vanna,
            BasicAxisTypes::Vomma,
            BasicAxisTypes::Veta,
            BasicAxisTypes::Charm,
            BasicAxisTypes::Color,
        ];

        for axis in axes {
            let result = chain.surface(&axis, &OptionStyle::Call, None, &Side::Long);
            assert!(result.is_ok(), "Failed for axis: {axis:?}");
        }
    }

    #[test]
    fn test_surface_with_empty_chain() {
        let empty_chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            "2024-12-31".to_string(),
            Some(dec!(0.05)),
            spos!(0.01),
        );

        let result = empty_chain.surface(
            &BasicAxisTypes::Delta,
            &OptionStyle::Call,
            None,
            &Side::Long,
        );

        assert!(result.is_err());
        match result {
            Err(ProjectionError::NoPoints { kind }) => {
                assert_eq!(kind, "surface");
            }
            other => panic!("unexpected projection error: {other:?}"),
        }
    }

    #[test]
    fn test_vanna_surface() {
        let chain = create_test_option_chain();
        let volatilities = vec![
            pos_or_panic!(0.15),
            pos_or_panic!(0.20),
            pos_or_panic!(0.25),
        ];
        let chain_result = chain.vanna_surface(volatilities);
        assert!(chain_result.is_ok());
    }

    #[test]
    fn test_vomma_surface() {
        let chain = create_test_option_chain();
        let volatilities = vec![
            pos_or_panic!(0.15),
            pos_or_panic!(0.20),
            pos_or_panic!(0.25),
        ];
        let chain_result = chain.vomma_surface(volatilities);
        assert!(chain_result.is_ok());
    }
}

#[cfg(test)]
mod tests_option_chain_time_surfaces {
    #![allow(clippy::indexing_slicing)]
    use super::*;
    use positive::{pos_or_panic, spos};

    use crate::utils::time::get_x_days_formatted;

    use rust_decimal_macros::dec;

    fn create_test_option_chain() -> OptionChain {
        let tomorrow_date = get_x_days_formatted(30);
        let mut chain = OptionChain::new("TEST", Positive::HUNDRED, tomorrow_date, None, None);

        // Add some test options
        chain.add_option(
            pos_or_panic!(90.0), // strike_price
            spos!(5.0),          // call_bid
            spos!(5.5),          // call_ask
            spos!(1.0),          // put_bid
            spos!(1.5),          // put_ask
            pos_or_panic!(0.2),  // implied_volatility
            Some(dec!(0.6)),     // delta
            Some(dec!(100.0)),   // volume
            Some(dec!(50.0)),    // open_interest
            None,
            None,
            None,
        );

        chain.add_option(
            Positive::HUNDRED,
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            pos_or_panic!(0.25),
            Some(dec!(0.5)),
            Some(dec!(150.0)),
            Some(dec!(75)),
            None,
            None,
            None,
        );

        chain.add_option(
            pos_or_panic!(110.0),
            spos!(1.0),
            spos!(1.5),
            spos!(5.0),
            spos!(5.5),
            pos_or_panic!(0.3),
            Some(dec!(0.4)),
            Some(dec!(80.0)),
            Some(dec!(40)),
            None,
            None,
            None,
        );
        chain.update_greeks();
        chain
    }

    #[test]
    fn test_time_surface_invalid_axis() {
        let chain = create_test_option_chain();
        let days_to_expiry = vec![
            pos_or_panic!(30.0),
            pos_or_panic!(60.0),
            pos_or_panic!(90.0),
        ];
        let result = chain.time_surface(
            &BasicAxisTypes::Strike,
            &OptionStyle::Call,
            days_to_expiry,
            &Side::Long,
        );

        assert!(result.is_err());
        match result {
            Err(ProjectionError::UnsupportedAxis { axis }) => {
                // The rejected axis is a field, not text to parse.
                assert!(!axis.is_empty(), "the unsupported axis is reported");
            }
            other => panic!("unexpected projection error: {other:?}"),
        }
    }

    #[test]
    fn test_time_surface_empty_dte_vector() {
        let chain = create_test_option_chain();
        let empty_dte: Vec<Positive> = vec![];

        let result = chain.time_surface(
            &BasicAxisTypes::Charm,
            &OptionStyle::Call,
            empty_dte,
            &Side::Long,
        );

        assert!(result.is_err());
        match result {
            Err(ProjectionError::NoPoints { kind }) => {
                assert_eq!(kind, "time surface");
            }
            other => panic!("unexpected projection error: {other:?}"),
        }
    }

    #[test]
    fn test_time_surface_different_option_styles() {
        let chain = create_test_option_chain();

        // Test for calls
        let days_to_expiry = vec![
            pos_or_panic!(30.0),
            pos_or_panic!(60.0),
            pos_or_panic!(90.0),
        ];
        let call_result = chain.time_surface(
            &BasicAxisTypes::Charm,
            &OptionStyle::Call,
            days_to_expiry,
            &Side::Long,
        );
        assert!(call_result.is_ok());

        // Test for puts
        let days_to_expiry = vec![
            pos_or_panic!(30.0),
            pos_or_panic!(60.0),
            pos_or_panic!(90.0),
        ];
        let put_result = chain.time_surface(
            &BasicAxisTypes::Charm,
            &OptionStyle::Put,
            days_to_expiry,
            &Side::Long,
        );
        assert!(put_result.is_ok());
    }

    #[test]
    fn test_time_surface_different_sides() {
        let chain = create_test_option_chain();

        // Test for long position
        let days_to_expiry = vec![
            pos_or_panic!(30.0),
            pos_or_panic!(60.0),
            pos_or_panic!(90.0),
        ];
        let long_result = chain.time_surface(
            &BasicAxisTypes::Color,
            &OptionStyle::Call,
            days_to_expiry,
            &Side::Long,
        );
        assert!(long_result.is_ok());

        // Test for short position
        let days_to_expiry = vec![
            pos_or_panic!(30.0),
            pos_or_panic!(60.0),
            pos_or_panic!(90.0),
        ];
        let short_result = chain.time_surface(
            &BasicAxisTypes::Color,
            &OptionStyle::Call,
            days_to_expiry,
            &Side::Short,
        );
        assert!(short_result.is_ok());
    }

    #[test]
    fn test_time_surface_different_greeks() {
        let chain = create_test_option_chain();
        let axes = vec![
            BasicAxisTypes::Delta,
            BasicAxisTypes::Gamma,
            BasicAxisTypes::Theta,
            BasicAxisTypes::Vega,
            BasicAxisTypes::Price,
            BasicAxisTypes::Vanna,
            BasicAxisTypes::Vomma,
            BasicAxisTypes::Veta,
            BasicAxisTypes::Charm,
            BasicAxisTypes::Color,
        ];

        for axis in axes {
            let days_to_expiry = vec![
                pos_or_panic!(30.0),
                pos_or_panic!(60.0),
                pos_or_panic!(90.0),
            ];
            let result = chain.time_surface(&axis, &OptionStyle::Call, days_to_expiry, &Side::Long);
            assert!(result.is_ok(), "Failed for axis: {axis:?}");
        }
    }

    #[test]
    fn test_time_surface_with_empty_chain() {
        let empty_chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            "2024-12-31".to_string(),
            Some(dec!(0.05)),
            spos!(0.01),
        );

        let empty_dte: Vec<Positive> = vec![];
        let result = empty_chain.time_surface(
            &BasicAxisTypes::Delta,
            &OptionStyle::Call,
            empty_dte,
            &Side::Long,
        );

        assert!(result.is_err());
        match result {
            Err(ProjectionError::NoPoints { kind }) => {
                assert_eq!(kind, "time surface");
            }
            other => panic!("unexpected projection error: {other:?}"),
        }
    }

    #[test]
    fn test_theta_time_surface() {
        let chain = create_test_option_chain();
        let days_to_expiry = vec![
            pos_or_panic!(30.0),
            pos_or_panic!(60.0),
            pos_or_panic!(90.0),
        ];
        let chain_result = chain.theta_time_surface(days_to_expiry);
        assert!(chain_result.is_ok());
    }

    #[test]
    fn test_veta_time_surface() {
        let chain = create_test_option_chain();
        let days_to_expiry = vec![
            pos_or_panic!(30.0),
            pos_or_panic!(60.0),
            pos_or_panic!(90.0),
        ];
        let chain_result = chain.veta_time_surface(days_to_expiry);
        assert!(chain_result.is_ok());
    }

    #[test]
    fn test_charm_time_surface() {
        let chain = create_test_option_chain();
        let days_to_expiry = vec![
            pos_or_panic!(30.0),
            pos_or_panic!(60.0),
            pos_or_panic!(90.0),
        ];
        let chain_result = chain.charm_time_surface(days_to_expiry);
        assert!(chain_result.is_ok());
    }

    #[test]
    fn test_color_time_surface() {
        let chain = create_test_option_chain();
        let days_to_expiry = vec![
            pos_or_panic!(30.0),
            pos_or_panic!(60.0),
            pos_or_panic!(90.0),
        ];
        let chain_result = chain.color_time_surface(days_to_expiry);
        assert!(chain_result.is_ok());
    }
}
