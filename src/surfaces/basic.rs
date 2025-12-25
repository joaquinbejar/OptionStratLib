/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 9/2/25
******************************************************************************/
use crate::error::SurfaceError;
use crate::greeks::Greeks;
use crate::model::BasicAxisTypes;
use crate::surfaces::Surface;
use crate::{OptionStyle, Options, Side};
use positive::Positive;
use rust_decimal::Decimal;
use std::sync::Arc;

/// # BasicSurfaces Trait
///
/// This trait defines operations for creating and analyzing option pricing surfaces,
/// which are three-dimensional representations of option metrics across different
/// parameters.
///
/// A surface typically maps option strike prices and volatilities to various
/// option metrics like delta, gamma, theta, vega, or price.
pub trait BasicSurfaces {
    /// Creates a surface visualization based on the specified axis type and option parameters.
    ///
    /// # Parameters
    ///
    /// * `axis` - The option metric to calculate and display on the surface (e.g., Delta, Gamma)
    /// * `option_style` - Whether the options are Calls or Puts
    /// * `volatility` - Optional vector of volatility values to use for surface calculations
    /// * `side` - Whether the options are Long or Short positions
    ///
    /// # Returns
    ///
    /// * `Result<Surface, SurfaceError>` - A constructed surface or an error if creation fails
    fn surface(
        &self,
        axis: &BasicAxisTypes,
        option_style: &OptionStyle,
        volatility: Option<Vec<Positive>>,
        side: &Side,
    ) -> Result<Surface, SurfaceError>;

    /// Calculates the relationship between strike price, implied volatility, and a selected
    /// option metric for a given option.
    ///
    /// This method uses the option's existing implied volatility value to calculate the
    /// desired metric (delta, gamma, theta, vega, vanna, vomma, veta or price).
    ///
    /// # Parameters
    ///
    /// * `axis` - The option metric to calculate (e.g., Delta, Gamma)
    /// * `option` - Reference to the option contract to analyze
    ///
    /// # Returns
    ///
    /// * `Result<(Decimal, Decimal, Decimal), SurfaceError>` - A tuple containing:
    ///   - Strike price
    ///   - Implied volatility
    ///   - Calculated metric value
    ///
    /// # Errors
    ///
    /// Returns a `SurfaceError` if the selected axis is not supported or if any calculation fails.
    fn get_surface_strike_versus(
        &self,
        axis: &BasicAxisTypes,
        option: &Arc<Options>,
    ) -> Result<(Decimal, Decimal, Decimal), SurfaceError> {
        // Create a modified copy of the option with the specified volatility
        let option_with_vol = (**option).clone();

        match axis {
            BasicAxisTypes::Delta => Ok((
                option_with_vol.strike_price.to_dec(),
                option_with_vol.implied_volatility.to_dec(),
                option_with_vol.delta()?,
            )),
            BasicAxisTypes::Gamma => Ok((
                option_with_vol.strike_price.to_dec(),
                option_with_vol.implied_volatility.to_dec(),
                option_with_vol.gamma()?,
            )),
            BasicAxisTypes::Theta => Ok((
                option_with_vol.strike_price.to_dec(),
                option_with_vol.implied_volatility.to_dec(),
                option_with_vol.theta()?,
            )),
            BasicAxisTypes::Vega => Ok((
                option_with_vol.strike_price.to_dec(),
                option_with_vol.implied_volatility.to_dec(),
                option_with_vol.vega()?,
            )),
            BasicAxisTypes::Vanna => Ok((
                option_with_vol.strike_price.to_dec(),
                option_with_vol.implied_volatility.to_dec(),
                option_with_vol.vanna()?,
            )),
            BasicAxisTypes::Vomma => Ok((
                option_with_vol.strike_price.to_dec(),
                option_with_vol.implied_volatility.to_dec(),
                option_with_vol.vomma()?,
            )),
            BasicAxisTypes::Veta => Ok((
                option_with_vol.strike_price.to_dec(),
                option_with_vol.implied_volatility.to_dec(),
                option_with_vol.veta()?,
            )),
            BasicAxisTypes::Charm => Ok((
                option_with_vol.strike_price.to_dec(),
                option_with_vol.implied_volatility.to_dec(),
                option_with_vol.charm()?,
            )),
            BasicAxisTypes::Color => Ok((
                option_with_vol.strike_price.to_dec(),
                option_with_vol.implied_volatility.to_dec(),
                option_with_vol.color()?,
            )),
            BasicAxisTypes::Price => Ok((
                option_with_vol.strike_price.to_dec(),
                option_with_vol.implied_volatility.to_dec(),
                option_with_vol.calculate_price_black_scholes()?,
            )),

            // Catch-all for unsupported combinations
            _ => Err(SurfaceError::OperationError(
                crate::error::OperationErrorKind::InvalidParameters {
                    operation: "get_strike_volatility_versus".to_string(),
                    reason: format!("Axis: {axis:?} not supported"),
                },
            )),
        }
    }

