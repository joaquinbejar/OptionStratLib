use super::base::{BreakEvenable, Positionable, StrategyType};
use crate::chains::OptionChain;
use crate::error::{
    position::{PositionError, PositionValidationErrorKind},
    GreeksError,
    ProbabilityError,
    StrategyError,
};
use crate::error::strategies::ProfitLossErrorKind;
use crate::greeks::Greeks;
use crate::model::{ProfitLossRange,
    position::Position,
    types::{OptionBasicType, OptionStyle, OptionType, Side},
};
use crate::pnl::{PnL, PnLCalculator};
use crate::pricing::payoff::Profit;
use crate::strategies::base::Optimizable;
use crate::strategies::delta_neutral::DeltaNeutrality;
use crate::strategies::probabilities::core::ProbabilityAnalysis;
use crate::strategies::probabilities::utils::{PriceTrend, VolatilityAdjustment};
use crate::strategies::utils::OptimizationCriteria;
use crate::strategies::{BasicAble, DeltaAdjustment, Strategable, Strategies, StrategyConstructor, Validable};
use crate::{ExpirationDate, Options, Positive, test_strategy_traits};
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use tracing::debug;

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
            None,
            None,
        );
        strategy
            .add_position(&long_call.clone())
            .expect("Invalid long call option");

        strategy
    }
}

impl BasicAble for LongCall {
    fn get_title(&self) -> String {
        let strategy_title = format!("{:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.long_call.get_title()]
            .iter()
            .map(|leg| leg.to_string())
            .collect();

        if leg_titles.is_empty() {
            strategy_title
        } else {
            format!("{}\n\t{}", strategy_title, leg_titles.join("\n\t"))
        }
    }
    fn get_option_basic_type(&self) -> HashSet<OptionBasicType> {
        let mut hash_set = HashSet::new();
        let long_call = &self.long_call.option;

        hash_set.insert(OptionBasicType {
            option_style: &long_call.option_style,
            side: &long_call.side,
            strike_price: &long_call.strike_price,
            expiration_date: &long_call.expiration_date,
        });

        hash_set
    }
    fn get_implied_volatility(&self) -> HashMap<OptionBasicType, &Positive> {
        let options = [(
            &self.long_call.option,
            &self.long_call.option.implied_volatility,
        )];

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
    fn get_quantity(&self) -> HashMap<OptionBasicType, &Positive> {
        let options = [(&self.long_call.option, &self.long_call.option.quantity)];

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
        self.long_call.one_option()
    }
    fn one_option_mut(&mut self) -> &mut Options {
        self.long_call.one_option_mut()
    }
    fn set_expiration_date(
        &mut self,
        expiration_date: ExpirationDate,
    ) -> Result<(), StrategyError> {
        self.long_call.option.expiration_date = expiration_date;
        Ok(())
    }
    fn set_underlying_price(&mut self, price: &Positive) -> Result<(), StrategyError> {
        self.long_call.option.underlying_price = *price;
        self.long_call.premium =
            Positive::from(self.long_call.option.calculate_price_black_scholes()?.abs());
        Ok(())
    }
    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        self.long_call.option.implied_volatility = *volatility;
        self.long_call.premium =
            Positive(self.long_call.option.calculate_price_black_scholes()?.abs());
        Ok(())
    }
}

impl Validable for LongCall {
    fn validate(&self) -> bool {
        if !self.long_call.validate() {
            debug!("Long call is invalid");
            return false;
        }
        true
    }
}

impl BreakEvenable for LongCall {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        self.break_even_points.push(
            (self.long_call.option.strike_price
                + self.get_net_cost()? / self.long_call.option.quantity)
                .round_to(2),
        );

        Ok(())
    }
}

impl Strategies for LongCall {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        let profit = self.calculate_profit_at(&self.long_call.option.strike_price)?;
        if profit >= Decimal::ZERO {
            Ok(profit.into())
        } else {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "Net premium received is negative".to_string(),
                },
            ))
        }
    }
    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        let loss = self.calculate_profit_at(&self.long_call.option.strike_price)?;
        if loss <= Decimal::ZERO {
            Ok(loss.abs().into())
        } else {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxLossError {
                    reason: "Max loss is negative".to_string(),
                },
            ))
        }
    }
    fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
        let high = self.get_max_profit().unwrap_or(Positive::ZERO);
        let base = self.long_call.option.strike_price - self.break_even_points[0];
        Ok((high * base / 200.0).into())
    }
    fn get_profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let max_profit = self.get_max_profit().unwrap_or(Positive::ZERO);
        let max_loss = self.get_max_loss().unwrap_or(Positive::ZERO);
        match (max_profit, max_loss) {
            (value, _) if value == Positive::ZERO => Ok(Decimal::ZERO),
            (_, value) if value == Positive::ZERO => Ok(Decimal::MAX),
            _ => Ok((max_profit / max_loss * 100.0).into()),
        }
    }
}

impl Profit for LongCall {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(price);
        self.long_call.pnl_at_expiration(&price)
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

impl StrategyConstructor for LongCall {
    fn get_strategy(_vec_positions: &[Position]) -> Result<Self, StrategyError> {
        todo!()
    }
}

impl Optimizable for LongCall {
    type Strategy = Self;

    fn find_optimal(
        &mut self,
        _option_chain: &OptionChain,
        _side: crate::strategies::FindOptimalSide,
        _criteria: OptimizationCriteria,
    ) {
        todo!()
    }
}

impl ProbabilityAnalysis for LongCall {
    fn expected_value(
        &self,
        _volatility_adj: Option<VolatilityAdjustment>,
        _trend: Option<PriceTrend>,
    ) -> Result<Positive, ProbabilityError> {
        todo!()
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        todo!()
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        todo!()
    }
}

impl Greeks for LongCall {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        todo!()
    }
}

impl DeltaNeutrality for LongCall {}

impl PnLCalculator for LongCall {
    fn calculate_pnl(
        &self,
        _market_price: &Positive,
        _expiration_date: ExpirationDate,
        _implied_volatility: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        todo!()
    }

    fn calculate_pnl_at_expiration(
        &self,
        _underlying_price: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        todo!()
    }

    fn adjustments_pnl(&self, _adjustment: &DeltaAdjustment) -> Result<PnL, Box<dyn Error>> {
        todo!()
    }
}

impl Strategable for LongCall {}


test_strategy_traits!(LongCall, test_long_call_implementations);
