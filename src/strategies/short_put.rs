use super::base::{
    Positionable, StrategyType,
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

pub(super) const SHORT_PUT_DESCRIPTION: &str = "A Short Put (or Naked Put) is an options strategy where the trader sells a put option without holding a short position in the underlying stock. \
    This strategy provides immediate income from the premium collected but includes substantial risk if the stock price falls below the strike price. \
    The breakeven point is the strike price minus the premium received. Short puts are typiputy employed when the trader has a bullish or neutral market outlook.";

/// Represents a Short Put options trading strategy.
///
/// A short put is a neutral to bullish strategy involving the sale of a put option.
/// This strategy generates a credit upfront, with the potential obligation to buy the underlying asset
/// at the strike price if the price falls below it. Below are the details stored in this struct:
///
/// Fields:
/// - `name`: The name identifier for this specific strategy instance. This is used to uniquely recognize
///   and distinguish this instance.
/// - `kind`: A field that identifies this as a ShortPut strategy type. It is of type `StrategyType`,
///   which categorizes different trading strategies.
/// - `description`: A detailed description of this strategy instance. This field allows for additional
///   explanation or metadata about why this strategy is being used or how it functions.
/// - `break_even_points`: A vector of price points (of type `Positive`) where the strategy neither makes
///   nor loses money. These are the threshold price levels that determine profitability.
/// - `short_put`: The short put position associated with this strategy. It is declared private (via
///   `pub(super)`) to restrict its accessibility from other modules, ensuring controlled and encapsulated
///   use.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShortPut {
    /// Name identifier for this specific strategy instance
    pub name: String,
    /// Identifies this as a ShortPut strategy type
    pub kind: StrategyType,
    /// Detailed description of this strategy instance
    pub description: String,
    /// Price points where the strategy neither makes nor loses money
    pub break_even_points: Vec<Positive>,
    /// The short put position
    pub(super) short_put: Position,
}

impl ShortPut {
    /// Creates a new `ShortPut` strategy instance with the given parameters.
    ///
    /// This function constructs a `ShortPut` strategy by initializing a short put position
    /// using the provided parameters and adding it to the strategy's list of positions. The
    /// required inputs are both descriptive and numerical attributes of the short put option,
    /// such as the underlying symbol, strike price, expiration date, volatility, and fees.
    ///
    /// # Parameters
    ///
    /// - `underlying_symbol`: A `String` representing the symbol of the underlying asset.
    /// - `short_put_strike`: A `Positive` value specifying the strike price of the short put option.
    ///   This should always be greater than zero.
    /// - `short_put_expiration`: The `ExpirationDate` when the short put option expires.
    /// - `implied_volatility`: A `Positive` value representing the implied volatility of the option.
    /// - `quantity`: A `Positive` value indicating the number of contracts in the position.
    /// - `underlying_price`: A `Positive` value representing the current price of the underlying asset.
    /// - `risk_free_rate`: A `Decimal` representing the risk-free interest rate, expressed as a decimal.
    /// - `dividend_yield`: A `Positive` value representing the dividend yield of the underlying asset.
    /// - `premium_short_put`: A `Positive` value representing the premium received when selling the put option.
    /// - `open_fee_short_put`: A `Positive` value indicating the opening fee for the short put position.
    /// - `close_fee_short_put`: A `Positive` value indicating the closing fee for the short put position.
    ///
    /// # Returns
    ///
    /// A new instance of `ShortPut` containing the initialized short put position.
    ///
    /// # Panics
    ///
    /// This function will panic if adding the short put position to the strategy fails,
    /// which may happen if the position is deemed invalid.
    ///
    #[allow(clippy::too_many_arguments, dead_code)]
    fn new(
        underlying_symbol: String,
        short_put_strike: Positive,
        short_put_expiration: ExpirationDate,
        implied_volatility: Positive,
        quantity: Positive,
        underlying_price: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        premium_short_put: Positive,
        open_fee_short_put: Positive,
        close_fee_short_put: Positive,
    ) -> Self {
        let mut strategy = ShortPut::default();

        let short_put_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            short_put_strike,
            short_put_expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Put,
            dividend_yield,
            None,
        );
        let short_put = Position::new(
            short_put_option,
            premium_short_put,
            Utc::now(),
            open_fee_short_put,
            close_fee_short_put,
        );
        strategy
            .add_position(&short_put.clone())
            .expect("Invalid short put option");

        strategy
    }
}

impl Positionable for ShortPut {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match (position.option.option_style, position.option.side) {
            (OptionStyle::Put, Side::Short) => {
                self.short_put = position.clone();
                Ok(())
            }
            _ => Err(PositionError::invalid_position_style(
                position.option.option_style,
                "Position is a Put or Long, it is not valid for ShortPut".to_string(),
            )),
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.short_put])
    }

    /// Gets mutable positions matching the specified criteria from the strategy.
    ///
    /// # Arguments
    /// * `option_style` - The style of the option (Put/Put)
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
            (Side::Short, OptionStyle::Put, strike)
                if *strike == self.short_put.option.strike_price =>
            {
                Ok(vec![&mut self.short_put])
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
            (Side::Short, OptionStyle::Put, strike)
                if *strike == self.short_put.option.strike_price =>
            {
                self.short_put = position.clone();
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

// test_strategy_traits!(ShortPut, test_short_put_implementations);