    /// Calculates the relationship between strike price, a specified volatility value, and a selected
    /// option metric for a given option.
    ///
    /// This method uses a custom volatility value (different from the option's current implied volatility)
    /// to calculate the desired metric (delta, gamma, theta, vega, vanna, vomma, veta or price).
    ///
    /// # Parameters
    ///
    /// * `axis` - The option metric to calculate (e.g., Delta, Gamma)
    /// * `option` - Reference to the option contract to analyze
    /// * `volatility` - The specific volatility value to use for the calculation
    ///
    /// # Returns
    ///
    /// * `Result<(Decimal, Decimal, Decimal), SurfaceError>` - A tuple containing:
    ///   - Strike price
    ///   - The provided volatility value
    ///   - Calculated metric value
    ///
    /// # Errors
    ///
    /// Returns a `SurfaceError` if the selected axis is not supported or if any calculation fails.
    fn get_surface_volatility_versus(
        &self,
        axis: &BasicAxisTypes,
        option: &Arc<Options>,
        volatility: Positive,
    ) -> Result<(Decimal, Decimal, Decimal), SurfaceError> {
        // Create a modified copy of the option with the specified volatility
        let mut option_with_vol = (**option).clone();
        option_with_vol.implied_volatility = volatility;
        match axis {
            BasicAxisTypes::Delta => Ok((
                option_with_vol.strike_price.to_dec(),
                volatility.to_dec(),
                option_with_vol.delta()?,
            )),
            BasicAxisTypes::Gamma => Ok((
                option_with_vol.strike_price.to_dec(),
                volatility.to_dec(),
                option_with_vol.gamma()?,
            )),
            BasicAxisTypes::Theta => Ok((
                option_with_vol.strike_price.to_dec(),
                volatility.to_dec(),
                option_with_vol.theta()?,
            )),
            BasicAxisTypes::Vega => Ok((
                option_with_vol.strike_price.to_dec(),
                volatility.to_dec(),
                option_with_vol.vega()?,
            )),
            BasicAxisTypes::Vanna => Ok((
                option_with_vol.strike_price.to_dec(),
                volatility.to_dec(),
                option_with_vol.vanna()?,
            )),
            BasicAxisTypes::Vomma => Ok((
                option_with_vol.strike_price.to_dec(),
                volatility.to_dec(),
                option_with_vol.vomma()?,
            )),
            BasicAxisTypes::Veta => Ok((
                option_with_vol.strike_price.to_dec(),
                volatility.to_dec(),
                option_with_vol.veta()?,
            )),
            BasicAxisTypes::Charm => Ok((
                option_with_vol.strike_price.to_dec(),
                volatility.to_dec(),
                option_with_vol.charm()?,
            )),
            BasicAxisTypes::Color => Ok((
                option_with_vol.strike_price.to_dec(),
                volatility.to_dec(),
                option_with_vol.color()?,
            )),
            BasicAxisTypes::Price => Ok((
                option_with_vol.strike_price.to_dec(),
                volatility.to_dec(),
                option_with_vol.calculate_price_black_scholes()?,
            )),

            // Catch-all for unsupported combinations
            _ => Err(SurfaceError::OperationError(
                crate::error::OperationErrorKind::InvalidParameters {
                    operation: "get_strike_volatility_versus".to_string(),
                    reason: format!("Axis: {axis:?} not supported"),
                },
            )),
        }
    }

