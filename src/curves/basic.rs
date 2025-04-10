/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 9/2/25
******************************************************************************/
use crate::curves::Curve;
use crate::error::CurveError;
use crate::greeks::Greeks;
use crate::model::BasicAxisTypes;
use crate::{OptionStyle, Options, Side};
use rust_decimal::Decimal;
use std::sync::Arc;

/// A trait for generating financial option curves based on different parameters.
///
/// This trait provides methods to create and retrieve option curves based on various
/// financial metrics. It allows for the generation of curves that plot relationships
/// between option strike prices and different option Greeks (Delta, Gamma, Theta, Vega),
/// implied volatility, or prices.
///
/// Implementors of this trait can define custom curve generation logic while using
/// the default implementation for extracting coordinate pairs for specific option metrics.
///
/// # Type Parameters
///
/// The trait is designed to work with options data structures and can generate curves
/// for different visualization and analysis purposes.
pub trait BasicCurves {
    /// Generates a curve for the specified axis type, option style, and market side.
    ///
    /// This method creates a curve that represents the relationship between strike prices
    /// and the selected option metric (as specified by the axis parameter).
    ///
    /// # Parameters
    ///
    /// * `axis` - The financial metric to be plotted on one of the axes (e.g., Delta, Gamma, Price)
    /// * `option_style` - The style of the option (Call or Put)
    /// * `side` - The market side perspective (Long or Short)
    ///
    /// # Returns
    ///
    /// * `Result<Curve, CurveError>` - A curve object containing the plotted data points,
    ///   or an error if the curve could not be generated
    fn curve(
        &self,
        axis: &BasicAxisTypes,
        option_style: &OptionStyle,
        side: &Side,
    ) -> Result<Curve, CurveError>;

    /// Generates coordinate pairs for a specific option and axis type.
    ///
    /// This method extracts a pair of values (strike price and the selected metric)
    /// from an option based on the specified axis type. The first value in the pair
    /// is always the strike price, and the second value is determined by the axis type.
    ///
    /// # Parameters
    ///
    /// * `axis` - The financial metric to extract (e.g., Delta, Gamma, Implied Volatility)
    /// * `option` - The option contract from which to extract the values
    ///
    /// # Returns
    ///
    /// * `Result<(Decimal, Decimal), CurveError>` - A tuple containing (strike price, metric value),
    ///   or an error if the values could not be extracted
    ///
    fn get_curve_strike_versus(
        &self,
        axis: &BasicAxisTypes,
        option: &Arc<Options>,
    ) -> Result<(Decimal, Decimal), CurveError> {
        match axis {
            BasicAxisTypes::Delta => Ok((option.strike_price.to_dec(), option.delta()?)),
            BasicAxisTypes::Gamma => Ok((option.strike_price.to_dec(), option.gamma()?)),
            BasicAxisTypes::Theta => Ok((option.strike_price.to_dec(), option.theta()?)),
            BasicAxisTypes::Vega => Ok((option.strike_price.to_dec(), option.vega()?)),
            BasicAxisTypes::Volatility => Ok((
                option.strike_price.to_dec(),
                option.implied_volatility.to_dec(),
            )),
            BasicAxisTypes::Price => Ok((
                option.strike_price.to_dec(),
                option.calculate_price_black_scholes()?,
            )),
            // Catch-all for unsupported combinations
            _ => Err(CurveError::OperationError(
                crate::error::OperationErrorKind::InvalidParameters {
                    operation: "get_axis_value".to_string(),
                    reason: format!("Axis: {:?} not supported", axis),
                },
            )),
        }
    }
}

#[cfg(test)]
mod tests_basic_curves_trait {
    use super::*;
    use crate::curves::Point2D;
    use crate::model::types::{OptionStyle, Side};
    use crate::{ExpirationDate, OptionType, Positive, pos};
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;
    use std::sync::Arc;
    use crate::error::OperationErrorKind;

    // Helper function to create a sample Options for testing
    fn create_test_option() -> Arc<Options> {
        Arc::new(Options::new(
            OptionType::European,
            Side::Long,
            "AAPL".to_string(),
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            Positive::ONE,
            pos!(105.0),
            dec!(0.05),
            OptionStyle::Call,
            pos!(0.01),
            None,
        ))
    }

    // Mock implementation of BasicCurves for testing
    struct TestBasicCurves;

    impl BasicCurves for TestBasicCurves {
        fn curve(
            &self,
            axis: &BasicAxisTypes,
            _option_style: &OptionStyle,
            _side: &Side,
        ) -> Result<Curve, CurveError> {
            // Simplified implementation for testing
            let option = create_test_option();
            let point = self.get_curve_strike_versus(axis, &option)?;
            Ok(Curve::new(BTreeSet::from([Point2D::new(point.0, point.1)])))
        }
    }

    #[test]
    fn test_get_strike_versus_delta() {
        let test_curves = TestBasicCurves;
        let option = create_test_option();

        let result = test_curves.get_curve_strike_versus(&BasicAxisTypes::Delta, &option);

        assert!(result.is_ok());
        let (x, y) = result.unwrap();

        assert_eq!(x, option.strike_price.to_dec());
        assert!(y.abs() <= dec!(1.0)); // Delta should be between -1 and 1
    }

