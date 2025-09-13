/*
Strangle Strategy

A strangle involves simultaneously buying an out-of-the-money call and an out-of-the-money put option with the same expiration date.
This strategy is similar to a straddle but typically has a lower cost and requires a larger price move to become profitable.

Key characteristics:
- Unlimited profit potential
- Lower cost than a straddle
- Requires a larger price move to become profitable
*/
use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, StrategyBasics, StrategyType, Validable,
};
use crate::{
    ExpirationDate, Options, Positive,
    chains::{StrategyLegs, chain::OptionChain, utils::OptionDataGroup},
    constants::ZERO,
    error::{
        GreeksError, OperationErrorKind,
        position::{PositionError, PositionValidationErrorKind},
        probability::ProbabilityError,
        strategies::{ProfitLossErrorKind, StrategyError},
    },
    greeks::Greeks,
    model::{
        ProfitLossRange, Trade, TradeStatusAble,
        position::Position,
        types::{Action, OptionBasicType, OptionStyle, OptionType, Side},
        utils::mean_and_std,
    },
    pnl::{PnLCalculator, utils::PnL},
    pricing::payoff::Profit,
    strategies::{
        BasicAble, DeltaAdjustment, Strategies, StrategyConstructor,
        delta_neutral::DeltaNeutrality,
        probabilities::{core::ProbabilityAnalysis, utils::VolatilityAdjustment},
        utils::{FindOptimalSide, OptimizationCriteria, calculate_price_range},
    },
    test_strategy_traits,
};
use chrono::Utc;
use num_traits::FromPrimitive;
use pretty_simple_display::{DebugPretty, DisplaySimple};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use tracing::{debug, error, trace, warn};

pub(super) const SHORT_STRANGLE_DESCRIPTION: &str = "A short strangle involves selling an out-of-the-money call and an \
out-of-the-money put with the same expiration date. This strategy is used when low volatility \
is expected and the underlying asset's price is anticipated to remain stable.";

/// # Short Strangle Strategy
///
/// A Short Strangle is an options trading strategy that involves simultaneously selling an out-of-the-money call option
/// and an out-of-the-money put option with the same expiration date but different strike prices.
///
/// ## Strategy Overview
/// - **Outlook**: Neutral, expecting low volatility
/// - **Risk**: Unlimited in both directions (if the underlying asset moves significantly)
/// - **Reward**: Limited to the combined premium received from selling both options
/// - **Breakeven Points**: Upper strike + net premium received, Lower strike - net premium received
/// - **Margin Requirements**: Typically high due to undefined risk
///
/// ## Properties
/// The `ShortStrangle` struct encapsulates all components and characteristics of a short strangle options strategy.
///
/// ## Fields
/// * `name`: A descriptive name for this specific strategy instance
/// * `kind`: The type of strategy (ShortStrangle)
/// * `description`: A detailed description of the specific strategy implementation
/// * `break_even_points`: The price points at which the strategy breaks even (no profit/loss)
/// * `short_call`: The short call position component of the strategy
/// * `short_put`: The short put position component of the strategy
///
/// ## Maximum Profit/Loss
/// * **Max Profit**: Limited to the total premium received from selling both options
/// * **Max Loss**: Unlimited. Upside loss = stock price - call strike - premium received.
///   Downside loss = put strike - stock price - premium received.
///
/// ## Ideal Market Conditions
/// Best implemented during periods of high implied volatility that is expected to decrease,
/// and when the underlying asset is anticipated to trade within a narrow price range until expiration.
///
/// ## Example
/// A short strangle might be constructed by selling a call with a strike price of $110
/// and selling a put with a strike price of $90, when the underlying is trading at $100.
/// The premium collected might be $2 for the call and $2 for the put, for a total of $4.
/// Break-even points would be at $86 ($90 - $4) and $114 ($110 + $4).
#[derive(Clone, DebugPretty, DisplaySimple, Serialize, Deserialize)]
pub struct ShortStrangle {
    /// Name identifier for this specific strategy instance
    pub name: String,
    /// Identifies this as a ShortStrangle strategy type
    pub kind: StrategyType,
    /// Detailed description of this particular strategy implementation
    pub description: String,
    /// Price points at which the strategy neither makes nor loses money
    pub break_even_points: Vec<Positive>,
    /// The short call leg of the strategy (typically out-of-the-money)
    pub short_call: Position,
    /// The short put leg of the strategy (typically out-of-the-money)
    pub short_put: Position,
}

impl ShortStrangle {
    /// Creates a new Short Strangle options strategy.
    ///
    /// A Short Strangle is created by selling an out-of-the-money call option and an out-of-the-money put option with
    /// the same expiration date. This strategy is profitable when the underlying asset's price remains stable
    /// between the two strike prices, allowing both options to expire worthless.
    ///
    /// # Parameters
    ///
    /// * `underlying_symbol` - Symbol of the underlying asset (e.g., "AAPL", "SPY")
    /// * `underlying_price` - Current market price of the underlying asset
    /// * `call_strike` - Strike price for the short call option. If set to zero, defaults to 110% of underlying price.
    /// * `put_strike` - Strike price for the short put option. If set to zero, defaults to 90% of underlying price.
    /// * `expiration` - Expiration date for both options
    /// * `implied_volatility` - Current implied volatility for the options
    /// * `risk_free_rate` - Risk-free interest rate as a decimal
    /// * `dividend_yield` - Expected dividend yield of the underlying asset
    /// * `quantity` - Number of contracts to trade
    /// * `premium_short_call` - Premium received for selling the call option
    /// * `premium_short_put` - Premium received for selling the put option
    /// * `open_fee_short_call` - Transaction fee for opening the short call position
    /// * `close_fee_short_call` - Transaction fee for closing the short call position
    /// * `open_fee_short_put` - Transaction fee for opening the short put position
    /// * `close_fee_short_put` - Transaction fee for closing the short put position
    ///
    /// # Returns
    ///
    /// A new `ShortStrangle` instance with both positions initialized and break-even points calculated.
    ///
    /// # Default Values
    ///
    /// - If `call_strike` is zero, it will be set to 110% of the underlying price
    /// - If `put_strike` is zero, it will be set to 90% of the underlying price
    ///
    /// # Break-Even Points
    ///
    /// The strategy has two break-even points:
    /// - Lower break-even: Put strike minus the total premium received per contract
    /// - Upper break-even: Call strike plus the total premium received per contract
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        mut call_strike: Positive,
        mut put_strike: Positive,
        expiration: ExpirationDate,
        call_implied_volatility: Positive,
        put_implied_volatility: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        quantity: Positive,
        premium_short_call: Positive,
        premium_short_put: Positive,
        open_fee_short_call: Positive,
        close_fee_short_call: Positive,
        open_fee_short_put: Positive,
        close_fee_short_put: Positive,
    ) -> Self {
        if call_strike == Positive::ZERO {
            call_strike = underlying_price * 1.1;
        }
        if put_strike == Positive::ZERO {
            put_strike = underlying_price * 0.9;
        }
        let mut strategy = ShortStrangle {
            name: "Short Strangle".to_string(),
            kind: StrategyType::ShortStrangle,
            description: SHORT_STRANGLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::default(),
            short_put: Position::default(),
        };

        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            call_strike,
            expiration,
            call_implied_volatility,
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
            None,
            None,
        );
        strategy
            .add_position(&short_call.clone())
            .expect("Invalid position");

        let short_put_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            put_strike,
            expiration,
            put_implied_volatility,
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
            None,
            None,
        );
        strategy
            .add_position(&short_put.clone())
            .expect("Invalid position");

        strategy
            .update_break_even_points()
            .expect("Unable to update break even points");
        strategy
    }
}

impl StrategyConstructor for ShortStrangle {
    fn get_strategy(vec_positions: &[Position]) -> Result<Self, StrategyError> {
        // Need exactly 2 options for a short strangle
        if vec_positions.len() != 2 {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Short Strangle get_strategy".to_string(),
                    reason: "Must have exactly 2 options".to_string(),
                },
            ));
        }

        // Sort options by option style to identify call and put
        let mut sorted_positions = vec_positions.to_vec();
        sorted_positions.sort_by(|a, b| {
            a.option
                .strike_price
                .partial_cmp(&b.option.strike_price)
                .unwrap()
        });

        let put_position = &sorted_positions[0]; // Put will be first
        let call_position = &sorted_positions[1]; // Call will be second

        // Validate one option is call and other is put
        if call_position.option.option_style != OptionStyle::Call
            || put_position.option.option_style != OptionStyle::Put
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Short Strangle get_strategy".to_string(),
                    reason: "One option must be a call and one must be a put".to_string(),
                },
            ));
        }

        // Validate both options are Short
        if call_position.option.side != Side::Short || put_position.option.side != Side::Short {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Short Strangle get_strategy".to_string(),
                    reason: "Both options must be Short positions".to_string(),
                },
            ));
        }

        // Validate call strike is higher than put strike
        if call_position.option.strike_price <= put_position.option.strike_price {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Short Strangle get_strategy".to_string(),
                    reason: "Call strike must be higher than put strike".to_string(),
                },
            ));
        }

        // Validate expiration dates match
        if call_position.option.expiration_date != put_position.option.expiration_date {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Short Strangle get_strategy".to_string(),
                    reason: "Options must have the same expiration date".to_string(),
                },
            ));
        }

        // Create positions
        let short_call = Position::new(
            call_position.option.clone(),
            call_position.premium,
            Utc::now(),
            call_position.open_fee,
            call_position.close_fee,
            call_position.epic.clone(),
            call_position.extra_fields.clone(),
        );

        let short_put = Position::new(
            put_position.option.clone(),
            put_position.premium,
            Utc::now(),
            put_position.open_fee,
            put_position.close_fee,
            put_position.epic.clone(),
            put_position.extra_fields.clone(),
        );

        // Create strategy
        let mut strategy = ShortStrangle {
            name: "Short Strangle".to_string(),
            kind: StrategyType::ShortStrangle,
            description: SHORT_STRANGLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call,
            short_put,
        };

        // Validate and update break-even points
        strategy.validate();
        strategy.update_break_even_points()?;

        Ok(strategy)
    }
}

impl BreakEvenable for ShortStrangle {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        let total_premium = self.get_net_premium_received()?;

        self.break_even_points.push(
            (self.short_put.option.strike_price
                - (total_premium / self.short_put.option.quantity).to_dec())
            .round_to(2),
        );

        self.break_even_points.push(
            (self.one_option().strike_price + (total_premium / self.one_option().quantity))
                .round_to(2),
        );

        self.break_even_points.sort();
        Ok(())
    }
}

