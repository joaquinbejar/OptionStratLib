use super::base::{BreakEvenable, Positionable, StrategyType};
use crate::chains::OptionChain;
use crate::error::strategies::ProfitLossErrorKind;
use crate::error::{
    GreeksError, ProbabilityError, StrategyError,
    position::{PositionError, PositionValidationErrorKind},
};
use crate::greeks::Greeks;
use crate::model::{
    ProfitLossRange,
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
use crate::strategies::{
    BasicAble, DeltaAdjustment, FindOptimalSide, Strategable, Strategies, StrategyConstructor,
    Validable,
};
use crate::{ExpirationDate, Options, Positive, test_strategy_traits};
use chrono::Utc;
use pretty_simple_display::{DebugPretty, DisplaySimple};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use tracing::debug;

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
#[derive(Clone, DebugPretty, DisplaySimple, Serialize, Deserialize)]
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
            None,
            None,
        );
        strategy
            .add_position(&short_call.clone())
            .expect("Invalid short call option");

        strategy
    }
}

impl BasicAble for ShortCall {
    fn get_title(&self) -> String {
        let strategy_title = format!("{:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.short_call.get_title()]
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

        hash_set.insert(OptionBasicType {
            option_style: &short_call.option_style,
            side: &short_call.side,
            strike_price: &short_call.strike_price,
            expiration_date: &short_call.expiration_date,
        });

        hash_set
    }
    fn get_implied_volatility(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let options = [(
            &self.short_call.option,
            &self.short_call.option.implied_volatility,
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
    fn get_quantity(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let options = [(&self.short_call.option, &self.short_call.option.quantity)];

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
        Ok(())
    }
    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        self.short_call.option.implied_volatility = *volatility;
        self.short_call.premium = Positive(
            self.short_call
                .option
                .calculate_price_black_scholes()?
                .abs(),
        );
        Ok(())
    }
}

impl Validable for ShortCall {
    fn validate(&self) -> bool {
        if !self.short_call.validate() {
            debug!("Long call is invalid");
            return false;
        }
        true
    }
}

impl BreakEvenable for ShortCall {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        // For a short call, net_cost() from Position returns (fees - premium_received).
        // Break-even = strike + (premium_received - fees) / quantity
        // So, break-even = strike - (fees - premium_received) / quantity
        // Which is strike - (net_cost_from_position / quantity)
        self.break_even_points.push(
            (self.short_call.option.strike_price
                - self.short_call.net_cost()? / self.short_call.option.quantity)
                .round_to(2),
        );

        Ok(())
    }
}

impl Strategies for ShortCall {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        let profit = self.calculate_profit_at(&self.short_call.option.strike_price)?;
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
        // Max loss for a short call is theoretically unlimited.
        Err(StrategyError::ProfitLossError(
            ProfitLossErrorKind::MaxLossError {
                reason: "Maximum loss is unlimited for a short call.".to_string(),
            },
        ))
    }
    fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
        let high = self.get_max_profit().unwrap_or(Positive::ZERO);
        let base = self.short_call.option.strike_price - self.break_even_points[0];
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

impl Profit for ShortCall {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(price);
        self.short_call.pnl_at_expiration(&price)
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

impl StrategyConstructor for ShortCall {
    fn get_strategy(_vec_positions: &[Position]) -> Result<Self, StrategyError> {
        todo!()
    }
}

impl Optimizable for ShortCall {
    type Strategy = Self;

    fn find_optimal(
        &mut self,
        _option_chain: &OptionChain,
        _side: FindOptimalSide,
        _criteria: OptimizationCriteria,
    ) {
        todo!()
    }
}

impl ProbabilityAnalysis for ShortCall {
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

impl Greeks for ShortCall {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        todo!()
    }
}

impl DeltaNeutrality for ShortCall {}

impl PnLCalculator for ShortCall {
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

impl Strategable for ShortCall {}

test_strategy_traits!(ShortCall, test_short_call_implementations);