    /// Calculates the relationship between strike price, time to expiration, and a selected
    /// option metric for a given option.
    ///
    /// This method modifies the option's expiration date to calculate the desired metric
    /// (veta, charm, theta, or other time-sensitive Greeks) across different time horizons.
    /// This is particularly useful for generating surfaces that show how metrics evolve
    /// as time passes (price vs. time surfaces).
    ///
    /// # Parameters
    ///
    /// * `axis` - The option metric to calculate (e.g., Veta, Charm, Theta)
    /// * `option` - Reference to the option contract to analyze
    /// * `days_to_expiry` - The specific number of days to expiration to use for the calculation
    ///
    /// # Returns
    ///
    /// * `Result<(Decimal, Decimal, Decimal), SurfaceError>` - A tuple containing:
    ///   - Strike price
    ///   - Days to expiration
    ///   - Calculated metric value
    ///
    /// # Errors
    ///
    /// Returns a `SurfaceError` if the selected axis is not supported or if any calculation fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use optionstratlib::surfaces::BasicSurfaces;
    /// use optionstratlib::model::BasicAxisTypes;
    /// use optionstratlib::pos_or_panic;
    ///
    /// // Generate veta values for different times to expiration
    /// let days_values = vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0), pos_or_panic!(60.0), pos_or_panic!(90.0)];
    /// for days in days_values {
    ///     let (strike, time, veta) = surfaces.get_surface_time_versus(
    ///         &BasicAxisTypes::Veta,
    ///         &option,
    ///         days
    ///     )?;
    /// }
    /// ```
    fn get_surface_time_versus(
        &self,
        axis: &BasicAxisTypes,
        option: &Arc<Options>,
        days_to_expiry: Positive,
    ) -> Result<(Decimal, Decimal, Decimal), SurfaceError> {
        use crate::ExpirationDate;

        // Create a modified copy of the option with the specified time to expiration
        let mut option_with_time = (**option).clone();
        option_with_time.expiration_date = ExpirationDate::Days(days_to_expiry);

        match axis {
            BasicAxisTypes::Delta => Ok((
                option_with_time.strike_price.to_dec(),
                days_to_expiry.to_dec(),
                option_with_time.delta()?,
            )),
            BasicAxisTypes::Gamma => Ok((
                option_with_time.strike_price.to_dec(),
                days_to_expiry.to_dec(),
                option_with_time.gamma()?,
            )),
            BasicAxisTypes::Theta => Ok((
                option_with_time.strike_price.to_dec(),
                days_to_expiry.to_dec(),
                option_with_time.theta()?,
            )),
            BasicAxisTypes::Vega => Ok((
                option_with_time.strike_price.to_dec(),
                days_to_expiry.to_dec(),
                option_with_time.vega()?,
            )),
            BasicAxisTypes::Vanna => Ok((
                option_with_time.strike_price.to_dec(),
                days_to_expiry.to_dec(),
                option_with_time.vanna()?,
            )),
            BasicAxisTypes::Vomma => Ok((
                option_with_time.strike_price.to_dec(),
                days_to_expiry.to_dec(),
                option_with_time.vomma()?,
            )),
            BasicAxisTypes::Veta => Ok((
                option_with_time.strike_price.to_dec(),
                days_to_expiry.to_dec(),
                option_with_time.veta()?,
            )),
            BasicAxisTypes::Charm => Ok((
                option_with_time.strike_price.to_dec(),
                days_to_expiry.to_dec(),
                option_with_time.charm()?,
            )),
            BasicAxisTypes::Color => Ok((
                option_with_time.strike_price.to_dec(),
                days_to_expiry.to_dec(),
                option_with_time.color()?,
            )),
            BasicAxisTypes::Price => Ok((
                option_with_time.strike_price.to_dec(),
                days_to_expiry.to_dec(),
                option_with_time.calculate_price_black_scholes()?,
            )),

            // Catch-all for unsupported combinations
            _ => Err(SurfaceError::OperationError(
                crate::error::OperationErrorKind::InvalidParameters {
                    operation: "get_surface_time_versus".to_string(),
                    reason: format!("Axis: {axis:?} not supported"),
                },
            )),
        }
    }

