use super::base::{
    Positionable, Strategable, StrategyType, Validable,
};
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
use std::error::Error;

pub(super) const LONG_CALL_DESCRIPTION: &str = "A Long Call is an options strategy where the trader buys a call option, acquiring the right (but not the obligation) to purchase the underlying asset at the strike price until expiration. \
    This strategy involves an upfront cost (the premium paid) and offers unlimited profit potential if the underlying asset's price increases significantly. \
    The breakeven point is the strike price plus the premium paid. Long calls are typically used to gain leveraged exposure to potential price increases with defined risk.";

/// Represents a Long Call strategy in options trading.
///
/// A Long Call is an options strategy where an investor purchases call options
/// with the expectation that the underlying asset's price will rise above the
/// strike price before expiration, allowing them to profit.
///
/// # Fields
/// * `name` - A unique identifier for this specific instance of the Long Call strategy.
/// * `kind` - The type of strategy, identified as a `LongCall` within the `StrategyType` enumeration.
/// * `description` - A detailed explanation or notes about this particular Long Call instance.
/// * `break_even_points` - A collection of price levels (as a vector of positive values) where the strategy reaches
///   its break-even — meaning no profit or loss occurs at these points.
/// * `long_call` - The position details representing the long call option, specifying the strike price,
///   premium, and quantity involved. This field is private within the module (`pub(super)` access level).
///
/// # Notes
/// This structure leverages the `Clone`, `Debug`, `Serialize`, and `Deserialize` traits for ease of duplication,
/// debugging, and storage/transfer as structured data.
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LongCall {
    /// Name identifier for this specific strategy instance
    pub name: String,
    /// Identifies this as a LongCall strategy type
    pub kind: StrategyType,
    /// Detailed description of this strategy instance
    pub description: String,
    /// Price points where the strategy neither makes nor loses money
    pub break_even_points: Vec<Positive>,
    /// The long call position
    pub(super) long_call: Position,
}

impl LongCall {
    /// Creates a new instance of a `LongCall` strategy with the provided parameters.
    ///
    /// The `new` function initializes a `LongCall` strategy for the given underlying symbol and options parameters.
    /// It sets up a long call position by creating an `Options` object and encapsulating it in a `Position` object,
    /// which includes fees and premiums associated with the long call position.
    ///
    /// # Parameters
    /// - `underlying_symbol`: The symbol of the underlying asset (e.g., a stock ticker symbol) as a `String`.
    /// - `long_call_strike`: The strike price of the long call option, represented as a `Positive` value.
    /// - `long_call_expiration`: The expiration date of the long call option, represented as an `ExpirationDate`.
    /// - `implied_volatility`: The implied volatility of the option, represented as a `Positive` value.
    /// - `quantity`: The quantity of options to include in the position, represented as a `Positive` value.
    /// - `underlying_price`: The current price of the underlying asset, represented as a `Positive` value.
    /// - `risk_free_rate`: The risk-free interest rate used for option pricing, as a `Decimal`.
    /// - `dividend_yield`: The yield of any dividends associated with the underlying asset, as a `Positive` value.
    /// - `premium_long_call`: The premium paid for the long call option, as a `Positive` value.
    /// - `open_fee_long_call`: The fee associated with opening the long call position, as a `Positive` value.
    /// - `close_fee_long_call`: The fee associated with closing the long call position, as a `Positive` value.
    ///
    /// # Returns
    /// An initialized instance of `LongCall` strategy configured with the provided parameters.
    ///
    /// # Panics
    /// - Panics if adding the long call position to the strategy fails.
    ///   This typically occurs if the created long call option is invalid.
    ///
    /// # Notes
    /// - The function relies on creating a default `LongCall` instance and then populating it with positions.
    /// - Uses the `Options` and `Position` structures to model and manage the long call position.
    /// - Assumes the current time (_via `Utc::now()`) when opening the long call position for tracking purposes.
    #[allow(clippy::too_many_arguments, dead_code)]
    fn new(
        underlying_symbol: String,
        long_call_strike: Positive,
        long_call_expiration: ExpirationDate,
        implied_volatility: Positive,
        quantity: Positive,
        underlying_price: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        premium_long_call: Positive,
        open_fee_long_call: Positive,
        close_fee_long_call: Positive,
    ) -> Self {
        let mut strategy = LongCall::default();

        let long_call_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol,
            long_call_strike,
            long_call_expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let long_call = Position::new(
            long_call_option,
            premium_long_call,
            Utc::now(),
            open_fee_long_call,
            close_fee_long_call,
        );
        strategy
            .add_position(&long_call.clone())
            .expect("Invalid long call option");

        strategy
    }
}

impl Positionable for LongCall {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match (position.option.option_style, position.option.side) {
            (OptionStyle::Call, Side::Long) => {
                self.long_call = position.clone();
                Ok(())
            }
            _ => Err(PositionError::invalid_position_style(
                position.option.option_style,
                "Position is a Put or Long, it is not valid for LongCall".to_string(),
            )),
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.long_call])
    }

    /// Gets mutable positions matching the specified criteria from the strategy.
    ///
    /// # Arguments
    /// * `option_style` - The style of the option (Put/Call)
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
            (Side::Long, OptionStyle::Call, strike)
                if *strike == self.long_call.option.strike_price =>
            {
                Ok(vec![&mut self.long_call])
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
            (Side::Long, OptionStyle::Call, strike)
                if *strike == self.long_call.option.strike_price =>
            {
                self.long_call = position.clone();
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

// test_strategy_traits!(LongCall, test_long_call_implementations);
