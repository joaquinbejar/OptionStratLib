use super::base::{Positionable, StrategyType};
use crate::{
    ExpirationDate, Options, Positive,
    error::position::{PositionError, PositionValidationErrorKind},
    model::{
        position::Position,
        types::{OptionStyle, OptionType, Side},
    },
};
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub(super) const LONG_PUT_DESCRIPTION: &str = "A Long Put is an options strategy where the trader purchases a put option, gaining the right (but not the obligation) to sell the underlying asset at the strike price until expiration. \
    This strategy requires an initial investment (the premium paid) and provides downside protection or profit potential if the underlying asset's price decreases. \
    The breakeven point is the strike price minus the premium paid. Long puts are commonly used as insurance against price declines or to express a bearish outlook.";

/// Represents a Long Put options trading strategy.
///
/// A Long Put strategy is used when a trader expects the price of the underlying asset
/// to decrease significantly. It involves purchasing a put option with the anticipation
/// of profiting as the underlying asset's price falls below the strike price of the option.
///
/// # Fields
///
/// * `name` - A unique name identifier for this specific instance of the Long Put strategy.
/// * `kind` - The type of strategy, identified specifically as `StrategyType::LongPut`.
/// * `description` - A detailed description of this particular instance of the Long Put strategy,
///   providing additional context or information.
/// * `break_even_points` - A vector of price points (`Positive`) where the strategy neither gains
///   nor loses money based on the underlying asset's movement.
/// * `long_put` - Represents the specific long put position within the strategy, detailing
///   the option contract being used.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LongPut {
    /// Name identifier for this specific strategy instance
    pub name: String,
    /// Identifies this as a LongPut strategy type
    pub kind: StrategyType,
    /// Detailed description of this strategy instance
    pub description: String,
    /// Price points where the strategy neither makes nor loses money
    pub break_even_points: Vec<Positive>,
    /// The long put position
    pub(super) long_put: Position,
}

impl LongPut {
    /// Constructs a new instance of a `LongPut` strategy.
    ///
    /// This method initializes a `LongPut` strategy based on the given parameters,
    /// including details about the underlying asset, option specifications, and associated fees.
    ///
    /// # Parameters
    ///
    /// * `underlying_symbol` - Symbol of the underlying asset for the long put option.
    /// * `long_put_strike` - Strike price of the long put option. Must be a positive value.
    /// * `long_put_expiration` - Expiration date of the long put option.
    /// * `implied_volatility` - Implied volatility of the underlying asset. Must be a positive value.
    /// * `quantity` - Quantity of contracts. Must be a positive value.
    /// * `underlying_price` - Current price of the underlying asset. Must be a positive value.
    /// * `risk_free_rate` - Risk-free rate used for pricing the option (in decimal format).
    /// * `dividend_yield` - Dividend yield of the underlying asset. Must be a positive value.
    /// * `premium_long_put` - Premium cost of the long put option. Must be a positive value.
    /// * `open_fee_long_put` - Fee incurred when opening the long put position. Must be a positive value.
    /// * `close_fee_long_put` - Fee incurred when closing the long put position. Must be a positive value.
    ///
    /// # Returns
    ///
    /// A new instance of the `LongPut` strategy initialized with the provided parameters.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// * The `add_position` method fails, which could happen due to invalid configurations of the long put position.
    ///
    #[allow(clippy::too_many_arguments, dead_code)]
    fn new(
        underlying_symbol: String,
        long_put_strike: Positive,
        long_put_expiration: ExpirationDate,
        implied_volatility: Positive,
        quantity: Positive,
        underlying_price: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        premium_long_put: Positive,
        open_fee_long_put: Positive,
        close_fee_long_put: Positive,
    ) -> Self {
        let mut strategy = LongPut::default();

        let long_put_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol,
            long_put_strike,
            long_put_expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Put,
            dividend_yield,
            None,
        );
        let long_put = Position::new(
            long_put_option,
            premium_long_put,
            Utc::now(),
            open_fee_long_put,
            close_fee_long_put,
        );
        strategy
            .add_position(&long_put.clone())
            .expect("Invalid long put option");

        strategy
    }
}

impl Positionable for LongPut {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match (position.option.option_style, position.option.side) {
            (OptionStyle::Put, Side::Long) => {
                self.long_put = position.clone();
                Ok(())
            }
            _ => Err(PositionError::invalid_position_style(
                position.option.option_style,
                "Position is a Put or Long, it is not valid for LongPut".to_string(),
            )),
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.long_put])
    }

    /// Gets mutable positions matching the specified criteria from the strategy.
    ///
    /// # Arguments
    /// * `option_style` - The style of the option (Put/Put)
    /// * `side` - The side of the position (Long/Long)
    /// * `strike` - The strike price of the option
    ///
    /// # Returns
    /// * `Ok(Vec<&mut Position>)` - A vector containing mutable references to matching positions
    /// * `Err(PositionError)` - If there was an error retrieving positions
    fn get_position(
        &mut self,
        option_style: &OptionStyle,
        side: &Side,
        strike: &Positive,
    ) -> Result<Vec<&mut Position>, PositionError> {
        match (side, option_style, strike) {
            (Side::Long, OptionStyle::Put, strike)
                if *strike == self.long_put.option.strike_price =>
            {
                Ok(vec![&mut self.long_put])
            }
            _ => Err(PositionError::invalid_position_type(
                *side,
                "Position not found".to_string(),
            )),
        }
    }

    /// Modifies an existing position in the strategy.
    ///
    /// # Arguments
    /// * `position` - The new position data to update
    ///
    /// # Returns
    /// * `Ok(())` if position was successfully modified
    /// * `Err(PositionError)` if position was not found or validation failed
    fn modify_position(&mut self, position: &Position) -> Result<(), PositionError> {
        if !position.validate() {
            return Err(PositionError::ValidationError(
                PositionValidationErrorKind::InvalidPosition {
                    reason: "Invalid position data".to_string(),
                },
            ));
        }

        match (
            &position.option.side,
            &position.option.option_style,
            &position.option.strike_price,
        ) {
            (Side::Long, OptionStyle::Put, strike)
                if *strike == self.long_put.option.strike_price =>
            {
                self.long_put = position.clone();
            }
            _ => {
                return Err(PositionError::invalid_position_type(
                    position.option.side,
                    "Position not found".to_string(),
                ));
            }
        }

        Ok(())
    }
}

// test_strategy_traits!(LongPut, test_long_put_implementations);
