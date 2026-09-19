// Scoped allow: bulk migration of unchecked `[]` indexing to
// `.get().ok_or_else(..)` tracked as follow-ups to #341. The existing
// call sites are internal to this file and audited for invariant-bound
// indices (fixed-length buffers, just-pushed slices, etc.).
#![allow(clippy::indexing_slicing)]

use super::base::{BreakEvenable, Positionable, StrategyType};
use crate::analytics::probability::VolatilityAdjustment;
use crate::chains::OptionChain;
use crate::error::{
    GreeksError, PricingError, ProbabilityError, StrategyError,
    position::{PositionError, PositionValidationErrorKind},
    probability::ProfitLossRangeErrorKind,
};
use crate::greeks::Greeks;
use crate::model::decimal::d_div;
use crate::model::{
    ProfitLossRange,
    position::Position,
    types::{OptionBasicType, OptionStyle, OptionType, Side},
};
use crate::pnl::{PnL, PnLCalculator};
use crate::pricing::payoff::Profit;
use crate::strategies::base::Optimizable;
use crate::strategies::base::lower_break_even;
use crate::strategies::base::price_gap;
use crate::strategies::delta_neutral::DeltaNeutrality;
use crate::strategies::probabilities::core::ProbabilityAnalysis;
use crate::strategies::utils::OptimizationCriteria;
use crate::strategies::{
    BasicAble, DeltaAdjustment, Strategable, Strategies, StrategyConstructor, Validable,
};
use crate::{ExpirationDate, Options};
use chrono::Utc;
use num_traits::FromPrimitive;
use positive::Positive;
use pretty_simple_display::{DebugPretty, DisplaySimple};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, warn};
use utoipa::ToSchema;

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
#[derive(DebugPretty, DisplaySimple, Clone, Serialize, Deserialize, ToSchema)]
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
    /// # Errors
    /// Returns `StrategyError` if the freshly-constructed long call leg
    /// cannot be added to the strategy. In practice this branch is
    /// unreachable for a freshly-built single-leg strategy and is surfaced
    /// only to keep the constructor panic-free.
    ///
    /// # Notes
    /// - The function relies on creating a default `LongCall` instance and then populating it with positions.
    /// - Uses the `Options` and `Position` structures to model and manage the long call position.
    /// - Assumes the current time (_via `Utc::now()`) when opening the long call position for tracking purposes.
    #[allow(clippy::too_many_arguments, dead_code)]
    #[inline(never)]
    pub fn new(
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
    ) -> Result<Self, StrategyError> {
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
        strategy.add_position(&long_call)?;

        Ok(strategy)
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
    fn get_option_basic_type(&self) -> HashSet<OptionBasicType<'_>> {
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
    fn get_implied_volatility(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
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
    fn get_quantity(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
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
            Positive::new_decimal(self.long_call.option.calculate_price_black_scholes()?.abs())
                .unwrap_or(Positive::ZERO);
        Ok(())
    }
    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        self.long_call.option.implied_volatility = *volatility;
        self.long_call.premium =
            Positive::new_decimal(self.long_call.option.calculate_price_black_scholes()?.abs())
                .unwrap_or(Positive::ZERO);
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

        // `lower_break_even` measures a distance below the strike and floors at
        // zero. A credit larger than the strike puts the break-even where the
        // underlying cannot trade, and `Positive::ZERO` is this layer's
        // sentinel for "no lower break-even" rather than an abort.
        let per_contract = d_div(
            self.get_net_cost()?,
            self.long_call.option.quantity.to_dec(),
            "LongCall::update_break_even_points",
        )?;
        self.break_even_points.push(
            lower_break_even(self.long_call.option.strike_price, -per_contract)
                .checked_round_to(2)?,
        );

        Ok(())
    }
}

impl Strategies for LongCall {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        Ok(Positive::MAX) // Theoretically unlimited
    }
    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        // Max loss for a long call is the premium paid (at any price ≤ strike).
        Ok(self.get_total_cost()?)
    }
    fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
        let high = self.get_max_profit().unwrap_or(Positive::ZERO);
        let break_even = self.break_even_points.first().ok_or_else(|| {
            StrategyError::empty_collection("LongCall::get_profit_area: no break-even points")
        })?;
        let base = price_gap(self.long_call.option.strike_price, *break_even);
        Ok(Decimal::from_f64(high.to_f64() * base.to_f64() / 200.0).unwrap_or(Decimal::ZERO))
    }
    fn get_profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let max_profit = self.get_max_profit().unwrap_or(Positive::ZERO);
        let max_loss = self.get_max_loss().unwrap_or(Positive::ZERO);
        match (max_profit, max_loss) {
            (value, _) if value == Positive::ZERO => Ok(Decimal::ZERO),
            (_, value) if value == Positive::ZERO => Ok(Decimal::MAX),
            _ => Ok(
                Decimal::from_f64(max_profit.to_f64() / max_loss.to_f64() * 100.0)
                    .unwrap_or(Decimal::ZERO),
            ),
        }
    }
}

impl Profit for LongCall {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError> {
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
        Err(StrategyError::operation_not_supported(
            "get_strategy",
            "LongCall",
        ))
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
        warn!("find_optimal: stub — no optimization performed for LongCall");
    }
}

impl ProbabilityAnalysis for LongCall {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        // Long call is profitable when price rises above break-even
        let break_even = self.break_even_points.first().ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "No break-even points found for long call".to_string(),
            })
        })?;

        let option = &self.long_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let mut profit_range = ProfitLossRange::new(Some(*break_even), None, Positive::ZERO)?;

        profit_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: option.implied_volatility,
                std_dev_adjustment: Positive::ZERO,
            }),
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;

        Ok(vec![profit_range])
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        // Long call has losses when price stays below break-even
        let break_even = self.break_even_points.first().ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "No break-even points found for long call".to_string(),
            })
        })?;

        let option = &self.long_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let mut loss_range = ProfitLossRange::new(None, Some(*break_even), Positive::ZERO)?;

        loss_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: option.implied_volatility,
                std_dev_adjustment: Positive::ZERO,
            }),
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;

        Ok(vec![loss_range])
    }
}

impl Greeks for LongCall {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.long_call.option])
    }
}

impl DeltaNeutrality for LongCall {}

impl PnLCalculator for LongCall {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, PricingError> {
        self.long_call
            .calculate_pnl(market_price, expiration_date, implied_volatility)
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, PricingError> {
        self.long_call.calculate_pnl_at_expiration(underlying_price)
    }

    fn adjustments_pnl(&self, _adjustment: &DeltaAdjustment) -> Result<PnL, PricingError> {
        // Single-leg strategies like LongCall don't typically require delta adjustments
        // as they are directional strategies. Delta adjustments are more relevant for
        // complex multi-leg strategies aiming for delta neutrality.
        Err(PricingError::DeltaAdjustmentNotApplicable {
            strategy: "LongCall",
        })
    }
}

impl Strategable for LongCall {}

#[cfg(test)]
mod tests_get_strategy {
    use super::*;

    #[test]
    fn test_get_strategy_returns_not_supported() {
        use crate::prelude::OperationErrorKind;
        let result = LongCall::get_strategy(&[]);
        match result {
            Err(StrategyError::OperationError(OperationErrorKind::NotSupported {
                operation,
                reason,
            })) => {
                assert_eq!(operation, "get_strategy");
                assert!(
                    reason.contains("LongCall"),
                    "expected reason to contain 'LongCall', got {reason}"
                );
            }
            other => panic!("expected NotSupported error, got {other:?}"),
        }
    }
}
