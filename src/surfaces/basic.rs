/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 9/2/25
******************************************************************************/
use crate::error::SurfaceError;
use crate::greeks::Greeks;
use crate::model::BasicAxisTypes;
use crate::surfaces::Surface;
use crate::{OptionStyle, Options, Positive, Side};
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
    /// desired metric (delta, gamma, theta, vega, or price).
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
    /// to calculate the desired metric (delta, gamma, theta, vega, or price).
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
}

#[cfg(test)]
mod tests_basic_surfaces {
    use super::*;
    use crate::{ExpirationDate, OptionType, pos};
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
    }

    // Helper function to create a test option
    fn create_test_option() -> Arc<Options> {
        Arc::new(Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            pos!(1.0),
            pos!(100.0),
            dec!(0.05),
            OptionStyle::Call,
            pos!(0.01),
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
        let custom_vol = pos!(0.3);

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
        let custom_vol = pos!(0.25);

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
        let custom_vol = pos!(0.15);

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
        let custom_vol = pos!(0.15);

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
        let custom_vol = pos!(0.4);

        let result =
            surfaces.get_surface_volatility_versus(&BasicAxisTypes::Vega, &option, custom_vol);

        assert!(result.is_ok());
        let (strike, vol, vega) = result.unwrap();
        assert_eq!(strike, dec!(100.0));
        assert_eq!(vol, dec!(0.4));
        assert!(vega >= dec!(0.0)); // Vega should be positive
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
}
