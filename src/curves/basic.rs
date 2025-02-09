/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 9/2/25
 ******************************************************************************/
use std::collections::HashMap;
use std::sync::Arc;
use chrono::Month::March;
use rust_decimal::Decimal;
use crate::curves::Curve;
use crate::error::CurveError;
use crate::model::BasicAxisTypes;
use crate::{OptionStyle, Options, Positive, Side};
use crate::chains::chain::OptionData;
use crate::greeks::Greeks;

pub trait BasicCurves {
    fn curve(
        &self,
        axis: (&BasicAxisTypes, &BasicAxisTypes),
        option_style: &OptionStyle,
        side: &Side,
    ) -> Result<Curve, CurveError>;

    fn curve_combinations(
        &self,
        axis: (&BasicAxisTypes, &BasicAxisTypes),
    ) -> Result<HashMap<(OptionStyle, Side), Curve>, CurveError> {
        let mut results = HashMap::new();

        let styles = [OptionStyle::Call, OptionStyle::Put];
        let sides = [Side::Long, Side::Short];

        for style in &styles {
            for side in &sides {
                let key = (style.clone(), side.clone());
                let curve = self.curve(axis, style, side)?;
                results.insert(key, curve);
            }
        }
        Ok(results)
    }

    fn get_axis_value(
        &self,
        item: usize,
        axis: (&BasicAxisTypes, &BasicAxisTypes),
        option: &Arc<Options>,
    ) -> Result<(Decimal, Decimal), CurveError> {
        match (axis.0, axis.1) {
            // Delta combinations
            (BasicAxisTypes::Delta, BasicAxisTypes::Price) | (BasicAxisTypes::Price, BasicAxisTypes::Delta) => {
                Ok((option.delta()?, option.calculate_price_black_scholes()?))
            },
            (BasicAxisTypes::Delta, BasicAxisTypes::Gamma) | (BasicAxisTypes::Gamma, BasicAxisTypes::Delta) => {
                Ok((option.gamma()?, option.delta()?))
            },
            (BasicAxisTypes::Delta, BasicAxisTypes::Theta) | (BasicAxisTypes::Theta, BasicAxisTypes::Delta) => {
                Ok((option.delta()?, option.theta()?))
            },
            (BasicAxisTypes::Delta, BasicAxisTypes::Vega) | (BasicAxisTypes::Vega, BasicAxisTypes::Delta) => {
                Ok((option.delta()?, option.vega()?))
            },
            (BasicAxisTypes::Delta, BasicAxisTypes::Volatility) | (BasicAxisTypes::Volatility, BasicAxisTypes::Delta) => {
                Ok((option.delta()?, option.implied_volatility.to_dec()))
            },
            (BasicAxisTypes::Delta, BasicAxisTypes::Strike) | (BasicAxisTypes::Strike, BasicAxisTypes::Delta) => {
                Ok()
            },
            (BasicAxisTypes::Delta, BasicAxisTypes::UnderlyingPrice) | (BasicAxisTypes::UnderlyingPrice, BasicAxisTypes::Delta) => {
                Ok()
            },
            (BasicAxisTypes::Delta, BasicAxisTypes::Expiration) | (BasicAxisTypes::Expiration, BasicAxisTypes::Delta) => {
                Ok()
            },



            // Gamma combinations
            (BasicAxisTypes::Gamma, BasicAxisTypes::Price) | (BasicAxisTypes::Price, BasicAxisTypes::Gamma) => {
                Ok((option.delta()?, option.calculate_price_black_scholes()?))
            },
            (BasicAxisTypes::Gamma, BasicAxisTypes::Theta) | (BasicAxisTypes::Theta, BasicAxisTypes::Gamma) => {
                Ok((option.delta()?, option.theta()?))
            },
            (BasicAxisTypes::Gamma, BasicAxisTypes::Vega) | (BasicAxisTypes::Vega, BasicAxisTypes::Gamma) => {
                Ok((option.delta()?, option.vega()?))
            },
            (BasicAxisTypes::Gamma, BasicAxisTypes::Volatility) | (BasicAxisTypes::Volatility, BasicAxisTypes::Gamma) => {
                Ok((option.delta()?, option.implied_volatility.to_dec()))
            },
            (BasicAxisTypes::Gamma, BasicAxisTypes::Strike) | (BasicAxisTypes::Strike, BasicAxisTypes::Gamma) => {
                todo!("Gamma and Strike")
            },
            (BasicAxisTypes::Gamma, BasicAxisTypes::UnderlyingPrice) | (BasicAxisTypes::UnderlyingPrice, BasicAxisTypes::Gamma) => {
                todo!("Gamma and UnderlyingPrice")
            },
            (BasicAxisTypes::Gamma, BasicAxisTypes::Expiration) | (BasicAxisTypes::Expiration, BasicAxisTypes::Gamma) => {
                todo!("Gamma and Expiration")
            },
            
            
            
            // Theta combinations
            (BasicAxisTypes::Theta, BasicAxisTypes::Price) | (BasicAxisTypes::Price, BasicAxisTypes::Theta) => {
                todo!("Theta and Price")
            },
            (BasicAxisTypes::Theta, BasicAxisTypes::Vega) | (BasicAxisTypes::Vega, BasicAxisTypes::Theta) => {
                todo!("Theta and Vega")
            },
            (BasicAxisTypes::Theta, BasicAxisTypes::Volatility) | (BasicAxisTypes::Volatility, BasicAxisTypes::Theta) => {
                todo!("Theta and Volatility")
            },
            (BasicAxisTypes::Theta, BasicAxisTypes::Strike) | (BasicAxisTypes::Strike, BasicAxisTypes::Theta) => {
                todo!("Theta and Strike")
            },
            (BasicAxisTypes::Theta, BasicAxisTypes::UnderlyingPrice) | (BasicAxisTypes::UnderlyingPrice, BasicAxisTypes::Theta) => {
                todo!("Theta and UnderlyingPrice")
            },
            (BasicAxisTypes::Theta, BasicAxisTypes::Expiration) | (BasicAxisTypes::Expiration, BasicAxisTypes::Theta) => {
                todo!("Theta and Expiration")
            },
            
            
            
            // Vega combinations
            (BasicAxisTypes::Vega, BasicAxisTypes::Price) | (BasicAxisTypes::Price, BasicAxisTypes::Vega) => {
                todo!("Vega and Price")
            },
            (BasicAxisTypes::Vega, BasicAxisTypes::Volatility) | (BasicAxisTypes::Volatility, BasicAxisTypes::Vega) => {
                todo!("Vega and Volatility")
            },
            (BasicAxisTypes::Vega, BasicAxisTypes::Strike) | (BasicAxisTypes::Strike, BasicAxisTypes::Vega) => {
                todo!("Vega and Strike")
            },
            (BasicAxisTypes::Vega, BasicAxisTypes::UnderlyingPrice) | (BasicAxisTypes::UnderlyingPrice, BasicAxisTypes::Vega) => {
                todo!("Vega and UnderlyingPrice")
            },
            (BasicAxisTypes::Vega, BasicAxisTypes::Expiration) | (BasicAxisTypes::Expiration, BasicAxisTypes::Vega) => {
                todo!("Vega and Expiration")
            },
            
            
            
            // Volatility combinations
            (BasicAxisTypes::Volatility, BasicAxisTypes::Price) | (BasicAxisTypes::Price, BasicAxisTypes::Volatility) => {
                todo!("Volatility and Price")
            },
            (BasicAxisTypes::Volatility, BasicAxisTypes::Strike) | (BasicAxisTypes::Strike, BasicAxisTypes::Volatility) => {
                todo!("Volatility and Strike")
            },
            (BasicAxisTypes::Volatility, BasicAxisTypes::UnderlyingPrice) | (BasicAxisTypes::UnderlyingPrice, BasicAxisTypes::Volatility) => {
                todo!("Volatility and UnderlyingPrice")
            },
            (BasicAxisTypes::Volatility, BasicAxisTypes::Expiration) | (BasicAxisTypes::Expiration, BasicAxisTypes::Volatility) => {
                todo!("Volatility and Expiration")
            },
            
            
            // Price combinations
            (BasicAxisTypes::Price, BasicAxisTypes::Strike) | (BasicAxisTypes::Strike, BasicAxisTypes::Price) => {
                todo!("Price and Strike")
            },
            (BasicAxisTypes::Price, BasicAxisTypes::UnderlyingPrice) | (BasicAxisTypes::UnderlyingPrice, BasicAxisTypes::Price) => {
                todo!("Price and UnderlyingPrice")
            },
            (BasicAxisTypes::Price, BasicAxisTypes::Expiration) | (BasicAxisTypes::Expiration, BasicAxisTypes::Price) => {
                todo!("Price and Expiration")
            },
            


            // Catch-all for unsupported combinations
            _ => Err(CurveError::OperationError(crate::error::OperationErrorKind::InvalidParameters {
                operation: "get_axis_value".to_string(),
                reason: format!("Invalid axis combination: {:?} - {:?}", axis.0, axis.1),
            }))
        }
    }
    