impl Positionable for ShortStrangle {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match (&position.option.option_style, &position.option.side) {
            (OptionStyle::Call, Side::Short) => {
                self.short_call = position.clone();
                Ok(())
            }
            (OptionStyle::Put, Side::Short) => {
                self.short_put = position.clone();
                Ok(())
            }
            _ => Err(PositionError::invalid_position_type(
                position.option.side,
                "Position side is Long, it is not valid for ShortStrangle".to_string(),
            )),
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.short_call, &self.short_put])
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
            (Side::Long, _, _) => Err(PositionError::invalid_position_type(
                *side,
                "Position side is Long, it is not valid for ShortStrangle".to_string(),
            )),
            (Side::Short, OptionStyle::Call, strike)
                if *strike == self.one_option().strike_price =>
            {
                Ok(vec![&mut self.short_call])
            }
            (Side::Short, OptionStyle::Put, strike)
                if *strike == self.short_put.option.strike_price =>
            {
                Ok(vec![&mut self.short_put])
            }
            _ => Err(PositionError::invalid_position_type(
                *side,
                "Strike not found in positions".to_string(),
            )),
        }
    }

    fn get_position_unique(
        &mut self,
        option_style: &OptionStyle,
        side: &Side,
    ) -> Result<&mut Position, PositionError> {
        match (side, option_style) {
            (Side::Long, _) => Err(PositionError::invalid_position_type(
                *side,
                "Position side is Long, it is not valid for ShortStrangle".to_string(),
            )),
            (Side::Short, OptionStyle::Call) => Ok(&mut self.short_call),
            (Side::Short, OptionStyle::Put) => Ok(&mut self.short_put),
        }
    }

    fn get_option_unique(
        &mut self,
        option_style: &OptionStyle,
        side: &Side,
    ) -> Result<&mut Options, PositionError> {
        match (side, option_style) {
            (Side::Long, _) => Err(PositionError::invalid_position_type(
                *side,
                "Option side is Long, it is not valid for ShortStrangle".to_string(),
            )),
            (Side::Short, OptionStyle::Call) => Ok(&mut self.short_call.option),
            (Side::Short, OptionStyle::Put) => Ok(&mut self.short_put.option),
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
            let err_msg = format!("modify_position: Invalid position data: \n{position}");
            return Err(PositionError::ValidationError(
                PositionValidationErrorKind::InvalidPosition { reason: err_msg },
            ));
        }

        if position.option.side == Side::Long {
            return Err(PositionError::invalid_position_type(
                position.option.side,
                "Position side is Long, it is not valid for ShortStrangle".to_string(),
            ));
        }

        if position.option.strike_price != self.one_option().strike_price
            && position.option.strike_price != self.short_put.option.strike_price
        {
            return Err(PositionError::invalid_position_type(
                position.option.side,
                "Strike not found in positions".to_string(),
            ));
        }

        if position.option.option_style == OptionStyle::Call {
            self.short_call = position.clone();
        }

        if position.option.option_style == OptionStyle::Put {
            self.short_put = position.clone();
        }

        self.update_break_even_points()?;

        Ok(())
    }

    fn replace_position(&mut self, position: &Position) -> Result<(), PositionError> {
        if !position.validate() {
            let err_msg = format!("modify_position: Invalid position data: \n{position}");
            return Err(PositionError::ValidationError(
                PositionValidationErrorKind::InvalidPosition { reason: err_msg },
            ));
        }

        if position.option.side == Side::Long {
            return Err(PositionError::invalid_position_type(
                position.option.side,
                "Position side is Long, it is not valid for ShortStrangle".to_string(),
            ));
        }

        if position.option.option_style == OptionStyle::Call {
            self.short_call = position.clone();
        }

        if position.option.option_style == OptionStyle::Put {
            self.short_put = position.clone();
        }

        // Update values passed in new position
        self.set_implied_volatility(&position.option.implied_volatility)?;
        self.set_expiration_date(position.option.expiration_date)?;
        self.set_underlying_price(&position.option.underlying_price)?;
        self.update_break_even_points()?;

        Ok(())
    }
}

impl Strategable for ShortStrangle {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl BasicAble for ShortStrangle {
    fn get_title(&self) -> String {
        let strategy_title = format!("{:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.short_call.get_title(), self.short_put.get_title()]
            .iter()
            .map(|leg| leg.to_string())
            .collect();

        if leg_titles.is_empty() {
            strategy_title
        } else {
            format!("{}\n\t{}", strategy_title, leg_titles.join("\n\t"))
        }
    }
    fn get_option_basic_type(&self) -> HashSet<OptionBasicType<'_>> {
        let mut hash_set = HashSet::new();
        let short_call = &self.short_call.option;
        let short_put = &self.short_put.option;
        hash_set.insert(OptionBasicType {
            option_style: &short_call.option_style,
            side: &short_call.side,
            strike_price: &short_call.strike_price,
            expiration_date: &short_call.expiration_date,
        });
        hash_set.insert(OptionBasicType {
            option_style: &short_put.option_style,
            side: &short_put.side,
            strike_price: &short_put.strike_price,
            expiration_date: &short_put.expiration_date,
        });

        hash_set
    }
    fn get_implied_volatility(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let options = [
            (
                &self.short_call.option,
                &self.short_call.option.implied_volatility,
            ),
            (
                &self.short_put.option,
                &self.short_put.option.implied_volatility,
            ),
        ];

        options
            .into_iter()
            .map(|(option, iv)| {
                (
                    OptionBasicType {
                        option_style: &option.option_style,
                        side: &option.side,
                        strike_price: &option.strike_price,
                        expiration_date: &option.expiration_date,
                    },
                    iv,
                )
            })
            .collect()
    }
    fn get_quantity(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let options = [
            (&self.short_call.option, &self.short_call.option.quantity),
            (&self.short_put.option, &self.short_put.option.quantity),
        ];

        options
            .into_iter()
            .map(|(option, quantity)| {
                (
                    OptionBasicType {
                        option_style: &option.option_style,
                        side: &option.side,
                        strike_price: &option.strike_price,
                        expiration_date: &option.expiration_date,
                    },
                    quantity,
                )
            })
            .collect()
    }
    fn one_option(&self) -> &Options {
        self.short_call.one_option()
    }
    fn one_option_mut(&mut self) -> &mut Options {
        self.short_call.one_option_mut()
    }
    fn set_expiration_date(
        &mut self,
        expiration_date: ExpirationDate,
    ) -> Result<(), StrategyError> {
        self.short_call.option.expiration_date = expiration_date;
        self.short_put.option.expiration_date = expiration_date;
        Ok(())
    }
    fn set_underlying_price(&mut self, price: &Positive) -> Result<(), StrategyError> {
        self.short_call.option.underlying_price = *price;
        self.short_call.premium = Positive::from(
            self.short_call
                .option
                .calculate_price_black_scholes()?
                .abs(),
        );
        self.short_put.option.underlying_price = *price;
        self.short_put.premium =
            Positive::from(self.short_put.option.calculate_price_black_scholes()?.abs());
        Ok(())
    }
    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        warn!("This set the implied volatility for both options in the Short Strangle strategy");
        self.short_call.option.implied_volatility = *volatility;
        self.short_put.option.implied_volatility = *volatility;
        self.short_call.premium = Positive(
            self.short_call
                .option
                .calculate_price_black_scholes()?
                .abs(),
        );
        self.short_put.premium =
            Positive(self.short_put.option.calculate_price_black_scholes()?.abs());
        Ok(())
    }
}

impl Strategies for ShortStrangle {
    fn get_volume(&mut self) -> Result<Positive, StrategyError> {
        let volume = self.short_call.option.quantity + self.short_put.option.quantity;
        Ok(volume)
    }

    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        let max_profit = self.get_net_premium_received().unwrap().to_f64();
        if max_profit < ZERO {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "Max profit is negative".to_string(),
                },
            ))
        } else {
            Ok(max_profit.into())
        }
    }

    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        Ok(Positive::INFINITY)
    }

    fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
        let max_profit = self.get_max_profit().unwrap_or(Positive::ZERO);
        if max_profit == Positive::ZERO {
            return Ok(Decimal::ZERO);
        }
        let strike_diff = self.one_option().strike_price - self.short_put.option.strike_price;
        let inner_square = strike_diff * max_profit;
        let break_even_diff = self.break_even_points[1] - self.break_even_points[0];
        let outer_square = break_even_diff * max_profit;
        let triangles = (outer_square - inner_square) / 2.0;
        let result = ((inner_square + triangles) / self.one_option().underlying_price).to_f64();
        Ok(Decimal::from_f64(result).unwrap())
    }

    fn get_profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let break_even_diff = self.break_even_points[1] - self.break_even_points[0];
        let result = match self.get_max_profit() {
            Ok(max_profit) => max_profit.to_f64() / break_even_diff * 100.0,
            Err(_) => ZERO,
        };
        Ok(Decimal::from_f64(result).unwrap())
    }

    fn get_best_range_to_show(&self, step: Positive) -> Result<Vec<Positive>, StrategyError> {
        let max_profit = self.get_max_profit().unwrap_or(Positive::ZERO);
        let (first_option, last_option) = (self.break_even_points[0], self.break_even_points[1]);
        let start_price = first_option - max_profit.to_dec();
        let end_price = last_option + max_profit;
        Ok(calculate_price_range(start_price, end_price, step))
    }

    fn roll_in(&mut self, position: &Position) -> Result<HashMap<Action, Trade>, StrategyError> {
        match (&position.option.option_style, &position.option.side) {
            (OptionStyle::Call, Side::Short) => {
                if self.one_option().strike_price <= position.option.strike_price {
                    return Err(StrategyError::operation_not_supported(
                        "Trying a Roll-out in a Roll-in operation",
                        &self.name,
                    ));
                } else {
                    if self.one_option().underlying_price != position.option.underlying_price {
                        self.set_underlying_price(&position.option.underlying_price)?;
                    }
                    if self.one_option().implied_volatility != position.option.implied_volatility {
                        self.set_implied_volatility(&position.option.implied_volatility)?;
                    }
                    if self.one_option().expiration_date != position.option.expiration_date {
                        self.set_expiration_date(position.option.expiration_date)?;
                    }
                }
                let mut result: HashMap<Action, Trade> = HashMap::new();

                let close_trade = self.short_call.close();
                result.insert(Action::Sell, close_trade);
                self.replace_position(position)?;
                let open_trade = self.short_call.open();
                result.insert(Action::Buy, open_trade);
                Ok(result)
            }
            (OptionStyle::Put, Side::Short) => {
                if self.short_put.option.strike_price >= position.option.strike_price {
                    return Err(StrategyError::operation_not_supported(
                        "Trying a Roll-out in a Roll-in operation",
                        &self.name,
                    ));
                } else {
                    if self.short_put.option.underlying_price != position.option.underlying_price {
                        self.set_underlying_price(&position.option.underlying_price)?;
                    }
                    if self.short_put.option.implied_volatility
                        != position.option.implied_volatility
                    {
                        self.set_implied_volatility(&position.option.implied_volatility)?;
                    }
                    if self.short_put.option.expiration_date != position.option.expiration_date {
                        self.set_expiration_date(position.option.expiration_date)?;
                    }
                }
                let mut result: HashMap<Action, Trade> = HashMap::new();
                let close_trade = self.short_put.close();
                result.insert(Action::Sell, close_trade);
                self.replace_position(position)?;
                let open_trade = self.short_put.open();
                result.insert(Action::Buy, open_trade);
                Ok(result)
            }
            _ => Err(StrategyError::operation_not_supported(
                "Roll-in is not supported for Long Options this strategy",
                &self.name,
            )),
        }
    }

    fn roll_out(&mut self, position: &Position) -> Result<HashMap<Action, Trade>, StrategyError> {
        match (&position.option.option_style, &position.option.side) {
            (OptionStyle::Call, Side::Short) => {
                if self.one_option().strike_price >= position.option.strike_price {
                    return Err(StrategyError::operation_not_supported(
                        "Trying a Roll-in in a Roll-out operation",
                        &self.name,
                    ));
                } else {
                    if self.one_option().underlying_price != position.option.underlying_price {
                        self.set_underlying_price(&position.option.underlying_price)?;
                    }
                    if self.one_option().implied_volatility != position.option.implied_volatility {
                        self.set_implied_volatility(&position.option.implied_volatility)?;
                    }
                    if self.one_option().expiration_date != position.option.expiration_date {
                        self.set_expiration_date(position.option.expiration_date)?;
                    }
                }
                let mut result: HashMap<Action, Trade> = HashMap::new();
                let close_trade = self.short_call.close();
                result.insert(Action::Sell, close_trade);
                self.replace_position(position)?;
                let open_trade = self.short_call.open();
                result.insert(Action::Buy, open_trade);
                Ok(result)
            }
            (OptionStyle::Put, Side::Short) => {
                if self.short_put.option.strike_price <= position.option.strike_price {
                    return Err(StrategyError::operation_not_supported(
                        "Trying a Roll-in in a Roll-out operation",
                        &self.name,
                    ));
                } else {
                    if self.short_put.option.underlying_price != position.option.underlying_price {
                        self.set_underlying_price(&position.option.underlying_price)?;
                    }
                    if self.short_put.option.implied_volatility
                        != position.option.implied_volatility
                    {
                        self.set_implied_volatility(&position.option.implied_volatility)?;
                    }
                    if self.short_put.option.expiration_date != position.option.expiration_date {
                        self.set_expiration_date(position.option.expiration_date)?;
                    }
                }
                let mut result: HashMap<Action, Trade> = HashMap::new();
                let close_trade = self.short_put.close();
                result.insert(Action::Sell, close_trade);
                self.replace_position(position)?;
                let open_trade = self.short_put.open();
                result.insert(Action::Buy, open_trade);
                Ok(result)
            }
            _ => Err(StrategyError::operation_not_supported(
                "Roll-in is not supported for Long Options this strategy",
                &self.name,
            )),
        }
    }
}