    #[test]
    fn test_get_strike_versus_gamma() {
        let test_curves = TestBasicCurves;
        let option = create_test_option();

        let result = test_curves.get_curve_strike_versus(&BasicAxisTypes::Gamma, &option);

        assert!(result.is_ok());
        let (x, y) = result.unwrap();

        assert_eq!(x, option.strike_price.to_dec());
        assert!(y >= Decimal::ZERO); // Gamma is always non-negative
    }

    #[test]
    fn test_get_strike_versus_theta() {
        let test_curves = TestBasicCurves;
        let option = create_test_option();

        let result = test_curves.get_curve_strike_versus(&BasicAxisTypes::Theta, &option);

        assert!(result.is_ok());
        let (x, _y) = result.unwrap();

        assert_eq!(x, option.strike_price.to_dec());
        // Theta can be positive or negative
    }

    #[test]
    fn test_get_strike_versus_vega() {
        let test_curves = TestBasicCurves;
        let option = create_test_option();

        let result = test_curves.get_curve_strike_versus(&BasicAxisTypes::Vega, &option);

        assert!(result.is_ok());
        let (x, y) = result.unwrap();

        assert_eq!(x, option.strike_price.to_dec());
        assert!(y >= Decimal::ZERO); // Vega is always non-negative
    }

    #[test]
    fn test_get_strike_versus_volatility() {
        let test_curves = TestBasicCurves;
        let option = create_test_option();

        let result = test_curves.get_curve_strike_versus(&BasicAxisTypes::Volatility, &option);

        assert!(result.is_ok());
        let (x, y) = result.unwrap();

        assert_eq!(x, option.strike_price.to_dec());
        assert_eq!(y, option.implied_volatility.to_dec());
    }

    #[test]
    fn test_get_strike_versus_price() {
        let test_curves = TestBasicCurves;
        let option = create_test_option();

        let result = test_curves.get_curve_strike_versus(&BasicAxisTypes::Price, &option);

        assert!(result.is_ok());
        let (x, y) = result.unwrap();

        assert_eq!(x, option.strike_price.to_dec());
        assert!(y > Decimal::ZERO); // Price should be positive
    }

    #[test]
    fn test_curve_method() {
        let test_curves = TestBasicCurves;

        let curve_result =
            test_curves.curve(&BasicAxisTypes::Delta, &OptionStyle::Call, &Side::Long);

        assert!(curve_result.is_ok());
        let curve = curve_result.unwrap();

        assert_eq!(curve.points.len(), 1);
    }

    #[test]
    fn test_get_strike_versus_black_scholes_price() {
        let test_curves = TestBasicCurves;
        let option = create_test_option();
        let result = test_curves.get_curve_strike_versus(&BasicAxisTypes::Price, &option);

        assert!(result.is_ok());
        let (strike, price) = result.unwrap();

        assert_eq!(strike, option.strike_price.to_dec());

        assert!(price > Decimal::ZERO);
        let direct_bs_price = option.calculate_price_black_scholes().unwrap();
        assert_eq!(price, direct_bs_price);
    }

    #[test]
    fn test_get_strike_versus_unsupported_axis() {
        let test_curves = TestBasicCurves;
        let option = create_test_option();
        let result = test_curves.get_curve_strike_versus(&BasicAxisTypes::Expiration, &option);

        assert!(result.is_err());
        match result {
            Err(CurveError::OperationError(
                crate::error::OperationErrorKind::InvalidParameters { operation, reason },
            )) => {
                assert_eq!(operation, "get_axis_value");
                assert!(reason.contains("not supported"));
                assert!(reason.contains("Expiration"));
            }
            _ => panic!("Expected OperationError with InvalidParameters"),
        }
    }

    // Add to src/curves/basic.rs in the tests_basic_curves_trait module

    #[test]
    fn test_invalid_axis_error_message() {
        // Test the specific error message format for an unsupported axis
        let test_curves = TestBasicCurves;
        let option = create_test_option();

        // Line 81: Tests the specific error formatting for the OperationErrorKind::InvalidParameters
        let result = test_curves.get_curve_strike_versus(&BasicAxisTypes::Expiration, &option);

        assert!(result.is_err());
        if let Err(CurveError::OperationError(OperationErrorKind::InvalidParameters { operation, reason })) = result {
            assert_eq!(operation, "get_axis_value");
            assert!(reason.contains("Axis: Expiration not supported"));
        } else {
            panic!("Expected OperationError with InvalidParameters");
        }
    }

    // Add a test for the curve method with different option styles and sides
    #[test]
    fn test_curve_with_various_params() {
        let test_curves = TestBasicCurves;

        // Test with different combinations of option style and side
        let curve_call_long = test_curves.curve(&BasicAxisTypes::Delta, &OptionStyle::Call, &Side::Long);
        let curve_call_short = test_curves.curve(&BasicAxisTypes::Delta, &OptionStyle::Call, &Side::Short);
        let curve_put_long = test_curves.curve(&BasicAxisTypes::Delta, &OptionStyle::Put, &Side::Long);
        let curve_put_short = test_curves.curve(&BasicAxisTypes::Delta, &OptionStyle::Put, &Side::Short);

        assert!(curve_call_long.is_ok());
        assert!(curve_call_short.is_ok());
        assert!(curve_put_long.is_ok());
        assert!(curve_put_short.is_ok());
    }
}