    /// Creates a time-based surface visualization based on the specified axis type and option parameters.
    ///
    /// This method generates a 3D surface where the axes are:
    /// - X: Strike price
    /// - Y: Days to expiration
    /// - Z: The selected metric value (e.g., Veta, Charm, Theta)
    ///
    /// This is particularly useful for visualizing how time-sensitive Greeks like Veta
    /// evolve across different strike prices and time horizons.
    ///
    /// # Parameters
    ///
    /// * `axis` - The option metric to calculate and display on the surface (e.g., Veta, Charm)
    /// * `option_style` - Whether the options are Calls or Puts
    /// * `days_to_expiry` - Vector of days to expiration values to use for surface calculations
    /// * `side` - Whether the options are Long or Short positions
    ///
    /// # Returns
    ///
    /// * `Result<Surface, SurfaceError>` - A constructed surface or an error if creation fails
    fn time_surface(
        &self,
        axis: &BasicAxisTypes,
        option_style: &OptionStyle,
        days_to_expiry: Vec<Positive>,
        side: &Side,
    ) -> Result<Surface, SurfaceError>;
}

#[cfg(test)]
mod tests_basic_surfaces {
    use super::*;
    use crate::{ExpirationDate, OptionType, assert_decimal_eq};
use positive::pos_or_panic;
    use rust_decimal_macros::dec;
    use std::sync::Arc;

    // Mock implementation of BasicSurfaces for testing
    struct MockBasicSurfaces;

    impl BasicSurfaces for MockBasicSurfaces {
        fn surface(
            &self,
            _axis: &BasicAxisTypes,
            _option_style: &OptionStyle,
            _volatility: Option<Vec<Positive>>,
            _side: &Side,
        ) -> Result<Surface, SurfaceError> {
            Ok(Surface::default())
        }

        fn time_surface(
            &self,
            _axis: &BasicAxisTypes,
            _option_style: &OptionStyle,
            _days_to_expiry: Vec<Positive>,
            _side: &Side,
        ) -> Result<Surface, SurfaceError> {
            Ok(Surface::default())
        }
    }