impl Validable for ShortStrangle {
    fn validate(&self) -> bool {
        self.short_call.validate()
            && self.short_put.validate()
            && self.one_option().strike_price > self.short_put.option.strike_price
    }
}

impl Optimizable for ShortStrangle {
    type Strategy = ShortStrangle;

    fn filter_combinations<'a>(
        &'a self,
        option_chain: &'a OptionChain,
        side: FindOptimalSide,
    ) -> impl Iterator<Item = OptionDataGroup<'a>> {
        let underlying_price = self.get_underlying_price();
        let strategy = self.clone();
        option_chain
            .get_double_iter()
            // Filter out invalid combinations based on FindOptimalSide
            .filter(move |(short_put, short_call)| match side {
                FindOptimalSide::Deltable(delta) => {
                    let (_, delta_put) = short_put.current_deltas();
                    let (delta_call, _) = short_call.current_deltas();

                    let is_valid = delta_put.unwrap() >= -delta.to_dec()
                        && delta_call.unwrap() <= delta.to_dec()
                        && delta_put.unwrap().is_sign_negative()
                        && delta_call.unwrap().is_sign_positive()
                        && !delta_call.unwrap().is_zero()
                        && !delta_put.unwrap().is_zero();
                    if !is_valid {
                        trace!(
                            "Not Valid Delta combination: PUT {:?} and CALL {:?}",
                            delta_put, delta_call
                        );
                    }
                    is_valid
                }
                FindOptimalSide::DeltaRange(min, max) => {
                    let (_, delta_put) = short_put.current_deltas();
                    let (delta_call, _) = short_call.current_deltas();
                    let delta_put_positive = delta_put.unwrap().abs();
                    delta_put_positive > min
                        && delta_put_positive < max
                        && delta_call.unwrap() > min
                        && delta_call.unwrap() < max
                }
                FindOptimalSide::Center => {
                    short_put.is_valid_optimal_side(underlying_price, &FindOptimalSide::Lower)
                        && short_call
                            .is_valid_optimal_side(underlying_price, &FindOptimalSide::Upper)
                }
                _ => {
                    short_put.is_valid_optimal_side(underlying_price, &side)
                        && short_call.is_valid_optimal_side(underlying_price, &side)
                }
            })
            .filter(move |(short_put, short_call)| short_put.strike_price < short_call.strike_price)
            // Filter out options with invalid bid/ask prices
            .filter(|(short_put, short_call)| {
                short_put.put_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && short_call.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
            })
            // Filter out options that don't meet strategy constraints
            .filter(move |(short_put, short_call)| {
                let legs = StrategyLegs::TwoLegs {
                    first: short_put,
                    second: short_call,
                };
                trace!("Legs: {:?}", legs);
                let strategy = strategy.create_strategy(option_chain, &legs);
                strategy.validate()
                    && strategy.get_max_profit().is_ok()
                    && strategy.get_max_loss().is_ok()
            })
            // Map to OptionDataGroup
            .map(move |(short_put, short_call)| OptionDataGroup::Two(short_put, short_call))
    }

    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
        // Ensure the strategy got the expiration date from the option chain
        let expiration_date = option_chain.get_expiration();
        if let Some(expiration) = expiration_date {
            let _ = self.set_expiration_date(expiration);
        }

        let mut best_value = Decimal::MIN;
        let strategy_clone = self.clone();
        let mut options_iter = strategy_clone
            .filter_combinations(option_chain, side)
            .peekable();

        // Panic if no options are found
        if options_iter.peek().is_none() {
            panic!("No valid option combinations found for the given criteria");
        }

        for option_data_group in options_iter {
            // Unpack the OptionDataGroup into individual options
            let (short_put, short_call) = match option_data_group {
                OptionDataGroup::Two(first, second) => (first, second),
                _ => panic!("Invalid OptionDataGroup"),
            };

            let legs = StrategyLegs::TwoLegs {
                first: short_put,
                second: short_call,
            };
            let strategy = self.create_strategy(option_chain, &legs);
            // Calculate the current value based on the optimization criteria
            let current_value = match criteria {
                OptimizationCriteria::Ratio => strategy.get_profit_ratio().unwrap(),
                OptimizationCriteria::Area => strategy.get_profit_area().unwrap(),
            };

            if current_value > best_value {
                // Update the best value and replace the current strategy
                debug!("Found better value: {}", current_value);
                best_value = current_value;
                *self = strategy.clone();
            }
        }
    }

    fn are_valid_legs(&self, legs: &StrategyLegs) -> bool {
        let (short_put, short_call) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        short_put.put_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
            && short_call.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
    }

    fn create_strategy(&self, chain: &OptionChain, legs: &StrategyLegs) -> Self::Strategy {
        let (put, call) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };

        if !call.valid_call() {
            panic!("Invalid Call options");
        }

        if !put.valid_put() {
            panic!("Invalid Put options");
        }
        let call_implied_volatility = call.implied_volatility;
        assert!(call_implied_volatility <= Positive::ONE);

        let put_implied_volatility = put.implied_volatility;
        assert!(put_implied_volatility <= Positive::ONE);

        let expiration = if let Some(expiration) = chain.get_expiration() {
            expiration
        } else {
            self.one_option().expiration_date
        };

        ShortStrangle::new(
            chain.symbol.clone(),
            chain.underlying_price,
            call.strike_price,
            put.strike_price,
            expiration,
            call_implied_volatility,
            put_implied_volatility,
            self.one_option().risk_free_rate,
            self.one_option().dividend_yield,
            self.one_option().quantity,
            call.call_bid.unwrap(),
            put.put_bid.unwrap(),
            self.short_call.open_fee,
            self.short_call.close_fee,
            self.short_put.open_fee,
            self.short_put.close_fee,
        )
    }
}

impl Profit for ShortStrangle {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = &Some(price);
        trace!(
            "Price: {:?} Strike: {} Call: {:.2} Strike: {} Put: {:.2} Profit: {:.2}",
            price,
            self.one_option().strike_price,
            self.short_call.pnl_at_expiration(price)?,
            self.short_put.option.strike_price,
            self.short_put.pnl_at_expiration(price)?,
            self.short_call.pnl_at_expiration(price)? + self.short_put.pnl_at_expiration(price)?
        );
        Ok(self.short_call.pnl_at_expiration(price)? + self.short_put.pnl_at_expiration(price)?)
    }
}

impl ProbabilityAnalysis for ShortStrangle {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_points = &self.get_break_even_points()?;
        let option = &self.one_option();
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            option.implied_volatility,
            self.short_put.option.implied_volatility,
        ]);

        let mut profit_range = ProfitLossRange::new(
            Some(break_even_points[0]),
            Some(break_even_points[1]),
            Positive::ZERO,
        )?;

        profit_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;

        Ok(vec![profit_range])
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let option = &self.one_option();
        let break_even_points = self.get_break_even_points()?;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            option.implied_volatility,
            self.short_put.option.implied_volatility,
        ]);

        let mut lower_loss_range =
            ProfitLossRange::new(None, Some(break_even_points[0]), Positive::ZERO)?;

        lower_loss_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;

        let mut upper_loss_range =
            ProfitLossRange::new(Some(break_even_points[1]), None, Positive::ZERO)?;

        upper_loss_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;

        Ok(vec![lower_loss_range, upper_loss_range])
    }
}

impl Greeks for ShortStrangle {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.short_call.option, &self.short_put.option])
    }
}

impl DeltaNeutrality for ShortStrangle {}

impl PnLCalculator for ShortStrangle {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        Ok(self
            .short_call
            .calculate_pnl(market_price, expiration_date, implied_volatility)?
            + self
                .short_put
                .calculate_pnl(market_price, expiration_date, implied_volatility)?)
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        Ok(self
            .short_call
            .calculate_pnl_at_expiration(underlying_price)?
            + self
                .short_put
                .calculate_pnl_at_expiration(underlying_price)?)
    }

    fn adjustments_pnl(&self, adjustment: &DeltaAdjustment) -> Result<PnL, Box<dyn Error>> {
        let pnl = {
            match adjustment {
                DeltaAdjustment::BuyOptions {
                    quantity,
                    strike,
                    option_style,
                    side,
                } => {
                    match (side, option_style) {
                        (Side::Short, OptionStyle::Call) => {
                            let mut position = self.short_call.clone();
                            position.option.side = Side::Short; // Buy the short call
                            position.option.quantity = *quantity;
                            position.option.strike_price = *strike;

                            PnL {
                                realized: None,
                                unrealized: None,
                                initial_costs: position.total_cost().unwrap(),
                                initial_income: position.premium_received().unwrap(),
                                date_time: Utc::now(),
                            }
                        }
                        (Side::Short, OptionStyle::Put) => {
                            let mut position = self.short_put.clone();
                            position.option.side = Side::Short; // Buy the short put
                            position.option.quantity = *quantity;
                            position.option.strike_price = *strike;

                            PnL {
                                realized: None,
                                unrealized: None,
                                initial_costs: position.total_cost().unwrap(),
                                initial_income: position.premium_received().unwrap(),
                                date_time: Utc::now(),
                            }
                        }
                        _ => {
                            error!("Invalid side or option style {} {}", side, option_style);
                            PnL::default()
                        }
                    }
                }
                DeltaAdjustment::SellOptions {
                    quantity,
                    strike,
                    option_style,
                    side,
                } => {
                    match (side, option_style) {
                        (Side::Short, OptionStyle::Call) => {
                            let mut position = self.short_call.clone();
                            position.option.side = Side::Long; // Sell the short call 
                            position.option.quantity = *quantity;
                            position.option.strike_price = *strike;

                            PnL {
                                realized: None,
                                unrealized: None,
                                initial_costs: position.total_cost().unwrap(),
                                initial_income: position.premium_received().unwrap(),
                                date_time: Utc::now(),
                            }
                        }
                        (Side::Short, OptionStyle::Put) => {
                            let mut position = self.short_put.clone();
                            position.option.side = Side::Long; // Sell the short put
                            position.option.quantity = *quantity;
                            position.option.strike_price = *strike;

                            PnL {
                                realized: None,
                                unrealized: None,
                                initial_costs: position.total_cost().unwrap(),
                                initial_income: position.premium_received().unwrap(),
                                date_time: Utc::now(),
                            }
                        }
                        _ => {
                            error!("Invalid side or option style {} {}", side, option_style);
                            PnL::default()
                        }
                    }
                }
                _ => {
                    error!("Invalid adjustment type {}", adjustment);
                    PnL::default()
                }
            }
        };

        Ok(pnl)
    }
}

