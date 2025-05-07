use crate::error::PositionError;
use crate::error::position::PositionValidationErrorKind;
use crate::model::Position;
use crate::strategies::base::{Positionable, StrategyType};
use crate::{ExpirationDate, OptionStyle, OptionType, Options, Positive, Side};
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub(super) const SHORT_CALL_DESCRIPTION: &str = "A Short Call (or Naked Call) is an options strategy where the trader sells a call option without owning the underlying stock. \
    This strategy generates immediate income through the premium received but carries unlimited risk if the stock price rises significantly. \
    The breakeven point is the strike price plus the premium received. Short calls are generally used when the trader has a bearish or neutral outlook on the underlying asset.";

/// Represents the details and structure of a Short Call options trading strategy.
///
/// A Short Call strategy involves selling a call option, which gives the buyer
/// the right to purchase the underlying asset at a specific strike price before
/// the expiration date. This strategy is generally employed when the trader
/// expects minimal movement or a decrease in the price of the underlying asset.
///
/// # Fields
///
/// * `name` - A unique name or identifier for this specific instance of the strategy.
/// * `kind` - Specifies that this instance is of the `ShortCall` strategy type.
/// * `description` - A detailed explanation providing more information about the strategy instance.
/// * `break_even_points` - A vector containing the price points where the strategy does not yield
///   any profit or loss. These points are represented as positive values.
/// * `short_call` - Represents the short call position in the strategy, which involves selling
///   a call option to generate premium income.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShortCall {
    /// Name identifier for this specific strategy instance
    pub name: String,
    /// Identifies this as a ShortCall strategy type
    pub kind: StrategyType,
    /// Detailed description of this strategy instance
    pub description: String,
    /// Price points where the strategy neither makes nor loses money
    pub break_even_points: Vec<Positive>,
    /// The short call option
    pub(super) short_call: Position,
}

impl ShortCall {
    /// Creates a new `ShortCall` strategy instance with the specified parameters.
    ///
    /// The `new` function initializes a short call option strategy by creating an associated
    /// option position and adding it to the strategy. This function is marked with
    /// `#[allow(clippy::too_many_arguments)]` because it takes several parameters required to
    /// define the short call options and associated financial metrics.
    ///
    /// # Parameters
    ///
    /// - `underlying_symbol` (`String`): The symbol of the underlying asset for the short call option.
    /// - `short_call_strike` (`Positive`): The strike price of the short call option.
    /// - `short_call_expiration` (`ExpirationDate`): The expiration date of the short call option.
    /// - `implied_volatility` (`Positive`): The implied volatility of the short call option.
    /// - `quantity` (`Positive`): The quantity of contracts for the short call option.
    /// - `underlying_price` (`Positive`): The current price of the underlying asset.
    /// - `risk_free_rate` (`Decimal`): The risk-free interest rate as a percentage.
    /// - `dividend_yield` (`Positive`): The dividend yield of the underlying asset as a percentage.
    /// - `premium_short_call` (`Positive`): Premium received for selling the short call option.
    /// - `open_fee_short_call` (`Positive`): Opening fee for the short call position.
    /// - `close_fee_short_call` (`Positive`): Closing fee for the short call position.
    ///
    /// # Returns
    ///
    /// Returns an initialized `ShortCall` strategy instance. The instance includes the short call
    /// option position with the specified parameters.
    ///
    /// # Panics
    ///
    /// This function will panic if the short call option created using the specified parameters
    /// fails to meet validity requirements during the `add_position` operation.
    ///
    #[allow(clippy::too_many_arguments, dead_code)]
    fn new(
        underlying_symbol: String,
        short_call_strike: Positive,
        short_call_expiration: ExpirationDate,
        implied_volatility: Positive,
        quantity: Positive,
        underlying_price: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        premium_short_call: Positive,
        open_fee_short_call: Positive,
        close_fee_short_call: Positive,
    ) -> Self {
        let mut strategy = ShortCall::default();

        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            short_call_strike,
            short_call_expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let short_call = Position::new(
            short_call_option,
            premium_short_call,
            Utc::now(),
            open_fee_short_call,
            close_fee_short_call,
        );
        strategy
            .add_position(&short_call.clone())
            .expect("Invalid short call option");

        strategy
    }
}

impl Positionable for ShortCall {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match (position.option.option_style, position.option.side) {
            (OptionStyle::Call, Side::Short) => {
                self.short_call = position.clone();
                Ok(())
            }
            _ => Err(PositionError::invalid_position_style(
                position.option.option_style,
                "Position is a Put or Long, it is not valid for ShortCall".to_string(),
            )),
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.short_call])
    }

    /// Gets mutable positions matching the specified criteria from the strategy.
    ///
    /// # Arguments
    /// * `option_style` - The style of the option (Put/Call)
    /// * `side` - The side of the position (Long/Short)
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
            (Side::Short, OptionStyle::Call, strike)
                if *strike == self.short_call.option.strike_price =>
            {
                Ok(vec![&mut self.short_call])
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
            (Side::Short, OptionStyle::Call, strike)
                if *strike == self.short_call.option.strike_price =>
            {
                self.short_call = position.clone();
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

// test_strategy_traits!(ShortCall, test_short_call_implementations);