    fn len(&self) -> usize;
    
}


#[cfg(test)]
mod tests {
    use crate::error::OperationErrorKind;
    use super::*;

    // Mock implementation for testing
    struct MockCurveGenerator;

    impl BasicCurves for MockCurveGenerator {
        fn curve(
            &self,
            axis: (&BasicAxisTypes, &BasicAxisTypes),
            _option_style: &OptionStyle,
            _side: &Side,
        ) -> Result<Curve, CurveError> {
            if axis.0 == axis.1 {
                return Err(CurveError::OperationError(OperationErrorKind::InvalidParameters {
                    operation: "curve".to_string(),
                    reason: format!("Cannot use same axis: {:?}",  axis.0)
                }));
            }
            Ok(Curve::default())
        }
    }

    #[test]
    fn test_single_curve_generation() {
        let generator = MockCurveGenerator;
        let result = generator.curve(
            (&BasicAxisTypes::Delta, &BasicAxisTypes::Price),
            &OptionStyle::Call,
            &Side::Long,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_axis_combination() {
        let generator = MockCurveGenerator;
        let result = generator.curve(
            (&BasicAxisTypes::Delta, &BasicAxisTypes::Delta),
            &OptionStyle::Call,
            &Side::Long,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_curve_all_combinations() {
        let generator = MockCurveGenerator;
        let result = generator.curve_combinations(
            (&BasicAxisTypes::Delta, &BasicAxisTypes::Price),
        );
        assert!(result.is_ok());

        let curves = result.unwrap();
        assert_eq!(curves.len(), 4); // 2 styles * 2 sides = 4 combinations

        // Check all combinations exist
        assert!(curves.contains_key(&(OptionStyle::Call, Side::Long)));
        assert!(curves.contains_key(&(OptionStyle::Call, Side::Short)));
        assert!(curves.contains_key(&(OptionStyle::Put, Side::Long)));
        assert!(curves.contains_key(&(OptionStyle::Put, Side::Short)));
    }

    #[test]
    fn test_hashmap_access() {
        let generator = MockCurveGenerator;
        let result = generator.curve_combinations(
            (&BasicAxisTypes::Delta, &BasicAxisTypes::Price),
        ).unwrap();

        // Test direct access to specific configurations
        let call_long = result.get(&(OptionStyle::Call, Side::Long));
        assert!(call_long.is_some());

        let put_short = result.get(&(OptionStyle::Put, Side::Short));
        assert!(put_short.is_some());
    }

    #[test]
    fn test_hashmap_iteration() {
        let generator = MockCurveGenerator;
        let curves = generator.curve_combinations(
            (&BasicAxisTypes::Delta, &BasicAxisTypes::Price),
        ).unwrap();

        let mut count = 0;
        for ((style, side), _curve) in curves.iter() {
            match (style, side) {
                (OptionStyle::Call, Side::Long) |
                (OptionStyle::Call, Side::Short) |
                (OptionStyle::Put, Side::Long) |
                (OptionStyle::Put, Side::Short) => count += 1,
            }
        }
        assert_eq!(count, 4);
    }

    #[test]
    fn test_invalid_axis_all_combinations() {
        let generator = MockCurveGenerator;
        let result = generator.curve_combinations(
            (&BasicAxisTypes::Delta, &BasicAxisTypes::Delta),
        );
        assert!(result.is_err());
    }
}