#[cfg(test)]
mod tests_short_strangle {
    use super::*;
    use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
    use crate::{pos, spos};
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn setup() -> ShortStrangle {
        ShortStrangle::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(155.0),
            pos!(145.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.19),
            pos!(0.22),
            dec!(0.01),
            pos!(0.02),
            pos!(100.0),
            Positive::TWO,
            pos!(1.5),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
        )
    }

    #[test]
    fn test_new() {
        let strategy = setup();
        assert_eq!(strategy.name, "Short Strangle");
        assert_eq!(strategy.kind, StrategyType::ShortStrangle);
        assert_eq!(
            strategy.description,
            "A short strangle involves selling an out-of-the-money call and an \
out-of-the-money put with the same expiration date. This strategy is used when low volatility \
is expected and the underlying asset's price is anticipated to remain stable."
        );
    }

    #[test]
    fn test_get_break_even_points() {
        let strategy = setup();
        assert_eq!(strategy.get_break_even_points().unwrap()[0], 141.9);
    }

    #[test]
    fn test_calculate_profit_at() {
        let strategy = setup();
        let price = 150.0;
        assert_eq!(
            strategy
                .calculate_profit_at(&pos!(price))
                .unwrap()
                .to_f64()
                .unwrap(),
            310.0
        );
    }

    #[test]
    fn test_max_profit() {
        let strategy = setup();
        assert_eq!(
            strategy.get_max_profit().unwrap_or(Positive::ZERO),
            strategy.get_net_premium_received().unwrap().to_f64()
        );
    }

    #[test]
    fn test_max_loss() {
        let strategy = setup();
        assert_eq!(
            strategy.get_max_loss().unwrap_or(Positive::ZERO),
            Positive::INFINITY
        );
    }

    #[test]
    fn test_total_cost() {
        let strategy = setup();
        assert_eq!(strategy.get_total_cost().unwrap(), 40.0);
    }

    #[test]
    fn test_net_premium_received() {
        let strategy = setup();
        assert_eq!(
            strategy.get_net_premium_received().unwrap().to_f64(),
            strategy.short_call.net_premium_received().unwrap()
                + strategy.short_put.net_premium_received().unwrap()
        );
    }

    #[test]
    fn test_fees() {
        let strategy = setup();
        let expected_fees = 40.0;
        assert_eq!(strategy.get_fees().unwrap().to_f64(), expected_fees);
    }

    #[test]
    fn test_area() {
        let strategy = setup();
        assert_eq!(
            strategy.get_profit_area().unwrap().to_f64().unwrap(),
            27.07333333333334
        );
    }

    #[test]
    fn test_add_leg() {
        let mut strategy = setup();
        let original_call = strategy.short_call.clone();
        let original_put = strategy.short_put.clone();

        // Test adding a new call leg
        strategy
            .add_position(&original_call.clone())
            .expect("Invalid position");
        assert_eq!(strategy.short_call, original_call);

        // Test adding a new put leg
        strategy
            .add_position(&original_put.clone())
            .expect("Invalid position");
        assert_eq!(strategy.short_put, original_put);
    }

    #[test]
    fn test_profit_ratio() {
        let strategy = setup();
        let break_even_diff = strategy.break_even_points[1] - strategy.break_even_points[0];
        let expected_ratio =
            strategy.get_max_profit().unwrap_or(Positive::ZERO) / break_even_diff * 100.0;
        assert_relative_eq!(
            strategy.get_profit_ratio().unwrap().to_f64().unwrap(),
            expected_ratio.to_f64(),
            epsilon = 0.0001
        );
    }

    #[test]
    fn test_best_ratio() {
        let mut strategy = setup();
        let option_chain = create_test_option_chain();

        strategy.get_best_ratio(&option_chain, FindOptimalSide::All);
        assert!(strategy.validate());
    }

    #[test]
    fn test_best_area() {
        let mut strategy = setup();
        let option_chain = create_test_option_chain();

        strategy.get_best_area(&option_chain, FindOptimalSide::All);
        assert!(strategy.validate());
    }

    #[test]
    fn test_is_valid_short_option() {
        let strategy = setup();
        let option_chain = create_test_option_chain();
        let option_data = option_chain
            .options
            .iter()
            .find(|option_data| option_data.valid_put())
            .unwrap();
        let min_strike = option_chain.options.first().unwrap().strike_price;
        let max_strike = option_chain.options.last().unwrap().strike_price;

        // Test FindOptimalSide::Lower
        assert!(strategy.is_valid_optimal_option(option_data, &FindOptimalSide::Lower));

        // Test FindOptimalSide::Upper
        assert!(!strategy.is_valid_optimal_option(option_data, &FindOptimalSide::Upper));

        // Test FindOptimalSide::All
        assert!(strategy.is_valid_optimal_option(option_data, &FindOptimalSide::All));

        // Test FindOptimalSide::Range
        assert!(
            strategy.is_valid_optimal_option(
                option_data,
                &FindOptimalSide::Range(min_strike, max_strike)
            )
        );
    }

    #[test]
    fn test_create_strategy() {
        let strategy = setup();
        let chain = create_test_option_chain();

        let call_option = chain
            .options
            .iter()
            .rev()
            .find(|option_data| option_data.valid_call())
            .unwrap();
        let put_option = chain
            .options
            .iter()
            .find(|option_data| option_data.valid_put())
            .unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: put_option,
            second: call_option,
        };

        let new_strategy = strategy.create_strategy(&chain, &legs);
        assert!(new_strategy.validate());
    }
    fn create_test_option_chain() -> OptionChain {
        let option_data_price_params = OptionDataPriceParams::new(
            Some(Box::new(pos!(100.0))),
            Some(ExpirationDate::Days(pos!(30.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            Some("AAPL".to_string()),
        );
        let option_chain_build_params = OptionChainBuildParams::new(
            "AAPL".to_string(),
            spos!(1.0),
            10,
            spos!(10.0),
            dec!(-0.2),
            dec!(0.1),
            pos!(0.01),
            2,
            option_data_price_params,
            pos!(0.2),
        );
        OptionChain::build_chain(&option_chain_build_params)
    }
}

#[cfg(test)]
mod tests_short_strangle_probability {
    use super::*;
    use crate::model::ExpirationDate;
    use crate::pos;
    use crate::strategies::probabilities::utils::PriceTrend;
    use rust_decimal_macros::dec;

    /// Helper function that creates a basic short strangle for testing purposes
    /// Returns a ShortStrangle instance with predefined test values
    fn create_test() -> ShortStrangle {
        ShortStrangle::new(
            "TEST".to_string(),
            pos!(100.0),                      // underlying_price
            pos!(110.0),                      // call_strike
            pos!(90.0),                       // put_strike
            ExpirationDate::Days(pos!(30.0)), // expiration
            pos!(0.19),                       // call_implied_volatility
            pos!(0.21),                       // put_implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(1.0),                        // quantity
            Positive::TWO,                    // premium_short_call
            Positive::TWO,                    // premium_short_put
            Positive::ZERO,                   // open_fee_short_call
            Positive::ZERO,                   // close_fee_short_call
            Positive::ZERO,                   // open_fee_short_put
            Positive::ZERO,                   // close_fee_short_put
        )
    }

    #[test]
    fn test_probability_of_profit_basic() {
        let strangle = create_test();
        let result = strangle.probability_of_profit(None, None);

        assert!(result.is_ok(), "Probability calculation should succeed");
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO, "Probability should be positive");
        assert!(prob <= pos!(1.0), "Probability should not exceed 1.0");
    }

    #[test]
    fn test_probability_of_profit_with_volatility_adjustment() {
        let strangle = create_test();
        let vol_adj = VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.05),
        };

        let result = strangle.probability_of_profit(Some(vol_adj), None);

        assert!(
            result.is_ok(),
            "Probability calculation with volatility adjustment should succeed"
        );
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO, "Probability should be positive");
        assert!(prob <= pos!(1.0), "Probability should not exceed 1.0");
    }

    #[test]
    fn test_probability_of_profit_with_trend() {
        let strangle = create_test();
        let trend = PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        };

        let result = strangle.probability_of_profit(None, Some(trend));

        assert!(
            result.is_ok(),
            "Probability calculation with trend should succeed"
        );
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO, "Probability should be positive");
        assert!(prob <= pos!(1.0), "Probability should not exceed 1.0");
    }

    #[test]
    fn test_probability_of_profit_with_downward_trend() {
        let strangle = create_test();
        let trend = PriceTrend {
            drift_rate: -0.1,
            confidence: 0.90,
        };

        let result = strangle.probability_of_profit(None, Some(trend));

        assert!(
            result.is_ok(),
            "Probability calculation with downward trend should succeed"
        );
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO, "Probability should be positive");
        assert!(prob <= pos!(1.0), "Probability should not exceed 1.0");
    }

    #[test]
    fn test_get_reference_price() {
        let strangle = create_test();
        let result = strangle.get_underlying_price();

        assert_eq!(
            *result,
            pos!(100.0),
            "Reference price should match underlying price"
        );
    }

    #[test]
    fn test_get_expiration() {
        let strangle = create_test();
        let expiration_date = *strangle.get_expiration().values().next().unwrap();
        assert_eq!(expiration_date, &ExpirationDate::Days(pos!(30.0)));
    }

    #[test]
    fn test_get_profit_ranges() {
        let strangle = create_test();
        let result = strangle.get_profit_ranges();

        assert!(result.is_ok(), "Profit ranges calculation should succeed");
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1, "Should have exactly one profit range");

        let range = &ranges[0];
        assert!(range.lower_bound.is_some(), "Lower bound should be defined");
        assert!(range.upper_bound.is_some(), "Upper bound should be defined");
        assert!(
            range.probability > Positive::ZERO,
            "Probability should be positive"
        );
    }
}

#[cfg(test)]
mod tests_short_strangle_probability_bis {
    use super::*;
    use crate::model::ExpirationDate;
    use crate::pos;
    use crate::strategies::probabilities::utils::PriceTrend;
    use rust_decimal_macros::dec;

    fn create_test() -> ShortStrangle {
        ShortStrangle::new(
            "TEST".to_string(),
            pos!(100.0),                      // underlying_price
            pos!(110.0),                      // call_strike
            pos!(90.0),                       // put_strike
            ExpirationDate::Days(pos!(30.0)), // expiration
            pos!(0.19),                       // call_implied_volatility
            pos!(0.21),                       // put_implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(1.0),                        // quantity
            Positive::TWO,                    // premium_short_call
            Positive::TWO,                    // premium_short_put
            Positive::ZERO,                   // open_fee_short_call
            Positive::ZERO,                   // close_fee_short_call
            Positive::ZERO,                   // open_fee_short_put
            Positive::ZERO,                   // close_fee_short_put
        )
    }

    #[test]
    fn test_get_risk_free_rate() {
        let strangle = create_test();
        let risk_free_rate = **strangle.get_risk_free_rate().values().next().unwrap();
        assert_eq!(risk_free_rate, dec!(0.05));
    }

    #[test]
    fn test_get_profit_ranges() {
        let strangle = create_test();
        let result = strangle.get_profit_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1); // Short strangle has one profit range

