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
#[derive(Clone, DebugPretty, DisplaySimple, Serialize, Deserialize)]
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
            None,
            None,
        );
        strategy
            .add_position(&long_put.clone())
            .expect("Invalid long put option");

        strategy
    }
}

impl BasicAble for LongPut {
    fn get_title(&self) -> String {
        let strategy_title = format!("{:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.long_put.get_title()]
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
        let long_put = &self.long_put.option;

        hash_set.insert(OptionBasicType {
            option_style: &long_put.option_style,
            side: &long_put.side,
            strike_price: &long_put.strike_price,
            expiration_date: &long_put.expiration_date,
        });

        hash_set
    }
    fn get_implied_volatility(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let options = [(
            &self.long_put.option,
            &self.long_put.option.implied_volatility,
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
        let options = [(&self.long_put.option, &self.long_put.option.quantity)];

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
        self.long_put.one_option()
    }
    fn one_option_mut(&mut self) -> &mut Options {
        self.long_put.one_option_mut()
    }
    fn set_expiration_date(
        &mut self,
        expiration_date: ExpirationDate,
    ) -> Result<(), StrategyError> {
        self.long_put.option.expiration_date = expiration_date;
        Ok(())
    }
    fn set_underlying_price(&mut self, price: &Positive) -> Result<(), StrategyError> {
        self.long_put.option.underlying_price = *price;
        self.long_put.premium =
            Positive::from(self.long_put.option.calculate_price_black_scholes()?.abs());
        Ok(())
    }
    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        self.long_put.option.implied_volatility = *volatility;
        self.long_put.premium =
            Positive(self.long_put.option.calculate_price_black_scholes()?.abs());
        Ok(())
    }
}

impl Validable for LongPut {
    fn validate(&self) -> bool {
        if !self.long_put.validate() {
            debug!("Long call is invalid");
            return false;
        }
        true
    }
}

impl BreakEvenable for LongPut {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        self.break_even_points.push(
            (self.long_put.option.strike_price
                + self.get_net_cost()? / self.long_put.option.quantity)
                .round_to(2),
        );

        Ok(())
    }
}

impl Strategies for LongPut {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        let profit = self.calculate_profit_at(&self.long_put.option.strike_price)?;
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
        let loss = self.calculate_profit_at(&self.long_put.option.strike_price)?;
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
        let base = self.long_put.option.strike_price - self.break_even_points[0];
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

impl Profit for LongPut {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(price);
        self.long_put.pnl_at_expiration(&price)
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

impl StrategyConstructor for LongPut {
    fn get_strategy(_vec_positions: &[Position]) -> Result<Self, StrategyError> {
        todo!()
    }
}

impl Optimizable for LongPut {
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

impl ProbabilityAnalysis for LongPut {
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

impl Greeks for LongPut {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        todo!()
    }
}

impl DeltaNeutrality for LongPut {}

impl PnLCalculator for LongPut {
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

impl Strategable for LongPut {}

test_strategy_traits!(LongPut, test_long_put_implementations);
