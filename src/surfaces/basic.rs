/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 9/2/25
 ******************************************************************************/
use std::sync::Arc;
use rust_decimal::Decimal;
use crate::model::BasicAxisTypes;
use crate::{OptionStyle, Options, Positive, Side};
use crate::error::SurfaceError;
use crate::greeks::Greeks;
use crate::surfaces::Surface;

pub trait BasicSurfaces {
    /// Creates a surface based on specified axes and option parameters
    fn surface(
        &self,
        primary_axis: &BasicAxisTypes,
        option_style: &OptionStyle,
        side: &Side,
    ) -> Result<Surface, SurfaceError>;

    /// Gets the values for a surface point, where volatility is always one of the axes
    /// Gets the values for a surface point with a specified volatility level
    fn get_strike_volatility_versus(
        &self,
        axis: &BasicAxisTypes,
        option: &Arc<Options>,
        volatility: Positive,
    ) -> Result<(Decimal, Decimal, Decimal), SurfaceError> {
        // Create a modified copy of the option with the specified volatility
        let mut option_with_vol = (**option).clone();
        option_with_vol.implied_volatility = volatility.try_into()
            .map_err(|_| SurfaceError::OperationError(
                crate::error::OperationErrorKind::InvalidParameters {
                    operation: "get_strike_volatility_versus".to_string(),
                    reason: "Invalid volatility value".to_string(),
                }
            ))?;

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
                    reason: format!("Axis: {:?} not supported", axis),
                },
            )),
        }
    }

    /// Returns the number of points in the surface
    fn len(&self) -> usize;

    /// Checks if the surface is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
}