        let range = &ranges[0];
        assert!(range.lower_bound.is_some());
        assert!(range.upper_bound.is_some());
    }

    #[test]
    fn test_get_loss_ranges() {
        let strangle = create_test();
        let result = strangle.get_loss_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 2); // Short strangle has two loss ranges

        // Verify ranges have correct bounds
        assert!(ranges[0].lower_bound.is_none()); // First loss range extends to negative infinity
        assert!(ranges[1].upper_bound.is_none()); // Second loss range extends to positive infinity
    }

    #[test]
    fn test_probability_of_profit() {
        let strangle = create_test();
        let result = strangle.probability_of_profit(None, None);

        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    fn test_probability_with_volatility_adjustment() {
        let strangle = create_test();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.1),
        });

        let result = strangle.probability_of_profit(vol_adj, None);
        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    fn test_probability_with_trend() {
        let strangle = create_test();
        let trend = Some(PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        });

        let result = strangle.probability_of_profit(None, trend);
        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    fn test_analyze_probabilities() {
        let strangle = create_test();
        let result = strangle.analyze_probabilities(None, None);

        assert!(result.is_ok());
        let analysis = result.unwrap();

        assert!(analysis.probability_of_profit > Positive::ZERO);
        assert!(analysis.probability_of_max_profit >= Positive::ZERO);
        assert!(analysis.probability_of_max_loss >= Positive::ZERO);
        assert!(analysis.expected_value > Positive::ZERO);
        assert!(!analysis.break_even_points.is_empty());
        assert!(analysis.risk_reward_ratio > Positive::ZERO);
    }

    #[test]
    fn test_calculate_extreme_probabilities() {
        let strangle = create_test();
        let result = strangle.calculate_extreme_probabilities(None, None);

        assert!(result.is_ok());
        let (max_profit_prob, max_loss_prob) = result.unwrap();
        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }
}

#[cfg(test)]
mod tests_short_strangle_delta {
    use super::*;
    use crate::greeks::Greeks;
    use crate::model::types::OptionStyle;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(call_strike: Positive, put_strike: Positive) -> ShortStrangle {
        let underlying_price = pos!(7138.5);
        ShortStrangle::new(
            "CL".to_string(),
            underlying_price, // underlying_price
            call_strike,      // call_strike 7450 (delta -0.415981)
            put_strike,       // put_strike 7050 (delta 0.417810)
            ExpirationDate::Days(pos!(45.0)),
            pos!(0.19),     // call_implied_volatility
            pos!(0.21),     // put_implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(84.2),     // premium_short_call
            pos!(353.2),    // premium_short_put
            pos!(7.01),     // open_fee_short_call
            pos!(7.01),     // close_fee_short_call
            pos!(7.01),     // open_fee_short_put
            pos!(7.01),     // close_fee_short_put
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        let size = dec!(0.2322829);
        // Calculate the expected adjustment quantity based on the new logic
        // The quantity now represents the adjustment, not the total neutral position
        let delta = pos!(0.433_568_413_729_547_24);
        let k = pos!(7250.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        match &binding[1] {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Put);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let short_call = strategy.short_call.option.clone();
        let mut short_put = strategy.short_put.option.clone();
        short_put.quantity = short_put.quantity - delta.to_dec();
        assert_decimal_eq!(
            short_call.delta().unwrap(),
            -short_put.delta().unwrap(),
            DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_increasing_adjustments() {
        let strike = pos!(7050.0);
        let strategy = get_strategy(pos!(7150.0), strike);
        let net_delta = dec!(-0.1544);
        // Update expected quantities according to the new calculation logic
        let call_qty = pos!(0.285_738_132_542_998_35);
        let put_qty = pos!(0.400_046_741_344_762_76);

        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            net_delta,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        assert_eq!(binding.len(), 3);
        for suggestion in binding.iter() {
            match suggestion {
                DeltaAdjustment::SellOptions {
                    quantity,
                    strike,
                    option_style,
                    side,
                } => {
                    assert_pos_relative_eq!(*quantity, call_qty, Positive(DELTA_THRESHOLD));
                    assert_pos_relative_eq!(*strike, pos!(7150.0), Positive(DELTA_THRESHOLD));
                    assert_eq!(*option_style, OptionStyle::Call);
                    assert_eq!(*side, Side::Short);
                }
                DeltaAdjustment::BuyOptions {
                    quantity,
                    strike,
                    option_style,
                    side,
                } => {
                    assert_pos_relative_eq!(*quantity, put_qty, Positive(DELTA_THRESHOLD));
                    assert_pos_relative_eq!(*strike, pos!(7050.0), Positive(DELTA_THRESHOLD));
                    assert_eq!(*option_style, OptionStyle::Put);
                    assert_eq!(*side, Side::Short);
                }
                DeltaAdjustment::SameSize(adj1, adj2) => {
                    // Get quantities from both adjustments
                    let qty1 = match **adj1 {
                        DeltaAdjustment::BuyOptions { quantity, .. } => quantity,
                        DeltaAdjustment::SellOptions { quantity, .. } => quantity,
                        _ => panic!("Invalid adjustment type in SameSize"),
                    };

                    let qty2 = match **adj2 {
                        DeltaAdjustment::BuyOptions { quantity, .. } => quantity,
                        DeltaAdjustment::SellOptions { quantity, .. } => quantity,
                        _ => panic!("Invalid adjustment type in SameSize"),
                    };

                    // Assert that quantities are equal
                    assert_eq!(
                        qty1, qty2,
                        "Adjustments in SameSize must have equal quantities"
                    );
                }
                _ => panic!("Invalid suggestion"),
            }
        }

        let mut option = strategy.short_put.option.clone();
        option.quantity = put_qty;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -net_delta, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos!(7339.2), pos!(7050.0));

        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
        assert!(strategy.is_delta_neutral());
        let suggestion = strategy.delta_adjustments().unwrap();
        assert_eq!(suggestion[0], DeltaAdjustment::NoAdjustmentNeeded);
    }
}

#[cfg(test)]
mod tests_short_strangle_delta_size {
    use super::*;
    use crate::greeks::Greeks;
    use crate::model::types::{Action, OptionStyle};
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(call_strike: Positive, put_strike: Positive) -> ShortStrangle {
        let underlying_price = pos!(7138.5);
        ShortStrangle::new(
            "CL".to_string(),
            underlying_price, // underlying_price
            call_strike,      // call_strike 7450 (delta -0.415981)
            put_strike,       // put_strike 7050 (delta 0.417810)
            ExpirationDate::Days(pos!(45.0)),
            pos!(0.19),     // call_implied_volatility
            pos!(0.21),     // put_implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(2.0),      // quantity
            pos!(84.2),     // premium_short_call
            pos!(353.2),    // premium_short_put
            pos!(7.01),     // open_fee_short_call
            pos!(7.01),     // close_fee_short_call
            pos!(7.01),     // open_fee_short_put
            pos!(7.01),     // close_fee_short_put
        )
    }

    #[test]
    fn create_test_apply_adjustments() {
        let strategy = get_strategy(pos!(7450.0), pos!(7250.0));

        let initial_delta_neutrality = strategy.delta_neutrality().unwrap();
        let binding = strategy.delta_adjustments().unwrap();
        for suggestion in binding.iter() {
            match suggestion {
                DeltaAdjustment::BuyOptions {
                    quantity,
                    strike,
                    option_style,
                    side,
                } => {
                    assert_pos_relative_eq!(*quantity, pos!(1.530876), Positive(DELTA_THRESHOLD));
                    assert_pos_relative_eq!(*strike, pos!(7450.0), Positive(DELTA_THRESHOLD));
                    assert_eq!(*option_style, OptionStyle::Call);
                    assert_eq!(*side, Side::Short);

                    let mut temp_strategy = strategy.clone();
                    let result = temp_strategy.adjust_option_position(
                        quantity.to_dec(),
                        strike,
                        option_style,
                        side,
                    );
                    assert!(result.is_ok());
                    let final_delta_neutrality = temp_strategy.delta_neutrality().unwrap();
                    assert_decimal_eq!(
                        final_delta_neutrality
                            .individual_deltas
                            .last()
                            .unwrap()
                            .delta_per_contract,
                        initial_delta_neutrality
                            .individual_deltas
                            .last()
                            .unwrap()
                            .delta_per_contract,
                        dec!(0.0001)
                    );
                    assert_decimal_eq!(
                        final_delta_neutrality
                            .individual_deltas
                            .first()
                            .unwrap()
                            .delta_per_contract,
                        initial_delta_neutrality
                            .individual_deltas
                            .first()
                            .unwrap()
                            .delta_per_contract,
                        dec!(0.0001)
                    );

                    assert!(temp_strategy.is_delta_neutral());
                }
                DeltaAdjustment::SellOptions {
                    quantity,
                    strike,
                    option_style,
                    side,
                } => {
                    assert_pos_relative_eq!(
                        *quantity,
                        pos!(0.86713682745),
                        Positive(DELTA_THRESHOLD)
                    );
                    assert_pos_relative_eq!(*strike, pos!(7250.0), Positive(DELTA_THRESHOLD));
                    assert_eq!(*option_style, OptionStyle::Put);
                    assert_eq!(*side, Side::Short);

                    let mut temp_strategy = strategy.clone();
                    let result = temp_strategy.adjust_option_position(
                        -quantity.to_dec(),
                        strike,
                        option_style,
                        side,
                    );
                    assert!(result.is_ok());
                    assert!(temp_strategy.is_delta_neutral());
                }
                DeltaAdjustment::SameSize(first, second) => {
                    let call_short_qty = match **first {
                        DeltaAdjustment::BuyOptions {
                            quantity,
                            strike,
                            option_style,
                            side,
                        } => {
                            assert_pos_relative_eq!(
                                quantity,
                                pos!(0.5535746566012),
                                Positive(DELTA_THRESHOLD)
                            );
                            assert_pos_relative_eq!(
                                strike,
                                pos!(7450.0),
                                Positive(DELTA_THRESHOLD)
                            );
                            assert_eq!(option_style, OptionStyle::Call);
                            assert_eq!(side, Side::Short);
                            quantity
                        }
                        _ => panic!("Invalid first adjustment"),
                    };
                    let put_short_qty = match **second {
                        DeltaAdjustment::SellOptions {
                            quantity,
                            strike,
                            option_style,
                            side,
                        } => {
                            assert_pos_relative_eq!(
                                quantity,
                                pos!(0.553574656),
                                Positive(DELTA_THRESHOLD)
                            );
                            assert_pos_relative_eq!(
                                strike,
                                pos!(7250.0),
                                Positive(DELTA_THRESHOLD)
                            );
                            assert_eq!(option_style, OptionStyle::Put);
                            assert_eq!(side, Side::Short);
                            quantity
                        }
                        _ => {
                            panic!("Invalid suggestion")
                        }
                    };

                    let mut temp_strategy = strategy.clone();
                    let result = temp_strategy.adjust_option_position(
                        call_short_qty.to_dec(),
                        &pos!(7450.0),
                        &OptionStyle::Call,
                        &Side::Short,
                    );
                    assert!(result.is_ok());
                    let result = temp_strategy.adjust_option_position(
                        -put_short_qty.to_dec(),
                        &pos!(7250.0),
                        &OptionStyle::Put,
                        &Side::Short,
                    );
                    assert!(result.is_ok());
                    assert!(temp_strategy.is_delta_neutral());
                }
                _ => panic!("Invalid suggestion"),
            }
        }
    }

    #[test]
    fn create_test_apply_delta_adjustments() {
        let mut strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        let result = strategy.apply_delta_adjustments(Some(Action::Buy));
        assert!(result.is_ok());
        assert!(strategy.is_delta_neutral());

        let mut strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        let result = strategy.apply_delta_adjustments(Some(Action::Sell));
        assert!(result.is_ok());
        assert!(strategy.is_delta_neutral());

        let mut strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        let result = strategy.apply_delta_adjustments(None);
        assert!(result.is_ok());
        assert!(strategy.is_delta_neutral());
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strike = pos!(7450.0);
        let mut strategy = get_strategy(strike, pos!(7250.0));
        let size_call = dec!(0.4645659);
        let delta_call = pos!(1.5308765);
        let k_call = pos!(7450.0);
        let k_put = pos!(7250.0);
        let delta_size = dec!(0.8671368);

        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size_call,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        let first_suggestion = binding.first().unwrap();
        match first_suggestion {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta_call, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k_call, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut short_call = strategy.short_call.option.clone();
        short_call.quantity += delta_call;
        let short_put = strategy.short_put.option.clone();
        assert_decimal_eq!(
            short_call.delta().unwrap(),
            -short_put.delta().unwrap(),
            DELTA_THRESHOLD
        );

        let result =
            strategy.adjust_option_position(-delta_size, &k_put, &OptionStyle::Put, &Side::Short);
        assert!(result.is_ok());
        assert!(strategy.is_delta_neutral());
    }

    #[test]
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos!(7150.0), pos!(7050.0));
        let size = dec!(-0.30886);
        // Update expected quantity according to the new calculation logic
        let delta = pos!(0.80009);
        let k = pos!(7050.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        match &binding[1] {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Put);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_put.option.clone();
        option.quantity = delta;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos!(7343.8), pos!(7045.0));

        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
        assert!(strategy.is_delta_neutral());
        let suggestion = strategy.delta_adjustments().unwrap();
        assert_eq!(suggestion[0], DeltaAdjustment::NoAdjustmentNeeded);
    }
}

#[cfg(test)]
mod tests_adjust_option_position_short {
    use super::*;
    use crate::model::types::{OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;

    // Helper function to create a test strategy
    fn create_test_strategy() -> ShortStrangle {
        ShortStrangle::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(110.0), // call_strike
            pos!(90.0),  // put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.19),     // call_implied_volatility
            pos!(0.21),     // put_implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(2.0),      // premium_short_call
            pos!(2.0),      // premium_short_put
            pos!(0.1),      // open_fee_short_call
            pos!(0.1),      // close_fee_short_call
            pos!(0.1),      // open_fee_short_put
            pos!(0.1),      // close_fee_short_put
        )
    }

    #[test]
    fn test_adjust_existing_call_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_call.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos!(110.0),
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_ok());
        assert_eq!(
            strategy.short_call.option.quantity,
            initial_quantity + adjustment
        );
    }

