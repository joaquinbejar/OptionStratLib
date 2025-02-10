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

pub trait BasicCurves {
    fn curve(
        &self,
        axis: &BasicAxisTypes,
        option_style: &OptionStyle,
        side: &Side,
    ) -> Result<Curve, CurveError>;

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
    use std::collections::BTreeSet;
    use super::*;
    use crate::model::types::{OptionStyle, Side};
    use crate::{pos, ExpirationDate, OptionType, Positive};
    use rust_decimal_macros::dec;
    use std::sync::Arc;
    use crate::curves::Point2D;

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

        let curve_result = test_curves.curve(
            &BasicAxisTypes::Delta,
            &OptionStyle::Call,
            &Side::Long
        );

        assert!(curve_result.is_ok());
        let curve = curve_result.unwrap();

        assert_eq!(curve.points.len(), 1);
    }
    
}