    // Helper function to create a test option
    fn create_test_option() -> Arc<Options> {
        Arc::new(Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED, // strike_price
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),   // implied_volatility
            Positive::ONE,   // quantity
            Positive::HUNDRED, // underlying_price
            dec!(0.05),           // risk_free_rate
            OptionStyle::Call,
            pos_or_panic!(0.01), // dividend_yield
            None,
        ))
    }

    #[test]
    fn test_get_strike_versus_delta() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();

        let result = surfaces.get_surface_strike_versus(&BasicAxisTypes::Delta, &option);

        assert!(result.is_ok());
        let (strike, vol, delta) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(vol, dec!(0.2));
        assert!(delta >= dec!(-1.0) && delta <= dec!(1.0));
    }

    #[test]
    fn test_get_strike_versus_gamma() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();

        let result = surfaces.get_surface_strike_versus(&BasicAxisTypes::Gamma, &option);

        assert!(result.is_ok());
        let (strike, vol, gamma) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(vol, dec!(0.2));
        assert!(gamma >= dec!(0.0));
    }

    #[test]
    fn test_get_strike_versus_theta() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();

        let result = surfaces.get_surface_strike_versus(&BasicAxisTypes::Theta, &option);

        assert!(result.is_ok());
        let (strike, vol, theta) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(vol, dec!(0.2));
        assert!(theta <= dec!(0.0));
    }

    #[test]
    fn test_get_strike_versus_vega() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();

        let result = surfaces.get_surface_strike_versus(&BasicAxisTypes::Vega, &option);

        assert!(result.is_ok());
        let (strike, vol, vega) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(vol, dec!(0.2));
        assert!(vega >= dec!(0.0));
    }

    #[test]
    fn test_get_strike_versus_vanna() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();

        let result = surfaces.get_surface_strike_versus(&BasicAxisTypes::Vanna, &option);

        assert!(result.is_ok());
        let (strike, vol, vanna) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(vol, dec!(0.2));
        assert!(vanna <= dec!(0.0));
    }

    #[test]
    fn test_get_strike_versus_vomma() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();

        let result = surfaces.get_surface_strike_versus(&BasicAxisTypes::Vomma, &option);

        assert!(result.is_ok());
        let (strike, vol, vomma) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(vol, dec!(0.2));
        assert!(vomma >= dec!(0.0));
    }

    #[test]
    fn test_get_strike_versus_veta() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();

        let result = surfaces.get_surface_strike_versus(&BasicAxisTypes::Veta, &option);

        assert!(result.is_ok());
        let (strike, vol, veta) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(vol, dec!(0.2));
        assert!(veta >= dec!(0.0));
    }

    #[test]
    fn test_get_strike_versus_charm() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();

        let result = surfaces.get_surface_strike_versus(&BasicAxisTypes::Charm, &option);

        assert!(result.is_ok());
        let (strike, vol, charm) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(vol, dec!(0.2));
        assert!(charm <= dec!(0.0));
    }

    #[test]
    fn test_get_strike_versus_color() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();

        let result = surfaces.get_surface_strike_versus(&BasicAxisTypes::Color, &option);

        assert!(result.is_ok());
        let (strike, vol, color) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(vol, dec!(0.2));
        assert!(color <= dec!(0.0));
    }

    #[test]
    fn test_get_strike_versus_invalid_axis() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();

        let result = surfaces.get_surface_strike_versus(&BasicAxisTypes::Strike, &option);

        assert!(result.is_err());
        match result {
            Err(SurfaceError::OperationError(error)) => {
                assert!(error.to_string().contains("not supported"));
            }
            _ => panic!("Expected OperationError"),
        }
    }

    #[test]
    fn test_get_volatility_versus_with_custom_volatility() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();
        let custom_vol = pos_or_panic!(0.3);

        let result =
            surfaces.get_surface_volatility_versus(&BasicAxisTypes::Delta, &option, custom_vol);

        assert!(result.is_ok());
        let (strike, vol, delta) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(vol, dec!(0.3));
        assert!(delta >= dec!(-1.0) && delta <= dec!(1.0));

        let result = surfaces.get_surface_volatility_versus(
            &BasicAxisTypes::Volatility,
            &option,
            custom_vol,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SurfaceError::OperationError(_)));
    }

    #[test]
    fn test_get_volatility_versus_price() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();
        let custom_vol = pos_or_panic!(0.25);

        let result =
            surfaces.get_surface_volatility_versus(&BasicAxisTypes::Price, &option, custom_vol);

        assert!(result.is_ok());
        let (strike, vol, price) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(vol, dec!(0.25));
        assert!(price > dec!(0.0));
    }

    #[test]
    fn test_get_volatility_versus_theta() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();
        let custom_vol = pos_or_panic!(0.15);

        let result =
            surfaces.get_surface_volatility_versus(&BasicAxisTypes::Theta, &option, custom_vol);

        assert!(result.is_ok());
        let (strike, vol, theta) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(vol, dec!(0.15));
        assert!(theta <= dec!(0.0)); // Theta should be negative or zero
    }

    #[test]
    fn test_get_volatility_versus_gamma() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();
        let custom_vol = pos_or_panic!(0.15);

        let result =
            surfaces.get_surface_volatility_versus(&BasicAxisTypes::Gamma, &option, custom_vol);

        assert!(result.is_ok());
        let (strike, vol, gamma) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(vol, dec!(0.15));
        assert!(gamma >= dec!(0.0));
    }

    #[test]
    fn test_get_volatility_versus_vega() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();
        let custom_vol = pos_or_panic!(0.4);

        let result =
            surfaces.get_surface_volatility_versus(&BasicAxisTypes::Vega, &option, custom_vol);

        assert!(result.is_ok());
        let (strike, vol, vega) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(vol, dec!(0.4));
        assert!(vega >= dec!(0.0)); // Vega should be positive
    }

    #[test]
    fn test_get_volatility_versus_vanna() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();
        let custom_vol = pos_or_panic!(0.4);

        let result =
            surfaces.get_surface_volatility_versus(&BasicAxisTypes::Vanna, &option, custom_vol);

        assert!(result.is_ok());
        let (strike, vol, vanna) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(vol, dec!(0.4));
        assert!(vanna >= dec!(0.0));
    }

    #[test]
    fn test_get_volatility_versus_vomma() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();
        let custom_vol = pos_or_panic!(0.4);

        let result =
            surfaces.get_surface_volatility_versus(&BasicAxisTypes::Vomma, &option, custom_vol);

        assert!(result.is_ok());
        let (strike, vol, vomma) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(vol, dec!(0.4));
        assert!(vomma <= dec!(0.0));
    }

    #[test]
    fn test_get_volatility_versus_veta() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();
        let custom_vol = pos_or_panic!(0.4);

        let result =
            surfaces.get_surface_volatility_versus(&BasicAxisTypes::Veta, &option, custom_vol);

        assert!(result.is_ok());
        let (strike, vol, veta) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(vol, dec!(0.4));
        assert!(veta >= dec!(0.0));
    }

    #[test]
    fn test_get_volatility_versus_charm() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();
        let custom_vol = pos_or_panic!(0.4);

        let result =
            surfaces.get_surface_volatility_versus(&BasicAxisTypes::Charm, &option, custom_vol);

        assert!(result.is_ok());
        let (strike, vol, charm) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(vol, dec!(0.4));
        assert_decimal_eq!(charm, dec!(-0.000506), dec!(0.000001));
    }

    #[test]
    fn test_get_volatility_versus_color() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();
        let custom_vol = pos_or_panic!(0.4);

        let result =
            surfaces.get_surface_volatility_versus(&BasicAxisTypes::Color, &option, custom_vol);

        assert!(result.is_ok());
        let (strike, vol, color) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(vol, dec!(0.4));
        assert_decimal_eq!(color, dec!(-0.000582), dec!(0.000001));
    }

    #[test]
    fn test_compare_strike_and_volatility_versus() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();
        let vol = option.implied_volatility;

        let strike_result = surfaces
            .get_surface_strike_versus(&BasicAxisTypes::Delta, &option)
            .unwrap();
        let vol_result = surfaces
            .get_surface_volatility_versus(&BasicAxisTypes::Delta, &option, vol)
            .unwrap();

        // Both methods should return the same results when using the same volatility
        assert_eq!(strike_result.0, vol_result.0); // Strike
        assert_eq!(strike_result.1, vol_result.1); // Volatility
        assert_eq!(strike_result.2, vol_result.2); // Delta
    }

    #[test]
    fn test_get_time_versus_veta() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();
        let days = pos_or_panic!(30.0);

        let result = surfaces.get_surface_time_versus(&BasicAxisTypes::Veta, &option, days);

        assert!(result.is_ok());
        let (strike, time, veta) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(time, dec!(30.0));
        // Veta should be a small positive or negative value
        assert!(veta.abs() < dec!(1.0));
    }

    #[test]
    fn test_get_time_versus_charm() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();
        let days = pos_or_panic!(30.0);

        let result = surfaces.get_surface_time_versus(&BasicAxisTypes::Charm, &option, days);

        assert!(result.is_ok());
        let (strike, time, charm) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(time, dec!(30.0));
        // Charm should be a small value
        assert!(charm.abs() < dec!(1.0));
    }

    #[test]
    fn test_get_time_versus_color() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();
        let days = pos_or_panic!(30.0);

        let result = surfaces.get_surface_time_versus(&BasicAxisTypes::Color, &option, days);

        assert!(result.is_ok());
        let (strike, time, color) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(time, dec!(30.0));
        // Color should be a small value
        assert!(color.abs() < dec!(1.0));
    }

    #[test]
    fn test_get_time_versus_theta() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();
        let days = pos_or_panic!(30.0);

        let result = surfaces.get_surface_time_versus(&BasicAxisTypes::Theta, &option, days);

        assert!(result.is_ok());
        let (strike, time, theta) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(time, dec!(30.0));
        // Theta should be negative for long options
        assert!(theta <= dec!(0.0));
    }

    #[test]
    fn test_get_time_versus_vanna() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();
        let days = pos_or_panic!(30.0);

        let result = surfaces.get_surface_time_versus(&BasicAxisTypes::Vanna, &option, days);

        assert!(result.is_ok());
        let (strike, time, _vanna) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(time, dec!(30.0));
    }

    #[test]
    fn test_get_time_versus_vomma() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();
        let days = pos_or_panic!(30.0);

        let result = surfaces.get_surface_time_versus(&BasicAxisTypes::Vomma, &option, days);

        assert!(result.is_ok());
        let (strike, time, _vomma) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(time, dec!(30.0));
    }

    #[test]
    fn test_get_time_versus_invalid_axis() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();
        let days = pos_or_panic!(30.0);

        let result = surfaces.get_surface_time_versus(&BasicAxisTypes::Strike, &option, days);

        assert!(result.is_err());
        match result {
            Err(SurfaceError::OperationError(error)) => {
                assert!(error.to_string().contains("not supported"));
            }
            _ => panic!("Expected OperationError"),
        }
    }

    #[test]
    fn test_get_time_versus_different_times() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();

        // Test with different time horizons
        let times = vec![
            pos_or_panic!(7.0),
            pos_or_panic!(14.0),
            pos_or_panic!(30.0),
            pos_or_panic!(60.0),
            pos_or_panic!(90.0),
        ];

        for days in times {
            let result = surfaces.get_surface_time_versus(&BasicAxisTypes::Veta, &option, days);
            assert!(result.is_ok());
            let (_, time, _) = result.unwrap();
            assert_eq!(time, days.to_dec());
        }
    }

    #[test]
    fn test_get_time_versus_near_expiration() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();
        let days = Positive::ONE; // Near expiration

        let result = surfaces.get_surface_time_versus(&BasicAxisTypes::Veta, &option, days);

        assert!(result.is_ok());
        let (strike, time, _veta) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(time, dec!(1.0));
    }

    #[test]
    fn test_get_time_versus_price() {
        let surfaces = MockBasicSurfaces;
        let option = create_test_option();
        let days = pos_or_panic!(30.0);

        let result = surfaces.get_surface_time_versus(&BasicAxisTypes::Price, &option, days);

        assert!(result.is_ok());
        let (strike, time, price) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(time, dec!(30.0));
        assert!(price > dec!(0.0));
    }
}