    #[test]
    fn test_adjust_existing_put_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_put.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos!(90.0),
            &OptionStyle::Put,
            &Side::Short,
        );

        assert!(result.is_ok());
        assert_eq!(
            strategy.short_put.option.quantity,
            initial_quantity + adjustment
        );
    }

    #[test]
    fn test_adjust_nonexistent_position() {
        let mut strategy = create_test_strategy();

        // Try to adjust a non-existent long call position
        let result = strategy.adjust_option_position(
            Decimal::ONE,
            &pos!(110.0),
            &OptionStyle::Call,
            &Side::Long,
        );

        assert!(result.is_err());
        match result.unwrap_err().downcast_ref::<PositionError>() {
            Some(PositionError::ValidationError(
                PositionValidationErrorKind::IncompatibleSide {
                    position_side: _,
                    reason,
                },
            )) => {
                assert_eq!(
                    reason,
                    "Position side is Long, it is not valid for ShortStrangle"
                );
            }
            _ => panic!("Expected PositionError::ValidationError"),
        }
    }

    #[test]
    fn test_adjust_with_invalid_strike() {
        let mut strategy = create_test_strategy();

        // Try to adjust position with wrong strike price
        let result = strategy.adjust_option_position(
            Decimal::ONE,
            &pos!(100.0), // Invalid strike price
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_zero_quantity_adjustment() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_call.option.quantity;

        let result = strategy.adjust_option_position(
            Decimal::ZERO,
            &pos!(110.0),
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_ok());
        assert_eq!(strategy.short_call.option.quantity, initial_quantity);
    }
}

#[cfg(test)]
mod tests_short_strangle_pnl {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn create_test_strangle() -> Result<ShortStrangle, StrategyError> {
        // Create short call position
        let short_call = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(100.0), // Underlying price
            pos!(1.0),   // Quantity
            pos!(105.0), // Strike price
            pos!(0.2),   // Implied volatility
        );

        // Create short put position
        let short_put = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            pos!(100.0), // Same underlying price
            pos!(1.0),   // Quantity
            pos!(95.0),  // Strike price
            pos!(0.2),   // Implied volatility
        );

        ShortStrangle::get_strategy(&vec![short_call, short_put])
    }

    #[test]
    fn test_calculate_pnl_at_money() {
        let strangle = create_test_strangle().unwrap();
        let market_price = pos!(100.0);
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.3);

        let result = strangle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // At the money, both options should have time value but no intrinsic value
        // Initial cost is 2 * fees = 2 * (1.0) = 2.0
        // Initial income is 2 * premium = 2 * 5.0 = 10.0
        assert_pos_relative_eq!(pnl.initial_costs, pos!(2.0), pos!(1e-6));
        assert_pos_relative_eq!(pnl.initial_income, pos!(10.0), pos!(1e-6));
        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(-0.748425), dec!(1e-6));
        // Unrealized loss should be less than max potential loss
        assert!(pnl.unrealized.unwrap() > dec!(-100.0)); // Using a large number as max theoretical loss is unlimited
    }

    #[test]
    fn test_calculate_pnl_above_call_strike() {
        let strangle = create_test_strangle().unwrap();
        let market_price = pos!(110.0); // Above call strike
        let expiration_date = ExpirationDate::Days(pos!(30.0));
        let implied_volatility = pos!(0.2);

        let result = strangle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Call is ITM against us by 5.0 (110 - 105)
        // Put still has some time value
        assert!(pnl.unrealized.unwrap() < dec!(0.0)); // Should be losing money
        assert!(pnl.unrealized.unwrap() > dec!(-10.0)); // But not maximum theoretical loss
    }

    #[test]
    fn test_calculate_pnl_below_put_strike() {
        let strangle = create_test_strangle().unwrap();
        let market_price = pos!(90.0); // Below put strike
        let expiration_date = ExpirationDate::Days(pos!(30.0));
        let implied_volatility = pos!(0.2);

        let result = strangle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Put is ITM against us by 5.0 (95 - 90)
        // Call still has some time value
        assert!(pnl.unrealized.unwrap() < dec!(0.0)); // Should be losing money
        assert!(pnl.unrealized.unwrap() > dec!(-10.0)); // But not maximum theoretical loss
    }

    #[test]
    fn test_calculate_pnl_at_expiration_max_profit() {
        let strangle = create_test_strangle().unwrap();
        let underlying_price = pos!(100.0); // At the money

        let result = strangle.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // At expiration, both options expire worthless
        // Max profit is the total premium received minus fees
        // Premium received = 10.0 (2 * 5.0)
        // Fees = 2.0 (2 * 1.0)
        assert_eq!(pnl.realized.unwrap(), dec!(8.0)); // 10.0 - 2.0
        assert_eq!(pnl.initial_costs, pos!(2.0));
        assert_eq!(pnl.initial_income, pos!(10.0));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_call_loss() {
        let strangle = create_test_strangle().unwrap();
        let underlying_price = pos!(115.0); // Well above call strike

        let result = strangle.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Call loss: -(115 - 105) = -10
        // Put expires worthless
        // Plus initial income (10.0) minus costs (2.0)
        assert_eq!(pnl.realized.unwrap(), dec!(-2.0));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_put_loss() {
        let strangle = create_test_strangle().unwrap();
        let underlying_price = pos!(85.0); // Well below put strike

        let result = strangle.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Put loss: -(95 - 85) = -10
        // Call expires worthless
        // Plus initial income (10.0) minus costs (2.0)
        assert_eq!(pnl.realized.unwrap(), dec!(-2.0));
    }

    #[test]
    fn test_calculate_pnl_with_higher_volatility() {
        let strangle = create_test_strangle().unwrap();
        let market_price = pos!(100.0);
        let expiration_date = ExpirationDate::Days(pos!(30.0));
        let implied_volatility = pos!(0.4); // Higher volatility

        let result = strangle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // With higher volatility, options should be worth more
        // This is bad for short options - should be larger loss than with lower volatility
        assert!(pnl.unrealized.unwrap() < dec!(0.0));
        // But still not at maximum theoretical loss
        assert!(pnl.unrealized.unwrap() > dec!(-100.0));
    }
}

#[cfg(test)]
mod tests_short_strangle_roll {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::{assert_pos_relative_eq, pos};
    use tracing::info;

    fn create_test_strangle() -> Result<ShortStrangle, StrategyError> {
        // Create short call position
        let short_call = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(100.0), // Underlying price
            pos!(1.0),   // Quantity
            pos!(105.0), // Strike price
            pos!(0.2),   // Implied volatility
        );

        // Create short put position
        let short_put = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            pos!(100.0), // Same underlying price
            pos!(1.0),   // Quantity
            pos!(95.0),  // Strike price
            pos!(0.2),   // Implied volatility
        );

        ShortStrangle::get_strategy(&vec![short_call, short_put])
    }

    #[test]
    fn test_calculate_roll_in_call() {
        let mut strangle = create_test_strangle().unwrap();

        info!("strangle: {:?}", strangle);
        let roll_in_short_call = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(98.0),  // Underlying price
            pos!(1.0),   // Quantity
            pos!(103.0), // Strike price
            pos!(0.2),   // Implied volatility
        );

        let result = strangle.roll_in(&roll_in_short_call);
        assert!(result.is_ok(), "{}", result.err().unwrap());
        let trades = result.unwrap();
        info!("strangle: {:?}", strangle);

        assert_pos_relative_eq!(
            strangle.short_call.option.underlying_price,
            pos!(98.0),
            pos!(1e-6)
        );
        assert_pos_relative_eq!(
            strangle.short_put.option.underlying_price,
            pos!(98.0),
            pos!(1e-6)
        );
        assert_pos_relative_eq!(strangle.short_call.option.quantity, pos!(1.0), pos!(1e-6));
        assert_pos_relative_eq!(
            strangle.short_call.option.strike_price,
            pos!(103.0),
            pos!(1e-6)
        );
        assert_pos_relative_eq!(
            strangle.short_call.option.implied_volatility,
            pos!(0.2),
            pos!(1e-6)
        );
        assert_pos_relative_eq!(
            strangle.short_put.option.implied_volatility,
            pos!(0.2),
            pos!(1e-6)
        );

        let open = trades.get(&Action::Buy);
        let close = trades.get(&Action::Sell);
        assert!(open.is_some());
        assert!(close.is_some());
        let open = open.unwrap();
        let close = close.unwrap();
        info!("open: {}", open);
        info!("close: {}", close);
        assert_pos_relative_eq!(open.quantity, pos!(1.0), pos!(1e-6));
        assert_pos_relative_eq!(close.quantity, pos!(1.0), pos!(1e-6));
        assert_pos_relative_eq!(open.underlying_price, pos!(98.0), pos!(1e-6));
        assert_pos_relative_eq!(close.underlying_price, pos!(98.0), pos!(1e-6));
    }

    #[test]
    fn test_calculate_roll_in_call_fail() {
        let mut strangle = create_test_strangle().unwrap();
        let roll_in_short_call = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(98.0),  // Underlying price
            pos!(1.0),   // Quantity
            pos!(107.0), // Strike price
            pos!(0.2),   // Implied volatility
        );
        let result = strangle.roll_in(&roll_in_short_call);
        assert!(result.is_err(), "{}", result.err().unwrap());
    }

    #[test]
    fn test_calculate_roll_in_put() {
        let mut strangle = create_test_strangle().unwrap();
        let roll_in_short_put = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            pos!(102.0), // Underlying price
            pos!(1.0),   // Quantity
            pos!(97.0),  // Strike price
            pos!(0.2),   // Implied volatility
        );

        let result = strangle.roll_in(&roll_in_short_put);
        assert!(result.is_ok(), "{}", result.err().unwrap());
        let trades = result.unwrap();

        assert_pos_relative_eq!(
            strangle.short_call.option.underlying_price,
            pos!(102.0),
            pos!(1e-6)
        );
        assert_pos_relative_eq!(
            strangle.short_put.option.underlying_price,
            pos!(102.0),
            pos!(1e-6)
        );
        assert_pos_relative_eq!(strangle.short_put.option.quantity, pos!(1.0), pos!(1e-6));
        assert_pos_relative_eq!(
            strangle.short_put.option.strike_price,
            pos!(97.0),
            pos!(1e-6)
        );
        assert_pos_relative_eq!(
            strangle.short_put.option.implied_volatility,
            pos!(0.2),
            pos!(1e-6)
        );
        assert_pos_relative_eq!(
            strangle.short_call.option.implied_volatility,
            pos!(0.2),
            pos!(1e-6)
        );

        let open = trades.get(&Action::Buy);
        let close = trades.get(&Action::Sell);
        assert!(open.is_some());
        assert!(close.is_some());
        let open = open.unwrap();
        let close = close.unwrap();
        info!("open: {}", open);
        info!("close: {}", close);
        assert_pos_relative_eq!(open.quantity, pos!(1.0), pos!(1e-6));
        assert_pos_relative_eq!(close.quantity, pos!(1.0), pos!(1e-6));
        assert_pos_relative_eq!(open.underlying_price, pos!(102.0), pos!(1e-6));
        assert_pos_relative_eq!(close.underlying_price, pos!(102.0), pos!(1e-6));
    }

    #[test]
    fn test_calculate_roll_in_put_fail() {
        let mut strangle = create_test_strangle().unwrap();
        let roll_in_short_put = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            pos!(98.0), // Underlying price
            pos!(1.0),  // Quantity
            pos!(93.0), // Strike price
            pos!(0.2),  // Implied volatility
        );
        let result = strangle.roll_in(&roll_in_short_put);
        assert!(result.is_err(), "{}", result.err().unwrap());
    }

    #[test]
    fn test_calculate_roll_out_call() {
        let mut strangle = create_test_strangle().unwrap();

        info!("strangle: {:?}", strangle);
        let roll_out_short_call = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(102.0), // Underlying price
            pos!(1.0),   // Quantity
            pos!(110.0), // Strike price (further OTM than original 105)
            pos!(0.2),   // Implied volatility
        );

        let result = strangle.roll_out(&roll_out_short_call);
        assert!(result.is_ok(), "{}", result.err().unwrap());
        let trades = result.unwrap();
        info!("strangle: {:?}", strangle);

        assert_pos_relative_eq!(
            strangle.short_call.option.underlying_price,
            pos!(102.0),
            pos!(1e-6)
        );
        assert_pos_relative_eq!(
            strangle.short_put.option.underlying_price,
            pos!(102.0),
            pos!(1e-6)
        );
        assert_pos_relative_eq!(strangle.short_call.option.quantity, pos!(1.0), pos!(1e-6));
        assert_pos_relative_eq!(
            strangle.short_call.option.strike_price,
            pos!(110.0),
            pos!(1e-6)
        );
        assert_pos_relative_eq!(
            strangle.short_call.option.implied_volatility,
            pos!(0.2),
            pos!(1e-6)
        );
        assert_pos_relative_eq!(
            strangle.short_put.option.implied_volatility,
            pos!(0.2),
            pos!(1e-6)
        );

        let open = trades.get(&Action::Buy);
        let close = trades.get(&Action::Sell);
        assert!(open.is_some());
        assert!(close.is_some());
        let open = open.unwrap();
        let close = close.unwrap();
        info!("open: {}", open);
        info!("close: {}", close);
        assert_pos_relative_eq!(open.quantity, pos!(1.0), pos!(1e-6));
        assert_pos_relative_eq!(close.quantity, pos!(1.0), pos!(1e-6));
        assert_pos_relative_eq!(open.underlying_price, pos!(102.0), pos!(1e-6));
        assert_pos_relative_eq!(close.underlying_price, pos!(102.0), pos!(1e-6));
    }

    #[test]
    fn test_calculate_roll_out_call_fail() {
        let mut strangle = create_test_strangle().unwrap();
        let roll_out_short_call = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(102.0), // Underlying price
            pos!(1.0),   // Quantity
            pos!(103.0), // Strike price (closer to ATM than original 105)
            pos!(0.2),   // Implied volatility
        );
        let result = strangle.roll_out(&roll_out_short_call);
        assert!(result.is_err(), "{}", result.err().unwrap());
    }

    #[test]
    fn test_calculate_roll_out_put() {
        let mut strangle = create_test_strangle().unwrap();

        info!("strangle: {:?}", strangle);
        let roll_out_short_put = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            pos!(98.0), // Underlying price
            pos!(1.0),  // Quantity
            pos!(90.0), // Strike price (further OTM than original 95)
            pos!(0.2),  // Implied volatility
        );

        let result = strangle.roll_out(&roll_out_short_put);
        assert!(result.is_ok(), "{}", result.err().unwrap());
        let trades = result.unwrap();
        info!("strangle: {:?}", strangle);

        assert_pos_relative_eq!(
            strangle.short_call.option.underlying_price,
            pos!(98.0),
            pos!(1e-6)
        );
        assert_pos_relative_eq!(
            strangle.short_put.option.underlying_price,
            pos!(98.0),
            pos!(1e-6)
        );
        assert_pos_relative_eq!(strangle.short_put.option.quantity, pos!(1.0), pos!(1e-6));
        assert_pos_relative_eq!(
            strangle.short_put.option.strike_price,
            pos!(90.0),
            pos!(1e-6)
        );
        assert_pos_relative_eq!(
            strangle.short_put.option.implied_volatility,
            pos!(0.2),
            pos!(1e-6)
        );
        assert_pos_relative_eq!(
            strangle.short_call.option.implied_volatility,
            pos!(0.2),
            pos!(1e-6)
        );

        let open = trades.get(&Action::Buy);
        let close = trades.get(&Action::Sell);
        assert!(open.is_some());
        assert!(close.is_some());
        let open = open.unwrap();
        let close = close.unwrap();
        info!("open: {}", open);
        info!("close: {}", close);
        assert_pos_relative_eq!(open.quantity, pos!(1.0), pos!(1e-6));
        assert_pos_relative_eq!(close.quantity, pos!(1.0), pos!(1e-6));
        assert_pos_relative_eq!(open.underlying_price, pos!(98.0), pos!(1e-6));
        assert_pos_relative_eq!(close.underlying_price, pos!(98.0), pos!(1e-6));
    }

    #[test]
    fn test_calculate_roll_out_put_fail() {
        let mut strangle = create_test_strangle().unwrap();
        let roll_out_short_put = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            pos!(98.0), // Underlying price
            pos!(1.0),  // Quantity
            pos!(97.0), // Strike price (closer to ATM than original 95)
            pos!(0.2),  // Implied volatility
        );
        let result = strangle.roll_out(&roll_out_short_put);
        assert!(result.is_err(), "{}", result.err().unwrap());
    }
}

#[cfg(test)]
mod tests_strategy_constructor {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::{OptionStyle, Side, pos};

    mod short_strangle_tests {
        use super::*;

        #[test]
        fn test_valid_short_strangle() {
            let options = vec![
                create_sample_position(
                    OptionStyle::Call,
                    Side::Short,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(110.0),
                    pos!(0.2),
                ),
                create_sample_position(
                    OptionStyle::Put,
                    Side::Short,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(90.0),
                    pos!(0.2),
                ),
            ];

            let result = ShortStrangle::get_strategy(&options);
            assert!(result.is_ok());

            let strategy = result.unwrap();
            assert_eq!(strategy.short_call.option.strike_price, pos!(110.0));
            assert_eq!(strategy.short_put.option.strike_price, pos!(90.0));
        }

        #[test]
        fn test_wrong_number_of_options() {
            let options = vec![create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(100.0),
                pos!(1.0),
                pos!(110.0),
                pos!(0.2),
            )];

            let result = ShortStrangle::get_strategy(&options);
            assert!(matches!(
                result,
                Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters { .. }
                ))
            ));
        }

        #[test]
        fn test_wrong_option_styles() {
            let options = vec![
                create_sample_position(
                    OptionStyle::Call,
                    Side::Short,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(110.0),
                    pos!(0.2),
                ),
                create_sample_position(
                    OptionStyle::Call,
                    Side::Short,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(90.0),
                    pos!(0.2),
                ),
            ];

            let result = ShortStrangle::get_strategy(&options);
            assert!(matches!(
                result,
                Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters { .. }
                ))
            ));
        }

        #[test]
        fn test_wrong_sides() {
            let options = vec![
                create_sample_position(
                    OptionStyle::Call,
                    Side::Short,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(110.0),
                    pos!(0.2),
                ),
                create_sample_position(
                    OptionStyle::Put,
                    Side::Long,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(90.0),
                    pos!(0.2),
                ),
            ];

            let result = ShortStrangle::get_strategy(&options);
            assert!(matches!(
                result,
                Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters { .. }
                ))
            ));
        }

        #[test]
        fn test_invalid_strikes() {
            let options = vec![
                create_sample_position(
                    OptionStyle::Call,
                    Side::Short,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(90.0),
                    pos!(0.2),
                ),
                create_sample_position(
                    OptionStyle::Put,
                    Side::Short,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(110.0),
                    pos!(0.2),
                ),
            ];

            let result = ShortStrangle::get_strategy(&options);
            assert!(matches!(
                result,
                Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters { .. }
                ))
            ));
        }
    }
}

#[cfg(test)]
mod test_adjustments_pnl {
    use crate::greeks::Greeks;
    use crate::pnl::PnLCalculator;
    use crate::strategies::{
        BasicAble, DELTA_THRESHOLD, DeltaAdjustment, DeltaNeutrality, ShortStrangle,
    };
    use crate::{
        ExpirationDate, OptionStyle, Positive, Side, assert_decimal_eq, assert_pos_relative_eq, pos,
    };

    use rust_decimal_macros::dec;
    use tracing::info;

    fn get_strategy(call_strike: Positive, put_strike: Positive) -> ShortStrangle {
        let underlying_price = pos!(7138.5);
        ShortStrangle::new(
            "CL".to_string(),
            underlying_price, // underlying_price
            call_strike,      // call_strike 7450 (delta -0.415981)
            put_strike,       // put_strike 7050 (delta 0.417810)
            ExpirationDate::Days(pos!(45.0)),
            pos!(0.19),     // call_implied_volatility
            pos!(0.21),     // put_implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(84.2),     // premium_short_call
            pos!(353.2),    // premium_short_put
            pos!(7.01),     // open_fee_short_call
            pos!(7.01),     // close_fee_short_call
            pos!(7.01),     // open_fee_short_put
            pos!(7.01),     // close_fee_short_put
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let mut strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        info!("short_call: {}", strategy.short_call.premium);
        info!("short_put: {}", strategy.short_put.premium);
        info!(
            "{}",
            strategy.calculate_pnl_at_expiration(&pos!(7138.5)).unwrap()
        );
        let size = dec!(0.232282);
        let delta = pos!(0.4335684137);
        let k = pos!(7250.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        info!("delta: {}", strategy.delta_neutrality().unwrap().net_delta);
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();

        match &binding[1] {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Put);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let pnl = strategy.adjustments_pnl(&binding[1]).unwrap();
        assert!(pnl.realized.is_none());
        assert!(pnl.unrealized.is_none());
        assert_pos_relative_eq!(pnl.initial_costs, pos!(159.2149928), pos!(1e-6));
        assert_pos_relative_eq!(pnl.initial_income, Positive::ZERO, pos!(1e-6));

        let short_call = strategy.short_call.option.clone();
        let mut short_put = strategy.short_put.option.clone();
        short_put.quantity = short_put.quantity - delta.to_dec();
        assert_decimal_eq!(
            short_call.delta().unwrap(),
            -short_put.delta().unwrap(),
            DELTA_THRESHOLD
        );
        let new_underlying_price = pos!(7350.0);
        strategy
            .set_underlying_price(&new_underlying_price)
            .unwrap();
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        info!("binding: {:?}", binding);
        match &binding[1] {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, pos!(0.236188315978), Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, pos!(7250.0), Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Put);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }
        let pnl = strategy.adjustments_pnl(&binding[1]).unwrap();
        assert!(pnl.realized.is_none());
        assert!(pnl.unrealized.is_none());
        assert_pos_relative_eq!(pnl.initial_costs, pos!(3.311360), pos!(1e-6));
        assert_pos_relative_eq!(pnl.initial_income, pos!(35.304050), pos!(1e-6));
    }
}

#[cfg(test)]
mod test_valid_premium_for_shorts {
    use crate::strategies::ShortStrangle;
    use crate::{ExpirationDate, Positive, pos};

    use crate::strategies::base::Positionable;
    use rust_decimal_macros::dec;

    fn get_strategy(call_strike: Positive, put_strike: Positive) -> ShortStrangle {
        let underlying_price = pos!(7138.5);
        ShortStrangle::new(
            "CL".to_string(),
            underlying_price, // underlying_price
            call_strike,      // call_strike 7450 (delta -0.415981)
            put_strike,       // put_strike 7050 (delta 0.417810)
            ExpirationDate::Days(pos!(45.0)),
            pos!(0.19),     // call_implied_volatility
            pos!(0.21),     // put_implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(84.2),     // premium_short_call
            pos!(353.2),    // premium_short_put
            pos!(7.01),     // open_fee_short_call
            pos!(7.01),     // close_fee_short_call
            pos!(7.01),     // open_fee_short_put
            pos!(7.01),     // close_fee_short_put
        )
    }

    #[test]
    fn create_test_strangle() {
        let strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        assert!(strategy.valid_premium_for_shorts(&pos!(10.0)));
        assert!(!strategy.valid_premium_for_shorts(&pos!(100.0)));
        assert!(!strategy.valid_premium_for_shorts(&pos!(400.0)));
    }
}

#[cfg(test)]
mod tests_strangle_position_management {
    use super::*;
    use crate::error::position::PositionValidationErrorKind;
    use crate::model::types::{OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;

    fn create_test_short_strangle() -> ShortStrangle {
        ShortStrangle::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(110.0), // call_strike
            pos!(90.0),  // put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.19),     // call_implied_volatility
            pos!(0.21),     // put_implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(2.0),      // premium_short_call
            pos!(2.0),      // premium_short_put
            pos!(0.1),      // open_fee_short_call
            pos!(0.1),      // close_fee_short_call
            pos!(0.1),      // open_fee_short_put
            pos!(0.1),      // close_fee_short_put
        )
    }

    #[test]
    fn test_short_strangle_get_position() {
        let mut strangle = create_test_short_strangle();

        // Test getting short call position
        let call_position = strangle.get_position(&OptionStyle::Call, &Side::Short, &pos!(110.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(110.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting short put position
        let put_position = strangle.get_position(&OptionStyle::Put, &Side::Short, &pos!(90.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(90.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Put);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting non-existent position
        let invalid_position =
            strangle.get_position(&OptionStyle::Call, &Side::Short, &pos!(100.0));
        assert!(invalid_position.is_err());
        match invalid_position {
            Err(PositionError::ValidationError(
                PositionValidationErrorKind::IncompatibleSide {
                    position_side: _,
                    reason,
                },
            )) => {
                assert_eq!(reason, "Strike not found in positions");
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn test_short_strangle_modify_position() {
        let mut strangle = create_test_short_strangle();

        // Modify short call position
        let mut modified_call = strangle.short_call.clone();
        modified_call.option.quantity = pos!(2.0);
        let result = strangle.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(strangle.short_call.option.quantity, pos!(2.0));

        // Modify short put position
        let mut modified_put = strangle.short_put.clone();
        modified_put.option.quantity = pos!(2.0);
        let result = strangle.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(strangle.short_put.option.quantity, pos!(2.0));

        // Test modifying with invalid position
        let mut invalid_position = strangle.short_call.clone();
        invalid_position.option.strike_price = pos!(95.0);
        let result = strangle.modify_position(&invalid_position);
        assert!(result.is_err());
        match result {
            Err(PositionError::ValidationError(kind)) => match kind {
                PositionValidationErrorKind::IncompatibleSide {
                    position_side: _,
                    reason,
                } => {
                    assert_eq!(reason, "Strike not found in positions");
                }
                _ => panic!("Expected ValidationError::InvalidPosition"),
            },
            _ => panic!("Expected ValidationError"),
        }
    }
}

#[cfg(test)]
mod tests_generate_delta_adjustments {
    use super::*;
    use crate::strategies::DELTA_THRESHOLD;
    use crate::{ExpirationDate, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn create_test_short_strangle() -> ShortStrangle {
        ShortStrangle::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(110.0), // call_strike
            pos!(90.0),  // put_strike
            ExpirationDate::Days(pos!(45.0)),
            pos!(0.32),     // call_implied_volatility
            pos!(0.35),     // put_implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(2.0),      // premium_short_call
            pos!(2.0),      // premium_short_put
            pos!(0.1),      // open_fee_short_call
            pos!(0.1),      // close_fee_short_call
            pos!(0.1),      // open_fee_short_put
            pos!(0.1),      // close_fee_short_put
        )
    }

    #[test]
    fn test_generate_delta_adjustments_positive_net_delta_positive_option_delta() {
        let mut strategy = create_test_short_strangle();
        let mut position: Position = strategy.short_put.clone();
        position.option.quantity = pos!(3.0);

        strategy.modify_position(&position).unwrap();
        let delta_neutral = strategy.delta_neutrality().unwrap();
        assert!(delta_neutral.net_delta.is_sign_positive());
        let net_delta = delta_neutral.net_delta;

        let option = position.option;
        let option_delta_per_contract = option.delta().unwrap() / option.quantity;
        assert!(option_delta_per_contract.is_sign_positive());

        // Act
        let adjustment = strategy
            .generate_delta_adjustments(net_delta, option_delta_per_contract, &option)
            .unwrap();

        assert_eq!(option.quantity, pos!(3.0));
        assert_eq!(strategy.short_call.option.quantity, pos!(1.0));

        // Assert
        match adjustment {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(quantity, pos!(1.6138759), Positive(DELTA_THRESHOLD));
                assert_eq!(strike, option.strike_price);
                assert_eq!(option_style, option.option_style);
                assert_eq!(side, option.side);
            }
            _ => panic!("Expected SellOptions adjustment"),
        }
    }

    #[test]
    fn test_generate_delta_adjustments_positive_net_delta_negative_option_delta() {
        let mut strategy = create_test_short_strangle();
        let mut position: Position = strategy.short_put.clone();
        position.option.quantity = pos!(3.0);

        strategy.modify_position(&position).unwrap();
        let delta_neutral = strategy.delta_neutrality().unwrap();
        assert!(delta_neutral.net_delta.is_sign_positive());
        let net_delta = delta_neutral.net_delta;

        let option = strategy.short_call.option.clone();
        let option_delta_per_contract = option.delta().unwrap() / option.quantity;
        assert!(option_delta_per_contract.is_sign_negative());

        // Act
        let adjustment = strategy
            .generate_delta_adjustments(net_delta, option_delta_per_contract, &option)
            .unwrap();

        assert_eq!(option.quantity, pos!(1.0));
        assert_eq!(strategy.short_put.option.quantity, pos!(3.0));

        // Assert
        match adjustment {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(quantity, pos!(1.1643083), Positive(DELTA_THRESHOLD));
                assert_eq!(strike, option.strike_price);
                assert_eq!(option_style, option.option_style);
                assert_eq!(side, option.side);
            }
            _ => panic!("Expected SellOptions adjustment"),
        }
    }

    #[test]
    fn test_generate_delta_adjustments_negative_net_delta_positive_option_delta() {
        let strategy = create_test_short_strangle();
        let delta_neutral = strategy.delta_neutrality().unwrap();
        assert!(delta_neutral.net_delta.is_sign_negative());
        let net_delta = delta_neutral.net_delta;

        let option = strategy.short_put.option.clone();
        let option_delta_per_contract = option.delta().unwrap() / option.quantity;
        assert!(option_delta_per_contract.is_sign_positive());

        // Act
        let adjustment = strategy
            .generate_delta_adjustments(net_delta, option_delta_per_contract, &option)
            .unwrap();

        assert_eq!(option.quantity, pos!(1.0));
        assert_eq!(strategy.short_call.option.quantity, pos!(1.0));

        // Assert
        match adjustment {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(quantity, pos!(0.3861240), Positive(DELTA_THRESHOLD));
                assert_eq!(strike, option.strike_price);
                assert_eq!(option_style, option.option_style);
                assert_eq!(side, option.side);
            }
            _ => panic!("Expected SellOptions adjustment"),
        }
    }

    #[test]
    fn test_generate_delta_adjustments_negative_net_delta_negarive_option_delta() {
        let strategy = create_test_short_strangle();
        let delta_neutral = strategy.delta_neutrality().unwrap();
        assert!(delta_neutral.net_delta.is_sign_negative());
        let net_delta = delta_neutral.net_delta;

        let option = strategy.short_call.option.clone();
        let option_delta_per_contract = option.delta().unwrap() / option.quantity;
        assert!(option_delta_per_contract.is_sign_negative());

        // Act
        let adjustment = strategy
            .generate_delta_adjustments(net_delta, option_delta_per_contract, &option)
            .unwrap();

        assert_eq!(option.quantity, pos!(1.0));
        assert_eq!(strategy.short_call.option.quantity, pos!(1.0));

        // Assert
        match adjustment {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(quantity, pos!(0.2785638), Positive(DELTA_THRESHOLD));
                assert_eq!(strike, option.strike_price);
                assert_eq!(option_style, option.option_style);
                assert_eq!(side, option.side);
            }
            _ => panic!("Expected SellOptions adjustment"),
        }
    }
}

test_strategy_traits!(ShortStrangle, test_short_strangle_implementations);
